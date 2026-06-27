use crate::db::Provider;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Write a curl header to a temporary file so API keys are not exposed on the command line.
pub fn write_curl_header_file(header: &str) -> Result<PathBuf, String> {
    let mut path = std::env::temp_dir();
    path.push(format!("coding-plan-header-{}.txt", Uuid::new_v4()));
    fs::write(&path, format!("{}\n", header)).map_err(|e| e.to_string())?;
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
    }
    Ok(path)
}

fn codex_dir() -> PathBuf {
    if let Some(dir) = std::env::var_os("CODEX_HOME") {
        return PathBuf::from(dir);
    }
    dirs::home_dir().unwrap_or_else(|| PathBuf::from(".")).join(".codex")
}

fn catalog_file() -> PathBuf {
    codex_dir().join("models-catalog.json")
}

/// Write Codex config.toml to point to the proxy with model mappings.
/// Preserves unrelated user settings (plugins, MCP servers, sandbox, etc.).
pub fn write_codex_config(model: &str, proxy_port: u16, context_window: u64, verified_providers: &[Provider]) -> Result<(), String> {
    let dir = codex_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let path = dir.join("config.toml");

    let mut config: toml::Value = if path.exists() {
        let text = fs::read_to_string(&path).map_err(|e| e.to_string())?;
        text.parse().map_err(|e: toml::de::Error| e.to_string())?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };

    let table = config.as_table_mut().ok_or("config.toml root is not a table")?;

    table.insert("model".to_string(), toml::Value::String(model.to_string()));
    table.insert("model_provider".to_string(), toml::Value::String("custom".to_string()));
    table.insert("model_context_window".to_string(), toml::Value::Integer(context_window as i64));

    let mut models = toml::map::Map::new();
    let mut seen = std::collections::HashSet::new();
    for p in verified_providers.iter().filter(|p| p.verified && !p.api_key.is_empty()).filter(|p| seen.insert(&p.model)) {
        models.insert(p.model.clone(), toml::Value::String(p.model.clone()));
    }

    let mut custom = toml::map::Map::new();
    custom.insert("name".to_string(), toml::Value::String("Coding Plan".to_string()));
    custom.insert("wire_api".to_string(), toml::Value::String("responses".to_string()));
    custom.insert("requires_openai_auth".to_string(), toml::Value::Boolean(true));
    custom.insert("base_url".to_string(), toml::Value::String(format!("http://127.0.0.1:{}/v1", proxy_port)));
    custom.insert("models".to_string(), toml::Value::Table(models));

    let providers = table.entry("model_providers").or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    if let Some(providers_table) = providers.as_table_mut() {
        providers_table.insert("custom".to_string(), toml::Value::Table(custom));
    }

    let features = table.entry("features").or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
    if let Some(features_table) = features.as_table_mut() {
        features_table.entry("js_repl").or_insert_with(|| toml::Value::Boolean(false));
    }

    let out = toml::to_string_pretty(&config).map_err(|e| e.to_string())?;
    fs::write(&path, out).map_err(|e| e.to_string())?;
    Ok(())
}

