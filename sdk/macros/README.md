# Aurora SDK Macros

This crate contains procedural macros for Aurora SDK.

## ContractMethod Derive Macro

The `ContractMethod` macro automatically implements the `ContractMethod` trait
for structs, simplifying the creation of contract methods.

### Usage

```rust
use aurora_sdk_rs::ContractMethod;
use serde::{Serialize, Deserialize};

#[derive(ContractMethod, Serialize, Deserialize)]
#[contract_method(
    method = "get_user_balance", 
    response = u128, 
    serialize_as = "json"
)]
pub struct GetUserBalance {
    #[contract_param]
    pub user_id: String,
    pub internal_data: u64,
}
```

### Attributes

#### Main attribute `#[contract_method(...)]`

Required for all structs with the following parameters:

- `method` - contract method name (string)
- `response` - method response type
- `serialize_as` - parameter serialization method: `"json"` or `"borsh"`
  (defaults to `"borsh"`)

#### Field attribute `#[contract_param]`

Optional attribute for selecting a field for serialization:

- If the attribute is specified on one field - only this field is serialized
- If the attribute is not specified on any field - empty parameters are returned
- Can only be specified on one field (macro limitation)
- **Special case**: Fields of type `Vec<u8>` are returned as-is without
  serialization

### Examples

#### Selective serialization with JSON

```rust
#[derive(ContractMethod, Serialize, Deserialize)]
#[contract_method(
    method = "get_user_data", 
    response = UserData, 
    serialize_as = "json"
)]
pub struct GetUserData {
    #[contract_param]
    pub user_id: String,
    pub cache_timestamp: u64,
    pub debug_info: String,
}
```

#### Vec<u8> field (returned as-is)

```rust
#[derive(ContractMethod)]
#[contract_method(
    method = "submit_raw_data", 
    response = (), 
    serialize_as = "json"
)]
pub struct SubmitRawData {
    #[contract_param]
    pub raw_data: Vec<u8>,
    pub metadata: String,
}
```

#### Selective serialization with Borsh

```rust
#[derive(ContractMethod, borsh::BorshSerialize, borsh::BorshDeserialize)]
#[contract_method(
    method = "transfer", 
    response = (), 
    serialize_as = "borsh"
)]
pub struct Transfer {
    #[contract_param]
    pub transfer_args: TransferArgs,
    pub metadata: String,
}
```

#### Unit struct (empty parameters)

```rust
#[derive(ContractMethod)]
#[contract_method(
    method = "get_version", 
    response = String, 
    serialize_as = "json"
)]
pub struct GetVersion;
```

#### Struct without parameters

```rust
#[derive(ContractMethod, Serialize, Deserialize)]
#[contract_method(
    method = "ping", 
    response = (), 
    serialize_as = "json"
)]
pub struct Ping {
    pub timestamp: u64,
    pub debug: String,
}
```

### Macro behavior

1. **With `#[contract_param]` attribute on `Vec<u8>`**: Field is returned as-is
   (without serialization)
2. **With `#[contract_param]` attribute on other types**: Serialized according
   to `serialize_as`
3. **Without attributes**: Returns empty vector `Vec::new()`
4. **Unit structs**: Always return empty vector
5. **Multiple fields with `#[contract_param]`**: Compilation error

### What the macro generates

The macro automatically implements the `ContractMethod` trait with the following
methods:

- `method_name(&self) -> &'static str` - returns the method name
- `params(&self) -> Result<Vec<u8>, std::io::Error>` - returns parameters
  according to the logic above

### Requirements

- For `serialize_as = "json"`: selected field must implement `serde::Serialize`
  (except `Vec<u8>`)
- For `serialize_as = "borsh"`: selected field must implement
  `borsh::BorshSerialize` (except `Vec<u8>`)
- Response type must implement `ContractMethodResponse`

### Supported response types

- `()` - empty response
- `String` - string response
- `Vec<u8>` - byte array
- `u128`, `u64`, `u32` - numeric types
- `AccountId` - NEAR account ID
- `Address` - Aurora address
- Other types implementing `ContractMethodResponse`
