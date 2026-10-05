use super::support_row_defaults::{
    default_bases_for_requirement, support_posture_for_requirement, support_state_for_posture,
};
use super::{
    WorthQueryGraphIndexLifecycleClass, WorthQueryGraphIndexLifecycleOwner,
    WorthQueryGraphIndexPosture, WorthQueryGraphIndexSupportState,
};
use crate::graph_read_access::{
    WorthQueryAdmittedGraphReadRelationDirection, WorthQueryGraphReadAccessComplexityContract,
    WorthQueryGraphReadAccessInvalidationBasis, WorthQueryGraphReadAccessRebuildBasis,
    WorthQueryGraphReadAccessRequirementKind, WorthQueryGraphReadLifecycleClass,
    WorthQueryGraphReadOrderingPosture, WorthQueryGraphReadPredicateFamily,
};
use std::fmt::{self, Write};

mod admitted;
mod digest;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryGraphIndexSupportRow {
    digest: String,
    requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    supported_relation_direction: Option<WorthQueryAdmittedGraphReadRelationDirection>,
    supported_predicate_family: Option<WorthQueryGraphReadPredicateFamily>,
    supported_ordering_posture: Option<WorthQueryGraphReadOrderingPosture>,
    supported_requirement_lifecycle: Option<WorthQueryGraphReadLifecycleClass>,
    lifecycle_owner: WorthQueryGraphIndexLifecycleOwner,
    lifecycle_class: WorthQueryGraphIndexLifecycleClass,
    rebuild_basis: WorthQueryGraphReadAccessRebuildBasis,
    invalidation_basis: WorthQueryGraphReadAccessInvalidationBasis,
    complexity_contract: WorthQueryGraphReadAccessComplexityContract,
    posture: WorthQueryGraphIndexPosture,
    support_state: WorthQueryGraphIndexSupportState,
    owning_milestone: Option<String>,
}

impl WorthQueryGraphIndexSupportRow {
    pub fn digest(&self) -> &str {
        &self.digest
    }

    pub fn requirement_kind(&self) -> &WorthQueryGraphReadAccessRequirementKind {
        &self.requirement_kind
    }

    pub fn supported_relation_direction(
        &self,
    ) -> Option<&WorthQueryAdmittedGraphReadRelationDirection> {
        self.supported_relation_direction.as_ref()
    }

    pub fn supported_predicate_family(&self) -> Option<&WorthQueryGraphReadPredicateFamily> {
        self.supported_predicate_family.as_ref()
    }

    pub fn supported_ordering_posture(&self) -> Option<&WorthQueryGraphReadOrderingPosture> {
        self.supported_ordering_posture.as_ref()
    }

    pub fn supported_requirement_lifecycle(&self) -> Option<&WorthQueryGraphReadLifecycleClass> {
        self.supported_requirement_lifecycle.as_ref()
    }

    pub fn lifecycle_owner(&self) -> &WorthQueryGraphIndexLifecycleOwner {
        &self.lifecycle_owner
    }

    pub fn lifecycle_class(&self) -> &WorthQueryGraphIndexLifecycleClass {
        &self.lifecycle_class
    }

    pub fn rebuild_basis(&self) -> &WorthQueryGraphReadAccessRebuildBasis {
        &self.rebuild_basis
    }

    pub fn invalidation_basis(&self) -> &WorthQueryGraphReadAccessInvalidationBasis {
        &self.invalidation_basis
    }

    pub fn complexity_contract(&self) -> &WorthQueryGraphReadAccessComplexityContract {
        &self.complexity_contract
    }

    pub fn posture(&self) -> &WorthQueryGraphIndexPosture {
        &self.posture
    }

    pub fn support_state(&self) -> &WorthQueryGraphIndexSupportState {
        &self.support_state
    }

    pub fn owning_milestone(&self) -> Option<&str> {
        self.owning_milestone.as_deref()
    }

    pub fn for_requirement_kind(
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
    ) -> Self {
        let (rebuild_basis, invalidation_basis, complexity_contract) =
            default_bases_for_requirement(&requirement_kind);
        let (lifecycle_owner, lifecycle_class, posture, owning_milestone) =
            support_posture_for_requirement(&requirement_kind);
        let support_state = support_state_for_posture(&posture);
        Self::new(
            requirement_kind,
            None,
            None,
            None,
            None,
            lifecycle_owner,
            lifecycle_class,
            rebuild_basis,
            invalidation_basis,
            complexity_contract,
            posture,
            support_state,
            owning_milestone,
        )
    }

