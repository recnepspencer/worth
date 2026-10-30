pub use worth_store::physical_runtime::ExternalPhysicalRecordLocator;

pub fn decode_locator(value: &str) -> ExternalPhysicalRecordLocator {
    worth_store::physical_runtime::ExternalPhysicalRecordLocator::decode(unhex(value)).unwrap()
}

pub fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

pub fn unhex(value: &str) -> [u8; 40] {
    let mut bytes = [0_u8; 40];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        bytes[index] = u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap();
    }
    bytes
}
