#!/usr/bin/env bash

export NEARCORE_HOME="/tmp/localnet"

AURORA_PREV_VERSION="2.9.1"
AURORA_LAST_VERSION="2.9.1"
EVM_CODE=$(cat docs/res/HelloWorld.hex)
ABI_PATH="docs/res/HelloWorld.abi"
ENGINE_PREV_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/$AURORA_PREV_VERSION/aurora-mainnet.wasm"
ENGINE_LAST_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/$AURORA_LAST_VERSION/aurora-mainnet.wasm"
XCC_ROUTER_LAST_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/$AURORA_LAST_VERSION/aurora-factory-mainnet.wasm"
ENGINE_WASM_PATH="/tmp/aurora-mainnet.wasm"
XCC_ROUTER_WASM_PATH="/tmp/aurora-factory-mainnet.wasm"
USER_BASE_BIN=$(python3 -m site --user-base)/bin
NODE_KEY_PATH=$NEARCORE_HOME/node0/validator_key.json
AURORA_KEY_PATH=$NEARCORE_HOME/node0/aurora_key.json
AURORA_SECRET_KEY=27cb3ddbd18037b38d7fb9ae3433a9d6f5cd554a4ba5768c8a15053f688ee167
ENGINE_ACCOUNT=aurora.node0
 ## CHANGE ME FOR TESTNET
NEW_NAMED_ACCOUNT_FOR_LEDGER_TESTNET=xyz1123456781.testnet
NEW_NAMED_ACCOUNT_FOR_LEDGER_LOCALNET=xyz1123456781.node0
## USE YOUR OWN LEDGER PUB KEY HERE
LEDGER_ACCOUNT_ID=3f6ab82e37adf3e6c91c9aa8f23ecfbb80d54e624ea56b0a619b0d94d6ee732c

export PATH="$PATH:$USER_BASE_BIN:$HOME/.cargo/bin"

# Install `nearup` utility if not installed before.
pip3 list | grep nearup > /dev/null || pip3 install --user nearup

start_node() {
  cmd="nearup run localnet --home $NEARCORE_HOME"

  if [[ $(uname -m) == "arm64" ]]; then # Check for local execution
    cmd="$cmd --binary-path $HOME/.nearup/near/localnet"
  fi

  $cmd > /dev/null 2>&1
}

stop_node() {
  nearup stop > /dev/null 2>&1
}

finish() {
  # Stop NEAR node.
  stop_node
  # Cleanup
  rm -rf $NEARCORE_HOME

  if [[ -z "$1" ]]; then
    exit 0
  else
    exit "$1"
  fi
}

error_exit() {
  finish 1
}

assert_eq() {
  if [[ $1 != $2 ]]; then
    echo "Unexpected result, should be $1 but actual is $2"
    finish 1
  fi
}

# Start NEAR node.
start_node
sleep 1

# Download Aurora EVM 2.8.1.
curl -sL $ENGINE_PREV_WASM_URL -o $ENGINE_WASM_PATH || error_exit

export NEAR_KEY_PATH=$NODE_KEY_PATH


## TESTNET ONLY

# Balance of Ledger account.
echo "View Ledger account: $LEDGER_ACCOUNT_ID (was already funded by another account)"
aurora-cli --network testnet view-account $LEDGER_ACCOUNT_ID || error_exit
sleep 1

# Create named account for the current ledger signe pub key.
echo "creating account $NEW_NAMED_ACCOUNT_FOR_LEDGER_TESTNET using ledger on testnet"
aurora-cli -u --network testnet create-account --account $NEW_NAMED_ACCOUNT_FOR_LEDGER_TESTNET --balance 1
sleep 2

# Balance of Ledger account.
echo "View Ledger account: $NEW_NAMED_ACCOUNT_FOR_LEDGER_TESTNET on testnet"
aurora-cli --network testnet view-account $NEW_NAMED_ACCOUNT_FOR_LEDGER_TESTNET || error_exit
sleep 2

## LOCALNET ONLY

# Create an account for Aurora EVM.
aurora-cli create-account --account $ENGINE_ACCOUNT --balance 100 > $AURORA_KEY_PATH || error_exit
sleep 1

# Fund the ledger account
echo "sending money to ledger account: $LEDGER_ACCOUNT_ID on localnet"
aurora-cli send-money --to $LEDGER_ACCOUNT_ID --amount 500
sleep 1

# Balance of Ledger account.
echo "View Ledger account: $LEDGER_ACCOUNT_ID on localnet"
aurora-cli view-account $LEDGER_ACCOUNT_ID || error_exit
sleep 2

# Stop NEAR node and clean up.
finish
