# elrond-rust

Generate keys and create and sign transactions on the Elrond network. This library and its dependencies are all written in pure Rust to allow easy WASM generation in the future. See the tests for more examples.

### Example interaction

```rust
use elrond_rust::client::Client;
use elrond_rust::account::{Account, ElrondAddress};
use elrond_rust::transaction::{UnsignedTransaction, Network};

// initialize new client
let client = Client::new();
// load account from private/secret key
let private_key = "a4b36a5d97176618b5a7fcc9228d2fd98ee2f14ddd3d6462ae03e40eb487d15b";
let account = Account::from_string(private_key).unwrap();
// look up the current account nonce to use for the tx
let nonce = client.get_address_nonce(&account.address).unwrap();
// create a new transaction to send 1 eGLD to some address
let tx = UnsignedTransaction::new(
    nonce, // current nonce
    "1", // amount of eGLD
    "erd16jats393r8rnut88yhvu5wvxxje57qzlj3tqk7n6jnf7f6cxs4uqfeh65k", // reciever
    &account.address.to_string(), // the account
    Network::MainNet
).unwrap();
// sign the transaction
let signed_tx = tx.sign(&account).unwrap();
// submit the transaction to the network
client.post_signed_transaction(signed_tx).unwrap();
```

### Key generation

```rust
use elrond_rust::account::Account;
let account = Account::generate();
// access key representations provided by ed25519_dalek 
let secret = account.secret;
let public = account.public;
```
