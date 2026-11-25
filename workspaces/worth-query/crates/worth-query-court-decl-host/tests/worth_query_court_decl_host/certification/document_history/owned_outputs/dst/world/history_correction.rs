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
    /// Rewrite the reversal fact model step `step` posted in an ordinary
    /// commit outside any navigation, then read it back there: `Err` carries
    /// the canonical read's refusal (ruling (u)'s read-trust negative court).
    pub fn rewrite_posted_fact(&mut self, step: u32) -> Result<(), String> {
        let id = self.nodes[step as usize].clone();
        let document = self.document.as_ref().expect("the document is held between calls");
        self.host
            .with_host_advancement(|phase| document.rewrite_posted_fact_for_test(&phase, &id))
            .expect("the host advances")
    }

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
        let before = read_entities(self.host, self.branch).map(|entity| entity.is_some());
        let calls = calls(coupled, before);
        let document = self.take();
        let corrected = match correction_envelope(self.host, document, self.branch, self.key(key), &receipt, undo, &calls) {
            Executed::Recorded { document, step, receipts } => {
                self.document = Some(document);
                let node = self.number(&step.id());
                self.reversed.insert(node, entity);
                self.record_correction_sources(node, undo, before, &calls, receipts);
                Corrected::Recorded(node)
            }
            Executed::AlreadyRecorded { document, step } => {
                self.document = Some(document);
                Corrected::AlreadyRecorded(self.number(&step.id()))
            }
            Executed::Refused { document, denial } => {
                self.document = Some(document);
                Corrected::Refused(format!("{denial:?}"))
            }
            Executed::Unselectable { document, denial } => {
                self.document = Some(document);
                Corrected::Refused(denial)
            }
        };
        // As after every call: follow the document to its physical branch.
        self.follow();
        corrected
    }
}

impl Document<'_> {
    /// The correction envelope (S2): the source stage is reversed as the
    /// document's next step, with the coupled stages after it, through
    /// `document.correction_envelope`.
    pub(super) fn correct(&mut self, key: u32, source: u32, entity: usize, coupled: &Stages) -> Outcome {
        // A step that created or retired nothing on `entity` has no
        // corrective binding there: name one of its stages and ask to restore
        // a retirement it never made, which the derivation refuses.
        let (receipt, undo) = self
            .sources
            .get(&(source, entity))
            .cloned()
            .or_else(|| self.step_receipts.get(&source).map(|receipt| (receipt.clone(), Undo::Retire(entity))))
            .unwrap_or_else(|| panic!("k{key}: step {source} has no envelope receipt for entity {entity}"));
        let before = read_entities(self.host, self.branch).map(|entity| entity.is_some());
        let calls = calls(coupled, before);
        let document = self.take();
        let corrected = correction_envelope(
            self.host,
            document,
            self.branch,
            self.key(key),
            &receipt,
            undo,
            &calls,
        );
        match corrected {
            Executed::Recorded { document, step, receipts } => {
                self.document = Some(document);
                let node = self.number(&step.id());
                self.reversed.insert(node, entity);
                self.record_correction_sources(node, undo, before, &calls, receipts);
                Outcome::Recorded { node }
            }
            Executed::AlreadyRecorded { document, step } => {
                self.document = Some(document);
                Outcome::AlreadyRecorded { node: self.number(&step.id()) }
            }
            // The source has no correction meaning: `MeaningDerivable` fails.
            Executed::Unselectable { document, denial } => {
                self.document = Some(document);
                assert!(denial.contains("SourceNotCorrectable"), "k{key}: a selection refusal the model does not name: {denial}");
                Outcome::SourceRefused
            }
            Executed::Refused { document, denial } => {
                self.document = Some(document);
                classify(&denial, false)
                    .unwrap_or_else(|| panic!("k{key}: a correction refusal the model does not name: {denial:?}"))
            }
        }
    }

    /// A recorded correction step is itself a correction source where it
    /// retired something (model `ops`: `Retired`): reversing a creation
    /// retires the entity, and `e1`'s co-retired `e2` with it, in the
    /// corrective (primary) stage; a coupled creation or retirement is its
    /// own stage. A restoring correction records `Restored`, which no
    /// correction reverses.
    fn record_correction_sources(
        &mut self,
        node: u32,
        undo: Undo,
        before: [bool; 2],
        calls: &[Call],
        receipts: Vec<(usize, Receipt)>,
    ) {
        let mut receipts = receipts.into_iter();
        let (_, primary) = receipts.next().expect("a recorded correction has its corrective receipt");
        self.step_receipts.insert(node, primary.clone());
        if let Undo::Create(entity) = undo {
            self.sources.insert((node, entity), (primary.clone(), Undo::Retire(entity)));
            let after = read_entities(self.host, self.branch).map(|entity| entity.is_some());
            let coupled_e2 = calls.iter().any(|call| call.entity() == super::super::reference::E2);
            if entity == super::super::reference::E1 && before[super::super::reference::E2] && !after[super::super::reference::E2] && !coupled_e2 {
                let e2 = super::super::reference::E2;
                self.sources.insert((node, e2), (primary, Undo::Retire(e2)));
            }
        }
        for ((entity, receipt), call) in receipts.zip(calls) {
            let undo = match call {
                Call::Create(..) => Undo::Create(entity),
                Call::Retire(_) => Undo::Retire(entity),
                Call::Set(..) => continue,
            };
            self.sources.insert((node, entity), (receipt, undo));
        }
    }
}
