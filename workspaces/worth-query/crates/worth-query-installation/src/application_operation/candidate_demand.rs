use worth_query_declaration::facade::application_operation::ApplicationMutationBindingDescriptor;

/// Exact largest candidate reservation declared for one installed operation.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(in crate::application_operation) struct WorthQueryApplicationCandidateDemand {
    candidate_items: u64,
    retained_representation_bytes: u64,
    validator_work: u64,
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
                    candidate_items,
                    retained_representation_bytes: u64::try_from(
                        requirements
                            .resources()
                            .maximum_retained_representation_bytes(),
                    )
                    .unwrap_or(u64::MAX),
                    validator_work: u64::try_from(
                        requirements.resources().maximum_validator_work(),
                    )
                    .unwrap_or(u64::MAX),
                }
            })
            .fold(Self::default(), |maximum, candidate| Self {
                candidate_items: maximum.candidate_items.max(candidate.candidate_items),
                retained_representation_bytes: maximum
                    .retained_representation_bytes
                    .max(candidate.retained_representation_bytes),
                validator_work: maximum.validator_work.max(candidate.validator_work),
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
}
