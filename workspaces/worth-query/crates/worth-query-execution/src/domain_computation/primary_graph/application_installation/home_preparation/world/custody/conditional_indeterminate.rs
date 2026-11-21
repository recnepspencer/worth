//! R9 `OperationIndeterminate` rows are classified before service, after World
//! readmitted every original and Query rejoined recovered originals to keys.
//!
//! Exactly three states remain: a recovered unpublished original joins the
//! row (a retry continues it); a published original finishes the row (a retry
//! returns it); no committed open, or a released original, finishes the row
//! as World's answer (a no-effect key may then execute fresh). Each row
//! resolves against the exact durable product branch occurrence it recorded;
//! an unresolved branch is refused rather than classified against root.
use super::originals::original_binding_denial;
use super::*;
use crate::domain_computation::primary_graph::application_attempt::business_original;
use crate::domain_computation::primary_graph::provider::WorthQueryConditionalKeyFate as Fate;
use crate::domain_computation::primary_graph::query_obligations::{
    QueryConditionalDecision, QueryConditionalDecisionRecord as Record,
    QueryConditionalOriginal as Original, QueryConditionalPosture as Posture,
    QueryManagedCleanupOriginal,
};
use QueryHomeCustodyReadmissionDenial as Denial;

impl<Schema: worth_query_installation::facade::ApplicationSchema + 'static, Program: 'static>
    ReadmittedHomeQueryService<Schema, Program>
{
    pub(super) fn settle_indeterminate_decisions(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let application = self.runtime.runtime();
        let Some(support) = application.installed_program_support() else {
            return Ok(());
        };
        let history = support.lifecycle();
        let rows = history
            .conditional_decisions()
            .into_iter()
            .filter_map(|row| match row.posture {
                Posture::Indeterminate(original) => Some((row, original)),
                Posture::CleanupRequired(_)
                | Posture::Retryable
                | Posture::CommitRetryable
                | Posture::ProductStale => None,
            })
            .collect::<Vec<_>>();
        if rows.is_empty() {
            return Ok(());
        }
        let installed = application
            .runtime
            .product_runtime
            .owner
            .observation_port()
            .observe_installed_product_branches(execution)
            .map_err(|_| Denial::OriginalBindingUnavailable)?;
        for (row, recorded) in rows {
            execution.consult().map_err(Denial::Execution)?;
            let occurrence = installed
                .branches()
                .iter()
                .find(|branch| {
                    branch.owner_identity().durable() == Some(row.product.owner())
                        && branch.lifecycle_incarnation().ordinal() == row.product.ordinal()
                })
                .ok_or(Denial::OriginalBindingUnavailable)?
                .lifecycle_incarnation();
            let product = application
                .runtime
                .product_runtime
                .admit_product_occurrence(occurrence)
                .map_err(|_| Denial::OriginalBindingUnavailable)?
                .publication_binding();
            let record = match self.classify(execution, &product, &row, recorded)? {
                Some(record) => record,
                None => continue,
            };
            history
                .record_conditional_decision(execution, record)
                .map_err(Denial::ProgramSupport)?;
        }
        Ok(())
    }

    /// The record that moves the row to its World state, if it must move.
    fn classify(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        product: &crate::domain_computation::execution_runtime::product_world::WorthQueryProductPublicationBinding,
        row: &QueryConditionalDecision,
        recorded: Original,
    ) -> Result<Option<Record>, Denial> {
        let provider = &self.runtime.runtime().runtime.primary_provider;
        let finish = Record::Finish {
            key: row.key,
            intent: row.intent,
        };
        match provider
            .conditional_key_fate(execution, product, row.key, row.intent)
            .map_err(original_binding_denial)?
        {
            // Published: the original occurrence answers every retry.
            Fate::Published(_) => Ok(Some(finish)),
            // No committed open (an empty one was released before this step):
            // proven no-effect, so the key may execute fresh.
            Fate::Absent => Ok(Some(finish)),
            // World released an original that kept its effect: World answers
            // the key from that released original, never by fresh execution.
            Fate::Released => Ok(Some(finish)),
            Fate::RecoveredUnpublished(handle) => {
                let effects = provider
                    .recovered_unpublished_application(product, handle)
                    .map_err(Denial::Recovery)?;
                let Some(QueryManagedCleanupOriginal::Attempt(attempt)) =
                    business_original::from_attempt(effects.attempt_identity())
                else {
                    return Err(Denial::ForeignOriginalAffinity);
                };
                match recorded {
                    Original::Unclassified => Ok(Some(Record::Retain(QueryConditionalDecision {
                        posture: Posture::Indeterminate(Original::Recovered(attempt)),
                        ..row.clone()
                    }))),
                    Original::Recovered(joined) if joined == attempt => Ok(None),
                    Original::Recovered(_) => Err(Denial::ConditionalOriginalMismatch),
                }
            }
            Fate::LiveUnpublishedOrPending => Err(Denial::UnclassifiedWorldCustody),
            Fate::OtherIntent => Err(Denial::ConditionalOriginalMismatch),
        }
    }
}
