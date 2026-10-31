//! Correlate all demanded loci before deciding observation uniqueness.
use super::*;

fn two_field_demand(reversed: bool) -> InstalledPreImageDemand {
    use crate::domain_computation::application_aftermath::aftermath_schema_fixture as fixture;
    let aftermath = if reversed {
        fixture::freeze_account_fields_reversed()
    } else {
        fixture::freeze_account_fields()
    };
    let Some(InstalledCorrectionMechanism::RecordedInverse(inverse)) = aftermath.mechanism() else {
        panic!("expected two-field inverse");
    };
    inverse.preimage_demand().clone()
}

fn root() -> Vec<WorthQueryObservedPreImageCandidate> {
    vec![
        observed("frozen", AspectValue::Bool(true), 1),
        observed("note", AspectValue::String("root".into()), 1),
    ]
}

fn mutations() -> Vec<(RecordRef, AspectFieldLocator)> {
    [mutating("frozen", [1, 2]), mutating("note", [1, 2])].concat()
}

#[test]
fn unique_complete_record_survives_incomplete_collateral_in_every_candidate_order() {
    for reversed in [false, true] {
        let demand = two_field_demand(reversed);
        // Both locus roles: the shared locus precedes or follows the unique locus.
        for shared in ["frozen", "note"] {
            let mut candidates = root();
            candidates.push(observed(
                shared,
                if shared == "frozen" {
                    AspectValue::Bool(false)
                } else {
                    AspectValue::String("collateral".into())
                },
                2,
            ));
            for order in [
                [0, 1, 2],
                [0, 2, 1],
                [1, 0, 2],
                [1, 2, 0],
                [2, 0, 1],
                [2, 1, 0],
            ] {
                let ordered = order.map(|index| candidates[index].clone());
                let retained = retain_from_exact_mutations(&demand, &ordered, &mutations())
                    .expect("all demanded loci identify the one complete mutated record");
                assert_eq!(
                    retained.target_record(),
                    Some(&RecordRef::Entity(entity(1)))
                );
                assert_eq!(
                    retained
                        .fields()
                        .iter()
                        .find(|f| f.locus().field() == "frozen")
                        .unwrap()
                        .value(),
                    &AspectValue::Bool(true)
                );
                assert_eq!(
                    retained
                        .fields()
                        .iter()
                        .find(|f| f.locus().field() == "note")
                        .unwrap()
                        .value(),
                    &AspectValue::String("root".into())
                );
            }
        }
    }
}

#[test]
fn complete_twins_and_disjoint_records_refuse_without_cross_record_assembly() {
    let demand = two_field_demand(false);
    let mut twins = root();
    twins.extend([
        observed("frozen", AspectValue::Bool(true), 2),
        observed("note", AspectValue::String("root".into()), 2),
    ]);
    let disjoint = [
        observed("frozen", AspectValue::Bool(true), 1),
        observed("note", AspectValue::String("root".into()), 2),
    ];
    for candidates in [&twins[..], &disjoint[..]] {
        assert_eq!(
            retain_from_exact_mutations(&demand, candidates, &mutations()).unwrap_err(),
            WorthQueryPreImageRetentionDenial::AmbiguousDemandedField
        );
    }
}

#[test]
fn selected_record_duplicates_refuse_even_equal_values_with_incomplete_collateral() {
    let demand = two_field_demand(false);
    for duplicate in [AspectValue::Bool(true), AspectValue::Bool(false)] {
        let mut candidates = root();
        candidates.push(observed("frozen", duplicate, 1));
        for collateral in [false, true] {
            if collateral {
                candidates.push(observed("frozen", AspectValue::Bool(true), 2));
            }
            assert_eq!(
                retain_from_exact_mutations(&demand, &candidates, &mutations()).unwrap_err(),
                WorthQueryPreImageRetentionDenial::AmbiguousDemandedField
            );
        }
    }
}

#[test]
fn missing_locus_takes_precedence_over_shared_locus_ambiguity() {
    let candidates = [
        observed("frozen", AspectValue::Bool(true), 1),
        observed("frozen", AspectValue::Bool(true), 2),
    ];
    assert_eq!(
        retain_from_exact_mutations(&two_field_demand(false), &candidates, &mutations())
            .unwrap_err(),
        WorthQueryPreImageRetentionDenial::MissingDemandedField
    );
}
