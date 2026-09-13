use worth_signal::facade::branch::{
    validate_signal_branch_name, SignalBranchRetentionReleaseOutcome, SignalOwnerCancellationSource,
};

use super::super::super::comparison::{
    advance_denial, neutral_basis, neutral_snapshot, readmission_denial,
};
use super::super::super::state::ModelWorld;
use super::super::super::transition::{ModelAction, ModelResult, ModelSuccess};
use super::super::trace_support::{
    assert_denial, current_model, model_denial, model_fork, model_lease, model_movement,
    model_observation, runtime, ORACLE_SEED,
};

#[test]
fn seeded_public_trace_matches_an_independent_oracle_and_covers_terminal_outcomes() {
    let mut runtime = runtime();
    let root_handle = runtime.current_branch();
    let root_basis = runtime
        .observe_signal_branch_basis(root_handle.clone())
        .expect("seed setup: bootstrap basis is admitted");
    let root = root_handle.id.0;
    let root_observation = neutral_basis(&root_basis);
    let mut model = ModelWorld::bootstrap(root, root_handle.name.clone(), root_observation.clone());
    let services = runtime
        .owner_component_services()
        .expect("seed setup: service facade is issuable");
    let basis = services.basis_port();
    let mutation = services.mutation_port();
    let lifecycle = services.lifecycle_port();
    let reference = basis
        .issue_managed_branch_reference(&root_basis)
        .expect("seed setup: managed root reference is owner-issued");

    let expected = model_observation(
        model.apply(ModelAction::Observe { branch: root }),
        "initial observe",
    );
    let observed = basis
        .observe_current(&reference)
        .expect("public observe succeeds for the admitted root");
    assert_eq!(expected, neutral_basis(&observed));

    let expected = model_observation(
        model.apply(ModelAction::Readmit {
            branch: root,
            expected: root_observation.clone(),
        }),
        "initial readmit",
    );
    let readmitted = basis
        .readmit_exact(&reference, root_basis.descriptor())
        .expect("public readmit compares and admits the exact descriptor");
    assert_eq!(expected, neutral_basis(&readmitted));

    let lease = basis
        .retain_exact(&root_basis)
        .expect("public retain opens one exact external obligation");
    let expected = model.apply(ModelAction::Retain {
        branch: root,
        observation: root_observation.clone(),
    });
    assert!(matches!(
        expected,
        ModelResult::Success(ModelSuccess::Lease)
    ));
    let lease_key = model_lease(&model, "before initial retain model application");
    let release = basis.release_exact(lease);
    let released = match release {
        SignalBranchRetentionReleaseOutcome::Released(receipt) => receipt,
        other => panic!("seed {ORACLE_SEED:#x}: initial release unexpectedly denied: {other:?}"),
    };
    assert_eq!(released.branch_id().0, root);
    let expected = model.apply(ModelAction::Release { lease: lease_key });
    assert!(matches!(
        expected,
        ModelResult::Success(ModelSuccess::Release)
    ));

    let child_name = "oracle-child";
    let fork = mutation
        .fork_exact(
            validate_signal_branch_name(child_name).expect("seed setup: child name validates"),
            &root_basis,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("public fork succeeds from the exact root basis");
    let (child_handle, child_basis) = fork.into_parts();
    let child = child_handle.id.0;
    let expected = model_fork(
        model.apply(ModelAction::Fork {
            source: root,
            child,
            child_name: child_name.to_owned(),
        }),
        "fork",
    );
    assert_eq!(expected, neutral_basis(&child_basis));
    assert_eq!(child_basis.branch_id().0, child);
    let model_child = model
        .branch(child)
        .expect("the independent model retains the forked child");
    assert_eq!(model_child.key, child);
    assert_eq!(model_child.parent, Some(root));
    assert_eq!(model_child.name, child_name);
    assert_eq!(child_handle.parent_branch_id.map(|id| id.0), Some(root));
    let child_reference = basis
        .issue_managed_branch_reference(&child_basis)
        .expect("the fork result carries owner-issued child authority");

    let expected = model_movement(
        model.apply(ModelAction::Advance {
            branch: root,
            expected: root_observation.clone(),
            cancelled: false,
        }),
        "first advance",
    );
    let advanced = mutation
        .advance_exact(
            &root_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("public advance performs one canonical movement");
    assert_eq!(expected, neutral_basis(advanced.advanced_basis()));
    let current = advanced.into_basis();

    let expected = model_denial(
        model.apply(ModelAction::Advance {
            branch: root,
            expected: root_observation.clone(),
            cancelled: false,
        }),
        "stale advance",
    );
    let stale = mutation.advance_exact(
        &root_basis,
        &mut (),
        &SignalOwnerCancellationSource::new().token(),
        |_| Ok(()),
    );
    let actual = stale
        .as_ref()
        .err()
        .map(advance_denial)
        .expect("stale advance is a denial, not a success-shaped no-op");
    assert_denial(expected, actual, "stale advance");
    assert_eq!(current_model(&model, root), neutral_basis(&current));

    let expected = model_denial(
        model.apply(ModelAction::Readmit {
            branch: root,
            expected: root_observation.clone(),
        }),
        "stale readmit",
    );
    let stale_readmit = basis
        .readmit_exact(&reference, root_basis.descriptor())
        .expect_err("readmit must reject the superseded descriptor");
    assert_denial(
        expected,
        readmission_denial(&stale_readmit),
        "stale readmit",
    );

    let capture = mutation
        .capture_exact(&current, &SignalOwnerCancellationSource::new().token())
        .expect("public capture advances the exact basis and stores a snapshot");
    let (snapshot, captured_basis) = capture.into_parts();
    let snapshot_model = neutral_snapshot(&snapshot);
    let expected = match model.apply(ModelAction::Capture {
        branch: root,
        expected: neutral_basis(&current),
        snapshot: snapshot_model.snapshot,
        cancelled: false,
    }) {
        ModelResult::Success(ModelSuccess::Capture { observation, .. }) => observation,
        other => panic!("seed {ORACLE_SEED:#x}: model capture mismatch: {other:?}"),
    };
    assert_eq!(expected, neutral_basis(&captured_basis));

    let historical_lease = basis
        .retain_exact(&root_basis)
        .expect("stale but exact historical root basis remains retainable");
    assert!(matches!(
        model.apply(ModelAction::Retain {
            branch: root,
            observation: root_observation.clone(),
        }),
        ModelResult::Success(ModelSuccess::Lease)
    ));
    let historical_lease_key = model_lease(&model, "historical retain");
    let retained = basis
        .readmit_retained_exact(root_basis.descriptor(), &historical_lease)
        .expect("retention readmission preserves the exact historical target");
    let expected = model_observation(
        model.apply(ModelAction::ReadmitRetained {
            branch: root,
            expected: root_observation.clone(),
            lease: historical_lease_key,
        }),
        "historical retained readmit",
    );
    assert_eq!(expected, neutral_basis(&retained));
    match basis.release_exact(historical_lease) {
        SignalBranchRetentionReleaseOutcome::Released(_) => {}
        other => panic!("seed {ORACLE_SEED:#x}: historical release denied: {other:?}"),
    }
    assert!(matches!(
        model.apply(ModelAction::Release {
            lease: historical_lease_key,
        }),
        ModelResult::Success(ModelSuccess::Release)
    ));

    let intervening_expected = neutral_basis(&captured_basis);
    let intervening = mutation
        .advance_exact(
            &captured_basis,
            &mut (),
            &SignalOwnerCancellationSource::new().token(),
            |_| Ok(()),
        )
        .expect("the intervening movement creates a restore target");
    let expected = model_movement(
        model.apply(ModelAction::Advance {
            branch: root,
            expected: intervening_expected,
            cancelled: false,
        }),
        "intervening advance",
    );
    assert_eq!(expected, neutral_basis(intervening.advanced_basis()));
    let intervening_basis = intervening.into_basis();

    let expected = model_movement(
        model.apply(ModelAction::Restore {
            branch: root,
            expected: neutral_basis(&intervening_basis),
            snapshot: snapshot_model.clone(),
            cancelled: false,
        }),
        "restore",
    );
    let restored = mutation
        .restore_exact(
            &intervening_basis,
            &snapshot,
            &SignalOwnerCancellationSource::new().token(),
        )
        .expect("public restore performs against the exact intervening basis");
    assert_eq!(expected, neutral_basis(&restored));
    let current = restored;

    let cancellation = SignalOwnerCancellationSource::new();
    cancellation.cancel();
    let expected = model_denial(
        model.apply(ModelAction::Advance {
            branch: root,
            expected: neutral_basis(&current),
            cancelled: true,
        }),
        "cancelled advance",
    );
    let cancelled = mutation.advance_exact(&current, &mut (), &cancellation.token(), |_| Ok(()));
    assert_denial(
        expected,
        cancelled
            .as_ref()
            .err()
            .map(advance_denial)
            .expect("pre-movement cancellation must deny"),
        "cancelled advance",
    );
    let expected_current = current_model(&model, root);
    assert_eq!(expected_current, neutral_basis(&current));

    super::super::public_trace_terminal::finish_terminal_trace(
        runtime,
        &basis,
        &lifecycle,
        &mut model,
        root,
        child,
        current,
        child_basis,
        &reference,
        &child_reference,
    );
}
