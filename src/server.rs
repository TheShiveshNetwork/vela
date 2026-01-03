use axum::{
    routing::get,
    Router,
    extract::{Query, Path as AxumPath},
    response::{Json, IntoResponse, Response},
    http::{StatusCode, header, HeaderMap},
    body::Body,
};
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf};
use anyhow::Result;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Serialize)]
struct FileEntry {
    name: String,
    is_dir: bool,
    path: String,
    size: u64,
}

pub async fn start(root: String) -> Result<()> {
    println!("Serving: {}", root);
    
    let root_clone = root.clone();
    let root_clone2 = root.clone();
    
    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/list", get(move |q| list_files(q, root_clone.clone())))
        .route("/files/{*path}", get(move |path| serve_file(path, root_clone2.clone())))
        .route("/app.js", get(serve_js))
        .route("/static/{*path}", get(serve_static));
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    print_access_urls();
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn serve_index() -> Response {
    match tokio::fs::read_to_string("src/static/index.html").await {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
            content
        ).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "index.html not found"
        ).into_response(),
    }
}

async fn serve_js() -> Response {
    match tokio::fs::read_to_string("src/static/app.js").await {
        Ok(content) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/javascript; charset=utf-8")],
            content
        ).into_response(),
        Err(_) => (
            StatusCode::NOT_FOUND,
            "app.js not found"
        ).into_response(),
    }
}

async fn serve_static(AxumPath(path): AxumPath<String>) -> Response {
    let file_path = format!("src/static/{}", path);
    match tokio::fs::read(&file_path).await {
        Ok(content) => {
            let content_type = match path.split('.').last() {
                Some("css") => "text/css",
                Some("js") => "application/javascript",
                Some("html") => "text/html",
                _ => "application/octet-stream",
            };
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, content_type)],
                content
            ).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

async fn serve_file(AxumPath(path): AxumPath<String>, root: String) -> Response {
    // Clean the path - remove leading slash if present
    let clean_path = path.trim_start_matches('/');
    let full_path = PathBuf::from(&root).join(clean_path);
    
    println!("=== FILE SERVING DEBUG ===");
    println!("Root: {}", root);
    println!("Requested path: {}", path);
    println!("Clean path: {}", clean_path);
    println!("Full path: {:?}", full_path);
    println!("Path exists: {}", full_path.exists());
    
    // Check if file exists and is actually a file
    let metadata = match tokio::fs::metadata(&full_path).await {
        Ok(metadata) => {
            println!("File size: {} bytes", metadata.len());
            println!("Is file: {}", metadata.is_file());
            println!("Is dir: {}", metadata.is_dir());
            println!("Is symlink: {}", metadata.is_symlink());
            
            if !metadata.is_file() {
                println!("ERROR: Path is not a file!");
                return (StatusCode::BAD_REQUEST, "Not a file").into_response();
            }
            metadata
        }
        Err(e) => {
            println!("ERROR: File not found or cannot access: {}", e);
            return (StatusCode::NOT_FOUND, format!("File not found: {}", e)).into_response();
        }
    };
    
    // Open the file
    let file = match File::open(&full_path).await {
        Ok(file) => {
            println!("Successfully opened file");
            file
        },
        Err(e) => {
            println!("ERROR: Failed to open file: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to open file: {}", e)).into_response();
        }
    };
    
    // Determine content type from extension
    let content_type = match full_path.extension().and_then(|s| s.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("heic") | Some("heif") => "image/heic",
        Some("ico") => "image/x-icon",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("avi") => "video/x-msvideo",
        Some("mkv") => "video/x-matroska",
        Some("flv") => "video/x-flv",
        Some("wmv") => "video/x-ms-wmv",
        Some("m4v") => "video/x-m4v",
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("flac") => "audio/flac",
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("zip") => "application/zip",
        Some("rar") => "application/x-rar-compressed",
        Some("7z") => "application/x-7z-compressed",
        Some("tar") => "application/x-tar",
        Some("gz") => "application/gzip",
        Some("doc") | Some("docx") => "application/msword",
        Some("xls") | Some("xlsx") => "application/vnd.ms-excel",
        Some("ppt") | Some("pptx") => "application/vnd.ms-powerpoint",
        _ => "application/octet-stream",
    };
    
    println!("Content-Type: {}", content_type);
    
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    
    let mut headers = HeaderMap::new();
    headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    headers.insert(header::CONTENT_LENGTH, metadata.len().to_string().parse().unwrap());
    
    // Add cache control for better performance
    headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
    
    println!("=== SERVING FILE SUCCESSFULLY ===\n");
    
    (StatusCode::OK, headers, body).into_response()
}

async fn list_files(
    Query(params): Query<std::collections::HashMap<String, String>>,
    root: String,
) -> Json<Vec<FileEntry>> {
    let rel_path = params.get("path").cloned().unwrap_or_default();
    let full_path = PathBuf::from(&root).join(rel_path.trim_start_matches('/'));
    
    let mut entries = Vec::new();
    
    if let Ok(read_dir) = std::fs::read_dir(&full_path) {
        for entry in read_dir.flatten() {
            if let Ok(meta) = entry.metadata() {
                let name = entry.file_name().to_string_lossy().to_string();
                let entry_path = if rel_path.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", rel_path, name)
                };
                
                entries.push(FileEntry {
                    name,
                    is_dir: meta.is_dir(),
                    path: entry_path,
                    size: meta.len(),
                });
            }
        }
    }
    
    // Sort: directories first, then files, alphabetically
    entries.sort_by(|a, b| {
        match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        }
    });
    
    Json(entries)
}

fn print_access_urls() {
    use std::process::Command;
    
    println!("\n📡 Vela is running!");
    println!("Local:  http://localhost:9000");
    
    if let Ok(output) = Command::new("ip").arg("route").output() {
        let text = String::from_utf8_lossy(&output.stdout);
        for line in text.lines() {
            if line.starts_with("default") {
                if let Some(ip) = line.split_whitespace().nth(8) {
                    println!("LAN:    http://{}:9000", ip);
                }
            }
        }
    }
    
    println!();
}
