use axum::{
    routing::{get, post},
    Router,
    extract::{Query, State, Path as AxumPath, Multipart},
    response::{Json, IntoResponse, Response},
    http::{StatusCode, header},
    body::Body,
};
use tower_cookies::{Cookie, CookieManagerLayer, Cookies};
use serde::Serialize;
use std::{net::SocketAddr, path::PathBuf, sync::Arc};
use anyhow::Result;
use tokio::fs::File;
use tokio::io::{AsyncSeekExt, AsyncReadExt, AsyncWriteExt};
use tokio_util::io::ReaderStream;

#[derive(Serialize)]
struct FileEntry {
    name: String,
    is_dir: bool,
    path: String,
    size: u64,
}

#[derive(Clone)]
struct ServerState {
    root: String,
    read_only: bool,
    token: Option<String>,
}

pub async fn start(root: String, read_only: bool, no_auth: bool) -> Result<()> {
    let token = if no_auth {
        None
    } else {
        Some(uuid::Uuid::new_v4().to_string())
    };

    println!("Serving ({}): {}", if read_only { "RO" } else { "RW" }, root);
    if let Some(ref t) = token {
        println!("🔐 Authentication enabled!");
        println!("🔑 ACCESS TOKEN: {}", t);
        println!("   The web UI will handle this via secure HttpOnly cookies.");
    } else {
        println!("⚠️ Authentication DISABLED!");
    }
    
    let state = Arc::new(ServerState {
        root: root.clone(),
        read_only,
        token,
    });

    let app = Router::new()
        .route("/", get(serve_index))
        .route("/api/list", get(list_files))
        .route("/api/upload", post(upload_file))
        .route("/api/auth", post(authenticate))
        .route("/files/{*path}", get(serve_file))
        .route("/app.js", get(serve_js))
        .route("/static/{*path}", get(serve_static))
        .layer(CookieManagerLayer::new())
        .with_state(state);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 9000));
    print_access_urls();
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

fn check_auth(state: &ServerState, cookies: &Cookies) -> bool {
    if state.token.is_none() {
        return true;
    }
    
    if let Some(cookie) = cookies.get("vela_token") {
        if let Some(ref t) = state.token {
            return cookie.value() == t;
        }
    }
    false
}

async fn authenticate(
    State(state): State<Arc<ServerState>>,
    cookies: Cookies,
    body: String,
) -> Response {
    if let Some(ref t) = state.token {
        if body.trim() == t {
            let mut cookie = Cookie::new("vela_token", t.clone());
            cookie.set_path("/");
            cookie.set_http_only(true);
            cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
            cookie.set_max_age(time::Duration::days(7));
            cookies.add(cookie);
            return (StatusCode::OK, "Authenticated").into_response();
        }
    }
    StatusCode::UNAUTHORIZED.into_response()
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
    State(state): State<Arc<ServerState>>,
    cookies: Cookies,
    AxumPath(path): AxumPath<String>, 
    headers: HeaderMap,
) -> Response {
    if !check_auth(&state, &cookies) {
        return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
    }

    let clean_path = path.trim_start_matches('/');
    let full_path = PathBuf::from(&state.root).join(clean_path);
    
    if !full_path.starts_with(&state.root) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    let metadata = match tokio::fs::metadata(&full_path).await {
        Ok(metadata) => {
            if !metadata.is_file() {
                return (StatusCode::BAD_REQUEST, "Not a file").into_response();
            }
            metadata
        }
        Err(e) => {
            return (StatusCode::NOT_FOUND, format!("File not found: {}", e)).into_response();
        }
    };
    
    let file_size = metadata.len();
    let content_type = get_content_type(&full_path);
    let range_header = headers.get(header::RANGE);
    
    if let Some(range) = range_header {
        return serve_range(full_path, range, file_size, content_type).await;
    }
    
    let file = match File::open(&full_path).await {
        Ok(file) => file,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response(),
    };
    
    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);
    
    let mut response_headers = HeaderMap::new();
    response_headers.insert(header::CONTENT_TYPE, content_type.parse().unwrap());
    response_headers.insert(header::CONTENT_LENGTH, file_size.to_string().parse().unwrap());
    response_headers.insert(header::ACCEPT_RANGES, "bytes".parse().unwrap());
    response_headers.insert(header::CACHE_CONTROL, "public, max-age=3600".parse().unwrap());
    
    (StatusCode::OK, response_headers, body).into_response()
}

