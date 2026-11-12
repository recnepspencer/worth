//! Candidate ownership validation reads each record's custody once per edit.
use super::*;
use worth_relational::facade::inspection::observe_candidate_custody_reads;

#[test]
fn candidate_ownership_reads_each_records_custody_about_once() {
    let _execution = WorthQueryExecutionObservation::install_on_this_thread();
    let host = install(false);
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    seed(&host, &request, 16);
    let (outcome, custody) = observe_candidate_custody_reads(|| fan_out(&host, &request, 16, 17));
    assert!(
        matches!(
            outcome,
            WorthQueryApplicationPerformedMutationOutcome::Performed(_)
        ),
        "the wide edit performs"
    );
    assert!(
        custody.distinct_records > 16,
        "the edit validates ownership of every writer's records: {custody:?}"
    );
    // Every walk hop used to read and authenticate the record's custody again:
    // about five reads per record. Reading each record once gives about one.
    assert!(
        custody.reads * 2 <= custody.distinct_records * 3,
        "custody reads per distinct record: {custody:?}"
    );
}
