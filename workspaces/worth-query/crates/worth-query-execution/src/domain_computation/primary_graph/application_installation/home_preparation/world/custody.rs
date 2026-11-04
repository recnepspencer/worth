//! Actual cold World custody and original Query keys admit the service phase.
use super::*;
use worth_proof::CustodyReadmitted;

#[derive(Debug)]
pub enum QueryHomeCustodyReadmissionDenial {
    Execution(worth_execution::WorkCeilingDenial),
    World(RuntimeWorldServiceDenial<RuntimeWorldRecoveryDenial>),
    Recovery(RuntimeWorldRecoveryDenial),
    UnclassifiedWorldCustody,
    OriginalBindingUnavailable,
    ForeignOriginalAffinity,
    OriginalMemoryAdmission,
}
pub struct QueryHomeCustodyReadmissionRefused<Schema, Program> {
    query: ReadmittedHomeQueryService<Schema, Program>,
    cause: QueryHomeCustodyReadmissionDenial,
}
impl<Schema, Program> QueryHomeCustodyReadmissionRefused<Schema, Program> {
    pub fn into_parts(
        self,
    ) -> (
        ReadmittedHomeQueryService<Schema, Program>,
        QueryHomeCustodyReadmissionDenial,
    ) {
        (self.query, self.cause)
    }
}
impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub fn readmit_custody(
        self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<CustodyReadmitted<Self>, QueryHomeCustodyReadmissionRefused<Schema, Program>> {
        if let Err(cause) = self.admit_original_custody(execution) {
            return Err(QueryHomeCustodyReadmissionRefused { query: self, cause });
        }
        Ok(CustodyReadmitted::from_owner(self))
    }
    fn admit_original_custody(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), QueryHomeCustodyReadmissionDenial> {
        use QueryHomeCustodyReadmissionDenial as Denial;
        let application = self.runtime.runtime();
        let owner = &application.product_runtime.owner;
        let snapshot = owner
            .inspection_port()
            .recovery_snapshot()
            .map_err(|cause| Denial::World(RuntimeWorldServiceDenial::OwnerUnavailable(cause)))?;
        if snapshot.reserved() != 0 || snapshot.abandoned() != 0 || snapshot.updating() != 0 {
            return Err(Denial::UnclassifiedWorldCustody);
        }
        let recovery = owner.recovery_port();
        let mut after = None;
        loop {
            execution.consult().map_err(Denial::Execution)?;
            let page = owner
                .inspection_port()
                .recovery_page(
                    after.as_ref(),
                    std::num::NonZeroUsize::new(32).expect("nonzero page"),
                )
                .map_err(Denial::World)?;
            for row in page.rows() {
                match row.state() {
                    RuntimeWorldRecoveryRecordState::Recovered => {}
                    RuntimeWorldRecoveryRecordState::Active
                    | RuntimeWorldRecoveryRecordState::Abandoned
                    | RuntimeWorldRecoveryRecordState::Retained
                    | RuntimeWorldRecoveryRecordState::Busy => {
                        return Err(Denial::UnclassifiedWorldCustody)
                    }
                }
                let effects = recovery
                    .inspect_recovered_effects(row.handle())
                    .map_err(Denial::Recovery)?;
                application.primary_provider.admit_recovered_idempotency(execution, row.handle().clone(), &effects).map_err(|cause| {
                    use crate::domain_computation::primary_graph::provider::WorthQueryProviderIdempotencyResolutionDenial as Source;
                    match cause {
                        Source::ForeignAffinity => Denial::ForeignOriginalAffinity,
                        Source::RetentionCapacityExhausted | Source::RetentionIdentityExhausted => Denial::OriginalMemoryAdmission,
                        Source::PreparationBusy | Source::ActiveSnapshotCapacityExhausted { .. } | Source::SnapshotIdentityExhausted | Source::RecordedIntentUnverifiable | Source::ExecutionDenied(_) | Source::Unavailable => Denial::OriginalBindingUnavailable,
                    }
                })?;
            }
            after = page.next_after().cloned();
            if after.is_none() {
                return Ok(());
            }
        }
    }
}
