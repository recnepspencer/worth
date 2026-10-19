use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateContract,
    ApplicationCandidateRequirements, ApplicationCandidateResourceCeiling,
    ApplicationMutationBindingDescriptor,
};

/// Largest installed operation demand, including Query's bounded settlement
/// sidecar only for bindings that require workflow authority.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::application_operation) struct WorthQueryApplicationCandidateDemand {
    candidate_items: Option<u64>,
    retained_representation_bytes: Option<u64>,
    candidate_ceiling: Option<ApplicationCandidateContract>,
    workflow_settlement_ceiling: Option<ApplicationCandidateRequirements>,
}

impl Default for WorthQueryApplicationCandidateDemand {
    fn default() -> Self {
        Self {
            candidate_items: Some(0),
            retained_representation_bytes: Some(0),
            candidate_ceiling: None,
            workflow_settlement_ceiling: None,
        }
    }
}

// Query owns this sidecar capacity. It does not enlarge a product handler's
// declared writer reservation; it bounds one local workflow settlement.
const WORKFLOW_SETTLEMENT_RETAINED_BYTES: u64 = 65_536;

fn workflow_settlement_ceiling() -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        // transition: seven structural fields, advancer/executor triplets,
        // receipt identity; instance: one Ready-state write. The producer is
        // workflow/instance/transition/settlement::visit_transition_facts_from_basis.
        ApplicationCandidateCardinalityCeiling::fixed(1, 0, 2, 1, 7 + 3 + 3 + 1 + 1, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(
            WORKFLOW_SETTLEMENT_RETAINED_BYTES as usize,
        ),
    )
}

impl WorthQueryApplicationCandidateDemand {
    pub(in crate::application_operation) fn derive(
        bindings: &[ApplicationMutationBindingDescriptor],
        operation: &str,
        input_type: &str,
    ) -> Result<Self, ()> {
        let mut demand = Self {
            candidate_items: Some(0),
            retained_representation_bytes: Some(0),
            ..Self::default()
        };
        for binding in bindings.iter().filter(|binding| {
            binding.operation_name() == operation && binding.input_identity().as_str() == input_type
        }) {
            demand.candidate_ceiling = Some(match demand.candidate_ceiling {
                Some(prior) => maximum_contract(prior, binding.candidates()),
                None => binding.candidates(),
            });
            if binding.requires_workflow_authority() {
                demand.workflow_settlement_ceiling = Some(workflow_settlement_ceiling());
            }
        }
        demand.include_platform_ceiling()?;
        Ok(demand)
    }

    fn include_platform_ceiling(&mut self) -> Result<(), ()> {
        let Some(contract) = self.candidate_ceiling else {
            return Ok(());
        };
        let sidecar = self.workflow_settlement_ceiling;
        let items = contract
            .cardinality()
            .map(cardinality_items)
            .transpose()?
            .map(|n| {
                n.checked_add(sidecar.map_or(Ok(0), |s| cardinality_items(s.cardinality()))?)
                    .ok_or(())
            })
            .transpose()?;
        let bytes = contract
            .fixed_requirements()
            .map(retained_bytes)
            .transpose()?
            .map(|n| {
                n.checked_add(sidecar.map_or(Ok(0), retained_bytes)?)
                    .ok_or(())
            })
            .transpose()?;
        self.candidate_items = items;
        self.retained_representation_bytes = bytes;
        Ok(())
    }
    pub(in crate::application_operation) const fn candidate_items(self) -> Option<u64> {
        self.candidate_items
    }
    pub(in crate::application_operation) const fn retained_representation_bytes(
        self,
    ) -> Option<u64> {
        self.retained_representation_bytes
    }
    pub(in crate::application_operation) const fn candidate_ceiling(
        self,
    ) -> Option<ApplicationCandidateContract> {
        self.candidate_ceiling
    }
    pub(in crate::application_operation) const fn workflow_settlement_ceiling(
        self,
    ) -> Option<ApplicationCandidateRequirements> {
        self.workflow_settlement_ceiling
    }
}

