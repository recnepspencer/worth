mod base_commit_patch_basis;
mod numeric_value_basis;
mod performance_receipt;
mod record_read_basis;
mod scalar_policy_binding;
mod source_basis;
mod visible_record_basis;

pub(crate) use numeric_value_basis::PolicyNumericValueBasis;
pub(crate) use performance_receipt::{PolicyValueLookupCounters, PolicyValueLookupReceipt};
pub(crate) use record_read_basis::resolve_policy_aspect_value_basis;
pub(crate) use scalar_policy_binding::{ScalarPolicyAspectBinding, ScalarPolicyBindingDenial};

use worth_foundational::facade::{
    prepare_locator_for_canonical_basis, AspectLocator, AspectValue, AspectValueLocator,
    CanonicalBasisReadyArtifact, CanonicalLocatorInput, CanonicalizationRuleVersion,
    EquivalenceBasisId, FoundationalBoundaryEvidenceSourceBasis, FoundationalBranchId,
    FoundationalMergeBaseSelectionBasis, FoundationalMergeBasis, FoundationalStrategyBasis,
    FoundationalTransitionBasisFamily, FoundationalTransitionBasisIdentity,
    FoundationalTransitionBasisVersion, LocatorAuthority,
};
use worth_proof::TransitionOutcome;

