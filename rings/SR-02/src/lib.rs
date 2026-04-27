//! SR-02 — MCP stdio Server
//!
//! JSON-RPC 2.0 over stdio for MCP protocol.
//! Delegates tool execution to SR-03.

use anyhow::Result;
use tri_mcp_sr01::{JsonRpcRequest, JsonRpcResponse, tool};
use tri_mcp_sr03::{ToolClient, client_from_env};
use serde_json::{json, Value};
use std::io::{self as stdio, BufRead, Write};
use tracing::info;

pub struct McpConfig {
    pub port: u16,
    pub server_host: String,
    pub auth_username: String,
    pub auth_password: String,
}

impl McpConfig {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(3026),
            server_host: std::env::var("SERVER_HOST")
                .unwrap_or_else(|_| "127.0.0.1".into()),
            auth_username: std::env::var("AUTH_USERNAME")
                .unwrap_or_else(|_| "perplexity".into()),
            auth_password: std::env::var("AUTH_PASSWORD")
                .unwrap_or_else(|_| "changeme".into()),
        }
    }
}

pub async fn run_stdio_loop() -> Result<()> {
    tracing_subscriber::fmt().init();

    let client = client_from_env()?;
    let stdin = stdio::stdin();
    let stdout = stdio::stdout();
    let mut out = stdout.lock();

    info!("mcp-server started (stdio)");

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let req: JsonRpcRequest = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                let resp = JsonRpcResponse::err(Value::Null, -32700, &e.to_string());
                writeln!(out, "{}", serde_json::to_string(&resp)?)?;
                out.flush()?;
                continue;
            }
        };

        let id = if req.id.is_null() {
            Value::Null
        } else {
            req.id.clone()
        };
        let params = if req.params.is_none() {
            json!({})
        } else {
            req.params.clone().unwrap()
        };

        let result = match req.method.as_str() {
            "initialize" => json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "tri-mcp", "version": "2.0.0" }
            }),
            "tools/list" => tools_list(),
            "tools/call" => {
                let name = params["name"].as_str().unwrap_or("");
                let args = &params["arguments"];
                call_tool(name, args, &client).await
            }
            "ping" => json!({ "pong": true }),
            other => {
                let resp = JsonRpcResponse::err(id.clone(), -32601, &format!("Method not found: {}", other));
                writeln!(out, "{}", serde_json::to_string(&resp)?)?;
                out.flush()?;
                continue;
            }
        };

        let resp = JsonRpcResponse::ok(id, result);
        writeln!(out, "{}", serde_json::to_string(&resp)?)?;
        out.flush()?;
    }

    Ok(())
}

fn tools_list() -> Value {
    json!({
        "tools": [
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
    })
}

async fn call_tool(name: &str, args: &Value, client: &ToolClient) -> Value {
    client.call_tool(name, args).await.unwrap_or_else(|e| {
        json!({
            "content": [{"type": "text", "text": e.to_string()}],
            "isError": true
        })
    })
}