fn maximum_contract(
    left: ApplicationCandidateContract,
    right: ApplicationCandidateContract,
) -> ApplicationCandidateContract {
    match (left, right) {
        (ApplicationCandidateContract::Prepared, _)
        | (_, ApplicationCandidateContract::Prepared) => ApplicationCandidateContract::Prepared,
        (
            ApplicationCandidateContract::FixedShape(left),
            ApplicationCandidateContract::FixedShape(right),
        ) => ApplicationCandidateContract::FixedShape(maximum_requirements(left, right)),
        (left, right) => ApplicationCandidateContract::FixedCardinality(maximum_cardinality(
            left.cardinality().expect("fixed count contract"),
            right.cardinality().expect("fixed count contract"),
        )),
    }
}
fn retained_bytes(requirements: ApplicationCandidateRequirements) -> Result<u64, ()> {
    u64::try_from(
        requirements
            .resources()
            .maximum_retained_representation_bytes(),
    )
    .map_err(|_| ())
}

fn cardinality_items(cardinality: ApplicationCandidateCardinalityCeiling) -> Result<u64, ()> {
    [
        cardinality.maximum_creates(),
        cardinality.maximum_deletes(),
        cardinality.maximum_links(),
        cardinality.maximum_unlinks(),
        cardinality.maximum_writes(),
        cardinality.maximum_emits(),
    ]
    .into_iter()
    .try_fold(0_u64, |total, value| {
        total
            .checked_add(u64::try_from(value).map_err(|_| ())?)
            .ok_or(())
    })
}

fn maximum_requirements(
    left: ApplicationCandidateRequirements,
    right: ApplicationCandidateRequirements,
) -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        maximum_cardinality(left.cardinality(), right.cardinality()),
        maximum_resources(left.resources(), right.resources()),
    )
}

fn maximum_cardinality(
    left_kinds: ApplicationCandidateCardinalityCeiling,
    right_kinds: ApplicationCandidateCardinalityCeiling,
) -> ApplicationCandidateCardinalityCeiling {
    ApplicationCandidateCardinalityCeiling::fixed(
        left_kinds
            .maximum_creates()
            .max(right_kinds.maximum_creates()),
        left_kinds
            .maximum_deletes()
            .max(right_kinds.maximum_deletes()),
        left_kinds.maximum_links().max(right_kinds.maximum_links()),
        left_kinds
            .maximum_unlinks()
            .max(right_kinds.maximum_unlinks()),
        left_kinds
            .maximum_writes()
            .max(right_kinds.maximum_writes()),
        left_kinds.maximum_emits().max(right_kinds.maximum_emits()),
    )
}

fn maximum_resources(
    left: ApplicationCandidateResourceCeiling,
    right: ApplicationCandidateResourceCeiling,
) -> ApplicationCandidateResourceCeiling {
    let bytes = left
        .maximum_retained_representation_bytes()
        .max(right.maximum_retained_representation_bytes());
    ApplicationCandidateResourceCeiling::representation_bytes(bytes)
}

#[cfg(test)]
mod tests {
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
                candidate_items: Some(11),
                candidate_ceiling: Some(ApplicationCandidateContract::FixedShape(ceiling)),
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
                candidate_ceiling: Some(ApplicationCandidateContract::FixedShape(overflowing)),
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
                candidate_ceiling: Some(ApplicationCandidateContract::FixedShape(ceiling)),
                workflow_settlement_ceiling: Some(workflow_settlement_ceiling()),
            };
            demand.include_platform_ceiling().unwrap();
            assert_eq!(demand.candidate_items(), Some(37));
            assert_eq!(demand.retained_representation_bytes(), Some(65_600));
        }
        #[cfg(target_pointer_width = "64")]
        {
            let mut demand = WorthQueryApplicationCandidateDemand {
                candidate_ceiling: Some(ApplicationCandidateContract::fixed_shape(
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
}
