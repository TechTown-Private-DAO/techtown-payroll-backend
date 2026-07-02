use reqwest::Client;
use serde_json::json;
use std::error::Error;

pub struct StellarService {
    rpc_url: String,
    network_passphrase: String,
    client: Client,
}

impl StellarService {
    pub fn new(rpc_url: String, network_passphrase: String) -> Self {
        Self {
            rpc_url,
            network_passphrase,
            client: Client::new(),
        }
    }

    pub async fn create_dao_contract(
        &self,
        admin_address: &str,
        name: &str,
        symbol: &str,
        multisig_threshold: i32,
    ) -> Result<String, Box<dyn Error>> {
        // In production, this would deploy the smart contract
        // For now, return a mock address
        Ok("CXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".to_string())
    }

    pub async fn execute_payroll_contract(
        &self,
        payroll_id: i64,
        dao_id: i64,
        proof: &str,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/execute_payroll", self.rpc_url))
            .json(&json!({
                "payroll_id": payroll_id,
                "dao_id": dao_id,
                "proof": proof,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err("Failed to execute payroll".into());
        }

        Ok(())
    }

    pub async fn claim_payroll(
        &self,
        payroll_id: i64,
        employee_id: i64,
        employee_address: &str,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/claim_payroll", self.rpc_url))
            .json(&json!({
                "payroll_id": payroll_id,
                "employee_id": employee_id,
                "employee_address": employee_address,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err("Failed to claim payroll".into());
        }

        Ok(())
    }

    pub async fn get_balance(
        &self,
        address: &str,
        token_address: &str,
    ) -> Result<i128, Box<dyn Error>> {
        let response = self.client
            .get(&format!("{}/balance/{}/{}", self.rpc_url, address, token_address))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err("Failed to get balance".into());
        }

        let data: serde_json::Value = response.json().await?;
        Ok(data["balance"].as_i64().unwrap_or(0) as i128)
    }

    pub async fn transfer(
        &self,
        from: &str,
        to: &str,
        token_address: &str,
        amount: i128,
    ) -> Result<String, Box<dyn Error>> {
        let response = self.client
            .post(&format!("{}/transfer", self.rpc_url))
            .json(&json!({
                "from": from,
                "to": to,
                "token_address": token_address,
                "amount": amount,
            }))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err("Failed to transfer".into());
        }

        let data: serde_json::Value = response.json().await?;
        Ok(data["tx_hash"].as_str().unwrap_or("").to_string())
    }
}