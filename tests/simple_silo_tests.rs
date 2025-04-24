mod common;

#[cfg(feature = "simple")]
mod simple_silo_tests {
	//use super::*;
	use super::common::*;
	use cmd_lib::run_cmd;
	use cmd_lib::run_fun;
	use cmd_lib::*;
	use serial_test::serial;
	
	use super::common::NEARCORE_HOME;
	use super::common::COUNTER_ABI_PATH;
	use super::common::ENGINE_SILO_WASM_PATH;
	use super::common::NODE_KEY_PATH;
	use super::common::AURORA_KEY_PATH;
	use super::common::AURORA_SECRET_KEY;
	use super::common::ENGINE_ACCOUNT;
	use super::common::get_counter_evm_code;
	
	
	
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
	fn test_01() -> Result<(), Box<dyn std::error::Error>> {
		
		let subcmd = "view-account";
		let ret1 = run_fun!($BIN_NAME $subcmd $ENGINE_ACCOUNT)?; //ok
		println!("---aurora.node0: {:?}", ret1);
		
		let node0_public_key = run_fun!(jq -r .public_key < $AURORA_KEY_PATH)?;
		println!("---node0_public_key: {}", node0_public_key);
		
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
	//#[ignore]
	fn test_02() -> Result<(), Box<dyn std::error::Error>> {
		//# Deploy Aurora EVM
		//export NEAR_KEY_PATH=$AURORA_KEY_PATH
		run_cmd!(
			echo "Deploy Aurora EVM...";
			$BIN_NAME --near-key-path $AURORA_KEY_PATH deploy-aurora $ENGINE_SILO_WASM_PATH;
			echo "done...";
			sleep 10;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_03() -> Result<(), Box<dyn std::error::Error>> {
		//# Initialize Aurora EVM
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
	
	
	
	/// SILO methods
	#[test]
	#[serial]
	fn test_04() -> Result<(), Box<dyn std::error::Error>> {
		//# Get fixed gas cost
		let result = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-fixed-gas-cost)?;
		println!("aurora fixed-gas-cost: {}", result);
		assert_eq!(result, "none");
		
		//# Set fixed gas cost
		run_cmd!(
			$BIN_NAME --engine $ENGINE_ACCOUNT set-silo-params --cost 0 --rollback-address 
				0x7e5f4552091a69125d5dfcb7b8c2659029395bdf;
		)?;
		
		//# Get fixed gas cost
		let result = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-fixed-gas-cost)?;
		assert_eq!(result, "0");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_05() -> Result<(), Box<dyn std::error::Error>> {
		//# Check whitelists statuses
		let result1 = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-whitelist-status admin)?;
		assert_eq!(result1, "1");
		
		let result2 = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-whitelist-status evm-admin)?;
		assert_eq!(result2, "1");
		
		let result3 = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-whitelist-status account)?;
		assert_eq!(result3, "1");
		
		let result4 = run_fun!($BIN_NAME --engine $ENGINE_ACCOUNT get-whitelist-status address)?;
		assert_eq!(result4, "1");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_06() -> Result<(), Box<dyn std::error::Error>> {
		//# Add whitelist batch
		run_cmd!(
			echo "add-entry-to-whitelist-batch...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH add-entry-to-whitelist-batch docs/res/batch_list.json;
			echo "done...";
			sleep 1;
		)?;
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_07() -> Result<(), Box<dyn std::error::Error>> {
		//# Add address into EvmAdmin whitelist to allow deploy EVM code
		run_cmd!(
			echo "add-entry-to-whitelist EvmAdmin...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH add-entry-to-whitelist --kind evm-admin 
				--entry 0xF388d9622737637cf0a80Bbd378e0b4D797a87c9;
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_08() -> Result<(), Box<dyn std::error::Error>> {
		//# Add account into Admin whitelist to allow deploy EVM code
		run_cmd!(
			echo "add-entry-to-whitelist Admin...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH add-entry-to-whitelist --kind admin --entry $ENGINE_ACCOUNT;
			echo "done...";
			sleep 1;
		)?;
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_09() -> Result<(), Box<dyn std::error::Error>> {		
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
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value --abi-path $COUNTER_ABI_PATH
		)?;
		assert_eq!(result, "5");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_10() -> Result<(), Box<dyn std::error::Error>> {
		//# Add address into Address whitelist to allow submit transactions
		run_cmd!(
			echo "Add address into Address whitelist to allow submit transactions...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH add-entry-to-whitelist 
				--kind address --entry 0x04B678962787cCD195a8e324d4C6bc4d5727F82B;
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_11() -> Result<(), Box<dyn std::error::Error>> {
		//# Add account into Account whitelist to allow submit transactions
		run_cmd!(
			echo "Add account into Account whitelist to allow submit transactions...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH add-entry-to-whitelist --kind account --entry $ENGINE_ACCOUNT;
			echo "done...";
			sleep 1;
		)?;
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_12() -> Result<(), Box<dyn std::error::Error>> {
		//# Submit increment transactions
		run_cmd!(
			echo "Submit increment transactions...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f increment 
				--abi-path $COUNTER_ABI_PATH --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c;
			echo "done...";
			sleep 1;
		)?;
		
		//# Check result
		let result = run_fun!(
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call 
			-a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value 
			--abi-path $COUNTER_ABI_PATH
		)?;
		assert_eq!(result, "6");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_13() -> Result<(), Box<dyn std::error::Error>> {
		//# Submit decrement transaction
		run_cmd!(
			echo "Submit decrement transaction...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f decrement 
				--abi-path $COUNTER_ABI_PATH --aurora-secret-key 611830d3641a68f94a690dcc25d1f4b0dac948325ac18f6dd32564371735f32c;
			echo "done...";
			sleep 1;
		)?;
		
		//# Check result
		let result = run_fun!(
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call 
			-a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value 
			--abi-path $COUNTER_ABI_PATH
		)?;
		assert_eq!(result, "5");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_14() -> Result<(), Box<dyn std::error::Error>> {
		//# Remove entries from Account and Address whitelists
		run_cmd!(
			echo "Remove entries from Account and Address whitelists...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH remove-entry-from-whitelist --kind address 
				--entry 0x04B678962787cCD195a8e324d4C6bc4d5727F82B;
			sleep 1;
			echo "Address removed...";
			
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH remove-entry-from-whitelist --kind account --entry $ENGINE_ACCOUNT;
			echo "Account removed...";
			sleep 1;
		)?;
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	fn test_15() -> Result<(), Box<dyn std::error::Error>> {
		//# Disable Account and Address whitelists to allow to submit transactions to anyone
		run_cmd!(
			echo "Disable Account and Address whitelists to allow to submit transactions to anyone...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH set-whitelist-status --kind account --status 0;
			sleep 1;
			echo "Account disabled...";
			
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH set-whitelist-status --kind address --status 0;
			sleep 1;
			echo "Address disabled...";
		)?;
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_16() -> Result<(), Box<dyn std::error::Error>> {
		//# Submit decrement transaction
		run_cmd!(
			echo "Submit decrement transaction...";
			$BIN_NAME --engine $ENGINE_ACCOUNT --near-key-path $AURORA_KEY_PATH call -a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f decrement 
				--abi-path $COUNTER_ABI_PATH --aurora-secret-key 591f4a18a51779f76ecb5943cb6b6e73bf5877520511b7209a342c176295805b;
			echo "done...";
			sleep 1;
		)?;
		
		//# Check result
		let result = run_fun!(
			$BIN_NAME --engine $ENGINE_ACCOUNT view-call 
			-a 0xa3078bf607d2e859dca0b1a13878ec2e607f30de -f value 
			--abi-path $COUNTER_ABI_PATH
		)?;
		assert_eq!(result, "4");
		
		Ok(())
	}
	
	
	
	#[test]
	#[serial]
	//#[ignore]
	fn test_17_cleanup() -> Result<(), Box<dyn std::error::Error>> {
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