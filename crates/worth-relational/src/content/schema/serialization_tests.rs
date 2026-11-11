//! Portable content schema meaning survives serialization without runtime issuance.
use super::*;
#[test]
fn content_schema_serialization_preserves_format_and_field_meaning() {
    let mut registry = crate::schema::data::RelationalSchemaRegistry::new();
    registry.content_formats = crate::content::RelationalContentFormatRegistry::from_recorded(
        RelationalContentSchemaSnapshot::new(
            vec![RelationalContentFormatDeclaration {
                identity: [7; 32],
                descriptor: ContentFormat::new("retained-format", 1).unwrap(),
                maximum_frame_bytes: 1024,
            }],
            Vec::new(),
        ),
    )
    .unwrap();
    let snapshot = registry
        .snapshot_for_compilation(crate::schema::SchemaCompilationControl::Configuration)
        .unwrap();
    let content = snapshot.content_formats.clone();
    let wire = rmp_serde::to_vec(&content).unwrap();
    let decoded: RelationalContentSchemaSnapshot = rmp_serde::from_slice(&wire).unwrap();
    assert_eq!(decoded, content);
    assert_eq!(decoded.formats().len(), 1);
    assert_eq!(decoded.formats()[0].descriptor.name(), "retained-format");
    assert_eq!(decoded.formats()[0].identity, [7; 32]);
    assert_eq!(decoded.formats()[0].maximum_frame_bytes, 1024);
    assert!(decoded.fields().is_empty());
}
