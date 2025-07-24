#!/usr/bin/env bash

EVM_CODE=$(cat docs/res/HelloWorld.hex)
ABI_PATH="docs/res/HelloWorld.abi"
AURORA_LAST_VERSION=$(curl -s https://api.github.com/repos/aurora-is-near/aurora-engine/releases/latest | jq -r .tag_name)
ENGINE_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/$AURORA_LAST_VERSION/aurora-mainnet.wasm"
ENGINE_WASM_PATH="/tmp/aurora-mainnet.wasm"
VENV=/tmp/venv
NEARD_PATH="$HOME/.nearup/near/localnet"
NEARD_VERSION=$(curl -s https://rpc.mainnet.near.org/status | jq -r .version.version)

export NEARCORE_HOME="/tmp/localnet"
export PATH="$HOME/NearProtocol/aurora/aurora-cli-rs/target/debug/:$PATH:$USER_BASE_BIN"

# Install `nearup` utility if not installed before.
python3 -m venv $VENV
source $VENV/bin/activate
pip list | grep nearup > /dev/null || pip install nearup > /dev/null

download_neard() {
  if [[ ! -f $NEARD_PATH/neard ]]; then
    mkdir -p $NEARD_PATH
    url="https://s3-us-west-1.amazonaws.com/build.nearprotocol.com/nearcore/$(uname)-$(uname -m)/$NEARD_VERSION/neard.tar.gz"
    curl -s $url -o $NEARD_PATH/neard.tar.gz || error_exit
    tar xvzf $NEARD_PATH/neard.tar.gz -C $NEARD_PATH --strip-components 1
    chmod +x $NEARD_PATH/neard
  fi
}

start_node() {
  cmd="nearup run localnet --home $NEARCORE_HOME --binary-path $NEARD_PATH --num-nodes 1"
  $cmd || error_exit
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
download_neard
start_node
sleep 3
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
