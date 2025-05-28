use aurora_engine_types::account_id::AccountId;

pub trait IntoAurora<T> {
    fn into_aurora(self) -> T;
}

impl IntoAurora<AccountId> for crate::near::primitives::types::AccountId {
    fn into_aurora(self) -> AccountId {
        AccountId::new(self.as_ref()).unwrap()
    }
}
