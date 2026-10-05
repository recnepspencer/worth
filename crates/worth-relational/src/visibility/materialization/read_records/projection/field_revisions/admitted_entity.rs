//! One exact-root field-revision read for unrestricted and admitted callers.

use worth_foundational::facade::{AspectFieldLocator, AspectShape, LocatorAuthority};

use crate::identity::data::EntityId;
use crate::storage::data::{
    RecordLifecycleState, RelationalFieldPresence, RelationalFieldRevision,
};
use crate::storage::overlay::PartitionAccess;

use super::super::VisibilityProjectionView;

impl VisibilityProjectionView<'_> {
    /// Admission covers the selected catalog walk, text comparisons and
    /// revision lookup before each read. There is no owned record projection.
    /// `None` retains the unrestricted read's unavailable/undeclared meaning.
    pub fn entity_field_revision_admitted<Stop>(
        &self,
        entity: EntityId,
        locator: &AspectFieldLocator,
        mut admit_work: impl FnMut(u64) -> Result<(), Stop>,
    ) -> Result<Option<RelationalFieldRevision>, Stop> {
        admit_work(1)?;
        if !self.is_exact_basis() || locator.aspect().authority() != LocatorAuthority::Authoritative
        {
            return Ok(None);
        }
        let [field] = locator.field_path().fields() else {
            return Ok(None);
        };
        let Some(root) = self.basis.root() else {
            return Ok(None);
        };
        // The persistent partition tree has one fixed-key visit per u32 bit.
        admit_work(33)?;
        let Some(partition) = root.get_partition(entity.partition_id) else {
            return Ok(None);
        };
        admit_work(3)?;
        let Some(slot) = partition.entity_arena.get(&entity) else {
            return Ok(None);
        };
        if slot.lifecycle() != RecordLifecycleState::Live {
            return Ok(None);
        }
        let Some(kind) = slot.kind_id() else {
            return Ok(None);
        };
        admit_work(btree_navigation_work(
            root.schema_authority().entity_aspect_plan_count(),
        ))?;
        let Some(plan) = self.entity_aspect_plan(kind) else {
            return Ok(None);
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
                .unwrap_or(u64::MAX)
                .saturating_add(1);
            let work = 2_u64
                .saturating_add(u64::try_from(aspect_text.len()).unwrap_or(u64::MAX))
                .saturating_add(
                    field_comparisons.saturating_mul(
                        u64::try_from(field_text.len())
                            .unwrap_or(u64::MAX)
                            .saturating_add(1),
                    ),
                );
            admit_work(work)?;
            if binding.aspect_key() == locator.aspect().aspect_key()
                && (binding.targets_entity_scalar_field(field)
                    || binding.targets_entity_struct_field(field))
            {
                declared = true;
                break;
            }
        }
        if !declared {
            return Ok(None);
        }
        admit_work(2)?;
        let Some(revisions) = partition
            .entity_arena
            .field_revisions_at(entity.slot_index())
        else {
            return Ok(None);
        };
        // StringInterner hashes both borrowed UTF-8 keys before the fixed
        // symbol-pair B-tree lookup. No string is cloned on this read.
        let text_work = aspect_text.len().saturating_add(field_text.len());
        admit_work(
            u64::try_from(text_work)
                .unwrap_or(u64::MAX)
                .saturating_add(2),
        )?;
        let (aspect_symbol, field_symbol) = self
            .runtime
            .services
            .symbols
            .with_read(|symbols| (symbols.symbol(aspect_text), symbols.symbol(field_text)));
        let explicit = if let Some(key) = aspect_symbol.zip(field_symbol) {
            admit_work(btree_navigation_work(revisions.len()))?;
            revisions.get(&key).copied()
        } else {
            None
        };
        admit_work(1)?;
        let Some(created) = partition
            .entity_arena
            .created_at_for_slot(entity.slot_index())
        else {
            return Ok(None);
        };
        Ok(Some(explicit.unwrap_or(RelationalFieldRevision::new(
            created,
            RelationalFieldPresence::Absent,
        ))))
    }
}

fn btree_navigation_work(entries: usize) -> u64 {
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