    pub fn with_runtime_support_posture(
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        lifecycle_owner: WorthQueryGraphIndexLifecycleOwner,
        lifecycle_class: WorthQueryGraphIndexLifecycleClass,
        posture: WorthQueryGraphIndexPosture,
        support_state: WorthQueryGraphIndexSupportState,
        owning_milestone: Option<String>,
    ) -> Self {
        let (rebuild_basis, invalidation_basis, complexity_contract) =
            default_bases_for_requirement(&requirement_kind);
        Self::new(
            requirement_kind,
            None,
            None,
            None,
            None,
            lifecycle_owner,
            lifecycle_class,
            rebuild_basis,
            invalidation_basis,
            complexity_contract,
            posture,
            support_state,
            owning_milestone,
        )
    }

    pub fn new(
        requirement_kind: WorthQueryGraphReadAccessRequirementKind,
        supported_relation_direction: Option<WorthQueryAdmittedGraphReadRelationDirection>,
        supported_predicate_family: Option<WorthQueryGraphReadPredicateFamily>,
        supported_ordering_posture: Option<WorthQueryGraphReadOrderingPosture>,
        supported_requirement_lifecycle: Option<WorthQueryGraphReadLifecycleClass>,
        lifecycle_owner: WorthQueryGraphIndexLifecycleOwner,
        lifecycle_class: WorthQueryGraphIndexLifecycleClass,
        rebuild_basis: WorthQueryGraphReadAccessRebuildBasis,
        invalidation_basis: WorthQueryGraphReadAccessInvalidationBasis,
        complexity_contract: WorthQueryGraphReadAccessComplexityContract,
        posture: WorthQueryGraphIndexPosture,
        support_state: WorthQueryGraphIndexSupportState,
        owning_milestone: Option<String>,
    ) -> Self {
        debug_assert!(
            posture != WorthQueryGraphIndexPosture::Verified
                || support_state.certifies_verified_support()
        );
        let mut row = Self {
            digest: String::new(),
            requirement_kind,
            supported_relation_direction,
            supported_predicate_family,
            supported_ordering_posture,
            supported_requirement_lifecycle,
            lifecycle_owner,
            lifecycle_class,
            rebuild_basis,
            invalidation_basis,
            complexity_contract,
            posture,
            support_state,
            owning_milestone,
        };
        row.digest = row.recompute_digest();
        row
    }

    pub fn digest_part(&self) -> String {
        let mut text = String::new();
        self.write_digest_part(&mut text)
            .expect("String formatting cannot fail");
        text
    }

    pub(crate) fn write_digest_part(&self, output: &mut dyn Write) -> fmt::Result {
        write!(output, "row:{}", self.digest)
    }

    pub fn with_supported_relation_direction(
        mut self,
        direction: WorthQueryAdmittedGraphReadRelationDirection,
    ) -> Self {
        self.supported_relation_direction = Some(direction);
        self.digest = self.recompute_digest();
        self
    }

    pub fn with_supported_predicate_family(
        mut self,
        family: WorthQueryGraphReadPredicateFamily,
    ) -> Self {
        self.supported_predicate_family = Some(family);
        self.digest = self.recompute_digest();
        self
    }

    pub fn with_supported_ordering_posture(
        mut self,
        posture: WorthQueryGraphReadOrderingPosture,
    ) -> Self {
        self.supported_ordering_posture = Some(posture);
        self.digest = self.recompute_digest();
        self
    }

    pub fn with_supported_requirement_lifecycle(
        mut self,
        lifecycle: WorthQueryGraphReadLifecycleClass,
    ) -> Self {
        self.supported_requirement_lifecycle = Some(lifecycle);
        self.digest = self.recompute_digest();
        self
    }

    pub fn with_rebuild_basis(
        mut self,
        rebuild_basis: WorthQueryGraphReadAccessRebuildBasis,
    ) -> Self {
        self.rebuild_basis = rebuild_basis;
        self.digest = self.recompute_digest();
        self
    }

    pub fn with_invalidation_basis(
        mut self,
        invalidation_basis: WorthQueryGraphReadAccessInvalidationBasis,
    ) -> Self {
        self.invalidation_basis = invalidation_basis;
        self.digest = self.recompute_digest();
        self
    }

    pub fn with_complexity_contract(
        mut self,
        complexity_contract: WorthQueryGraphReadAccessComplexityContract,
    ) -> Self {
        self.complexity_contract = complexity_contract;
        self.digest = self.recompute_digest();
        self
    }

    fn recompute_digest(&self) -> String {
        digest::ordinary_digest(self)
    }
}
