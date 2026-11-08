//! Public owned publications keep a fixed membership delta as the roster grows.
use super::*;
mod members;
use cold_owned_revision::{create, settle, RequiredProgram, Root};
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::inspection::observe_maintenance_work;

fn measure<const N: usize>() -> Vec<(u64, Vec<usize>)> {
    type S<const N: usize> = Model<69, heterogeneous::Absent, members::Members<N>>;
    let host = correction_courts::install_definition::<S<N>, RequiredProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let mut rows = Vec::new();
    for value in 1..=4 {
        let started_at = std::time::Instant::now();
        let (_, work) = observe_maintenance_work(64, || {
            let outcome = request
                .advanced()
                .mutate(parameter_inverse::Edit { id: 10, value })
                .without_source()
                .idempotency(&format!("membership-edit-{value}"))
                .execute_performed::<RequiredProgram, Root<S<N>>>(
                    &host,
                    ExecutionAllocationPolicy::SystemAllocation,
                )
                .unwrap();
            let WorthQueryApplicationPerformedMutationOutcome::Performed(performed) = outcome
            else {
                panic!("source edit refused");
            };
            let mut started = performed
                .start_required_outputs(&host, &request, WorthQueryOutputDemandControls::default())
                .unwrap_or_else(|failure| panic!("{:?}", failure.denial()));
            for _ in 0..64 {
                if let WorthQueryApplicationProgramOutputProgress::Settled(output) = started
                    .required_output_mut()
                    .advance(&host, &request)
                    .unwrap()
                {
                    let current = request.retain_current_read().unwrap();
                    request
                        .at(&current)
                        .require_current_program_output(&output)
                        .unwrap();
                    return;
                }
            }
            panic!("membership edit must settle");
        });
        let micros = started_at.elapsed().as_micros();
        assert!(work.ownership_rows > 0, "real publication must be observed");
        assert!(work.commit_payload_hash_bytes > 0);
        assert_eq!(
            work.commit_payload_hash_bytes, work.visibility_payload_hash_bytes,
            "each distinct commitment consumes the shared payload once; linkage must not rehash it"
        );
        assert_eq!(
            work.omitted_publications, 0,
            "publication evidence must be complete"
        );
        assert!(
            !work.published_rows.is_empty(),
            "real canonical publications must be observed"
        );
        eprintln!("OWNERSHIP_EDIT members={N} edit={value} rows={} content_index_bytes={} partition_value_bytes={} payload_bytes={} visibility_bytes={} commits={:?} elapsed_us={micros}",work.ownership_rows,work.content_index_hash_bytes,work.partition_value_hash_bytes,work.commit_payload_hash_bytes,work.visibility_payload_hash_bytes,work.published_rows);
        rows.push((work.ownership_rows, work.published_rows));
    }
    rows
}
#[test]
fn fixed_membership_change_writes_flat_rows_as_unchanged_publication_doubles() {
    let small = measure::<32>();
    let large = measure::<64>();
    for ((small, small_commits), (large, large_commits)) in small.iter().zip(large) {
        assert_eq!(
            small_commits, &large_commits,
            "fixed change must publish the same canonical row counts"
        );
        assert!(
            large * 2 < small * 3,
            "unchanged custody must not amplify writes: {small} -> {large}"
        );
    }
}
