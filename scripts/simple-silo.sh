#!/usr/bin/env bash

export NEARCORE_HOME="/tmp/localnet"

EVM_CODE=$(cat docs/res/Counter.hex)
ABI_PATH=docs/res/Counter.abi
ENGINE_WASM_PATH="docs/res/aurora-mainnet-silo.wasm"
NODE_KEY_PATH=$NEARCORE_HOME/node0/validator_key.json
AURORA_KEY_PATH=$NEARCORE_HOME/node0/aurora_key.json
AURORA_SECRET_KEY=27cb3ddbd18037b38d7fb9ae3433a9d6f5cd554a4ba5768c8a15053f688ee167
ENGINE_ACCOUNT=aurora.node0
VENV=/tmp/venv


# Install `nearup` utility if not installed before.
python3 -m venv $VENV
source $VENV/bin/activate
pip list | grep nearup > /dev/null || pip install nearup > /dev/null

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

assert_eq() {
  if [[ $1 != $2 ]]; then
    echo "Unexpected result, should be $1 but actual is $2"
    finish 1
  fi
}

wait_for_block() {
  sleep 1.5
}

# Start NEAR node.
start_node
sleep 2

export NEAR_KEY_PATH=$NODE_KEY_PATH
# Create an account for Aurora EVM.
aurora-cli create-account --account $ENGINE_ACCOUNT --balance 100 > $AURORA_KEY_PATH || error_exit
wait_for_block

# View info of created account.
aurora-cli view-account $ENGINE_ACCOUNT || error_exit
wait_for_block

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

# Prerequisites for the start-hashchain command: Change the key manager of ENGINE_ACCOUNT to ENGINE_ACCOUNT
aurora-cli --engine $ENGINE_ACCOUNT set-key-manager $ENGINE_ACCOUNT || error_exit
wait_for_block

# Prerequisites for the start-hashchain command: The contract must be paused
aurora-cli --engine $ENGINE_ACCOUNT pause-contract
wait_for_block

# Start Hashchain. The aurora-engine will resume the contract automatically
aurora-cli --engine $ENGINE_ACCOUNT start-hashchain --block-height 0 --block-hashchain 0000000000000000000000000000000000000000000000000000000000000000
wait_for_block

# Set eth connector params
aurora-cli --engine $ENGINE_ACCOUNT set-eth-connector-contract-account --account-id 0f2bb8ad44996a7762256477fe3b42b27e63a00d.factory.bridge.near --withdraw-ser borsh
wait_for_block

# Silo methods
# Get fixed gas
result=$(aurora-cli --engine $ENGINE_ACCOUNT get-fixed-gas || error_exit)
assert_eq "none" "$result"
# Set fixed gas
aurora-cli --engine $ENGINE_ACCOUNT set-silo-params --gas 0 --fallback-address \
  0x7e5f4552091a69125d5dfcb7b8c2659029395bdf || error_exit
wait_for_block
# Get fixed gas
result=$(aurora-cli --engine $ENGINE_ACCOUNT get-fixed-gas || error_exit)
assert_eq "0" "$result"

# Check whitelists statuses
result=$(aurora-cli --engine $ENGINE_ACCOUNT get-whitelist-status admin || error_exit)
assert_eq "1" "$result"
result=$(aurora-cli --engine $ENGINE_ACCOUNT get-whitelist-status evm-admin || error_exit)
assert_eq "1" "$result"
result=$(aurora-cli --engine $ENGINE_ACCOUNT get-whitelist-status account || error_exit)
assert_eq "1" "$result"
result=$(aurora-cli --engine $ENGINE_ACCOUNT get-whitelist-status address || error_exit)
assert_eq "1" "$result"

# Add whitelist batch
aurora-cli --engine $ENGINE_ACCOUNT add-entry-to-whitelist-batch docs/res/batch_list.json || error_exit
wait_for_block

# Add address into EvmAdmin whitelist to allow deploy EVM code
aurora-cli --engine $ENGINE_ACCOUNT add-entry-to-whitelist --kind evm-admin \
  --entry 0xF388d9622737637cf0a80Bbd378e0b4D797a87c9 || error_exit
wait_for_block
# Add account into Admin whitelist to allow deploy EVM code
aurora-cli --engine $ENGINE_ACCOUNT add-entry-to-whitelist --kind admin --entry $ENGINE_ACCOUNT || error_exit
wait_for_block

# Deploy Counter EVM code.
aurora-cli --engine $ENGINE_ACCOUNT deploy --code $EVM_CODE --abi-path $ABI_PATH --args '{"init_value":"5"}' \
  --aurora-secret-key $AURORA_SECRET_KEY || error_exit
wait_for_block
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value \
  --abi-path $ABI_PATH --from 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D || error_exit)
assert_eq "$result" "5"
wait_for_block

# Add address into Address whitelist to allow submit transactions
aurora-cli --engine $ENGINE_ACCOUNT add-entry-to-whitelist --kind address \
  --entry 0x04B678962787cCD195a8e324d4C6bc4d5727F82B || error_exit
wait_for_block

# Add account into Account whitelist to allow submit transactions
aurora-cli --engine $ENGINE_ACCOUNT add-entry-to-whitelist --kind account --entry $ENGINE_ACCOUNT || error_exit
wait_for_block

# Submit increment transactions
aurora-cli --engine $ENGINE_ACCOUNT call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f increment \
  --abi-path $ABI_PATH \
  --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c || error_exit
sleep 2

# Check result
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value \
  --abi-path $ABI_PATH --from 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D || error_exit)
assert_eq "$result" "6"
wait_for_block

# Submit decrement transaction
aurora-cli --engine $ENGINE_ACCOUNT call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f decrement \
  --abi-path $ABI_PATH \
  --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c || error_exit
wait_for_block

# Check result
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value \
  --abi-path $ABI_PATH --from 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D  || error_exit)
assert_eq "$result" "5"
wait_for_block

# Remove entries from Account and Address whitelists.
aurora-cli --engine $ENGINE_ACCOUNT remove-entry-from-whitelist --kind address \
  --entry 0x04B678962787cCD195a8e324d4C6bc4d5727F82B || error_exit
wait_for_block

aurora-cli --engine $ENGINE_ACCOUNT remove-entry-from-whitelist --kind account --entry $ENGINE_ACCOUNT || error_exit
wait_for_block

# Disable Account and Address whitelists to allow to submit transactions to anyone.
aurora-cli --engine $ENGINE_ACCOUNT set-whitelist-status --kind account --status 0 || error_exit
wait_for_block
aurora-cli --engine $ENGINE_ACCOUNT set-whitelist-status --kind address --status 0 || error_exit
wait_for_block

# Submit decrement transaction
aurora-cli --engine $ENGINE_ACCOUNT call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f decrement \
  --abi-path $ABI_PATH \
  --aurora-secret-key 591f4a18a51779f76ecb5943cb6b6e73bf5877520511b7209a342c176295805b || error_exit
wait_for_block

# Check result
result=$(aurora-cli --engine $ENGINE_ACCOUNT view-call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value \
  --abi-path $ABI_PATH --from 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D || error_exit)
assert_eq "$result" "4"
wait_for_block

# Stop NEAR node and clean up.
finish
