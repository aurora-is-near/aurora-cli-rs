use clap::Subcommand;
use ethabi::{Address, Token, Uint};
use std::str::FromStr;

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
                let to = Address::from_str(&to_address_hex)?;
                let amount = Uint::from_str_radix(&amount, 10)?;
                let input = [
                    TRANSFER_SELECTOR,
                    &ethabi::encode(&[Token::Address(to), Token::Uint(amount)]),
                ]
                .concat();
                Ok(input)
            }
            Self::Allowance {
                owner_address_hex,
                spender_address_hex,
            } => {
                let spender = Address::from_str(&spender_address_hex)?;
                let owner = Address::from_str(&owner_address_hex)?;
                let input = [
                    ALLOWANCE_SELECTOR,
                    &ethabi::encode(&[Token::Address(owner), Token::Address(spender)]),
                ]
                .concat();
                Ok(input)
            }
            Self::Approve {
                spender_address_hex,
                amount,
            } => {
                let spender = Address::from_str(&spender_address_hex)?;
                let amount = Uint::from_str_radix(&amount, 10)?;
                let input = [
                    APPROVE_SELECTOR,
                    &ethabi::encode(&[Token::Address(spender), Token::Uint(amount)]),
                ]
                .concat();
                Ok(input)
            }
            Self::BalanceOf { address_hex } => {
                let address = Address::from_str(&address_hex)?;
                let input = [
                    BALANCE_OF_SELECTOR,
                    &ethabi::encode(&[Token::Address(address)]),
                ]
                .concat();
                Ok(input)
            }
            Self::TransferFrom {
                to_address_hex,
                amount,
                from_address_hex,
            } => {
                let from = Address::from_str(&from_address_hex)?;
                let to = Address::from_str(&to_address_hex)?;
                let amount = Uint::from_str_radix(&amount, 10)?;
                let input = [
                    TRANSFER_FROM_SELECTOR,
                    &ethabi::encode(&[
                        Token::Address(from),
                        Token::Address(to),
                        Token::Uint(amount),
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
    use ethabi::{Address, Function, ParamType, Token, Uint};

    #[test]
    fn test_total_supply_encoding() {
        #[allow(deprecated)]
        let total_supply_function = Function {
            name: "totalSupply".into(),
            inputs: vec![],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ParamType::Uint(256),
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
        let address: [u8; 20] = rand::random();

        let address = Address::from(address);

        #[allow(deprecated)]
        let balance_of_function = Function {
            name: "balanceOf".into(),
            inputs: vec![ethabi::Param {
                name: "account".into(),
                kind: ParamType::Address,
                internal_type: None,
            }],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ParamType::Uint(256),
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::View,
        };
        let expected_tx_data = balance_of_function
            .encode_input(&[Token::Address(address)])
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
        let address: [u8; 20] = rand::random();
        let amount: [u8; 32] = rand::random();

        let address = Address::from(address);
        let amount = Uint::from_big_endian(&amount);

        #[allow(deprecated)]
        let transfer_function = Function {
            name: "transfer".into(),
            inputs: vec![
                ethabi::Param {
                    name: "to".into(),
                    kind: ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "amount".into(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ParamType::Bool,
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::NonPayable,
        };
        let expected_tx_data = transfer_function
            .encode_input(&[Token::Address(address), Token::Uint(amount)])
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
        let owner: [u8; 20] = rand::random();
        let spender: [u8; 20] = rand::random();

        let owner = Address::from(owner);
        let spender = Address::from(spender);

        #[allow(deprecated)]
        let allowance_function = Function {
            name: "allowance".into(),
            inputs: vec![
                ethabi::Param {
                    name: "owner".into(),
                    kind: ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "spender".into(),
                    kind: ParamType::Address,
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ParamType::Uint(256),
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::View,
        };
        let expected_tx_data = allowance_function
            .encode_input(&[Token::Address(owner), Token::Address(spender)])
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
        let address: [u8; 20] = rand::random();
        let amount: [u8; 32] = rand::random();

        let address = Address::from(address);
        let amount = Uint::from_big_endian(&amount);

        #[allow(deprecated)]
        let approve_function = Function {
            name: "approve".into(),
            inputs: vec![
                ethabi::Param {
                    name: "spender".into(),
                    kind: ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "amount".into(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ParamType::Bool,
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::NonPayable,
        };
        let expected_tx_data = approve_function
            .encode_input(&[Token::Address(address), Token::Uint(amount)])
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
        let from: [u8; 20] = rand::random();
        let to: [u8; 20] = rand::random();
        let amount: [u8; 32] = rand::random();

        let from = Address::from(from);
        let to = Address::from(to);
        let amount = Uint::from_big_endian(&amount);

        #[allow(deprecated)]
        let transfer_from_function = Function {
            name: "transferFrom".into(),
            inputs: vec![
                ethabi::Param {
                    name: "from".into(),
                    kind: ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "to".into(),
                    kind: ParamType::Address,
                    internal_type: None,
                },
                ethabi::Param {
                    name: "amount".into(),
                    kind: ParamType::Uint(256),
                    internal_type: None,
                },
            ],
            outputs: vec![ethabi::Param {
                name: String::new(),
                kind: ParamType::Bool,
                internal_type: None,
            }],
            constant: None,
            state_mutability: ethabi::StateMutability::NonPayable,
        };
        let expected_tx_data = transfer_from_function
            .encode_input(&[
                Token::Address(from),
                Token::Address(to),
                Token::Uint(amount),
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
