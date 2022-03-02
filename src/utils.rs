pub(crate) fn hex_to_arr32(h: &str) -> [u8; 32] {
    let mut output = [0u8; 32];
    hex::decode_to_slice(h, &mut output).unwrap();
    output
}
