//! The original owner shares Query's one publication driver and acknowledgment rule.
use super::*;
use journal::*;
use worth_execution::ExecutionRequest;
use worth_foundational::facade::ProgramSupportObligationIdentity;
impl QueryObligationHistory {
    pub(in crate::domain_computation::primary_graph) fn retain_original_operation(
        &self,
        request: &ExecutionRequest<'_, '_>,
        data: QueryOriginalOperationData,
    ) -> Result<
        (
            ProgramSupportObligationIdentity,
            Option<QueryObligationDenial>,
        ),
        QueryObligationDenial,
    > {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let identity = {
            let mut state = self.state();
            let journal = state
                .journal
                .as_mut()
                .ok_or(QueryObligationDenial::SupportUnavailable)?;
            let position = journal
                .ledger
                .position()
                .checked_add(1)
                .and_then(std::num::NonZeroU64::new)
                .ok_or(QueryObligationDenial::IdentityExhausted)?;
            let identity = ProgramSupportObligationIdentity::from_recorded(medium.home(), position);
            journal.prepare_pending(QueryOriginalOperationRecord::Retain(
                QueryOriginalOperationObligation {
                    identity,
                    data,
                    position: QueryOriginalOperationPosition::Initial,
                },
            ))?;
            identity
        };
        let pending = driver.finish_pending(medium.as_ref(), request).err();
        Ok((identity, pending))
    }
    pub(in crate::domain_computation::primary_graph) fn original_operation(
        &self,
        request: &ExecutionRequest<'_, '_>,
        identity: ProgramSupportObligationIdentity,
    ) -> Result<QueryOriginalOperationObligation, QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        self.lock_publication()?
            .finish_pending(medium.as_ref(), request)?;
        if identity.home() != medium.home() {
            return Err(QueryObligationDenial::ForeignHome);
        }
        self.state()
            .journal
            .as_ref()
            .and_then(|journal| journal.ledger.original_operations.get(&identity))
            .cloned()
            .ok_or(QueryObligationDenial::UnknownObligation)
    }
    pub(in crate::domain_computation::primary_graph) fn progress_original_operation(
        &self,
        request: &ExecutionRequest<'_, '_>,
        record: QueryOriginalOperationRecord,
    ) -> Result<(), QueryObligationDenial> {
        let medium = self
            .medium
            .get()
            .ok_or(QueryObligationDenial::SupportUnavailable)?;
        let driver = self.lock_publication()?;
        driver.finish_pending(medium.as_ref(), request)?;
        let current = {
            let state = self.state();
            let journal = state
                .journal
                .as_ref()
                .ok_or(QueryObligationDenial::SupportUnavailable)?;
            match &record {
                QueryOriginalOperationRecord::Advance { identity, position } => journal
                    .ledger
                    .original_operations
                    .get(identity)
                    .is_some_and(|row| row.position == *position),
                QueryOriginalOperationRecord::Discharge { identity } => {
                    !journal.ledger.original_operations.contains_key(identity)
                }
                QueryOriginalOperationRecord::Retain(_) => false,
            }
        };
        if current {
            return Ok(());
        }
        driver.publish(medium.as_ref(), record, request)
    }
}
