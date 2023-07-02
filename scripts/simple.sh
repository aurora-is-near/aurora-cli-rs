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

export PATH="$PATH:$USER_BASE_BIN:$HOME/.cargo/bin"

# Install `nearup` utility if not installed before.
pip3 list | grep nearup > /dev/null || pip3 install --user nearup

start_node() {
  cmd="nearup run localnet --home $NEARCORE_HOME"

  if [[ $(uname -m) == "arm64" ]]; then # Check for local execution
    cmd="$cmd --binary-path $HOME/.nearup/near/localnet --num-nodes 1"
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
# Create an account for Aurora EVM.
aurora-cli create-account --account $ENGINE_ACCOUNT --balance 100 > $AURORA_KEY_PATH || error_exit
sleep 1

# View info of created account.
aurora-cli view-account $ENGINE_ACCOUNT || error_exit
sleep 1

# Deploy Aurora EVM.
export NEAR_KEY_PATH=$AURORA_KEY_PATH
aurora-cli deploy-aurora $ENGINE_WASM_PATH || error_exit
sleep 4
# Init Aurora EVM.
aurora-cli --engine $ENGINE_ACCOUNT init \
  --chain-id 1313161556 \
  --owner-id $ENGINE_ACCOUNT \
  --bridge-prover-id "prover" \
  --upgrade-delay-blocks 1 \
  --custodian-address 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D \
  --ft-metadata-path docs/res/ft_metadata.json || error_exit
sleep 2

# Deploy Hello World EVM code.
aurora-cli --engine $ENGINE_ACCOUNT deploy --code $EVM_CODE --aurora-secret-key $AURORA_SECRET_KEY || error_exit
sleep 1
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f greet \
  --abi-path $ABI_PATH || error_exit)
assert_eq "$result" "Hello, World!"
sleep 1

# Deploy Counter EVM code.
EVM_CODE=$(cat docs/res/Counter.hex)
ABI_PATH=docs/res/Counter.abi
aurora-cli --engine $ENGINE_ACCOUNT deploy --code $EVM_CODE --abi-path $ABI_PATH --args '{"init_value":"5"}' \
  --aurora-secret-key $AURORA_SECRET_KEY || error_exit
sleep 1
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f value \
  --abi-path $ABI_PATH || error_exit)
assert_eq "$result" "5"
sleep 1
aurora-cli --engine $ENGINE_ACCOUNT call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f increment \
  --abi-path $ABI_PATH \
  --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c || error_exit
sleep 1
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f value \
  --abi-path $ABI_PATH || error_exit)
assert_eq "$result" "6"
sleep 1
aurora-cli --engine $ENGINE_ACCOUNT call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f decrement \
  --abi-path $ABI_PATH \
  --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c || error_exit
sleep 1
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f value \
  --abi-path $ABI_PATH || error_exit)
assert_eq "$result" "5"
sleep 1

# Check read operations.
aurora-cli --engine $ENGINE_ACCOUNT get-chain-id || error_exit
version=$(aurora-cli --engine $ENGINE_ACCOUNT get-version || error_exit)
assert_eq "$version" $AURORA_PREV_VERSION
echo "$version"
aurora-cli --engine $ENGINE_ACCOUNT get-owner || error_exit
aurora-cli --engine $ENGINE_ACCOUNT get-bridge-prover || error_exit
aurora-cli --engine $ENGINE_ACCOUNT get-balance 0x04b678962787ccd195a8e324d4c6bc4d5727f82b || error_exit
aurora-cli --engine $ENGINE_ACCOUNT get-code 0xa3078bf607d2e859dca0b1a13878ec2e607f30de || error_exit
aurora-cli key-pair --seed 1 || error_exit
block_hash=$(aurora-cli --network mainnet get-block-hash 88300374 || error_exit)
assert_eq "$block_hash" "0xd31857e9ce14083a7a74092b71f9ac48b8c0d4988ad40074182c1f0ffa296ec5"

# Register a new relayer address
aurora-cli --engine $ENGINE_ACCOUNT register-relayer 0xf644ad75e048eaeb28844dd75bd384984e8cd508 || error_exit
sleep 1

# Upgrading Aurora EVM to 2.9.0.
curl -sL $ENGINE_LAST_WASM_URL -o $ENGINE_WASM_PATH || error_exit
aurora-cli --engine $ENGINE_ACCOUNT stage-upgrade $ENGINE_WASM_PATH || error_exit
sleep 2
aurora-cli --engine $ENGINE_ACCOUNT deploy-upgrade || error_exit
sleep 1
version=$(aurora-cli --engine $ENGINE_ACCOUNT get-version || error_exit)
assert_eq "$version" $AURORA_LAST_VERSION
echo "$version"

# Set a new owner. The functionality has not been released yet.
aurora-cli --engine $ENGINE_ACCOUNT set-owner node0 || error_exit
sleep 1
owner=$(aurora-cli --engine $ENGINE_ACCOUNT get-owner || error_exit)
assert_eq "$owner" node0
export NEAR_KEY_PATH=$NODE_KEY_PATH
aurora-cli --engine $ENGINE_ACCOUNT set-owner aurora.node0 || error_exit
sleep 1
owner=$(aurora-cli --engine $ENGINE_ACCOUNT get-owner || error_exit)
assert_eq "$owner" $ENGINE_ACCOUNT

# Check pausing precompiles. Not working on the current release because of
# hardcoded aurora account in EngineAuthorizer.
export NEAR_KEY_PATH=$AURORA_KEY_PATH
mask=$(aurora-cli --engine $ENGINE_ACCOUNT paused-precompiles || error_exit)
assert_eq "$mask" 0
aurora-cli --engine $ENGINE_ACCOUNT pause-precompiles 1 || error_exit
sleep 1
mask=$(aurora-cli --engine $ENGINE_ACCOUNT paused-precompiles || error_exit)
assert_eq "$mask" 1
aurora-cli --engine $ENGINE_ACCOUNT pause-precompiles 2 || error_exit
sleep 1
mask=$(aurora-cli --engine $ENGINE_ACCOUNT paused-precompiles || error_exit)
assert_eq "$mask" 3
aurora-cli --engine $ENGINE_ACCOUNT resume-precompiles 3 || error_exit
sleep 1
mask=$(aurora-cli --engine $ENGINE_ACCOUNT paused-precompiles || error_exit)
assert_eq "$mask" 0

# XCC router operations.
# Download XCC router contract.
curl -sL $XCC_ROUTER_LAST_WASM_URL -o $XCC_ROUTER_WASM_PATH || error_exit
aurora-cli --engine $ENGINE_ACCOUNT factory-update $XCC_ROUTER_WASM_PATH || error_exit
sleep 1
aurora-cli --engine $ENGINE_ACCOUNT factory-set-wnear-address 0x80c6a002756e29b8bf2a587f7d975a726d5de8b9 || error_exit
sleep 1
aurora-cli --engine $ENGINE_ACCOUNT fund-xcc-sub-account 0x43a4969cc2c22d0000c591ff4bd71983ea8a8be9 some_account.near 25.5 || error_exit

# Stop NEAR node and clean up.
finish
