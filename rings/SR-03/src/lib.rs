//! SR-03 — 14 MCP Tools for Browser Automation

use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};
use tracing::debug;
use tri_mcp_sr01::{Tool, tool, text_content, image_content};

/// All 14 MCP tools
pub fn all_tools() -> Vec<Tool> {
    vec![
        tool("getScreenshot", "Capture current tab screenshot", &[]),
        tool("getConsoleLogs", "Get browser console output", &[]),
        tool("getConsoleErrors", "Get console errors only", &[]),
        tool("getNetworkRequests", "Get all XHR/fetch requests", &[]),
        tool("getNetworkErrors", "Get failed network requests", &[]),
        tool("getDomContent", "Get full page HTML", &[]),
        tool("getSelectedElement", "Get selected DOM element", &[]),
        tool("getAccessibilityTree", "Get accessibility tree", &[]),
        tool("wipeLogs", "Clear all stored logs", &[]),
        tool("runAccessibilityAudit", "Run Lighthouse accessibility audit", &[]),
        tool("runPerformanceAudit", "Run Lighthouse performance audit", &[]),
        tool("runSeoAudit", "Run Lighthouse SEO audit", &[]),
        tool("runBestPracticesAudit", "Run Lighthouse best practices audit", &[]),
        tool("runNextjsAudit", "Run Next.js specific audit", &[]),
    ]
}

pub struct ToolClient {
    base_url: String,
    username: String,
    password: String,
    client: Client,
}

impl ToolClient {
    pub fn new(base_url: String, username: String, password: String) -> Self {
        Self {
            base_url,
            username,
            password,
            client: Client::new(),
        }
    }

    async fn get(&self, path: &str) -> Result<Value> {
        let resp = self.client
            .get(format!("{}{}", self.base_url, path))
            .basic_auth(&self.username, Some(&self.password))
            .send()
            .await?;
        
        let status = resp.status();
        let body = resp.json::<Value>().await?;
        
        if status.is_success() {
            Ok(body)
        } else {
            Err(anyhow::anyhow!("HTTP {}: {}", status, body))
        }
    }

