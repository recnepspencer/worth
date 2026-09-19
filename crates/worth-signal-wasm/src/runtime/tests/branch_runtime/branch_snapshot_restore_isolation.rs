use super::super::support::*;

#[test]
fn restoring_inactive_branch_snapshot_then_editing_other_field_keeps_branch_local_source_values() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "gearTeeth".to_owned(),
            initial: SignalValue::Number(16.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_source(SourceSpec {
            id: "gearThickness".to_owned(),
            initial: SignalValue::Number(0.42),
            produces_aspects: None,
        })
        .unwrap();

    let main_branch = runtime.current_branch();
    let feature_branch = runtime.create_branch("what-if".to_owned()).unwrap();

    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "gearTeeth".to_owned(),
            value: SignalValue::Number(8.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let main_snapshot = runtime.branch_snapshot_id(main_branch.id.0).unwrap();

    runtime.switch_branch(feature_branch.id.0).unwrap();
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "gearTeeth".to_owned(),
            value: SignalValue::Number(32.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
    let feature_snapshot = runtime.branch_snapshot_id(feature_branch.id.0).unwrap();

    runtime
        .restore_branch_snapshot_by_id(main_branch.id.0, main_snapshot)
        .unwrap();
    runtime.switch_branch(main_branch.id.0).unwrap();
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "gearThickness".to_owned(),
            value: SignalValue::Number(0.1),
            aspect: None,
            aspects: None,
        }])
        .unwrap();

    assert_eq!(
        runtime.read_value("gearTeeth").unwrap(),
        SignalValue::Number(8.0),
        "restoring main then editing thickness must not inherit feature teeth"
    );
    assert_eq!(
        runtime.read_value("gearThickness").unwrap(),
        SignalValue::Number(0.1)
    );

    runtime
        .restore_branch_snapshot_by_id(feature_branch.id.0, feature_snapshot)
        .unwrap();
    runtime.switch_branch(feature_branch.id.0).unwrap();
    assert_eq!(
        runtime.read_value("gearTeeth").unwrap(),
        SignalValue::Number(32.0),
        "feature branch teeth should remain isolated after main restore/edit"
    );
}

#[test]
fn general_restore_reactivates_the_snapshot_branch_and_rejects_a_substituted_payload() {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: "value".to_owned(),
            initial: SignalValue::Number(1.0),
            produces_aspects: None,
        })
        .unwrap();
    let mut main_snapshot = runtime.snapshot().unwrap();
    let feature = runtime.create_branch("feature".to_owned()).unwrap();
    runtime.switch_branch(feature.id.0).unwrap();
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: "value".to_owned(),
            value: SignalValue::Number(2.0),
            aspect: None,
            aspects: None,
        }])
        .unwrap();

    // A snapshot belongs to the branch it was captured on: restoring it from
    // another branch reactivates that branch, and the other branch keeps its
    // own truth.
    let main_branch_id = main_snapshot.snapshot.meta.branch_id.0;
    runtime.restore_snapshot(main_snapshot.clone()).unwrap();
    assert_eq!(runtime.current_branch().id.0, main_branch_id);
    assert_eq!(
        runtime.read_value("value").unwrap(),
        SignalValue::Number(1.0)
    );
    runtime.switch_branch(feature.id.0).unwrap();
    assert_eq!(
        runtime.read_value("value").unwrap(),
        SignalValue::Number(2.0),
        "the feature branch is untouched by restoring main"
    );

    // A refused restore leaves the active branch where it was: the payload
    // check runs before any branch switch.
    main_snapshot
        .snapshot
        .meta
        .branch_name
        .push_str("-substituted");
    let substituted = runtime.restore_snapshot(main_snapshot).unwrap_err();
    assert!(substituted
        .message
        .contains("does not match the owner-admitted snapshot"));
    assert_eq!(runtime.current_branch().id, feature.id);
    assert_eq!(
        runtime.read_value("value").unwrap(),
        SignalValue::Number(2.0)
    );
}
