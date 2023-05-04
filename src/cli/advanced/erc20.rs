use crate::utils;
use aurora_engine_types::U256;
use clap::Subcommand;

const APPROVE_SELECTOR: &[u8] = &[0x09, 0x5e, 0xa7, 0xb3];
const BALANCE_OF_SELECTOR: &[u8] = &[0x70, 0xa0, 0x82, 0x31];
const TOTAL_SUPPLY_SELECTOR: &[u8] = &[0x18, 0x16, 0x0d, 0xdd];
const TRANSFER_SELECTOR: &[u8] = &[0xa9, 0x05, 0x9c, 0xbb];
const ALLOWANCE_SELECTOR: &[u8] = &[0xdd, 0x62, 0xed, 0x3e];
const TRANSFER_FROM_SELECTOR: &[u8] = &[0x23, 0xb8, 0x72, 0xdd];

#[derive(Subcommand)]
pub enum Erc20 {
    TotalSupply,
    BalanceOf {
        address_hex: String,
    },
    Transfer {
        #[clap(short, long)]
        to_address_hex: String,
        #[clap(short, long)]
        amount: String,
    },
    Allowance {
        #[clap(short, long)]
        owner_address_hex: String,
        #[clap(short, long)]
        spender_address_hex: String,
    },
    Approve {
        #[clap(short, long)]
        spender_address_hex: String,
        #[clap(short, long)]
        amount: String,
    },
    TransferFrom {
        #[clap(short, long)]
        from_address_hex: String,
        #[clap(short, long)]
        to_address_hex: String,
        #[clap(short, long)]
        amount: String,
    },
}

