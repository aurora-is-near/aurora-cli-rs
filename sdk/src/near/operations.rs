use aurora_engine_types::types::NearGas;
use near_jsonrpc_client::JsonRpcClient;
use near_token::NearToken;

const ONE_TERA_GAS: u64 = 10u64.pow(12);
const MAX_GAS: NearGas = NearGas::new(300 * ONE_TERA_GAS);

pub(crate) const DEFAULT_CALL_FN_GAS: NearGas = NearGas::new(10 * ONE_TERA_GAS);
pub(crate) const DEFAULT_CALL_DEPOSIT: NearToken = NearToken::from_near(0);
pub(crate) const DEFAULT_PRIORITY_FEE: u64 = 0;

pub struct Function {
    pub(crate) name: String,
    pub(crate) args: anyhow::Result<Vec<u8>>,
    pub(crate) deposit: NearToken,
    pub(crate) gas: NearGas,
}

impl Function {
    /// Initialize a new instance of [`Function`], tied to a specific function on a
    /// contract that lives directly on a contract we've specified in [`Transaction`].
    pub fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            args: Ok(vec![]),
            deposit: DEFAULT_CALL_DEPOSIT,
            gas: DEFAULT_CALL_FN_GAS,
        }
    }

    /// Provide the arguments for the call. These args are serialized bytes from either
    /// a JSON or Borsh serializable set of arguments. To use the more specific versions
    /// with better quality of life, use `args_json` or `args_borsh`.
    pub fn args(mut self, args: Vec<u8>) -> Self {
        if self.args.is_err() {
            return self;
        }
        self.args = Ok(args);
        self
    }

    /// Similar to `args`, specify an argument that is JSON serializable and can be
    /// accepted by the equivalent contract. Recommend to use something like
    /// `serde_json::json!` macro to easily serialize the arguments.
    pub fn args_json<U: serde::Serialize>(mut self, args: U) -> Self {
        self.args = serde_json::to_vec(&args).map_err(Into::into);
        self
    }

    /// Similar to `args`, specify an argument that is borsh serializable and can be
    /// accepted by the equivalent contract.
    pub fn args_borsh<U: borsh::BorshSerialize>(mut self, args: U) -> Self {
        self.args = borsh::to_vec(&args).map_err(Into::into);
        self
    }

    /// Specify the amount of tokens to be deposited where `deposit` is the amount of
    /// tokens in yocto near.
    pub fn deposit(mut self, deposit: NearToken) -> Self {
        self.deposit = deposit;
        self
    }

    /// Specify the amount of gas to be used.
    pub fn gas(mut self, gas: NearGas) -> Self {
        self.gas = gas;
        self
    }

    /// Use the maximum amount of gas possible to perform this function call into the contract.
    pub fn max_gas(self) -> Self {
        self.gas(MAX_GAS)
    }
}

pub struct Transaction {
    client: JsonRpcClient,
}
