use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

#[derive(Clone)]
pub struct StellarRpcClient {
    http: Client,
    url: String,
}

#[derive(Debug, Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'static str,
    id: u64,
    method: &'a str,
    params: Value,
}

#[derive(Debug, Deserialize)]
struct RpcResponse {
    result: Option<Value>,
    error: Option<RpcError>,
}

#[derive(Debug, Deserialize)]
struct RpcError {
    code: i64,
    message: String,
}

impl StellarRpcClient {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            url: url.into(),
        }
    }

    pub async fn latest_ledger(&self) -> Result<Value, String> {
        let request = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method: "getLatestLedger",
            params: json!({}),
        };
        let response = self
            .http
            .post(&self.url)
            .json(&request)
            .send()
            .await
            .map_err(|error| format!("Stellar RPC request failed: {error}"))?;

        if !response.status().is_success() {
            return Err(format!("Stellar RPC returned HTTP {}", response.status()));
        }

        let payload: RpcResponse = response
            .json()
            .await
            .map_err(|error| format!("invalid Stellar RPC response: {error}"))?;
        if let Some(error) = payload.error {
            return Err(format!(
                "Stellar RPC error {}: {}",
                error.code, error.message
            ));
        }
        payload
            .result
            .ok_or_else(|| "Stellar RPC response did not contain a result".to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_preserves_configured_rpc_url() {
        let client = StellarRpcClient::new("https://rpc.testnet.stellar.org");
        assert_eq!(client.url, "https://rpc.testnet.stellar.org");
    }
}
