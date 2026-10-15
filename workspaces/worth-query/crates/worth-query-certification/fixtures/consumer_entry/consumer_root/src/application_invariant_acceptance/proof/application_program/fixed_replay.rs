//! Ordinary fixed replay rejoins real registered output custody after reply loss.
use super::*;
use crate::application_program::ConsumerFixedAction;
use worth_query_host::facade::application_entry::{
    WorthQueryOperationAdvance as Advance, WorthQueryOperationProgress as Progress,
};
use worth_query_topology_entry::{
    planar_source_adjustment_contacts, reset_planar_source_adjustment_contacts,
};

#[test]
fn fixed_registered_reply_loss_resumes_without_handler_rerun() {
    let foreign = crate::installed_schema();
    let mut world = installation::install(&foreign);
    let scope = authentication::request_scope();
    let request_runtime = world.application.clone();
    let constructing = std::mem::replace(&mut world.application, request_runtime);
    let adapter = authentication::admit(world.application.installed_schema());
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
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let first = {
        let construction_scope = authentication::request_scope();
        let construction_principal = authentication::block_on(adapter.authenticate(
            authentication::LocalCredential::issued_for_model_owner(),
            &construction_scope,
        ))
        .unwrap();
        let construction_request =
            constructing.request(&construction_principal, &construction_scope);
        construction_request
            .mutate(input.clone())
            .expect_source(source.clone())
            .idempotency_owned(10_091)
            .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
            .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&constructing)
            .unwrap_or_else(|(_, cause)| panic!("construction: {cause:?}"))
    };
    drop(constructing);
    let first = match first.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("first source publication performs"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let original_commit = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .runtime()
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    drop(first);
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1
    );
    let replay = request
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency_owned(10_091)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("replay construction: {cause:?}"));
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceReplayObserved,
        } => operation,
        _ => panic!("replay retains actual committed evidence"),
    };
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::RegisteredOutputsRecovered,
        } => operation,
        Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
            panic!("registered replay denial: {denial:?}")
        }
        _ => panic!("registered replay recovers the actual native demand"),
    };
    let before_refusal = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .runtime()
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    let mut held = Vec::new();
    for _ in 0..64 {
        match request.retain_current_read() {
            Ok(observation) => held.push(observation),
            Err(_) => break,
        }
    }
    assert!(!held.is_empty());
    let replay = match replay.advance(&request) {
        Advance::Refused { operation, denial: worth_query_host::facade::application_entry::WorthQueryOperationAdvanceDenial::Observation(
            worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::Refused(
                worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionRefusalKind::ObservationCapacityExhausted
            )) } => operation,
        _ => panic!("real snapshot exhaustion retains the replay owner before recovery"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    drop(held);
    let after_refusal = world
        .application
        .with_host_advancement(|phase| {
            world
                .application
                .runtime()
                .on_branch(world.application.current_world())
                .select(&phase)
                .unwrap()
                .product()
                .selected_commit()
                .clone()
        })
        .unwrap();
    assert_eq!(before_refusal, after_refusal);
    let mut operation = Some(replay);
    let settled = (0..32)
        .find_map(|_| {
            let fresh_scope = authentication::request_scope();
            let fresh_principal = authentication::block_on(adapter.authenticate(
                authentication::LocalCredential::issued_for_model_owner(),
                &fresh_scope,
            ))
            .unwrap();
            match operation
                .take()
                .unwrap()
                .advance(&world.application.request(&fresh_principal, &fresh_scope))
            {
                Advance::Progress {
                    operation: next, ..
                } => {
                    operation = Some(next);
                    None
                }
                Advance::Settled(settled) => Some(settled),
                Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => {
                    panic!("registered progression: {denial:?}")
                }
            }
        })
        .expect("authored output graph settles");
    assert!(settled.handler_result().is_none());
    assert_eq!(settled.settlement().output_count(), 4);
    assert_eq!(
        settled.settlement().root_producer_contacts_in_this_demand(),
        1
    );
    assert_eq!(
        settled.read_current(&request).unwrap().rows()[0].value,
        length(3)
    );
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    let historical_output = request
        .at(settled.settlement().observation())
        .query(worth_query_topology_entry::PlanarOutputRead {
            body_key: "final:anchor-b".to_owned(),
        })
        .execute()
        .unwrap();
    assert_eq!(historical_output.rows()[0].value, length(4));
    drop(historical_output);
    settled
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("native cleanup: {cause:?}"));
    let historical = request
        .mutate(input.clone())
        .expect_source(source)
        .idempotency_owned(10_091)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("historical construction: {cause:?}"));
    let historical = match historical.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceReplayObserved,
        } => operation,
        _ => panic!("historical replay retains genuine committed source evidence"),
    };
    let terminal = match historical.advance(&request) {
        Advance::Terminal {
            operation,
            denial:
                worth_query_host::facade::application_entry::WorthQueryOperationAdvanceDenial::Outputs(
                    cause,
                ),
        } => {
            assert_eq!(cause.recovery_posture(), worth_query_host::facade::application_entry::WorthQueryRequiredOutputRecoveryPosture::Terminal);
            assert!(
                matches!(cause, worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(ref native)) if native.kind() == worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable)
            );
            operation
        }
        _ => panic!("completed registration cannot manufacture output completion"),
    };
    assert!(matches!(terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_))));
    let historical_receipt = terminal.native_outcome().unwrap().receipt().unwrap();
    assert_eq!(
        historical_receipt
            .committed_product_publication()
            .composite_commit(),
        &original_commit
    );
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("historical cleanup: {cause:?}"));
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
}

