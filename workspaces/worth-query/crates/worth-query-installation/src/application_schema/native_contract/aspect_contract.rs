use std::collections::BTreeSet;
use std::sync::Arc;

use worth_foundational::facade::{
    AspectBinding, AspectContract, CanonicalBasisReadyArtifact, FieldKey,
};

use super::WorthQueryInstalledApplicationAspectLocus;

/// Installed, immutable native contract for one application entity/aspect locus.
#[derive(Clone)]
pub struct WorthQueryInstalledApplicationAspectContract {
    locus: WorthQueryInstalledApplicationAspectLocus,
    contract: AspectContract,
    fields: BTreeSet<FieldKey>,
    binding: AspectBinding,
    canonical_contract_basis: Arc<CanonicalBasisReadyArtifact>,
    canonical_contract_material: Arc<str>,
}

impl WorthQueryInstalledApplicationAspectContract {
    pub(crate) fn new(
        locus: WorthQueryInstalledApplicationAspectLocus,
        contract: AspectContract,
        fields: BTreeSet<FieldKey>,
        binding: AspectBinding,
        canonical_contract_basis: CanonicalBasisReadyArtifact,
        canonical_contract_material: String,
    ) -> Self {
        Self {
            locus,
            contract,
            fields,
            binding,
            canonical_contract_basis: Arc::new(canonical_contract_basis),
            canonical_contract_material: Arc::from(canonical_contract_material),
        }
    }

    pub fn locus(&self) -> &WorthQueryInstalledApplicationAspectLocus {
        &self.locus
    }

    pub fn contract(&self) -> &AspectContract {
        &self.contract
    }

    pub fn fields(&self) -> impl ExactSizeIterator<Item = &FieldKey> {
        self.fields.iter()
    }

    pub fn contains_field(&self, field: &FieldKey) -> bool {
        self.fields.contains(field)
    }

    pub fn binding(&self) -> &AspectBinding {
        &self.binding
    }

    /// Interpret only fields belonging to this actual installed storage binding.
    pub(crate) fn field_value_contract(
        &self,
        field: &FieldKey,
    ) -> Option<(
        worth_foundational::facade::ScalarAspectType,
        worth_foundational::facade::FieldRequirement,
        Option<worth_foundational::facade::CanonicalDigestId>,
    )> {
        use worth_foundational::facade::{AspectShape, FieldRequirement};
        if !self.contains_field(field) {
            return None;
        }
        match (self.contract.shape(), &self.binding) {
            (AspectShape::Scalar(family), AspectBinding::EntityField { field: physical })
                if physical == field && self.fields.len() == 1 =>
            {
                Some((*family, FieldRequirement::Required, None))
            }
            (AspectShape::Struct(shape), _) => shape
                .field(field)
                .map(|field| (field.value_type(), field.requirement(), field.codec())),
            _ => None,
        }
    }

    pub fn canonical_contract_basis(&self) -> &CanonicalBasisReadyArtifact {
        self.canonical_contract_basis.as_ref()
    }

    pub(crate) fn retain_canonical_contract_basis(&self) -> Arc<CanonicalBasisReadyArtifact> {
        Arc::clone(&self.canonical_contract_basis)
    }

    pub fn canonical_contract_material(&self) -> &str {
        self.canonical_contract_material.as_ref()
    }

    pub(crate) fn retain_canonical_contract_material(&self) -> Arc<str> {
        Arc::clone(&self.canonical_contract_material)
    }
}

impl std::fmt::Debug for WorthQueryInstalledApplicationAspectContract {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("WorthQueryInstalledApplicationAspectContract")
            .field("locus", &self.locus)
            .field("contract", &self.contract)
            .field("fields", &self.fields)
            .field("binding", &self.binding)
            .finish_non_exhaustive()
    }
}
