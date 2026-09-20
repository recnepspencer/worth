//! Performed effect recovery across branch-local program adoption.

use bank_domain::estate::EstateAction;
use bank_external_rail::test_control::FaultScript;
use bank_external_rail::LedgerStatus;
use bank_server::{BankApplicationP1, BankCommitDenialKind, BankMutationCommitOutcome};
use worth_query_host::facade::application_entry::WorthQueryBranchAdoptionPublicationOutcome;
use worth_query_host::facade::application_installation::WorthQueryProgramOwner;
use worth_query_host::facade::declaration::application_program::ApplicationSemanticChangeKind;
use worth_query_host::facade::domain::WorthQueryProgramCustodyInventoryKind;
use worth_query_host::facade::primary_graph::{
    WorthQueryProgramCustodyDispositionInventory, WorthQueryProgramCustodyDispositionKind,
};

use super::phase8_cross_gate::world;
use crate::support::request_scope;

#[test]
fn performed_effect_recovery_survives_removal_from_the_current_program() {
    let world = world::cross_gate_world("safe-retry-after-program-removal");
    world
        .transport
        .under(FaultScript::CommitThenLoseResponse, world::PATIENT);
    let receipt = world.commit_notification(88);
    let correlation = world.transport.attempts()[0].clone();
    let handle = world.open_recovery(&receipt);
    let specialist = world.fixture.authenticate_specialist();
    let application = world.fixture.world.runtime.application_program();
    let branch = application.current_world();
    let target_owner = application
        .supported_program::<BankApplicationP1>()
        .expect("Bank P1 is rostered beside P0");
    let target = target_owner.owned_revision().clone();
    let adoption_scope = request_scope();
    let programs = world
        .fixture
        .world
        .runtime
        .request(&specialist, &adoption_scope)
        .on_branch(branch)
        .programs();
    let requirements = programs.compare(&target).expect("P0 to P1 compares");
    let recovery = requirements
        .custody_inventory_requirements()
        .iter()
        .find(|requirement| {
            requirement.kind() == WorthQueryProgramCustodyInventoryKind::ExternalEffectRecovery
                && requirement.change() == ApplicationSemanticChangeKind::Removed
        })
        .expect("removed effectful operation requires exact recovery custody");
    let prepared = programs
        .adopt(&target)
        .requirements(&requirements)
        .prepare(128)
        .expect("Bank P1 adoption prepares");
    assert!(has_exact_recovery(prepared.custody(), recovery));
    let performed = match prepared.publish() {
        WorthQueryBranchAdoptionPublicationOutcome::Performed(performed) => performed,
        _ => panic!("Bank P1 adoption must publish"),
    };
    assert!(has_exact_recovery(performed.custody(), recovery));

    let attempts_before_ordinary = world.transport.attempts().len();
    let ordinary = world.fixture.world.runtime.notify_estate_death_with_key(
        &specialist,
        EstateAction::NotifyDeath {
            estate: world.fixture.second_estate,
            notice: world.fixture.second_notice,
            subject: world.fixture.other_subject,
        },
        &world::idempotency(89),
        &request_scope(),
    );
    assert!(
        matches!(
            &ordinary,
            Ok(BankMutationCommitOutcome::Denied {
                kind: BankCommitDenialKind::ProgramNotActiveOnOccurrence,
                ..
            })
        ),
        "removed ordinary operation returned {ordinary:?}"
    );
    assert_eq!(world.transport.attempts().len(), attempts_before_ordinary);

    world.transport.under(FaultScript::Succeed, world::PATIENT);
    let recovered = world
        .fixture
        .world
        .runtime
        .safe_retry_commit_recovery(
            handle,
            &specialist,
            world.specialist_action(),
            &request_scope(),
        )
        .expect("the exact P0 effect remains recoverable after P1 removes ordinary admission");
    assert!(recovered.is_external_completion());
    assert_eq!(
        world.transport.ledger_status(&correlation),
        LedgerStatus::Completed
    );
    assert_eq!(world.transport.admission_count(), 1);
    let dispatches = world.transport.production_dispatches();
    assert_eq!(dispatches.len(), 2);
    assert_eq!(dispatches[0].correlation, dispatches[1].correlation);
    assert_eq!(dispatches[0].payload, dispatches[1].payload);
    assert_eq!(world.transport.completed_effect_count(), 1);
}

fn has_exact_recovery(
    inventory: &WorthQueryProgramCustodyDispositionInventory,
    requirement: &worth_query_host::facade::domain::WorthQueryProgramCustodyInventoryRequirement,
) -> bool {
    inventory.dispositions().iter().any(|disposition| {
        disposition.requirement() == requirement
            && disposition.kind()
                == WorthQueryProgramCustodyDispositionKind::RetainExactEffectRecovery
    })
}
