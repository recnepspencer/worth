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
    candidate_ceiling: Option<ApplicationCandidateRequirements>,
    workflow_settlement_ceiling: Option<ApplicationCandidateRequirements>,
}

// Query owns this sidecar capacity. It does not enlarge a product handler's
// declared writer reservation; it bounds one local workflow settlement.
const WORKFLOW_SETTLEMENT_RETAINED_BYTES: u64 = 65_536;

fn workflow_settlement_ceiling() -> ApplicationCandidateRequirements {
    ApplicationCandidateRequirements::fixed_shape(
        // transition: seven structural fields, advancer/executor triplets,
        // receipt identity; instance: state and the ending actor for Terminated.
        // The producer is
        // workflow/instance/transition/settlement::visit_transition_facts_from_basis.
        ApplicationCandidateCardinalityCeiling::fixed(1, 0, 2, 1, 7 + 3 + 3 + 1 + 1 + 3 + 2, 0),
        ApplicationCandidateResourceCeiling::representation_bytes(
            WORKFLOW_SETTLEMENT_RETAINED_BYTES as usize,
        ),
    )
}

fn output_settlement_ceiling(
    binding: &ApplicationMutationBindingDescriptor,
) -> Result<ApplicationCandidateRequirements, ()> {
    // Fixed inventories have a tight structural bound. Family members already
    // consume the product's retained-representation budget, at least one byte
    // each; that existing budget bounds the dynamic inventory without a new quota.
    let dynamic = if binding.output_role_families().is_empty() {
        0
    } else {
        binding
            .candidates()
            .resources()
            .maximum_retained_representation_bytes()
    };
    let outputs = binding
        .output_roles()
        .len()
        .checked_add(dynamic)
        .ok_or(())?;
    settlement_ceiling_for_outputs(outputs)
}

fn settlement_ceiling_for_outputs(outputs: usize) -> Result<ApplicationCandidateRequirements, ()> {
    let base = workflow_settlement_ceiling();
    let count = base.cardinality();
    Ok(ApplicationCandidateRequirements::fixed_shape(
        ApplicationCandidateCardinalityCeiling::fixed(
            count.maximum_creates().checked_add(outputs).ok_or(())?,
            0,
            count
                .maximum_links()
                .checked_add(outputs.checked_mul(2).ok_or(())?)
                .ok_or(())?,
            count.maximum_unlinks(),
            count
                .maximum_writes()
                .checked_add(outputs.checked_mul(3).ok_or(())?)
                .ok_or(())?,
            0,
        ),
        // Capture shares the existing bounded native settlement byte budget.
        // A context that cannot fit refuses before the business commit.
        base.resources(),
    ))
}

impl WorthQueryApplicationCandidateDemand {
    pub(in crate::application_operation) fn with_workflow_succession(
        mut self,
        succeeds: bool,
    ) -> Result<Self, ()> {
        if succeeds {
            if self.workflow_settlement_ceiling.is_some() {
                return Err(());
            }
            // A fresh existing-subject instance, its three native links and the
            // protected predecessor link, nine identity/subject fields, start
            // revision, admitted actor triplet and optional deadline.
            self.workflow_settlement_ceiling = Some(ApplicationCandidateRequirements::fixed_shape(
                ApplicationCandidateCardinalityCeiling::fixed(1, 0, 4, 0, 14, 0),
                ApplicationCandidateResourceCeiling::representation_bytes(65_536),
            ));
            self.include_platform_ceiling()?;
        }
        Ok(self)
    }
    pub(in crate::application_operation) fn with_workflow_birth(
        mut self,
        enrolled: bool,
    ) -> Result<Self, ()> {
        if enrolled {
            if self.workflow_settlement_ceiling.is_some() {
                return Err(());
            }
            // One fresh instance, four endpoint links, six identity/state
            // fields, start revision, admitted actor and optional native deadline. Never broadens the
            // business handler's independently admitted writer reservation.
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
            self.candidate_items = self
                .candidate_items
                .checked_add(cardinality_items(ceiling.cardinality())?)
                .ok_or(())?;
            self.retained_representation_bytes = self
                .retained_representation_bytes
                .checked_add(retained_bytes(ceiling)?)
                .ok_or(())?;
        }
        Ok(self)
    }
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
                let workflow_settlement = binding
                    .requires_workflow_authority()
                    .then(|| output_settlement_ceiling(binding))
                    .transpose()?;
                let cardinality = requirements.cardinality();
                let candidate_items = cardinality_items(cardinality)?;
                Ok(Self {
                    candidate_items: candidate_items
                        .checked_add(
                            workflow_settlement
                                .map(|ceiling| cardinality_items(ceiling.cardinality()))
                                .transpose()?
                                .unwrap_or(0),
                        )
                        .ok_or(())?,
                    retained_representation_bytes: u64::try_from(
                        requirements
                            .resources()
                            .maximum_retained_representation_bytes(),
                    )
                    .map_err(|_| ())?
                    .checked_add(if workflow_settlement.is_some() {
                        WORKFLOW_SETTLEMENT_RETAINED_BYTES
                    } else {
                        0
                    })
                    .ok_or(())?,
                    candidate_ceiling: Some(requirements),
                    workflow_settlement_ceiling: workflow_settlement,
                })
            })
            .try_fold(
                Self::default(),
                |maximum, candidate: Result<Self, ()>| -> Result<Self, ()> {
                    Ok(maximum.maximum(candidate?))
                },
            )?;
        demand.include_platform_ceiling()?;
        Ok(demand)
    }

    fn maximum(self, other: Self) -> Self {
        let merge = |left, right| match (left, right) {
            (Some(left), Some(right)) => Some(maximum_requirements(left, right)),
            (Some(ceiling), None) | (None, Some(ceiling)) => Some(ceiling),
            (None, None) => None,
        };
        Self {
            candidate_items: self.candidate_items.max(other.candidate_items),
            retained_representation_bytes: self
                .retained_representation_bytes
                .max(other.retained_representation_bytes),
            candidate_ceiling: merge(self.candidate_ceiling, other.candidate_ceiling),
            workflow_settlement_ceiling: merge(
                self.workflow_settlement_ceiling,
                other.workflow_settlement_ceiling,
            ),
        }
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
            self.candidate_items = self.candidate_items.max(items);
            self.retained_representation_bytes = self.retained_representation_bytes.max(bytes);
        }
        Ok(())
    }

    pub(in crate::application_operation) const fn candidate_items(self) -> u64 {
        self.candidate_items
    }

    pub(in crate::application_operation) const fn retained_representation_bytes(self) -> u64 {
        self.retained_representation_bytes
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
    ApplicationCandidateResourceCeiling::representation_bytes(bytes)
}

#[cfg(test)]
mod tests;
