use crate::state::{CategoriesResult, CategoryCount, Package, PackageSearchResult, PlatformStats};
use reqwest::multipart;
use reqwest::Client;

const BASE_URL: &str = "https://skillvault.md";

pub struct ApiClient {
    client: Client,
    token: Option<String>,
}

impl ApiClient {
    pub fn new(token: Option<String>) -> Self {
        Self {
            client: Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap_or_else(|_| Client::new()),
            token,
        }
    }

    pub async fn search_packages(
        &self,
        query: &str,
        category: Option<&str>,
        sort: Option<&str>,
        page: u32,
        limit: u32,
        compat: Option<&str>,
    ) -> Result<PackageSearchResult, String> {
        let mut url = format!(
            "{}/api/packages?q={}&page={}&limit={}",
            BASE_URL,
            urlencoded(query),
            page,
            limit,
        );

        if let Some(cat) = category {
            url.push_str(&format!("&category={}", urlencoded(cat)));
        }
        if let Some(s) = sort {
            url.push_str(&format!("&sort={}", urlencoded(s)));
        }
        if let Some(c) = compat {
            url.push_str(&format!("&compat={}", urlencoded(c)));
        }

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        resp.json::<PackageSearchResult>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn get_package(&self, author: &str, name: &str) -> Result<Package, String> {
        let url = format!("{}/api/packages/{}/{}", BASE_URL, author, name);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        resp.json::<Package>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn get_trending(&self) -> Result<Vec<Package>, String> {
        let url = format!("{}/api/trending", BASE_URL);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let result = resp
            .json::<PackageSearchResult>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(result.packages)
    }

    pub async fn get_categories(&self) -> Result<Vec<CategoryCount>, String> {
        let url = format!("{}/api/categories", BASE_URL);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        let result = resp
            .json::<CategoriesResult>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))?;

        Ok(result.categories)
    }

    pub async fn get_stats(&self) -> Result<PlatformStats, String> {
        let url = format!("{}/api/stats", BASE_URL);

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Request failed: {}", e))?;

        resp.json::<PlatformStats>()
            .await
            .map_err(|e| format!("Failed to parse response: {}", e))
    }

    pub async fn create_package(
        &self,
        name: &str,
        display_name: &str,
        tagline: &str,
        category: &str,
    ) -> Result<(), String> {
        let url = format!("{}/api/packages", BASE_URL);

        let token = self
            .token
            .as_ref()
            .ok_or("Not authenticated — add your API token in Settings")?;

        let body = serde_json::json!({
            "name": name,
            "display_name": display_name,
            "tagline": tagline,
            "category": category,
        });

        let resp = self
            .client
            .post(&url)
            .header("Authorization", format!("Bearer {}", token))
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("Create package request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Create package failed ({}): {}", status, text));
        }

        Ok(())
    }

    pub async fn upload_version(
        &self,
        author: &str,
        name: &str,
        version: &str,
        zip_bytes: Vec<u8>,
    ) -> Result<(), String> {
        let url = format!("{}/api/packages/{}/{}/upload", BASE_URL, author, name);

        let token = self
            .token
            .as_ref()
            .ok_or("Not authenticated — add your API token in Settings")?;

        let file_part = multipart::Part::bytes(zip_bytes)
            .file_name(format!("{}-{}.zip", name, version))
            .mime_str("application/zip")
            .map_err(|e| format!("Failed to create multipart: {}", e))?;

        let form = multipart::Form::new()
            .text("version", version.to_string())
            .part("file", file_part);

        let resp = self
            .client
            .put(&url)
            .header("Authorization", format!("Bearer {}", token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Upload request failed: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(format!("Upload failed ({}): {}", status, text));
        }

        Ok(())
    }

    pub async fn download_package(&self, author: &str, name: &str) -> Result<Vec<u8>, String> {
        let url = format!("{}/api/packages/{}/{}/download", BASE_URL, author, name);

        let mut req = self.client.get(&url);
        if let Some(token) = &self.token {
            req = req.header("Authorization", format!("Bearer {}", token));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| format!("Download failed: {}", e))?;

        if !resp.status().is_success() {
            return Err(format!("Download failed with status: {}", resp.status()));
        }

        resp.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| format!("Failed to read download bytes: {}", e))
    }
}

pub(crate) fn urlencoded(s: &str) -> String {
    let mut result = String::with_capacity(s.len() * 3);
    for byte in s.as_bytes() {
        let c = *byte as char;
        match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                result.push(c);
            }
            _ => {
                result.push_str(&format!("%{:02X}", byte));
            }
        }
    }
    result
}
