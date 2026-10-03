//! Exact accepted-publication lookup. The reader's newer product coordinate
//! never substitutes for the coordinate carried by the accepted authority.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;

use crate::domain_computation::primary_graph::application_output_demand::{
    ReadyCompletion, WorthQueryAcceptedOutputAuthority,
};

use super::{
    input_cutoff::RetainedInputCutoffCandidate,
    invalidation::{FullVerificationReason, InvalidationEditAdmission},
    ProductCoordinate, RecordedSourceIdentity, SemanticSource, WorthQueryApplicationOutputLineage,
};

mod current_accepted;
pub(in crate::domain_computation::primary_graph) use current_accepted::{
    BoundCurrentAcceptedOutput, CurrentAcceptedResult, CurrentAcceptedStop,
};

/// Only exact accepted authority resolution can mint this candidate. A prior
/// partition lookup cannot substitute a newer row or certify current demand.
pub(in crate::domain_computation::primary_graph) struct AcceptedCurrentCandidate {
    selected: RetainedInputCutoffCandidate,
    ready: ReadyCompletion,
}

impl AcceptedCurrentCandidate {
    /// The actor mark is keyed by this exact retained recorded row. Its
    /// identity remains pinned by the accepted candidate during the read.
    pub(in crate::domain_computation::primary_graph) fn recorded_identity(
        &self,
    ) -> &super::RecordedSettlementIdentity {
        self.selected.settlement_identity().as_ref()
    }

