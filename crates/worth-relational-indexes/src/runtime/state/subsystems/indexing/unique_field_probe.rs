//! Exact-head equality probes for an installed entity-kind-scoped index.
use super::{IndexDefinitionCatalog, IndexDefinitionReadBinding};
use crate::branch::indexes::data::bounded_entity_field_lookup::BoundedEntityFieldLookupDenialOwnerPort as _;
use crate::branch::indexes::data::{
    BoundedEntityFieldLookupDenial as Error, BoundedEntityFieldLookupDenialKind as Denial,
    DerivedIndexEntries,
};
use crate::contracts::{
    execution::{denial_cause::RelationalExecutionDenialCause as Cause, request_stage},
    history::data::coordinates::BranchId,
    identity::data::{EntityId, KindId},
    indexes::data::{
        definition::{DerivedIndexId, DerivedIndexKind},
        maintenance::SelectedIndexGenerationAdmissionStop,
    },
    storage::data::authoritative_field_comparison_key::AuthoritativeFieldComparisonKey,
};
use std::collections::BTreeSet;
use worth_foundational::facade::AspectFieldLocator;
use worth_relational_branch::internal_port::branch::root::RelationalBranchRoot;

impl IndexDefinitionCatalog {
    pub fn has_unique_fields_since(&self, frontier: u64) -> bool {
        self.definitions
            .range(DerivedIndexId(frontier)..)
            .any(|(_, definition)| definition.unique_entity_kind.is_some())
    }

    /// Installed scoped indexes carry the native uniqueness declaration.
    pub fn unique_entity_fields(
        &self,
    ) -> impl Iterator<Item = (KindId, DerivedIndexId, &AspectFieldLocator)> {
        self.definitions.values().filter_map(|definition| {
            let kind = definition.unique_entity_kind?;
            let DerivedIndexKind::EntityField { field_locator } = &definition.kind else {
                return None;
            };
            Some((kind, definition.index_id, field_locator))
        })
    }
}

