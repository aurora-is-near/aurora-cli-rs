pub fn hex_to_vec(hex: &str) -> anyhow::Result<Vec<u8>> {
    hex::decode(hex.trim_start_matches("0x"))
        .map_err(|e| anyhow::anyhow!("Couldn't create vector from the hex: {hex}, {e}"))
}

pub fn hex_to_arr<const SIZE: usize>(hex: &str) -> anyhow::Result<[u8; SIZE]> {
    let mut output = [0u8; SIZE];

    hex::decode_to_slice(hex.trim_start_matches("0x"), &mut output)
        .map(|()| output)
        .map_err(|e| anyhow::anyhow!("Couldn't create array from the hex: {hex}, {e}"))
}
