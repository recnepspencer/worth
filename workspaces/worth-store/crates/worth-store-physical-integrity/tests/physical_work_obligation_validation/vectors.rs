pub const STORE_BYTES: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];

const OPERATION_3_HEX: &str = "575045464645435406060000000000000102030405060708090a0b0c0d0e0f10010000000000000002000000000000000300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000b2a18aee7dbc138ec738507b8fcd7f722280d499d9bf0f3962294fccfe50bd0c";
const OPERATION_4_HEX: &str = "575045464645435406050000000000000102030405060708090a0b0c0d0e0f1001000000000000000200000000000000040000000000000009000000000000000a00000000000000abababababababababababababababababababababababababababababababab0601000000000000070000000000000008000000000000004cad4be6809b3b053cf2951815f20c0d1cb2649b87a61b582c1a0a3a89010b90";
pub const OPERATION_3_SHA: [u8; 32] = [
    0xb2, 0xa1, 0x8a, 0xee, 0x7d, 0xbc, 0x13, 0x8e, 0xc7, 0x38, 0x50, 0x7b, 0x8f, 0xcd, 0x7f, 0x72,
    0x22, 0x80, 0xd4, 0x99, 0xd9, 0xbf, 0x0f, 0x39, 0x62, 0x29, 0x4f, 0xcc, 0xfe, 0x50, 0xbd, 0x0c,
];
pub const OPERATION_4_SHA: [u8; 32] = [
    0x4c, 0xad, 0x4b, 0xe6, 0x80, 0x9b, 0x3b, 0x05, 0x3c, 0xf2, 0x95, 0x18, 0x15, 0xf2, 0x0c, 0x0d,
    0x1c, 0xb2, 0x64, 0x9b, 0x87, 0xa6, 0x1b, 0x58, 0x2c, 0x1a, 0x0a, 0x3a, 0x89, 0x01, 0x0b, 0x90,
];
pub const OPERATION_3_SCOPE_SHA: [u8; 32] = [
    0x9d, 0xba, 0xb8, 0x59, 0xef, 0x54, 0x8a, 0xe2, 0xbc, 0x88, 0xad, 0x37, 0x76, 0x80, 0xb3, 0x11,
    0x11, 0xbb, 0x56, 0x42, 0xb2, 0x37, 0x6e, 0x06, 0xd7, 0x89, 0x95, 0xa9, 0x66, 0xe6, 0xfb, 0x06,
];

pub fn operation_3() -> [u8; 160] {
    literal(OPERATION_3_HEX)
}

pub fn operation_4() -> [u8; 160] {
    literal(OPERATION_4_HEX)
}

fn literal(hex: &str) -> [u8; 160] {
    let mut bytes = [0_u8; 160];
    for (slot, pair) in bytes.iter_mut().zip(hex.as_bytes().chunks_exact(2)) {
        *slot = u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16)
            .expect("frozen physical-work hex byte");
    }
    bytes
}
