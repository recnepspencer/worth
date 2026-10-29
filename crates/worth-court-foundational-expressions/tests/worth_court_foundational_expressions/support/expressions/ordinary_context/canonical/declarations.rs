// Canonical identity and the draft codec: source, builder, and decoded
// drafts agree; meaning changes identity and spelling does not.
pub(crate) use {worth_foundational::expression_api::expressions, worth_foundational::expression_api::ExpressionBuilder, worth_foundational::expression_api::ExpressionDenialDetail, worth_foundational::expression_api::ExpressionDenialFamily as Family, worth_foundational::expression_api::ExpressionFunctionCatalog, worth_foundational::expression_api::ExpressionProfile, worth_foundational::expression_api::ExpressionType};

pub(crate) use {crate::expressions::ordinary_context::admit, crate::expressions::ordinary_context::draft, crate::expressions::ordinary_context::empty_catalog, crate::expressions::ordinary_context::function, crate::expressions::ordinary_context::schema};

pub(crate) fn digest(source: &str) -> Vec<u8> {
    let admitted = admit(source).unwrap_or_else(|denial| panic!("{source}: {denial:?}"));
    admitted.identity().digest().value().bytes().to_vec()
}

pub(crate) fn header(wire: u16, language: u16, count: u32) -> Vec<u8> {
    let mut bytes = b"WXDR".to_vec();
    bytes.extend_from_slice(&wire.to_le_bytes());
    bytes.extend_from_slice(&language.to_le_bytes());
    bytes.extend_from_slice(&count.to_le_bytes());
    bytes
}

pub(crate) fn bool_node(value: u8) -> [u8; 2] {
    [0, value]
}

pub(crate) fn decode_family(bytes: &[u8]) -> Family {
    expressions()
        .decode(bytes)
        .expect_err("the encoding is invalid")
        .family()
}