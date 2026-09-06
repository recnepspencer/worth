//! The creation cell where the second owner denies after the first has already
//! forked. Creation never advances an owner, so the only way to reach a
//! partial creation is to hold the Signal destination the plan names.

use super::*;

fn create_partial_effects(
    owner: &TestOwner,
    source: &ProductBranchObservation,
    intent: ProductBranchCreationIntent,
) -> crate::recovery::ProductUnpublishedOwnerEffects {
    let cancellation = RuntimeWorldCancellationSource::new();
    let outcome = RuntimeWorldBranchService::create_product_branch(
        owner,
        RuntimeWorldBranchCreationRequest::new(source.clone(), intent, &cancellation.token()),
    )
    .expect("the later sibling denial is a product-unpublished outcome");
    match outcome {
        RuntimeWorldBranchCreationOutcome::Performed(_) => {
            panic!("a held Signal destination must deny the Signal fork")
        }
        RuntimeWorldBranchCreationOutcome::ProductUnpublished(effects) => effects,
    }
}

#[test]
fn forked_relational_effect_is_retained_when_later_signal_sibling_denies() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let history_before = owner.state.history.len();
    let custody_before = owner.state.retention.active_component_obligation_count();
    // The Signal owner already holds this exact destination, so the second leg
    // of the creation denies after the Relational fork has really happened.
    let held = owner
        .state
        .signal
        .mutation_port()
        .reserve_fork_exact(
            validate_signal_branch_name("signal-branch-partial").expect("valid Signal name"),
            source.basis().signal_basis(),
        )
        .expect("the first owner-issued reservation holds the destination");
    let effects = create_partial_effects(
        &owner,
        &source,
        fork_intent(
            "branch-fork-partial",
            relational_fork("relational-branch-partial"),
            signal_fork("signal-branch-partial"),
        ),
    );
    assert_eq!(effects.cause(), ProductUnpublishedCause::SiblingOwnerDenied);
    assert_eq!(effects.owner_effect_count(), 1);
    assert_eq!(
        effects.progress().relational_posture(),
        crate::publication::RelationalAttemptProgressPosture::Performed
    );
    assert_eq!(
        effects.progress().signal_posture(),
        crate::publication::SignalAttemptProgressPosture::Untouched
    );
    assert_ne!(
        effects
            .successor_commit()
            .expect("a retained fork installs its successor occurrence"),
        source.selected_commit()
    );
    assert_ne!(
        effects
            .successor_basis()
            .expect("retained fork successor basis")
            .relational_basis()
            .identity(),
        source.basis().relational_basis().identity()
    );
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    assert_eq!(owner.state.history.len(), history_before + 1);
    assert_eq!(owner.recovery_record_count(), 1);
    assert!(
        owner.state.retention.active_component_obligation_count() > custody_before,
        "partial owner effects retain bounded component custody"
    );
    assert_cleanup_releases_custody(&owner, effects, &source, custody_before);
    drop(held);
}

/// Cleaning up the retained record is what releases the custody the partial
/// creation charged, and it leaves the source observation exactly as it was.
///
/// INT-003. The Relational fork really happened, so the cleanup that retires
/// the record must hand back the component branch that fork created. A cleanup
/// that released the record and left the custody installed would leave a real
/// Relational branch under an occurrence no product reference will ever name.
fn assert_cleanup_releases_custody(
    owner: &TestOwner,
    effects: crate::recovery::ProductUnpublishedOwnerEffects,
    source: &ProductBranchObservation,
    custody_before: usize,
) {
    let handle = effects.recovery_handle();
    assert!(owner.inspect_recovery(&handle).is_some());
    let (destination, incarnation) = effects
        .destination_branch()
        .expect("a creation record names the occurrence it reserved");
    let destination = destination.clone();
    let custody_installed = owner.state.custody.installed();
    assert_eq!(
        custody_installed, 1,
        "the performed Relational fork is the one charged custody record"
    );
    let work = owner
        .cleanup_recovery(effects)
        .expect("the retained record is released by its own caller capability");
    assert_eq!(
        work,
        vec![OwnerRetirementWork::RelationalBranchRetirement {
            target: BranchId("relational-branch-partial".to_owned()),
        }],
        "cleanup returns the forked owner's retirement work, typed and exact"
    );
    assert_eq!(
        owner.state.custody.installed(),
        0,
        "the occurrence's custody is empty once the cleanup drained it"
    );
    assert!(
        owner
            .state
            .custody
            .take_for_incarnation(&destination, incarnation)
            .is_empty(),
        "a second drain of the same occurrence finds nothing left to retire"
    );
    assert_eq!(owner.recovery_record_count(), 0);
    assert_eq!(
        owner.state.retention.active_component_obligation_count(),
        custody_before + 2 // Unreachable successor history awaits explicit reclamation.
    );
    let source_after =
        RuntimeWorldObservationService::observe_product_branch(owner, source.branch_identity())
            .expect("source remains observable after recovery cleanup");
    assert_eq!(&source_after, source);
    assert_eq!(owner.state.branches.branch_count(), 1);
}

