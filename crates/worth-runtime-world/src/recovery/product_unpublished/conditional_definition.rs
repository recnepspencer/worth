use super::*;
use crate::publication::{
    ConditionalDefinitionAttemptCustody, SettledRelationalPublicationAdoption,
    SignalAttemptProgress,
};
use crate::recovery::RuntimeWorldRecoveryDenial as Denial;

impl ProductUnpublishedOwnerEffectsRecord {
    pub(crate) fn conditional_recovery_parts(
        &self,
    ) -> Result<
        (
            ProductBranchObservation,
            Option<SettledRelationalPublicationAdoption>,
            Option<SignalAttemptProgress>,
            Arc<ConditionalDefinitionAttemptCustody>,
        ),
        Denial,
    > {
        if self.progress.relational_requires_settlement() {
            return Err(Denial::SettlementRequired);
        }
        let custody = self
            .retention
            .conditional_definition
            .as_ref()
            .cloned()
            .ok_or(Denial::SettlementEvidenceUnavailable)?;
        let relational = match self.component_results.relational_posture() {
            crate::history::CompositeComponentChangePosture::RetainExact => None,
            crate::history::CompositeComponentChangePosture::Published => {
                let (identity, basis, result) = self
                    .component_results
                    .settled_relational_parts()
                    .ok_or(Denial::SettlementEvidenceUnavailable)?;
                Some(SettledRelationalPublicationAdoption::new(
                    identity, basis, result,
                ))
            }
        };
        let signal = match self.progress.signal_posture() {
            crate::publication::SignalAttemptProgressPosture::Untouched => None,
            crate::publication::SignalAttemptProgressPosture::Performed
                if self.component_results.signal().advanced_outcome().is_some() =>
            {
                self.component_results.signal().settled_advance()
            }
            _ => return Err(Denial::SettlementEvidenceUnavailable),
        };
        if !custody.can_resume(signal.is_some()) {
            return Err(Denial::SettlementEvidenceUnavailable);
        }
        Ok((self.expected_head.clone(), relational, signal, custody))
    }

    pub(crate) fn take_conditional_definition(
        &mut self,
    ) -> Option<Arc<ConditionalDefinitionAttemptCustody>> {
        self.retention.conditional_definition.take()
    }

    pub(crate) fn restore_conditional_definition(
        &mut self,
        custody: Arc<ConditionalDefinitionAttemptCustody>,
    ) {
        assert!(self.retention.conditional_definition.is_none());
        self.retention.conditional_definition = Some(custody);
    }
}
