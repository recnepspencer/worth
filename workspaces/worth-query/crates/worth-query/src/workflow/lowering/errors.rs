use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::workflow::{
    QueryWorkflowDeclaration, WorkflowDeclarationFamily, WorkflowLoweringCounters,
};

use super::counters::{lowering_denial_counters, LoweringDenialClass};
use super::terms::WorkflowStalenessClass;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorkflowLoweringFailureClass {
    EvidenceIdentityDerivationDenied(IdentityDerivationDenial),
    InvalidWorkflowDeclarationFamily,
    UnsupportedMergeFamily,
    UnsupportedRelationalStrategyTarget,
    UnsupportedWritebackFamily,
    InvalidMergeBranchPairing,
    UnsupportedWritebackCausality,
    StaleWorkflowDenied,
    ExplicitRebindRequired,
    LoweringSerializationFailed,
}

impl WorkflowLoweringFailureClass {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::EvidenceIdentityDerivationDenied(_) => "evidence_identity_derivation_denied",
            Self::InvalidWorkflowDeclarationFamily => "invalid_workflow_declaration_family",
            Self::UnsupportedMergeFamily => "unsupported_merge_family",
            Self::UnsupportedRelationalStrategyTarget => "unsupported_relational_strategy_target",
            Self::UnsupportedWritebackFamily => "unsupported_writeback_family",
            Self::InvalidMergeBranchPairing => "invalid_merge_branch_pairing",
            Self::UnsupportedWritebackCausality => "unsupported_writeback_causality",
            Self::StaleWorkflowDenied => "stale_workflow_denied",
            Self::ExplicitRebindRequired => "explicit_rebind_required",
            Self::LoweringSerializationFailed => "lowering_serialization_failed",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorkflowLoweringError {
    failure_class: WorkflowLoweringFailureClass,
    message: &'static str,
    staleness_class: WorkflowStalenessClass,
    counters: WorkflowLoweringCounters,
}

impl WorkflowLoweringError {
    pub(super) fn identity_derivation_denied(
        cause: IdentityDerivationDenial,
        staleness_class: WorkflowStalenessClass,
        counters: WorkflowLoweringCounters,
    ) -> Self {
        Self::new(
            WorkflowLoweringFailureClass::EvidenceIdentityDerivationDenied(cause),
            "workflow lowering evidence identity refused",
            staleness_class,
            counters,
        )
    }

    pub(super) fn new(
        failure_class: WorkflowLoweringFailureClass,
        message: &'static str,
        staleness_class: WorkflowStalenessClass,
        counters: WorkflowLoweringCounters,
    ) -> Self {
        Self {
            failure_class,
            message,
            staleness_class,
            counters,
        }
    }

    pub fn failure_class(&self) -> &WorkflowLoweringFailureClass {
        &self.failure_class
    }

    pub fn message(&self) -> &'static str {
        self.message
    }

    pub fn staleness_class(&self) -> &WorkflowStalenessClass {
        &self.staleness_class
    }

    pub fn counters(&self) -> &WorkflowLoweringCounters {
        &self.counters
    }
}

pub(super) fn ensure_mutation_workflow_family(
    declaration: &QueryWorkflowDeclaration,
) -> Result<(), WorkflowLoweringError> {
    if declaration.request().declaration_family()
        == &WorkflowDeclarationFamily::MutationLoweringNarrow
    {
        Ok(())
    } else {
        Err(WorkflowLoweringError::new(
            WorkflowLoweringFailureClass::InvalidWorkflowDeclarationFamily,
            "workflow lowering entrypoints may only lower their matching declaration family",
            WorkflowStalenessClass::ExactBasisPreserved,
            lowering_denial_counters(0, LoweringDenialClass::General),
        ))
    }
}

pub(super) fn ensure_merge_workflow_family(
    declaration: &QueryWorkflowDeclaration,
) -> Result<(), WorkflowLoweringError> {
    if declaration.request().declaration_family() == &WorkflowDeclarationFamily::MergeLoweringNarrow
    {
        Ok(())
    } else {
        Err(WorkflowLoweringError::new(
            WorkflowLoweringFailureClass::UnsupportedMergeFamily,
            "merge lowering entrypoints may only lower admitted merge workflow declarations",
            WorkflowStalenessClass::ExactBasisPreserved,
            lowering_denial_counters(0, LoweringDenialClass::MergeDenied),
        ))
    }
}

pub(super) fn ensure_writeback_workflow_family(
    declaration: &QueryWorkflowDeclaration,
) -> Result<(), WorkflowLoweringError> {
    if declaration.request().declaration_family()
        == &WorkflowDeclarationFamily::WritebackLoweringNarrow
    {
        Ok(())
    } else {
        Err(WorkflowLoweringError::new(
            WorkflowLoweringFailureClass::UnsupportedWritebackFamily,
            "writeback lowering entrypoints may only lower admitted writeback workflow declarations",
            WorkflowStalenessClass::ExactBasisPreserved,
            lowering_denial_counters(0, LoweringDenialClass::WritebackDenied),
        ))
    }
}
