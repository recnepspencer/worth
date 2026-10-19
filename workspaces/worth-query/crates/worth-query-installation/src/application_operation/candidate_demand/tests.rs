use super::*;

#[test]
fn complementary_platform_ceilings_quote_all_permitted_items() {
    let first = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(7, 0, 0, 0, 0, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(32),
    );
    let second = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 11, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(64),
    );
    for ceiling in [
        maximum_requirements(first, second),
        maximum_requirements(second, first),
    ] {
        assert_eq!(ceiling.cardinality().maximum_creates(), 7);
        assert_eq!(ceiling.cardinality().maximum_writes(), 11);
        let mut demand = WorthQueryApplicationCandidateDemand {
            candidate_items: 11,
            candidate_ceiling: Some(ceiling),
            ..WorthQueryApplicationCandidateDemand::default()
        };
        demand.include_platform_ceiling().unwrap();
        assert_eq!(demand.candidate_items(), 18);
    }
    let overflowing = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(usize::MAX, 0, 0, 0, 1, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(64),
    );
    #[cfg(target_pointer_width = "64")]
    {
        let mut demand = WorthQueryApplicationCandidateDemand {
            candidate_ceiling: Some(overflowing),
            ..WorthQueryApplicationCandidateDemand::default()
        };
        assert_eq!(demand.include_platform_ceiling(), Err(()));
        assert_eq!(demand.candidate_items(), 0);
    }
    #[cfg(not(target_pointer_width = "64"))]
    assert_eq!(
        cardinality_items(overflowing.cardinality()),
        Ok(usize::MAX as u64 + 1)
    );
}

#[test]
fn complementary_workflow_quote_covers_both_admitted_ceilings() {
    let first = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(7, 0, 0, 0, 0, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(32),
    );
    let second = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 11, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(64),
    );
    for ceiling in [
        maximum_requirements(first, second),
        maximum_requirements(second, first),
    ] {
        let mut demand = WorthQueryApplicationCandidateDemand {
            candidate_items: 20,
            retained_representation_bytes: 65_568,
            candidate_ceiling: Some(ceiling),
            workflow_settlement_ceiling: Some(workflow_settlement_ceiling()),
        };
        demand.include_platform_ceiling().unwrap();
        assert_eq!(
            demand.candidate_items(),
            cardinality_items(ceiling.cardinality()).unwrap()
                + cardinality_items(workflow_settlement_ceiling().cardinality()).unwrap()
        );
        assert_eq!(demand.retained_representation_bytes(), 65_600);
    }
    #[cfg(target_pointer_width = "64")]
    {
        let mut demand = WorthQueryApplicationCandidateDemand {
            candidate_ceiling: Some(ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(usize::MAX, 0, 0, 0, 0, 0),
                ApplicationCandidateResourceCeiling::representation_bytes(64),
            )),
            workflow_settlement_ceiling: Some(workflow_settlement_ceiling()),
            ..WorthQueryApplicationCandidateDemand::default()
        };
        let before = demand;
        assert_eq!(demand.include_platform_ceiling(), Err(()));
        assert_eq!(demand, before);
    }
}

#[test]
fn workflow_output_sidecars_merge_independently_of_binding_order() {
    let demands = [1, 3].map(|outputs| {
        let ceiling = settlement_ceiling_for_outputs(outputs).unwrap();
        WorthQueryApplicationCandidateDemand {
            candidate_items: cardinality_items(ceiling.cardinality()).unwrap(),
            workflow_settlement_ceiling: Some(ceiling),
            ..WorthQueryApplicationCandidateDemand::default()
        }
    });
    let expected = demands[1];
    for [first, second] in [demands, [demands[1], demands[0]]] {
        let mut maximum = first.maximum(second);
        maximum.include_platform_ceiling().unwrap();
        assert_eq!(
            maximum.workflow_settlement_ceiling(),
            expected.workflow_settlement_ceiling()
        );
        assert!(maximum.candidate_items() >= expected.candidate_items());
    }
    assert!(settlement_ceiling_for_outputs(usize::MAX).is_err());
}
