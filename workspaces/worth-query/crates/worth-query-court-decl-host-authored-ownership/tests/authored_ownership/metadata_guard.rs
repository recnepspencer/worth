//! Every private roster and position kind has one Native writer.
use super::{
    correction_courts::install_definition,
    coupled::multi::{create, settle, TwoProgram},
    lifecycle::*,
    *,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::*;
use worth_relational::facade::{
    publication::AuthoredRetirementRefusal, transactions::TransactionCommitError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MemoryBasisCommitment {
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

pub(super) fn memory_basis_state<
    S: Scenario,
    P: ApplicationProgramDefinition<S, Contributions = (Contribution,)> + 'static,
>(
    host: &WorthQueryProgramApplicationRuntime<S, P>,
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

#[test]
fn ordinary_create_update_and_delete_of_each_current_metadata_kind_refuse_before_effect() {
    type S = Model<6>;
    // TwoProgram declares correction output material, which Directory Home refuses
    // with UnsupportedCorrectionOutputMaterial; this court tests Native's in-process
    // metadata write guard on MemoryOnly and checks full World + Native + Query
    // basis commitment equality before and after the refused probes.
    let host = install_definition::<S, TwoProgram>();
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.runtime().request(&principal, &scope);
    let body = create(&host, &request, 10, vec![20]);
    settle(&host, &request, 10);
    let owner = body
        .outputs_of::<CreatedOutputs>()
        .unwrap()
        .entity::<Created<S>>()
        .unwrap()
        .entity_id();
    let before = memory_basis_state(&host);
    let refusals = host
        .runtime()
        .probe_current_metadata_writes_for_test(body.product_branch(), owner, &scope)
        .unwrap();
    assert_eq!(
        refusals.len(),
        28,
        "three ordinary verbs for the nine realized kinds, and creation of the input link this body never consumes"
    );
    for (kind, verb, result) in refusals {
        let error = result.expect_err("ordinary metadata writes never prepare");
        let TransactionCommitError::Publication { error, .. } = error else {
            panic!("{kind:?}/{verb}: {error:?}")
        };
        assert_eq!(
            error.authored_retirement_refusal(),
            Some(AuthoredRetirementRefusal::OrdinaryMetadataWrite),
            "{kind:?}/{verb}: {error:?}"
        );
    }
    let after = memory_basis_state(&host);
    assert_eq!(
        after, before,
        "ordinary metadata refusal changes no World, Native, or Query state"
    );
    for id in [10, 20, 101, 102] {
        assert!(request.query(query::Input { id }).execute().is_ok());
    }
}
