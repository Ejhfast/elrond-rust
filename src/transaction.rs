//! Logic for constructing transactions on the Elrond network.

use serde::Serialize;
use super::{Account, ElrondAddress, ElrondClientError, Result};
use bigdecimal::BigDecimal;
use std::str::FromStr;

/// Network representation (e.g., MainNet or TestNet)
pub enum Network {
    MainNet,
    Custom(String)
}

impl Network {
    /// Get chain id associated with network
    pub fn chain_id(&self) -> String {
        match self {
            Network::MainNet => "1".to_string(),
            Network::Custom(s) => s.clone()
        }
    }
}

/// eGLD representation. "1 eGLD" is 10^18 on the blockchain
#[derive(Clone, Debug, PartialEq)]
pub struct ElrondCurrencyAmount {
    inner: String
}

impl ElrondCurrencyAmount {
    /// Create an amount of eGLD from a human input, e.g., "2 eGLD"
    pub fn new(amount: &str) -> Result<Self> {
        let amount = BigDecimal::from_str(amount).map_err(|_| {
            ElrondClientError::new("could not parse amount as bigdecimal")
        })?;
        let multiplier = BigDecimal::from_str("1000000000000000000").unwrap(); // safe
        let converted_amount = (amount * multiplier).with_scale(0);
        let inner = format!("{}", converted_amount);
        Ok(Self { inner })
    }
    /// Parse blockchain representation of eGLD into human readable form
    pub fn from_blockchain_precision(blockchain_amount: &str) -> Result<Self> {
        let amount = BigDecimal::from_str(blockchain_amount).map_err(|_| {
            ElrondClientError::new("could not parse amount as bigdecimal")
        })?;
        let divisor = BigDecimal::from_str("1000000000000000000").unwrap(); // safe
        let converted_amount = amount / divisor;
        let inner = format!("{}", converted_amount);
        Ok(Self { inner })
    }
    /// Convert to string for serialization
    pub fn to_string(&self) -> String {
        self.inner.clone()
    }
}

/// Transaction representation before it has been signed by an account
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")] 
pub struct UnsignedTransaction{
    nonce: u64,
    value: String,
    receiver: String,
    sender: String,
    gas_price: u64,
    gas_limit: u64,
    // 'chain_ID' needs to be weirdly cased due to requirements of Elrond API
    chain_ID: String,
    version: u64
}

impl UnsignedTransaction {
    /// Create new unsigned transaction
    pub fn new(
        nonce: u64,
        value: &str,
        receiver: &str,
        sender: &str,
        network: Network,
    ) -> Result<Self> {
        Ok(Self {
            nonce,
            value: ElrondCurrencyAmount::new(value)?.to_string(),
            receiver: ElrondAddress::new(receiver)?.to_string(),
            sender: ElrondAddress::new(sender)?.to_string(),
            gas_price: 1000000000,
            gas_limit: 50000,
            chain_ID: network.chain_id(),
            version: 1
        })
    }
    /// Serialize transaction for signing
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_| {
            ElrondClientError::new("could not serialize unsigned transaction")
        })
    }
    /// Sign the transaction with an `Account` to produce a `SignedTransaction`
    pub fn sign(&self, account: &Account) -> Result<SignedTransaction> {
        let serialized_tx = self.serialize()?;
        let signature = account.sign(&serialized_tx)?;
        Ok(SignedTransaction {
            nonce: self.nonce,
            value: self.value.clone(),
            receiver: self.receiver.clone(),
            sender: self.sender.clone(),
            gas_price: self.gas_price,
            gas_limit: self.gas_limit,
            chain_ID: self.chain_ID.clone(),
            version: self.version,
            // signed transaction requires empty data field if no data
            data: "".to_string(),
            signature
        })
    }
}

/// Representation of a signed transaction. Differs from unsigned transaction only by the
/// addition of a signature field
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")] 
pub struct SignedTransaction{
    nonce: u64,
    value: String,
    receiver: String,
    sender: String,
    gas_price: u64,
    gas_limit: u64,
    data: String,
    // 'chain_ID' needs to be weirdly cased due to requirements of Elrond API
    chain_ID: String,
    version: u64,
    signature: String
}

impl SignedTransaction {
    pub fn serialize(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|_| {
            ElrondClientError::new("could not serialize signed transaction")
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{UnsignedTransaction, Network, ElrondCurrencyAmount};
    use super::super::account::Account;
    #[test]
    fn create_serialize_and_sign_tx(){
        let private_key = "a4b36a5d97176618b5a7fcc9228d2fd98ee2f14ddd3d6462ae03e40eb487d15b";
        let account = Account::from_string(private_key).unwrap();
        let tx = UnsignedTransaction::new(
            0,
            "0.001",
            "erd16jats393r8rnut88yhvu5wvxxje57qzlj3tqk7n6jnf7f6cxs4uqfeh65k",
            &account.address.to_string(),
            Network::MainNet
        ).unwrap();
        let signed_tx = tx.sign(&account).unwrap();
        let serialized_correct = "{\"nonce\":0,\"value\":\"1000000000000000\",\"receiver\":\"erd16jats393r8rnut88yhvu5wvxxje57qzlj3tqk7n6jnf7f6cxs4uqfeh65k\",\"sender\":\"erd146apxa83wr7paz3gsg07dhcpg98ascjtpg9p8l8g5rpmg6chhchq9ccvmc\",\"gasPrice\":1000000000,\"gasLimit\":50000,\"data\":\"\",\"chainID\":\"1\",\"version\":1,\"signature\":\"78b7a59aeb9ff1a51637e23f29e0e22528a92a3508d69479806af93114496977ee1eacf045146fdbae89efd2cd3d9e0bcb2c52406515fa0548e2873554a0ac0d\"}";
        assert_eq!(serialized_correct, signed_tx.serialize().unwrap());
    }

    #[test]
    fn test_currency_precision(){
        let amount = ElrondCurrencyAmount::new("0.001").unwrap();
        assert_eq!(amount.to_string(), "1000000000000000")
    }
}