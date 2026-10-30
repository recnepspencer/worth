//! The exact structural verifier and borrowed frame used by Query's real receivers.
use std::{any::Any, collections::BTreeMap, sync::Arc};
use worth_proof::*;
use ManagedCoverageDisposition::{Branch, Excluded, Matched, Pruned};

fn tree() -> BTreeMap<usize, ManagedCoverageRegion> {
    [
        (
            10,
            ManagedCoverageRegion {
                parent: None,
                children: [Some(20), Some(30)],
                has_source: true,
            },
        ),
        (
            20,
            ManagedCoverageRegion {
                parent: Some(10),
                children: [None, None],
                has_source: true,
            },
        ),
        (
            30,
            ManagedCoverageRegion {
                parent: Some(10),
                children: [Some(40), None],
                has_source: false,
            },
        ),
        (
            40,
            ManagedCoverageRegion {
                parent: Some(30),
                children: [None, None],
                has_source: true,
            },
        ),
    ]
    .into()
}
fn verify(
    issuer: &ManagedIndexCoverageIssuer,
    root: Option<usize>,
    pruning: bool,
    input: &[(usize, ManagedCoverageDisposition)],
) -> Result<AdmittedManagedIndexCoverage, ManagedIndexCoverageDenial> {
    let graph = tree();
    let index: BTreeMap<_, _> = input.iter().copied().collect();
    let mut steps = input.iter().map(|(node, disposition)| ManagedCoverageStep {
        node: *node,
        disposition: *disposition,
    });
    issuer.verify(ManagedCoverageFrontier {
        root,
        recorded_regions: index.len(),
        allow_pruning: pruning,
        steps: &mut steps,
        region: &mut |node| graph.get(&node).copied(),
        disposition: &mut |node| index.get(&node).copied(),
        checkpoint: &mut || true,
    })
}
#[test]
fn managed_coverage_requires_the_actual_cut_and_internal_source_contributions() {
    let issuer = ManagedIndexCoverageIssuer::fresh();
    let full = [(10, Matched), (20, Excluded), (30, Branch), (40, Matched)];
    let proof = verify(&issuer, Some(10), false, &full).unwrap();
    assert_eq!(proof.matched_source_count(), 2);
    assert!(issuer.authority().admits(&proof));
    assert!(!ManagedIndexCoverageIssuer::fresh()
        .authority()
        .admits(&proof));
    for (input, cause) in [
        (
            vec![(20, Excluded)],
            ManagedIndexCoverageDenial::MissingRoot,
        ),
        (
            vec![(10, Matched), (30, Pruned)],
            ManagedIndexCoverageDenial::MissingChild,
        ),
        (
            vec![(10, Matched), (20, Excluded), (20, Excluded), (30, Pruned)],
            ManagedIndexCoverageDenial::UnorderedRegion,
        ),
        (
            vec![(10, Matched), (20, Excluded), (30, Pruned), (40, Matched)],
            ManagedIndexCoverageDenial::InvalidParent,
        ),
        (
            vec![(10, Branch), (20, Excluded), (30, Pruned)],
            ManagedIndexCoverageDenial::SourceDisposition,
        ),
        (
            vec![(10, Matched), (20, Excluded), (30, Pruned), (99, Matched)],
            ManagedIndexCoverageDenial::MissingRegion,
        ),
    ] {
        assert_eq!(verify(&issuer, Some(10), true, &input).unwrap_err(), cause);
    }
    assert_eq!(
        verify(&issuer, Some(10), false, &[(10, Pruned)]).unwrap_err(),
        ManagedIndexCoverageDenial::PruningUnavailable
    );
    assert_eq!(
        verify(&issuer, Some(10), true, &[(10, Pruned)])
            .unwrap()
            .matched_source_count(),
        0
    );
    assert_eq!(
        verify(&issuer, None, false, &[])
            .unwrap()
            .matched_source_count(),
        0
    );
}
#[test]
fn managed_coverage_rejects_an_unconsumed_stream_and_interruption() {
    let issuer = ManagedIndexCoverageIssuer::fresh();
    let graph = tree();
    let index = BTreeMap::from([(10, Matched), (20, Excluded), (30, Branch), (40, Matched)]);
    let mut steps = [(10, Matched), (20, Excluded), (30, Branch)]
        .into_iter()
        .map(|(node, disposition)| ManagedCoverageStep { node, disposition });
    let denial = issuer
        .verify(ManagedCoverageFrontier {
            root: Some(10),
            recorded_regions: index.len(),
            allow_pruning: false,
            steps: &mut steps,
            region: &mut |node| graph.get(&node).copied(),
            disposition: &mut |node| index.get(&node).copied(),
            checkpoint: &mut || true,
        })
        .unwrap_err();
    assert_eq!(denial, ManagedIndexCoverageDenial::IncompleteFrontier);
    let denial = issuer
        .verify(ManagedCoverageFrontier {
            root: None,
            recorded_regions: 0,
            allow_pruning: false,
            steps: &mut std::iter::empty(),
            region: &mut |_| None,
            disposition: &mut |_| None,
            checkpoint: &mut || false,
        })
        .unwrap_err();
    assert_eq!(denial, ManagedIndexCoverageDenial::Interrupted);
}
#[test]
fn managed_coverage_binds_real_borrowed_root_source_probe_and_invocation() {
    let root: Arc<dyn Any + Send + Sync> = Arc::new(vec![1u64, 2]);
    let foreign_root: Arc<dyn Any + Send + Sync> = Arc::new(vec![1u64, 2]);
    let source = vec![1u64, 2];
    let other_source = source.clone();
    let probe = String::from("alpha");
    let other_probe = probe.clone();
    let audience = vec![7u64];
    let other_audience = audience.clone();
    let frame = BorrowedManagedCoverageFrame::new(&root, &source, &probe, &audience);
    assert!(frame.same_invocation(&frame));
    for other in [
        BorrowedManagedCoverageFrame::new(&foreign_root, &source, &probe, &audience),
        BorrowedManagedCoverageFrame::new(&root, &other_source, &probe, &audience),
        BorrowedManagedCoverageFrame::new(&root, &source, &other_probe, &audience),
        BorrowedManagedCoverageFrame::new(&root, &source, &probe, &other_audience),
    ] {
        assert!(!frame.same_invocation(&other));
    }
}
