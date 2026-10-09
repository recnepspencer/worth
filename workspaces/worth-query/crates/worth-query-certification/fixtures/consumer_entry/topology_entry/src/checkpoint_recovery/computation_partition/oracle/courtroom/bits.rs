//! Signed zero is a real gathered-fact edit, not numeric equality.
use super::super::differential::alphabet::{Change, Lcg};
use super::*;

#[test]
fn signed_zero_requires_one_kernel_and_one_tree_recombination() {
    let _guard = checkpoint_recovery_test_guard();
    let mut rng = Lcg(SEEDS[0]);
    let mut model = Model::signed_zero(&mut rng);
    let application = install(|graph| model.seed(graph));
    let (scope, principal) = authenticate(&application);
    let request = application.request(&principal, &scope);
    let (_, first) = demand(&request, &application);
    assert_eq!(first[0].outcome.as_ref().unwrap().0, (-0.0_f64).to_bits());
    let before = model.clone();
    let step = model.step(Kind::SignedZero, &mut rng);
    assert!(model != before, "the model compares canonical bits");
    for change in step.changes {
        let Change::Entry(change) = change else {
            panic!("zero changes one entry")
        };
        edit(&request, &application, change, 0x612_b175);
    }
    let (contacts, runs) = demand(&request, &application);
    assert_eq!(contacts, 1);
    assert_eq!(runs[0].calls, model.expected_calls(Some(&before)));
    assert_eq!(runs[0].outcome.as_ref().unwrap().0, 0.0_f64.to_bits());
    assert_eq!(
        runs[0].tree_runs[0].metrics().recombined_nodes,
        tree::expected_nodes(&model, Some(&before))
    );
    assert_eq!(runs[0].tree_runs[0].metrics().recombined_nodes, 1);
}