impl IndexDefinitionReadBinding {
    /// The exact generation is selected without reconstruction. Inspect at
    /// most one surviving row plus the candidate's excluded retiring rows.
    pub fn unique_field_has_survivor(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &BranchId,
        root: &RelationalBranchRoot,
        scope: (KindId, DerivedIndexId, &AspectFieldLocator),
        key: &AuthoritativeFieldComparisonKey,
        writer: EntityId,
        retiring: &BTreeSet<EntityId>,
    ) -> Result<bool, Error> {
        let deny = |cause| Error::new(cause, scope.1);
        let (result, _) = request
            .run_native_with_checkpoint(|context| {
                request_stage::checkpoint(context, 1)
                    .map_err(|stop| deny(Denial::Execution(stop.into())))?;
                let Some(generation) = self.pin_unique_field_index(context, branch, root, scope)?
                else {
                    return Ok(false);
                };
                let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
                    return Err(deny(Denial::WrongIndexKind));
                };
                let depth = u64::from(usize::BITS - entries.len().leading_zeros()) * 11 + 1;
                request_stage::checkpoint(
                    context,
                    depth.saturating_mul(key.canonical_value_bytes().len() as u64 + 1),
                )
                .map_err(|stop| deny(Denial::Execution(stop.into())))?;
                let Some(rows) = entries.get(key) else {
                    return Ok(false);
                };
                let exclusion_work =
                    u64::from(usize::BITS - retiring.len().leading_zeros()) * 11 + 1;
                for holder in rows {
                    request_stage::checkpoint(context, exclusion_work)
                        .map_err(|stop| deny(Denial::Execution(stop.into())))?;
                    if *holder != writer && !retiring.contains(holder) {
                        return Ok(true);
                    }
                }
                Ok(false)
            })
            .map_err(|stop| deny(Denial::Execution(stop.into())))?;
        result
    }

    /// Explicit diagnostic lane. Its population walk is over index keys only.
    pub fn unique_field_audit(
        &self,
        request: &worth_execution::ExecutionRequest<'_, '_>,
        branch: &BranchId,
        root: &RelationalBranchRoot,
        scope: (KindId, DerivedIndexId, &AspectFieldLocator),
    ) -> Result<
        Option<(
            worth_foundational::facade::AspectValue,
            Option<worth_execution::ExecutionMemoryReservation>,
        )>,
        Error,
    > {
        let deny = |cause| Error::new(cause, scope.1);
        let (result, _) = request
            .run_native_with_checkpoint(|context| {
                let Some(generation) = self.pin_unique_field_index(context, branch, root, scope)?
                else {
                    return Ok(None);
                };
                let DerivedIndexEntries::EntityField(entries) = &generation.entries else {
                    return Err(deny(Denial::WrongIndexKind));
                };
                for (key, rows) in entries.iter() {
                    request_stage::checkpoint(context, 1)
                        .map_err(|stop| deny(Denial::Execution(stop.into())))?;
                    if rows.len() < 2 {
                        continue;
                    }
                    let bytes = key.canonical_value_bytes();
                    let work = (bytes.len() as u64).saturating_mul(4).saturating_add(512);
                    request_stage::checkpoint(context, work)
                        .map_err(|stop| deny(Denial::Execution(stop.into())))?;
                    let memory = request
                        .reserve_retained_memory_in_stage(context, work)
                        .map_err(|stop| deny(Denial::Execution(stop.into())))?;
                    let value = AuthoritativeFieldComparisonKey::value_from_canonical_bytes(bytes)
                        .map_err(|_| deny(Denial::CorruptIndexEntries))?;
                    return Ok(Some((value, memory)));
                }
                Ok(None)
            })
            .map_err(|stop| deny(Denial::Execution(stop.into())))?;
        result
    }

    fn pin_unique_field_index(
        &self,
        context: &mut worth_execution::MapKernelContext<'_, '_>,
        branch: &BranchId,
        root: &RelationalBranchRoot,
        scope: (KindId, DerivedIndexId, &AspectFieldLocator),
    ) -> Result<Option<std::sync::Arc<crate::branch::indexes::data::DerivedIndexGeneration>>, Error>
    {
        let (kind, index, locator) = scope;
        let deny = |cause| Error::new(cause, index);
        let state = self.state.read();
        let checkpoint =
            |stop: worth_execution::MapKernelStop| deny(Denial::Execution(stop.into()));
        super::admitted_definition::navigation(context, state.catalog.definition_count(), 1)
            .map_err(checkpoint)?;
        let definition = state
            .catalog
            .definition(index)
            .ok_or_else(|| deny(Denial::IndexNotInstalled))?;
        validate_scope(&definition, kind, locator)?;
        let Some(commit) = root.commit_id() else {
            return Ok(None);
        };
        let version = crate::contracts::identity::data::VersionId(
            root.axes()
                .ok_or_else(|| deny(Denial::ExactGenerationUnavailable))?
                .storage_version,
        );
        let schema = root.schema_authority().schema_version();
        let branch = definition.branch_scoped.then_some(branch);
        super::admitted_definition::navigation(context, state.bases.len(), 1)
            .map_err(checkpoint)?;
        let generation = if let Some(selection) = state.bases.get(branch, version) {
            if selection.commit != commit || selection.schema != schema {
                return Err(deny(Denial::ExactGenerationUnavailable));
            }
            super::admitted_definition::navigation(context, state.catalog.definition_count(), 1)
                .map_err(checkpoint)?;
            if selection.generation_is_empty(index) {
                return Ok(None);
            }
            selection.generation(index)
        } else {
            state
                .generations
                .published_for_commit_admitted(index, branch, commit, version, &mut |units, _| {
                    request_stage::checkpoint(context, units)
                })
                .map_err(|stop| {
                    let cause = match stop {
                        SelectedIndexGenerationAdmissionStop::Admission(stop) => stop.into(),
                        SelectedIndexGenerationAdmissionStop::AccountingOverflow => {
                            Cause::WorkCounterOverflow
                        }
                    };
                    deny(Denial::Execution(cause))
                })?
                .filter(|generation| generation.applicability.schema_version == schema)
        }
        .ok_or_else(|| deny(Denial::ExactGenerationUnavailable))?;
        Ok(Some(generation))
    }
}

fn validate_scope(
    definition: &crate::contracts::indexes::data::definition::DerivedIndexDefinition,
    kind: KindId,
    locator: &AspectFieldLocator,
) -> Result<(), Error> {
    if definition.unique_entity_kind == Some(kind)
        && matches!(&definition.kind, DerivedIndexKind::EntityField { field_locator } if field_locator == locator)
    {
        return Ok(());
    }
    Err(Error::new(Denial::WrongIndexKind, definition.index_id))
}
