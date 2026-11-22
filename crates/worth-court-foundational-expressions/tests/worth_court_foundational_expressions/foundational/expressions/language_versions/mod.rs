// Language versions: V1-only meaning keeps its V1 draft bytes and canonical
// identity exactly, V2 vocabulary records V2, and a header that misstates
// the version its vocabulary needs is refused.
pub(crate) use crate::expressions::ordinary_context::{admit, draft, function, nominal, schema};
use worth_foundational::expression_api::{
    expressions, ExpressionDenialFamily, ExpressionSchema, ExpressionType,
};
use worth_foundational::facade::CanonicalBasisValue;

/// Identities and encodings recorded before V2 vocabulary existed.
const V1_IDENTITIES: [(&str, &str); 4] = [
    (
        "width - 2.0 * depth / 1.0 > 0.5 || frame.material == Material::Steel",
        "fc250d62597810946e936147b9f0d3a8780b3550a6eed2935efa80c371cccdbe",
    ),
    (
        "starts_with(label ?? \"\", \"Ste\") && contains(label ?? \"\", \"el\") && !ends_with(label ?? \"\", \"x\")",
        "0cb696437bf89698cd9e39930343dec48c3c673d2d1b08fcf378a665013d893c",
    ),
    (
        "members.map(m, m.thickness * 2.0)",
        "25f0bab63389a63acfa7e1ac9b5c5ac84bdb3c7357f4a5b82a5749e9bfbcace0",
    ),
    (
        "count <= 4 && bus == bus && is_some(label)",
        "b9b666a04b8f5fa1c2383cbc4a8d446e88e7d873c0f78bdd812552b5caf4f328",
    ),
];
const V1_TEXT_DRAFT: &str = "5758445201000100120000000401000000050000006c6162656c0300000000080d000000000100000003030000005374650b010000000b0000007374617274735f77697468000000000200000002000000030000000401000000050000006c6162656c0300000000080d05000000060000000302000000656c0b0100000008000000636f6e7461696e7300000000020000000700000008000000080b04000000090000000401000000050000006c6162656c0300000000080d0b0000000c0000000301000000780b0100000009000000656e64735f7769746800000000020000000d0000000e00000007000f000000080b0a00000010000000";
const V1_PRESENCE_DRAFT: &str = "57584452010001000a000000040100000005000000636f756e7401040000000000000000000000000000000806000000000100000004010000000300000062757304010000000300000062757308090300000004000000080b02000000050000000401000000050000006c6162656c0b010000000700000069735f736f6d65000000000100000007000000080b0600000008000000";

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn unhex(text: &str) -> Vec<u8> {
    (0..text.len())
        .step_by(2)
        .map(|at| u8::from_str_radix(&text[at..at + 2], 16).expect("fixture hex"))
        .collect()
}

fn digest(source: &str) -> String {
    let admitted = admit(source).unwrap_or_else(|denial| panic!("{source}: {denial:?}"));
    hex(admitted.identity().digest().value().bytes())
}

fn recorded_language(source: &str) -> String {
    let admitted = admit(source).unwrap_or_else(|denial| panic!("{source}: {denial:?}"));
    language_label(&admitted)
}

fn language_label(admitted: &worth_foundational::expression_api::AdmittedExpression) -> String {
    let label = admitted
        .identity()
        .basis()
        .entries()
        .iter()
        .find_map(|entry| match entry.value() {
            CanonicalBasisValue::ExactText(text) => text
                .raw_text()
                .filter(|text| text.starts_with("worth-expression-v"))
                .map(str::to_owned),
            _ => None,
        });
    label.expect("every identity records its language version")
}

fn declared_language(bytes: &[u8]) -> u16 {
    u16::from_le_bytes([bytes[6], bytes[7]])
}

fn with_declared_language(bytes: &[u8], language: u16) -> Vec<u8> {
    let mut changed = bytes.to_vec();
    changed[6..8].copy_from_slice(&language.to_le_bytes());
    changed
}

