use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidateRequirements,
    ApplicationCandidateResourceCeiling, ApplicationMutationBindingDescriptor,
};

/// Largest installed operation demand, including Query's bounded settlement
/// sidecar only for bindings that require workflow authority.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::application_operation) struct WorthQueryApplicationCandidateDemand {
    candidate_items: u64,
    retained_representation_bytes: u64,
    validator_work: u64,
    candidate_ceiling: Option<ApplicationCandidateRequirements>,
    workflow_settlement_ceiling: Option<ApplicationCandidateRequirements>,
}

// Query owns this sidecar capacity. It does not enlarge a product handler's
// declared writer reservation; it bounds one local workflow settlement.
const WORKFLOW_SETTLEMENT_ITEMS: u64 = 13;
const WORKFLOW_SETTLEMENT_RETAINED_BYTES: u64 = 65_536;
const WORKFLOW_SETTLEMENT_VALIDATOR_WORK: u64 = 13;

fn workflow_settlement_ceiling() -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(1, 0, 2, 1, 9, 0),
        ApplicationCandidateResourceCeiling::bounded(
            WORKFLOW_SETTLEMENT_RETAINED_BYTES as usize,
            WORKFLOW_SETTLEMENT_VALIDATOR_WORK as usize,
        ),
    )
}

impl WorthQueryApplicationCandidateDemand {
    pub(in crate::application_operation) fn derive(
        bindings: &[ApplicationMutationBindingDescriptor],
        operation: &str,
        input_type: &str,
    ) -> Result<Self, ()> {
        let mut demand = bindings
            .iter()
            .filter(|binding| {
                binding.operation_name() == operation
                    && binding.input_identity().as_str() == input_type
            })
            .map(|binding| {
                let requirements = binding.candidates();
                let workflow_settlement = binding.requires_workflow_authority();
                let cardinality = requirements.cardinality();
                let candidate_items = cardinality_items(cardinality)?;
                Ok(Self {
                    candidate_items: candidate_items
                        .checked_add(if workflow_settlement {
                            WORKFLOW_SETTLEMENT_ITEMS
                        } else {
                            0
                        })
                        .ok_or(())?,
                    retained_representation_bytes: u64::try_from(
                        requirements
                            .resources()
                            .maximum_retained_representation_bytes(),
                    )
                    .map_err(|_| ())?
                    .checked_add(if workflow_settlement {
                        WORKFLOW_SETTLEMENT_RETAINED_BYTES
                    } else {
                        0
                    })
                    .ok_or(())?,
                    validator_work: u64::try_from(
                        requirements
                            .resources()
                            .maximum_validator_work()
                            .unwrap_or(0),
                    )
                    .map_err(|_| ())?
                    .checked_add(if workflow_settlement {
                        WORKFLOW_SETTLEMENT_VALIDATOR_WORK
                    } else {
                        0
                    })
                    .ok_or(())?,
                    candidate_ceiling: Some(requirements),
                    workflow_settlement_ceiling: workflow_settlement
                        .then(workflow_settlement_ceiling),
                })
            })
            .try_fold(
                Self::default(),
                |maximum, candidate: Result<Self, ()>| -> Result<Self, ()> {
                    let candidate = candidate?;
                    Ok(Self {
                        candidate_items: maximum.candidate_items.max(candidate.candidate_items),
                        retained_representation_bytes: maximum
                            .retained_representation_bytes
                            .max(candidate.retained_representation_bytes),
                        validator_work: maximum.validator_work.max(candidate.validator_work),
                        candidate_ceiling: match (
                            maximum.candidate_ceiling,
                            candidate.candidate_ceiling,
                        ) {
                            (Some(left), Some(right)) => Some(maximum_requirements(left, right)),
                            (Some(ceiling), None) | (None, Some(ceiling)) => Some(ceiling),
                            (None, None) => None,
                        },
                        workflow_settlement_ceiling: maximum
                            .workflow_settlement_ceiling
                            .or(candidate.workflow_settlement_ceiling),
                    })
                },
            )?;
        demand.include_platform_ceiling()?;
        Ok(demand)
    }

