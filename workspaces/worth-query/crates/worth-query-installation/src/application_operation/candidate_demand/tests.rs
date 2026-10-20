use super::*;

#[test]
fn absent_dimensions_absorb_policy_union_and_preserve_independent_sidecar_quotes() {
    let finite = ApplicationCandidatePolicy::finite(ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(7, 0, 0, 0, 0, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(32),
    ));
    for (left, right) in [
        (
            finite,
            ApplicationCandidatePolicy::new(None, finite.resources()),
        ),
        (
            ApplicationCandidatePolicy::new(None, finite.resources()),
            finite,
        ),
    ] {
        let policy = maximum_policy(left, right);
        let mut demand = WorthQueryApplicationCandidateDemand {
            candidate_ceiling: Some(policy),
            workflow_settlement_ceiling: Some(workflow_settlement_ceiling()),
            ..Default::default()
        };
        demand.include_platform_ceiling().unwrap();
        assert_eq!(demand.candidate_items(), None);
        assert_eq!(demand.retained_representation_bytes(), Some(65_568));
        assert_eq!(demand.candidate_ceiling().unwrap().cardinality(), None);
    }
    let policy = maximum_policy(
        finite,
        ApplicationCandidatePolicy::new(finite.cardinality(), None),
    );
    let mut demand = WorthQueryApplicationCandidateDemand {
        candidate_ceiling: Some(policy),
        ..Default::default()
    };
    demand.include_platform_ceiling().unwrap();
    assert_eq!(demand.candidate_items(), Some(7));
    assert_eq!(demand.retained_representation_bytes(), None);
}

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
            candidate_items: Some(11),
            candidate_ceiling: Some(ApplicationCandidatePolicy::finite(ceiling)),
            ..WorthQueryApplicationCandidateDemand::default()
        };
        demand.include_platform_ceiling().unwrap();
        assert_eq!(demand.candidate_items(), Some(18));
    }
    let overflowing = ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(usize::MAX, 0, 0, 0, 1, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(64),
    );
    #[cfg(target_pointer_width = "64")]
    {
        let mut demand = WorthQueryApplicationCandidateDemand {
            candidate_ceiling: Some(ApplicationCandidatePolicy::finite(overflowing)),
            ..WorthQueryApplicationCandidateDemand::default()
        };
        assert_eq!(demand.include_platform_ceiling(), Err(()));
        assert_eq!(demand.candidate_items(), Some(0));
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
            candidate_items: Some(20),
            retained_representation_bytes: Some(65_568),
            candidate_ceiling: Some(ApplicationCandidatePolicy::finite(ceiling)),
            workflow_settlement_ceiling: Some(workflow_settlement_ceiling()),
        };
        demand.include_platform_ceiling().unwrap();
        assert_eq!(demand.candidate_items(), Some(37));
        assert_eq!(demand.retained_representation_bytes(), Some(65_600));
    }
    #[cfg(target_pointer_width = "64")]
    {
        let mut demand = WorthQueryApplicationCandidateDemand {
            candidate_ceiling: Some(ApplicationCandidatePolicy::finite(
                ApplicationCandidateRequirements::fixed_shape(
                    ApplicationCandidateCardinalityCeiling::fixed(usize::MAX, 0, 0, 0, 0, 0),
                    ApplicationCandidateResourceCeiling::representation_bytes(64),
                ),
            )),
            workflow_settlement_ceiling: Some(workflow_settlement_ceiling()),
            ..WorthQueryApplicationCandidateDemand::default()
        };
        let before = demand;
        assert_eq!(demand.include_platform_ceiling(), Err(()));
        assert_eq!(demand, before);
    }
}
