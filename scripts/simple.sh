#!/usr/bin/env bash

export NEARCORE_HOME="/tmp/localnet"

EVM_CODE=$(cat docs/res/HelloWorld.hex)
ABI_PATH="docs/res/HelloWorld.abi"
ENGINE_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/latest/aurora-mainnet.wasm"
ENGINE_WASM_PATH="/tmp/aurora-mainnet.wasm"
USER_BASE_BIN=$(python3 -m site --user-base)/bin
NODE_KEY_PATH=$NEARCORE_HOME/node0/validator_key.json
AURORA_KEY_PATH=$NEARCORE_HOME/node0/aurora_key.json
AURORA_SECRET_KEY=27cb3ddbd18037b38d7fb9ae3433a9d6f5cd554a4ba5768c8a15053f688ee167
ENGINE_ACCOUNT=aurora.node0

export PATH="$PATH:$USER_BASE_BIN:$HOME/.cargo/bin"

# Install `nearup` utility if not installed before.
pip3 list | grep nearup > /dev/null || pip3 install --user nearup

start_node() {
  cmd="nearup run localnet --num-nodes 1 --home $NEARCORE_HOME --no-watcher --account-id near"

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
  exit
}

# Start NEAR node.
start_node
sleep 1

# Download Aurora EVM.
curl -sL $ENGINE_WASM_URL -o $ENGINE_WASM_PATH || finish

export NEAR_KEY_PATH=$NODE_KEY_PATH
# Create an account for Aurora EVM.
aurora-cli create-account --account $ENGINE_ACCOUNT --balance 100 > $AURORA_KEY_PATH || finish
sleep 2
# View info of created account.
aurora-cli view-account $ENGINE_ACCOUNT || finish
sleep 2
# Deploy Aurora EVM.
export NEAR_KEY_PATH=$AURORA_KEY_PATH
aurora-cli deploy-aurora $ENGINE_WASM_PATH || finish
sleep 2
# Init Aurora EVM.
aurora-cli --engine $ENGINE_ACCOUNT init || finish
#  --chain-id 1313161556 \
#  --owner-id $ENGINE_ACCOUNT \
#  --bridge-prover-id "prover" \
#  --upgrade-delay-blocks 5 \
#  --custodian-address 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D \
#  --ft-metadata-path /path/to/metadata.json || finish
sleep 2
# Deploy EVM code.
aurora-cli --engine $ENGINE_ACCOUNT deploy-evm-code --code $EVM_CODE --aurora-secret-key $AURORA_SECRET_KEY || finish
sleep 2

aurora-cli --engine $ENGINE_ACCOUNT get-chain-id || finish
aurora-cli --engine $ENGINE_ACCOUNT get-version || finish
aurora-cli --engine $ENGINE_ACCOUNT get-owner || finish
aurora-cli --engine $ENGINE_ACCOUNT get-bridge-prover || finish
aurora-cli --engine $ENGINE_ACCOUNT get-upgrade-index || finish
aurora-cli --engine $ENGINE_ACCOUNT get-balance 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D || finish
aurora-cli --engine $ENGINE_ACCOUNT get-code 0xa3078bf607d2e859dca0b1a13878ec2e607f30de || finish
aurora-cli key-pair --seed 1

# Stop NEAR node and clean up.
finish