impl Erc20 {
    pub fn abi_encode(self) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::TotalSupply => Ok(TOTAL_SUPPLY_SELECTOR.to_vec()),
            Self::Transfer {
                to_address_hex,
                amount,
            } => {
                let to = utils::hex_to_address(&to_address_hex)?;
                let amount = U256::from_str_radix(&amount, 10)?;
                let input = [
                    TRANSFER_SELECTOR,
                    &ethabi::encode(&[
                        ethabi::Token::Address(to.raw()),
                        ethabi::Token::Uint(amount),
                    ]),
                ]
                .concat();
                Ok(input)
            }
            Self::Allowance {
                owner_address_hex,
                spender_address_hex,
            } => {
                let spender = utils::hex_to_address(&spender_address_hex)?;
                let owner = utils::hex_to_address(&owner_address_hex)?;
                let input = [
                    ALLOWANCE_SELECTOR,
                    &ethabi::encode(&[
                        ethabi::Token::Address(owner.raw()),
                        ethabi::Token::Address(spender.raw()),
                    ]),
                ]
                .concat();
                Ok(input)
            }
            Self::Approve {
                spender_address_hex,
                amount,
            } => {
                let spender = utils::hex_to_address(&spender_address_hex)?;
                let amount = U256::from_str_radix(&amount, 10)?;
                let input = [
                    APPROVE_SELECTOR,
                    &ethabi::encode(&[
                        ethabi::Token::Address(spender.raw()),
                        ethabi::Token::Uint(amount),
                    ]),
                ]
                .concat();
                Ok(input)
            }
            Self::BalanceOf { address_hex } => {
                let address = utils::hex_to_address(&address_hex)?;
                let input = [
                    BALANCE_OF_SELECTOR,
                    &ethabi::encode(&[ethabi::Token::Address(address.raw())]),
                ]
                .concat();
                Ok(input)
            }
            Self::TransferFrom {
                to_address_hex,
                amount,
                from_address_hex,
            } => {
                let from = utils::hex_to_address(&from_address_hex)?;
                let to = utils::hex_to_address(&to_address_hex)?;
                let amount = U256::from_str_radix(&amount, 10)?;
                let input = [
                    TRANSFER_FROM_SELECTOR,
                    &ethabi::encode(&[
                        ethabi::Token::Address(from.raw()),
                        ethabi::Token::Address(to.raw()),
                        ethabi::Token::Uint(amount),
                    ]),
                ]
                .concat();
                Ok(input)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use aurora_engine_types::{H160, U256};
    use rand::Rng;

    #[test]
    fn test_total_supply_encoding() {
        #[allow(deprecated)]
        let total_supply_function = ethabi::Function {
            name: "totalSupply".into(),
            inputs: vec![],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ethabi::ParamType::Uint(256),
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::View,
        };
        let expected_tx_data = total_supply_function.encode_input(&[]).unwrap();

        assert_eq!(
            super::Erc20::TotalSupply.abi_encode().unwrap(),
            expected_tx_data
        );
    }

    #[test]
    fn test_balance_of_encoding() {
        let mut rng = rand::thread_rng();
        let address: [u8; 20] = rng.gen();

        let address = H160(address);

        #[allow(deprecated)]
        let balance_of_function = ethabi::Function {
            name: "balanceOf".into(),
            inputs: vec![ethabi::Param {
                name: "account".into(),
                kind: ethabi::ParamType::Address,
                internal_type: None,
            }],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ethabi::ParamType::Uint(256),
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::View,
        };
        let expected_tx_data = balance_of_function
            .encode_input(&[ethabi::Token::Address(address)])
            .unwrap();

        assert_eq!(
            super::Erc20::BalanceOf {
                address_hex: hex::encode(address.as_bytes())
            }
            .abi_encode()
            .unwrap(),
            expected_tx_data
        );
    }

    #[test]
    fn test_transfer_encoding() {
        let mut rng = rand::thread_rng();
        let address: [u8; 20] = rng.gen();
        let amount: [u8; 32] = rng.gen();

        let address = H160(address);
        let amount = U256::from_big_endian(&amount);

        #[allow(deprecated)]
        let transfer_function = ethabi::Function {
            name: "transfer".into(),
            inputs: vec![
                ethabi::Param {
                    name: "to".into(),
                    kind: ethabi::ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "amount".into(),
                    kind: ethabi::ParamType::Uint(256),
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ethabi::ParamType::Bool,
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::NonPayable,
        };
        let expected_tx_data = transfer_function
            .encode_input(&[ethabi::Token::Address(address), ethabi::Token::Uint(amount)])
            .unwrap();

        assert_eq!(
            super::Erc20::Transfer {
                to_address_hex: hex::encode(address.as_bytes()),
                amount: amount.to_string()
            }
            .abi_encode()
            .unwrap(),
            expected_tx_data
        );
    }

    #[test]
    fn test_allowance_encoding() {
        let mut rng = rand::thread_rng();
        let owner: [u8; 20] = rng.gen();
        let spender: [u8; 20] = rng.gen();

        let owner = H160(owner);
        let spender = H160(spender);

        #[allow(deprecated)]
        let allowance_function = ethabi::Function {
            name: "allowance".into(),
            inputs: vec![
                ethabi::Param {
                    name: "owner".into(),
                    kind: ethabi::ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "spender".into(),
                    kind: ethabi::ParamType::Address,
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ethabi::ParamType::Uint(256),
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::View,
        };
        let expected_tx_data = allowance_function
            .encode_input(&[
                ethabi::Token::Address(owner),
                ethabi::Token::Address(spender),
            ])
            .unwrap();

        assert_eq!(
            super::Erc20::Allowance {
                owner_address_hex: hex::encode(owner.as_bytes()),
                spender_address_hex: hex::encode(spender.as_bytes()),
            }
            .abi_encode()
            .unwrap(),
            expected_tx_data
        );
    }

    #[test]
    fn test_approve_encoding() {
        let mut rng = rand::thread_rng();
        let address: [u8; 20] = rng.gen();
        let amount: [u8; 32] = rng.gen();

        let address = H160(address);
        let amount = U256::from_big_endian(&amount);

        #[allow(deprecated)]
        let approve_function = ethabi::Function {
            name: "approve".into(),
            inputs: vec![
                ethabi::Param {
                    name: "spender".into(),
                    kind: ethabi::ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "amount".into(),
                    kind: ethabi::ParamType::Uint(256),
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ethabi::ParamType::Bool,
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::NonPayable,
        };
        let expected_tx_data = approve_function
            .encode_input(&[ethabi::Token::Address(address), ethabi::Token::Uint(amount)])
            .unwrap();

        assert_eq!(
            super::Erc20::Approve {
                spender_address_hex: hex::encode(address.as_bytes()),
                amount: amount.to_string()
            }
            .abi_encode()
            .unwrap(),
            expected_tx_data
        );
    }

    #[test]
    fn test_transfer_from_encoding() {
        let mut rng = rand::thread_rng();
        let from: [u8; 20] = rng.gen();
        let to: [u8; 20] = rng.gen();
        let amount: [u8; 32] = rng.gen();

        let from = H160(from);
        let to = H160(to);
        let amount = U256::from_big_endian(&amount);

        #[allow(deprecated)]
        let transfer_from_function = ethabi::Function {
            name: "transferFrom".into(),
            inputs: vec![
                ethabi::Param {
                    name: "from".into(),
                    kind: ethabi::ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "to".into(),
                    kind: ethabi::ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "amount".into(),
                    kind: ethabi::ParamType::Uint(256),
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ethabi::ParamType::Bool,
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::NonPayable,
        };
        let expected_tx_data = transfer_from_function
            .encode_input(&[
                ethabi::Token::Address(from),
                ethabi::Token::Address(to),
                ethabi::Token::Uint(amount),
            ])
            .unwrap();

        assert_eq!(
            super::Erc20::TransferFrom {
                from_address_hex: hex::encode(from.as_bytes()),
                to_address_hex: hex::encode(to.as_bytes()),
                amount: amount.to_string()
            }
            .abi_encode()
            .unwrap(),
            expected_tx_data
        );
    }
}