    fn include_platform_ceiling(&mut self) -> Result<(), ()> {
        // Platform and workflow reservations consume independent ceilings.
        // Quote the merged platform maxima plus the admitted sidecar, not
        // only the largest individual binding's sidecar-inclusive demand.
        if let Some(ceiling) = self.candidate_ceiling {
            let sidecar = self.workflow_settlement_ceiling;
            let items = cardinality_items(ceiling.cardinality())?
                .checked_add(sidecar.map_or(Ok(0), |s| cardinality_items(s.cardinality()))?)
                .ok_or(())?;
            let bytes = retained_bytes(ceiling)?
                .checked_add(sidecar.map_or(Ok(0), retained_bytes)?)
                .ok_or(())?;
            let work = validator_units(ceiling)?
                .checked_add(sidecar.map_or(Ok(0), validator_units)?)
                .ok_or(())?;
            self.candidate_items = self.candidate_items.max(items);
            self.retained_representation_bytes = self.retained_representation_bytes.max(bytes);
            self.validator_work = self.validator_work.max(work);
        }
        Ok(())
    }

    pub(in crate::application_operation) const fn candidate_items(self) -> u64 {
        self.candidate_items
    }

    pub(in crate::application_operation) const fn retained_representation_bytes(self) -> u64 {
        self.retained_representation_bytes
    }

    pub(in crate::application_operation) const fn validator_work(self) -> u64 {
        self.validator_work
    }

    pub(in crate::application_operation) const fn candidate_ceiling(
        self,
    ) -> Option<ApplicationCandidateRequirements> {
        self.candidate_ceiling
    }

    pub(in crate::application_operation) const fn workflow_settlement_ceiling(
        self,
    ) -> Option<ApplicationCandidateRequirements> {
        self.workflow_settlement_ceiling
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

fn validator_units(requirements: ApplicationCandidateRequirements) -> Result<u64, ()> {
    u64::try_from(
        requirements
            .resources()
            .maximum_validator_work()
            .unwrap_or(0),
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
    let (left_kinds, right_kinds) = (left.cardinality(), right.cardinality());
    let (left_resources, right_resources) = (left.resources(), right.resources());
    ApplicationCandidateRequirements::fixed_shape(
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
        ),
        maximum_resources(left_resources, right_resources),
    )
}

fn maximum_resources(
    left: ApplicationCandidateResourceCeiling,
    right: ApplicationCandidateResourceCeiling,
) -> ApplicationCandidateResourceCeiling {
    let bytes = left
        .maximum_retained_representation_bytes()
        .max(right.maximum_retained_representation_bytes());
    match (
        left.maximum_validator_work(),
        right.maximum_validator_work(),
    ) {
        (Some(left), Some(right)) => {
            ApplicationCandidateResourceCeiling::bounded(bytes, left.max(right))
        }
        _ => ApplicationCandidateResourceCeiling::representation_bytes(bytes),
    }
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
            ApplicationCandidateResourceCeiling::bounded(32, 7),
        );
        let second = ApplicationCandidateRequirements::fixed_shape(
            ApplicationCandidateCardinalityCeiling::fixed(0, 0, 0, 0, 11, 0),
            ApplicationCandidateResourceCeiling::bounded(64, 11),
        );
        for ceiling in [
            maximum_requirements(first, second),
            maximum_requirements(second, first),
        ] {
            let mut demand = WorthQueryApplicationCandidateDemand {
                candidate_items: 20,
                retained_representation_bytes: 65_568,
                validator_work: 20,
                candidate_ceiling: Some(ceiling),
                workflow_settlement_ceiling: Some(workflow_settlement_ceiling()),
            };
            demand.include_platform_ceiling().unwrap();
            assert_eq!(demand.candidate_items(), 31);
            assert_eq!(demand.retained_representation_bytes(), 65_600);
            assert_eq!(demand.validator_work(), 24);
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
    fn any_omitted_binding_keeps_shared_operation_work_budget_absent() {
        let automatic = ApplicationCandidateResourceCeiling::representation_bytes(16);
        let explicit = ApplicationCandidateResourceCeiling::bounded(32, 7);
        for resources in [
            maximum_resources(automatic, explicit),
            maximum_resources(explicit, automatic),
        ] {
            assert_eq!(resources.maximum_validator_work(), None);
            assert_eq!(resources.maximum_retained_representation_bytes(), 32);
        }
    }
}