use source_basis::PolicyValueSourceBasis;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyValueLookupFailure {
    MissingRecordBasis,
    MissingField,
    InvalidValueShape,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PolicyValueProvenance {
    SourceVisibleState,
    TargetVisibleState,
    BaseReadViewState,
    BaseCommitPatch,
}

impl PolicyValueProvenance {
    pub(super) const fn source_basis_label(self) -> &'static str {
        match self {
            Self::SourceVisibleState => "source_visible_state",
            Self::TargetVisibleState => "target_visible_state",
            Self::BaseReadViewState => "base_read_view_state",
            Self::BaseCommitPatch => "base_commit_patch",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PolicyScalarValue {
    value: AspectValue,
    provenance: PolicyValueProvenance,
    locator: AspectValueLocator,
    locator_basis: Option<CanonicalBasisReadyArtifact>,
    source_basis: FoundationalBoundaryEvidenceSourceBasis,
}

impl PolicyScalarValue {
    fn new(
        value: AspectValue,
        provenance: PolicyValueProvenance,
        aspect_key: &worth_foundational::facade::AspectKey,
        source_basis: PolicyValueSourceBasis,
    ) -> Self {
        let locator = AspectValueLocator::whole_aspect(AspectLocator::new(
            LocatorAuthority::Authoritative,
            aspect_key.clone(),
        ));
        let locator_basis = prepare_policy_value_locator_for_canonical_basis(&locator);
        Self {
            value,
            provenance,
            locator,
            locator_basis,
            source_basis: source_basis.into_foundational_source_basis(provenance, aspect_key),
        }
    }

    pub(crate) fn value(&self) -> &AspectValue {
        &self.value
    }

    pub(crate) const fn provenance(&self) -> PolicyValueProvenance {
        self.provenance
    }

    pub(crate) fn locator(&self) -> &AspectValueLocator {
        &self.locator
    }

    pub(crate) fn locator_basis(&self) -> Option<&CanonicalBasisReadyArtifact> {
        self.locator_basis.as_ref()
    }

    pub(crate) fn source_basis(&self) -> &FoundationalBoundaryEvidenceSourceBasis {
        &self.source_basis
    }
}

fn prepare_policy_value_locator_for_canonical_basis(
    locator: &AspectValueLocator,
) -> Option<CanonicalBasisReadyArtifact> {
    let version = CanonicalizationRuleVersion::new("worth.relational.merge.policy_value_basis.v1")
        .expect(/* Proof: This constructor receives a fixed nonempty literal in the required grammar and version/domain; it has no runtime-dependent input. */ "policy value basis canonicalization version is static and non-empty");
    match prepare_locator_for_canonical_basis(
        version,
        CanonicalLocatorInput::Value(locator.clone()),
    ) {
        TransitionOutcome::Success(ready) => Some(ready),
        TransitionOutcome::Denied(_) => None,
        TransitionOutcome::Deferred(_)
        | TransitionOutcome::RebindRequired(_)
        | TransitionOutcome::Failed(_) => None,
    }
}

#[derive(Debug, Clone)]
pub(crate) struct PolicyAspectValueBasis {
    binding: ScalarPolicyAspectBinding,
    merge_basis: Option<FoundationalMergeBasis>,
    merge_base_selection_basis: FoundationalMergeBaseSelectionBasis,
    strategy_basis: FoundationalStrategyBasis,
    source: Result<PolicyScalarValue, PolicyValueLookupFailure>,
    target: Result<PolicyScalarValue, PolicyValueLookupFailure>,
    base: Result<PolicyScalarValue, PolicyValueLookupFailure>,
    receipt: PolicyValueLookupReceipt,
}

impl PolicyAspectValueBasis {
    fn new(
        binding: ScalarPolicyAspectBinding,
        merge_basis: Option<FoundationalMergeBasis>,
        merge_base_selection_basis: FoundationalMergeBaseSelectionBasis,
        strategy_basis: FoundationalStrategyBasis,
        source: Result<PolicyScalarValue, PolicyValueLookupFailure>,
        target: Result<PolicyScalarValue, PolicyValueLookupFailure>,
        base: Result<PolicyScalarValue, PolicyValueLookupFailure>,
        receipt: PolicyValueLookupReceipt,
    ) -> Self {
        Self {
            binding,
            merge_basis,
            merge_base_selection_basis,
            strategy_basis,
            source,
            target,
            base,
            receipt,
        }
    }

    pub(crate) fn binding(&self) -> &ScalarPolicyAspectBinding {
        &self.binding
    }

    pub(crate) fn merge_basis(&self) -> Option<&FoundationalMergeBasis> {
        self.merge_basis.as_ref()
    }

    pub(crate) const fn merge_base_selection_basis(&self) -> FoundationalMergeBaseSelectionBasis {
        self.merge_base_selection_basis
    }

    pub(crate) const fn strategy_basis(&self) -> FoundationalStrategyBasis {
        self.strategy_basis
    }

    pub(crate) fn source(&self) -> Result<&PolicyScalarValue, PolicyValueLookupFailure> {
        self.source.as_ref().map_err(|failure| *failure)
    }

    pub(crate) fn target(&self) -> Result<&PolicyScalarValue, PolicyValueLookupFailure> {
        self.target.as_ref().map_err(|failure| *failure)
    }

    pub(crate) fn base(&self) -> Result<&PolicyScalarValue, PolicyValueLookupFailure> {
        self.base.as_ref().map_err(|failure| *failure)
    }

    pub(crate) fn numeric(&self) -> PolicyNumericValueBasis<'_> {
        PolicyNumericValueBasis::new(self)
    }

    pub(crate) fn receipt(&self) -> &PolicyValueLookupReceipt {
        &self.receipt
    }
}

pub(crate) fn foundational_policy_value_transition_basis(
    source_branch: &crate::history::data::BranchId,
    target_branch: &crate::history::data::BranchId,
    aspect_key: &worth_foundational::facade::AspectKey,
    base_commit_id: crate::history::data::CommitId,
) -> (
    Option<FoundationalMergeBasis>,
    FoundationalMergeBaseSelectionBasis,
    FoundationalStrategyBasis,
) {
    let basis_id = EquivalenceBasisId::new(policy_value_basis_id(
        source_branch,
        target_branch,
        aspect_key,
        base_commit_id,
    ));
    (
        foundational_merge_basis(source_branch, target_branch, basis_id),
        FoundationalMergeBaseSelectionBasis::new(basis_id),
        FoundationalStrategyBasis::new(basis_id),
    )
}

fn foundational_merge_basis(
    source_branch: &crate::history::data::BranchId,
    target_branch: &crate::history::data::BranchId,
    basis_id: EquivalenceBasisId,
) -> Option<FoundationalMergeBasis> {
    Some(FoundationalMergeBasis::new(
        FoundationalTransitionBasisIdentity::new(basis_id),
        FoundationalTransitionBasisFamily::new("worth.relational.merge.policy_value_basis").ok()?,
        FoundationalTransitionBasisVersion::new("v1").ok()?,
        FoundationalBranchId::new(source_branch.0.clone()).ok()?,
        FoundationalBranchId::new(target_branch.0.clone()).ok()?,
    ))
}

fn policy_value_basis_id(
    source_branch: &crate::history::data::BranchId,
    target_branch: &crate::history::data::BranchId,
    aspect_key: &worth_foundational::facade::AspectKey,
    base_commit_id: crate::history::data::CommitId,
) -> u64 {
    let mut hash = 14695981039346656037_u64;
    mix_policy_value_basis_bytes(&mut hash, source_branch.0.as_bytes());
    mix_policy_value_basis_bytes(&mut hash, target_branch.0.as_bytes());
    mix_policy_value_basis_bytes(&mut hash, aspect_key.as_str().as_bytes());
    mix_policy_value_basis_bytes(&mut hash, &base_commit_id.0.to_le_bytes());
    hash
}

fn mix_policy_value_basis_bytes(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash ^= u64::from(*byte);
        *hash = hash.wrapping_mul(1099511628211_u64);
    }
}
