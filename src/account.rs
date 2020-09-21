use bech32::{self, FromBase32, ToBase32};
use ed25519_dalek::{PublicKey, SecretKey, Keypair, Signer};
use super::errors::{Result, ElrondClientError};
use rand::rngs::OsRng;

/// Representation for an address on the Elrond network. Addresses on Elrond are derived from
/// public keys on the ed25519 curve, encoded with the Bech32 format originally created for 
/// segwit on Bitcoin in BIP 0173.
#[derive(Clone, Debug, PartialEq)]
pub struct ElrondAddress {
    inner: String
}

/// Decode a `bech32` encoded value and return an ed25519 public key if it is a valid elrond address
fn check_elrond_address(addr_str: &str) -> Result<Option<PublicKey>> {
    let (hrd, data) = bech32::decode(addr_str).map_err(|_| {
        ElrondClientError::new("could not decode address from string")
    })?;
    if hrd == "erd" {
        let public_key_bytes = Vec::<u8>::from_base32(&data).map_err(|_| {
            ElrondClientError::new("could not convert base32 to bytes")
        })?;
        let public_key = PublicKey::from_bytes(&public_key_bytes).map_err(|_| {
            ElrondClientError::new("bytes in bech32 are not a valid public key")
        })?;
        Ok(Some(public_key))
    } else {
        Ok(None)
    }
}

impl ElrondAddress {
    /// Create a new `ElrondAddress` from a string value. This will check validity.
    pub fn new(addr_str: &str) -> Result<Self> {
        // verify if address is valid first
        if let Some(_pk) = check_elrond_address(addr_str)? {
            Ok(Self { inner: addr_str.to_string() })
        } 
        else {
            Err(ElrondClientError::new(
                &format!(
                    "'{}' is not a valid elrond address",
                    addr_str
                )
            ))
        }
    }
    /// Covert `ElrondAddress` to a public key
    pub fn to_public_key(&self) -> PublicKey {
        // this is safe as the only way to modify an inner value is via "new", which checks validity
        check_elrond_address(&self.inner)
            .expect("inner valid of elrond address corrupted (encoding)")
            .expect("inner valid of elrond address corrupted (not elrond address)")
    }
    /// Create a new `ElrondAddress` from a ed25519 public key
    pub fn from_public_key(public_key: &PublicKey) -> Result<Self> {
        let inner = bech32::encode("erd", public_key.to_base32()).map_err(|_| {
            ElrondClientError::new("could not encode public key as bech32")
        })?;
        Ok(Self { inner })
    }
    /// Get string representation of address
    pub fn to_string(&self) -> String {
        self.inner.clone()
    }
}

/// An account is derived from a ed25519 keypair. New transactions are signed with the secret key and
/// signatures can be verified with the public key. An address derived from the public key (in Bech32 
// format) may be the recipient of other transactions.
pub struct Account {
    pub secret: SecretKey,
    pub public: PublicKey,
    pub address: ElrondAddress
}

impl Account {
    /// Generate a new Elrond account
    pub fn generate() -> Result<Self> {
        let mut csprng = OsRng{};
        let secret = SecretKey::generate(&mut csprng);
        let public = (&secret).into();
        let address = ElrondAddress::from_public_key(&public)?;
        Ok(Self {
            secret,
            public,
            address
        })
    }
    /// Import an Elrond account from an existing secret (private key)
    pub fn from_secret(secret: SecretKey) -> Result<Self> {
        let public = (&secret).into();
        let address = ElrondAddress::from_public_key(&public)?;
        Ok(Self {
            secret,
            public,
            address
        })
    }
    /// Sign data with account and return signature as a hex string
    pub fn sign(&self, data: &str) -> Result<String> {
        let public_bytes = self.public.to_bytes();
        let secret_bytes = self.secret.to_bytes();
        let combined: Vec<u8> = secret_bytes.iter().chain(&public_bytes).map(|x| x.clone()).collect();
        // ed25519_dalek requires keypair for signing, can't just use secret
        // so perhaps save keypair and not the individual keys in the struct
        let keypair = Keypair::from_bytes(&combined).map_err(|_|{
            ElrondClientError::new("could not load keypair from public and secret key data")
        })?;
        let signature = keypair.sign(data.as_bytes()).to_bytes();
        assert_eq!(signature.len(), 64);
        Ok(hex::encode(signature.to_vec()))
    }
    /// Returns a hex string representation of the secret/private key associated with an account.
    /// You can restore an account from this data using `Account::from_string`.
    pub fn to_string(&self) -> String {
        let secret_bytes = self.secret.to_bytes();
        hex::encode(&secret_bytes.to_vec())
    }
    /// Load an account from a hex string representation of a secret/private key
    pub fn from_string(hex_str: &str) -> Result<Self> {
        let bytes = hex::decode(hex_str).map_err(|_| {
            ElrondClientError::new("could not decode hex string")
        })?;
        let secret = SecretKey::from_bytes(&bytes).map_err(|_| {
            ElrondClientError::new("hex string bytes do not encode valid secret key")
        })?;
        Self::from_secret(secret)
    }
}

#[cfg(test)]
mod tests {
    use super::{Account, ElrondAddress, SecretKey};
    #[test]
    fn generate_and_test_account() {
        let account = Account::generate().unwrap();
        let secret_bytes = &account.secret.to_bytes();
        let secret_copy = SecretKey::from_bytes(secret_bytes).unwrap();
        let account2 = Account::from_secret(secret_copy).unwrap();
        assert_eq!(&account.secret.to_bytes(), &account2.secret.to_bytes());
        assert_eq!(&account.public.to_bytes(), &account2.public.to_bytes());
        assert_eq!(&account.address, &account2.address);
    }

    #[test]
    fn validate_address(){
        let addr_str = "erd16jats393r8rnut88yhvu5wvxxje57qzlj3tqk7n6jnf7f6cxs4uqfeh65k";
        let address = ElrondAddress::new(addr_str).unwrap();
        let public_key = address.to_public_key();
        let address2 = ElrondAddress::from_public_key(&public_key).unwrap();
        assert_eq!(address, address2);
    }

    #[test]
    fn signing_completes(){
        let private_key = "a4b36a5d97176618b5a7fcc9228d2fd98ee2f14ddd3d6462ae03e40eb487d15b";
        let account = Account::from_string(private_key).unwrap();
        let sig = account.sign("dummy data").unwrap();
        assert_eq!(sig, "a25d1b1e24cd9299396e0c6e191b80a8e4f54e19c495955f13d6c168e5784dd642f540938333851fb1abcaa6d13f327ac2341607ad2fbb941dfdeeb6c4fcd803");
    }

    #[test]
    fn save_and_load_account(){
        let account = Account::generate().unwrap();
        let account_as_string = account.to_string();
        println!("{}", &account_as_string);
        let account_copy = Account::from_string(&account_as_string).unwrap();
        assert_eq!(&account.secret.to_bytes(), &account_copy.secret.to_bytes());
        assert_eq!(&account.public.to_bytes(), &account_copy.public.to_bytes());
        assert_eq!(account.address.to_string(), account_copy.address.to_string());
    }

    #[test]
    fn address_from_private_key(){
        let private_key = "a4b36a5d97176618b5a7fcc9228d2fd98ee2f14ddd3d6462ae03e40eb487d15b";
        let account = Account::from_string(private_key).unwrap();
        assert_eq!("erd146apxa83wr7paz3gsg07dhcpg98ascjtpg9p8l8g5rpmg6chhchq9ccvmc", &account.address.to_string());
    }

}