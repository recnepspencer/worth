use worth_foundational::facade::{AspectFieldLocator, LocatorAuthority};

use crate::identity::data::{EntityId, RelationId};
use crate::storage::data::{
    RecordLifecycleState, RelationalFieldPresence, RelationalFieldRevision,
};
use crate::storage::overlay::PartitionAccess;

use super::VisibilityProjectionView;

#[path = "field_revisions/admitted_entity.rs"]
mod admitted_entity;

impl VisibilityProjectionView<'_> {
    /// Native field status and revision at an exact immutable root. `None`
    /// means unavailable provenance; it never means an absent field. Current
    /// checkpoint images retain native revisions, including absent revisions.
    pub fn entity_field_revision(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        locator: &AspectFieldLocator,
    ) -> Result<Option<RelationalFieldRevision>, crate::execution::RelationalExecutionDenialCause>
    {
        Ok(self
            .entity_field_revision_admitted(execution, entity, locator, |_| {
                Ok::<_, std::convert::Infallible>(())
            })?
            .unwrap_or_else(|never| match never {}))
    }

    pub fn relation_field_revision(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        relation: RelationId,
        locator: &AspectFieldLocator,
    ) -> Result<Option<RelationalFieldRevision>, crate::execution::RelationalExecutionDenialCause>
    {
        crate::visibility::materialization::read_records::read_request::checked(execution, || {
            if !self.is_exact_basis()
                || locator.aspect().authority() != LocatorAuthority::Authoritative
            {
                return None;
            }
            let [field] = locator.field_path().fields() else {
                return None;
            };
            let root = self.basis.root()?;
            let partition = root.get_partition(relation.partition_id)?;
            let slot = partition.relation_arena.get(&relation)?;
            if slot.lifecycle() != RecordLifecycleState::Live {
                return None;
            }
            let kind = slot.kind_id()?;
            let declared = self
                .relation_aspect_plan(kind)?
                .executable_bindings
                .iter()
                .any(|binding| {
                    binding.aspect_key() == locator.aspect().aspect_key()
                        && (binding.targets_relation_scalar_field(field)
                            || binding.targets_relation_struct_field(field))
                });
            if !declared {
                return None;
            }
            let revisions = partition
                .relation_arena
                .field_revisions_at(relation.slot_index())?;
            let (aspect_symbol, field_symbol) =
                self.runtime.services.symbols.with_read(|symbols| {
                    (
                        symbols.symbol(locator.aspect().aspect_key().as_str()),
                        symbols.symbol(field.as_str()),
                    )
                });
            let explicit = aspect_symbol
                .zip(field_symbol)
                .and_then(|key| revisions.get(&key).copied());
            Some(
                explicit.unwrap_or(RelationalFieldRevision::new(
                    partition
                        .relation_arena
                        .created_at_for_slot(relation.slot_index())?,
                    RelationalFieldPresence::Absent,
                )),
            )
        })
    }
}

#[cfg(test)]
#[path = "field_revisions/restore_tests.rs"]
mod restore_tests;

#[cfg(test)]
#[path = "field_revisions/tests.rs"]
mod tests;
