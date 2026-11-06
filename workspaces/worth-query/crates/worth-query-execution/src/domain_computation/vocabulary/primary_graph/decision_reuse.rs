//! Moved whole from `domain_computation::primary_graph::application_attempt::read_set::decision_reuse::court_doors_for_test`: the items below and every impl they own.
use crate::domain_computation::vocabulary::authorization::operation_scope_binding::WorthQueryDecisionScopeWitness;
use crate::domain_computation::vocabulary::primary_graph::decision_context_use::DecisionContextUse;
use crate::domain_computation::vocabulary::primary_graph::freshness::WorthQueryPrincipalReuseWitness;
use crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryDecisionContextDependencies;
use crate::domain_computation::vocabulary::primary_graph::input_reuse_contract::WorthQueryProducerInputReuseContract;
use worth_foundational::facade::DeterminismContract;

impl PreparedDecisionReuseContext {
    #[doc(hidden)]
    pub fn matches_completed_for_test(&self, prior: &CompletedDecisionReuseProof) -> bool {
        Self::matches_completed(self, prior)
    }
}

/// Invocation evidence awaiting the completed, exactly covered handler read.
/// The static producer declaration supplies the upper bound on context use.
pub struct PreparedDecisionReuseContext {
    contract: WorthQueryProducerInputReuseContract,
    key: Option<[u8; 32]>,
    principal: Option<WorthQueryPrincipalReuseWitness>,
    scope: Option<WorthQueryDecisionScopeWitness>,
}

/// Recorded fixed native and canonical context for a completed producer read.
/// An absent proof is ineligible; the handler fact count remains independent.
pub struct CompletedDecisionReuseProof {
    key: Option<[u8; 32]>,
    principal: Option<WorthQueryPrincipalReuseWitness>,
    scope: Option<WorthQueryDecisionScopeWitness>,
}

impl CompletedDecisionReuseProof {
    /// The completed decision a republished output continues. Restoring the
    /// exact retained output makes no new decision, so its recorded context
    /// is still the one that produced this output.
    pub(in crate::domain_computation) fn continued_by_republication(&self) -> Self {
        Self {
            key: self.key,
            principal: self.principal,
            scope: self.scope.clone(),
        }
    }
}

impl PreparedDecisionReuseContext {
    pub(in crate::domain_computation) fn new(
        contract: WorthQueryProducerInputReuseContract,
        key: Option<[u8; 32]>,
        principal: Option<WorthQueryPrincipalReuseWitness>,
        scope: Option<WorthQueryDecisionScopeWitness>,
    ) -> Option<Self> {
        if contract.determinism() != DeterminismContract::CanonicalBitwise {
            return None;
        }
        let declared = contract.context();
        if declared.contains(WorthQueryDecisionContextDependencies::KEY) != key.is_some()
            || declared.contains(WorthQueryDecisionContextDependencies::PRINCIPAL)
                != principal.is_some()
            || declared.contains(WorthQueryDecisionContextDependencies::SCOPE) != scope.is_some()
        {
            return None;
        }
        Some(Self {
            contract,
            key,
            principal,
            scope,
        })
    }

    pub(in crate::domain_computation) fn seal_after_completed_read(
        self,
        actual: DecisionContextUse,
    ) -> Option<CompletedDecisionReuseProof> {
        if actual.has_untracked_context()
            || actual.declared_context_bits() & !self.contract.context().bits() != 0
        {
            return None;
        }
        Some(CompletedDecisionReuseProof {
            key: self.key,
            principal: self.principal,
            scope: self.scope,
        })
    }

    /// A fresh invocation can compare its descriptive admitted context with
    /// prior completed evidence without pretending the new handler ran.
    pub(in crate::domain_computation) fn matches_completed(
        &self,
        prior: &CompletedDecisionReuseProof,
    ) -> bool {
        self.key == prior.key && self.principal == prior.principal && self.scope == prior.scope
    }
}

mod court_doors_for_test;
