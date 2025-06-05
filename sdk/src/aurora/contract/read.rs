use crate::aurora::ContractMethod;
use aurora_engine_types::types::{Address, EthGas};
use near_primitives::types::AccountId;

macro_rules! view_method {
    ($name:ident, $method:literal, $response:ty) => {
        pub struct $name;

        impl ContractMethod for $name {
            type Response = $response;

            fn method_type() -> crate::aurora::MethodType {
                crate::aurora::MethodType::View
            }

            fn method_name(&self) -> &'static str {
                $method
            }
        }
    };
}

view_method!(GetOwner, "get_owner", AccountId);
view_method!(GetFixedGas, "get_fixed_gas", Option<EthGas>);
view_method!(
    GetFallbackAddress,
    "get_erc20_fallback_address",
    Option<Address>
);

pub struct GetBalance {
    pub address: Address,
}

impl ContractMethod for GetBalance {
    type Response = Vec<u8>;

    fn method_name(&self) -> &'static str {
        "get_balance"
    }

    fn params(&self) -> Result<Vec<u8>, std::io::Error> {
        Ok(self.address.as_bytes().to_vec())
    }
}
