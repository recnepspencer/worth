//! History's memory profile refuses unsupported journal replay before any effect.
use super::super::unaccessed_medium;
use super::*;
use worth_relational::facade::mvcc::{
    RelationalOwnedGraphSelectionPreparationError as Error, RelationalPublicationOutcome,
};

#[test]
fn graph_selection_with_an_attached_medium_refuses_before_any_medium_contact() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let source = installed_runtime();
    let model = create_entity(execution, &source, "model");
    let (_, basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let graph = source
        .prepare_owned_model_graph_selection(
            execution,
            &basis,
            &basis,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap();
    let durable = RelationalRuntimeApi::builder()
        .schema_registry(registry())
        .build_durable(Box::new(unaccessed_medium::UnaccessedMedium))
        .unwrap();
    let error = durable
        .preparation_port()
        .admit_owned_graph_selection(execution, begin(&durable), graph)
        .unwrap_err();
    assert!(
        matches!(error, Error::DurabilityUnsupported { model: subject } if subject == model),
        "{error:?}"
    );
    assert_eq!(
        durable
            .admit_branch_basis(&durable.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        None
    );
}

#[test]
fn graph_selection_raw_tail_replay_refuses_its_named_profile_without_installing_selection() {
    let host = serial_request();
    let execution = &ExecutionRequest::serial(&host);
    let source = installed_runtime();
    let model = create_entity(execution, &source, "model");
    let (_, basis) = source
        .observe_branch(&source.main_branch_identity())
        .unwrap();
    let graph = source
        .prepare_owned_model_graph_selection(
            execution,
            &basis,
            &basis,
            &ownership(),
            &ownership(),
            model,
        )
        .unwrap();
    let port = source.preparation_port();
    let input = port
        .admit_owned_graph_selection(execution, begin(&source), graph)
        .unwrap();
    let candidate = port
        .prepare_owned_graph_selection(execution, input, None)
        .unwrap();
    let RelationalPublicationOutcome::Performed(performed) = source
        .publication_port()
        .compare_and_publish(execution, candidate)
    else {
        panic!("memory graph selection must publish")
    };
    let committed = source.settle_performed_publication(performed).unwrap();
    let plan = source
        .durability()
        .recovery_plan(
            execution,
            RecoveryVerificationMode::NormalRecoveryVerification,
        )
        .unwrap();
    let mut cold = installed_runtime();
    let RelationalRecoveryStop::Durability(cause) = cold
        .durability_recovery()
        .recover(execution, plan)
        .unwrap_err()
    else {
        panic!("raw graph replay must retain its specific refusal")
    };
    assert_eq!(
        cause.class,
        RecoveryFailureClass::UnsupportedOwnedGraphSelectionReplay
    );
    assert_ne!(
        cold.admit_branch_basis(&cold.main_branch_identity())
            .unwrap()
            .observation()
            .commit_id(),
        Some(committed.commit.commit_id)
    );
    release_test_commit_snapshot(&source, &committed);
}
