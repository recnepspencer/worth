//! Fresh operation admission observes the current action before output work.
use super::*;
use crate::application_program::{
    fixed_successors::{ChangedFixed, RemovedFixed},
    ConsumerFixedAction,
};
use worth_query_decl::facade::application_program::ApplicationProgramAuthoring;
use worth_query_host::facade::{
    application_entry::{
        WorthQueryOperationAdvance as Advance, WorthQueryOperationAdvanceDenial as Denial,
        WorthQueryOperationProgress as Progress,
    },
    application_installation::WorthQueryApplicationProgramRoster,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};
#[test]
fn fixed_fresh_readmission_rejects_removed_action_before_output_dispatch() {
    refuse_changed_action(false);
}
#[test]
fn fixed_fresh_readmission_rejects_changed_metadata_before_output_dispatch() {
    refuse_changed_action(true);
}
fn refuse_changed_action(metadata_changed: bool) {
    let changed = ApplicationProgramAuthoring::<ConsumerSchema, ChangedFixed>::begin()
        .with_schema(&ConsumerSchema::declaration().expect("the completed schema is valid"))
        .validated_program()
        .unwrap();
    let changed_revision = *changed.revision();
    let removed = ApplicationProgramAuthoring::<ConsumerSchema, RemovedFixed>::begin()
        .with_schema(&ConsumerSchema::declaration().expect("the completed schema is valid"))
        .validated_program()
        .unwrap();
    let removed_revision = if metadata_changed {
        changed_revision
    } else {
        *removed.revision()
    };
    let original = ApplicationProgramAuthoring::<ConsumerSchema, crate::ConsumerProgram>::begin()
        .with_schema(&ConsumerSchema::declaration().expect("the completed schema is valid"))
        .validated_program()
        .unwrap();
    let original_revision = *original.revision();
    let grants = [
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<crate::ConsumerProgram>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<RemovedFixed>(
            "model-owner",
            "model-owner",
        ),
        crate::program_deployment::ConsumerDeploymentGrantSeed::for_program::<ChangedFixed>(
            "model-owner",
            "model-owner",
        ),
    ];
    let world = installation::install_with_roster(
        &crate::installed_schema(),
        WorthQueryApplicationProgramRoster::new()
            .support(removed)
            .support(changed),
        &grants,
    )
    .unwrap();
    let adapter = authentication::admit(world.application.installed_schema());
    let scope = authentication::request_scope();
    let principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &scope,
    ))
    .unwrap();
    let request = world.application.request(&principal, &scope);
    let input = PlanarSourceAdjustment {
        scope_key: "anchor-b".to_owned(),
        replacement_y: length(2),
    };
    reset_planar_source_adjustment_contacts(&input);
    let operation = request
        .mutate(input.clone())
        .expect_source(
            request
                .query(PlanarRead {
                    body_key: input.scope_key.clone(),
                })
                .execute()
                .unwrap()
                .observed_sources()[0]
                .clone(),
        )
        .idempotency_owned(10_196)
        .unwrap_or_else(|(_, _, d)| panic!("capture: {d:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, d)| panic!("construction: {d:?}"));
    let operation = match operation.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("the genuine source publishes before the action changes"),
    };
    let adopt = |revision| {
        let programs = request.programs();
        let requirements = programs.compare(&revision).unwrap();
        let prepared = programs
            .adopt(&requirements)
            .unwrap()
            .prepare(4096)
            .unwrap();
        assert!(matches!(request.programs().publish(prepared),worth_query_host::facade::primary_graph::WorthQueryBranchAdoptionPublicationOutcome::Performed(_)));
    };
    adopt(removed_revision);
    let fresh_scope = authentication::request_scope();
    let fresh_principal = authentication::block_on(adapter.authenticate(
        authentication::LocalCredential::issued_for_model_owner(),
        &fresh_scope,
    ))
    .unwrap();
    let fresh = world.application.request(&fresh_principal, &fresh_scope);
    let before = fresh
        .retain_current_read()
        .unwrap()
        .selected_commit()
        .clone();
    let before_data = fresh
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .rows()[0]
        .y;
    let operation = match operation.advance(&fresh) {
        Advance::Refused {
            operation,
            denial: Denial::Action(cause),
        } => {
            assert_eq!(
                cause,
                if metadata_changed {
                    worth_query_host::facade::domain::WorthQueryActionResolutionDenial::MeaningMismatch
                } else {
                    worth_query_host::facade::domain::WorthQueryActionResolutionDenial::Missing
                }
            );
            operation
        }
        Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
            panic!("wrong readmission boundary: {denial:?}")
        }
        _ => {
            panic!("operation fresh admission allowed output dispatch after its action was removed")
        }
    };
    assert_eq!(
        fresh.retain_current_read().unwrap().selected_commit(),
        &before
    );
    assert_eq!(
        fresh
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        before_data
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    adopt(original_revision);
    let mut operation = Some(operation);
    let settled = (0..32)
        .find_map(|_| match operation.take().unwrap().advance(&fresh) {
            Advance::Progress {
                operation: next, ..
            } => {
                operation = Some(next);
                None
            }
            Advance::Settled(settled) => Some(settled),
            Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                panic!("restored action: {denial:?}")
            }
        })
        .expect("same original owner progresses when the lawful action is current again");
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    settled
        .close(&fresh)
        .unwrap_or_else(|(_, d)| panic!("close: {d:?}"));
}
