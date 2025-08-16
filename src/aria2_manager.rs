use std::process::{Command, Child, Stdio};
use std::io;
use tokio::time::{sleep, Duration};
use reqwest::Client;

pub struct Aria2Manager {
    process: Option<Child>,
    client: Client,
}

impl Aria2Manager {
    pub fn new() -> Self {
        Self {
            process: None,
            client: Client::new(),
        }
    }

    /// Check if aria2 RPC is already running
    pub async fn is_aria2_running(&self) -> bool {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "aria2.getVersion",
            "id": "test"
        });

        match self.client
            .post("http://localhost:6800/jsonrpc")
            .json(&payload)
            .send()
            .await
        {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        }
    }

    /// Check if aria2c command is available
    pub fn is_aria2_installed(&self) -> bool {
        Command::new("aria2c")
            .arg("--version")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    }

    /// Start aria2c process if not already running
    pub async fn ensure_aria2_running(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if self.is_aria2_running().await {
            println!("✅ aria2c RPC server is already running");
            return Ok(());
        }

        // Check if aria2c is installed
        if !self.is_aria2_installed() {
            return Err("aria2c not found. Please install aria2: brew install aria2 (macOS) or apt install aria2 (Ubuntu)".into());
        }

        println!("🚀 Starting aria2c RPC server...");
        
        // Try to start aria2c
        let child = Command::new("aria2c")
            .args(&[
                "--enable-rpc",
                "--rpc-listen-all=true",
                "--rpc-allow-origin-all=true",
                "--rpc-listen-port=6800",
                "--continue=true",
                "--max-connection-per-server=16",
                "--max-concurrent-downloads=16",
                "--split=16",
                "--min-split-size=1M",
                "--daemon=false", // Don't daemonize so we can manage the process
            ])
            .stdout(Stdio::null()) // Suppress aria2c output
            .stderr(Stdio::null())
            .spawn();

        match child {
            Ok(mut process) => {
                // Wait a moment for aria2c to start up
                sleep(Duration::from_secs(2)).await;
                
                // Check if it's actually running
                if self.is_aria2_running().await {
                    println!("✅ aria2c RPC server started successfully");
                    self.process = Some(process);
                    Ok(())
                } else {
                    // Kill the process if it's not responding
                    let _ = process.kill();
                    Err("Failed to start aria2c RPC server - process started but not responding".into())
                }
            }
            Err(e) => {
                if e.kind() == io::ErrorKind::NotFound {
                    Err("aria2c not found. Please install aria2: brew install aria2 (macOS) or apt install aria2 (Ubuntu)".into())
                } else {
                    Err(format!("Failed to start aria2c: {}", e).into())
                }
            }
        }
    }

    /// Get aria2 version info
    pub async fn get_version(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "aria2.getVersion",
            "id": "version"
        });

        let response = self.client
            .post("http://localhost:6800/jsonrpc")
            .json(&payload)
            .send()
            .await?;

        let json: serde_json::Value = response.json().await?;
        
        if let Some(result) = json.get("result") {
            if let Some(version) = result.get("version") {
                return Ok(version.as_str().unwrap_or("unknown").to_string());
            }
        }

        Ok("unknown".to_string())
    }

    /// Stop the managed aria2c process
    pub fn stop(&mut self) {
        if let Some(mut process) = self.process.take() {
            println!("🛑 Stopping aria2c process...");
            let _ = process.kill();
            let _ = process.wait();
        }
    }
}

impl Drop for Aria2Manager {
    fn drop(&mut self) {
        self.stop();
    }
}

impl Default for Aria2Manager {
    fn default() -> Self {
        Self::new()
    }
}