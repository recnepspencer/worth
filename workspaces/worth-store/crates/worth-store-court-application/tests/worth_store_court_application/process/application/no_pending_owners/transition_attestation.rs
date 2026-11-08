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

#[test]
fn home_planning_request_records_the_actual_owner_and_refuses_other_source_admissions_before_effect(
) {
    use worth_proof::HomeProgramTransitionAdmissionDenial as Admission;
    use worth_relational::facade::runtime::RelationalHomeProgramContractDenial as Denial;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    {
        let predecessor = open(directory.path(), &execution);
        let bootstrap = predecessor.bootstrap_root(&execution);
        let cancellation = worth_runtime_world::facade::RuntimeWorldCancellationSource::new();
        let prepared = predecessor.prepare_publication(
            &execution,
            bootstrap.product_branch().clone(),
            crate::batches::create(0),
            &cancellation.token(),
        );
        drop(
            predecessor
                .publish(&execution, prepared, &cancellation.token())
                .consume(),
        );
        drop(bootstrap);
        close(predecessor.home, &execution);
    }
    let fixture = super::transition_open_original::reopen(directory.path(), &execution);
    let home = fixture.home;
    let source = home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = home.relational().runtime();
    let admitted = home
        .admit_home_program_transition_planning_request(&source, &execution)
        .unwrap();
    assert_eq!(
        admitted.source().actor(),
        home.world().owner_identity().durable().unwrap()
    );
    assert_eq!(
        native
            .admit_home_program_transition_planning_request(
                &admitted,
                source.basis().relational_basis()
            )
            .unwrap(),
        admitted.source().actor()
    );
    // Re-observing one immutable root deliberately shares its admission.
    // The real earlier World commit provides a genuinely different source.
    let observation = home.world().observation_port();
    let mut cursor = observation
        .open_exact_product_history(execution, &source)
        .unwrap();
    let another_admission = loop {
        let prepared = observation
            .prepare_exact_product_history_page(execution, &cursor)
            .unwrap();
        let (page, next) = cursor.accept_page(prepared).unwrap();
        if let Some(commit) = page.commit() {
            let historical = commit.basis().relational_basis();
            if historical.admission_identity()
                != source.basis().relational_basis().admission_identity()
            {
                break historical.clone();
            }
        }
        assert!(
            !page.is_complete(),
            "the actual authored predecessor retains its original root"
        );
        cursor = next;
    };
    assert_ne!(
        another_admission.admission_identity(),
        source.basis().relational_basis().admission_identity()
    );
    assert!(matches!(
        native.admit_home_program_transition_planning_request(&admitted, &another_admission),
        Err(Denial::SourceBasisMismatch)
    ));
    let impostor = worth_proof::HomeProgramTransitionOwner::fresh(
        admitted.source().actor().home().bytes(),
        home.incarnation().fence(),
    )
    .admit_planning_request(admitted.source().clone());
    assert!(matches!(
        native.admit_home_program_transition_planning_request(
            &impostor,
            source.basis().relational_basis()
        ),
        Err(Denial::Admission(Admission::ForeignOwner))
    ));
    assert!(matches!(
        native.begin_branch_transaction(
            source.basis().relational_basis(),
            RelationalTransactionIntent::ordinary()
        ),
        Err(
            RelationalBranchTransactionAdmissionDenial::HomeProgramTransition(
                RelationalHomeProgramWriteDenial::TransitionRequired
            )
        )
    ));
    assert!(home.home_program_transition_intent().is_none());
    assert_eq!(
        home.world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    drop((another_admission, source, observation, cursor));
    // The issued proof contains no Native port or mutation lease. Keep it alive
    // while the genuine owner closes; do not release it to make close succeed.
    close(home, &execution);
    assert_eq!(
        admitted.source().actor().home().bytes(),
        impostor.source().actor().home().bytes()
    );
}

#[test]
fn retained_home_planning_request_refuses_after_graceful_reopen_before_source_inspection() {
    use worth_proof::HomeProgramTransitionAdmissionDenial as Admission;
    use worth_relational::facade::runtime::RelationalHomeProgramContractDenial as Denial;
    let directory = tempfile::tempdir().unwrap();
    let request = crate::workload::serial_request();
    let execution = ExecutionRequest::serial(&request);
    let fixture = super::transition_opening::open_transition(directory.path(), &execution);
    let home = fixture.home;
    let source = home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let admitted = home
        .admit_home_program_transition_planning_request(&source, &execution)
        .unwrap();
    let incarnation = home.incarnation();
    drop(source);
    close(home, &execution);
    let fixture = worth_store_testkit_application_oracle::world_home::open(
        super::transition_opening::successor_request(directory.path()),
        &execution,
    )
    .expect("same actual Requested Home reopens gracefully");
    let home = fixture.home;
    assert_ne!(home.incarnation(), incarnation);
    let source = home
        .world()
        .observation_port()
        .observe_root_product_branch()
        .unwrap();
    let native = home.relational().runtime();
    assert!(matches!(
        native.admit_home_program_transition_planning_request(
            &admitted,
            source.basis().relational_basis()
        ),
        Err(Denial::Admission(Admission::StaleIncarnation))
    ));
    let current = home
        .admit_home_program_transition_planning_request(&source, &execution)
        .unwrap();
    assert_eq!(
        native
            .admit_home_program_transition_planning_request(
                &current,
                source.basis().relational_basis()
            )
            .unwrap(),
        admitted.source().actor()
    );
    assert!(home.home_program_transition_intent().is_none());
    assert_eq!(
        home.world()
            .observation_port()
            .observe_root_product_branch()
            .unwrap(),
        source
    );
    drop(source);
    close(home, &execution);
}
