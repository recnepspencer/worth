//! Public #358: a readmitted consumer chain gets its mark rows at the head
//! where readmission compared it in full, bottom-up, so the consumer's
//! registration sees resolved upstreams and is Clean there.

use std::sync::{Arc, OnceLock};

use worth_relational::facade::runtime::PositionedRelationalSnapshot;
use worth_relational::facade::runtime::RelationalRuntime;
use worth_relational::facade::snapshots::SnapshotHandle;

use super::super::super::{ComparableSourceFacts, RecordedSettlementIdentity};
use super::super::super::{SealedNativeOutputWitness, SelectionCoordinate};
use super::super::derived::SourceSettlementCurrentness as Currentness;
use super::super::mark_state::FullVerificationReason;
use super::super::verification::SettlementVerificationStop;
use super::super::SourceInvalidationOwner;

/// One row of a readmitted chain, as readmission proved it at the head.
pub struct ReadmittedNode {
    pub identity: Arc<RecordedSettlementIdentity>,
    pub facts: ComparableSourceFacts,
    pub witness: Arc<OnceLock<SealedNativeOutputWitness>>,
    /// The re-identified rows this one consumed; empty for a root.
    pub upstream: im::OrdSet<Arc<RecordedSettlementIdentity>>,
    pub historical: Option<SelectionCoordinate>,
    /// Readmission compared this row's facts in full at the head (public
    /// #429), so a fact marks cannot certify (`UnsupportedFact`) is proven.
    pub fully_compared: bool,
}

/// Whether readmission accepts `node` at `currentness` without changing it:
/// Clean; or, for a fully compared row, `UnsupportedFact`, or pending only on
/// rows of this chain already accepted (a consumer of an `UnsupportedFact`
/// row is registered pending on it, public #429).
fn accepts(
    node: &ReadmittedNode,
    currentness: &Currentness,
    accepted: &im::OrdSet<Arc<RecordedSettlementIdentity>>,
) -> bool {
    match currentness {
        Currentness::Clean => true,
        Currentness::FullVerificationRequired(FullVerificationReason::UnsupportedFact) => {
            node.fully_compared
        }
        Currentness::PendingUpstream(edges) => {
            node.fully_compared && edges.iter().all(|edge| accepted.contains(edge))
        }
        _ => false,
    }
}

impl SourceInvalidationOwner {
    /// Post `nodes` (bottom-up, the consumer last) at `selected`, the live
    /// head readmission compared. A missing row is registered; a Clean one is
    /// kept, as is a fully compared one marks cannot certify ([`accepts`]); a
    /// delivery gap or declared change is re-established. Anything else
    /// refuses (`false`). Rows posted before a refusal stay: each was proven
    /// at `selected`. Returns whether the consumer is accepted there.
    pub fn post_readmitted_chain(
        &self,
        runtime: &RelationalRuntime,
        snapshot: &SnapshotHandle,
        selected: &PositionedRelationalSnapshot,
        nodes: &[ReadmittedNode],
    ) -> Result<bool, SettlementVerificationStop> {
        let Some(consumer) = nodes.last() else {
            return Ok(false);
        };
        let mut accepted = im::OrdSet::new();
        for node in nodes {
            let Some(witness) = node.witness.get() else {
                return Ok(false);
            };
            let Some(cell) = self.cell_for_read(selected)? else {
                return Ok(false);
            };
            let image = cell.read_image();
            if image.root_id() != selected.root_id()
                || image.commit_id() != selected.commit_id()
                || image.position() != selected.position()
            {
                return Ok(false);
            }
            let present = image
                .payload()
                .current
                .settlements
                .contains_key(&node.identity);
            drop(image);
            if !present {
                if node.upstream.is_empty() {
                    if !self.establish_verified_root(
                        selected,
                        &node.identity,
                        &node.facts,
                        witness,
                        node.historical.as_ref(),
                    )? {
                        return Ok(false);
                    }
                } else {
                    self.establish_verified_consumed(
                        selected,
                        &node.identity,
                        &node.facts,
                        witness,
                        node.upstream.clone(),
                        node.historical.as_ref(),
                    )?;
                }
                accepted.insert(Arc::clone(&node.identity));
                continue;
            }
            match self.currentness(selected, &node.identity)? {
                currentness if accepts(node, &currentness, &accepted) => {}
                Currentness::FullVerificationRequired(
                    FullVerificationReason::RetainedDeliveryGap
                    | FullVerificationReason::DeclaredChangeUnavailable,
                ) => {
                    if !self.reestablish_verified(
                        runtime,
                        snapshot,
                        selected,
                        &node.identity,
                        &node.facts,
                    )? {
                        return Ok(false);
                    }
                }
                Currentness::Clean
                | Currentness::Dirty(_)
                | Currentness::PendingUpstream(_)
                | Currentness::FullVerificationRequired(_)
                | Currentness::Foreign => return Ok(false),
            }
            accepted.insert(Arc::clone(&node.identity));
        }
        let clean = accepts(consumer, &self.currentness(selected, &consumer.identity)?, &accepted);
        if clean {
            super::super::readmission_observation::record_readmitted_chain();
        }
        Ok(clean)
    }
}
