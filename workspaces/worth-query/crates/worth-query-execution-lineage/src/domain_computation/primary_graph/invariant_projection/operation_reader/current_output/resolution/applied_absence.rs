//! Installed coverage and the actual acquired READ jointly prove applied absence.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::SourcePublicationRead;
impl<Schema: ApplicationSchema, Operation>
    WorthQueryApplicationOperationInvariantProjectionReader<'_, '_, Schema, Operation>
{
    pub(crate) fn read_applied_absence<Family: WorthQueryProducerOutputFamily<Schema>>(
        &mut self,
        scope: worth_relational::facade::identity::EntityId,
    ) -> Result<bool, WorthQueryCurrentOutputDenial> {
        let unavailable = || {
            WorthQueryCurrentOutputDenial::new(
                WorthQueryCurrentOutputDenialKind::OutputUnavailable,
                Family::IDENTITY,
            )
        };
        let Some(access) = self.source_query else {
            return Ok(false);
        };
        let Some((loan_source, loan_selected)) = access.applied_source_publication() else {
            return Ok(false);
        };
        let Some(candidate) = self.reader.native.candidate_identity().cloned() else {
            return Ok(false);
        };
        let address = loan_source.address();
        if address.runtime() != self.runtime_authority.as_u64()
            || address.schema() != self.binding_identity
            || address.family().as_str() != Family::IDENTITY
            || address.scope() != crate::domain_computation::authorization::WorthQueryOperationScopeEntityBinding::from_entity(scope)
            || Some(address.partition()) != self.reader.selected_source_partition_identity
            || Some(address.occurrence()) != self.reader.selected_product_occurrence
            || Some(loan_source.generation()) != self.reader.selected_product_generation
        { return Ok(false); }
        let principal = self.admitted_principal.ok_or_else(unavailable)?;
        if !loan_source
            .read_closure()
            .admits_applied_read(principal, scope, &candidate)
        {
            return Ok(false);
        }
        self.charge_current_output_work(1, Family::IDENTITY)?;
        let observation = {
            let lineage = self
                .reader
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let covered = lineage
                .covered_source_publication(loan_source)
                .map_err(|_| unavailable())?;
            std::sync::Arc::new(
                self.reader
                    .invalidation_owner
                    .observe_source_publication(&covered, loan_selected, None)
                    .map_err(|stop| match stop {
                        crate::domain_computation::primary_graph::output_lineage::invalidation::SettlementRegistrationStop::Admission(
                            worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause),
                        ) => WorthQueryCurrentOutputDenial::new(
                            WorthQueryCurrentOutputDenialKind::ExecutionRequest(cause),
                            Family::IDENTITY,
                        ),
                        _ => unavailable(),
                    })?,
            )
        };
        if !observation.admits_applied_absence(principal, scope, &candidate) {
            return Ok(false);
        }
        let admission = self.admission_identity.ok_or_else(unavailable)?;
        self.decision_facts
            .record_source_publication(
                SourcePublicationRead::new(admission, std::sync::Arc::clone(&observation)),
                observation,
            )
            .map_err(|stop| match stop {
                worth_relational::facade::mvcc::CompanionPreflightStop::Execution(cause) => {
                    WorthQueryCurrentOutputDenial::new(
                        WorthQueryCurrentOutputDenialKind::ExecutionRequest(cause),
                        Family::IDENTITY,
                    )
                }
                _ => unavailable(),
            })?;
        Ok(true)
    }
}
