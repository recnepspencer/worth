//! The relation revision uses the same catalog and symbolic-key prices as its entity twin.
use super::super::VisibilityProjectionView;
use crate::identity::data::RelationId;
use crate::storage::data::{
    RecordLifecycleState, RelationalFieldPresence, RelationalFieldRevision,
};
use worth_foundational::facade::{AspectFieldLocator, AspectShape, LocatorAuthority};
impl VisibilityProjectionView {
    pub(super) fn read_relation_field_revision_funded(
        &self,
        partition: crate::storage::residency::RelationalRegionHeadView<'_>,
        relation: RelationId,
        locator: &AspectFieldLocator,
        mut admit_work: impl FnMut(u64) -> Result<(), worth_execution::MapKernelStop>,
    ) -> Result<Option<RelationalFieldRevision>, worth_execution::MapKernelStop> {
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
        admit_work(3)?;
        let Some(slot) = partition.relation_arena().get(&relation) else {
            return Ok(None);
        };
        if slot.lifecycle() != RecordLifecycleState::Live {
            return Ok(None);
        }
        let Some(kind) = slot.kind_id() else {
            return Ok(None);
        };
        admit_work(super::admitted_entity::btree_navigation_work(
            root.schema_authority().relation_aspect_plan_count(),
        ))?;
        let Some(plan) = self.relation_aspect_plan(kind) else {
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
            let field_comparisons = (struct_fields as u64).saturating_add(1);
            let work = 2_u64
                .saturating_add(aspect_text.len() as u64)
                .saturating_add(
                    field_comparisons.saturating_mul((field_text.len() as u64).saturating_add(1)),
                );
            admit_work(work)?;
            if binding.aspect_key() == locator.aspect().aspect_key()
                && (binding.targets_relation_scalar_field(field)
                    || binding.targets_relation_struct_field(field))
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
            .relation_arena()
            .field_revisions_at(relation.slot_index())
        else {
            return Ok(None);
        };
        // StringInterner hashes both borrowed UTF-8 keys before the fixed
        // symbol-pair B-tree lookup. No string is cloned on this read.
        let text_work = aspect_text.len().saturating_add(field_text.len());
        admit_work((text_work as u64).saturating_add(2))?;
        let (aspect_symbol, field_symbol) = (
            self.symbols
                .with_read(|symbols| symbols.symbol(aspect_text)),
            self.symbols.with_read(|symbols| symbols.symbol(field_text)),
        );
        let explicit = if let Some(key) = aspect_symbol.zip(field_symbol) {
            admit_work(super::admitted_entity::btree_navigation_work(
                revisions.len(),
            ))?;
            revisions.get(&key).copied()
        } else {
            None
        };
        admit_work(1)?;
        let Some(created) = partition
            .relation_arena()
            .created_at_for_slot(relation.slot_index())
        else {
            return Ok(None);
        };
        Ok(Some(explicit.unwrap_or(RelationalFieldRevision::new(
            created,
            RelationalFieldPresence::Absent,
        ))))
    }
}
