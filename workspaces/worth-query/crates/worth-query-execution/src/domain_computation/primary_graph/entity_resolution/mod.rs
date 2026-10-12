//! Exact Relational truth used to resolve one application entity.

mod admitted_scope;
mod freshness;
#[cfg(test)]
pub(in crate::domain_computation::primary_graph) mod interruption_control;
mod product;
pub(in crate::domain_computation) use product::WorthQueryIssuedSelectedScope;
mod resolved;
#[cfg(test)]
mod tests;

pub use resolved::WorthQueryApplicationEntityIdentity;
use resolved::WorthQueryEntityResolutionSubject;
pub(in crate::domain_computation) use resolved::WorthQueryResolvedEntity;

use std::sync::Arc;

use worth_query_admission::facade::authenticated_principal::{
    WorthQueryRequestInterruption, WorthQueryRequestScope,
};
use worth_query_installation::facade::{
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationScalarValueBinding, ApplicationSchema,
    EqualityPredicate, WritePosture,
};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

use super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind, WorthQueryPrimaryGraph,
    WorthQueryPrincipalResolutionMode,
};

#[derive(Clone)]
pub(in crate::domain_computation) struct WorthQueryInstalledEntityResolutionContext {
    runtime_authority:
        crate::domain_computation::execution_runtime::WorthQueryRuntimeAuthorityIdentity,
    relational_runtime_instance_id: u64,
    binding_identity: worth_query_installation::facade::ApplicationSchemaBindingIdentity,
    layout: Arc<WorthQueryPrimaryGraphLayout>,
}

pub(in crate::domain_computation) struct WorthQueryEntityResolutionTruth<'truth> {
    installed: &'truth WorthQueryInstalledEntityResolutionContext,
    relational: &'truth worth_relational::facade::runtime::RelationalRuntime,
    snapshot: &'truth worth_relational::facade::snapshots::SnapshotHandle,
    mode: WorthQueryPrincipalResolutionMode,
}

impl WorthQueryPrimaryGraph {
    pub(in crate::domain_computation) fn retain_entity_resolution_context(
        &self,
    ) -> WorthQueryInstalledEntityResolutionContext {
        WorthQueryInstalledEntityResolutionContext {
            runtime_authority: self.runtime_authority(),
            relational_runtime_instance_id: self.relational_runtime_instance_id(),
            binding_identity: self.binding_identity().clone(),
            layout: Arc::clone(&self.layout),
        }
    }
}

impl WorthQueryInstalledEntityResolutionContext {
    pub(in crate::domain_computation) fn at_snapshot<'truth>(
        &'truth self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relational: &'truth worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &'truth worth_relational::facade::snapshots::SnapshotHandle,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryEntityResolutionTruth<'truth>, WorthQueryEntityResolutionDenial> {
        if snapshot.runtime_instance_id() != self.relational_runtime_instance_id
            || relational
                .read_truth()
                .project_snapshot(snapshot, execution)
                .is_none()
        {
            return Err(entity_denial(
                WorthQueryEntityResolutionDenialKind::ForeignResolutionTruth,
                "entity-resolution snapshot",
            ));
        }
        Ok(WorthQueryEntityResolutionTruth {
            installed: self,
            relational,
            snapshot,
            mode,
        })
    }
}

