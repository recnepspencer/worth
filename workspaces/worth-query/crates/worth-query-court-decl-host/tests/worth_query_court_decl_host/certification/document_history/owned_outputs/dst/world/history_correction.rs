//! A correction of a recorded stage, recorded by the document as one step.
use super::super::fixture::envelope::Executed;
use super::super::fixture::history_correction::correction_envelope;
use super::*;

/// What one history correction did, for the S2 courts.
#[derive(Debug)]
pub enum Corrected {
    Recorded(u32),
    AlreadyRecorded(u32),
    Refused(String),
}

impl Document<'_> {
    /// `run`'s result, and the history journal's node probes while it ran.
    pub fn node_probes_during<T>(&mut self, run: impl FnOnce(&mut Self) -> T) -> (T, u64) {
        let journal = self
            .document
            .as_mut()
            .expect("the document is held between calls")
            .observe_history_enrollment_for_test();
        let from = journal.snapshot().node_probes;
        let value = run(self);
        (value, journal.snapshot().node_probes - from)
    }

    /// Reverse `entity`'s stage in step `source` through
    /// `document.correction_envelope` under model key `key`, with `coupled`
    /// resolved against the branch's live members after it.
    pub fn correct_recorded(&mut self, key: u32, source: u32, entity: usize, coupled: &Stages) -> Corrected {
        let (receipt, undo) = self
            .sources
            .get(&(source, entity))
            .cloned()
            .unwrap_or_else(|| panic!("k{key}: step {source} has no envelope receipt for entity {entity}"));
        let calls = calls(coupled, read_entities(self.host, self.branch).map(|entity| entity.is_some()));
        let document = self.take();
        let corrected = match correction_envelope(self.host, document, self.branch, self.key(key), &receipt, undo, &calls) {
            Executed::Recorded { document, step, .. } => {
                self.document = Some(document);
                Corrected::Recorded(self.number(&step.id()))
            }
            Executed::AlreadyRecorded { document, step } => {
                self.document = Some(document);
                Corrected::AlreadyRecorded(self.number(&step.id()))
            }
            Executed::Refused { document, denial } => {
                self.document = Some(document);
                Corrected::Refused(format!("{denial:?}"))
            }
        };
        // As after every call: follow the document to its physical branch.
        self.follow();
        corrected
    }
}