/// Write auth.json (proxy handles real auth)
pub fn write_codex_auth() -> Result<(), String> {
    let dir = codex_dir();
    fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let auth = json!({"OPENAI_API_KEY": "proxy-managed"});
    fs::write(dir.join("auth.json"), serde_json::to_string_pretty(&auth).unwrap())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Read current Codex config.toml
pub fn read_codex_config() -> Result<String, String> {
    let path = codex_dir().join("config.toml");
    if path.exists() {
        fs::read_to_string(&path).map_err(|e| e.to_string())
    } else {
        Ok(String::new())
    }
}

/// Generate model catalog JSON for verified providers only
pub fn write_model_catalog(providers: &[Provider]) -> Result<(), String> {
    let mut seen = std::collections::HashSet::new();
    let models: Vec<serde_json::Value> = providers.iter()
        .filter(|p| p.enabled && p.verified && !p.api_key.is_empty())
        .filter(|p| seen.insert(&p.model))
        .map(|p| {
            let efforts: Vec<serde_json::Value> = if p.model.contains("deepseek") {
                vec!["low","medium","high","xhigh"]
            } else if p.model.contains("glm") {
                vec!["low","medium","high"]
            } else {
                vec!["medium"]
            }.into_iter().map(|e| json!({"effort": e, "description": e})).collect();

            json!({
                "slug": p.model,
                "display_name": p.name,
                "description": "",
                "default_reasoning_level": "medium",
                "supported_reasoning_levels": efforts,
                "shell_type": "shell_command",
                "visibility": "list",
                "supported_in_api": true,
                "priority": 0,
                "additional_speed_tiers": [],
                "supports_reasoning_summaries": false,
                "default_reasoning_summary": "none",
                "support_verbosity": false,
                "default_verbosity": "low",
                "apply_patch_tool_type": "freeform",
                "web_search_tool_type": "text_and_image",
                "truncation_policy": {"mode": "tokens", "limit": 10000},
                "supports_parallel_tool_calls": true,
                "supports_image_detail_original": true,
                "experimental_supported_tools": ["web_search", "web_search_preview", "web_search_2025_08_26", "browser", "chrome", "computer", "computer_use", "computer_use_preview", "file_search", "tool_search", "apply_patch", "function_shell", "container_auto", "namespace_tool", "custom_tool", "local_skill", "inline_skill", "local_environment"],
                "input_modalities": ["text", "image"],
                "supports_search_tool": true,
                "use_responses_lite": false,
                "base_instructions": "",
                "instructions_variables": {
                    "personality_default": "",
                    "personality_friendly": "",
                    "personality_pragmatic": ""
                },
                "availability_nux": null,
                "upgrade": null,
                "service_tiers": [],
                "context_window": p.context_window,
                "max_context_window": p.context_window,
                "max_output_tokens": p.max_output_tokens,
                "effective_context_window_percent": 95,
            })
        })
        .collect();

    let catalog = json!({"models": models});
    let path = catalog_file();
    fs::write(&path, serde_json::to_string_pretty(&catalog).unwrap())
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Test connectivity to a provider's upstream
pub async fn test_provider_connection(provider: &Provider) -> Result<String, String> {
    let url = provider.upstream.trim_end_matches('/').to_string();
    let is_chat = !url.contains("/anthropic");
    
    let body = if is_chat {
        serde_json::json!({"model": provider.model, "messages": [{"role": "user", "content": "hi"}], "max_tokens": 5})
    } else {
        serde_json::json!({"model": provider.model, "max_tokens": 5, "messages": [{"role": "user", "content": "hi"}]})
    };

    let endpoint = if is_chat { format!("{url}/chat/completions") } else { format!("{url}/messages") };

    let mut cmd = std::process::Command::new("curl");
    let auth_header = if is_chat { format!("Authorization: Bearer {}", provider.api_key) } else { format!("x-api-key: {}\nAuthorization: Bearer {}", provider.api_key, provider.api_key) };
    let header_path = write_curl_header_file(&auth_header)?;
    cmd.arg("-s").arg("--max-time").arg("10").arg("--noproxy").arg("*")
        .arg(&endpoint)
        .arg("-H").arg(format!("@{}", header_path.display()))
        .arg("-H").arg("content-type: application/json")
        .arg("-d").arg(serde_json::to_string(&body).unwrap_or_default())
        .arg("-w").arg("\nHTTP_STATUS:%{http_code}");

    if !is_chat { cmd.arg("-H").arg("anthropic-version: 2023-06-01"); }

    // Hide console window on Windows
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
    }

    let output_res = cmd.output();
    let _ = fs::remove_file(&header_path);
    let output = output_res.map_err(|e| format!("curl error: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let status_match = stdout.rfind("HTTP_STATUS:");
    let status = status_match.map(|i| stdout[i + 12..].trim().to_string()).unwrap_or_default();
    let body_text = status_match.map(|i| stdout[..i].trim()).unwrap_or(stdout.trim());
    let stderr = String::from_utf8_lossy(&output.stderr);

    if status.starts_with('2') {
        Ok("ok".to_string())
    } else {
        let msg = if !stderr.is_empty() { stderr } else { body_text.into() };
        Err(format!("HTTP {}: {}", status, msg.chars().take(300).collect::<String>()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_codex_config_preserves_unrelated_settings() {
        // Use a temp CODEX_HOME so we don't touch the real ~/.codex.
        let tmp = std::env::temp_dir().join(format!("codexproxy-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&tmp).unwrap();
        std::env::set_var("CODEX_HOME", &tmp);

        let initial = r#"
[plugins."browser@openai-bundled"]
enabled = true

[windows]
sandbox = "elevated"

[mcp_servers.node_repl]
command = "node_repl"
"#;
        std::fs::write(tmp.join("config.toml"), initial).unwrap();

        write_codex_config("test-model", 15731, 100000, &[]).unwrap();

        let out = std::fs::read_to_string(tmp.join("config.toml")).unwrap();
        let parsed: toml::Value = out.parse().unwrap();
        let table = parsed.as_table().unwrap();

        assert_eq!(table.get("model").unwrap().as_str().unwrap(), "test-model");
        assert!(table.get("plugins").is_some());
        assert_eq!(table["windows"]["sandbox"].as_str().unwrap(), "elevated");
        assert!(table.get("mcp_servers").is_some());

        // cleanup
        let _ = std::fs::remove_dir_all(&tmp);
    }
}
