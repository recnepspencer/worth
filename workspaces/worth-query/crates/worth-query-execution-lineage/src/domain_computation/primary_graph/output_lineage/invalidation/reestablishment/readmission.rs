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
}

impl SourceInvalidationOwner {
    /// Post `nodes` (bottom-up, the consumer last) at `selected`, the live
    /// head readmission compared. A missing row is registered; a Clean one is
    /// kept; a delivery gap or declared change is re-established. Anything
    /// else refuses (`false`). Rows posted before a refusal stay: each was
    /// proven at `selected`. Returns whether the consumer is Clean there.
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
                continue;
            }
            match self.currentness(selected, &node.identity)? {
                Currentness::Clean => {}
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
                Currentness::Dirty(_)
                | Currentness::PendingUpstream(_)
                | Currentness::FullVerificationRequired(_)
                | Currentness::Foreign => return Ok(false),
            }
        }
        let clean = matches!(
            self.currentness(selected, &consumer.identity)?,
            Currentness::Clean
        );
        if clean {
            super::super::readmission_observation::record_readmitted_chain();
        }
        Ok(clean)
    }
}
