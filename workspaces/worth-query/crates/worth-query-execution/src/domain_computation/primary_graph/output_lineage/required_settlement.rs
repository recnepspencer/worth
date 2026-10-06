//! Exact accepted-publication lookup. The reader's newer product coordinate
//! never substitutes for the coordinate carried by the accepted authority.

use std::sync::Arc;

use worth_query_installation::facade::ApplicationSchemaBindingIdentity;
use worth_relational::facade::mvcc::CompanionPreflightStop;

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
#[cfg(test)]
mod tests;

/// What ends an exact accepted-row lookup, before any effect. Each one denies
/// the request; a reason only leaves the row uncertified.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(in crate::domain_computation::primary_graph) enum RequiredSettlementStop {
    Admission(CompanionPreflightStop),
    /// The accepted authority belongs to another runtime or schema.
    Foreign,
}

/// The lookup's two early ends, which `resolve_required_settlement` keeps
/// apart.
enum Unresolved {
    Stop(RequiredSettlementStop),
    Reason(FullVerificationReason),
}

impl From<CompanionPreflightStop> for Unresolved {
    fn from(stop: CompanionPreflightStop) -> Self {
        Self::Stop(RequiredSettlementStop::Admission(stop))
    }
}

impl From<FullVerificationReason> for Unresolved {
    fn from(reason: FullVerificationReason) -> Self {
        Self::Reason(reason)
    }
}

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
    /// The exact accepted row for `completion`. A stop is the outer error. A
    /// reason the row cannot be certified exactly is the inner one, and the
    /// reader then verifies the output in full.
    #[allow(clippy::type_complexity)]
    pub(in crate::domain_computation::primary_graph) fn resolve_required_settlement(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        completion: &ReadyCompletion,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<
        Result<Option<AcceptedCurrentCandidate>, FullVerificationReason>,
        RequiredSettlementStop,
    > {
        match self.exact_required_settlement(runtime_authority, schema, completion, admission) {
            Ok(candidate) => Ok(Ok(candidate)),
            Err(Unresolved::Reason(reason)) => Ok(Err(reason)),
            Err(Unresolved::Stop(stop)) => Err(stop),
        }
    }

    fn exact_required_settlement(
        &self,
        runtime_authority: u64,
        schema: &ApplicationSchemaBindingIdentity,
        completion: &ReadyCompletion,
        admission: &mut InvalidationEditAdmission,
    ) -> Result<Option<AcceptedCurrentCandidate>, Unresolved> {
        admission.charge_external_work(2)?;
        let authority = &completion.authority;
        let (correspondence, scope, coordinate, source_identity, partition, dependency, key, pin) =
            match authority {
                WorthQueryAcceptedOutputAuthority::Committed(receipt) => {
                    if receipt.principal_scope().runtime_authority() != runtime_authority
                        || receipt.principal_scope().binding_identity() != schema
                    {
                        return Err(Unresolved::Stop(RequiredSettlementStop::Foreign));
                    }
                    let publication = receipt.committed_product_publication();
                    let idempotency = receipt.idempotency_binding();
                    let Some(source) = idempotency.source_identity() else {
                        return Err(FullVerificationReason::NativeRevisionUnavailable.into());
                    };
                    let Some(partition) = idempotency.source_partition_identity() else {
                        return Err(FullVerificationReason::UnsupportedFact.into());
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
        let source = SemanticSource {
            runtime_authority,
            schema: schema.clone(),
            scope,
            output_binding,
        };
        let pin = match (pin, authority) {
            (Some(pin), _) => pin,
            // A restored authority carries no settlement pin. Its recorded row
            // is the one restoration placed at this exact partition address.
            (None, WorthQueryAcceptedOutputAuthority::Restored(_)) => self
                .partition_index
                .at_generation(
                    &source,
                    coordinate.occurrence,
                    coordinate.generation,
                    partition,
                )
                .and_then(|slot| {
                    self.by_source
                        .get(&source)?
                        .get(&coordinate.occurrence)?
                        .get(&coordinate.generation)?
                        .get(slot)?
                        .get()
                })
                .map(|recorded| &recorded.settlement_identity)
                .ok_or(FullVerificationReason::CheckpointRestore)?,
            (None, _) => return Err(FullVerificationReason::NativeRevisionUnavailable.into()),
        };
        if pin.source() != &source || pin.coordinate() != coordinate {
            return Err(FullVerificationReason::NativeRevisionUnavailable.into());
        }
        // The accepted coordinate chooses one row at each index level. Each
        // ordered descent is paid before the corresponding selected read.
        admission.charge_external_work(
            super::prepared_slot::tree_work::<SemanticSource>(self.by_source.len())
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        let Some(occurrences) = self.by_source.get(&source) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable.into());
        };
        admission
            .charge_external_work(
                super::prepared_slot::tree_work::<
                    worth_runtime_world::facade::ProductBranchIncarnation,
                >(occurrences.len())
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
            )?;
        let Some(history) = occurrences.get(&coordinate.occurrence) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable.into());
        };
        admission.charge_external_work(
            super::prepared_slot::tree_work::<u64>(history.len())
                .ok_or(CompanionPreflightStop::WorkCounterOverflow)?,
        )?;
        let Some(records) = history.get(&coordinate.generation) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable.into());
        };
        admission.charge_external_work(12)?;
        let Some(cell) = records.get(pin.slot()) else {
            return Err(FullVerificationReason::NativeRevisionUnavailable.into());
        };
        let Some(recorded) = cell.get() else {
            return Err(FullVerificationReason::NativeRevisionUnavailable.into());
        };
        if !Arc::ptr_eq(&recorded.settlement_identity, pin)
            || !std::ptr::eq(recorded.correspondence.as_ref(), correspondence)
            || recorded.source_partition_identity != Some(partition)
            || recorded.source_identity != Some(source_identity)
            || recorded.producer_dependency_identity != dependency
            || recorded.idempotency_key_identity != key
            || recorded.correspondence.binding_type() != Some(output_binding)
        {
            return Err(FullVerificationReason::NativeRevisionUnavailable.into());
        }
        Ok(Some(AcceptedCurrentCandidate {
            selected: RetainedInputCutoffCandidate::from_exact_cell(Arc::clone(cell)),
            ready: completion.clone(),
        }))
    }
}
