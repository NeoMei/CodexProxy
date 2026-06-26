use std::process::{Child, Command};
use std::sync::Mutex;
use std::sync::Arc;
use std::net::TcpStream;
use std::time::Duration;
#[cfg(windows)]
use std::os::windows::process::CommandExt;

pub struct ProxyManager {
    child: Mutex<Option<Child>>,
    port: u16,
}

impl ProxyManager {
    pub fn new(port: u16) -> Self {
        ProxyManager { child: Mutex::new(None), port }
    }

    pub fn is_port_listening(&self) -> bool {
        TcpStream::connect_timeout(
            &format!("127.0.0.1:{}", self.port).parse().unwrap(),
            Duration::from_millis(200),
        ).is_ok()
    }

    pub fn is_running(&self) -> bool {
        if let Ok(mut guard) = self.child.lock() {
            if let Some(ref mut child) = *guard {
                match child.try_wait() {
                    Ok(Some(status)) => {
                        log::info!("Proxy exited: {:?}", status.code());
                        *guard = None;
                    }
                    Ok(None) => return true,
                    Err(_) => { *guard = None; }
                }
            }
        }
        self.is_port_listening()
    }

    pub fn port(&self) -> u16 { self.port }

    pub fn start(&self, proxy_path: &str) -> Result<(), String> {
        let owns_child = self.child.lock().map(|g| g.is_some()).unwrap_or(false);
        if !owns_child && self.is_port_listening() {
            kill_port_occupants(self.port);
            for _ in 0..6 {
                if !self.is_port_listening() { break; }
                std::thread::sleep(Duration::from_millis(500));
            }
        }
        if self.is_port_listening() {
            log::info!("Proxy port {} already in use", self.port);
            return Ok(());
        }
        if self.is_running() { return Ok(()); }

        log::info!("Starting proxy: node {}", proxy_path);
        let mut cmd = Command::new("node");
        cmd.arg(proxy_path).env("PROXY_PORT", self.port.to_string());
        #[cfg(windows)] { cmd.creation_flags(0x08000000); }

        let child = cmd.spawn().map_err(|e| format!("Failed to start proxy: {}", e))?;
        std::thread::sleep(Duration::from_millis(500));
        if let Ok(mut guard) = self.child.lock() { *guard = Some(child); }
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        // Kill our child
        if let Ok(mut guard) = self.child.lock() {
            if let Some(ref mut child) = *guard {
                child.kill().ok();
                child.wait().ok();
            }
            *guard = None;
        }
        // Kill any remaining proxy process on the port, but only if its command line
        // contains the proxy script path. This avoids killing unrelated node processes.
        kill_port_occupants(self.port);
        // Wait for port to free (max 3s)
        for _ in 0..6 {
            if !self.is_port_listening() { return Ok(()); }
            std::thread::sleep(Duration::from_millis(500));
        }
        if self.is_port_listening() {
            Err(format!("Port {} still busy. Stop the process manually.", self.port))
        } else {
            Ok(())
        }
    }
}

/// Kill processes listening on the given port whose command line references the proxy script.
#[cfg(windows)]
fn kill_port_occupants(port: u16) {
    let ps_cmd = format!(
        "Get-NetTCPConnection -LocalPort {port} -ErrorAction SilentlyContinue | ForEach-Object {{ \
            $p = Get-CimInstance Win32_Process -Filter \"ProcessId = $($_.OwningProcess)\" -ErrorAction SilentlyContinue; \
            if ($p -and $p.CommandLine -like '*proxy*index.mjs*') {{ \
                Stop-Process -Id $_.OwningProcess -Force -ErrorAction SilentlyContinue \
            }} \
        }}"
    );
    let _ = std::process::Command::new("powershell")
        .args(["-Command", &ps_cmd])
        .creation_flags(0x08000000)
        .output();
}

#[cfg(not(windows))]
fn kill_port_occupants(port: u16) {
    let Ok(output) = std::process::Command::new("lsof")
        .args(["-ti", &format!(":{port}")])
        .output()
    else { return };
    let text = String::from_utf8_lossy(&output.stdout);
    for pid_str in text.split_whitespace() {
        let Ok(pid) = pid_str.parse::<u32>() else { continue };
        let cmdline_path = std::path::PathBuf::from(format!("/proc/{pid}/cmdline"));
        let Ok(cmdline) = std::fs::read_to_string(&cmdline_path) else { continue };
        if !cmdline.contains("proxy/index.mjs") { continue; }
        let _ = std::process::Command::new("kill").args(["-9", pid_str]).output();
    }
}

pub type SharedProxyManager = Arc<ProxyManager>;
