use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateCardinalityCeiling, ApplicationCandidatePolicy,
    ApplicationCandidateRequirements, ApplicationCandidateResourceCeiling,
    ApplicationMutationBindingDescriptor,
};

/// Installed policy quote, separate from a handler's concrete actual demand.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::application_operation) struct WorthQueryApplicationCandidateDemand {
    candidate_items: Option<u64>,
    retained_representation_bytes: Option<u64>,
    candidate_ceiling: Option<ApplicationCandidatePolicy>,
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

const WORKFLOW_SETTLEMENT_RETAINED_BYTES: u64 = 65_536;

fn workflow_settlement_ceiling() -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        // transition identity, actor/executor triplets and Ready-state write.
        ApplicationCandidateCardinalityCeiling::fixed(1, 0, 2, 1, 7 + 3 + 3 + 1 + 1, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(
            WORKFLOW_SETTLEMENT_RETAINED_BYTES as usize,
        ),
    )
}

impl WorthQueryApplicationCandidateDemand {
    pub(in crate::application_operation) fn with_workflow_birth(
        mut self,
        enrolled: bool,
    ) -> Result<Self, ()> {
        if enrolled {
            if self.workflow_settlement_ceiling.is_some() {
                return Err(());
            }
            self.workflow_settlement_ceiling = Some(ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(1, 0, 4, 0, 11, 0),
                ApplicationCandidateResourceCeiling::representation_bytes(65_536),
            ));
            self.include_platform_ceiling()?;
        }
        Ok(self)
    }

    pub(in crate::application_operation) fn with_signing_lifecycle(
        mut self,
        ceiling: Option<ApplicationCandidateRequirements>,
    ) -> Result<Self, ()> {
        if let Some(ceiling) = ceiling {
            self.candidate_items = add_optional(
                self.candidate_items,
                cardinality_items(ceiling.cardinality())?,
            )?;
            self.retained_representation_bytes =
                add_optional(self.retained_representation_bytes, retained_bytes(ceiling)?)?;
        }
        Ok(self)
    }

    pub(in crate::application_operation) fn derive(
        bindings: &[ApplicationMutationBindingDescriptor],
        operation: &str,
        input_type: &str,
    ) -> Result<Self, ()> {
        let mut demand = Self::default();
        for binding in bindings.iter().filter(|binding| {
            binding.operation_name() == operation && binding.input_identity().as_str() == input_type
        }) {
            let policy = binding.candidates();
            // An absent policy is absorbing in the union, independently per dimension.
            demand.candidate_ceiling = Some(match demand.candidate_ceiling {
                None => policy,
                Some(previous) => maximum_policy(previous, policy),
            });
            if binding.requires_workflow_authority() {
                demand.workflow_settlement_ceiling = Some(workflow_settlement_ceiling());
            }
        }
        demand.include_platform_ceiling()?;
        Ok(demand)
    }

    fn include_platform_ceiling(&mut self) -> Result<(), ()> {
        if let Some(policy) = self.candidate_ceiling {
            let sidecar = self.workflow_settlement_ceiling;
            let items = policy.cardinality().map(cardinality_items).transpose()?;
            let bytes = policy
                .resources()
                .map(|resources| {
                    u64::try_from(resources.maximum_retained_representation_bytes()).map_err(|_| ())
                })
                .transpose()?;
            let items = add_optional(
                items,
                sidecar.map_or(Ok(0), |s| cardinality_items(s.cardinality()))?,
            )?;
            let bytes = add_optional(bytes, sidecar.map_or(Ok(0), retained_bytes)?)?;
            self.candidate_items = maximum_optional(self.candidate_items, items);
            self.retained_representation_bytes =
                maximum_optional(self.retained_representation_bytes, bytes);
        } else if let Some(sidecar) = self.workflow_settlement_ceiling {
            self.candidate_items = maximum_optional(
                self.candidate_items,
                Some(cardinality_items(sidecar.cardinality())?),
            );
            self.retained_representation_bytes = maximum_optional(
                self.retained_representation_bytes,
                Some(retained_bytes(sidecar)?),
            );
        }
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
    ) -> Option<ApplicationCandidatePolicy> {
        self.candidate_ceiling
    }
    pub(in crate::application_operation) const fn workflow_settlement_ceiling(
        self,
    ) -> Option<ApplicationCandidateRequirements> {
        self.workflow_settlement_ceiling
    }
}

fn maximum_optional(left: Option<u64>, right: Option<u64>) -> Option<u64> {
    Some(left?.max(right?))
}

fn add_optional(value: Option<u64>, added: u64) -> Result<Option<u64>, ()> {
    value
        .map(|value| value.checked_add(added).ok_or(()))
        .transpose()
}

fn maximum_policy(
    left: ApplicationCandidatePolicy,
    right: ApplicationCandidatePolicy,
) -> ApplicationCandidatePolicy {
    let cardinality = left
        .cardinality()
        .zip(right.cardinality())
        .map(|(left, right)| maximum_cardinality(left, right));
    let resources = left
        .resources()
        .zip(right.resources())
        .map(|(left, right)| maximum_resources(left, right));
    ApplicationCandidatePolicy::new(cardinality, resources)
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

#[cfg(test)]
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
    left: ApplicationCandidateCardinalityCeiling,
    right: ApplicationCandidateCardinalityCeiling,
) -> ApplicationCandidateCardinalityCeiling {
    ApplicationCandidateCardinalityCeiling::fixed(
        left.maximum_creates().max(right.maximum_creates()),
        left.maximum_deletes().max(right.maximum_deletes()),
        left.maximum_links().max(right.maximum_links()),
        left.maximum_unlinks().max(right.maximum_unlinks()),
        left.maximum_writes().max(right.maximum_writes()),
        left.maximum_emits().max(right.maximum_emits()),
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
mod tests;
