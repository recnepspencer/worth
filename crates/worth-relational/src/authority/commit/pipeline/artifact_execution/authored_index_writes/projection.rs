//! Canonical authored targets, independent of semantic-delta equality.
use super::*;
use crate::identity::data::EntityId;
use crate::indexes::data::DerivedIndexKind;
use crate::storage::data::AuthoritativeFieldComparisonKey as Comparison;
use crate::transactions::data::{EntityMutationIntent as Entity, MutationIntent};
use worth_foundational::facade::{PortableAspectPatchOperation, PortableRecordAspectPatch};

pub(super) struct Projection<'a, 'kernel, 'controls, 'request, 'authority> {
    pub(super) execution: ExecutionRequest<'request, 'authority>,
    pub(super) runtime: &'a RelationalPreparationRuntime,
    pub(super) catalog: &'a crate::runtime::IndexDefinitionCatalog,
    pub(super) unavailable: bool,
    pub(super) selected: &'a SelectedRelationalBranchState,
    pub(super) working: &'a WorkingState,
    pub(super) context: &'a mut MapKernelContext<'kernel, 'controls>,
    pub(super) output: funding::Output,
}

impl Projection<'_, '_, '_, '_, '_> {
    pub(super) fn project(
        &mut self,
        plan: RelationalSemanticPlan<'_>,
        allocations: &[crate::history::data::CanonicalRecordAllocation],
    ) -> Result<(), Denial> {
        for allocation in allocations {
            work(self.context, 1)?;
            if let RecordRef::Entity(entity) = allocation.record() {
                self.record(*entity, None)?;
            }
        }
        if let Some(selection) = plan.selection() {
            work(self.context, 1)?;
            self.record(selection.entity(), None)?;
        }
        for intent in &plan.commands().merged_intents {
            work(self.context, 1)?;
            match intent {
                MutationIntent::Entity(Entity::UpdateFields(intent)) => {
                    for locator in intent.fields.locators() {
                        work(self.context, locator.field_path().fields().len() as u64)?;
                        if let [field] = locator.field_path().fields() {
                            self.field(
                                intent.entity_id,
                                &Locus {
                                    aspect: locator.aspect().aspect_key(),
                                    field,
                                },
                            )?;
                        }
                    }
                }
                MutationIntent::Entity(Entity::ApplyAspectPatch(intent)) => {
                    self.patch(intent.entity_id, &intent.aspect_patch)?
                }
                MutationIntent::Entity(Entity::Replace(intent)) => {
                    self.record(intent.entity_id, None)?
                }
                MutationIntent::Entity(Entity::Delete(intent)) => {
                    self.record(intent.entity_id, None)?
                }
                MutationIntent::Materialization(intent) => {
                    if let RecordRef::Entity(entity) = intent.record() {
                        self.record(entity, None)?;
                    }
                }
                MutationIntent::Create(_)
                | MutationIntent::Relation(_)
                | MutationIntent::Entity(Entity::Revalidate(_)) => {}
            }
        }
        Ok(())
    }

    fn patch(&mut self, entity: EntityId, patch: &PortableRecordAspectPatch) -> Result<(), Denial> {
        for operation in patch.operations() {
            work(self.context, 1)?;
            let aspect = operation.basis().key();
            match operation {
                PortableAspectPatchOperation::SetWhole { .. }
                | PortableAspectPatchOperation::ClearWhole { .. } => {
                    self.record(entity, Some(aspect))?
                }
                PortableAspectPatchOperation::PatchFields {
                    field_sets,
                    field_clears,
                    ..
                } => {
                    for field in field_sets
                        .iter()
                        .map(|set| set.field())
                        .chain(field_clears.iter())
                    {
                        self.field(entity, &Locus { aspect, field })?;
                    }
                }
            }
        }
        Ok(())
    }

    fn record(&mut self, entity: EntityId, aspect: Option<&AspectKey>) -> Result<(), Denial> {
        let before = values::live(&self.execution, self.selected, entity)?;
        let after = values::live(&self.execution, self.working, entity)?;
        let schema = self
            .runtime
            .config
            .schema
            .registry
            .primary_schema_version_id();
        let Some(routing) = self.catalog.routing(schema) else {
            self.unavailable = true;
            return Ok(());
        };
        let old_kind = before.map(|(kind, _)| kind);
        let new_kind = after.map(|(kind, _)| kind);
        for kind in old_kind
            .into_iter()
            .chain(new_kind.filter(|kind| Some(*kind) != old_kind))
        {
            let levels = usize::BITS - routing.kind_count().leading_zeros();
            work(self.context, u64::from(levels) * 11 + 1)?;
            let locators = if let Some(aspect) = aspect {
                let fields = routing.entity_fields(kind).len();
                let field_levels = usize::BITS - fields.leading_zeros();
                let width = (aspect.as_str().len() as u64)
                    .checked_add(1)
                    .ok_or_else(overflow)?;
                let lookup = u64::from(field_levels)
                    .checked_mul(11)
                    .and_then(|units| units.checked_mul(width))
                    .and_then(|units| units.checked_add(u64::from(levels) * 11 + 2))
                    .ok_or_else(overflow)?;
                work(self.context, lookup)?;
                routing.entity_fields_in_aspect(kind, aspect)
            } else {
                routing.entity_fields(kind)
            };
            for locator in locators {
                work(self.context, 1)?;
                if let [field] = locator.field_path().fields() {
                    self.field(
                        entity,
                        &Locus {
                            aspect: locator.aspect().aspect_key(),
                            field,
                        },
                    )?;
                }
            }
        }
        Ok(())
    }

    fn field(&mut self, entity: EntityId, locus: &Locus<'_>) -> Result<(), Denial> {
        let before = values::live(&self.execution, self.selected, entity)?;
        let after = values::live(&self.execution, self.working, entity)?;
        let context = &mut *self.context;
        let output = &mut self.output;
        self.catalog.with_field_admitted(
            locus.aspect,
            locus.field,
            context,
            |definitions, context| {
                for definition in definitions {
                    work(context, 1)?;
                    let DerivedIndexKind::EntityField { field_locator } = &definition.kind else {
                        continue;
                    };
                    if !Definition::supports_locator(field_locator) {
                        continue;
                    }
                    let old = values::field(
                        before
                            .as_ref()
                            .and_then(|(_, slot)| slot.extra().authoritative_aspect_state.as_ref()),
                        field_locator,
                        context,
                    )?;
                    let new = values::field(
                        after
                            .as_ref()
                            .and_then(|(_, slot)| slot.extra().authoritative_aspect_state.as_ref()),
                        field_locator,
                        context,
                    )?;
                    let value_work = old
                        .into_iter()
                        .chain(new)
                        .try_fold(0u64, |n, value| {
                            n.checked_add(value.semantic_byte_width() as u64)
                        })
                        .ok_or_else(overflow)?;
                    work(context, value_work)?;
                    let keys = old
                        .into_iter()
                        .chain(new)
                        .try_fold(0u64, |n, value| {
                            n.checked_add(Comparison::required_encoded_capacity_bytes(value)?)
                        })
                        .ok_or(Denial::SizeOverflow)?;
                    work(context, field_locator.field_path().fields().len() as u64)?;
                    let bytes = (definition.name.capacity() as u64)
                        .checked_add(
                            field_locator
                                .aspect()
                                .aspect_key()
                                .owned_allocation_capacity_bytes()
                                as u64,
                        )
                        .and_then(|n| {
                            n.checked_add(
                                field_locator.field_path().owned_allocation_capacity_bytes() as u64,
                            )
                        })
                        .and_then(|n| n.checked_add(keys))
                        .ok_or(Denial::SizeOverflow)?;
                    output.push(context, bytes, || Entry {
                        definition: Definition {
                            index_id: definition.index_id,
                            name: definition.name.clone(),
                            branch_scoped: definition.branch_scoped,
                            locator: field_locator.clone(),
                        },
                        entity,
                        old_kind: before.as_ref().map(|(kind, _)| *kind),
                        new_kind: after.as_ref().map(|(kind, _)| *kind),
                        old_key: old.map(Comparison::from_aspect_value),
                        new_key: new.map(Comparison::from_aspect_value),
                    })?;
                }
                Ok(())
            },
        )
    }
}
