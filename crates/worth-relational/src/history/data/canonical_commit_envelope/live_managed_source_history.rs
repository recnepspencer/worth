//! Disposable live metadata; never canonical serialized truth or Source authority.
use super::{CanonicalCommitAuthorityKind, CanonicalCommitEnvelope};
use crate::validation::candidate_source::authored_history::{
    material::RetainedSourceHistoryMaterial, PreparedSourceImpactHistory,
};
use std::sync::{Arc, Weak};
use worth_execution::ExecutionMemoryReservation;

#[derive(Default)]
pub(crate) struct LiveManagedSourceHistory(Option<Arc<BoundHistory>>);
struct BoundHistory {
    prepared: Arc<PreparedSourceImpactHistory>,
    owner: Weak<CanonicalCommitEnvelope>,
    material: RetainedSourceHistoryMaterial,
    _memory: Option<ExecutionMemoryReservation>,
}
impl Clone for LiveManagedSourceHistory {
    fn clone(&self) -> Self {
        // Only Arc clones of the actual frozen envelope retain its live data.
        // Deep canonical-value cloning must not copy issuance or Weak custody.
        Self::default()
    }
}
impl std::fmt::Debug for LiveManagedSourceHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("LiveManagedSourceHistory")
            .field(&self.0.is_some())
            .finish()
    }
}
impl PartialEq for LiveManagedSourceHistory {
    fn eq(&self, _: &Self) -> bool {
        // Canonical value equality deliberately excludes live acceleration.
        true
    }
}
impl Eq for LiveManagedSourceHistory {}
impl LiveManagedSourceHistory {
    pub(crate) fn verified(
        envelope: &Arc<CanonicalCommitEnvelope>,
    ) -> Option<&PreparedSourceImpactHistory> {
        Some(&Self::bound(envelope)?.prepared)
    }
    pub(crate) fn verified_material(
        envelope: &Arc<CanonicalCommitEnvelope>,
    ) -> Option<&RetainedSourceHistoryMaterial> {
        Some(&Self::bound(envelope)?.material)
    }
    fn bound(envelope: &Arc<CanonicalCommitEnvelope>) -> Option<&BoundHistory> {
        let live = envelope.live_managed_source_history.0.as_ref()?;
        let owner = live.owner.upgrade()?;
        Arc::ptr_eq(&owner, envelope).then_some(live)
    }
}
/// The exact unpublished envelope whose canonical meaning matches the Source packet.
pub(crate) struct SelectedManagedSourceHistory {
    envelope: CanonicalCommitEnvelope,
    packet: Arc<PreparedSourceImpactHistory>,
}

impl CanonicalCommitEnvelope {
    pub(crate) fn select_managed_source_history(
        mut self,
        packet: Option<Arc<PreparedSourceImpactHistory>>,
    ) -> Result<SelectedManagedSourceHistory, Arc<Self>> {
        if let Some(packet) = packet.filter(|packet| packet_matches(packet, &self)) {
            Ok(SelectedManagedSourceHistory {
                envelope: self,
                packet,
            })
        } else {
            self.live_managed_source_history = LiveManagedSourceHistory::default();
            Err(Arc::new(self))
        }
    }
}

impl SelectedManagedSourceHistory {
    pub(crate) fn packet(&self) -> &PreparedSourceImpactHistory {
        &self.packet
    }

    pub(crate) fn freeze(
        self,
        material: RetainedSourceHistoryMaterial,
        memory: Option<ExecutionMemoryReservation>,
    ) -> Arc<CanonicalCommitEnvelope> {
        let Self {
            mut envelope,
            packet,
        } = self;
        Arc::new_cyclic(|owner| {
            envelope.live_managed_source_history =
                LiveManagedSourceHistory(Some(Arc::new(BoundHistory {
                    prepared: packet,
                    owner: owner.clone(),
                    material,
                    _memory: memory,
                })));
            envelope
        })
    }
}

pub(crate) fn backing_bytes() -> Option<u64> {
    let layout = std::alloc::Layout::new::<BoundHistory>()
        .extend(std::alloc::Layout::new::<[usize; 2]>())
        .ok()?
        .0
        .pad_to_align();
    Some(layout.size() as u64)
}

fn packet_matches(
    packet: &PreparedSourceImpactHistory,
    envelope: &CanonicalCommitEnvelope,
) -> bool {
    let parent_matches = match packet.parent {
        None => envelope.commit.parents.is_empty(),
        Some(parent) => envelope.commit.parents.as_slice() == [parent],
    };
    parent_matches
        && packet.branch == envelope.branch_context
        && packet.branch == envelope.commit.branch_id
        && packet.proposed == envelope.commit.version_id
        && envelope.authority_kind == CanonicalCommitAuthorityKind::VersionedTransaction
        && envelope.schema_transition.is_none()
        && envelope.merge_execution_authority.is_none()
        && envelope.merge_parent_branches.is_empty()
        && envelope.merge_base_commits.is_empty()
        && envelope.required_output_materialization.is_none()
        && envelope.required_output_codec_retirements.is_empty()
}
