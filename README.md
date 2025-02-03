<br />
<p align="center">
<img src="img/aurora-cli-logo.png" width=500 alt="Aurora CLI">
</p>

<p align="center">
<strong>An instant, zero-config Aurora engine operator</strong>
</p>

<br />

[![CI](https://github.com/aurora-is-near/aurora-cli-rs/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/aurora-is-near/aurora-cli-rs/actions/workflows/rust.yml)
![rust 1.70.0+ required](https://img.shields.io/badge/rust-1.70.0+-blue.svg?label=MSRV)

## What is Engine?

[Aurora](https://doc.aurora.dev/getting-started/aurora-engine/) is an Ethereum Virtual Machine (EVM)
project built on the NEAR Protocol, that provides a solution for developers to deploy their apps
on an Ethereum-compatible, high-throughput, scalable and future-safe platform, with low transaction costs
for their users. Engine is the Aurora's implementation for it.

## What is Aurora CLI?

Aurora CLI is a command line interface to bootstrap Aurora Engine with rapid speed built with rust.

Aurora CLI comes pre-configuration with opinionated, sensible defaults for standard testing environments.
If other projects mention testing on Aurora, they are referring to the settings defined in this repo.

**Aurora CLI has the following advantages over api:**

- :pencil: **Easily modifiable EVM states through terminal**
- :handshake: **Quick to interact for rapid iterations**

See also prior version [aurora-cli](https://github.com/aurora-is-near/aurora-cli).

## Prerequisites

- :crab: Rust

## Quickstart

- 📦 Install `aurora-cli-rs` and start interacting with it:
  *`cargo install --git https://github.com/aurora-is-near/aurora-cli-rs.git`*
- 🔍 Check out what each command is for in the [Commands Reference](#commands-reference) section
- ✋ Have questions? Ask them at the official Aurora [forum](https://forum.aurora.dev/)

## Usage

In the following example, we will see how to deploy Aurora EVM on the `localnet`. Also, we will deploy a simple EVM
smart contract and will be interacting with it.

### **Requirements**

- Rust 1.75.0 or newer
- Python3

First what we need to do is to install `aurora-cli`:

### **Installing aurora-cli**

```shell
git clone https://github.com/aurora-engine/aurora-cli-rs
cd aurora-cli-rs && cargo install --path . 
```

Next we need to start a NEAR node locally. We can use the NEAR utility, `nearup`.

### **Start a NEAR node locally**

Install `nearup`:

```shell
pip3 install --user nearup
```

Start NEAR node:

```shell
nearup run localnet --home /tmp/localnet
```

When running the `nearup run localnet` command on Apple’s M-based hardware, a local build of `neard` is required due to
compatibility issues with the architecture.

Start NEAR node (Apple's M-based hardware):

```shell
nearup run localnet --home /tmp/localnet --binary-path /path/to/nearcore/target/release
```

Replace `/path/to/nearcore/target/release` with the actual path to the locally built `neard` binary.

### **Prepare an account and create a private key file for Aurora EVM**

```shell
aurora-cli --near-key-path /tmp/localnet/node0/validator_key.json create-account \
  --account aurora.node0 --balance 100 > /tmp/localnet/aurora_key.json
```

Let's check if the account has been created successfully:

```shell
aurora-cli view-account aurora.node0
```

### **Download and deploy Aurora EVM**

To download the latest version, run the following command:

```shell
curl -sL https://github.com/aurora-is-near/aurora-engine/releases/download/latest/aurora-mainnet.wasm -o /tmp/aurora-mainnet.wasm
```

Deploy Aurora EVM:

```shell
aurora-cli --near-key-path /tmp/localnet/aurora_key.json deploy-aurora /tmp/aurora-mainnet.wasm
```

Initialize Aurora EVM:

```shell
aurora-cli --engine aurora.node0 --near-key-path /tmp/localnet/aurora_key.json init --chain-id 1313161556 --owner-id aurora.node0
```

### **Deploy the EVM smart contract**

And now we can deploy the EVM smart contract. In our example, it will be a simple counter that can return its current
value and increment and decrement its value.

But before that we need to generate a private key for signing transactions:

```shell
aurora-cli key-pair --random
```

The response should be similar to this:

```json
{
  "address": "0xa003a6e0e1a1dc40aa9e496c1c058b2667c409f5",
  "secret_key": "3fac6dca1c6fc056b971a4e9090afbbfbdf3bc443e9cda595facb653cb1c01e1"
}
```

**_NOTE:_** The key should be used for demonstration only.

Deploy EVM smart contract:

```shell
aurora-cli --engine aurora.node0 --near-key-path /tmp/localnet/aurora_key.json deploy \
  --code $(cat docs/res/Counter.hex) \
  --abi-path docs/res/Counter.abi \
  --args '{"init_value":"5"}' \
  --aurora-secret-key 3fac6dca1c6fc056b971a4e9090afbbfbdf3bc443e9cda595facb653cb1c01e1
```

If everything went well, the response should be like this:

```
Contract has been deployed to address: 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf successfully
```

So. Now we have deployed the smart contract at address: `0x53a9fed853e02a39bf8d298f751374de8b5a6ddf`.

### **Interact with the smart contract**

First, let's check that the current value is the same as we set in the
initialization stage. For that, we will use the `view-call` operation, which doesn't demand a private key
because it is a read-only operation:

```shell
aurora-cli --engine aurora.node0 view-call -a 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf -f value \
  --abi-path docs/res/Counter.abi
```

If we see `5` then everything is right.

Now let's try to increment the value:

```shell
aurora-cli --engine aurora.node0 --near-key-path /tmp/localnet/aurora_key.json submit \
  --address 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf \
  -f increment \
  --abi-path docs/res/Counter.abi \
  --aurora-secret-key 3fac6dca1c6fc056b971a4e9090afbbfbdf3bc443e9cda595facb653cb1c01e1
```

In the response, we can see if the transaction was successful and the amount of gas used for the execution of this
transaction.

Let's make sure that our value was incremented:

```shell
aurora-cli --engine aurora.node0 view-call -a 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf -f value \
  --abi-path docs/res/Counter.abi
```

So, if we can see `6` in the output then the demo was successful. That's it!

### **Build aurora-cli with the advanced command line interface (Advanced CLI)**

Advanced CLI provides more options andadvanced features. You can try it by building with the following command:

```shell
cargo install --path . --no-default-features -F advanced
```

Documentation on how to work with the advanced version of `aurora-cli` can be found [here](docs/localnet.md).

### **Browse Deployed EVM Metadata**

```shell
aurora-cli --engine aurora.node0 get-version
aurora-cli --engine aurora.node0 get-owner
aurora-cli --engine aurora.node0 get-bridge-prover
aurora-cli --engine aurora.node0 get-chain-id
```

### **Examining EVM contract state**

```shell
aurora-cli --engine aurora.node0 get-nonce 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf
aurora-cli --engine aurora.node0 get-code 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf
aurora-cli --engine aurora.node0 get-balance 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf
aurora-cli --engine aurora.node0 get-storage-at \
  --address 0x53a9fed853e02a39bf8d298f751374de8b5a6ddf \
  --key 0x0000000000000000000000000000000000000000000000000000000000000000
```

### **Silo methods**

Retrieves the current fixed gas set in the Silo contract.

```shell
aurora-cli --engine aurora.node0 get-fixed-gas
```

Sets the fixed gas in the Silo contract to a specific value.

```shell
aurora-cli --engine aurora.node0 set-fixed-gas 0
```

Check whitelists statuses

```shell
aurora-cli --engine aurora.node0 get-whitelist-status admin
aurora-cli --engine aurora.node0 get-whitelist-status evm-admin
aurora-cli --engine aurora.node0 get-whitelist-status account
aurora-cli --engine aurora.node0 get-whitelist-status address
```

Add whitelist entry

```shell
aurora-cli --engine aurora.node0 add-entry-to-whitelist --kind <whitelist-kind> --entry <entry-value>
```

Remove whitelist entry

```shell
aurora-cli --engine aurora.node0 remove-entry-from-whitelist --kind <whitelist-kind> --entry <entry-value>
```

Disable whitelist status

```shell
aurora-cli --engine aurora.node0 set-whitelist-status --kind <whitelist-kind> --status 0
```

Replace `<whitelist-kind>` with the desired whitelist type (admin, evm-admin, account, or address), and `<entry-value>`
with the address or account to be whitelisted or removed.

Add whitelist batch

```shell
aurora-cli --engine aurora.node0 add-entry-to-whitelist-batch path/to/batch_list.json
```

The batch should be provided in a JSON format. Each entry in the JSON array should have two properties: `kind` and
either `account_id` or `address`, depending on the type of whitelist being updated.

Example JSON batch file (`batch_list.json`):

```json
[
  {
    "kind": "Admin",
    "account_id": "account.near"
  },
  {
    "kind": "EvmAdmin",
    "address": "0xef5d992c74e531bba6bf92ca1476d8ca4ca1b997"
  },
  {
    "kind": "Account",
    "account_id": "account1.near"
  },
  {
    "kind": "Address",
    "address": "0x92f854dadc0526717893da71cb44012fd4b8faac"
  }
]
```

## Commands Reference

- [`aurora-cli help`](#aurora-cli-help)
- [`aurora-cli create-account`](#aurora-cli-create-account)
- [`aurora-cli view-account`](#aurora-cli-view-account)
- [`aurora-cli deploy-aurora`](#aurora-cli-deploy-aurora)
- [`aurora-cli init`](#aurora-cli-init)
- [`aurora-cli get-chain-id`](#aurora-cli-get-chain-id)
- [`aurora-cli get-nonce`](#aurora-cli-get-nonce)
- [`aurora-cli get-block-hash`](#aurora-cli-get-block-hash)
- [`aurora-cli get-code`](#aurora-cli-get-code)
- [`aurora-cli get-balance`](#aurora-cli-get-balance)
- [`aurora-cli get-upgrade-index`](#aurora-cli-get-upgrade-index)
- [`aurora-cli get-version`](#aurora-cli-get-version)
- [`aurora-cli get-owner`](#aurora-cli-get-owner)
- [`aurora-cli set-owner`](#aurora-cli-set-owner)
- [`aurora-cli get-bridge-prover`](#aurora-cli-get-bridge-prover)
- [`aurora-cli get-storage-at`](#aurora-cli-get-storage-at)
- [`aurora-cli register-relayer`](#aurora-cli-register-relayer)
- [`aurora-cli pause-precompiles`](#aurora-cli-pause-precompiles)
- [`aurora-cli resume-precompiles`](#aurora-cli-resume-precompiles)
- [`aurora-cli paused-precompiles`](#aurora-cli-paused-precompiles)
- [`aurora-cli factory-update`](#aurora-cli-factory-update)
- [`aurora-cli factory-get-wnear-address`](#aurora-cli-factory-get-wnear-address)
- [`aurora-cli factory-set-wnear-address`](#aurora-cli-factory-set-wnear-address)
- [`aurora-cli fund-xcc-sub-account`](#aurora-cli-fund-xcc-sub-account)
- [`aurora-cli upgrade`](#aurora-cli-upgrade)
- [`aurora-cli stage-upgrade`](#aurora-cli-stage-upgrade)
- [`aurora-cli deploy-upgrade`](#aurora-cli-deploy-upgrade)
- [`aurora-cli deploy`](#aurora-cli-deploy)
- [`aurora-cli view-call`](#aurora-cli-view-call)
- [`aurora-cli call`](#aurora-cli-call)
- [`aurora-cli submit`](#aurora-cli-submit)
- [`aurora-cli encode-address`](#aurora-cli-encode-address)
- [`aurora-cli key-pair`](#aurora-cli-key-pair)
- [`aurora-cli generate-near-key`](#aurora-cli-generate-near-key)
- [`aurora-cli get-fixed-gas`](#aurora-cli-get-fixed-gas)
- [`aurora-cli set-fixed-gas`](#aurora-cli-set-fixed-gas)
- [`aurora-cli set-silo-params`](#aurora-cli-set-silo-params)
- [`aurora-cli get-whitelist-status`](#aurora-cli-get-whitelist-status)
- [`aurora-cli set-whitelist-status`](#aurora-cli-set-whitelist-status)
- [`aurora-cli add-entry-to-whitelist`](#aurora-cli-add-entry-to-whitelist)
- [`aurora-cli add-entry-to-whitelist-batch`](#aurora-cli-add-entry-to-whitelist-batch)
- [`aurora-cli remove-entry-from-whitelist`](#aurora-cli-remove-entry-from-whitelist)
- [`aurora-cli set-key-manager`](#aurora-cli-set-key-manager)
- [`aurora-cli add-relayer-key`](#aurora-cli-add-relayer-key)
- [`aurora-cli remove-relayer-key`](#aurora-cli-remove-relayer-key)
- [`aurora-cli get-upgrade-delay-blocks`](#aurora-cli-get-upgrade-delay-blocks)
- [`aurora-cli set-upgrade-delay-blocks`](#aurora-cli-set-upgrade-delay-blocks)
- [`aurora-cli get-erc20-from-nep141`](#aurora-cli-get-erc20-from-nep141)
- [`aurora-cli get-nep141-from-erc20`](#aurora-cli-get-nep141-from-erc20)
- [`aurora-cli get-erc20-metadata`](#aurora-cli-get-erc20-metadata)
- [`aurora-cli set-erc20-metadata`](#aurora-cli-set-erc20-metadata)
- [`aurora-cli mirror-erc20-token`](#aurora-cli-mirror-erc20-token)
- [`aurora-cli set-eth-connector-contract-account`](#aurora-cli-set-eth-connector-contract-account)
- [`aurora-cli get-eth-connector-contract-account`](#aurora-cli-get-eth-connector-contract-account)
- [`aurora-cli set-eth-connector-contract-data`](#aurora-cli-set-eth-connector-contract-data)
- [`aurora-cli get-paused_flags`](#aurora-cli-get-paused-flags)
- [`aurora-cli set-paused_flags`](#aurora-cli-set-paused-flags)

### `aurora-cli help`

```console
$ aurora-cli help
Simple command line interface for communication with Aurora Engine

Usage: aurora-cli [OPTIONS] <COMMAND>

Commands:
  create-account                      Create new NEAR account
  view-account                        View NEAR account
  deploy-aurora                       Deploy Aurora EVM smart contract
  init                                Initialize Aurora EVM and ETH connector
  get-chain-id                        Return chain id of the network
  get-nonce                           Return next nonce for address
  get-block-hash                      Return block hash of the specified height
  get-code                            Return smart contract's code for contract address
  get-balance                         Return balance for address
  get-upgrade-index                   Return a height for a staged upgrade
  get-version                         Return Aurora EVM version
  get-owner                           Return Aurora EVM owner
  set-owner                           Set a new owner of Aurora EVM
  get-bridge-prover                   Return bridge prover
  get-storage-at                      Return a value from storage at address with key
  register-relayer                    Register relayer address
  pause-precompiles                   Pause precompiles
  resume-precompiles                  Resume precompiles
  paused-precompiles                  Return paused precompiles
  factory-update                      Updates the bytecode for user's router contracts
  factory-get-wnear-address           Return the address of the `wNEAR` ERC-20 contract
  factory-set-wnear-address           Sets the address for the `wNEAR` ERC-20 contract
  fund-xcc-sub-account                Create and/or fund an XCC sub-account directly
  upgrade                             Upgrade contract with provided code
  stage-upgrade                       Stage a new code for upgrade
  deploy-upgrade                      Deploy staged upgrade
  deploy                              Deploy EVM smart contract's code in hex
  call                                Call a method of the smart contract
  view-call                           Call a view method of the smart contract
  submit                              Call a modified method of the smart contract
  encode-address                      Encode address
  key-pair                            Return Public and Secret ED25519 keys
  generate-near-key                   Return randomly generated NEAR key for AccountId
  get-fixed-gas                       Return fixed gas
  set-fixed-gas                       Set fixed gas
  set-silo-params                     Set SILO params
  get-whitelist-status                Return a status of the whitelist
  set-whitelist-status                Set a status for the whitelist
  add-entry-to-whitelist              Add entry into the whitelist
  add-entry-to-whitelist-batch        Add entries into the whitelist
  remove-entry-from-whitelist         Remove the entry from the whitelist
  set-key-manager                     Set relayer key manager
  add-relayer-key                     Add relayer public key
  remove-relayer-key                  Remove relayer public key
  get-upgrade-delay-blocks            Get delay for upgrade in blocks
  set-upgrade-delay-blocks            Set delay for upgrade in blocks
  get-erc20-from-nep141               Get ERC-20 from NEP-141
  get-nep141-from-erc20               Get NEP-141 from ERC-20
  get-erc20-metadata                  Get ERC-20 metadata
  set-erc20-metadata                  Set ERC-20 metadata
  mirror-erc20-token                  Mirror ERC-20 token
  set-eth-connector-contract-account  Set eth connector account id
  get-eth-connector-contract-account  Get eth connector account id
  set-eth-connector-contract-data     Set eth connector data
  set-paused-flags                    Set eth connector paused flags
  get-paused-flags                    Get eth connector paused flags
  help                                Print this message or the help of the given subcommand(s)

Options:
      --network <NETWORK>              NEAR network ID [default: localnet]
      --engine <ACCOUNT_ID>            Aurora EVM account [default: aurora]
      --near-key-path <NEAR_KEY_PATH>  Path to file with NEAR account id and secret key in JSON format
  -h, --help                           Print help
  -V, --version                        Print version
```

### `aurora-cli view-account`

```console
$ aurora-cli help create-account
Create new NEAR account

Usage: aurora-cli create-account --account <ACCOUNT> --balance <BALANCE>

Options:
  -a, --account <ACCOUNT>  AccountId
  -b, --balance <BALANCE>  Initial account balance in NEAR
  -h, --help               Print help
```

### `aurora-cli create-account`

```console
$ aurora-cli help view-account
View NEAR account

Usage: aurora-cli view-account <ACCOUNT>

Arguments:
  <ACCOUNT>  AccountId

Options:
  -h, --help  Print help
```

### `aurora-cli deploy-aurora`

```console
$ aurora-cli help deploy-aurora
Deploy Aurora EVM smart contract

Usage: aurora-cli deploy-aurora <PATH>

Arguments:
  <PATH>  Path to the WASM file

Options:
  -h, --help  Print help
```

### `aurora-cli init`

```console
$ aurora-cli help init
Initialize Aurora EVM and ETH connector

Usage: aurora-cli init [OPTIONS]

Options:
      --chain-id <CHAIN_ID>
          Chain ID [default: 1313161556]
      --owner-id <OWNER_ID>
          Owner of the Aurora EVM
      --bridge-prover-id <BRIDGE_PROVER_ID>
          Account of the bridge prover
      --upgrade-delay-blocks <UPGRADE_DELAY_BLOCKS>
          How many blocks after staging upgrade can deploy it
      --custodian-address <CUSTODIAN_ADDRESS>
          Custodian ETH address
      --ft-metadata-path <FT_METADATA_PATH>
          Path to the file with the metadata of the fungible token
  -h, --help
          Print help
```

### `aurora-cli get-chain-id`

```console
$ aurora-cli help get-chain-id
Return chain id of the network

Usage: aurora-cli get-chain-id

Options:
  -h, --help  Print help
```

### `aurora-cli get-nonce`

```console
$ aurora-cli help get-nonce
Return next nonce for address

Usage: aurora-cli get-nonce <ADDRESS>

Arguments:
  <ADDRESS>  

Options:
  -h, --help  Print help

```

### `aurora-cli get-block-hash`

```console
$ aurora-cli help get-block-hash
Return block hash of the specified height

Usage: aurora-cli get-block-hash <HEIGHT>

Arguments:
  <HEIGHT>  

Options:
  -h, --help  Print help
```

### `aurora-cli get-code`

```console
$ aurora-cli help get-code
Return smart contract's code for contract address

Usage: aurora-cli get-code <ADDRESS>

Arguments:
  <ADDRESS>  

Options:
  -h, --help  Print help

```

### `aurora-cli get-balance`

```console
$ aurora-cli help get-balance
Return balance for address

Usage: aurora-cli get-balance <ADDRESS>

Arguments:
  <ADDRESS>  

Options:
  -h, --help  Print help
```

### `aurora-cli get-upgrade-index`

```console
$ aurora-cli help get-upgrade-index
Return a height for a staged upgrade

Usage: aurora-cli get-upgrade-index

Options:
  -h, --help  Print help
```

### `aurora-cli get-version`

```console
$ aurora-cli help get-version
Return Aurora EVM version

Usage: aurora-cli get-version

Options:
  -h, --help  Print help
```

### `aurora-cli get-owner`

```console
$ aurora-cli help get-owner
Return Aurora EVM owner

Usage: aurora-cli get-owner

Options:
  -h, --help  Print help
```

### `aurora-cli set-owner`

```console
$ aurora-cli help set-owner
Set a new owner of Aurora EVM

Usage: aurora-cli set-owner <ACCOUNT_ID>

Arguments:
  <ACCOUNT_ID>  

Options:
  -h, --help  Print help
```

### `aurora-cli get-bridge-prover`

```console
$ aurora-cli help get-bridge-prover
Return bridge prover

Usage: aurora-cli get-bridge-prover

Options:
  -h, --help  Print help
```

### `aurora-cli get-storage-at`

```console
$ aurora-cli help get-storage-at
Return a value from storage at address with key

Usage: aurora-cli get-storage-at --address <ADDRESS> --key <KEY>

Options:
  -a, --address <ADDRESS>  
  -k, --key <KEY>          
  -h, --help  Print help
```

### `aurora-cli register-relayer`

```console
$ aurora-cli help register-relayer
Register relayer address

Usage: aurora-cli register-relayer <ADDRESS>

Arguments:
  <ADDRESS>  

Options:
  -h, --help  Print help
```

### `aurora-cli pause-precompiles`

```console
$ aurora-cli help pause-precompiles
Pause precompiles

Usage: aurora-cli pause-precompiles <MASK>

Arguments:
  <MASK>  

Options:
  -h, --help  Print help
```

### `aurora-cli resume-precompiles`

```console
$ aurora-cli help resume-precompiles
Resume precompiles

Usage: aurora-cli resume-precompiles <MASK>

Arguments:
  <MASK>  

Options:
  -h, --help  Print help
```

### `aurora-cli paused-precompiles`

```console
$ aurora-cli help paused-precompiles
Return paused precompiles

Usage: aurora-cli paused-precompiles

Options:
  -h, --help  Print help
```

### `aurora-cli factory-update`

```console
$ aurora-cli help factory-update
Updates the bytecode for user's router contracts

Usage: aurora-cli factory-update <PATH>

Arguments:
  <PATH>  

Options:
  -h, --help  Print help
```

### `aurora-cli factory-get-wnear-address`

```console
$ aurora-cli help factory-get-wnear-address
Return the address of the `wNEAR` ERC-20 contract

Usage: aurora-cli factory-get-wnear-address

Options:
  -h, --help  Print help
```

### `aurora-cli factory-set-wnear-address`

```console
$ aurora-cli help factory-set-wnear-address
Sets the address for the `wNEAR` ERC-20 contract

Usage: aurora-cli factory-set-wnear-address <ADDRESS>

Arguments:
  <ADDRESS>  

Options:
  -h, --help  Print help
```

### `aurora-cli fund-xcc-sub-account`

```console
$ aurora-cli help fund-xcc-sub-account
Create and/or fund an XCC sub-account directly

Usage: aurora-cli fund-xcc-sub-account <TARGET> [WNEAR_ACCOUNT_ID] <DEPOSIT>

Arguments:
  <TARGET>            Address of the target
  [WNEAR_ACCOUNT_ID]  Wnear Account Id
  <DEPOSIT>           Attached deposit in NEAR

Options:
  -h, --help  Print help
```

### `aurora-cli stage-upgrade`

```console
$ aurora-cli help stage-upgrade
Stage a new code for upgrade

Usage: aurora-cli stage-upgrade <PATH>

Arguments:
  <PATH>  

Options:
  -h, --help  Print help
```

### `aurora-cli upgrade`

```console
$ aurora-cli help upgrade
Upgrade contract with provided code

Usage: aurora-cli upgrade <PATH>

Arguments:
  <PATH>  

Options:
  -h, --help  Print help
```

### `aurora-cli deploy-upgrade`

```console
$ aurora-cli help deploy-upgrade
Deploy staged upgrade

Usage: aurora-cli deploy-upgrade

Options:
  -h, --help  Print help
```

### `aurora-cli deploy`

```console
$ aurora-cli help deploy
Deploy EVM smart contract's code in hex

Usage: aurora-cli deploy [OPTIONS] --code <CODE>

Options:
      --code <CODE>                            Code in HEX to deploy
      --args <ARGS>                            Constructor arguments with values in JSON
      --abi-path <ABI_PATH>                    Path to ABI of the contract
      --aurora-secret-key <AURORA_SECRET_KEY>  Aurora EVM secret key
  -h, --help                                   Print help
```

### `aurora-cli view-call`

```console
$ aurora-cli help view-call
Call a view method of the smart contract

Usage: aurora-cli view-call [OPTIONS] --address <ADDRESS> --function <FUNCTION> --abi-path <ABI_PATH>

Options:
  -a, --address <ADDRESS>    Address of the smart contract
  -f, --function <FUNCTION>  Name of the function to call
      --args <ARGS>          Arguments with values in JSON
      --abi-path <ABI_PATH>  Path to ABI of the contract
  -h, --help                 Print help
```

### `aurora-cli call`

```console"
$ aurora-cli help call
Call a method of the smart contract

Usage: aurora-cli call [OPTIONS] --address <ADDRESS>

Options:
      --address <ADDRESS>  Address of the smart contract
      --input <INPUT>      Input data of the EVM transaction encoded in hex
      --value <VALUE>      Attached value in EVM transaction
  -h, --help               Print help
 ```

### `aurora-cli submit`

```console
$ aurora-cli help submit
Call a modified method of the smart contract

Usage: aurora-cli call [OPTIONS] --address <ADDRESS> --function <FUNCTION> --abi-path <ABI_PATH>

Options:
  -a, --address <ADDRESS>                      Address of the smart contract
  -f, --function <FUNCTION>                    Name of the function to call
      --args <ARGS>                            Arguments with values in JSON
      --abi-path <ABI_PATH>                    Path to ABI of the contract
      --value <VALUE>                          Value sending in EVM transaction
      --aurora-secret-key <AURORA_SECRET_KEY>  Aurora EVM secret key
  -h, --help                                   Print help
```

### `aurora-cli encode-address`

```console
$ aurora-cli help encode-address
Encode address

Usage: aurora-cli encode-address <ACCOUNT>

Arguments:
  <ACCOUNT>  

Options:
  -h, --help  Print help
```

### `aurora-cli key-pair`

```console
$ aurora-cli help key-pair
Return Public and Secret ED25519 keys

Usage: aurora-cli key-pair [OPTIONS]

Options:
      --random       Random
      --seed <SEED>  From seed
  -h, --help         Print help
```

### `aurora-cli generate-near-key`

```console
$ aurora-cli help generate-near-key
Return randomly generated NEAR key for AccountId

Usage: aurora-cli generate-near-key <ACCOUNT_ID> <KEY_TYPE>

Arguments:
  <ACCOUNT_ID>  AccountId
  <KEY_TYPE>    Key type: ed25519 or secp256k1

Options:
  -h, --help  Print help
```

### `aurora-cli get-fixed-gas`

```console
$ aurora-cli help get-fixed-gas
Return fixed gas

Usage: aurora-cli get-fixed-gas

Options:
  -h, --help  Print help
```

### `aurora-cli set-fixed-gas`

```console
$ aurora-cli help set-fixed-gas
Set fixed gas

Usage: aurora-cli set-fixed-gas <COST>

Arguments:
  <COST>  Fixed gas in EthGas

Options:
  -h, --help  Print help
```

### `aurora-cli set-silo-params`

```console
$ aurora-cli help set-silo-params
Set SILO params

Usage: aurora-cli set-silo-params --gas <GAS> --fallback-address <FALLBACK_ADDRESS>

Options:
  -g, --gas <GAS>                            Fixed gas in EthGas
  -f, --fallback-address <FALLBACK_ADDRESS>  Fallback EVM address
  -h, --help                                 Print help
```

### `aurora-cli get-whitelist-status`

```console
$ aurora-cli help get-whitelist-status
Return a status of the whitelist

Usage: aurora-cli get-whitelist-status <KIND>

Arguments:
  <KIND>  Kind of the whitelist

Options:
  -h, --help  Print help
```

### `aurora-cli set-whitelist-status`

```console
$ aurora-cli help set-whitelist-status
Set a status for the whitelist

Usage: aurora-cli set-whitelist-status --kind <KIND> --status <STATUS>

Options:
      --kind <KIND>      Kind of the whitelist
      --status <STATUS>  Status of the whitelist, 0/1
  -h, --help             Print help
```

### `aurora-cli add-entry-to-whitelist`

```console
$ aurora-cli help add-entry-to-whitelist
Add entry into the whitelist

Usage: aurora-cli add-entry-to-whitelist --kind <KIND> --entry <ENTRY>

Options:
      --kind <KIND>    Kind of the whitelist
      --entry <ENTRY>  Entry for adding to the whitelist
  -h, --help           Print help
```

### `aurora-cli add-entry-to-whitelist-batch`

```console
$ aurora-cli help add-entry-to-whitelist-batch
Add entries into the whitelist

Usage: aurora-cli add-entry-to-whitelist-batch <PATH>

Arguments:
  <PATH>  Path to JSON file with array of entries

Options:
  -h, --help  Print help
```

### `aurora-cli remove-entry-from-whitelist`

```console
$ aurora-cli help remove-entry-from-whitelist
Remove the entry from the whitelist

Usage: aurora-cli remove-entry-from-whitelist --kind <KIND> --entry <ENTRY>

Options:
      --kind <KIND>    Kind of the whitelist
      --entry <ENTRY>  Entry for removing from the whitelist
  -h, --help           Print help
```

### `aurora-cli set-key-manager`

```console
$ aurora-cli help set-key-manager
Set relayer key manager

Usage: aurora-cli set-key-manager [ACCOUNT_ID]

Arguments:
  [ACCOUNT_ID] AccountId of the key manager

Options:
  -h, --help  Print help
```

### `aurora-cli add-relayer-key`

```console
$ aurora-cli help add-relayer-key
Add relayer public key

Usage: aurora-cli add-relayer-key --public-key <PUBLIC_KEY> --allowance <ALLOWANCE>

Options:
      --public-key <PUBLIC_KEY>  Public key
      --allowance <ALLOWANCE>    Allowance
  -h, --help                     Print help
```

### `aurora-cli remove-relayer-key`

```console
$ aurora-cli help remove-relayer-key
Remove relayer public key

Usage: aurora-cli remove-relayer-key <PUBLIC_KEY>

Arguments:
  <PUBLIC_KEY>  Public key

Options:
  -h, --help  Print help
```

### `aurora-cli get-upgrade-delay-blocks`

```console
$ aurora-cli help  get-upgrade-delay-blocks
Get delay for upgrade in blocks

Usage: aurora-cli get-upgrade-delay-blocks

Options:
-h, --help  Print help
```

### `aurora-cli set-upgrade-delay-blocks`

```console
$ aurora-cli help set-upgrade-delay-blocks
Set delay for upgrade in blocks

Usage: aurora-cli set-upgrade-delay-blocks <BLOCKS>

Arguments:
  <BLOCKS>  Number blocks

Options:
  -h, --help  Print help
```

### `aurora-cli get-erc20-from-nep141`

```console
$ aurora-cli help get-erc20-from-nep141
Get ERC-20 from NEP-141

Usage: aurora-cli get-erc20-from-nep141 <ACCOUNT_ID>

Arguments:
  <ACCOUNT_ID> Account id of NEP-141

Options:
  -h, --help  Print help
```

### `aurora-cli get-nep141-from-erc20`

```console
$ aurora-cli help get-nep141-from-erc20
Get NEP-141 from ERC-20

Usage: aurora-cli get-nep141-from-erc20 <ADDRESS>

Arguments:
  <ADDRESS>  Address for ERC-20

Options:
  -h, --help  Print help
```

### `aurora-cli get-erc20-metadata`

```console
$ aurora-cli help get-erc20-metadata
Get ERC-20 metadata

Usage: aurora-cli get-erc20-metadata <ERC20_ID>

Arguments:
  <ERC20_ID>  Address or account id of the ERC-20 contract

Options:
  -h, --help  Print help
```

### `aurora-cli set-erc20-metadata`

```console
$ aurora-cli help set-erc20-metadata
Set ERC-20 metadata

Usage: aurora-cli set-erc20-metadata --erc20-id <ERC20_ID> --name <NAME> --symbol <SYMBOL> --decimals <DECIMALS>

Options:
      --erc20-id <ERC20_ID>  Address or account id of the ERC-20 contract
      --name <NAME>          Name of the token
      --symbol <SYMBOL>      Symbol of the token
      --decimals <DECIMALS>  Decimals of the token
  -h, --help                 Print help
```

### `aurora-cli mirror-erc20-token`

```console
$ aurora-cli help mirror-erc20-token
Mirror ERC-20 token

Usage: aurora-cli mirror-erc20-token --contract-id <CONTRACT_ID> --nep141 <NEP141>

Options:
      --contract-id <CONTRACT_ID>  Account of contract where ERC-20 has been deployed
      --nep141 <NEP141>            Account of corresponding NEP-141
  -h, --help                       Print help
```

### `aurora-cli set-eth-connector-contract-account`

```console
$ aurora-cli help set-eth-connector-contract-account
Set eth connector account id

Usage: aurora-cli set-eth-connector-contract-account [OPTIONS] --account-id <ACCOUNT_ID>

Options:
      --account-id <ACCOUNT_ID>      Account id of eth connector
      --withdraw-ser <WITHDRAW_SER>  Serialization type in withdraw method
  -h, --help           
```

### `aurora-cli get-eth-connector-contract-account`

```console
$ aurora-cli help get-eth-connector-contract-account
Get eth connector account id

Usage: aurora-cli get-eth-connector-contract-account

Options:
  -h, --help  Print help
```

### `aurora-cli set-eth-connector-contract-data`

```console
$ aurora-cli help set-eth-connector-contract-data
Set eth connector data

Usage: aurora-cli set-eth-connector-contract-data --prover-id <PROVER_ID> --custodian-address <CUSTODIAN_ADDRESS> --ft-metadata-path <FT_METADATA_PATH>

Options:
      --prover-id <PROVER_ID>
          Prover account id
      --custodian-address <CUSTODIAN_ADDRESS>
          Custodian ETH address
      --ft-metadata-path <FT_METADATA_PATH>
          Path to the file with the metadata of the fungible token
  -h, --help
          Print help
```

### `aurora-cli set-paused-flags`

```console
$ aurora-cli help set-paused-flags
Set eth connector paused flags

Usage: aurora-cli set-paused-flags <MASK>

Arguments:
  <MASK>  Pause mask

Options:
  -h, --help  Print help
```

### `aurora-cli get-paused-flags`

```console
$ aurora-cli help get-paused-flags
Get eth connector paused flags

Usage: aurora-cli get-paused-flags

Options:
  -h, --help  Print help
```
