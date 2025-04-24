mod common;

#[cfg(feature = "simple")]
mod simple_tests {
	
	use super::common::*;
	use cmd_lib::run_cmd;
	use cmd_lib::run_fun;
	use cmd_lib::*;
	use serial_test::serial;
	
	use super::common::NEARCORE_HOME; //"/tmp/localnet";

	use super::common::AURORA_PREV_VERSION; //"2.10.0";
	use super::common::AURORA_LAST_VERSION; //"3.0.0";
	use super::common::ABI_PATH; //"docs/res/HelloWorld.abi";
	
	// static COUNTER_EVM_CODE: &str = &run_fun!(cat docs/res/Counter.hex)?;
	use super::common::COUNTER_ABI_PATH; //"docs/res/Counter.abi";
	
	use super::common::ENGINE_PREV_WASM_URL; //"https://github.com/aurora-is-near/aurora-engine/releases/download/2.10.0/aurora-mainnet.wasm"; //, AURORA_PREV_VERSION);
	use super::common::ENGINE_LAST_WASM_URL; //"https://github.com/aurora-is-near/aurora-engine/releases/download/3.0.0/aurora-mainnet.wasm"; //, AURORA_LAST_VERSION);
	use super::common::XCC_ROUTER_LAST_WASM_URL; //"https://github.com/aurora-is-near/aurora-engine/releases/download/3.0.0/aurora-factory-mainnet.wasm"; //, AURORA_LAST_VERSION);
	use super::common::ENGINE_WASM_PATH; //"/tmp/aurora-mainnet.wasm";
	use super::common::XCC_ROUTER_WASM_PATH; //"/tmp/aurora-factory-mainnet.wasm";
	use super::common::NODE_KEY_PATH; //"/tmp/localnet/node0/validator_key.json"; //, NEARCORE_HOME);
	use super::common::AURORA_KEY_PATH; //"/tmp/localnet/node0/aurora_key.json"; //, NEARCORE_HOME);
	use super::common::MANAGER_KEY_PATH; //"/tmp/localnet/node0/manager_key.json"; //, NEARCORE_HOME);
	use super::common::RELAYER_KEY_PATH; //"/tmp/localnet/node0/relayer_key.json"; //, NEARCORE_HOME);
	use super::common::AURORA_SECRET_KEY; //"27cb3ddbd18037b38d7fb9ae3433a9d6f5cd554a4ba5768c8a15053f688ee167";
	use super::common::ENGINE_ACCOUNT; //"aurora.node0";
	// the following line for MANAGER_ACCOUNT throws error: Non-sub accounts could be created for mainnet or testnet only
	//# NOTE: use key-manager-aurora.node0 instead of key-manager.aurora.node0
	use super::common::MANAGER_ACCOUNT; //"key-manager-aurora.node0";
	
	use super::common::get_evm_code;
	use super::common::get_counter_evm_code;
	use super::common::get_relayer_public_key;
	
	
	
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
		//EVM_CODE = run_fun!(cat docs/res/HelloWorld.hex)?;
		//println!("---init_0.EVM_CODE: {:?}", cfg.EVM_CODE);
		let EVM_CODE: &str = &get_evm_code().to_string();
		println!("---init_0.EVM_CODE: {:?}", EVM_CODE);
		run_cmd!(echo "---EVM_CODE: " $EVM_CODE)?;
		run_cmd!(
			echo "---EVM_CODE: " "$EVM_CODE";
			echo "sleep 5 sec";
			sleep 5;
			echo "done...";
		)?;
		
		let counter_evm_code: &str = &get_counter_evm_code().to_string();
		println!("---init_0.counter_evm_code: {:?}", counter_evm_code);
		run_cmd!(echo "---counter_evm_code: " $counter_evm_code)?;
		
