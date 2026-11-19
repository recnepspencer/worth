//! Independent current wire vectors.
pub const CURRENT_PACKAGE_ROOT_FRAMES_HEX: &str = "0015000100000000000000190000000d61636d652e776f726b666c6f770000000300000007|00150002000000010000000e0000000a67726170682e72656164|00150003000000020000001400000010776f726b666c6f772e72756e74696d65|00150004000000030000000b0000000764757261626c65|00150005000000040000002600010000000862616c616e636564000000146465626974732d657175616c2d63726564697473|00150005000000050000002800020000000d6163636f756e742d62792d69640000001165786163742d656e746974792d72656164|00150005000000060000002e00030000000e7061796d656e742d6e6f746963650000001665787465726e616c2d6566666563742d66616d696c79|0015000a0000000700000009000000056175646974";

pub fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    bytes.iter().fold(
        String::with_capacity(bytes.len() * 2),
        |mut encoded, byte| {
            write!(&mut encoded, "{byte:02x}").expect("writing to String cannot fail");
            encoded
        },
    )
}

pub fn decode_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0, "golden hex must contain whole bytes");
    encoded
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let digits = std::str::from_utf8(pair).expect("golden hex is ASCII");
            u8::from_str_radix(digits, 16).expect("golden hex contains valid digits")
        })
        .collect()
}
