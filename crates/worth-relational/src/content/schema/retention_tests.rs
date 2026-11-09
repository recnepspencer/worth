//! A snapshot extracted from a carried projection retains its original bill.
use super::*;
#[test]
fn extracted_content_snapshot_retains_original_request_bill_until_last_clone() {
    use worth_execution::{
        CancellationSource, ExecutionRequest, SerialMemoryBudget, SerialRequest,
    };
    let budget = SerialMemoryBudget::new(Some(1024 * 1024));
    let host = SerialRequest::from_memory(budget.clone(), CancellationSource::new().token(), None);
    let request = ExecutionRequest::serial(&host);
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
        .snapshot_for_compilation(crate::schema::SchemaCompilationControl::Request(request))
        .unwrap();
    let content = snapshot.content_formats.clone();
    let projection_bytes = budget.charged_memory_bytes();
    drop(registry);
    drop(snapshot);
    let content_bytes = budget.charged_memory_bytes();
    assert!(content_bytes > 0);
    assert!(content_bytes < projection_bytes);
    let last = content.clone();
    let wire = rmp_serde::to_vec(&content).unwrap();
    let decoded: RelationalContentSchemaSnapshot = rmp_serde::from_slice(&wire).unwrap();
    assert_eq!(
        decoded, content,
        "request custody is absent from durable meaning"
    );
    drop(content);
    assert_eq!(budget.charged_memory_bytes(), content_bytes);
    drop(last);
    assert_eq!(budget.charged_memory_bytes(), 0);
}