#[test]
fn v1_identities_and_encodings_are_byte_for_byte_unchanged() {
    for (source, identity) in V1_IDENTITIES {
        assert_eq!(digest(source), identity, "{source}");
        assert_eq!(recorded_language(source), "worth-expression-v1");
    }
    for (index, recorded) in [(1, V1_TEXT_DRAFT), (3, V1_PRESENCE_DRAFT)] {
        let (source, identity) = V1_IDENTITIES[index];
        assert_eq!(hex(&draft(source).encode()), recorded);
        let decoded = expressions()
            .decode(&unhex(recorded))
            .expect("recorded V1 drafts still decode");
        assert_eq!(hex(&decoded.encode()), recorded);
        let schema = crate::expressions::ordinary_context::schema();
        let catalog = crate::expressions::ordinary_context::empty_catalog(&schema);
        let admitted = decoded
            .admit(
                &schema,
                &catalog,
                worth_foundational::expression_api::ExpressionProfile::interactive(),
            )
            .expect("recorded V1 drafts still admit");
        assert_eq!(hex(admitted.identity().digest().value().bytes()), identity);
    }
}

#[test]
fn v2_vocabulary_declares_and_records_language_two() {
    for source in [
        "contains_ignore_ascii_case(label ?? \"\", \"STEEL\")",
        "is_some(none<Date>)",
        "is_some(none<List<Option<Date>>>)",
    ] {
        let encoded = draft(source).encode();
        assert_eq!(declared_language(&encoded), 2, "{source}");
        let decoded = expressions().decode(&encoded).expect("V2 drafts decode");
        assert_eq!(decoded.encode(), encoded);
        assert_eq!(recorded_language(source), "worth-expression-v2");
    }
}

#[test]
fn a_header_that_misstates_the_required_language_is_refused() {
    let v2 = draft("ends_with_ignore_ascii_case(label ?? \"\", \"EL\")").encode();
    let v1 = unhex(V1_TEXT_DRAFT);
    for misstated in [
        with_declared_language(&v2, 1),
        with_declared_language(&v1, 2),
    ] {
        let denial = expressions().decode(&misstated).unwrap_err();
        assert_eq!(
            denial.family(),
            ExpressionDenialFamily::InvalidValue,
            "{denial:?}"
        );
    }
    let unknown = expressions()
        .decode(&with_declared_language(&v1, 3))
        .unwrap_err();
    assert_eq!(unknown.family(), ExpressionDenialFamily::UnsupportedVersion);
}

#[test]
fn nominal_declarations_cannot_shadow_the_date_type() {
    let shadowed = ExpressionSchema::builder().record("Date", 1, [("days", ExpressionType::INT64)]);
    assert!(shadowed.is_err());
    let unshadowed = ExpressionSchema::builder()
        .record("Deadline", 1, [("on", ExpressionType::Date)])
        .and_then(|schema| schema.operand("deadline", ExpressionType::Record(nominal("Deadline"))))
        .expect("Date is an ordinary field type")
        .build();
    let catalog = worth_foundational::expression_api::ExpressionFunctionCatalog::builder(
        &unshadowed,
        worth_foundational::expression_api::ExpressionProfile::interactive(),
    )
    .build();
    let admitted = expressions()
        .parse("deadline == deadline")
        .and_then(|draft| {
            draft.admit(
                &unshadowed,
                &catalog,
                worth_foundational::expression_api::ExpressionProfile::interactive(),
            )
        })
        .expect("records with Date fields admit");
    let records_v2 = admitted.identity().basis().entries().iter().any(|entry| {
        matches!(entry.value(), CanonicalBasisValue::ExactText(text)
            if text.raw_text() == Some("worth-expression-v2"))
    });
    assert!(records_v2, "comparing records compares their Date fields");
}

#[test]
fn a_caller_records_the_language_of_the_functions_it_calls() {
    let schema = schema();
    let profile = worth_foundational::expression_api::ExpressionProfile::interactive();
    let mut builder =
        worth_foundational::expression_api::ExpressionFunctionCatalog::builder(&schema, profile);
    builder
        .install(function(
            "text::mentions_steel",
            &[("text", ExpressionType::String)],
            ExpressionType::Bool,
            "contains_ignore_ascii_case(text, \"steel\")",
        ))
        .expect("V2 functions install");
    let catalog = builder.build();
    let caller = draft("text::mentions_steel(label ?? \"\")");
    assert_eq!(
        declared_language(&caller.encode()),
        1,
        "its own vocabulary is V1"
    );
    let admitted = caller
        .admit(&schema, &catalog, profile)
        .expect("the caller admits");
    assert_eq!(language_label(&admitted), "worth-expression-v2");
}
