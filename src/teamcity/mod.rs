use color_eyre::Result;
use color_eyre::eyre::eyre;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub mod types;
use types::{Build, BuildType, BuildTypes, Builds};

#[derive(Serialize, Deserialize, Clone, Debug)]
struct PersistentCacheEntry<T> {
    data: T,
    timestamp: u64,
    ttl_seconds: u64,
}

impl<T> PersistentCacheEntry<T> {
    fn new(data: T, ttl: Duration) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            data,
            timestamp,
            ttl_seconds: ttl.as_secs(),
        }
    }

    fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        now > self.timestamp + self.ttl_seconds
    }
}

#[derive(Serialize, Deserialize, Default, Debug)]
struct PersistentCache {
    entries: HashMap<String, PersistentCacheEntry<Vec<BuildType>>>,
}

#[derive(Clone)]
pub struct TeamCityClient {
    base_url: String,
    client: reqwest::Client,
    cache_file: PathBuf,
    default_ttl: Duration,
}

impl TeamCityClient {
    pub fn new(base_url: String, token: String) -> Self {
        let mut auth_header = HeaderMap::new();
        auth_header.insert("Authorization", format!("Bearer {token}").parse().unwrap());

        let client = reqwest::Client::builder()
            .default_headers(auth_header)
            .build()
            .unwrap();

        let cache_file = Self::get_cache_file_path();

        Self {
            base_url,
            client,
            cache_file,
            default_ttl: Duration::from_secs(3600),
        }
    }

    fn get_cache_file_path() -> PathBuf {
        if let Some(cache_dir) = dirs::cache_dir() {
            let app_cache_dir = cache_dir.join("teamcity-client");
            std::fs::create_dir_all(&app_cache_dir).ok();
            app_cache_dir.join("build_configs_cache.json")
        } else {
            // Fallback to current directory
            // TODO:write better fallback
            PathBuf::from("teamcity_cache.json")
        }
    }

    async fn load_cache(&self) -> PersistentCache {
        println!("Loading cache from {}", self.cache_file.display());
        match async_fs::read_to_string(&self.cache_file).await {
            Ok(content) => match serde_json::from_str::<PersistentCache>(&content) {
                Ok(cache) => {
                    let mut cleaned_cache = PersistentCache::default();
                    for (key, entry) in cache.entries {
                        if !entry.is_expired() {
                            cleaned_cache.entries.insert(key, entry);
                        }
                    }
                    cleaned_cache
                }
                Err(_) => PersistentCache::default(),
            },
            Err(_) => PersistentCache::default(),
        }
    }

    async fn save_cache(&self, cache: &PersistentCache) -> Result<()> {
        let content = serde_json::to_string_pretty(cache)?;

        if let Some(parent) = self.cache_file.parent() {
            async_fs::create_dir_all(parent).await?;
        }

        async_fs::write(&self.cache_file, content).await?;
        Ok(())
    }

    pub async fn clear_cache(&self) -> Result<()> {
        if self.cache_file.exists() {
            async_fs::remove_file(&self.cache_file).await?;
        }
        Ok(())
    }

    pub async fn get_cache_info(&self) -> (usize, u64) {
        let cache = self.load_cache().await;
        let total_entries = cache.entries.len();
        let cache_size = if self.cache_file.exists() {
            async_fs::metadata(&self.cache_file)
                .await
                .map(|m| m.len())
                .unwrap_or(0)
        } else {
            0
        };
        (total_entries, cache_size)
    }

    pub async fn get_build_configurations_by_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<BuildType>> {
        let cache_key = format!("project_{}", project_id);
        let mut cache = self.load_cache().await;

        if let Some(entry) = cache.entries.get(&cache_key) {
            if !entry.is_expired() {
                println!(
                    "Using cached build configurations for project {}",
                    project_id
                );
                return Ok(entry.data.clone());
            }
        }

        let result = self
            .fetch_build_configurations_by_project(project_id)
            .await?;

        cache.entries.insert(
            cache_key,
            PersistentCacheEntry::new(result.clone(), self.default_ttl),
        );

        if let Err(e) = self.save_cache(&cache).await {
            eprintln!("Warning: Failed to save cache: {}", e);
        }

        Ok(result)
    }

    pub async fn get_build_configurations_by_projects(
        &self,
        project_ids: &Vec<String>,
    ) -> Result<Vec<BuildType>> {
        if project_ids.is_empty() {
            return Err(eyre!("You need to specify at least one project ID"));
        }

        let mut all_build_types = Vec::new();

        for project_id in project_ids {
            match self.get_build_configurations_by_project(project_id).await {
                Ok(mut build_types) => all_build_types.append(&mut build_types),
                Err(e) => eprintln!(
                    "Error fetching build types for project {}: {}",
                    project_id, e
                ),
            }
        }

        Ok(all_build_types)
    }

    async fn fetch_build_configurations_by_project(
        &self,
        project_id: &str,
    ) -> Result<Vec<BuildType>> {
        let url = format!("{}/app/rest/buildTypes", self.base_url);
        let fields = "count,href,buildType(id,name,type,description,projectName,projectId,href,links,webUrl)";

        let request = self
            .client
            .get(&url)
            .query(&[
                ("locator", format!("affectedProject:(id:{})", project_id)),
                ("fields", fields.to_string()),
            ])
            .header("Accept", "application/json");

        let response = request.send().await?;

        if !response.status().is_success() {
            return Err(eyre!("Request failed with status: {}", response.status()).into());
        }

        let build_types: BuildTypes = response.json().await?;
        Ok(build_types.build_type)
    }

    pub async fn get_build_configuration_details(&self, build_type_id: &str) -> Result<BuildType> {
        let url = format!("{}/app/rest/buildTypes/id:{}", self.base_url, build_type_id);

        let response = self
            .client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Request failed with status: {}", response.status()).into());
        }

        let build_type: BuildType = response.json().await?;
        Ok(build_type)
    }

    pub async fn get_builds_by_project(&self, project_id: &str) -> Result<Vec<Build>> {
        let url = format!("{}/app/rest/builds", self.base_url);

        let teamcity_build_fields = "count,build(id,number,branchName,statusText,status,state,webUrl,buildTypeId,startDate,finishDate,changes(change(comment,username)))";
        let default_build_count = "100";

        let params = [
            ("locator", format!("buildType:{}", project_id)),
            ("count", default_build_count.to_string()),
            ("fields", teamcity_build_fields.to_string()),
        ];

        let response = self
            .client
            .get(&url)
            .query(&params)
            .header("Accept", "application/json")
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Request failed with status: {}", response.status()));
        }

        let builds: Builds = response.json().await?;
        Ok(builds.build)
    }

    // TODO: test if downloading and unpacking zip archive will be more efficient
    pub async fn get_build_log_text(&self, build_id: &i64) -> Result<String> {
        let url = format!("{}/downloadBuildLog.html", self.base_url);
        let response = self
            .client
            .get(&url)
            .query(&[
                ("buildId", build_id.to_string()),
                ("plain", "true".to_string()),
            ])
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(eyre!("Request failed with status: {}", response.status()).into());
        }

        let text = response.text().await?;
        Ok(text)
    }

    pub async fn download_build_log_to<P: AsRef<std::path::Path>>(
        &self,
        build_id: &i64,
        path: P,
    ) -> Result<()> {
        let text = self.get_build_log_text(build_id).await?;
        async_fs::write(path.as_ref(), text).await?;
        Ok(())
    }
}
