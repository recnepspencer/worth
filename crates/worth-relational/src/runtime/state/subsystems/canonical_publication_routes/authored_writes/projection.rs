//! Project canonical authored intents, never the semantic-delta-only patch.
use super::{AuthoredWriteGeneration, Denial, Key};
use crate::history::data::{CanonicalCommitEnvelope, CommitId};
use crate::history::RelationalAuthoredWriteScope as Scope;
use crate::transactions::data::{
    AspectFieldPatch, EntityMutationIntent, MutationIntent, RecordRef, RelationMutationIntent,
};
use worth_execution::{ExecutionRequest, MapKernelContext};
use worth_foundational::facade::{
    AspectKey, CanonicalFieldPath, PortableAspectPatchOperation, PortableRecordAspectPatch,
};

pub(super) fn append(
    execution: ExecutionRequest<'_, '_>,
    context: &mut MapKernelContext<'_, '_>,
    generation: &mut AuthoredWriteGeneration,
    envelope: &CanonicalCommitEnvelope,
) -> Result<(), Denial> {
    let mut output = Projection {
        execution,
        context,
        generation,
        commit: envelope.commit.commit_id,
    };
    for allocation in envelope.record_allocations() {
        output.lifecycle(allocation.record())?;
    }
    for intent in &envelope.merged_plan.merged_intents {
        crate::history::visit_authored_write(output.context)?;
        match intent {
            // New records already have their actual resolved identities above.
            MutationIntent::Create(_) => {}
            MutationIntent::Entity(intent) => match intent {
                EntityMutationIntent::UpdateFields(intent) => {
                    output.fields(&RecordRef::Entity(intent.entity_id), &intent.fields)?
                }
                EntityMutationIntent::ApplyAspectPatch(intent) => {
                    output.aspects(&RecordRef::Entity(intent.entity_id), &intent.aspect_patch)?
                }
                EntityMutationIntent::Replace(intent) => {
                    output.lifecycle(&RecordRef::Entity(intent.entity_id))?
                }
                EntityMutationIntent::Delete(intent) => {
                    output.lifecycle(&RecordRef::Entity(intent.entity_id))?
                }
                // A rule demand is not an authored write.
                EntityMutationIntent::Revalidate(_) => {}
            },
            MutationIntent::Relation(intent) => match intent {
                RelationMutationIntent::ApplyAspectPatch(intent) => output.aspects(
                    &RecordRef::Relation(intent.relation_id),
                    &intent.aspect_patch,
                )?,
                RelationMutationIntent::UpdateEndpoints(intent) => {
                    output.lifecycle(&RecordRef::Relation(intent.relation_id))?
                }
                RelationMutationIntent::Delete(intent) => {
                    output.lifecycle(&RecordRef::Relation(intent.relation_id))?
                }
                RelationMutationIntent::Revalidate(_) => {}
            },
            MutationIntent::Materialization(intent) => output.lifecycle(&intent.record())?,
        }
    }
    Ok(())
}

struct Projection<'a, 'request, 'authority, 'kernel, 'controls> {
    execution: ExecutionRequest<'request, 'authority>,
    context: &'a mut MapKernelContext<'kernel, 'controls>,
    generation: &'a mut AuthoredWriteGeneration,
    commit: CommitId,
}

impl Projection<'_, '_, '_, '_, '_> {
    fn emit(&mut self, bytes: usize, make: impl FnOnce() -> Key) -> Result<(), Denial> {
        let work = u64::try_from(bytes)
            .ok()
            .and_then(|bytes| bytes.checked_add(1))
            .ok_or(Denial::Interrupted(
                worth_execution::MapKernelStop::WorkCounterOverflow,
            ))?;
        if self.context.has_accounting() {
            self.context.checkpoint(work)?;
        } else {
            self.context.poll()?;
        }
        // Fund temporary key construction before cloning. The map separately
        // admits its persistent key copy; both coexist until this call returns.
        let _scratch = self.execution.reserve_retained_memory(bytes as u64)?;
        self.generation
            .append(self.execution, self.context, &make(), self.commit)
    }

    fn lifecycle(&mut self, record: &RecordRef) -> Result<(), Denial> {
        self.emit(0, || Key::Record {
            record: record.clone(),
            scope: Scope::Lifecycle,
        })
    }

    fn fields(&mut self, record: &RecordRef, fields: &AspectFieldPatch) -> Result<(), Denial> {
        for locator in fields.locators() {
            let aspect = locator.aspect().aspect_key();
            let path = locator.field_path();
            self.field(record, aspect, path)?;
        }
        Ok(())
    }

    fn field(
        &mut self,
        record: &RecordRef,
        aspect: &AspectKey,
        path: &CanonicalFieldPath,
    ) -> Result<(), Denial> {
        let bytes = aspect
            .owned_allocation_capacity_bytes()
            .checked_add(path.owned_allocation_capacity_bytes())
            .ok_or(Denial::SizeOverflow)?;
        self.emit(bytes, || Key::Record {
            record: record.clone(),
            scope: Scope::Field(aspect.clone(), path.clone()),
        })
    }

    fn aspects(
        &mut self,
        record: &RecordRef,
        patch: &PortableRecordAspectPatch,
    ) -> Result<(), Denial> {
        for operation in patch.operations() {
            let aspect = operation.basis().key();
            match operation {
                PortableAspectPatchOperation::SetWhole { .. }
                | PortableAspectPatchOperation::ClearWhole { .. } => {
                    self.emit(aspect.owned_allocation_capacity_bytes(), || Key::Record {
                        record: record.clone(),
                        scope: Scope::WholeAspect(aspect.clone()),
                    })?;
                }
                PortableAspectPatchOperation::PatchFields {
                    field_sets,
                    field_clears,
                    ..
                } => {
                    for field in field_sets
                        .iter()
                        .map(|field| field.field())
                        .chain(field_clears.iter())
                    {
                        let bytes = aspect
                            .owned_allocation_capacity_bytes()
                            .checked_add(std::mem::size_of_val(field))
                            .and_then(|bytes| {
                                bytes.checked_add(field.owned_allocation_capacity_bytes())
                            })
                            .ok_or(Denial::SizeOverflow)?;
                        self.emit(bytes, || Key::Record {
                            record: record.clone(),
                            scope: Scope::Field(
                                aspect.clone(),
                                CanonicalFieldPath::single(field.clone()),
                            ),
                        })?;
                    }
                }
            }
        }
        Ok(())
    }
}