use axum::http::{HeaderMap, HeaderValue};

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
    
    let mut file = match File::open(&full_path).await {
        Ok(file) => file,
        Err(_) => return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to open file").into_response(),
    };
    
    if let Err(_) = file.seek(std::io::SeekFrom::Start(start)).await {
        return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to seek").into_response();
    }
    
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
            (StatusCode::PARTIAL_CONTENT, response_headers, buffer).into_response()
        }
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to read file").into_response()
    }
}

async fn upload_file(
    State(state): State<Arc<ServerState>>,
    cookies: Cookies,
    mut multipart: Multipart,
) -> Response {
    if !check_auth(&state, &cookies) {
        return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
    }

    if state.read_only {
        return (StatusCode::FORBIDDEN, "Server is in read-only mode").into_response();
    }

    let mut target_dir = PathBuf::from(&state.root);
    let mut filename = String::new();

    println!("=== UPLOAD REQUEST (STREAMING) ===");
    
    // We first need to find the filename and target path from the multipart fields
    // However, multipart fields are sequential. To be safe and efficient, we 
    // stream the file field last or handle it when it appears.
    
    while let Ok(Some(mut field)) = multipart.next_field().await {
        let name = field.name().unwrap_or_default().to_string();
        
        if name == "path" {
            let path_val = field.text().await.unwrap_or_default();
            target_dir = target_dir.join(path_val.trim_start_matches('/'));
        } else if name == "file" {
            filename = field.file_name().unwrap_or_default().to_string();
            
            if filename.is_empty() {
                filename = format!("upload_{}.bin", uuid::Uuid::new_v4());
            }

            // Safety: Path Traversal Check before creating the file
            if !target_dir.starts_with(&state.root) {
                println!("ERROR: Invalid path traversal blocked during stream.");
                return (StatusCode::FORBIDDEN, "Invalid path").into_response();
            }

            let dest_path = target_dir.join(&filename);
            println!("Streaming data to: {:?}", dest_path);

            let mut file = match File::create(&dest_path).await {
                Ok(file) => file,
                Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to create file: {}", e)).into_response(),
            };

            let mut total_bytes = 0;
            while let Ok(Some(chunk)) = field.chunk().await {
                if let Err(e) = file.write_all(&chunk).await {
                    println!("ERROR writing chunk: {}", e);
                    return (StatusCode::INTERNAL_SERVER_ERROR, "Write error during streaming").into_response();
                }
                total_bytes += chunk.len();
            }
            
            println!("Stream complete. Received {} bytes.", total_bytes);
        }
    }

    if filename.is_empty() {
        return (StatusCode::BAD_REQUEST, "No file provided").into_response();
    }

    StatusCode::OK.into_response()
}

fn get_content_type(path: &PathBuf) -> String {
    let ext = path.extension().and_then(|s| s.to_str()).map(|s| s.to_lowercase());
    match ext.as_deref() {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("webp") => "image/webp",
        Some("pdf") => "application/pdf",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        _ => "application/octet-stream",
    }.to_string()
}

async fn list_files(
    State(state): State<Arc<ServerState>>,
    cookies: Cookies,
    Query(params): Query<std::collections::HashMap<String, String>>,
) -> Response {
    if !check_auth(&state, &cookies) {
        return (StatusCode::UNAUTHORIZED, "Authentication required").into_response();
    }

    let rel_path = params.get("path").cloned().unwrap_or_default();
    let rel_path = rel_path.trim_start_matches('/');
    let full_path = PathBuf::from(&state.root).join(rel_path);
    
    if !full_path.starts_with(&state.root) {
        return (StatusCode::FORBIDDEN, "Access denied").into_response();
    }

    let mut entries = Vec::new();
    if let Ok(mut read_dir) = tokio::fs::read_dir(&full_path).await {
        while let Ok(Some(entry)) = read_dir.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if let Ok(meta) = entry.metadata().await {
                let entry_path = if rel_path.is_empty() { name.clone() } else { format!("{}/{}", rel_path, name) };
                entries.push(FileEntry {
                    name,
                    is_dir: meta.is_dir(),
                    path: entry_path,
                    size: if meta.is_dir() { 0 } else { meta.len() },
                });
            }
        }
    }
    
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    
    Json(entries).into_response()
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