/// Releasing the record by its handle is the same release as consuming the
/// capability: one catalog authority reads the occurrence the record named
/// and drains its custody, so neither path can leave the performed fork
/// installed under an occurrence nothing will ever name.
#[test]
fn releasing_the_record_by_handle_drains_the_same_fork_custody() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let held = owner
        .state
        .signal
        .mutation_port()
        .reserve_fork_exact(
            validate_signal_branch_name("signal-branch-handle").expect("valid Signal name"),
            source.basis().signal_basis(),
        )
        .expect("the Signal destination this creation will ask for is already held");
    let effects = create_partial_effects(
        &owner,
        &source,
        fork_intent(
            "branch-fork-handle",
            relational_fork("relational-branch-handle"),
            signal_fork("signal-branch-handle"),
        ),
    );
    drop(held);
    let handle = effects.recovery_handle();
    drop(effects);
    assert_eq!(owner.state.custody.installed(), 1);

    let work = owner
        .cleanup_recovery_handle(&handle)
        .expect("a settled record with no live capability is released by its handle");

    assert_eq!(
        work,
        vec![OwnerRetirementWork::RelationalBranchRetirement {
            target: BranchId("relational-branch-handle".to_owned()),
        }],
        "the handle release returns the forked owner's retirement work, typed and exact"
    );
    assert_eq!(
        owner.state.custody.installed(),
        0,
        "the handle release drains the occurrence's custody exactly as the capability release does"
    );
    assert_eq!(owner.recovery_record_count(), 0);
}

/// The performed leg of a partial creation is a component branch that really
/// exists. Whatever the creation does with the rest of its reservations, that
/// branch must stay named in custody under the occurrence that made it: losing
/// the record is how a component branch becomes unreachable garbage.
#[test]
fn relational_fork_then_signal_owner_loss_retains_exact_fork_custody() {
    let (_fixture, owner, source) = setup_with_relational_source(3);
    let held = owner
        .state
        .signal
        .mutation_port()
        .reserve_fork_exact(
            validate_signal_branch_name("signal-branch-fork-loss").expect("valid Signal name"),
            source.basis().signal_basis(),
        )
        .expect("the Signal destination this creation will ask for is already held");
    let effects = create_partial_effects(
        &owner,
        &source,
        fork_intent(
            "branch-fork-loss",
            relational_fork("relational-branch-fork-loss"),
            signal_fork("signal-branch-fork-loss"),
        ),
    );
    drop(held);

    assert_eq!(effects.cause(), ProductUnpublishedCause::SiblingOwnerDenied);
    assert_eq!(effects.owner_effect_count(), 1);
    assert_eq!(
        effects.progress().relational_posture(),
        crate::publication::RelationalAttemptProgressPosture::Performed
    );
    assert_eq!(
        effects.progress().signal_posture(),
        crate::publication::SignalAttemptProgressPosture::Untouched
    );

    assert_exact_fork_custody(&owner);
    assert_eq!(owner.state.branches.branch_count(), 1);
    assert_eq!(owner.state.branches.reserved_branch_count(), 0);
    drop(effects);
}

/// Exactly one record, naming the destination the performed fork created, keyed
/// to the product-branch occurrence that reserved it.
fn assert_exact_fork_custody(owner: &TestOwner) {
    let records = owner.state.custody.installed_records();
    assert_eq!(
        records.len(),
        1,
        "the denied Signal leg charges nothing and the performed Relational leg charges once"
    );
    assert_eq!(
        records[0].target(),
        &crate::branch::ComponentBranchTarget::Relational(BranchId(
            "relational-branch-fork-loss".to_owned()
        )),
        "custody names the exact destination the performed fork created"
    );
    assert_eq!(
        records[0].product_branch(),
        &crate::identity::ProductBranchIdentity::issued(
            owner.owner_identity(),
            crate::branch::ProductBranchCreationIntent::named("branch-fork-loss")
                .expect("valid product branch name")
                .name()
                .clone(),
        ),
        "the record is keyed to the product-branch occurrence that reserved it"
    );
}
