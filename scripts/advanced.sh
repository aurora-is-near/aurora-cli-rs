#!/usr/bin/env bash

set -e

EVM_CODE=$(cat docs/res/HelloWorld.hex)
ABI_PATH="docs/res/HelloWorld.abi"
ENGINE_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/latest/aurora-mainnet.wasm"
ENGINE_WASM_PATH="/tmp/aurora-mainnet.wasm"
USER_BASE_BIN=$(python3 -m site --user-base)/bin

export PATH="$PATH:$USER_BASE_BIN:$HOME/.cargo/bin"
export NEARCORE_HOME="/tmp/localnet"

# Install `nearup` utility if not installed before.
if [[ $(pip3 list | grep nearup > /dev/null) -ne 0 ]]; then
  pip3 install --user nearup
fi

start_node() {
  cmd="nearup run localnet --num-nodes 1 --home $NEARCORE_HOME --no-watcher --account-id near"

  if [[ $(uname -m) == "arm64" ]]; then # Check for local execution
    cmd="$cmd --binary-path $HOME/.nearup/near/localnet"
  fi

  $cmd > /dev/null 2>&1
}

finish() {
  # Stop NEAR node.
  nearup stop > /dev/null 2>&1
  # Cleanup
  rm -rf $NEARCORE_HOME
  exit
}

# Download `neard` and preparing config files.
start_node
nearup stop > /dev/null 2>&1
sleep 2

# Update configs and add aurora key.
aurora-cli near init genesis --path $NEARCORE_HOME/node0/genesis.json
aurora-cli near init local-config -n $NEARCORE_HOME/node0/config.json -a $NEARCORE_HOME/node0/aurora_key.json

# Start NEAR node.
rm -rf $NEARCORE_HOME/node0/data
start_node
sleep 1

# Download Aurora EVM.
curl -sL $ENGINE_WASM_URL -o $ENGINE_WASM_PATH || finish

# Deploy and init Aurora EVM smart contract.
aurora-cli near write engine-init -w $ENGINE_WASM_PATH || finish
sleep 2

# Deploy EVM code.
aurora-cli near write deploy-code $EVM_CODE || finish
sleep 2

# Run EVM view call.
aurora-cli near read solidity -t 0x592186c059e3d9564cac6b1ada6f2dc7ff1d78e9 call-args-by-name \
    --abi-path $ABI_PATH -m "greet" --arg '{}'

finish
