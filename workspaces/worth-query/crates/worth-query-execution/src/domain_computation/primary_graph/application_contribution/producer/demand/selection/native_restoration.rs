//! The actual Native correction supplies prior custody for a recreated owning scope.
use super::*;
impl<Schema: ApplicationSchema + 'static> WorthQueryPrimaryGraphApplicationRuntime<Schema> {
    pub(super) fn record_native_restored_output_prior<Family>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        source: &WorthQueryObservedSource<<<Family as WorthQueryProducerOutputFamily<Schema>>::Source as worth_query_declaration::facade::application_query::ApplicationQueryBinding<Schema>>::Query>,
        selected: &crate::domain_computation::primary_graph::WorthQuerySelectedProductOperation<
            '_,
            Schema,
        >,
        native: &mut crate::domain_computation::primary_graph::application_query::ApplicationQueryNativeAccess<'_>,
    ) -> Result<(), WorthQueryOutputDemandDenial>
    where
        Family: WorthQueryProducerOutputFamily<Schema>,
    {
        let foreign = || {
            WorthQueryOutputDemandDenial::new(
                WorthQueryOutputDemandDenialKind::ForeignSource,
                Family::IDENTITY,
            )
        };
        let entry = self
            .installed_producers
            .entries
            .values()
            .find(|entry| {
                entry.declaration.output_family_type
                    == crate::equality_type_id::EqualityTypeId::of::<Family>()
                    && entry.declaration.owned_scope_entity.is_some()
            })
            .ok_or_else(foreign)?;
        let publisher = self
            .runtime
            .primary_provider
            .graph
            .application_publication
            .as_ref()
            .ok_or_else(foreign)?;
        let graph = self.runtime.primary_graph().ok_or_else(foreign)?;
        let meaning = worth_relational::facade::mvcc::RequiredOutputPublicationMeaning {
            implementation:
                worth_relational::facade::mvcc::RequiredOutputProducingImplementation::new(
                    entry.edition.build_identity(),
                    entry.edition.digest(),
                ),
            family: Family::IDENTITY.to_owned(),
            source_scope: source.source_root(),
            source_partition: source.partition_identity(),
            source_epoch: source.checkpoint_identity().bytes(),
        };
        let restored = native.with_runtime_mut(graph, execution, foreign, |runtime| {
            runtime
                .readmit_restored_producer_prior(
                    execution,
                    &publisher.native,
                    selected.product().relational_basis(),
                    &meaning,
                )
                .map_err(native_denial)
        })?;
        if let Some(prior) = restored {
            self.runtime
                .primary_provider
                .graph
                .output_lineage
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .record_native_restored_prior(
                    execution,
                    self.runtime.authority_identity().as_u64(),
                    &self.installed_schema.binding_identity(),
                    selected.product().observation(),
                    prior,
                )
                .map_err(|cause| transport_denial(cause, source.source_root()))?;
        }
        Ok(())
    }
}
fn native_denial(
    error: worth_relational::facade::mvcc::TransactionCommitError,
) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::{
        WorthQueryApplicationCommitDenial, WorthQueryApplicationCommitDenialStage,
        WorthQueryApplicationCommitOutcome,
    };
    let denied = WorthQueryApplicationCommitDenial::invariant_execution_denied(WorthQueryApplicationCommitDenialStage::InvariantExecution,
        crate::domain_computation::primary_graph::provider::invariant_execution_failure::map_validation_failure(error));
    match super::super::super::execution::commit_receipt(
        "Native restored producer prior",
        WorthQueryApplicationCommitOutcome::Denied(denied),
    ) {
        Err(failure) => failure,
        Ok(_) => unreachable!("an original Native refusal cannot issue a producer receipt"),
    }
}
fn transport_denial(
    error: crate::domain_computation::primary_graph::application_checkpoint::CheckpointDecodeDenial,
    source: worth_relational::facade::identity::EntityId,
) -> WorthQueryOutputDemandDenial {
    use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphInstallationDenialKind as Kind;
    let error = error.into_installation_denial();
    let kind = match error.kind() {
        Kind::ExecutionDenied { kind } => WorthQueryOutputDemandDenialKind::SourceQueryExecution(
            crate::domain_computation::primary_graph::WorthQueryApplicationOneShotDenialKind::ExecutionRequest(kind)),
        Kind::ExecutionControlStopped { kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind::Cancelled } => WorthQueryOutputDemandDenialKind::Cancelled,
        Kind::ExecutionControlStopped { kind: crate::domain_computation::WorthQueryProviderSessionControlStopKind::TimedOut } => WorthQueryOutputDemandDenialKind::TimedOut,
        _ => WorthQueryOutputDemandDenialKind::RetainedBasisUnavailable,
    };
    WorthQueryOutputDemandDenial::new(
        kind,
        format!("Native restored producer prior {source:?}: {error:?}"),
    )
}
