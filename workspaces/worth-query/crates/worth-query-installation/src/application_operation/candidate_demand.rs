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
    ) -> Self {
        bindings
            .iter()
            .filter(|binding| {
                binding.operation_name() == operation
                    && binding.input_identity().as_str() == input_type
            })
            .map(|binding| {
                let requirements = binding.candidates();
                let workflow_settlement = binding.requires_workflow_authority();
                let cardinality = requirements.cardinality();
                let candidate_items = [
                    cardinality.maximum_creates(),
                    cardinality.maximum_deletes(),
                    cardinality.maximum_links(),
                    cardinality.maximum_unlinks(),
                    cardinality.maximum_writes(),
                    cardinality.maximum_emits(),
                ]
                .into_iter()
                .map(|value| u64::try_from(value).unwrap_or(u64::MAX))
                .fold(0_u64, u64::saturating_add);
                Self {
                    candidate_items: candidate_items.saturating_add(if workflow_settlement {
                        WORKFLOW_SETTLEMENT_ITEMS
                    } else {
                        0
                    }),
                    retained_representation_bytes: u64::try_from(
                        requirements
                            .resources()
                            .maximum_retained_representation_bytes(),
                    )
                    .unwrap_or(u64::MAX)
                    .saturating_add(if workflow_settlement {
                        WORKFLOW_SETTLEMENT_RETAINED_BYTES
                    } else {
                        0
                    }),
                    validator_work: u64::try_from(
                        requirements.resources().maximum_validator_work(),
                    )
                    .unwrap_or(u64::MAX)
                    .saturating_add(if workflow_settlement {
                        WORKFLOW_SETTLEMENT_VALIDATOR_WORK
                    } else {
                        0
                    }),
                    candidate_ceiling: Some(requirements),
                    workflow_settlement_ceiling: workflow_settlement
                        .then(workflow_settlement_ceiling),
                }
            })
            .fold(Self::default(), |maximum, candidate| Self {
                candidate_items: maximum.candidate_items.max(candidate.candidate_items),
                retained_representation_bytes: maximum
                    .retained_representation_bytes
                    .max(candidate.retained_representation_bytes),
                validator_work: maximum.validator_work.max(candidate.validator_work),
                candidate_ceiling: match (maximum.candidate_ceiling, candidate.candidate_ceiling) {
                    (Some(left), Some(right)) => Some(maximum_requirements(left, right)),
                    (Some(ceiling), None) | (None, Some(ceiling)) => Some(ceiling),
                    (None, None) => None,
                },
                workflow_settlement_ceiling: maximum
                    .workflow_settlement_ceiling
                    .or(candidate.workflow_settlement_ceiling),
            })
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
        ApplicationCandidateResourceCeiling::bounded(
            left_resources
                .maximum_retained_representation_bytes()
                .max(right_resources.maximum_retained_representation_bytes()),
            left_resources
                .maximum_validator_work()
                .max(right_resources.maximum_validator_work()),
        ),
    )
}
