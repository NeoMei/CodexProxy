use std::process::{Child, Command};
use std::sync::Mutex;
use std::sync::Arc;

pub struct ProxyManager {
    child: Mutex<Option<Child>>,
    port: u16,
}

impl ProxyManager {
    pub fn new(port: u16) -> Self {
        ProxyManager { child: Mutex::new(None), port }
    }

    pub fn is_running(&self) -> bool {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(ref mut child) = *guard {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        log::warn!("Proxy exited with status: {:?}", status.code());
                        *guard = None;
                        false
                    }
                    Ok(None) => true,
                    Err(e) => {
                        log::warn!("Proxy wait error: {}", e);
                        *guard = None;
                        false
                    }
                }
            } else { false }
        } else { false }
    }

    pub fn port(&self) -> u16 { self.port }

    pub fn start(&self, proxy_path: &str) -> Result<(), String> {
        if self.is_running() { return Ok(()); }

        log::info!("Starting proxy: node {}", proxy_path);

        let mut cmd = Command::new("node");
        cmd.arg(proxy_path)
            .env("PROXY_PORT", self.port.to_string());

        // Hide console window on Windows
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
        }

        let child = cmd.spawn()
            .map_err(|e| format!("Failed to start proxy: {}", e))?;

        if let Ok(mut guard) = self.child.lock() {
            *guard = Some(child);
        }
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(ref mut child) = *guard {
                child.kill().map_err(|e| format!("Failed to stop proxy: {}", e))?;
                child.wait().ok();
            }
            *guard = None;
        }
        Ok(())
    }
}

pub type SharedProxyManager = Arc<ProxyManager>;
