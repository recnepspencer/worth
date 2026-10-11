use super::super::record_map::DemandRecords;

use super::*;

/// A refresh that stopped for the request meeting it, its budget or its
/// cancellation, leaves the head row to a later advance; only a stop intrinsic
/// to the row, met or recorded, fails its dependents.
#[test]
fn only_a_stop_intrinsic_to_the_head_row_fails_its_dependents() {
    let head = key("required-stop-head", 1, 1);
    let failed = |kind| {
        let mut records = DemandRecords::new();
        records.insert(
            head.clone(),
            record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(kind, "")),
                0,
            ),
        );
        records
    };
    for kind in [
        WorthQueryOutputDemandDenialKind::WorkBudgetExceeded,
        WorthQueryOutputDemandDenialKind::Cancelled,
        WorthQueryOutputDemandDenialKind::TimedOut,
    ] {
        assert!(
            super::super::required_stop::head_stop(&failed(kind.clone()), &head).is_none(),
            "{kind:?} belongs to the advance that met it"
        );
    }
    let intrinsic = super::super::required_stop::head_stop(
        &failed(WorthQueryOutputDemandDenialKind::ProducerUnavailable),
        &head,
    );
    assert_eq!(
        intrinsic.map(|denial| denial.kind()),
        Some(WorthQueryOutputDemandDenialKind::ProducerUnavailable)
    );

    let mut recorded = failed(WorthQueryOutputDemandDenialKind::Cancelled);
    recorded.get_mut(&head).unwrap().required_stop =
        Some(WorthQueryOutputDemandDenialKind::MissingApplicableProducer);
    assert_eq!(
        super::super::required_stop::head_stop(&recorded, &head).map(|denial| denial.kind()),
        Some(WorthQueryOutputDemandDenialKind::MissingApplicableProducer)
    );
}
