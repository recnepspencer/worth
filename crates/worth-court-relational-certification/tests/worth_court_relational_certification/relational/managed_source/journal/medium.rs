//! Byte-only public durable ports; no Native envelopes or live sidecars are retained.
use super::*;
use worth_relational::facade::{
    branch::RelationalBranchTransitionSequence, durability::*, publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause as Cause,
};

#[derive(Debug, Default)]
pub(super) struct Bytes {
    commits: Mutex<BTreeMap<u64, Vec<u8>>>,
    transitions: Mutex<BTreeMap<u64, Vec<u8>>>,
    pub reads: Mutex<Vec<PatchStreamPosition>>,
}
#[derive(Debug)]
pub(super) struct Attachment(pub Arc<Bytes>);
fn store(
    stream: &Mutex<BTreeMap<u64, Vec<u8>>>,
    position: u64,
    bytes: &[u8],
) -> DurablePublicationFate<String> {
    let mut stream = stream.lock().unwrap();
    if stream.get(&position).is_some_and(|old| old == bytes) {
        return DurablePublicationFate::Committed;
    }
    if stream
        .keys()
        .next_back()
        .copied()
        .unwrap_or(0)
        .checked_add(1)
        != Some(position)
    {
        return DurablePublicationFate::ProvenNoEffect {
            detail: "journal requires the next position or identical retry".into(),
        };
    }
    stream.insert(position, bytes.to_vec());
    DurablePublicationFate::Committed
}
impl RelationalDurableMedium for Attachment {
    fn cold_source(&self) -> Arc<dyn RelationalDurableColdSource> {
        self.0.clone()
    }
    fn read_transition(
        &self,
        sequence: RelationalBranchTransitionSequence,
    ) -> Result<RelationalBranchTransitionArtifact, RelationalDurableTransitionReadFailure> {
        let stream = self.0.transitions.lock().unwrap();
        let bytes = stream
            .get(&sequence.0)
            .ok_or(RelationalDurableTransitionReadFailure::Missing(sequence))?;
        RelationalBranchTransitionArtifact::from_untrusted_bytes(bytes.clone())
            .map_err(RelationalDurableTransitionReadFailure::Frame)
    }
    fn publish(
        &self,
        artifact: &RelationalCanonicalCommitArtifact,
    ) -> DurablePublicationFate<String> {
        store(&self.0.commits, artifact.position().0, artifact.as_bytes())
    }
    fn publish_transition(
        &self,
        artifact: &RelationalBranchTransitionArtifact,
    ) -> DurablePublicationFate<String> {
        store(
            &self.0.transitions,
            artifact.sequence().0,
            artifact.as_bytes(),
        )
    }
}
impl RelationalDurableColdSource for Bytes {
    fn retain_members(
        &self,
        members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, Cause> {
        if !members.is_empty() {
            return Err(Cause::SourceUnavailable);
        }
        Ok(RelationalDurableMemberCustody::new(()))
    }
    fn read_member(&self, _: RelationalDurableMemberLocator, _: u64) -> Result<Vec<u8>, Cause> {
        Err(Cause::SourceUnavailable)
    }
    fn read_commit(&self, position: PatchStreamPosition, limit: u64) -> Result<Vec<u8>, Cause> {
        self.reads.lock().unwrap().push(position);
        let stream = self.commits.lock().unwrap();
        let bytes = stream.get(&position.0).ok_or(Cause::SourceUnavailable)?;
        if bytes.len() as u64 > limit {
            return Err(Cause::LimitExceeded {
                length: bytes.len() as u64,
                limit,
            });
        }
        Ok(bytes.clone())
    }
}
