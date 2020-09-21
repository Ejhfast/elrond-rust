use super::errors::{Result, ElrondClientError};
use serde_json::{Map, Value};
use super::account::ElrondAddress;
use super::transaction::{SignedTransaction, ElrondCurrencyAmount};

/// Internal helper for type of outgoing request
enum RequestType {
    GET,
    POST
}

/// Client manages interactions with the Elrond network
pub struct Client {
    endpoint: String
}

impl Client {
    /// Internal helper for submitting requests to the network
    fn request(&self, path: &str, request_type: RequestType, data: Option<Value>) -> Result<Value> {
        let full_path = format!("{}/{}", self.endpoint, path);
        let resp = match (request_type, data) {
            (RequestType::GET, Some(json)) => {
                ureq::get(&full_path).send_json(json)
            },
            (RequestType::GET, None) => {
                ureq::get(&full_path).call()
            },
            (RequestType::POST, Some(json)) => {
                ureq::post(&full_path).send_json(json)
            },
            (RequestType::POST, None) => {
                ureq::post(&full_path).call()
            }
        };
        if resp.ok() {
            let response = resp.into_json().map_err(|_| {
                ElrondClientError::new("could not decode response to JSON")
            })?;
            Ok(response)
        } else {
            let status = &resp.status();
            let error_response = resp.into_string().map_err(|_|{
                ElrondClientError::new("could not decode error response to string")
            })?;
            Err(ElrondClientError::new(
                &format!(
                    "error code {}, data='{}'",
                    status,
                    error_response
                )
            ))
        }
    }
}

/// Unpackage 'data' object from Elrond API response
fn parse_response_data(response: &Value) -> Result<&Map<String, Value>> {
    response
        .as_object()
        .ok_or(
            ElrondClientError::new("response is not a JSON object")
        )?
        .get("data")
        .ok_or(
            ElrondClientError::new("response does not contain 'data' field")
        )?
        .as_object()
        .ok_or(
            ElrondClientError::new("'data' is not a JSON object")
        )
}


impl Client {
    /// Create a new client that will work on Elrond MainNet
    pub fn new() -> Self {
        Self { endpoint: "https://api.elrond.com".to_string() }
    }

    /// Get the current nonce associated with an address
    pub fn get_address_nonce(&self, addr_str: &str) -> Result<u64> {
        let address = ElrondAddress::new(addr_str)?;
        let path = format!("address/{}/nonce", address.to_string());
        let response = self.request(&path, RequestType::GET, None)?;
        let nonce = parse_response_data(&response)?
            .get("nonce")
            .ok_or(
                ElrondClientError::new("response does not contain 'nonce' field")
            )?
            .as_u64()
            .ok_or(
                ElrondClientError::new("'nonce' is not a number")
            )?;
        Ok(nonce)
    }

    /// Post signed transaction to Elrond network and return hash of transaction
    pub fn post_signed_transaction(&self, signed_tx: SignedTransaction) -> Result<String>{
        let path = "transaction/send";
        let serialized_tx = signed_tx.serialize()?;
        // this unwrap is safe, just serialized it...
        let json_tx = serde_json::from_str(&serialized_tx).unwrap();
        let response = self.request(path, RequestType::POST, Some(json_tx))?;
        let tx_hash = parse_response_data(&response)?
            .get("txHash")
            .ok_or(ElrondClientError::new("response does not contain 'txHash' field"))?
            .as_str()
            .ok_or(ElrondClientError::new("tx hash is not a string"))?;
        Ok(tx_hash.to_string())
    }

    /// Get the balance associated with an Elrond address
    pub fn get_address_balance(&self, addr_str: &str) -> Result<ElrondCurrencyAmount> {
        let address = ElrondAddress::new(addr_str)?;
        let path = format!("address/{}/balance", address.to_string());
        let response = self.request(&path, RequestType::GET, None)?;
        let balance = parse_response_data(&response)?
            .get("balance")
            .ok_or(
                ElrondClientError::new("response does not contain 'balance' field")
            )?
            .as_str()
            .ok_or(
                ElrondClientError::new("'balance' is not a number")
            )?;
        Ok(ElrondCurrencyAmount::from_blockchain_precision(balance)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use super::super::account::Account;
    
    #[test]
    pub fn get_address_nonce(){
        let address = "erd16jats393r8rnut88yhvu5wvxxje57qzlj3tqk7n6jnf7f6cxs4uqfeh65k";
        let client = Client::new();
        let response = client.get_address_nonce(address).unwrap();
        println!("{:?}", response);
    }

    #[test]
    pub fn get_newly_generated_address_nonce() {
        let account = Account::generate().unwrap();
        let address = account.address.to_string();
        println!("{}", address);
        let client = Client::new();
        let nonce = client.get_address_nonce(&address).unwrap();
        println!("{:?}", nonce);
        assert_eq!(nonce, 0);
    }

    #[test]
    pub fn get_address_balance() {
        let client = Client::new();
        let address = "erd18gx50mf0xvz3c3xljm0s5pkz0zugsprltl86ux6f0sz94gqeua3q7l77wd";
        let balance = client.get_address_balance(address).unwrap();
        assert_eq!(balance.to_string(), "0.0001");
    }

}



