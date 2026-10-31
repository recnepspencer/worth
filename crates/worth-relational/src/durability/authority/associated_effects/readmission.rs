//! Read the exact discovered source outside owner locks and match actual state.
use super::*;

impl DurabilityAuthority<'_> {
    pub fn readmit_associated_effect(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        subject: &RelationalAssociatedEffectSubject,
    ) -> Result<ReadmittedRelationalAssociatedEffect, RelationalAssociatedEffectRefusal> {
        use RelationalAssociatedEffectRefusal as Refusal;
        crate::execution::check_request(execution)
            .map_err(|cause| Refusal::Recovery(RelationalRecoveryStop::Request(cause)))?;
        if subject.runtime_instance_id != self.runtime.runtime_instance_id() {
            return Err(Refusal::ForeignRuntime);
        }
        let _operation = self
            .runtime
            .owner_binding()
            .admit()
            .ok_or(Refusal::OwnerUnavailable)?;
        let residency = subject
            .residency
            .upgrade()
            .ok_or(Refusal::OwnerUnavailable)?;
        let source = subject.source.upgrade().ok_or(Refusal::OwnerUnavailable)?;
        let (entry, actual_source) = residency
            .journal_placement()
            .ok_or(Refusal::CatalogSubjectChanged)?;
        if entry.as_ref() != subject.entry.as_ref() || !Arc::ptr_eq(&actual_source, &source) {
            return Err(Refusal::CatalogSubjectChanged);
        }
        let current = self
            .runtime
            .history
            .commit_artifact(subject.entry.commit.commit_id);
        match current {
            Some(current)
                if Arc::ptr_eq(&current.envelope_residency(), &residency)
                    && current.receipt() == subject.entry.commit
                    && current.roots() == &subject.entry.root
                    && current.schema_version() == subject.entry.schema_version => {}
            None if self
                .runtime
                .history
                .performed_envelope_residency(subject.entry.commit.commit_id)
                .is_some_and(|actual| Arc::ptr_eq(&actual, &residency)) => {}
            _ => return Err(Refusal::CatalogSubjectChanged),
        }
        let positioned = RelationalCommitEnvelopeResidency::readmit_positioned(
            &subject.entry,
            source.as_ref(),
            execution,
            1 << 30,
        )
        .map_err(Refusal::Recovery)?;
        if positioned.envelope().caller_association() != subject.association()
            || subject.association().is_none()
        {
            return Err(Refusal::SubjectMismatch);
        }
        let settlement = self
            .runtime
            .history
            .associated_effect_settlement(&subject.entry, &residency)
            .ok_or(Refusal::MissingSettlementEvidence)?;
        let semantic = RecoveredRelationalSemanticResult {
            semantic: positioned.envelope().semantic_result_arc(),
        };
        Ok(ReadmittedRelationalAssociatedEffect {
            subject: subject.clone(),
            semantic,
            positioned,
            settlement,
        })
    }
}