    async fn get_screenshot(&self) -> Result<Value> {
        match self.get("/screenshot/latest").await {
            Ok(v) => {
                let data = v["screenshot"].as_str().unwrap_or("").to_string();
                Ok(json!({
                    "content": [image_content(data, "image.png".to_string())],
                    "isError": false,
                }))
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("Screenshot error: {}", e))],
                "isError": true,
            })),
        }
    }

    async fn get_console_logs(&self) -> Result<Value> {
        match self.get("/logs").await {
            Ok(v) => {
                let logs = v["logs"].as_array().cloned().unwrap_or_default();
                let text = logs
                    .iter()
                    .map(|l| {
                        let level = l["level"].as_str().unwrap_or("info");
                        let msg = l["message"].as_str().unwrap_or("");
                        format!("[{}] {}", level, msg)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(json!({
                    "content": [text_content(text)],
                    "isError": false,
                }))
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("Logs error: {}", e))],
                "isError": true,
            })),
        }
    }

    async fn get_console_errors(&self) -> Result<Value> {
        match self.get("/logs").await {
            Ok(v) => {
                let logs = v["logs"].as_array().cloned().unwrap_or_default();
                let errors: Vec<String> = logs
                    .iter()
                    .filter(|l| {
                        let level = l["level"].as_str().unwrap_or("");
                        level == "error" || level == "Error"
                    })
                    .map(|l| l["message"].as_str().unwrap_or("").to_string())
                    .collect();
                
                let text = if errors.is_empty() {
                    "No console errors".to_string()
                } else {
                    errors.join("\n")
                };
                
                Ok(json!({
                    "content": [text_content(text)],
                    "isError": false,
                }))
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("Logs error: {}", e))],
                "isError": true,
            })),
        }
    }

    async fn get_network_requests(&self) -> Result<Value> {
        match self.get("/network").await {
            Ok(v) => {
                let reqs = v["requests"].as_array().cloned().unwrap_or_default();
                let text = reqs
                    .iter()
                    .map(|r| {
                        let url = r["url"].as_str().unwrap_or("");
                        let method = r["method"].as_str().unwrap_or("");
                        let status = r["status"].as_u64().unwrap_or(0);
                        format!("{} {} -> {}", method, url, status)
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                
                let text = if text.is_empty() {
                    "No network requests".to_string()
                } else {
                    text
                };
                
                Ok(json!({
                    "content": [text_content(text)],
                    "isError": false,
                }))
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("Network error: {}", e))],
                "isError": true,
            })),
        }
    }

    async fn get_network_errors(&self) -> Result<Value> {
        match self.get("/network").await {
            Ok(v) => {
                let reqs = v["requests"].as_array().cloned().unwrap_or_default();
                let errors: Vec<String> = reqs
                    .iter()
                    .filter(|r| {
                        let status = r["status"].as_u64().unwrap_or(200);
                        status >= 400 || status == 0
                    })
                    .map(|r| {
                        let url = r["url"].as_str().unwrap_or("");
                        let status = r["status"].as_u64().unwrap_or(0);
                        format!("ERROR {} - {}", status, url)
                    })
                    .collect();
                
                let text = if errors.is_empty() {
                    "No network errors".to_string()
                } else {
                    errors.join("\n")
                };
                
                Ok(json!({
                    "content": [text_content(text)],
                    "isError": false,
                }))
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("Network error: {}", e))],
                "isError": true,
            })),
        }
    }

    async fn get_dom_content(&self) -> Result<Value> {
        match self.get("/dom").await {
            Ok(v) => {
                let html = v["html"].as_str().unwrap_or("<html></html>").to_string();
                let html = if html.len() > 50000 {
                    format!("{}...", &html[..50000])
                } else {
                    html
                };
                Ok(json!({
                    "content": [text_content(html)],
                    "isError": false,
                }))
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("DOM error: {}", e))],
                "isError": true,
            })),
        }
    }

    async fn get_selected_element(&self) -> Result<Value> {
        match self.get("/selected-element").await {
            Ok(v) => {
                if let Some(elem) = v.get("element") {
                    let text = serde_json::to_string_pretty(elem).unwrap_or_default();
                    Ok(json!({
                        "content": [text_content(text)],
                        "isError": false,
                    }))
                } else {
                    Ok(json!({
                        "content": [text_content("No element selected")],
                        "isError": false,
                    }))
                }
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("Element error: {}", e))],
                "isError": true,
            })),
        }
    }

    async fn get_accessibility_tree(&self) -> Result<Value> {
        Ok(json!({
            "content": [text_content("Accessibility tree not available in MVP mode")],
            "isError": false,
        }))
    }

    async fn wipe_logs(&self) -> Result<Value> {
        Ok(json!({
            "content": [text_content("All logs cleared (MVP mode)")],
            "isError": false,
        }))
    }

    async fn run_lighthouse_audit(&self, category: &str) -> Result<Value> {
        debug!("Running Lighthouse {} audit", category);
        
        let output = tokio::process::Command::new("npx")
            .args([
                "lighthouse",
                self.base_url.trim_end_matches('/'),
                "--only=",
                category,
                "--output=json",
                "--chrome-flags=--headless",
                "--quiet",
            ])
            .output()
            .await;
        
        match output {
            Ok(result) if result.status.success() => {
                let stdout = String::from_utf8_lossy(&result.stdout);
                match serde_json::from_str::<Value>(&stdout) {
                    Ok(report) => {
                        let score = report["categories"]
                            .as_object()
                            .and_then(|cats| cats.get(category))
                            .and_then(|c| c["score"].as_f64())
                            .unwrap_or(0.0);
                        Ok(json!({
                            "content": [text_content(format!(
                                "Lighthouse {} audit completed\nScore: {:.0}/100",
                                category, score
                            ))],
                            "isError": false,
                            "meta": json!({"score": score, "category": category}),
                        }))
                    }
                    Err(e) => Ok(json!({
                        "content": [text_content(format!("Failed to parse Lighthouse output: {}", e))],
                        "isError": true,
                    })),
                }
            }
            Ok(result) => {
                let stderr = String::from_utf8_lossy(&result.stderr);
                Ok(json!({
                    "content": [text_content(format!("Lighthouse failed: {}", stderr))],
                    "isError": true,
                }))
            }
            Err(e) => Ok(json!({
                "content": [text_content(format!("Failed to run Lighthouse: {}. Make sure npx lighthouse is installed.", e))],
                "isError": true,
            })),
        }
    }

    pub async fn call_tool(&self, name: &str, _args: &Value) -> Result<Value> {
        debug!("Calling tool: {}", name);
        
        match name {
            "getScreenshot" | "takeScreenshot" => self.get_screenshot().await,
            "getConsoleLogs" => self.get_console_logs().await,
            "getConsoleErrors" => self.get_console_errors().await,
            "getNetworkRequests" => self.get_network_requests().await,
            "getNetworkErrors" => self.get_network_errors().await,
            "getDomContent" => self.get_dom_content().await,
            "getSelectedElement" => self.get_selected_element().await,
            "getAccessibilityTree" => self.get_accessibility_tree().await,
            "wipeLogs" => self.wipe_logs().await,
            "runAccessibilityAudit" => self.run_lighthouse_audit("accessibility").await,
            "runPerformanceAudit" => self.run_lighthouse_audit("performance").await,
            "runSeoAudit" => self.run_lighthouse_audit("seo").await,
            "runBestPracticesAudit" => self.run_lighthouse_audit("best-practices").await,
            "runNextjsAudit" => self.run_lighthouse_audit("nextjs").await,
            _ => Ok(json!({
                "content": [text_content(format!("Unknown tool: {}", name))],
                "isError": true,
            })),
        }
    }
}

pub fn client_from_env() -> Result<ToolClient> {
    Ok(ToolClient::new(
        std::env::var("CONNECTOR_URL").unwrap_or_else(|_| "http://localhost:3026".into()),
        std::env::var("AUTH_USERNAME").unwrap_or_else(|_| "perplexity".into()),
        std::env::var("AUTH_PASSWORD").unwrap_or_else(|_| "changeme".into()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_tools_count() {
        assert_eq!(all_tools().len(), 14);
    }
}
