pub use assert_cmd::prelude::*; // Add methods on commands
pub use predicates::prelude::*; // Used for writing assertions
pub use std::process::Command;  // Run programs
use cmd_lib::run_fun;

pub static BIN_NAME: &str = "aurora-cli";

// export NEARCORE_HOME="/tmp/localnet"
pub static NEARCORE_HOME: &str = "/tmp/localnet";

//static EVM_CODE: &str = &run_fun!(cat docs/res/HelloWorld.hex)?; //$(cat docs/res/HelloWorld.hex);
pub static ABI_PATH: &str = "docs/res/HelloWorld.abi";

//static COUNTER_EVM_CODE: &str = &run_fun!(cat docs/res/Counter.hex)?;
pub static COUNTER_ABI_PATH: &str = "docs/res/Counter.abi";

pub static ENGINE_SILO_WASM_PATH: &str = "docs/res/aurora-mainnet-silo.wasm";

pub static GENESIS_FILE_PATH: &str = "/tmp/localnet/node0/genesis.json";
pub static CONFIG_FILE_PATH: &str = "/tmp/localnet/node0/config.json";

pub static NODE_KEY_PATH: &str = "/tmp/localnet/node0/validator_key.json";
pub static AURORA_KEY_PATH: &str = "/tmp/localnet/node0/aurora_key.json";

pub static AURORA_SECRET_KEY: &str = "27cb3ddbd18037b38d7fb9ae3433a9d6f5cd554a4ba5768c8a15053f688ee167";
pub static ENGINE_ACCOUNT: &str = "aurora.node0";



pub static AURORA_PREV_VERSION: &str = "2.10.0";
pub static AURORA_LAST_VERSION: &str = "3.0.0";

pub static ENGINE_WASM_URL: &str = 			"https://github.com/aurora-is-near/aurora-engine/releases/download/3.0.0/aurora-mainnet.wasm";
pub static ENGINE_PREV_WASM_URL: &str = "https://github.com/aurora-is-near/aurora-engine/releases/download/2.10.0/aurora-mainnet.wasm"; //, AURORA_PREV_VERSION);
pub static ENGINE_LAST_WASM_URL: &str = "https://github.com/aurora-is-near/aurora-engine/releases/download/3.0.0/aurora-mainnet.wasm"; //, AURORA_LAST_VERSION);

pub static XCC_ROUTER_LAST_WASM_URL: &str = "https://github.com/aurora-is-near/aurora-engine/releases/download/3.0.0/aurora-factory-mainnet.wasm"; //, AURORA_LAST_VERSION);

pub static ENGINE_WASM_PATH: &str = "/tmp/aurora-mainnet.wasm";
pub static XCC_ROUTER_WASM_PATH: &str = "/tmp/aurora-factory-mainnet.wasm";
//static USER_BASE_BIN: &str = $(python3 -m site --user-base)/bin
//pub static NODE_KEY_PATH: &str = "/tmp/localnet/node0/validator_key.json"; //, NEARCORE_HOME);
//pub static AURORA_KEY_PATH: &str = "/tmp/localnet/node0/aurora_key.json"; //, NEARCORE_HOME);
pub static MANAGER_KEY_PATH: &str = "/tmp/localnet/node0/manager_key.json"; //, NEARCORE_HOME);
pub static RELAYER_KEY_PATH: &str = "/tmp/localnet/node0/relayer_key.json"; //, NEARCORE_HOME);
//pub static AURORA_SECRET_KEY: &str = "27cb3ddbd18037b38d7fb9ae3433a9d6f5cd554a4ba5768c8a15053f688ee167";
//pub static ENGINE_ACCOUNT: &str = "aurora.node0";
// the following line for MANAGER_ACCOUNT throws error: Non-sub accounts could be created for mainnet or testnet only
//static MANAGER_ACCOUNT: &str = "key-manager.aurora.node0"; //# NOTE: use key-manager-aurora.node0 instead of key-manager.aurora.node0
pub static MANAGER_ACCOUNT: &str = "key-manager-aurora.node0";



pub fn get_evm_code() -> String {
	//$(cat docs/res/HelloWorld.hex);
	let code = run_fun!(cat docs/res/HelloWorld.hex).unwrap();
	String::from(code)
}

pub fn get_counter_evm_code() -> String {
	//$(cat docs/res/Counter.hex);
	let code = run_fun!(cat docs/res/Counter.hex).unwrap();
	String::from(code)
}

pub fn get_relayer_public_key() -> String {
	let relayer_public_key = run_fun!(jq -r .public_key < $RELAYER_KEY_PATH).unwrap();
	String::from(relayer_public_key)
}



/// setup Command with given args from string
pub fn setup_cmd(test_args: &str) -> Command {
   let args: Vec<&str> = test_args.split_whitespace().collect();
   println!("---args: {:?}", args);
   
   //let mut cmd = Command::cargo_bin(BIN_NAME)?;
   let mut cmd = Command::cargo_bin(BIN_NAME).unwrap();
   for arg1 in args {
      cmd.arg(arg1);
   }
   cmd
}
