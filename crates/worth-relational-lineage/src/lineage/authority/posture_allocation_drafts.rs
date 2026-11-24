//! Lineage-allocating event drafts (`Create` and `Restore`) in their one
//! canonical order.
//!
//! Allocating drafts form a single ascending record-target sequence, so the
//! k-th entity allocation of a commit binds the k-th created entity in
//! ascending `EntityId` order. Durable checkpoint capture relies on exactly
//! this order to publish lineage nodes without consulting patch order.

use std::collections::BTreeMap;

use crate::contracts::history::data::commit_receipt::RelationalCommitReceipt;
use crate::contracts::identity::data::primitives::LineageId;
use crate::contracts::lineage::data::naming::PersistentNameIdentity;
use crate::contracts::lineage::data::naming_denial::{LineageNameDenial, LineagePostureKind};
use crate::contracts::lineage::data::postures::LineageRecordTarget;
use worth_relational_storage::internal_port::storage::overlay::working_state::WorkingState;

use super::posture_drafts::PostureEventDraft;
use super::staged_effects::created_entity_lineage_slot;

/// A record target's lineage allocation before lineage ids are reserved.
enum AllocationDraft {
    Create {
        kept_name: Option<PersistentNameIdentity>,
    },
    Restore {
        source_lineage: LineageId,
        name: PersistentNameIdentity,
    },
}

/// Orders every allocating draft by record target; returns the drafts and
/// how many fresh lineage ids they reserve.
///
/// Complexity: O(A log A) in allocating targets.
pub(super) fn ordered_allocation_drafts(
    staged: &WorkingState,
    commit: &RelationalCommitReceipt,
    create_targets: BTreeMap<LineageRecordTarget, Option<PersistentNameIdentity>>,
    restore_targets: BTreeMap<LineageRecordTarget, (LineageId, PersistentNameIdentity)>,
) -> Result<(Vec<PostureEventDraft>, usize), LineageNameDenial> {
    let mut allocations = BTreeMap::new();
    for (target, kept_name) in create_targets {
        allocations.insert(target, AllocationDraft::Create { kept_name });
    }
    for (target, (source_lineage, name)) in restore_targets {
        let restore = AllocationDraft::Restore {
            source_lineage,
            name,
        };
        if allocations.insert(target, restore).is_some() {
            return Err(LineageNameDenial::LineagePostureMismatch {
                posture: LineagePostureKind::Restore,
                subject: target.subject(),
                holder: target.to_holder(),
                detail: "a restored target is also claimed as a created target".to_owned(),
            });
        }
    }
    let mut drafts = Vec::with_capacity(allocations.len());
    let mut reserved = 0usize;
    for (target, allocation) in allocations {
        let existing_lineage_id = match target {
            LineageRecordTarget::Entity(entity_id) => {
                created_entity_lineage_slot(staged, entity_id, commit.version_id).flatten()
            }
            LineageRecordTarget::Relation(_) => None,
        };
        reserved += usize::from(existing_lineage_id.is_none());
        drafts.push(match allocation {
            AllocationDraft::Create { kept_name } => PostureEventDraft::Create {
                target,
                existing_lineage_id,
                kept_name,
            },
            AllocationDraft::Restore {
                source_lineage,
                name,
            } => PostureEventDraft::Restore {
                target,
                source_lineage,
                name,
                existing_lineage_id,
            },
        });
    }
    Ok((drafts, reserved))
}
