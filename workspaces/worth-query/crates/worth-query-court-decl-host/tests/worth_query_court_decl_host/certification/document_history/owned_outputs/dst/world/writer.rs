//! Writers that hold the move-only document (model `WriterBegin`,
//! `WriterCommit`, `WriterDiscard`).
use super::super::fixture::writers::{commit, discard};
use super::super::reference::Writer;
use super::*;

impl Document<'_> {
    /// The writer takes the document; it starts at commit or discard.
    pub(super) fn begin(&mut self, class: Writer) -> Outcome {
        let document = self.take();
        self.writer = Some(Held { class, document });
        Outcome::Begun
    }

    /// Record the writer's step under `key` with `stages`.
    pub(super) fn commit(&mut self, key: u32, stages: &Stages) -> Outcome {
        let held = self.writer.take().expect("Commit resolves only with a writer open");
        // A writer's key is not an envelope's: the calls resolve at the head,
        // and an envelope that later names the key resolves its own
        // (refinement (v); traced, seed 0x7b020035).
        let calls = calls(stages, read_entities(self.host, self.branch).map(|entity| entity.is_some()));
        let gesture = (u128::from(self.id) << 64) | u128::from(key);
        let (document, step) = commit(self.host, held, self.key(key), gesture, &calls);
        self.document = Some(document);
        Outcome::Recorded { node: self.number(&step.id()) }
    }

    /// Give the writer up: the document comes back unchanged.
    pub(super) fn discard(&mut self) -> Outcome {
        let held = self.writer.take().expect("Discard resolves only with a writer open");
        self.navigation += 1;
        let gesture = (u128::from(self.id) << 64) | (1 << 63) | self.navigation;
        self.document = Some(discard(self.host, held, gesture));
        Outcome::Discarded
    }
}
