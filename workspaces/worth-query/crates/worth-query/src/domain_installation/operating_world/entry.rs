use crate::basis_lifecycle::{
    basis_lifecycle, AdmittedBasisCapability, BasisEligibilityCounters, BasisIntentDenial,
    BasisIntentDenialKind, BasisOperationLane, DeniedBasisCapability, DeniedBasisCapabilityKind,
    MutationPreparationLaneWitness, ObservationLaneWitness,
};

/// Query-owned authority entry for one installed operating world.
///
/// Callers choose semantic world intent through the constructors below. The
/// admitted basis remains private and is carried by Query after this boundary.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct WorthQueryOperatingWorldEntry<L: BasisOperationLane> {
    capability: AdmittedBasisCapability<L>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthQueryOperatingWorldEntryDenial {
    Intent(BasisIntentDenial),
    Admission(DeniedBasisCapability),
    Product {
        kind: WorthQueryOperatingWorldProductDenial,
        basis_eligibility: BasisEligibilityCounters,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperatingWorldProductDenial {
    SelectionRequired,
    RuntimeUnavailable,
    Admission(worth_query_execution::facade::primary_graph::WorthQueryProductBranchAdmissionDenial),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryOperatingWorldEntryDenialKind {
    Intent(BasisIntentDenialKind),
    Admission(DeniedBasisCapabilityKind),
    Product(WorthQueryOperatingWorldProductDenial),
}

impl WorthQueryOperatingWorldEntry<ObservationLaneWitness> {
    pub(crate) fn observe_current() -> Result<Self, WorthQueryOperatingWorldEntryDenial> {
        let path = basis_lifecycle()
            .current_head()
            .for_observation()
            .map_err(WorthQueryOperatingWorldEntryDenial::Intent)?;
        let admitted = path
            .admit()
            .map_err(WorthQueryOperatingWorldEntryDenial::Admission)?;
        Ok(Self {
            capability: admitted.capability().clone(),
        })
    }

    pub(crate) fn observe_branch(
        branch_identity: &super::WorthQueryBranchHeadIdentity,
    ) -> Result<Self, WorthQueryOperatingWorldEntryDenial> {
        Self::observe_branch_name(branch_identity.as_str())
    }

    pub(crate) fn observe_product_component(
        product: &worth_query_execution::facade::primary_graph::WorthQueryProductBranchLease,
    ) -> Result<Self, WorthQueryOperatingWorldEntryDenial> {
        Self::observe_branch_name(&product.relational_basis_descriptor().branch_id().0)
    }

    fn observe_branch_name(branch: &str) -> Result<Self, WorthQueryOperatingWorldEntryDenial> {
        let path = basis_lifecycle()
            .branch_head(branch, true)
            .for_observation()
            .map_err(WorthQueryOperatingWorldEntryDenial::Intent)?;
        let admitted = path
            .admit()
            .map_err(WorthQueryOperatingWorldEntryDenial::Admission)?;
        Ok(Self {
            capability: admitted.capability().clone(),
        })
    }
}

impl WorthQueryOperatingWorldEntry<MutationPreparationLaneWitness> {
    pub(crate) fn prepare_current_mutation() -> Result<Self, WorthQueryOperatingWorldEntryDenial> {
        Self::prepare_mutation(basis_lifecycle().current_head())
    }

    pub(crate) fn prepare_branch_mutation(
        branch_identity: &super::WorthQueryBranchHeadIdentity,
    ) -> Result<Self, WorthQueryOperatingWorldEntryDenial> {
        Self::prepare_mutation(basis_lifecycle().branch_head(branch_identity.as_str(), true))
    }

    fn prepare_mutation(
        basis: crate::basis_lifecycle::BasisLifecycleIntentDraft,
    ) -> Result<Self, WorthQueryOperatingWorldEntryDenial> {
        let capability = basis
            .for_mutation_preparation()
            .map_err(WorthQueryOperatingWorldEntryDenial::Intent)?
            .admit()
            .map_err(WorthQueryOperatingWorldEntryDenial::Admission)?;
        Ok(Self { capability })
    }
}

impl<L: BasisOperationLane> WorthQueryOperatingWorldEntry<L> {
    pub(crate) fn into_capability(self) -> AdmittedBasisCapability<L> {
        self.capability
    }
}

impl WorthQueryOperatingWorldEntryDenial {
    pub(crate) fn product(kind: WorthQueryOperatingWorldProductDenial) -> Self {
        Self::Product {
            kind,
            basis_eligibility: BasisEligibilityCounters::default(),
        }
    }

    pub fn kind(&self) -> WorthQueryOperatingWorldEntryDenialKind {
        match self {
            Self::Intent(denial) => {
                WorthQueryOperatingWorldEntryDenialKind::Intent(denial.denial_kind())
            }
            Self::Admission(denial) => {
                WorthQueryOperatingWorldEntryDenialKind::Admission(denial.denial_kind())
            }
            Self::Product { kind, .. } => WorthQueryOperatingWorldEntryDenialKind::Product(*kind),
        }
    }

    pub fn counters(&self) -> &BasisEligibilityCounters {
        match self {
            Self::Intent(denial) => denial.counters(),
            Self::Admission(denial) => denial.counters(),
            Self::Product {
                basis_eligibility, ..
            } => basis_eligibility,
        }
    }
}
