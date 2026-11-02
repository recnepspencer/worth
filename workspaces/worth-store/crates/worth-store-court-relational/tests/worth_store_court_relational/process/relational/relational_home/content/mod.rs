//! Native content crosses the genuine Store journal and checkpoint boundaries.
mod corruption;
mod fixture;
mod publication_fate;
use fixture::*;
use worth_execution::ExecutionRequest;
use worth_relational::facade::content::RelationalContentDenial;

#[test]
fn content_store_reopen_tail_replacement_and_retained_reader_preserve_exact_values() {
    let host = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&host);
    let directory = tempfile::tempdir().unwrap();
    let location = directory.path().join("home");
    let home = open(&location, &execution).unwrap();
    let (old, entity) = publish(&home, &execution, i64::MIN, None);
    home.checkpoint(&execution).unwrap();
    home.close();
    let home = open(&location, &execution).unwrap();
    assert_value(&home, &execution, entity, old, i64::MIN);
    let (_, historical) = home
        .runtime()
        .observe_branch(&home.runtime().main_branch_identity())
        .unwrap();
    let retained = historical.read_content(&execution, old, FORMAT).unwrap();
    let (new, _) = publish(&home, &execution, 7, Some(entity));
    assert_ne!(old, new);
    // First reopen must reconstruct the content-bearing journal tail over the
    // earlier checkpoint; the second must use its replacement checkpoint.
    assert_eq!(
        historical
            .read_content(&execution, old, FORMAT)
            .unwrap()
            .bytes(),
        retained.bytes()
    );
    drop(historical);
    home.close();
    let home = open(&location, &execution).unwrap();
    assert_value(&home, &execution, entity, new, 7);
    let (_, current) = home
        .runtime()
        .observe_branch(&home.runtime().main_branch_identity())
        .unwrap();
    assert_eq!(
        current.read_content(&execution, old, FORMAT).unwrap_err(),
        RelationalContentDenial::MissingContent
    );
    home.checkpoint(&execution).unwrap();
    drop(current);
    home.close();
    let home = open(&location, &execution).unwrap();
    assert_value(&home, &execution, entity, new, 7);
    assert_eq!(
        i64::from_le_bytes(retained.bytes()[43..51].try_into().unwrap()),
        i64::MIN
    );
    home.close();
}
