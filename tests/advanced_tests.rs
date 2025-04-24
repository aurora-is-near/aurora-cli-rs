mod common;

#[cfg(feature = "advanced")]
mod advanced_tests {
	use super::common::*;
	use cmd_lib::run_cmd;
	use cmd_lib::run_fun;
	use cmd_lib::*;
	use serial_test::serial;
	
	// export NEARCORE_HOME="/tmp/localnet"
	//static NEARCORE_HOME: &str = "/tmp/localnet";
	
	// static EVM_CODE = &run_fun!(cat docs/res/Counter.hex)?;
	//static COUNTER_ABI_PATH: &str = "docs/res/Counter.abi";
	//static ENGINE_SILO_WASM_PATH: &str = "docs/res/aurora-mainnet-silo.wasm";
	//static NODE_KEY_PATH: &str = "/tmp/localnet/node0/validator_key.json";
	//static AURORA_KEY_PATH: &str = "/tmp/localnet/node0/aurora_key.json";
	//static AURORA_SECRET_KEY: &str = "27cb3ddbd18037b38d7fb9ae3433a9d6f5cd554a4ba5768c8a15053f688ee167";
	//static ENGINE_ACCOUNT: &str = "aurora.node0";
	
	// static EVM_CODE=$(cat docs/res/HelloWorld.hex)
	//static ABI_PATH="docs/res/HelloWorld.abi";
	// AURORA_LAST_VERSION="2.9.2"
	//AURORA_LAST_VERSION="3.0.0"
	//ENGINE_WASM_URL="https://github.com/aurora-is-near/aurora-engine/releases/download/3.0.0/aurora-mainnet.wasm"
	//ENGINE_WASM_PATH="/tmp/aurora-mainnet.wasm"
	// USER_BASE_BIN=$(python3 -m site --user-base)/bin
	
	//export PATH="$PATH:$USER_BASE_BIN:$HOME/.cargo/bin"
	//export NEARCORE_HOME="/tmp/localnet"
	
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn init_0_logger() -> Result<(), Box<dyn std::error::Error>> {
		// init logger
		init_builtin_logger();
		Ok(())
	}
	
	
	
	#[test]
	//#[serial("ordered init fn 0")] //new serial group based on given text
	#[serial]
	//#[ignore]
	fn init_1() -> Result<(), Box<dyn std::error::Error>> {
		//# stop near node first
		run_cmd!(
			echo "stop nearup node";
			nearup stop;
			rm -rf $NEARCORE_HOME/node0/data;
			echo "done...";
		)?;
		Ok(())
	}
	
	
	
	#[test]
	//#[serial("ordered init fn 1")]
	#[serial]
	//#[ignore]
	fn init_2() -> Result<(), Box<dyn std::error::Error>> {
		//# Update configs and add aurora key.
		run_cmd!(
			echo "Update configs and add aurora key...";
			$BIN_NAME near init genesis --path $GENESIS_FILE_PATH;
			echo "genesis done...";
			sleep 1;
			$BIN_NAME near init local-config -n $CONFIG_FILE_PATH -a $AURORA_KEY_PATH;
			echo "config done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn init_3() -> Result<(), Box<dyn std::error::Error>> {
		// Start NEAR node:
		//# nearup run localnet --home $NEARCORE_HOME --num-nodes 1
		run_cmd!(
			echo "nearup run...";
			nearup run localnet --home $NEARCORE_HOME --num-nodes 1;
			sleep 20;
			echo "done...";
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_00() -> Result<(), Box<dyn std::error::Error>> {
		//# Download Aurora EVM.
		run_cmd!(
			echo "Download Aurora EVM...";
			curl -sL $ENGINE_WASM_URL -o $ENGINE_WASM_PATH;
			sleep 15;
			echo "done...";
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_01() -> Result<(), Box<dyn std::error::Error>> {
		//# Deploy and init Aurora EVM smart contract.
		run_cmd!(
			echo "Deploy and init Aurora EVM smart contract...";
			$BIN_NAME near write engine-init -w $ENGINE_WASM_PATH;
			sleep 2;
		)?;
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_02() -> Result<(), Box<dyn std::error::Error>> {
		//# Deploy EVM code
		//evm_code = $(cat docs/res/HelloWorld.hex)
		let EVM_CODE: &str = &get_evm_code().to_string();
		run_cmd!(
			echo "Deploy EVM code...";
			$BIN_NAME near write deploy-code $EVM_CODE;
			echo "done...";
			sleep 2;
		)?;
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_03() -> Result<(), Box<dyn std::error::Error>> {
		//# Run EVM view call
		run_cmd!(
			echo "Run EVM view call...";
			$BIN_NAME near read solidity -t 0x592186c059e3d9564cac6b1ada6f2dc7ff1d78e9 call-args-by-name 
				--abi-path $ABI_PATH -m "greet" --arg "{}";
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_04_cleanup() -> Result<(), Box<dyn std::error::Error>> {
		//nearup stop;
		//rm -rf $NEARCORE_HOME;
		run_cmd!(
			echo "nearup stop node...";
			nearup stop;
			echo "Cleanup...";
			rm -rf $NEARCORE_HOME;
			echo "finished.";
		)?;
		Ok(())
	}
}