use crate::application::{
    WorthQueryDeclarationEntryOrchestrationChecked, WorthQueryDeclarationEntryOrchestrationProof,
    WorthQueryDeclarationInput, WorthQueryDomainEntryMarker, WorthQueryDomainOperatingContext,
};
use crate::continuation_pipeline::{
    WorthQueryContinuationExecutionChecked, WorthQueryContinuationExecutionTranscript,
    WorthQueryPreparedContinuationChecked, WorthQueryPreparedContinuationTranscript,
};
use crate::contribution_composed_orchestration::{
    WorthQueryContributionComposedOrchestrationChecked,
    WorthQueryContributionComposedOrchestrationTranscript,
};
use crate::evidence_identity::WorthQueryEvidenceIdentityDerivationDenial as IdentityDerivationDenial;
use crate::grouped_authoring::{
    WorthQueryGroupedOrchestrationChecked, WorthQueryGroupedOrchestrationTranscript,
};
use crate::ordinary_outcome::WorthQueryOrdinaryOutcome;
use crate::recovery_boundary::{
    worth_query_recovery_brief_from_continuation_execution_checked,
    worth_query_recovery_brief_from_continuation_execution_proof,
    worth_query_recovery_brief_from_contribution_composed_checked,
    worth_query_recovery_brief_from_contribution_composed_proof,
    worth_query_recovery_brief_from_declaration_entry_checked,
    worth_query_recovery_brief_from_declaration_entry_proof,
    worth_query_recovery_brief_from_declaration_receipt_checked,
    worth_query_recovery_brief_from_declaration_route_plan_checked,
    worth_query_recovery_brief_from_grouped_orchestration_checked,
    worth_query_recovery_brief_from_grouped_orchestration_proof,
    worth_query_recovery_brief_from_ordinary_outcome,
    worth_query_recovery_brief_from_prepared_continuation_checked,
    worth_query_recovery_brief_from_prepared_continuation_proof,
    worth_query_recovery_brief_from_signal_compatibility_checked,
    worth_query_recovery_brief_from_signal_compatibility_proof, WorthQueryRecoveryBrief,
};
use crate::signal_compatibility_orchestration::{
    WorthQuerySignalCompatibilityOrchestrationChecked,
    WorthQuerySignalCompatibilityOrchestrationTranscript,
};

use super::WorthQueryInstalledDomainDeclarationContext;

impl<D: WorthQueryDomainEntryMarker, C: WorthQueryDomainOperatingContext<D>>
    WorthQueryInstalledDomainDeclarationContext<D, C>
{
    pub fn recover_from_outcome<T>(
        &self,
        outcome: &WorthQueryOrdinaryOutcome<T>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_ordinary_outcome(outcome)
    }

    pub fn recover_from_declaration_entry_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: WorthQueryDeclarationEntryOrchestrationChecked<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_declaration_entry_checked(checked)
    }

    pub fn recover_from_declaration_entry_proof<I: WorthQueryDeclarationInput<D>>(
        &self,
        proof: &WorthQueryDeclarationEntryOrchestrationProof<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_declaration_entry_proof(proof)
    }

    pub fn recover_from_declaration_route_plan_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: crate::application::WorthQueryDeclarationRoutePlanChecked<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_declaration_route_plan_checked(checked)
    }

    pub fn recover_from_declaration_receipt_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: crate::application::WorthQueryDeclarationReceiptChecked<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_declaration_receipt_checked(checked)
    }

    pub fn recover_from_prepared_continuation_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: WorthQueryPreparedContinuationChecked<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_prepared_continuation_checked(checked)
    }

    pub fn recover_from_prepared_continuation_proof<I: WorthQueryDeclarationInput<D>>(
        &self,
        proof: WorthQueryPreparedContinuationTranscript<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_prepared_continuation_proof(proof)
    }

    pub fn recover_from_continuation_execution_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: WorthQueryContinuationExecutionChecked<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_continuation_execution_checked(checked)
    }

    pub fn recover_from_continuation_execution_proof<I: WorthQueryDeclarationInput<D>>(
        &self,
        proof: WorthQueryContinuationExecutionTranscript<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_continuation_execution_proof(proof)
    }

    pub fn recover_from_signal_compatibility_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: WorthQuerySignalCompatibilityOrchestrationChecked<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_signal_compatibility_checked(checked)
    }

    pub fn recover_from_signal_compatibility_proof<I: WorthQueryDeclarationInput<D>>(
        &self,
        proof: WorthQuerySignalCompatibilityOrchestrationTranscript<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_signal_compatibility_proof(proof)
    }

    pub fn recover_from_contribution_composed_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: WorthQueryContributionComposedOrchestrationChecked<D, I>,
    ) -> Result<Option<WorthQueryRecoveryBrief>, IdentityDerivationDenial> {
        worth_query_recovery_brief_from_contribution_composed_checked(checked)
    }

    pub fn recover_from_contribution_composed_proof<I: WorthQueryDeclarationInput<D>>(
        &self,
        proof: WorthQueryContributionComposedOrchestrationTranscript<D, I>,
    ) -> Result<Option<WorthQueryRecoveryBrief>, IdentityDerivationDenial> {
        worth_query_recovery_brief_from_contribution_composed_proof(proof)
    }

    pub fn recover_from_grouped_orchestration_checked<I: WorthQueryDeclarationInput<D>>(
        &self,
        checked: WorthQueryGroupedOrchestrationChecked<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_grouped_orchestration_checked(checked)
    }

    pub fn recover_from_grouped_orchestration_proof<I: WorthQueryDeclarationInput<D>>(
        &self,
        proof: WorthQueryGroupedOrchestrationTranscript<D, I>,
    ) -> Option<WorthQueryRecoveryBrief> {
        worth_query_recovery_brief_from_grouped_orchestration_proof(proof)
    }
}
