//! File upload to Google Drive.

use crate::auth::AuthProvider;
use std::path::Path;
use tor_capture_core::GDriveError;

/// Upload result.
#[derive(Debug)]
pub struct UploadResult {
    pub file_id: String,
    pub name: String,
    pub mime_type: String,
    pub size: u64,
    pub web_view_link: Option<String>,
}

/// Upload a file to Google Drive.
pub async fn upload_file(
    auth: &dyn AuthProvider,
    file_path: &Path,
    folder_id: Option<&str>,
    mime_type: &str,
) -> Result<UploadResult, GDriveError> {
    let token = auth.get_token().await?;
    let file_name = file_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("untitled");

    // Read file content
    let content = tokio::fs::read(file_path)
        .await
        .map_err(|e| GDriveError::UploadFailed(format!("Read error: {}", e)))?;

    let file_size = content.len() as u64;

    // Build metadata
    let mut metadata = serde_json::json!({
        "name": file_name,
        "mimeType": mime_type,
    });

    if let Some(parent) = folder_id {
        metadata["parents"] = serde_json::json!([parent]);
    }

    // Use multipart upload for files under 5MB
    if file_size < 5 * 1024 * 1024 {
        upload_multipart(&token.token, &metadata, &content, mime_type).await
    } else {
        upload_resumable(&token.token, &metadata, &content, mime_type).await
    }
}

/// Upload using multipart (for small files).
async fn upload_multipart(
    token: &str,
    metadata: &serde_json::Value,
    content: &[u8],
    mime_type: &str,
) -> Result<UploadResult, GDriveError> {
    let client = reqwest::Client::new();

    let boundary = "----WebKitFormBoundary7MA4YWxkTrZu0gW";
    let metadata_json = serde_json::to_string(metadata)
        .map_err(|e| GDriveError::UploadFailed(e.to_string()))?;

    // Build multipart body
    let mut body = Vec::new();
    body.extend(format!("--{}\r\n", boundary).as_bytes());
    body.extend(b"Content-Type: application/json; charset=UTF-8\r\n\r\n");
    body.extend(metadata_json.as_bytes());
    body.extend(format!("\r\n--{}\r\n", boundary).as_bytes());
    body.extend(format!("Content-Type: {}\r\n\r\n", mime_type).as_bytes());
    body.extend(content);
    body.extend(format!("\r\n--{}--", boundary).as_bytes());

    let response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=multipart&fields=id,name,mimeType,size,webViewLink")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", format!("multipart/related; boundary={}", boundary))
        .body(body)
        .send()
        .await
        .map_err(|e| GDriveError::UploadFailed(e.to_string()))?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(GDriveError::UploadFailed(error_text));
    }

    let json: serde_json::Value = response
        .json()
        .await
        .map_err(|e| GDriveError::UploadFailed(e.to_string()))?;

    Ok(UploadResult {
        file_id: json["id"].as_str().unwrap_or_default().to_string(),
        name: json["name"].as_str().unwrap_or_default().to_string(),
        mime_type: json["mimeType"].as_str().unwrap_or_default().to_string(),
        size: json["size"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        web_view_link: json["webViewLink"].as_str().map(|s| s.to_string()),
    })
}

/// Upload using resumable upload (for large files).
async fn upload_resumable(
    token: &str,
    metadata: &serde_json::Value,
    content: &[u8],
    mime_type: &str,
) -> Result<UploadResult, GDriveError> {
    let client = reqwest::Client::new();

    // Step 1: Initiate resumable upload
    let init_response = client
        .post("https://www.googleapis.com/upload/drive/v3/files?uploadType=resumable")
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json; charset=UTF-8")
        .header("X-Upload-Content-Type", mime_type)
        .header("X-Upload-Content-Length", content.len().to_string())
        .json(metadata)
        .send()
        .await
        .map_err(|e| GDriveError::UploadFailed(e.to_string()))?;

    if !init_response.status().is_success() {
        let error_text = init_response.text().await.unwrap_or_default();
        return Err(GDriveError::UploadFailed(error_text));
    }

    let upload_uri = init_response
        .headers()
        .get("location")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| GDriveError::UploadFailed("No upload URI".to_string()))?
        .to_string();

    // Step 2: Upload content
    let upload_response = client
        .put(&upload_uri)
        .header("Content-Length", content.len().to_string())
        .header("Content-Type", mime_type)
        .body(content.to_vec())
        .send()
        .await
        .map_err(|e| GDriveError::UploadFailed(e.to_string()))?;

    if !upload_response.status().is_success() {
        let error_text = upload_response.text().await.unwrap_or_default();
        return Err(GDriveError::UploadFailed(error_text));
    }

    let json: serde_json::Value = upload_response
        .json()
        .await
        .map_err(|e| GDriveError::UploadFailed(e.to_string()))?;

    Ok(UploadResult {
        file_id: json["id"].as_str().unwrap_or_default().to_string(),
        name: json["name"].as_str().unwrap_or_default().to_string(),
        mime_type: json["mimeType"].as_str().unwrap_or_default().to_string(),
        size: json["size"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0),
        web_view_link: json["webViewLink"].as_str().map(|s| s.to_string()),
    })
}

/// Upload bytes directly (not from file).
pub async fn upload_bytes(
    auth: &dyn AuthProvider,
    bytes: &[u8],
    file_name: &str,
    folder_id: Option<&str>,
    mime_type: &str,
) -> Result<UploadResult, GDriveError> {
    let token = auth.get_token().await?;

    let mut metadata = serde_json::json!({
        "name": file_name,
        "mimeType": mime_type,
    });

    if let Some(parent) = folder_id {
        metadata["parents"] = serde_json::json!([parent]);
    }

    if bytes.len() < 5 * 1024 * 1024 {
        upload_multipart(&token.token, &metadata, bytes, mime_type).await
    } else {
        upload_resumable(&token.token, &metadata, bytes, mime_type).await
    }
}
