use std::str::FromStr;

pub trait AsPrimitive<T> {
    fn as_primitive(&self) -> T;
}

impl AsPrimitive<near_primitives::types::AccountId> for aurora_engine_types::account_id::AccountId {
    fn as_primitive(&self) -> near_primitives::types::AccountId {
        near_primitives::types::AccountId::from_str(self.as_ref()).unwrap()
    }
}
