#!/usr/bin/env bash

EVM_CODE=$(cat docs/res/HelloWorld.hex)
ABI_PATH="docs/res/HelloWorld.abi"
AURORA_LAST_VERSION=$(curl -s https://api.github.com/repos/aurora-is-near/aurora-engine/releases/latest | jq -r .tag_name)
ENGINE_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/$AURORA_LAST_VERSION/aurora-mainnet.wasm"
ENGINE_WASM_PATH="/tmp/aurora-mainnet.wasm"
VENV=/tmp/venv

export NEARCORE_HOME="/tmp/localnet"

# Install `nearup` utility if not installed before.
python3 -m venv $VENV
source $VENV/bin/activate
pip list | grep nearup > /dev/null || pip install --user nearup > /dev/null

start_node() {
  cmd="nearup run localnet --home $NEARCORE_HOME --num-nodes 1"

  if [[ $(uname -m) == "arm64" ]]; then # Check for local execution
    cmd="$cmd --binary-path $HOME/.nearup/near/localnet"
  fi

  $cmd > /dev/null 2>&1
}

finish() {
  # Stop NEAR node.
  nearup stop > /dev/null 2>&1
  # Cleanup
  deactivate
  rm -rf $NEARCORE_HOME $VENV

  if [[ -z "$1" ]]; then
    exit 0
  else
    exit "$1"
  fi
}

error_exit() {
  finish 1
}

wait_for_block() {
  sleep 1.5
}

# Download `neard` and preparing config files.
start_node
nearup stop > /dev/null 2>&1
wait_for_block

# Update configs and add aurora key.
aurora-cli near init genesis --path $NEARCORE_HOME/node0/genesis.json
aurora-cli near init local-config -n $NEARCORE_HOME/node0/config.json -a $NEARCORE_HOME/node0/aurora_key.json

# Start NEAR node.
rm -rf $NEARCORE_HOME/node0/data
start_node
wait_for_block

# Download Aurora EVM.
curl -sL $ENGINE_WASM_URL -o $ENGINE_WASM_PATH || error_exit

# Deploy and init Aurora EVM smart contract.
aurora-cli near write engine-init -w $ENGINE_WASM_PATH || error_exit
wait_for_block

# Deploy EVM code.
aurora-cli near write deploy-code $EVM_CODE || error_exit
wait_for_block

# Run EVM view call.
aurora-cli near read solidity -t 0x592186c059e3d9564cac6b1ada6f2dc7ff1d78e9 call-args-by-name \
    --abi-path $ABI_PATH -m "greet" --arg '{}' || error_exit

finish
