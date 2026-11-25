//! A later synchronous producer refuses the source and the earlier writer too.
use super::*;
use worth_query_host::facade::runtime::ExecutionAllocationPolicy;

pub(in crate::certification::document_history::owned_outputs) fn checkpoint<
    P: ApplicationProgramDefinition<Schema>,
>(
    host: &world::DirHost<P>,
) -> Vec<u8> {
    host.capture_native_checkpoint_frame()
}

/// Complete World + Relational + Query commitment for MemoryOnly courts whose
/// setups hit intentional Directory Home guards (controlled synthetic clocks,
/// non-durable ownership material, or `PreparingQueryCandidate` pending open).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::certification::document_history::owned_outputs) struct MemoryBasisCommitment {
    world_ordinal: u64,
    descriptor: worth_relational::facade::branch::RelationalBranchBasisDescriptor,
    primary_truth_snapshot: String,
    world_history_state: (usize, usize, String),
    output_checkpoint_snapshot_state: (usize, usize),
    registry_row_count: usize,
    published_required_rows: Vec<(String, usize, bool, String)>,
    undelivered_publication_counts: (usize, usize),
    retained_source_custody_count: usize,
    prepared_required_output_source_count: usize,
    active_attempts: (usize, usize),
    active_live_consumers: usize,
    indexed_product_branch_count: usize,
}

pub(in crate::certification::document_history::owned_outputs) fn memory_basis_state<
    P: ApplicationProgramDefinition<Schema>,
>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
) -> MemoryBasisCommitment {
    host.with_host_advancement(|phase| {
        let retained =
            worth_query_host::facade::application_entry::WorthQueryApplicationReadObservation::retain_on_branch(
                &phase,
                host.runtime(),
                host.current_world(),
            )
            .unwrap();
        let selected = retained.select_on(&phase, host.runtime()).unwrap();
        let rt = host.runtime();
        let history = rt.world_history_snapshot_for_test();
        MemoryBasisCommitment {
            world_ordinal: retained.selected_commit().ordinal(),
            descriptor: selected.product().relational_basis_descriptor().clone(),
            primary_truth_snapshot: format!("{:?}", rt.primary_truth_snapshot_for_test()),
            world_history_state: (
                history.installed_commits(),
                history.reserved_commits(),
                format!("{:?}", history.metadata()),
            ),
            output_checkpoint_snapshot_state: rt.output_checkpoint_snapshot_state_for_test(),
            registry_row_count: rt.registry_row_count_for_test(),
            published_required_rows: rt.published_required_rows_for_test(),
            undelivered_publication_counts: rt.undelivered_publication_counts_for_test(),
            retained_source_custody_count: rt.retained_source_custody_count_for_test(),
            prepared_required_output_source_count: rt
                .prepared_required_output_source_count_for_test(),
            active_attempts: (
                rt.active_application_attempts_for_test(),
                rt.world_active_publication_attempts_for_test(),
            ),
            active_live_consumers: rt.active_live_consumers_for_test(),
            indexed_product_branch_count: rt.indexed_product_branch_count_for_test(),
        }
    })
    .unwrap()
}

pub(super) fn lose_next_commit_answer<P: ApplicationProgramDefinition<Schema>>(
    host: &world::DirHost<P>,
) {
    host.runtime().lose_next_commit_answer_for_test();
}

pub(in crate::certification::document_history::owned_outputs) fn assert_memory_basis_unchanged<
    P: ApplicationProgramDefinition<Schema>,
>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    before: &MemoryBasisCommitment,
) {
    assert_eq!(
        &memory_basis_state(host),
        before,
        "world ordinal and Native truth/schema/visibility roots must remain unchanged"
    );
}

fn second_refuses(value: u64) -> u64 {
    if value == 2 {
        u64::MAX
    } else {
        second::derived(value)
    }
}

fn refuses_without_effect(worker: bool) {
    let limits = if worker {
        world::worker_limits()
    } else {
        host_limits(worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system())
    };
    let host = world::open_directory_configured::<program::Program>(
        limits,
        1,
        None,
        world::Configuration {
            reconciliation_indices: false,
            reconciliation_peer_caps: false,
            reconciliation_emission: false,
            envelope_records: false,
            second_derivation: Some(second_refuses),
            ..Default::default()
        },
    );
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    first::settle(&host, &request, "a");
    second::settle(&host, &request, "a");
    let original = (
        world::model(&host, host.current_world()),
        world::outputs(&host, host.current_world()),
    );
    let contacts = host.producer_contacts_on_this_thread_for_test();
    let before = checkpoint(&host);
    let outcome = request
        .advanced()
        .mutate(Intent {
            key: "a".into(),
            value: 2,
        })
        .without_source()
        .idempotency(&"refused-synchronous-edit".to_owned())
        .execute_in_program(&host, ExecutionAllocationPolicy::SystemAllocation)
        .unwrap();
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("the complete edit must refuse: {outcome:?}")
    };
    let producer = denial
        .coupled_producer_denial()
        .expect("the true producer's typed refusal");
    assert_eq!(
        producer.kind(),
        WorthQueryOutputDemandDenialKind::ProducerDomainDenied
    );
    assert!(
        producer.subject().contains("owned-outputs.second.preserve"),
        "{producer:?}"
    );
    assert_eq!(producer.domain_reason(), Some("DerivedValueOutOfRange"));
    assert_eq!(
        host.producer_contacts_on_this_thread_for_test(),
        contacts + 2,
        "failure follows one actual earlier writer"
    );
    assert_eq!(
        (
            world::model(&host, host.current_world()),
            world::outputs(&host, host.current_world())
        ),
        original
    );
    assert_model_unchanged(&host, &before);
}

#[test]
fn a_later_producer_refusal_keeps_the_complete_model_byte_identical() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_without_effect(false);
}
#[test]
fn worker_only_later_producer_refusal_keeps_the_complete_model_byte_identical() {
    let _observation = WorthQueryExecutionObservation::install_on_this_thread();
    refuses_without_effect(true);
}

#[path = "failure/coupled_durability.rs"]
mod coupled_durability;

pub(in crate::certification::document_history::owned_outputs) fn assert_model_unchanged<
    P: ApplicationProgramDefinition<Schema>,
>(
    host: &world::DirHost<P>,
    before: &[u8],
) {
    let after = checkpoint(host);
    worth_query_testkit_document_retention::facade::directory_home::assert_native_checkpoint_model_unchanged(
        before, &after,
    );
}
