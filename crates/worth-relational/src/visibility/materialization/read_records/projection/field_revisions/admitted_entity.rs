//! One exact-root field-revision read for unrestricted and admitted callers.

use worth_foundational::facade::{AspectFieldLocator, AspectShape, LocatorAuthority};

use crate::execution::RelationalExecutionDenialCause as Cause;
use crate::identity::data::EntityId;
use crate::storage::data::{
    RecordLifecycleState, RelationalFieldPresence, RelationalFieldRevision,
};

use super::super::VisibilityProjectionView;

impl VisibilityProjectionView {
    /// Admission covers the selected catalog walk, text comparisons and
    /// revision lookup before each read. There is no owned record projection.
    /// `None` retains the unrestricted read's unavailable/undeclared meaning.
    pub fn entity_field_revision_admitted<Stop>(
        &self,
        execution: &worth_execution::ExecutionRequest<'_, '_>,
        entity: EntityId,
        locator: &AspectFieldLocator,
        mut admit_work: impl FnMut(u64) -> Result<(), Stop>,
    ) -> Result<
        Result<Option<RelationalFieldRevision>, Stop>,
        crate::execution::RelationalExecutionDenialCause,
    > {
        crate::execution::check_request(execution)?;
        if self.is_exact_basis() {
            self.demand_regions_head(execution, [entity.partition_id])?;
        }
        super::super::super::read_request::checked(execution, || {
            Ok('read: {
                if let Err(stop) = admit_work(1) {
                    break 'read Err(stop);
                }
                if !self.is_exact_basis()
                    || locator.aspect().authority() != LocatorAuthority::Authoritative
                {
                    break 'read Ok(None);
                }
                let [field] = locator.field_path().fields() else {
                    break 'read Ok(None);
                };
                let Some(root) = self.basis.root() else {
                    break 'read Ok(None);
                };
                // The persistent partition tree has one fixed-key visit per u32 bit.
                if let Err(stop) = admit_work(33) {
                    break 'read Err(stop);
                }
                let Some(partition) = root.admitted_head_region(entity.partition_id) else {
                    break 'read Ok(None);
                };
                if let Err(stop) = admit_work(3) {
                    break 'read Err(stop);
                }
                let Some(slot) = partition.entity_arena().get(&entity) else {
                    break 'read Ok(None);
                };
                if slot.lifecycle() != RecordLifecycleState::Live {
                    break 'read Ok(None);
                }
                let Some(kind) = slot.kind_id() else {
                    break 'read Ok(None);
                };
                if let Err(stop) = admit_work(btree_navigation_work(
                    root.schema_authority().entity_aspect_plan_count(),
                )) {
                    break 'read Err(stop);
                }
                let Some(plan) = self.entity_aspect_plan(kind) else {
                    break 'read Ok(None);
                };
                let aspect_text = locator.aspect().aspect_key().as_str();
                let field_text = field.as_str();
                let mut declared = false;
                for binding in &plan.executable_bindings {
                    let struct_fields = match binding.contract.shape() {
                        AspectShape::Struct(shape) => shape.fields().len(),
                        _ => 0,
                    };
                    let field_comparisons = u64::try_from(struct_fields)
                        .ok()
                        .and_then(|count| count.checked_add(1))
                        .ok_or(Cause::WorkCounterOverflow)?;
                    let aspect_bytes =
                        u64::try_from(aspect_text.len()).map_err(|_| Cause::WorkCounterOverflow)?;
                    let field_bytes = u64::try_from(field_text.len())
                        .ok()
                        .and_then(|bytes| bytes.checked_add(1))
                        .ok_or(Cause::WorkCounterOverflow)?;
                    let work = field_comparisons
                        .checked_mul(field_bytes)
                        .and_then(|work| work.checked_add(aspect_bytes))
                        .and_then(|work| work.checked_add(2))
                        .ok_or(Cause::WorkCounterOverflow)?;
                    if let Err(stop) = admit_work(work) {
                        break 'read Err(stop);
                    }
                    if binding.aspect_key() == locator.aspect().aspect_key()
                        && (binding.targets_entity_scalar_field(field)
                            || binding.targets_entity_struct_field(field))
                    {
                        declared = true;
                        break;
                    }
                }
                if !declared {
                    break 'read Ok(None);
                }
                if let Err(stop) = admit_work(2) {
                    break 'read Err(stop);
                }
                let Some(revisions) = partition
                    .entity_arena()
                    .field_revisions_at(entity.slot_index())
                else {
                    break 'read Ok(None);
                };
                // StringInterner hashes both borrowed UTF-8 keys before the fixed
                // symbol-pair B-tree lookup. No string is cloned on this read.
                let text_work = u64::try_from(aspect_text.len())
                    .ok()
                    .zip(u64::try_from(field_text.len()).ok())
                    .and_then(|(aspect, field)| aspect.checked_add(field))
                    .and_then(|work| work.checked_add(2))
                    .ok_or(Cause::WorkCounterOverflow)?;
                if let Err(stop) = admit_work(text_work) {
                    break 'read Err(stop);
                }
                let (aspect_symbol, field_symbol) = (
                    self.symbols
                        .with_read(|symbols| symbols.symbol(aspect_text)),
                    self.symbols.with_read(|symbols| symbols.symbol(field_text)),
                );
                let explicit = if let Some(key) = aspect_symbol.zip(field_symbol) {
                    if let Err(stop) = admit_work(btree_navigation_work(revisions.len())) {
                        break 'read Err(stop);
                    }
                    revisions.get(&key).copied()
                } else {
                    None
                };
                if let Err(stop) = admit_work(1) {
                    break 'read Err(stop);
                }
                let Some(created) = partition
                    .entity_arena()
                    .created_at_for_slot(entity.slot_index())
                else {
                    break 'read Ok(None);
                };
                Ok(Some(explicit.unwrap_or(RelationalFieldRevision::new(
                    created,
                    RelationalFieldPresence::Absent,
                ))))
            })
        })
        .and_then(|read| read)
    }
}

pub(super) fn btree_navigation_work(entries: usize) -> u64 {
    // A nonleaf root has two children; every other std B-tree node has at
    // least six. Eleven fixed-key comparisons per occupied level is safe.
    let mut minimum = 1_usize;
    let mut levels = 1_u64;
    while entries > minimum {
        minimum = minimum.saturating_mul(6).saturating_add(5);
        levels = levels.saturating_add(1);
        if minimum == usize::MAX {
            break;
        }
    }
    levels.saturating_mul(11)
}
