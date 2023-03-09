# Setting up an Aurora localnet

## Prerequisites

### 1. Download `neard`

You can obtain the latest nearcore node release by cloning [the repo](https://github.com/near/nearcore) and compiling the binary yourself (use the command `make neard`), or by using [nearup](https://github.com/near-guildnet/nearup). See the [Near documentation](https://near-nodes.io/rpc/run-rpc-node-without-nearup) for more information.

### 2. Download `aurora-engine`

You can get the latest Aurora Engine Wasm artifact either from the [GitHub releases](https://github.com/aurora-is-near/aurora-engine/releases/latest) or by cloning [the repo](https://github.com/aurora-is-near/aurora-engine) and compiling the binary yourself (use the command `cargo make --profile mainnet build`). Note: if you want to be able to mint your own ETH tokens directly in the EVM on your localnet then you will need to build the Engine yourself (use the command `cargo make --profile mainnet build-test`).

### 3. Build the CLI

From the root of this repository (aurora-cli-rs) run the command `cargo build --release`. This requires having Rust installed.

## Setup localnet

### 1. Choose a directory for the local instance of nearcore to use

Choose any directory you like and set the `NEARCORE_HOME` environment variable. For example:

```
export NEARCORE_HOME=/home/$USER/.near/localnet/
```

### 2. Generate nearcore genesis and config files

Using the `neard` binary you obtained in the prerequisites:

```
neard --home $NEARCORE_HOME localnet --validators 1
```

### 3. Add the aurora account to the nearcore genesis file

Use the `aurora-cli-rs` binary you built in the perquisites:

```
aurora-cli-rs near init genesis --path $NEARCORE_HOME/node0/genesis.json
```

### 4. Update the CLI config with the RPC address of the local nearcore node

Use the `aurora-cli-rs` binary again:

```
aurora-cli-rs near init local-config -n $NEARCORE_HOME/node0/config.json -a $NEARCORE_HOME/node0/aurora_key.json
```

### 5. Start the local nearcore node in the background

Using the `neard` binary again:

```
nohup neard --home $NEARCORE_HOME/node0/ run > node.log &
```

Note: this command assumes you have `nohup` installed. If you are working on a platform without this utility then you can simply run the `neard` command in another terminal.

Note: if you are using `nohup` don't forget to kill the node when you are done with it using `jobs` and `kill` commands.

### 6. Deploy Aurora Engine

Suppose the Aurora Engine Wasm binary is located at a path given by the environment variable `ENGINE_WASM_PATH`. Then we can use this CLI to deploy the engine:

```
aurora-cli-rs near write engine-init -w $ENGINE_WASM_PATH -c 1313161556 -o aurora
```

## Using the localnet

After completing the setup above you can use this CLI to interact with it just like you would the testnet or mainnet.

For example, suppose you wanted to deploy a simple "Hello, World!" contract:

```
$ aurora-cli-rs near \
    write \
    deploy-code \
    $(cat docs/res/HelloWorld.hex)

Contact deployed to address: 0x592186c059e3d9564cac6b1ada6f2dc7ff1d78e9
```

```
$ aurora-cli-rs near \
    read \
    solidity -t 0x592186c059e3d9564cac6b1ada6f2dc7ff1d78e9 \
    call-args-by-name \
    --abi-path docs/res/HelloWorld.abi \
    -m "greet" \
    --arg '{}'

[String("Hello, World!")]
```
