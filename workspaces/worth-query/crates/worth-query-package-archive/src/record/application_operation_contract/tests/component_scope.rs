//! Scope4 shares the schema member's descriptive codec under protocol18.
use super::*;
use worth_query_declaration::facade::{application_query::*, application_schema::*};
use worth_query_installation::facade::WorthQueryPortableOperationGraphReadScope;
#[test]
fn component_graph_scope_round_trips_without_changing_old_contract_bytes() {
    let mut parts = fixture::complete_parts();
    let definition = ApplicationOperationComponentReadMeaning::from_untrusted_parts(
        WorthQueryPortableOperationComponentReadParts {
            operation: "settle".into(),
            set: "Accounts".into(),
            root: "Account".into(),
            member: "Account".into(),
            paths: vec![ApplicationQueryRootPathMeaning::from_untrusted_parts(
                WorthQueryPortableApplicationQueryRootPathParts {
                    start_entity: "Account".into(),
                    terminal_entity: "Account".into(),
                    steps: vec![],
                    guards: vec![],
                },
            )],
        },
    );
    parts
        .graph_reads
        .push(WorthQueryPortableOperationGraphReadScope::Components {
            schema: parts.schema.clone(),
            definition,
        });
    let record = WorthQueryPortableApplicationOperationContractRecord::from_untrusted_parts(parts);
    let bytes = encode_untrusted(&record, WorthQueryPackageArchiveLimits::DEFAULT);
    let mut decoder =
        WorthQueryPackageArchiveRecordDecoder::new(WorthQueryPackageArchiveLimits::DEFAULT);
    assert_eq!(
        decoder.decode_frame(&bytes).unwrap().record(),
        &fixture::wrapped(record)
    );
    // The existing complete-contract court independently keeps its frozen v18 bytes.
}
