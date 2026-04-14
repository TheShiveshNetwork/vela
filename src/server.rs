use axum::{
    routing::get,
    Router,
    extract::{Query, Path as AxumPath},
    response::{Json, IntoResponse, Response},
    http::{StatusCode, header, HeaderMap, HeaderValue},
    body::Body,
};
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf};
use anyhow::Result;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncReadExt};
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
        .route("/files/{*path}", get(move |path, headers| serve_file(path, headers, root_clone2.clone())))
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

async fn serve_file(
    AxumPath(path): AxumPath<String>, 
    headers: HeaderMap,
    root: String
) -> Response {
    // Clean the path
    let clean_path = path.trim_start_matches('/');
    let full_path = PathBuf::from(&root).join(clean_path);
    
    println!("=== FILE SERVING ===");
    println!("Requested: {}", path);
    println!("Full path: {:?}", full_path);
    
    // Get file metadata
    let metadata = match tokio::fs::metadata(&full_path).await {
        Ok(metadata) => {
            if !metadata.is_file() {
                return (StatusCode::BAD_REQUEST, "Not a file").into_response();
            }
            metadata
        }
        Err(e) => {
            println!("ERROR: {}", e);
            return (StatusCode::NOT_FOUND, format!("File not found: {}", e)).into_response();
        }
    };
    
    let file_size = metadata.len();
    println!("File size: {} bytes", file_size);
    
    // Determine content type
    let content_type = get_content_type(&full_path);
    println!("Content-Type: {}", content_type);
    
    // Check for Range header (for video streaming)
    let range_header = headers.get(header::RANGE);
    
    if let Some(range) = range_header {
        println!("Range request: {:?}", range);
        return serve_range(full_path, range, file_size, content_type).await;
    }
    
    // Serve entire file
    let file = match File::open(&full_path).await {
        Ok(file) => file,
        Err(e) => {
            println!("ERROR opening file: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response();
        }
    };
    
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    response_headers.insert(header::CONTENT_LENGTH, file_size.to_string().parse().unwrap());
    response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    response_headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
    
    println!("Serving full file\n");
    
    (StatusCode::OK, response_headers, body).into_response()
}

async fn serve_range(
    full_path: PathBuf,
    range_header: &HeaderValue,
    file_size: u64,
    content_type: String,
) -> Response {
    let range_str = match range_header.to_str() {
        Ok(s) => s,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid range header").into_response(),
    };
    
    // Parse range header (format: "bytes=start-end")
    let range_str = range_str.trim_start_matches("bytes=");
    let parts: Vec<&str> = range_str.split('-').collect();
    
    if parts.len() != 2 {
        return (StatusCode::BAD_REQUEST, "Invalid range format").into_response();
    }
    
    let start: u64 = parts[0].parse().unwrap_or(0);
    let end: u64 = if parts[1].is_empty() {
        file_size - 1
    } else {
        parts[1].parse().unwrap_or(file_size - 1).min(file_size - 1)
    };
    
    println!("Range: {}-{} of {}", start, end, file_size);
    
    let mut file = match File::open(&full_path).await {
        Ok(file) => file,
        Err(e) => {
            println!("ERROR opening file: {}", e);
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response();
        }
    };
    
    // Seek to start position
    if let Err(e) = file.seek(std::io::SeekFrom::Start(start)).await {
        println!("ERROR seeking: {}", e);
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seek").into_response();
    }
    
    // Read the range
    let length = (end - start + 1) as usize;
    let mut buffer = vec![0u8; length];
    
    match file.read_exact(&mut buffer).await {
        Ok(_) => {
            let mut response_headers = HeaderMap::new();
            response_headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
            response_headers.insert(header::CONTENT_LENGTH, length.to_string().parse().unwrap());
            response_headers.insert(
                header::CONTENT_RANGE,
                format!("bytes {}-{}/{}", start, end, file_size).parse().unwrap(),
            );
            response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
            
            println!("Serving range successfully\n");
            
            (StatusCode::PARTIAL_CONTENT, response_headers, buffer).into_response()
        }
        Err(e) => {
            println!("ERROR reading file: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
        }
    }
}

fn get_content_type(path: &PathBuf) -> String {
    // Get extension and convert to lowercase for case-insensitive matching
    let ext = path.extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_lowercase());
    
    match ext.as_deref() {
        // Images
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("bmp") => "image/bmp",
        Some("svg") => "image/svg+xml",
        Some("heic") | Some("heif") => "image/heic",
        Some("ico") => "image/x-icon",
        Some("tiff") | Some("tif") => "image/tiff",
        Some("avif") => "image/avif",
        
        // Videos
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("avi") => "video/x-msvideo",
        Some("mkv") => "video/x-matroska",
        Some("flv") => "video/x-flv",
        Some("wmv") => "video/x-ms-wmv",
        Some("m4v") => "video/x-m4v",
        Some("mpg") | Some("mpeg") => "video/mpeg",
        Some("3gp") => "video/3gpp",
        Some("ogv") => "video/ogg",
        
        // Audio
        Some("mp3") => "audio/mpeg",
        Some("wav") => "audio/wav",
        Some("ogg") | Some("oga") => "audio/ogg",
        Some("m4a") => "audio/mp4",
        Some("flac") => "audio/flac",
        Some("aac") => "audio/aac",
        Some("wma") => "audio/x-ms-wma",
        Some("opus") => "audio/opus",
        
        // Documents
        Some("pdf") => "application/pdf",
        Some("txt") => "text/plain; charset=utf-8",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("csv") => "text/csv",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        
        // Archives
        Some("zip") => "application/zip",
        Some("rar") => "application/x-rar-compressed",
        Some("7z") => "application/x-7z-compressed",
        Some("tar") => "application/x-tar",
        Some("gz") => "application/gzip",
        Some("bz2") => "application/x-bzip2",
        
        // All other files - downloadable
        _ => "application/octet-stream",
    }.to_string()
}

async fn list_files(
    Query(params): Query<std::collections::HashMap<String, String>>,
    root: String,
) -> Json<Vec<FileEntry>> {
    let rel_path = params.get("path").cloned().unwrap_or_default();
    let rel_path = rel_path.trim_start_matches('/');
    let full_path = PathBuf::from(&root).join(rel_path);
    
    let mut entries = Vec::new();
    
    // Use tokio's async read_dir
    if let Ok(mut read_dir) = tokio::fs::read_dir(&full_path).await {
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let entry_full_path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            
            // Use symlink_metadata to get accurate file type without following symlinks
            // Then use regular metadata to get actual file size (in case of symlinks)
            if let Ok(symlink_meta) = tokio::fs::symlink_metadata(&entry_full_path).await {
                let is_dir = symlink_meta.is_dir();
                let is_symlink = symlink_meta.is_symlink();
                
                // For symlinks, resolve to get actual size and type
                let (actual_size, actual_is_dir) = if is_symlink {
                    if let Ok(real_meta) = tokio::fs::metadata(&entry_full_path).await {
                        (real_meta.len(), real_meta.is_dir())
                    } else {
                        (0, is_dir)
                    }
                } else {
                    (symlink_meta.len(), is_dir)
                };
                
                // Construct the relative path for this entry
                let entry_path = if rel_path.is_empty() {
                    name.clone()
                } else {
                    format!("{}/{}", rel_path, name)
                };
                
                // Store 0 size for directories, actual size for files
                let display_size = if actual_is_dir { 0 } else { actual_size };
                
                entries.push(FileEntry {
                    name,
                    is_dir: actual_is_dir,
                    path: entry_path,
                    size: display_size,
                });
            }
        }
    }
    
    // Sort: directories first, then alphabetically
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
