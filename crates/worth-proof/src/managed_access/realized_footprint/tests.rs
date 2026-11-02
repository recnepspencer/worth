use super::*;
use crate::{ManagedCoverageFrontier, ManagedIndexCoverageIssuer};

fn empty(issuer: &ManagedIndexCoverageIssuer) -> AdmittedManagedIndexCoverage {
    issuer
        .verify(ManagedCoverageFrontier {
            root: None,
            recorded_regions: 0,
            allow_pruning: false,
            steps: &mut std::iter::empty(),
            region: &mut |_| None,
            disposition: &mut |_| None,
            checkpoint: &mut || true,
        })
        .unwrap()
}

#[test]
fn realized_footprint_requires_installed_coverage_and_receiver() {
    let coverage = ManagedIndexCoverageIssuer::fresh();
    let installed = RealizedFootprintIssuer::fresh(coverage.authority());
    let foreign = RealizedFootprintIssuer::fresh(coverage.authority());
    let frame = BorrowedRealizedFootprintFrame::new(&1u32, &2u32, &3u32, &4u32);
    assert!(matches!(
        installed.capture(frame, &empty(&ManagedIndexCoverageIssuer::fresh())),
        Err(RealizedFootprintDenial::ForeignCoverage)
    ));
    let proof = installed
        .capture(frame, &empty(&coverage))
        .unwrap()
        .verify(frame)
        .unwrap();
    assert!(installed.authority().admits(&proof));
    assert!(!foreign.authority().admits(&proof));
}

#[test]
fn realized_footprint_refuses_substituted_admitted_source_parameters_or_audience() {
    let coverage = ManagedIndexCoverageIssuer::fresh();
    let installed = RealizedFootprintIssuer::fresh(coverage.authority());
    let original = [1u32, 2, 3, 4];
    let other = [1u32, 2, 3, 4];
    let frame =
        BorrowedRealizedFootprintFrame::new(&original[0], &original[1], &original[2], &original[3]);
    for which in 0..4 {
        let parts: [&dyn Any; 4] = std::array::from_fn(|i| {
            if i == which {
                &other[i] as &dyn Any
            } else {
                &original[i] as &dyn Any
            }
        });
        let substituted =
            BorrowedRealizedFootprintFrame::new(parts[0], parts[1], parts[2], parts[3]);
        assert!(matches!(
            installed
                .capture(frame, &empty(&coverage))
                .unwrap()
                .verify(substituted),
            Err(RealizedFootprintDenial::ForeignInvocation)
        ));
    }
}