		//let NODE_KEY_PATH = cfg.NODE_KEY_PATH;
		run_cmd!(echo "NODE_KEY_PATH:" $NODE_KEY_PATH)?;
		run_cmd!(echo "NODE_KEY_PATH:" "$NODE_KEY_PATH")?;
		println!("---init_0.end---");
		Ok(())
	}
	
	
	
	#[test]
	//#[serial("ordered init fn 1")]
	#[serial]
	//#[ignore]
	fn init_2() -> Result<(), Box<dyn std::error::Error>> {
		// Start NEAR node:
		run_cmd!(
			echo "nearup stop...";
			nearup stop;
			echo "done...";
			sleep 1;
			echo "nearup run...";
			nearup run localnet --home $NEARCORE_HOME;
			sleep 20;
			echo "done...";
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_00() -> Result<(), Box<dyn std::error::Error>> {
		//# Create an account for Aurora EVM.
		run_cmd!(
			echo "Create an account for Aurora EVM...";
			$BIN_NAME --near-key-path $NODE_KEY_PATH create-account 
				--account $ENGINE_ACCOUNT --balance 100 > $AURORA_KEY_PATH;
			sleep 1;
			echo "done...";
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_01() -> Result<(), Box<dyn std::error::Error>> {
		//let test_args = "--near-key-path /tmp/localnet/node1/validator_key.json create-account \
		//--account aurora.node1 --balance 100";
		let test_args = "view-account aurora.node0";
		let mut cmd = setup_cmd(test_args);
		//cmd.arg("> /tmp/localnet/aurora_key1.json"); //FAILS
		
		//cmd.assert().success();
		//cmd.output()
		//	.expect("my_string");
		
		// cmd returns this:
		// Ok(Output { status: ExitStatus(unix_wait_status(0)), stdout: "", stderr: "Action #0: Can't create a new account AccountId(\"aurora.node1\"), because it already exists\n" })
		
		//let outinfo = cmd.output()?.stdout; //output();
		//let outinfo = cmd.output().stdout; //output();
		//let output1 = cmd.output().expect("failed to execute process");
		let output1 = cmd.output()?;
		println!("---output1: {:?}", output1);
		
		run_cmd!(
			echo "testing test_1...";
			sleep 3;
			echo "done.";
		)?;
		
		//println!("---output1.status: {:?}", output1.status);
		if output1.status.success() {
			//println!("---output1.stdout: {:?}", String::from_utf8_lossy(&output1.stdout));
			println!("---output1.stdout: {:?}", String::from_utf8(output1.stdout)?);
		}
		else { //err
			println!("---output1.stderr: {:?}", String::from_utf8_lossy(&output1.stderr));
		}
		Ok(())
  }
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_02() -> Result<(), Box<dyn std::error::Error>> {
		//let test_args = "view-account aurora.node0";
		//cmd.arg("> /tmp/localnet/aurora_key1.json"); //FAILS
		//run_cmd!(aurora-cli view-account aurora.node0 > /tmp/localnet/aurora_key_temp.json)?;
		
		// using "run_cmd!" ret1 is: ()
		let ret1 = run_fun!(aurora-cli view-account aurora.node0)?;
		println!("---aurora.node0: {:?}", ret1);
		
		//let node0_public_key = run_fun!("jq -r .public_key < /tmp/localnet/node0/aurora_key.json")?;
		//let node0_public_key = run_fun!("jq -r .public_key < $AURORA_KEY_PATH")?; //FAILS
		let node0_public_key = run_fun!(jq -r .public_key < $AURORA_KEY_PATH)?;
		println!("---node0_public_key: {}", node0_public_key);
		
		//run_cmd!(echo $ret1 > /tmp/localnet/aurora_key_temp3.json)?;
		// run_cmd!(aurora-cli view-account aurora.node0 | echo)?;
		//let s1 = "demo string";
		//run_cmd!(echo $s1 > /tmp/localnet/aurora_key_temp4.json)?;
		
		Ok(())
  }
	
	
	
	#[test]
	#[serial]
	fn test_03() -> Result<(), Box<dyn std::error::Error>> {
		
		//# view_existing_account_test1
		//let ret1 = run_fun!($BIN_NAME view-account aurora.node0)?; //ok
		
		//let test_args = "view-account aurora.node0";
		//let ret1 = run_fun!($BIN_NAME $test_args)?; //FAILS
		
		//let cmd = format!("{} view-account aurora.node0", BIN_NAME);
		//let ret1 = run_fun!($cmd)?; //FAILS
		
		let subcmd = "view-account";
		//let arg = "aurora.node0";
		let ret1 = run_fun!($BIN_NAME $subcmd $ENGINE_ACCOUNT)?; //ok
		
		println!("---aurora.node0: {:?}", ret1);
		//run_cmd!(echo $ret1 > /tmp/localnet/aurora_key_temp3.json)?;
		
		assert!(ret1.contains("amount") 
			&& ret1.contains("locked") 
			&& ret1.contains("code_hash") 
			&& ret1.contains("storage_usage") 
			&& ret1.contains("storage_paid_at")
		);
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_04() -> Result<(), Box<dyn std::error::Error>> {
		
		//# view_non_existent_account_test1
		//init_builtin_logger();
		let ret1 = run_fun!($BIN_NAME view-account aurora.nodex)?;
		println!("---aurora.nodex: {:?}", ret1);
		assert_eq!(ret1, ""); //no data (empty string) for non existent account
		
		//assert!(run_cmd!($BIN_NAME view-account aurora.nodex).is_err()); //FAILS
		
		let mut proc = spawn_with_output!($BIN_NAME view-account aurora.nodex)?;
		let ret2 = proc.wait_with_output()?;
		println!("---[2]aurora.nodex: {:?}", ret2);
		assert_eq!(ret2, ""); //no data (empty string) for non existent account
		
		//stderr(predicate::str::contains("handler error: [account aurora.nodex does not exist while viewing]"
		
		Ok(())
  }
	
	
	
	#[test]
	#[serial]
	fn test_05() -> Result<(), Box<dyn std::error::Error>> {
		//# Download Aurora EVM
		run_cmd!(
			echo "Download Aurora EVM...";
			curl -sL $ENGINE_PREV_WASM_URL -o $ENGINE_WASM_PATH;
			sleep 15;
			echo "done...";
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_06() -> Result<(), Box<dyn std::error::Error>> {
		//# Deploy Aurora EVM
		//export NEAR_KEY_PATH=$AURORA_KEY_PATH
		//aurora-cli --near-key-path /tmp/localnet/node0/aurora_key.json deploy-aurora /tmp/aurora-mainnet.wasm || error_exit
		run_cmd!(
			echo "Deploy Aurora EVM...";
			$BIN_NAME --near-key-path $AURORA_KEY_PATH deploy-aurora $ENGINE_WASM_PATH;
			echo "done...";
			sleep 10;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_07() -> Result<(), Box<dyn std::error::Error>> {
		//# Initialize Aurora EVM
		//aurora-cli --engine aurora.node0 --near-key-path /tmp/localnet/node0/aurora_key.json init \
		//	--chain-id 1313161556 --owner-id aurora.node0
		//	--bridge-prover-id "prover" 
		//	--upgrade-delay-blocks 1 
		//	--custodian-address 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D 
		//	--ft-metadata-path docs/res/ft_metadata.json;
		
		run_cmd!(
			echo "Initialize Aurora EVM...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH init 
				--chain-id 1313161556 --owner-id $ENGINE_ACCOUNT 
				--bridge-prover-id "prover" 
				--upgrade-delay-blocks 1 
				--custodian-address 0x1B16948F011686AE64BB2Ba0477aeFA2Ea97084D 
				--ft-metadata-path docs/res/ft_metadata.json;
			echo "done...";
			sleep 10;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_08() -> Result<(), Box<dyn std::error::Error>> {
		//# Upgrading Aurora EVM to 3.0.0
		let version = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-version)?;
		println!("aurora version: {}", version);
		assert_eq!(version, AURORA_PREV_VERSION);
		
		run_cmd!(
			echo "download ENGINE_LAST_WASM_URL...";
			curl -sL $ENGINE_LAST_WASM_URL -o $ENGINE_WASM_PATH;
			sleep 15;
			echo "stage-upgrade...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH stage-upgrade $ENGINE_WASM_PATH;
			sleep 2;
			echo "deploy-upgrade...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH deploy-upgrade;
			echo "done...";
			sleep 1;
		)?;
		
		let version = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-version)?;
		println!("aurora last version: {}", version);
		assert_eq!(version, AURORA_LAST_VERSION);
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_09() -> Result<(), Box<dyn std::error::Error>> {
		//# Create account id for key manager
		run_cmd!(
			echo "create MANAGER_ACCOUNT...";
			$BIN_NAME --near-key-path $NODE_KEY_PATH create-account --account $MANAGER_ACCOUNT --balance 10 > $MANAGER_KEY_PATH;
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_10() -> Result<(), Box<dyn std::error::Error>> {
		//# Set key manager
		run_cmd!(
			echo "set-key-manager...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH set-key-manager $MANAGER_ACCOUNT;
			echo "done...";
			sleep 1;
		)?;
		// on success it prints out: The key manager key-manager-aurora.node0 has been set successfully
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_11() -> Result<(), Box<dyn std::error::Error>> {
		//# Create new keys for relayer
		run_cmd!(
			echo "generate-near-key for relayer...";
			$BIN_NAME generate-near-key $ENGINE_ACCOUNT ed25519 > $RELAYER_KEY_PATH;
			echo "done...";
			sleep 1;
		)?;
		
		//run_fun!(jq -r .public_key < $RELAYER_KEY_PATH)?;
		let relayer_public_key: &str = &get_relayer_public_key().to_string();
		println!("---relayer_public_key: {}", relayer_public_key);
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_12() -> Result<(), Box<dyn std::error::Error>> {
		//# Add relayer key by key manager
		let relayer_public_key: &str = &get_relayer_public_key().to_string();
		println!("---test13.relayer_public_key: {}", relayer_public_key);
		run_cmd!(
			echo "add-relayer-key...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $MANAGER_KEY_PATH add-relayer-key 
				--public-key "$relayer_public_key" --allowance "0.5";
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_13() -> Result<(), Box<dyn std::error::Error>> {
		//# Deploy Hello World EVM code with relayer key.
		//export NEAR_KEY_PATH=$RELAYER_KEY_PATH
		let EVM_CODE: &str = &get_evm_code().to_string();
		
		// --near-key-path AURORA_KEY_PATH
		run_cmd!(
			echo "Deploy Hello World EVM code with relayer key...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $RELAYER_KEY_PATH deploy 
				--code "$EVM_CODE" --aurora-secret-key $AURORA_SECRET_KEY;
			echo "done...";
			sleep 1;
		)?;
		
		let result = run_fun!(
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de 
			-f greet --abi-path $ABI_PATH
		)?;
		assert_eq!(result, "Hello, World!");
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_14() -> Result<(), Box<dyn std::error::Error>> {
		//# Remove relayer key
		//export NEAR_KEY_PATH=$MANAGER_KEY_PATH
		let relayer_public_key: &str = &get_relayer_public_key().to_string();
		run_cmd!(
			echo "remove-relayer-key...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $MANAGER_KEY_PATH remove-relayer-key "$relayer_public_key";
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_15() -> Result<(), Box<dyn std::error::Error>> {
		//# Deploy Counter EVM code.
		//export NEAR_KEY_PATH=$AURORA_KEY_PATH
		
		// EVM_CODE=$(cat docs/res/Counter.hex)
		// ABI_PATH=docs/res/Counter.abi
		let COUNTER_EVM_CODE: &str = &get_counter_evm_code().to_string();
		//let COUNTER_ABI_PATH: &str = "docs/res/Counter.abi";
		run_cmd!(
			echo "Deploy Counter EVM code...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH deploy 
				--code $COUNTER_EVM_CODE --abi-path $COUNTER_ABI_PATH --args "{\"init_value\":\"5\"}" 
				--aurora-secret-key $AURORA_SECRET_KEY;
			sleep 1;
			echo "done...";
		)?;
		
		let result = run_fun!(
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f value --abi-path $COUNTER_ABI_PATH
		)?;
		assert_eq!(result, "5");
		
		run_cmd!(
			sleep 1;
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f increment 
				--abi-path $COUNTER_ABI_PATH --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c;
			sleep 1;
		)?;
		
		let result = run_fun!(
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f value --abi-path $COUNTER_ABI_PATH
		)?;
		assert_eq!(result, "6");
		
		run_cmd!(
			sleep 1;
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f decrement
				--abi-path $COUNTER_ABI_PATH --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c;
			sleep 1;
		)?;
		
		let result = run_fun!(
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call -a 0x4cf003049d1a9c4918c73e9bf62464d904184555 -f value --abi-path $COUNTER_ABI_PATH
		)?;
		assert_eq!(result, "5");
		//run_cmd!(sleep 1)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_16() -> Result<(), Box<dyn std::error::Error>> {
		//# Check read operations
		let chain_id = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-chain-id)?;
		println!("---chain_id: {:?}", chain_id);
		
		let owner = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-owner)?;
		println!("---owner: {:?}", owner);
		assert_eq!(owner, ENGINE_ACCOUNT);
		
		let bridge_prover = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-bridge-prover)?;
		println!("---bridge_prover: {:?}", bridge_prover);
		
		let balance = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-balance 0x04b678962787ccd195a8e324d4c6bc4d5727f82b)?;
		println!("---balance: {:?}", balance);
		
		let code = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-code 0xa3078bf607d2e859dca0b1a13878ec2e607f30de)?;
		println!("---code: {:?}", code);
		
		let key_pair = run_fun!($BIN_NAME key-pair --seed 1)?;
		println!("---key_pair: {:?}", key_pair);
		
		let block_hash = run_fun!($BIN_NAME --network mainnet get-block-hash 88300374)?;
		println!("---block_hash: {:?}", block_hash);
		assert_eq!(block_hash, "0xd31857e9ce14083a7a74092b71f9ac48b8c0d4988ad40074182c1f0ffa296ec5");
		
		let block_hash_testnet = run_fun!($BIN_NAME --network testnet get-block-hash 88300374)?;
		println!("---block_hash_testnet: {:?}", block_hash_testnet);
		assert_eq!(block_hash_testnet, "0xb069f8e8a24dfd7fcfb6178db3c253d558b92a38394d3abab71524877025a597");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_17() -> Result<(), Box<dyn std::error::Error>> {
		//# Register a new relayer address
		run_cmd!(
			echo "register-relayer...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH register-relayer 0xf644ad75e048eaeb28844dd75bd384984e8cd508;
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_18() -> Result<(), Box<dyn std::error::Error>> {
		//# Set a new owner. The functionality has not been released yet ???
		run_cmd!(
			echo "set-owner node0...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH set-owner node0;
			echo "done...";
			sleep 1;
		)?;
		
		let owner = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-owner)?;
		assert_eq!(owner, "node0");
		
		//export NEAR_KEY_PATH=$NODE_KEY_PATH
		run_cmd!(
			echo "set-owner aurora.node0...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $NODE_KEY_PATH set-owner aurora.node0;
			echo "done...";
			sleep 1;
		)?;
		
		let owner = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-owner)?;
		assert_eq!(owner, ENGINE_ACCOUNT);
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_19() -> Result<(), Box<dyn std::error::Error>> {
		//# Check pausing precompiles. Not working on the current release because of
		//# hardcoded aurora account in EngineAuthorizer.
		//export NEAR_KEY_PATH=$AURORA_KEY_PATH
		let mask = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT paused-precompiles)?;
		assert_eq!(mask, "0");
		
		run_cmd!(
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH pause-precompiles 1;
			sleep 1;
		)?;
		// output msg: The precompiles have been paused successfully
		
		let mask = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT paused-precompiles)?;
		assert_eq!(mask, "1");
		
		run_cmd!(
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH pause-precompiles 2;
			sleep 1;
		)?;
		// output msg: The precompiles have been paused successfully
		
		let mask = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT paused-precompiles)?;
		assert_eq!(mask, "3");
		
		run_cmd!(
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH resume-precompiles 3;
			sleep 1;
		)?;
		// output msg: The precompiles have been resumed successfully
		
		let mask = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT paused-precompiles)?;
		assert_eq!(mask, "0");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_20() -> Result<(), Box<dyn std::error::Error>> {
		//# XCC router operations.
		//# Download XCC router contract.
		run_cmd!(
			echo "XCC router operations...";
			echo "Download XCC router contract...";
			curl -sL $XCC_ROUTER_LAST_WASM_URL -o $XCC_ROUTER_WASM_PATH;
			sleep 15;
			echo "factory-update...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH factory-update $XCC_ROUTER_WASM_PATH;
			sleep 1;
			echo "factory-set-wnear-address...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH factory-set-wnear-address 0x80c6a002756e29b8bf2a587f7d975a726d5de8b9;
			sleep 1;
			echo "fund-xcc-sub-account...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH fund-xcc-sub-account 0x43a4969cc2c22d0000c591ff4bd71983ea8a8be9 some_account.near 25.5;
		)?;
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_21_cleanup() -> Result<(), Box<dyn std::error::Error>> {
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