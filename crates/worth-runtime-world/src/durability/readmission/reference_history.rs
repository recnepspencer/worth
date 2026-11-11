//! Exact branch references are indexed independently of their shared commit node.
use super::{WorldCheckpointImage, WorldReadmissionDenial as Denial};
use crate::durability::{checkpoint::WorldImageHead, DurableWorldHead};
use worth_execution::ExecutionRequest;

impl WorldCheckpointImage {
    pub(in crate::durability) fn remember_reference(
        &mut self,
        head: &WorldImageHead,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<(), Denial> {
        let (_, incarnation, _, generation) = head.fields();
        let key = (incarnation, generation);
        if self.references.contains_key(&key) {
            return Err(Denial::DuplicateIdentity);
        }
        super::journal::insert(&mut self.references, &key, head, request)
    }

    pub(in crate::durability) fn recorded_reference(
        &self,
        expected: &DurableWorldHead,
    ) -> Option<&WorldImageHead> {
        self.references
            .get(&(expected.incarnation, expected.generation))
            .filter(|head| head.matches(expected))
    }
}
