use super::super::refreshed_rejoin::awaited_by_stale_owner_admitted;
use super::*;

#[test]
fn occurrence_work_is_identical_with_zero_or_many_unrelated_rows() {
    for unrelated in [0, 1000] {
        let selected = key("local-work", 2, 1);
        let older = key("local-work", 1, 1);
        let mut records = super::super::record_map::DemandRecords::new();
        records.insert(
            selected.clone(),
            record(occurrence(), DemandState::Admitted, 0),
        );
        records.insert(
            older,
            record(
                occurrence(),
                DemandState::Failed(WorthQueryOutputDemandDenial::new(
                    WorthQueryOutputDemandDenialKind::Superseded,
                    "",
                )),
                1,
            ),
        );
        for n in 0..unrelated {
            records.insert(
                key(&format!("unrelated-{n}"), 1, 1),
                record(occurrence(), DemandState::Admitted, 0),
            );
        }
        let mut admission = record_admission();
        assert!(awaited_by_stale_owner_admitted(&records, &selected, &mut admission).unwrap());
        assert_eq!(
            admission.charged_work(),
            2 + 2,
            "two range seeks plus one visit per occurrence row"
        );
    }
}
