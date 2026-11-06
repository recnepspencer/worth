//! The real Requested Home fences Native before any transition intent or effect.
use super::*;
use worth_relational::facade::{
    branch::{
        RelationalBranchArchiveDenial, RelationalBranchDeleteDenial,
        RelationalBranchTransitionStop, RelationalForkDenial,
    },
    durability::{RelationalDurablePublicationRefusal, RelationalHomeProgramWriteDenial},
    history::BranchId,
    mvcc::{RelationalBranchTransactionAdmissionDenial, RelationalTransactionIntent},
};

#[test]
fn a_requested_home_refuses_ordinary_native_and_live_branch_writers_before_any_intent_or_effect() {
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let expected = fixture
        .home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = fixture.home.relational().runtime();
    let identity = expected.basis().relational_basis().identity();
    assert!(fixture.home.home_program_transition_intent().is_none());
    assert!(matches!(
        native.begin_branch_transaction(
            expected.basis().relational_basis(),
            RelationalTransactionIntent::ordinary()
        ),
        Err(
            RelationalBranchTransactionAdmissionDenial::HomeProgramTransition(
                RelationalHomeProgramWriteDenial::TransitionRequired
            )
        )
    ));
    let target = BranchId("must-not-be-reserved".into());
    assert!(matches!(
        native.fork_port().reserve_fork_target(target.clone()),
        Err(RelationalForkDenial::DurablePublicationRefused(
            RelationalDurablePublicationRefusal::HomeProgramTransition(
                RelationalHomeProgramWriteDenial::TransitionRequired
            )
        ))
    ));
    let lifecycle = native.owner_component_services().lifecycle_port();
    assert!(matches!(
        lifecycle.archive_branch(&execution, identity),
        Err(RelationalBranchTransitionStop::Denied(
            RelationalBranchArchiveDenial::DurablePublicationRefused(
                RelationalDurablePublicationRefusal::HomeProgramTransition(
                    RelationalHomeProgramWriteDenial::TransitionRequired
                )
            )
        ))
    ));
    assert!(matches!(
        lifecycle.delete_branch(&execution, identity),
        Err(RelationalBranchTransitionStop::Denied(
            RelationalBranchDeleteDenial::DurablePublicationRefused(
                RelationalDurablePublicationRefusal::HomeProgramTransition(
                    RelationalHomeProgramWriteDenial::TransitionRequired
                )
            )
        ))
    ));
    assert!(native.branch_identity(&target).is_err());
    assert_eq!(
        fixture
            .home
            .world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        expected
    );
    drop((expected, lifecycle));
    close(fixture.home, &execution);
}
