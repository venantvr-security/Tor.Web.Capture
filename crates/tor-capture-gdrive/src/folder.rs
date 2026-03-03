//! Google Drive folder management.

use crate::auth::AuthProvider;
use chrono::Utc;
use tor_capture_core::GDriveError;

/// Folder information.
#[derive(Debug)]
pub struct FolderInfo {
    pub id: String,
    pub name: String,
}

/// Create a folder in Google Drive.
pub async fn create_folder(
    auth: &dyn AuthProvider,
    name: &str,
    parent_id: Option<&str>,
) -> Result<FolderInfo, GDriveError> {
    let token = auth.get_token().await?;
    let client = reqwest::Client::new();

    let mut metadata = serde_json::json!({
        "name": name,
        "mimeType": "application/vnd.google-apps.folder",
    });

    if let Some(parent) = parent_id {
        metadata["parents"] = serde_json::json!([parent]);
    }

    let response = client
        .post("https://www.googleapis.com/drive/v3/files")
        .header("Authorization", format!("Bearer {}", token.token))
        .header("Content-Type", "application/json")
        .json(&metadata)
        .send()
        .await
        .map_err(|e| GDriveError::FolderCreationFailed(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(GDriveError::FolderCreationFailed(error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| GDriveError::FolderCreationFailed(e.to_string()))?;

    Ok(FolderInfo {
        id: json["id"].as_str().unwrap_or_default().to_string(),
        name: json["name"].as_str().unwrap_or_default().to_string(),
    })
}

/// Find a folder by name.
pub async fn find_folder(
    auth: &dyn AuthProvider,
    name: &str,
    parent_id: Option<&str>,
) -> Result<Option<FolderInfo>, GDriveError> {
    let token = auth.get_token().await?;
    let client = reqwest::Client::new();

    let mut query = format!(
        "name = '{}' and mimeType = 'application/vnd.google-apps.folder' and trashed = false",
        name.replace('\'', "\\'")
    );

    if let Some(parent) = parent_id {
        query.push_str(&format!(" and '{}' in parents", parent));
    }

    let url = format!(
        "https://www.googleapis.com/drive/v3/files?q={}&fields=files(id,name)",
        urlencoding::encode(&query)
    );

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", token.token))
        .send()
        .await
        .map_err(|e| GDriveError::ApiError(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(GDriveError::ApiError(error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| GDriveError::ApiError(e.to_string()))?;

    let files = json["files"].as_array();
    if let Some(files) = files {
        if let Some(first) = files.first() {
            return Ok(Some(FolderInfo {
                id: first["id"].as_str().unwrap_or_default().to_string(),
                name: first["name"].as_str().unwrap_or_default().to_string(),
            }));
        }
    }

    Ok(None)
}

/// Get or create a folder.
pub async fn get_or_create_folder(
    auth: &dyn AuthProvider,
    name: &str,
    parent_id: Option<&str>,
) -> Result<FolderInfo, GDriveError> {
    if let Some(folder) = find_folder(auth, name, parent_id).await? {
        return Ok(folder);
    }

    create_folder(auth, name, parent_id).await
}

/// Create date-based folder structure: /YYYY/MM/DD
pub async fn create_date_folder(
    auth: &dyn AuthProvider,
    base_folder_id: &str,
) -> Result<FolderInfo, GDriveError> {
    let now = Utc::now();

    // Create year folder
    let year_folder = get_or_create_folder(
        auth,
        &now.format("%Y").to_string(),
        Some(base_folder_id),
    )
    .await?;

    // Create month folder
    let month_folder = get_or_create_folder(
        auth,
        &now.format("%m").to_string(),
        Some(&year_folder.id),
    )
    .await?;

    // Create day folder
    let day_folder = get_or_create_folder(
        auth,
        &now.format("%d").to_string(),
        Some(&month_folder.id),
    )
    .await?;

    Ok(day_folder)
}

mod urlencoding {
    pub fn encode(s: &str) -> String {
        url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
    }
}
