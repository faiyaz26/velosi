use std::process::Command;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

const PROXY_PORT: u16 = 62828;

// List of manually blocked domains
const BLOCKED_DOMAINS: &[&str] = &[
    "facebook.com",
    "www.facebook.com",
    "fb.com",
    "www.fb.com",
    "instagram.com",
    "www.instagram.com",
];

#[derive(Clone)]
pub struct LocalProxyBlocker {
    app_handle: Option<AppHandle>,
    proxy_logs: Arc<Mutex<Vec<String>>>,
}

impl LocalProxyBlocker {
    pub fn new() -> Self {
        Self {
            app_handle: None,
            proxy_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_app_handle(app_handle: AppHandle) -> Self {
        Self {
            app_handle: Some(app_handle),
            proxy_logs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn start_proxy_server(&self) -> Result<(), String> {
        // Try to bind to the fixed port
        let listener = TcpListener::bind(format!("127.0.0.1:{}", PROXY_PORT))
            .await
            .map_err(|e| format!("Failed to bind to port {}: {}", PROXY_PORT, e))?;

        println!("✅ Proxy server started on port: {}", PROXY_PORT);
        Self::log_event(
            &self.proxy_logs,
            "START",
            "proxy",
            "Proxy server started",
            &self.app_handle,
        );

        let app_handle = self.app_handle.clone();
        let proxy_logs = self.proxy_logs.clone();

        tokio::spawn(async move {
            while let Ok((stream, addr)) = listener.accept().await {
                println!("📡 New connection from: {}", addr);
                Self::log_event(
                    &proxy_logs,
                    "CONNECT",
                    "client",
                    &format!("New connection from {}", addr),
                    &app_handle,
                );

                let app_handle = app_handle.clone();
                let proxy_logs = proxy_logs.clone();

                tokio::spawn(async move {
                    if let Err(e) = Self::handle_client(stream, app_handle, proxy_logs).await {
                        eprintln!("Error handling client: {}", e);
                    }
                });
            }
        });

        Ok(())
    }

    async fn handle_client(
        mut stream: TcpStream,
        app_handle: Option<AppHandle>,
        proxy_logs: Arc<Mutex<Vec<String>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut buffer = [0; 4096];

        // Read request with timeout
        let read_timeout = Duration::from_secs(5);
        let n = match tokio::time::timeout(read_timeout, stream.read(&mut buffer)).await {
            Ok(Ok(bytes_read)) => bytes_read,
            Ok(Err(e)) => {
                let error_msg = format!("Error reading request: {}", e);
                println!("❌ {}", error_msg);
                Self::log_event(&proxy_logs, "ERROR", "client", &error_msg, &app_handle);
                return Err(Box::new(e));
            }
            Err(_) => {
                let error_msg = "Request read timeout";
                println!("⏰ {}", error_msg);
                Self::log_event(&proxy_logs, "TIMEOUT", "client", error_msg, &app_handle);
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    error_msg,
                )));
            }
        };

        let request = String::from_utf8_lossy(&buffer[..n]);
        println!("🌐 Received request ({} bytes)", n);

        // Parse the request
        if let Some(first_line) = request.lines().next() {
            let parts: Vec<&str> = first_line.split_whitespace().collect();

            if parts.len() >= 3 {
                let method = parts[0];
                let url = parts[1];

                println!("🔍 Method: {}, URL: {}", method, url);

                // Extract host from URL
                if let Some(host) = Self::extract_host_from_url(url) {
                    println!("🏠 Extracted host: {}", host);

                    // Check if domain should be blocked
                    if Self::is_domain_blocked(&host) {
                        println!("🚫 BLOCKED: {}", host);
                        Self::log_event(
                            &proxy_logs,
                            "BLOCKED",
                            &host,
                            "Website blocked",
                            &app_handle,
                        );

                        // Emit event for frontend notification
                        if let Some(handle) = &app_handle {
                            use serde_json::json;
                            let _ = handle.emit(
                                "website-blocked",
                                json!({
                                    "url": host,
                                    "reason": "Website blocked by proxy",
                                    "timestamp": chrono::Utc::now().to_rfc3339()
                                }),
                            );
                        }

                        // Return blocked page
                        let blocked_response = Self::generate_blocked_response(&host);
                        stream.write_all(blocked_response.as_bytes()).await?;
                        stream.flush().await?;
                        return Ok(());
                    } else {
                        println!("✅ ALLOWED: {}", host);
                        Self::log_event(
                            &proxy_logs,
                            "ALLOWED",
                            &host,
                            "Website allowed - forwarding",
                            &app_handle,
                        );

                        // Handle CONNECT method (HTTPS)
                        if method == "CONNECT" {
                            return Self::handle_https_connect(
                                stream, host, app_handle, proxy_logs,
                            )
                            .await;
                        }

                        // Handle HTTP requests
                        return Self::handle_http_request(
                            stream,
                            request.to_string(),
                            host,
                            app_handle,
                            proxy_logs,
                        )
                        .await;
                    }
                }
            }
        }

        // If we can't parse the request, return a simple error response
        Self::log_event(
            &proxy_logs,
            "ERROR",
            "client",
            "Invalid request format",
            &app_handle,
        );
        let response = "HTTP/1.1 400 Bad Request\r\nContent-Type: text/plain\r\n\r\nBad Request";
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(())
    }

    fn is_domain_blocked(host: &str) -> bool {
        let clean_host = host.split(':').next().unwrap_or(host).to_lowercase();

        // Check exact matches
        if BLOCKED_DOMAINS.contains(&clean_host.as_str()) {
            return true;
        }

        // Check if any blocked domain is a suffix (for subdomains)
        for blocked_domain in BLOCKED_DOMAINS {
            if clean_host == *blocked_domain
                || clean_host.ends_with(&format!(".{}", blocked_domain))
            {
                return true;
            }
        }

        false
    }

    async fn handle_http_request(
        mut client_stream: TcpStream,
        request: String,
        host: String,
        app_handle: Option<AppHandle>,
        proxy_logs: Arc<Mutex<Vec<String>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🌐 Forwarding HTTP request to: {}", host);

        // Parse host and port
        let (server_host, server_port) = if host.contains(':') {
            let parts: Vec<&str> = host.split(':').collect();
            (parts[0].to_string(), parts[1].parse::<u16>().unwrap_or(80))
        } else {
            (host.clone(), 80)
        };

        // Connect to target server with timeout
        let connect_timeout = Duration::from_secs(10);
        let mut server_stream = match tokio::time::timeout(
            connect_timeout,
            TcpStream::connect((server_host.as_str(), server_port)),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                let error_msg = format!(
                    "Failed to connect to {}:{} - {}",
                    server_host, server_port, e
                );
                println!("❌ {}", error_msg);
                Self::log_event(&proxy_logs, "ERROR", &host, &error_msg, &app_handle);

                let error_response = "HTTP/1.1 502 Bad Gateway\r\nContent-Type: text/plain\r\n\r\nConnection failed\r\n";
                let _ = client_stream.write_all(error_response.as_bytes()).await;
                let _ = client_stream.flush().await;
                return Err(Box::new(e));
            }
            Err(_) => {
                let error_msg = format!("Connection timeout to {}:{}", server_host, server_port);
                println!("⏰ {}", error_msg);
                Self::log_event(
                    &proxy_logs,
                    "TIMEOUT",
                    &host,
                    "Connection timeout",
                    &app_handle,
                );

                let error_response = "HTTP/1.1 504 Gateway Timeout\r\nContent-Type: text/plain\r\n\r\nGateway timeout\r\n";
                let _ = client_stream.write_all(error_response.as_bytes()).await;
                let _ = client_stream.flush().await;
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    error_msg,
                )));
            }
        };

        // Send the request to the server
        server_stream.write_all(request.as_bytes()).await?;
        server_stream.flush().await?;

        // Forward response back to client with timeout
        let transfer_timeout = Duration::from_secs(30);
        let copy_result = tokio::time::timeout(
            transfer_timeout,
            tokio::io::copy(&mut server_stream, &mut client_stream),
        )
        .await;

        match copy_result {
            Ok(Ok(bytes_copied)) => {
                Self::log_event(
                    &proxy_logs,
                    "COMPLETE",
                    &host,
                    &format!("HTTP forwarding complete ({} bytes)", bytes_copied),
                    &app_handle,
                );
                println!("✅ HTTP request forwarded successfully");
                Ok(())
            }
            Ok(Err(e)) => {
                let error_msg = format!("Error during data transfer: {}", e);
                println!("❌ {}", error_msg);
                Self::log_event(&proxy_logs, "ERROR", &host, &error_msg, &app_handle);
                Err(Box::new(e))
            }
            Err(_) => {
                let error_msg = "Data transfer timeout";
                println!("⏰ {}", error_msg);
                Self::log_event(&proxy_logs, "TIMEOUT", &host, error_msg, &app_handle);
                Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    error_msg,
                )))
            }
        }
    }

    async fn handle_https_connect(
        mut client_stream: TcpStream,
        host: String,
        app_handle: Option<AppHandle>,
        proxy_logs: Arc<Mutex<Vec<String>>>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        println!("🔒 Establishing HTTPS tunnel to: {}", host);

        // Parse host and port
        let (server_host, server_port) = if host.contains(':') {
            let parts: Vec<&str> = host.split(':').collect();
            (parts[0].to_string(), parts[1].parse::<u16>().unwrap_or(443))
        } else {
            (host.clone(), 443)
        };

        // Connect to target server with timeout
        let connect_timeout = Duration::from_secs(10);
        let mut server_stream = match tokio::time::timeout(
            connect_timeout,
            TcpStream::connect((server_host.as_str(), server_port)),
        )
        .await
        {
            Ok(Ok(stream)) => stream,
            Ok(Err(e)) => {
                let error_msg = format!(
                    "Failed to connect to {}:{} - {}",
                    server_host, server_port, e
                );
                println!("❌ {}", error_msg);
                Self::log_event(&proxy_logs, "ERROR", &host, &error_msg, &app_handle);

                let error_response = "HTTP/1.1 502 Bad Gateway\r\n\r\n";
                let _ = client_stream.write_all(error_response.as_bytes()).await;
                let _ = client_stream.flush().await;
                return Err(Box::new(e));
            }
            Err(_) => {
                let error_msg = format!("Connection timeout to {}:{}", server_host, server_port);
                println!("⏰ {}", error_msg);
                Self::log_event(
                    &proxy_logs,
                    "TIMEOUT",
                    &host,
                    "Connection timeout",
                    &app_handle,
                );

                let error_response = "HTTP/1.1 504 Gateway Timeout\r\n\r\n";
                let _ = client_stream.write_all(error_response.as_bytes()).await;
                let _ = client_stream.flush().await;
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::TimedOut,
                    error_msg,
                )));
            }
        };

        // Send connection established response to client
        let response = "HTTP/1.1 200 Connection established\r\n\r\n";
        client_stream.write_all(response.as_bytes()).await?;
        client_stream.flush().await?;

        Self::log_event(
            &proxy_logs,
            "TUNNEL",
            &host,
            "HTTPS tunnel established",
            &app_handle,
        );

        // Split streams for bidirectional copying
        let (mut client_reader, mut client_writer) = client_stream.split();
        let (mut server_reader, mut server_writer) = server_stream.split();

        // Create bidirectional copy tasks with timeout
        let transfer_timeout = Duration::from_secs(60);
        let client_to_server = tokio::time::timeout(
            transfer_timeout,
            tokio::io::copy(&mut client_reader, &mut server_writer),
        );
        let server_to_client = tokio::time::timeout(
            transfer_timeout,
            tokio::io::copy(&mut server_reader, &mut client_writer),
        );

        // Run both directions concurrently
        let (result1, result2) = tokio::join!(client_to_server, server_to_client);

        // Log results
        let mut errors = 0;

        if let Err(_) = result1 {
            Self::log_event(
                &proxy_logs,
                "TIMEOUT",
                &host,
                "Client->Server transfer timeout",
                &app_handle,
            );
            errors += 1;
        }

        if let Err(_) = result2 {
            Self::log_event(
                &proxy_logs,
                "TIMEOUT",
                &host,
                "Server->Client transfer timeout",
                &app_handle,
            );
            errors += 1;
        }

        if errors == 0 {
            Self::log_event(
                &proxy_logs,
                "COMPLETE",
                &host,
                "HTTPS tunnel closed successfully",
                &app_handle,
            );
        } else {
            Self::log_event(
                &proxy_logs,
                "CLOSED",
                &host,
                &format!("HTTPS tunnel closed with {} errors", errors),
                &app_handle,
            );
        }

        println!("✅ HTTPS tunnel closed");
        Ok(())
    }

    fn extract_host_from_url(url: &str) -> Option<String> {
        if url.starts_with("http://") {
            let without_protocol = &url[7..];
            Some(without_protocol.split('/').next()?.to_string())
        } else if url.starts_with("https://") {
            let without_protocol = &url[8..];
            Some(without_protocol.split('/').next()?.to_string())
        } else if url.contains("://") {
            let parts: Vec<&str> = url.split("://").collect();
            if parts.len() >= 2 {
                Some(parts[1].split('/').next()?.to_string())
            } else {
                None
            }
        } else {
            Some(url.split('/').next()?.to_string())
        }
    }

    fn generate_blocked_response(domain: &str) -> String {
        let message = format!("Website {} is blocked by proxy settings.", domain);
        format!(
            "HTTP/1.1 403 Forbidden\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
            message.len(),
            message
        )
    }

    fn log_event(
        proxy_logs: &Arc<Mutex<Vec<String>>>,
        event_type: &str,
        target: &str,
        details: &str,
        app_handle: &Option<AppHandle>,
    ) {
        let timestamp = chrono::Utc::now().format("%H:%M:%S");
        let log_entry = format!("[{}] {}: {} - {}", timestamp, event_type, target, details);

        // Add to internal logs
        if let Ok(mut logs) = proxy_logs.try_lock() {
            logs.push(log_entry.clone());
            if logs.len() > 100 {
                logs.remove(0);
            }
        }

        // Send to UI via Tauri event
        if let Some(handle) = app_handle {
            use serde_json::json;
            let log_data = json!({
                "message": log_entry,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            let _ = handle.emit("proxy-log", log_data);
        }
    }

    pub async fn get_proxy_logs(&self) -> Vec<String> {
        let logs = self.proxy_logs.lock().await;
        logs.clone()
    }

    pub fn get_blocked_domains(&self) -> Vec<String> {
        BLOCKED_DOMAINS.iter().map(|s| s.to_string()).collect()
    }

    pub async fn get_proxy_info(&self) -> (String, u16) {
        ("127.0.0.1".to_string(), PROXY_PORT)
    }

    pub async fn is_blocking(&self) -> bool {
        true // Always blocking with the hardcoded list
    }

    pub async fn check_proxy_permissions(&self) -> Result<(), String> {
        Ok(()) // No special permissions needed for this simplified version
    }

    pub async fn enable_website_blocking(&self, _urls: Vec<String>) -> Result<(), String> {
        // Enable system proxy settings
        self.enable_system_proxy().await?;
        println!("✅ Website blocking enabled with system proxy");
        Ok(())
    }

    pub async fn disable_website_blocking(&self) -> Result<(), String> {
        // Disable system proxy settings
        self.disable_system_proxy().await?;
        println!("✅ Website blocking disabled, system proxy removed");
        Ok(())
    }

    pub async fn start_proxy_server_only(&self) -> Result<(), String> {
        self.start_proxy_server().await
    }

    pub async fn is_system_proxy_enabled(&self) -> Result<bool, String> {
        #[cfg(target_os = "macos")]
        {
            // Check HTTP proxy
            let output = Command::new("networksetup")
                .args(&["-getwebproxy", "Wi-Fi"])
                .output()
                .map_err(|e| format!("Failed to check HTTP proxy: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "networksetup HTTP check failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            let output_str = String::from_utf8_lossy(&output.stdout);
            let is_http_enabled = output_str.contains("Enabled: Yes")
                && output_str.contains("Server: 127.0.0.1")
                && output_str.contains(&format!("Port: {}", PROXY_PORT));

            // Check HTTPS proxy
            let output = Command::new("networksetup")
                .args(&["-getsecurewebproxy", "Wi-Fi"])
                .output()
                .map_err(|e| format!("Failed to check HTTPS proxy: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "networksetup HTTPS check failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            let output_str = String::from_utf8_lossy(&output.stdout);
            let is_https_enabled = output_str.contains("Enabled: Yes")
                && output_str.contains("Server: 127.0.0.1")
                && output_str.contains(&format!("Port: {}", PROXY_PORT));

            Ok(is_http_enabled && is_https_enabled)
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("netsh")
                .args(&["winhttp", "show", "proxy"])
                .output()
                .map_err(|e| format!("Failed to check system proxy: {}", e))?;

            if !output.status.success() {
                return Err(format!(
                    "netsh check failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            let output_str = String::from_utf8_lossy(&output.stdout);
            let expected_proxy = format!("127.0.0.1:{}", PROXY_PORT);
            Ok(output_str.contains(&expected_proxy))
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err("System proxy status check not supported on this OS".to_string())
        }
    }

    pub async fn enable_system_proxy(&self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Enable HTTP proxy
            let output = Command::new("networksetup")
                .args(&[
                    "-setwebproxy",
                    "Wi-Fi",
                    "127.0.0.1",
                    &PROXY_PORT.to_string(),
                ])
                .output()
                .map_err(|e| format!("Failed to enable HTTP proxy: {}", e))?;
            if !output.status.success() {
                return Err(format!(
                    "networksetup HTTP failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Enable HTTPS proxy
            let output = Command::new("networksetup")
                .args(&[
                    "-setsecurewebproxy",
                    "Wi-Fi",
                    "127.0.0.1",
                    &PROXY_PORT.to_string(),
                ])
                .output()
                .map_err(|e| format!("Failed to enable HTTPS proxy: {}", e))?;
            if !output.status.success() {
                return Err(format!(
                    "networksetup HTTPS failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            println!("✅ System proxy enabled on macOS");

            // Emit event to notify frontend of proxy status change
            if let Some(ref app_handle) = self.app_handle {
                let _ = app_handle.emit("system-proxy-changed", true);
            }

            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("netsh")
                .args(&[
                    "winhttp",
                    "set",
                    "proxy",
                    &format!(
                        "proxy-server=\"http=127.0.0.1:{};https=127.0.0.1:{}\"",
                        PROXY_PORT, PROXY_PORT
                    ),
                ])
                .output()
                .map_err(|e| format!("Failed to enable system proxy: {}", e))?;
            if !output.status.success() {
                return Err(format!(
                    "netsh failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            println!("✅ System proxy enabled on Windows");

            // Emit event to notify frontend of proxy status change
            if let Some(ref app_handle) = self.app_handle {
                let _ = app_handle.emit("system-proxy-changed", true);
            }

            Ok(())
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err("System proxy configuration not supported on this OS".to_string())
        }
    }

    pub async fn disable_system_proxy(&self) -> Result<(), String> {
        #[cfg(target_os = "macos")]
        {
            // Disable HTTP proxy
            let output = Command::new("networksetup")
                .args(&["-setwebproxystate", "Wi-Fi", "off"])
                .output()
                .map_err(|e| format!("Failed to disable HTTP proxy: {}", e))?;
            if !output.status.success() {
                return Err(format!(
                    "networksetup HTTP disable failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            // Disable HTTPS proxy
            let output = Command::new("networksetup")
                .args(&["-setsecurewebproxystate", "Wi-Fi", "off"])
                .output()
                .map_err(|e| format!("Failed to disable HTTPS proxy: {}", e))?;
            if !output.status.success() {
                return Err(format!(
                    "networksetup HTTPS disable failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            println!("✅ System proxy disabled on macOS");

            // Emit event to notify frontend of proxy status change
            if let Some(ref app_handle) = self.app_handle {
                let _ = app_handle.emit("system-proxy-changed", false);
            }

            Ok(())
        }

        #[cfg(target_os = "windows")]
        {
            let output = Command::new("netsh")
                .args(&["winhttp", "reset", "proxy"])
                .output()
                .map_err(|e| format!("Failed to disable system proxy: {}", e))?;
            if !output.status.success() {
                return Err(format!(
                    "netsh reset failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                ));
            }

            println!("✅ System proxy disabled on Windows");

            // Emit event to notify frontend of proxy status change
            if let Some(ref app_handle) = self.app_handle {
                let _ = app_handle.emit("system-proxy-changed", false);
            }

            Ok(())
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err("System proxy configuration not supported on this OS".to_string())
        }
    }
}
