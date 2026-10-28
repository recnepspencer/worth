#[allow(unused_imports)]
pub(crate) use crate::fast::relational_certification::observation_contracts::*;

#[test]
fn every_delta_has_a_distinct_expected_semantic_observation() {
    let baseline = SupplyChainBaseline::operating(SupplyChainScale::court());
    let mut digests = std::collections::BTreeSet::new();
    for delta in DeltaId::ALL {
        let branch = baseline
            .branch
            .fork(delta.branch(), BranchLabel::Operating)
            .unwrap();
        let next = apply(&branch, delta).unwrap();
        let expected = ExpectedSupplyChainObservation::from_branch(&next);
        assert!(digests.insert(digest(&expected)));
    }
}
