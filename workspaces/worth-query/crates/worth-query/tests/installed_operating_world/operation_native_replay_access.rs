use worth_foundational::facade::{FieldKey, InternedString};
use worth_query::facade::{certification, domain, foundation};

use super::installed_operation_fixture::{
    workflow_workspace, GeometryDomain, ReadFamily, WorkflowRead,
};
use super::operation_reexecution::intent;

#[test]
fn ordinary_publication_consumption_converges_without_receiving_replay_authority() {
    let mut workspace = workflow_workspace("replay-publication-parity").unwrap();
    let original_bound = bind(&workspace);
    let (original_request, original_keys) =
        native_replay_request(original_bound.consumer_projection_contract().unwrap());
    let original = original_bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    let replay_bound = bind(&workspace);
    let replay = certification::replay_installed_workflow(
        certification::issue_query_certification_replay_capability(),
        &original,
        replay_bound,
        intent(),
        crate::suite::installed_operation_fixture::execution_resource_request(),
        &mut workspace,
    )
    .unwrap();
    let ordinary_bound = bind(&workspace);
    let (ordinary_request, ordinary_keys) =
        native_replay_request(ordinary_bound.consumer_projection_contract().unwrap());
    let ordinary = ordinary_bound
        .admit_workflow_resources(
            crate::suite::installed_operation_fixture::execution_resource_request(),
            &workspace,
        )
        .unwrap()
        .reexecute(intent(), &mut workspace)
        .unwrap();
    assert_eq!(
        domain::compare_exact_workflow_traces(
            replay.replay_semantics(),
            &ordinary.semantics(),
            Default::default(),
        ),
        domain::WorthQueryReplayComparison::Equivalent
    );
    assert_ne!(replay.replay_trace_identity(), ordinary.identity());

    let original_settled = original
        .publish()
        .unwrap()
        .consume_bound(original_request)
        .unwrap()
        .settle()
        .unwrap();
    let ordinary_settled = ordinary
        .publish()
        .unwrap()
        .consume_bound(ordinary_request)
        .unwrap()
        .settle()
        .unwrap();
    assert_eq!(original_settled.counters(), ordinary_settled.counters());
    assert_eq!(
        original_settled.native_access_binding_counters(),
        ordinary_settled.native_access_binding_counters()
    );
    assert_eq!(original_settled.warnings(), ordinary_settled.warnings());
    assert_ne!(original_settled.identity(), ordinary_settled.identity());
    for (original_key, ordinary_key) in original_keys.iter().zip(&ordinary_keys) {
        let original_access = original_settled.native_value(original_key, 0).unwrap();
        let ordinary_access = ordinary_settled.native_value(ordinary_key, 0).unwrap();
        assert_eq!(
            original_access.fact().as_interned_string(),
            Ok(&InternedString::Raw("synthetic-anchor".into()))
        );
        assert_eq!(ordinary_access.value(), original_access.value());
        assert_eq!(ordinary_access.counters(), original_access.counters());
        assert_eq!(original_access.counters().indexed_accesses, 1);
        assert_eq!(original_access.counters().refinement_checks, 1);
        assert_eq!(original_access.counters().fact_scans, 0);
        assert_eq!(original_access.counters().row_scans, 0);
        assert_eq!(original_access.counters().path_parses, 0);
        assert_eq!(
            ordinary_settled
                .native_value(original_key, 0)
                .unwrap_err()
                .kind(),
            domain::WorthQueryNativeAccessDenialKind::CapabilityMismatch
        );
        assert_eq!(
            original_settled
                .native_value(ordinary_key, 0)
                .unwrap_err()
                .kind(),
            domain::WorthQueryNativeAccessDenialKind::CapabilityMismatch
        );
    }
}

fn native_replay_request<D, O, F, L: foundation::BasisOperationLane>(
    consumer: domain::WorthQueryConsumerProjectionContract<D, O, F, L>,
) -> (
    domain::WorthQueryBoundProjectionRequest<D, O, F, L>,
    Vec<domain::WorthQueryNativeAccessKey>,
) {
    let mut builder = consumer.projection_request();
    let display = builder
        .select_display_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let derived = builder
        .select_derived_native_field(FieldKey::new("id").unwrap())
        .unwrap();
    let request = builder.build().unwrap();
    let keys = [display, derived]
        .iter()
        .map(|selection| request.resolve_native_key(selection).unwrap().into_key())
        .collect();
    (request, keys)
}

fn bind(
    workspace: &worth_query::facade::runtime::WorthQueryWorkspace,
) -> domain::WorthQueryBoundDomainOperation<
    GeometryDomain,
    WorkflowRead,
    ReadFamily,
    foundation::ObservationLaneWitness,
> {
    let installed = workspace.domain(GeometryDomain).unwrap();
    workspace
        .observe_operating_world(workspace.current_world())
        .unwrap()
        .family(ReadFamily)
        .bind(&installed, WorkflowRead)
        .unwrap()
}