    /// A candidate can certify only the exact Ready cell that selected it.
    pub(in crate::domain_computation::primary_graph) fn matches_ready_admitted(
        &self,
        ready: &ReadyCompletion,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<bool, worth_relational::facade::mvcc::CompanionPreflightStop> {
        admission.charge_external_work(1)?;
        Ok(self.ready.same_cell(ready))
    }
}

impl WorthQueryApplicationOutputLineage {
    pub(in crate::domain_computation::primary_graph) fn resolve_required_settlement(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        completion: &ReadyCompletion,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<AcceptedCurrentCandidate>, FullVerificationReason> {
        admission
            .charge_external_work(2)
            .map_err(FullVerificationReason::MarkingAdmissionDenied)?;
        let authority = &completion.authority;
        let (correspondence, scope, coordinate, source_identity, partition, dependency, key, pin) =
            match authority {
                WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                    if receipt.principal_scope().runtime_authority() != runtime_authority
                        || receipt.principal_scope().binding_identity() != schema
                    {
                        return Err(FullVerificationReason::ForeignSource);
                    }
                    let publication = receipt.committed_product_publication();
                    let idempotency = receipt.idempotency_binding();
                    let Some(source) = idempotency.source_identity() else {
                        return Err(FullVerificationReason::NativeRevisionUnavailable);
                    };
                    let Some(partition) = idempotency.source_partition_identity() else {
                        return Err(FullVerificationReason::UnsupportedFact);
                    };
                    (
                        receipt.output_correspondence(),
                        receipt.principal_scope().scope(),
                        ProductCoordinate {
                            occurrence: publication.product_incarnation(),
                            generation: publication.product_generation().get(),
                        },
                        RecordedSourceIdentity::Runtime(
                            super::super::application_query::WorthQueryRuntimeSourceIdentity::new(
                                source,
                            ),
                        ),
                        partition,
                        idempotency.producer_dependency_identity(),
                        *idempotency.key_identity(),
                        receipt.exact_output_settlement(),
                    )
                }
                WorthQueryAcceptedOutputAuthority::Stable(stable) => (
                    stable.output_correspondence().as_ref(),
                    stable.exact_settlement().source().scope,
                    ProductCoordinate {
                        occurrence: stable.observation().lifecycle_incarnation(),
                        generation: stable.observation().reference_generation().get(),
                    },
                    stable
                        .source_identity()
                        .ok_or(FullVerificationReason::NativeRevisionUnavailable)?,
                    stable
                        .source_partition_identity()
                        .ok_or(FullVerificationReason::NativeRevisionUnavailable)?,
                    stable.producer_dependency_identity(),
                    stable.idempotency_key_identity(),
                    Some(stable.exact_settlement()),
                ),
                WorthQueryAcceptedOutputAuthority::Restored(restored) => (
                    restored.correspondence.as_ref(),
                    restored.source_scope,
                    ProductCoordinate {
                        occurrence: restored.observation.lifecycle_incarnation(),
                        generation: restored.observation.reference_generation().get(),
                    },
                    RecordedSourceIdentity::Checkpoint(restored.source_identity),
                    restored.checkpoint.source_partition,
                    restored.checkpoint.producer_dependency,
                    restored.checkpoint.idempotency_key,
                    None,
                ),
            };
        let Some(output_binding) = correspondence.binding_type() else {
            return Ok(None);
        };
        let pin = pin.ok_or_else(|| match authority {
            WorthQueryAcceptedOutputAuthority::Committed(_) => {
                FullVerificationReason::NativeRevisionUnavailable
            }
            WorthQueryAcceptedOutputAuthority::Stable(_) => {
                FullVerificationReason::NativeRevisionUnavailable
            }
            WorthQueryAcceptedOutputAuthority::Restored(_) => {
                FullVerificationReason::CheckpointRestore
            }
        })?;
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope,
            output_binding,
        };
        if pin.source() != &source || pin.coordinate() != coordinate {
            return Err(FullVerificationReason::NativeRevisionUnavailable);
        }
        // The accepted coordinate chooses one row at each index level. Each
        // ordered descent is paid before the corresponding selected read.
        admission
            .charge_external_work(
                super::prepared_slot::tree_work::<SemanticSource>(self.by_source.len()).ok_or(
                    FullVerificationReason::MarkingAdmissionDenied(
                        worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                    ),
                )?,
            )
            .map_err(FullVerificationReason::MarkingAdmissionDenied)?;
        let Some(occurrences) = self.by_source.get(&source) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable);
        };
        admission
            .charge_external_work(
                super::prepared_slot::tree_work::<
                    worth_runtime_world::facade::ProductBranchIncarnation,
                >(occurrences.len())
                .ok_or(FullVerificationReason::MarkingAdmissionDenied(
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                ))?,
            )
            .map_err(FullVerificationReason::MarkingAdmissionDenied)?;
        let Some(history) = occurrences.get(&coordinate.occurrence) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable);
        };
        admission
            .charge_external_work(super::prepared_slot::tree_work::<u64>(history.len()).ok_or(
                FullVerificationReason::MarkingAdmissionDenied(
                    worth_relational::facade::mvcc::CompanionPreflightStop::WorkCounterOverflow,
                ),
            )?)
            .map_err(FullVerificationReason::MarkingAdmissionDenied)?;
        let Some(records) = history.get(&coordinate.generation) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable);
        };
        admission
            .charge_external_work(12)
            .map_err(FullVerificationReason::MarkingAdmissionDenied)?;
        let Some(cell) = records.get(pin.slot()) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable);
        };
        let Some(recorded) = cell.get() else {
            return Err(FullVerificationReason::NativeRevisionUnavailable);
        };
        if !Arc::ptr_eq(&recorded.settlement_identity, pin)
            || !std::ptr::eq(recorded.correspondence.as_ref(), correspondence)
            || recorded.source_partition_identity != Some(partition)
            || recorded.source_identity != Some(source_identity)
            || recorded.producer_dependency_identity != dependency
            || recorded.idempotency_key_identity != key
            || recorded.correspondence.binding_type() != Some(output_binding)
        {
            return Err(FullVerificationReason::NativeRevisionUnavailable);
        }
        Ok(Some(AcceptedCurrentCandidate {
            selected: RetainedInputCutoffCandidate::from_exact_cell(Arc::clone(cell)),
            ready: completion.clone(),
        }))
    }
}