impl WorthQueryEntityResolutionTruth<'_> {
    pub(in crate::domain_computation) fn resolve(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryResolvedEntity, WorthQueryEntityResolutionDenial> {
        self.resolve_with_work(execution, entity, aspect, field, expected)
            .0
    }

    pub(in crate::domain_computation) fn resolve_with_work(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: &str,
        aspect: &str,
        field: &str,
        expected: worth_foundational::facade::AspectValue,
    ) -> (
        Result<WorthQueryResolvedEntity, WorthQueryEntityResolutionDenial>,
        usize,
    ) {
        let mut examined_candidate_count = 0;
        let result = (|| {
            let layout = self
                .installed
                .layout
                .equality_field(entity, aspect, field)
                .ok_or_else(|| {
                    entity_denial(
                        WorthQueryEntityResolutionDenialKind::FieldNotInstalled,
                        field,
                    )
                })?;
            let index_id = layout.equality_index_id.ok_or_else(|| {
                entity_denial(
                    WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
                    field,
                )
            })?;
            let request = BoundedEntityFieldLookupRequest::new(
                self.snapshot.clone(),
                index_id,
                layout.entity_kind,
                layout.locator.clone(),
                expected.clone(),
                2,
            )
            .map_err(|denial| map_index_denial(denial.kind(), field))?;
            #[cfg(test)]
            interruption_control::cancel_before_lookup_for_test();
            let lookup = self
                .relational
                .index_access()
                .execute_bounded_entity_field_lookup(request, parity(self.mode), execution)
                .map_err(|denial| {
                    examined_candidate_count = denial.examined_entry_count();
                    map_index_denial(denial.kind(), field)
                })?;
            examined_candidate_count = lookup.examined_entry_count();
            if lookup.overflowed() || lookup.candidate_entity_ids().len() > 1 {
                return Err(entity_denial(
                    WorthQueryEntityResolutionDenialKind::AmbiguousEntity,
                    field,
                ));
            }
            lookup.candidate_entity_ids().first().ok_or_else(|| {
                entity_denial(WorthQueryEntityResolutionDenialKind::UnknownEntity, field)
            })?;
            Ok(WorthQueryResolvedEntity::from_lookup(
                self.installed,
                layout,
                WorthQueryEntityResolutionSubject::new(entity, expected, self.mode),
                &lookup,
            ))
        })();
        (result, examined_candidate_count)
    }

    pub(in crate::domain_computation) fn validate_entity_freshness<Schema, Entity>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        freshness::validate(self, execution, identity)
    }
}

fn parity(mode: WorthQueryPrincipalResolutionMode) -> BoundedIndexParityMode {
    match mode {
        WorthQueryPrincipalResolutionMode::Ordinary => BoundedIndexParityMode::Production,
        WorthQueryPrincipalResolutionMode::Certification => BoundedIndexParityMode::Certification,
    }
}

fn admit_request(
    scope: &WorthQueryRequestScope,
    subject: &str,
) -> Result<(), WorthQueryEntityResolutionDenial> {
    match scope.interruption() {
        Some(WorthQueryRequestInterruption::Cancelled) => Err(entity_denial(
            WorthQueryEntityResolutionDenialKind::Cancelled,
            subject,
        )),
        Some(WorthQueryRequestInterruption::DeadlineExceeded) => Err(entity_denial(
            WorthQueryEntityResolutionDenialKind::DeadlineExceeded,
            subject,
        )),
        None => Ok(()),
    }
}

fn map_index_denial(
    kind: BoundedEntityFieldLookupDenialKind,
    subject: &str,
) -> WorthQueryEntityResolutionDenial {
    use worth_relational::facade::transactions::RelationalExecutionDenialCause as Cause;
    let kind = match kind {
        BoundedEntityFieldLookupDenialKind::Execution(Cause::Cancelled) => {
            WorthQueryEntityResolutionDenialKind::Cancelled
        }
        BoundedEntityFieldLookupDenialKind::Execution(Cause::DeadlineElapsed) => {
            WorthQueryEntityResolutionDenialKind::DeadlineExceeded
        }
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
            WorthQueryEntityResolutionDenialKind::CorruptIdentityIndex
        }
        BoundedEntityFieldLookupDenialKind::Execution(
            Cause::WorkerLimitExceedsParent
            | Cause::MemoryLimitExceedsParent
            | Cause::WorkLimitExceedsParent
            | Cause::PolicyMemoryExhausted { .. }
            | Cause::ProcessMemoryExhausted { .. }
            | Cause::DeclaredMemoryExhausted { .. }
            | Cause::ChargedBytesOverflow
            | Cause::UnrelatedNestedLease
            | Cause::NoActiveExecutionScope
            | Cause::EquivalenceContractUnavailable
            | Cause::WorkCounterOverflow
            | Cause::WorkExhausted
            | Cause::NestedStopped
            | Cause::ResultCapacityExceeded
            | Cause::WorkerFailed
            | Cause::ScratchCapacityExceeded
            | Cause::UncheckedCustomKernel
            | Cause::ExpectedIdentitiesNotCanonical
            | Cause::MemoryOverflow,
        )
        | BoundedEntityFieldLookupDenialKind::InvalidCandidateLimit
        | BoundedEntityFieldLookupDenialKind::SnapshotUnavailable
        | BoundedEntityFieldLookupDenialKind::IndexNotInstalled
        | BoundedEntityFieldLookupDenialKind::WrongIndexKind
        | BoundedEntityFieldLookupDenialKind::ExactGenerationUnavailable => {
            WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable
        }
    };
    entity_denial(kind, subject)
}

fn entity_denial(
    kind: WorthQueryEntityResolutionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryEntityResolutionDenial {
    WorthQueryEntityResolutionDenial::new(kind, subject)
}
