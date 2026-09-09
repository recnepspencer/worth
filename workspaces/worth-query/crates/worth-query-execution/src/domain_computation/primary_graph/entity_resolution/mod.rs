//! Exact Relational truth used to resolve one application entity.

mod freshness;
mod product;
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
    ApplicationFieldRef, ApplicationFieldUnit, ApplicationSchema, EqualityPredicate,
    TypedApplicationValue, WritePosture,
};
use worth_relational::facade::indexes::{
    BoundedEntityFieldLookupDenialKind, BoundedEntityFieldLookupRequest, BoundedIndexParityMode,
};

use super::schema_layout::WorthQueryPrimaryGraphLayout;
use super::{
    WorthQueryEntityResolutionDenial, WorthQueryEntityResolutionDenialKind, WorthQueryPrimaryGraph,
    WorthQueryPrimaryGraphApplicationRuntime, WorthQueryPrincipalResolutionMode,
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
        relational: &'truth worth_relational::facade::runtime::RelationalRuntime,
        snapshot: &'truth worth_relational::facade::snapshots::SnapshotHandle,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryEntityResolutionTruth<'truth>, WorthQueryEntityResolutionDenial> {
        if snapshot.runtime_instance_id() != self.relational_runtime_instance_id
            || relational.read_truth().project_snapshot(snapshot).is_none()
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
        entity: &str,
        aspect: &str,
        field: &str,
        expected: worth_foundational::facade::AspectValue,
    ) -> Result<WorthQueryResolvedEntity, WorthQueryEntityResolutionDenial> {
        self.resolve_with_work(entity, aspect, field, expected).0
    }

    pub(in crate::domain_computation) fn resolve_with_work(
        &self,
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
            let lookup = self
                .relational
                .index_access()
                .execute_bounded_entity_field_lookup(request, parity(self.mode))
                .map_err(|denial| map_index_denial(denial.kind(), field))?;
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
        identity: &WorthQueryApplicationEntityIdentity<Schema, Entity>,
    ) -> Result<(), WorthQueryEntityResolutionDenial> {
        freshness::validate(self, identity)
    }
}

impl<Schema> WorthQueryPrimaryGraphApplicationRuntime<Schema>
where
    Schema: ApplicationSchema,
{
    pub fn resolve_entity<Aspect, Entity, Field, Value, Write, Unit>(
        &self,
        field: ApplicationFieldRef<
            Schema,
            Entity,
            Aspect,
            Field,
            Value,
            Write,
            EqualityPredicate,
            Unit,
        >,
        value: Value,
        scope: &WorthQueryRequestScope,
        mode: WorthQueryPrincipalResolutionMode,
    ) -> Result<WorthQueryApplicationEntityIdentity<Schema, Entity>, WorthQueryEntityResolutionDenial>
    where
        Value: TypedApplicationValue,
        Write: WritePosture,
        Unit: ApplicationFieldUnit,
    {
        admit_request(scope, field.field())?;
        let graph = self.runtime.primary_graph().ok_or_else(|| {
            entity_denial(
                WorthQueryEntityResolutionDenialKind::PrimaryGraphNotInstalled,
                field.entity(),
            )
        })?;
        let installed = graph.retain_entity_resolution_context();
        let result = graph.integration_handle().with_runtime_mut(|relational| {
            let snapshot = super::exact_basis_access::open_current_main_snapshot(relational)
                .map_err(|basis_denial| {
                    let kind = match basis_denial {
                        super::WorthQueryExactBasisSnapshotDenial::ActiveSnapshotCapacityExhausted {
                            maximum_active_snapshots,
                        } => WorthQueryEntityResolutionDenialKind::ActiveSnapshotCapacityExhausted {
                            maximum_active_snapshots,
                        },
                        super::WorthQueryExactBasisSnapshotDenial::RetentionCapacityExhausted => {
                            WorthQueryEntityResolutionDenialKind::RetentionCapacityExhausted
                        }
                        super::WorthQueryExactBasisSnapshotDenial::RetentionIdentityExhausted => {
                            WorthQueryEntityResolutionDenialKind::RetentionIdentityExhausted
                        }
                        super::WorthQueryExactBasisSnapshotDenial::SnapshotIdentityExhausted => {
                            WorthQueryEntityResolutionDenialKind::SnapshotIdentityExhausted
                        }
                        _ => WorthQueryEntityResolutionDenialKind::ForeignResolutionTruth,
                    };
                    entity_denial(kind, field.field())
                })?;
            let result = installed
                .at_snapshot(relational, &snapshot, mode)
                .and_then(|truth| {
                    truth.resolve(
                        field.entity(),
                        field.aspect(),
                        field.field(),
                        value.into_foundational_value(),
                    )
                });
            crate::relational_snapshot_release::release_query_snapshot(relational, &snapshot);
            result
        })?;
        admit_request(scope, field.field())?;
        Ok(result.into_application_identity())
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
    let kind = match kind {
        BoundedEntityFieldLookupDenialKind::CorruptIndexEntries
        | BoundedEntityFieldLookupDenialKind::StorageParityMismatch => {
            WorthQueryEntityResolutionDenialKind::CorruptIdentityIndex
        }
        _ => WorthQueryEntityResolutionDenialKind::EqualityIndexUnavailable,
    };
    entity_denial(kind, subject)
}

fn entity_denial(
    kind: WorthQueryEntityResolutionDenialKind,
    subject: impl Into<String>,
) -> WorthQueryEntityResolutionDenial {
    WorthQueryEntityResolutionDenial::new(kind, subject)
}