#[test]
fn fixed_unavailable_registration_returns_historical_evidence() {
    let foreign = crate::installed_schema();
    let world = installation::install(&foreign);
    let scope = authentication::request_scope();
    let adapter = authentication::admit(world.application.installed_schema());
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
    let source = request
        .query(PlanarRead {
            body_key: input.scope_key.clone(),
        })
        .execute()
        .unwrap()
        .observed_sources()[0]
        .clone();
    let first = request
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency_owned(10_091)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("construction: {cause:?}"));
    let first = match first.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourcePerformed,
        } => operation,
        _ => panic!("first source publication performs"),
    };
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
    drop(first);
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        1
    );
    let replay = request
        .mutate(input.clone())
        .expect_source(source.clone())
        .idempotency_owned(10_091)
        .unwrap_or_else(|(_, _, denial)| panic!("input capture: {denial:?}"))
        .retain_operation::<crate::ConsumerProgram, ConsumerFixedAction>(&world.application)
        .unwrap_or_else(|(_, cause)| panic!("replay construction: {cause:?}"));
    let replay = match replay.advance(&request) {
        Advance::Progress {
            operation,
            progress: Progress::SourceReplayObserved,
        } => operation,
        _ => panic!("replay retains actual committed evidence"),
    };
    let mut held = Vec::new();
    for _ in 0..64 {
        match request.retain_current_read() {
            Ok(observation) => held.push(observation),
            Err(cause) => {
                assert!(matches!(cause, worth_query_host::facade::application_entry::WorthQueryApplicationReadObservationDenial::ProductSelection(worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionDenial::Refused(worth_query_host::facade::primary_graph::WorthQueryProductBranchAdmissionRefusalKind::ObservationCapacityExhausted))));
                break;
            }
        }
    }
    assert_eq!(
        world
            .application
            .prepared_required_output_source_count_for_test(),
        0,
        "actual abandoned registration is no longer available"
    );
    drop(held);
    let terminal = match replay.advance(&request) {
        Advance::Terminal { operation, denial: worth_query_host::facade::application_entry::WorthQueryOperationAdvanceDenial::Outputs(
            worth_query_host::facade::application_entry::WorthQueryRequiredOutputPreparationDenial::Demand(
                worth_query_host::facade::application_entry::WorthQueryApplicationOutputDemandDenial::Demand(cause))) } => {
            assert_eq!(cause.kind(), worth_query_host::facade::primary_graph::WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable);
            operation
        },
        Advance::Refused { denial, .. } | Advance::Terminal { denial, .. } => panic!("wrong registration refusal: {denial:?}"),
        _ => panic!("absent native registration cannot invent output completion"),
    };
    assert!(matches!(terminal.native_outcome(), Some(worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::AlreadyCommitted(_))));
    assert_eq!(
        request
            .query(PlanarRead {
                body_key: input.scope_key.clone()
            })
            .execute()
            .unwrap()
            .rows()[0]
            .y,
        length(2)
    );
    terminal
        .close(&request)
        .unwrap_or_else(|(_, cause)| panic!("historical cleanup: {cause:?}"));
    assert_eq!(planar_source_adjustment_contacts(&input), (1, 1));
}

mod supersession;

mod historical;
