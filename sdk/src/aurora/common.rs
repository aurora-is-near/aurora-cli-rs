use aurora_engine_types::{account_id::AccountId, types::Address};

pub trait IntoAurora<T> {
    fn into_aurora(self) -> T;
}

impl IntoAurora<AccountId> for crate::near::primitives::types::AccountId {
    fn into_aurora(self) -> AccountId {
        AccountId::new(self.as_ref()).unwrap()
    }
}

pub fn hex_to_arr<const SIZE: usize>(hex: &str) -> anyhow::Result<[u8; SIZE]> {
    let mut output = [0u8; SIZE];

    hex::decode_to_slice(hex.trim_start_matches("0x"), &mut output)
        .map(|()| output)
        .map_err(|e| anyhow::anyhow!("Couldn't create array from the hex: {hex}, {e}"))
}

pub fn hex_to_address(h: &str) -> anyhow::Result<Address> {
    hex_to_arr(h).map(Address::from_array)
}
