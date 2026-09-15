//! Merging a child branch into its parent must carry the child's committed
//! truth regardless of what was *observed* in between. Snapshots, branch
//! snapshot artifacts, and branch-state proofs are observations: minting them
//! (which the worker-first root does after every mutation) must not change
//! what a later merge adopts, and the merged parent must read the merged
//! truth on the first read after the switch, and keep recomputing afterwards.

use super::super::support::*;

const COUNT: &str = "count";
const DOUBLED: &str = "probe:doubled";
const OUTPUT: &str = "probe.doubled";

fn runtime_with_published_doubled_output() -> RuntimeCore {
    let mut runtime = RuntimeCore::new(RuntimePolicySpec::default()).unwrap();
    runtime
        .define_source(SourceSpec {
            id: COUNT.to_owned(),
            initial: SignalValue::Number(9.0),
            produces_aspects: None,
        })
        .unwrap();
    runtime
        .define_web_computed(
            DOUBLED.to_owned(),
            RecipeSpec {
                id: DOUBLED.to_owned(),
                reads: vec![RecipeReadSpec::LegacyId(COUNT.to_owned())],
                expr: Expr::Sum {
                    args: vec![read(COUNT), read(COUNT)],
                },
                when: None,
                identity: Some(IdentitySpec::Exact),
                produces_aspects: None,
            },
        )
        .unwrap();
    runtime
        .define_web_output(
            OUTPUT.to_owned(),
            RecipeSpec {
                id: OUTPUT.to_owned(),
                reads: vec![RecipeReadSpec::LegacyId(DOUBLED.to_owned())],
                expr: read(DOUBLED),
                when: None,
                identity: Some(IdentitySpec::Exact),
                produces_aspects: None,
            },
        )
        .unwrap();
    assert_eq!(runtime.read_value(OUTPUT).unwrap(), SignalValue::Number(18.0));
    runtime
}

fn set_count(runtime: &mut RuntimeCore, value: f64) {
    runtime
        .apply_transaction(vec![TransactionOp::Set {
            id: COUNT.to_owned(),
            value: SignalValue::Number(value),
            aspect: None,
            aspects: None,
        }])
        .unwrap();
}

/// Everything the worker-first root's cached import context asks the worker
/// for after each mutation, restricted to the calls that touch branch state.
fn observe_like_the_worker_first_root(runtime: &mut RuntimeCore) {
    runtime.snapshot().unwrap();
    for branch in runtime.branches() {
        runtime.branch_snapshot(branch.id.0).unwrap();
        runtime.branch_snapshot_envelope(branch.id.0).unwrap();
        runtime.branch_state_proof(branch.id.0).unwrap();
        runtime.replay_for_branch(branch.id.0).unwrap();
    }
}

fn assert_parent_reads_merged_truth(runtime: &mut RuntimeCore, main_branch: u64) {
    runtime.switch_branch(main_branch).unwrap();
    assert_eq!(runtime.current_branch().id.0, main_branch);
    assert_eq!(
        runtime.read_value(COUNT).unwrap(),
        SignalValue::Number(17.0),
        "merged parent must read the child's committed source value"
    );
    assert_eq!(
        runtime.read_value(DOUBLED).unwrap(),
        SignalValue::Number(34.0),
        "merged parent must recompute the computed on first read"
    );
    assert_eq!(
        runtime.read_value(OUTPUT).unwrap(),
        SignalValue::Number(34.0),
        "merged parent must recompute the published output on first read"
    );
    set_count(runtime, 21.0);
    assert_eq!(
        runtime.read_value(OUTPUT).unwrap(),
        SignalValue::Number(42.0),
        "merged parent must keep recomputing after later writes"
    );
}

#[test]
fn merging_from_the_active_child_carries_its_truth_into_the_parent() {
    let mut runtime = runtime_with_published_doubled_output();
    let main_branch = runtime.current_branch().id.0;
    let feature_branch = runtime.create_branch("fp".to_owned()).unwrap().id.0;
    runtime.switch_branch(feature_branch).unwrap();
    set_count(&mut runtime, 17.0);
    assert_eq!(runtime.read_value(OUTPUT).unwrap(), SignalValue::Number(34.0));

    let result = runtime
        .merge_branches(feature_branch, main_branch)
        .unwrap();

    assert_eq!(runtime.current_branch().id.0, feature_branch);
    assert!(
        result.records.iter().any(|record| record.action == "Adopted"),
        "merge must adopt the child's edit: {:?}",
        result.records.iter().map(|record| (record.source_node.clone(), record.action.clone())).collect::<Vec<_>>()
    );
    assert_parent_reads_merged_truth(&mut runtime, main_branch);
}

#[test]
fn observing_branches_between_the_edit_and_the_merge_does_not_change_what_merges() {
    let mut runtime = runtime_with_published_doubled_output();
    let main_branch = runtime.current_branch().id.0;
    observe_like_the_worker_first_root(&mut runtime);
    let feature_branch = runtime.create_branch("fp".to_owned()).unwrap().id.0;
    observe_like_the_worker_first_root(&mut runtime);
    runtime.switch_branch(feature_branch).unwrap();
    observe_like_the_worker_first_root(&mut runtime);
    set_count(&mut runtime, 17.0);
    observe_like_the_worker_first_root(&mut runtime);
    assert_eq!(runtime.read_value(OUTPUT).unwrap(), SignalValue::Number(34.0));
    observe_like_the_worker_first_root(&mut runtime);

    let result = runtime
        .merge_branches(feature_branch, main_branch)
        .unwrap();

    assert!(
        result.records.iter().any(|record| record.action == "Adopted"),
        "observation must not empty the merge's source slice: {:?}",
        result.records.iter().map(|record| (record.source_node.clone(), record.action.clone())).collect::<Vec<_>>()
    );
    observe_like_the_worker_first_root(&mut runtime);
    assert_parent_reads_merged_truth(&mut runtime, main_branch);
}

#[test]
fn merged_parent_recomputes_the_published_output_without_reading_the_computed_first() {
    let mut runtime = runtime_with_published_doubled_output();
    let main_branch = runtime.current_branch().id.0;
    let feature_branch = runtime.create_branch("fp".to_owned()).unwrap().id.0;
    runtime.switch_branch(feature_branch).unwrap();
    set_count(&mut runtime, 17.0);
    assert_eq!(runtime.read_value(OUTPUT).unwrap(), SignalValue::Number(34.0));
    runtime
        .merge_branches(feature_branch, main_branch)
        .unwrap();

    runtime.switch_branch(main_branch).unwrap();
    assert_eq!(
        runtime.peek_value(OUTPUT).unwrap(),
        SignalValue::Number(34.0),
        "published outputs are standing demand: the switch itself completes them"
    );
    assert_eq!(
        runtime.read_value(OUTPUT).unwrap(),
        SignalValue::Number(34.0),
        "the published output must recompute through the intermediate computed"
    );
    set_count(&mut runtime, 21.0);
    assert_eq!(runtime.read_value(OUTPUT).unwrap(), SignalValue::Number(42.0));
}
