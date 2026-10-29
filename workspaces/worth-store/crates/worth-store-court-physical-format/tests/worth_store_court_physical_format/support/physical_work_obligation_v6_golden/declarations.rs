

pub(crate) use {crate::store_physical_format_support::independent_sha256};
pub(crate) use {worth_store_physical_format::physical_work_obligation::decode_physical_work_obligation_v6, worth_store_physical_format::physical_work_obligation::encode_physical_work_obligation_v6, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationOperationCode, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationTargetCode, worth_store_physical_format::physical_work_obligation::PhysicalWorkObligationV6};

pub(crate) const OPERATION_3_HEX: &str = "575045464645435406060000000000000102030405060708090a0b0c0d0e0f10010000000000000002000000000000000300000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000050000000000000000000000000000000000000000000000b2a18aee7dbc138ec738507b8fcd7f722280d499d9bf0f3962294fccfe50bd0c";
pub(crate) const OPERATION_4_HEX: &str = "575045464645435406050000000000000102030405060708090a0b0c0d0e0f1001000000000000000200000000000000040000000000000009000000000000000a00000000000000abababababababababababababababababababababababababababababababab0601000000000000070000000000000008000000000000004cad4be6809b3b053cf2951815f20c0d1cb2649b87a61b582c1a0a3a89010b90";
pub(crate) const OPERATION_3_SHA: &str = "b2a18aee7dbc138ec738507b8fcd7f722280d499d9bf0f3962294fccfe50bd0c";
pub(crate) const OPERATION_4_SHA: &str = "4cad4be6809b3b053cf2951815f20c0d1cb2649b87a61b582c1a0a3a89010b90";

pub(crate) fn literal(hex: &str) -> Vec<u8> {
    hex.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).expect("literal hex byte")
        })
        .collect()
}