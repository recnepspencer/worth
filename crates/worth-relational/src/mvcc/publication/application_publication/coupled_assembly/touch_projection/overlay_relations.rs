//! The relations of a reopening chain's own-output view (#277): each relation
//! record the chain's rounds touched whose live adjacency differs between the
//! chain head's pre-pin and the current candidate, with its live endpoints on
//! each side. Query serves adjacency over these as "current minus the chain's
//! own effects"; `Chain::intervened` keeps them exclusive to the chain.
use super::super::super::custody;
use crate::identity::data::{EntityId, KindId, RelationId};
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use crate::transactions::data::primitives::RecordRef;
use crate::visibility::materialization::read_records::reader::pinned_read::{
    RelationalAppliedCandidateReadIdentity, RelationalPinnedRead,
};
use worth_execution::ExecutionRequest;
use crate::storage::data::{RecordLifecycleState, RelationReadRecord};

/// One live side of an overlay relation: its kind and endpoints.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct RelationalCoupledOverlayEndpoints {
    kind: KindId,
    source: EntityId,
    target: EntityId,
}

impl RelationalCoupledOverlayEndpoints {
    pub const fn kind(&self) -> KindId {
        self.kind
    }
    pub const fn source(&self) -> EntityId {
        self.source
    }
    pub const fn target(&self) -> EntityId {
        self.target
    }
}

/// A chain relation whose live adjacency changed since the chain head's
/// pre-pin. `head` is absent for a relation the chain created, `current` for
/// one it deleted; both are present, with different endpoints, for one it
/// moved (`UpdateEndpoints`).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RelationalCoupledOverlayRelation {
    id: RelationId,
    head: Option<RelationalCoupledOverlayEndpoints>,
    current: Option<RelationalCoupledOverlayEndpoints>,
}

impl RelationalCoupledOverlayRelation {
    pub const fn id(&self) -> RelationId {
        self.id
    }
    pub const fn head(&self) -> Option<RelationalCoupledOverlayEndpoints> {
        self.head
    }
    pub const fn current(&self) -> Option<RelationalCoupledOverlayEndpoints> {
        self.current
    }
}

/// The overlay relations of `records`, sorted by id: each relation record
/// read at `head` and `current`, kept only when its live sides differ.
/// O(relation records) point reads, each metered by the projection read.
pub(super) fn overlay_relations(
    execution: &ExecutionRequest<'_, '_>,
    records: &[RecordRef],
    head: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
    current: &RelationalPinnedRead<RelationalAppliedCandidateReadIdentity>,
) -> Result<Vec<RelationalCoupledOverlayRelation>, TransactionCommitError> {
    let head = head.project_candidate();
    let current = current.project_candidate();
    let mut relations = Vec::new();
    for record in records {
        let RecordRef::Relation(id) = record else {
            continue;
        };
        let before = head
            .authoritative_relation_record(execution, *id)
            .map_err(custody::execution_stop)?;
        let after = current
            .authoritative_relation_record(execution, *id)
            .map_err(custody::execution_stop)?;
        let (before, after) = (live_side(before.as_ref()), live_side(after.as_ref()));
        if before != after {
            relations.push(RelationalCoupledOverlayRelation {
                id: *id,
                head: before,
                current: after,
            });
        }
    }
    relations.sort_unstable_by_key(|relation| relation.id);
    relations.dedup_by_key(|relation| relation.id);
    Ok(relations)
}

/// A record's kind and endpoints when it is live there. A retired or
/// audit-retained slot is no live adjacency, exactly as adjacency reads see it.
fn live_side(record: Option<&RelationReadRecord>) -> Option<RelationalCoupledOverlayEndpoints> {
    record
        .filter(|record| record.lifecycle == RecordLifecycleState::Live)
        .map(|record| RelationalCoupledOverlayEndpoints {
            kind: record.kind.kind_id,
            source: record.source,
            target: record.target,
        })
}
