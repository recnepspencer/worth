//! Historical V1 owners remain pinned while an independent V2 installation adopts.
use super::super::{
    derived_query as old_query, host as old_host, output as old_output, producer as old_producer,
    program as old_program, schema as old_schema,
};
use super::*;
use crate::document_retention_model::operator_identity::{authenticate_operator, request_scope};
use worth_execution::{CancellationSource, ExecutionRequest, ExecutionWorkCeiling, LeaseRequest};
use worth_foundational::{ExecutionBudget, ExecutionRequestPolicy};
use worth_query_host::facade::application_installation::ApplicationBuildIdentity;
use worth_query_host::facade::{
    primary_graph::{
        WorthQueryInvariantProjectionDenialKind, WorthQueryProductBranchAdmissionDenial,
        WorthQueryProductBranchAdmissionRefusalKind, WorthQueryValueDecodeDenial,
    },
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn held_v1_result_snapshot_output_and_observation_survive_independent_v2_adoption() {
    let authored = model::authored_value();
    let (predecessor, projection) = old_host::install_with_projection(authored.clone()).unwrap();
    let old_catalogue = predecessor.installed_schema().value_bindings().clone();
    let source_id = old_host::source_id(&predecessor);
    let scope = request_scope();
    let principal = authenticate_operator(predecessor.installed_schema(), &scope);
    let runtime = predecessor.runtime();
    let request = runtime
        .request(&principal, &scope)
        .on_branch(runtime.current_world());
    let mut old_demand = request
        .demand(old_producer::Demand)
        .start_in_program::<_, old_program::Root>(&predecessor)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(old_settled) =
        old_demand.settle(&request).unwrap()
    else {
        panic!("the actual V1 producer must publish before retaining historical readers");
    };
    assert_eq!(old_settled.producer_contacts_in_this_demand(), 1);
    let old_output_id = old_settled
        .outputs_of::<old_output::DerivedOutputs>()
        .unwrap()
        .entity::<old_output::CreatedOutput>()
        .unwrap()
        .entity_id();
    let old_observation = old_settled.observation().retain();
    let old_result = request
        .at(&old_observation)
        .query(old_query::ReadValue {
            identity: "derived-report".to_owned(),
        })
        .execute()
        .unwrap();
    assert_v1_literals(&old_result.rows()[0]);
    let projected = runtime
        .with_application_advancement(&scope, |phase| {
            projection.project(
                &phase,
                |reader| {
                    reader
                        .resolve_entity(
                            old_schema::DerivedIdentityField::reference(),
                            "derived-report".to_owned(),
                        )
                        .unwrap()
                },
                ExecutionAllocationPolicy::SystemAllocation,
            )
        })
        .unwrap()
        .unwrap();
    let (old_identity, old_snapshot, _) = projected.into_parts();
    assert_eq!(old_identity.entity_id(), old_output_id);
    let old_version = old_snapshot.version();
    let (checkpoint, sections) = runtime
        .with_host_advancement(|phase| {
            runtime
                .capture_application_checkpoint_with_sections(
                    &phase,
                    CapturePolicy::SystemAllocation,
                )
                .unwrap()
        })
        .unwrap();
    assert_eq!(sections.accepted_output_count(), 1);
    let original_bytes = checkpoint.bytes().to_vec();
    let retained_checkpoint = checkpoint.clone();
    let expected_predecessor = installation::WorthQueryCheckpointProgramPredecessor::new(
        &old_host::program().revision().to_string(),
    )
    .unwrap();
    output::reset_decodes();
    let application = installation::in_memory_rostered_program_from_checkpoint_with_transition(
        host::program(),
        installation::WorthQueryApplicationProgramRoster::new(),
        schema::CanonicalSchema::declaration().unwrap(),
        ((),),
        ApplicationBuildIdentity::parse("11111111-1111-1111-1111-111111111111").unwrap(),
        host::limits(),
        checkpoint,
        expected_predecessor,
        installation::WorthQueryCheckpointTransitionResources::bounded(4096, 32, 16384).unwrap(),
        CapturePolicy::SystemAllocation,
        |_, _| Ok(()),
    )
    .expect("actual derived provenance admits V2 while V1's historical holders remain live");
    assert_eq!(output::decodes(), 0);
    assert!(old_catalogue.same_installation_as(predecessor.installed_schema().value_bindings()));
    assert!(!old_catalogue.same_installation_as(application.installed_schema().value_bindings()));
    assert_v1_literals(&old_result.rows()[0]);
    let historical = request
        .at(&old_observation)
        .query(old_query::ReadValue {
            identity: "derived-report".to_owned(),
        })
        .execute()
        .unwrap();
    assert_v1_literals(&historical.rows()[0]);
    let pinned = runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            old_snapshot
                .field(
                    &execution,
                    &old_identity,
                    old_schema::DerivedValueField::reference(),
                )
                .unwrap()
                .unwrap()
        })
        .unwrap();
    assert_v1_literals(&pinned);
    assert_eq!(old_snapshot.version(), old_version);
    assert_eq!(
        old_settled
            .outputs_of::<old_output::DerivedOutputs>()
            .unwrap()
            .entity::<old_output::CreatedOutput>()
            .unwrap()
            .entity_id(),
        old_output_id
    );
    let foreign = application
        .runtime()
        .with_host_advancement(|phase| {
            old_observation
                .select_on(&phase, application.runtime())
                .err()
                .expect("the actual V1 retained observation cannot select the V2 owner")
        })
        .unwrap();
    assert!(matches!(
        foreign,
        WorthQueryProductBranchAdmissionDenial::Refused(
            WorthQueryProductBranchAdmissionRefusalKind::ForeignOwner
        )
    ));
    assert_eq!(
        output::decodes(),
        0,
        "historical V1 reads and foreign-owner denial never enter V2 decoding"
    );
    runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            let read_domain_output = |cancel_at_handoff: bool| {
                let cancellation = CancellationSource::new();
                let child = execution
                    .in_scope(|parent| {
                        let parent = parent.expect("the original V1 phase lends its actual lease");
                        parent
                            .child(LeaseRequest {
                                policy: ExecutionRequestPolicy::new(
                                    parent.policy().posture(),
                                    parent.policy().determinism(),
                                    ExecutionBudget::new(
                                        std::num::NonZeroUsize::MIN,
                                        parent.policy().charged_memory_bytes(),
                                        parent.policy().work_ceiling(),
                                    ),
                                ),
                                cancellation: cancellation.token(),
                                deadline: scope.deadline(),
                            })
                            .unwrap()
                    })
                    .unwrap();
                let request = ExecutionRequest::leased(&child);
                request
                    .run(
                        ExecutionWorkCeiling::new(child.policy().work_ceiling()),
                        |_| {
                            old_snapshot.read_fields(&request, |reader| {
                                let value = reader
                                    .field(
                                        &old_identity,
                                        old_schema::DerivedValueField::reference(),
                                    )
                                    .unwrap()
                                    .unwrap();
                                assert_v1_literals(&value);
                                if cancel_at_handoff {
                                    cancellation.cancel();
                                }
                                Err::<(), _>("historical-domain-refusal")
                            })
                        },
                    )
                    .expect("the same child owns the original historical read and its handoff")
                    .0
            };
            let cancelled = read_domain_output(true)
                .expect_err("native cancellation must veto even an application Err output");
            assert_eq!(
                cancelled.kind(),
                WorthQueryInvariantProjectionDenialKind::ValueDecode(
                    WorthQueryValueDecodeDenial::Scope(
                        worth_execution::WorkCeilingDenial::Stopped(
                            worth_execution::MapKernelStop::Cancelled
                        )
                    )
                )
            );
            assert_eq!(
                read_domain_output(false).expect("a fresh funded child reads the same native pin"),
                Err("historical-domain-refusal")
            );
        })
        .unwrap();
    assert_eq!(old_snapshot.version(), old_version);
    assert_eq!(
        output::decodes(),
        0,
        "the original-request cancellation twin stays in V1's decoder owner"
    );
    let closed = runtime.close_live_delivery();
    assert!(closed.owner_terminal());
    let pinned_after_close = runtime
        .with_application_advancement(&scope, |phase| {
            let execution = phase
                .checked_execution_request_for(runtime.product_runtime())
                .unwrap();
            old_snapshot
                .field(
                    &execution,
                    &old_identity,
                    old_schema::DerivedValueField::reference(),
                )
                .unwrap()
                .unwrap()
        })
        .unwrap();
    assert_v1_literals(&pinned_after_close);
    assert_eq!(output::decodes(), 0);
    let fresh_scope = request_scope();
    let fresh_principal = authenticate_operator(application.installed_schema(), &fresh_scope);
    let fresh = application
        .runtime()
        .request(&fresh_principal, &fresh_scope)
        .on_branch(application.runtime().current_world());
    let mut demand = fresh
        .demand(producer::Demand)
        .start_in_program::<_, program::Root>(&application)
        .unwrap();
    let WorthQueryApplicationOutputDemandProgress::Settled(settled) =
        demand.settle(&fresh).unwrap()
    else {
        panic!("the independent V2 producer reconstructs its retired native output");
    };
    assert_eq!(settled.producer_contacts_in_this_demand(), 1);
    let new_output_id = settled
        .outputs_of::<output::DerivedOutputs>()
        .unwrap()
        .entity::<output::CreatedOutput>()
        .unwrap()
        .entity_id();
    assert_ne!(new_output_id, old_output_id);
    let produced = host::read_derived(&application);
    assert_eq!(produced.0.caption, "ASSEMBLY Β");
    assert_eq!(produced.0.total_rank, 12);
    assert_eq!(produced.0.labels, ["left", "right"]);
    drop(demand);
    assert_edit_preserves_settled_output(&application, &settled);
    let current = application
        .runtime()
        .with_host_advancement(|phase| {
            application
                .runtime()
                .capture_application_checkpoint(&phase, CapturePolicy::SystemAllocation)
                .unwrap()
        })
        .unwrap();
    drop(application);
    let reopened = host::restore(current);
    assert_eq!(host::source_id(&reopened), source_id);
    let value = host::read_derived(&reopened);
    assert_eq!(value.0.caption, "REGENERATED Γ");
    assert_eq!(value.0.total_rank, 37);
    assert_eq!(value.0.labels, ["updated-left", "right"]);
    assert_v1_literals(&old_result.rows()[0]);
    assert_eq!(old_snapshot.version(), old_version);
    assert_eq!(retained_checkpoint.bytes(), original_bytes);
    drop(old_demand);
    drop(predecessor);
}

fn assert_v1_literals(value: &old_output::DerivedValue) {
    assert_eq!(value.caption, "ASSEMBLY Β");
    assert_eq!(value.rank_sum, 12);
    assert_eq!(value.tags, ["left", "right"]);
}
