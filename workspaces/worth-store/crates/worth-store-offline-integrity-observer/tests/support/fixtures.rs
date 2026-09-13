use std::path::PathBuf;

use super::TemporaryRoot;

const NAMESPACE_IDENTITY_HEX: &str = concat!(
    "5753544e5349440001000100480000000100000001001000010203040506070809",
    "0a0b0c0d0e0f10d7bf452a4916b29cdc460a8a54a3e86fbe47fa87b5da5f1c",
    "933f26fd6a667e93"
);

const CURRENT_SELECTOR_HEX: &str = concat!(
    "5752433546524d000b0201000040000001010118300000003b0000000b000000",
    "0000000000000000000000004b44e9520102030405060708090a0b0c0d0e0f10",
    "010100000000000000000000000000000000000000000000000100004000000101",
    "01180000000000000000"
);
const PREVIOUS_SELECTOR_HEX: &str = concat!(
    "5752433546524d000b0201000040000001010118300000003b0000000c000000",
    "0000000000000000000000003cba79a70102030405060708090a0b0c0d0e0f10",
    "020100000000000000000000000000000000000000000000000100004000000101",
    "01180000000000000000"
);
const ROOT_MANIFEST_HEX: &str = concat!(
    "5752433546524d00020201000040000001010118300000004001000001000000",
    "000000000000000000000000cabfb33701000000000000000700000000000000",
    "0200000000000000000000000000000001000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000443322110000000000000000000000000000000000000000",
    "0000000000000000000000000000000000000000000000000000000000000000",
    "0000000000000000000000000000000001000000000000000100000000000000",
    "0100000000000000010000000000000000000000887766550100000000000000",
    "0100000000000000010000000000000001000000000000000000000000000000",
    "00000000000000000000000000000000"
);

pub(crate) struct StoreFixture {
    _temporary: TemporaryRoot,
    pub(crate) store: PathBuf,
    pub(crate) records: PathBuf,
    pub(crate) roots: PathBuf,
    pub(crate) report: PathBuf,
}

pub(crate) fn clean_store(label: &str) -> StoreFixture {
    let temporary = TemporaryRoot::new(label);
    let store = temporary.path().join("store");
    let records = store.join("families/records");
    let roots = records.join("roots");
    let namespace = store.join("namespace");
    let reports = temporary.path().join("reports");
    std::fs::create_dir_all(&roots).unwrap();
    std::fs::create_dir(&namespace).unwrap();
    std::fs::create_dir(&reports).unwrap();
    std::fs::write(namespace.join("identity"), namespace_identity_bytes()).unwrap();
    std::fs::write(
        records.join("root-current.selector"),
        current_selector_bytes(),
    )
    .unwrap();
    std::fs::write(
        records.join("root-previous.selector"),
        previous_selector_bytes(),
    )
    .unwrap();
    std::fs::write(
        roots.join("root-0000000000000001.manifest"),
        root_manifest_bytes(),
    )
    .unwrap();
    StoreFixture {
        _temporary: temporary,
        store,
        records,
        roots,
        report: reports.join("observation.json"),
    }
}

pub(crate) fn namespace_identity_bytes() -> Vec<u8> {
    decode_hex(NAMESPACE_IDENTITY_HEX)
}

pub(crate) fn current_selector_bytes() -> Vec<u8> {
    decode_hex(CURRENT_SELECTOR_HEX)
}
pub(crate) fn previous_selector_bytes() -> Vec<u8> {
    decode_hex(PREVIOUS_SELECTOR_HEX)
}
pub(crate) fn root_manifest_bytes() -> Vec<u8> {
    decode_hex(ROOT_MANIFEST_HEX)
}

fn decode_hex(value: &str) -> Vec<u8> {
    assert_eq!(value.len() % 2, 0);
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| (hex_digit(pair[0]) << 4) | hex_digit(pair[1]))
        .collect()
}

fn hex_digit(value: u8) -> u8 {
    match value {
        b'0'..=b'9' => value - b'0',
        b'a'..=b'f' => value - b'a' + 10,
        _ => panic!("literal fixture contains non-hex byte"),
    }
}
