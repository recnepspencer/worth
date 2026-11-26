//! A correction of a recorded stage, recorded by the document as one step.
use super::super::fixture::envelope::Executed;
use super::super::fixture::history_correction::{correction_envelope, correction_envelope_limited};
use super::*;

/// What one history correction did, for the S2 courts.
#[derive(Debug)]
pub enum Corrected {
    Recorded(u32),
    AlreadyRecorded(u32),
    Refused(String),
    /// The request stopped; nothing recorded.
    Stopped,
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

    /// The commit receipt and corrective binding of `entity`'s creation or
    /// retirement in model step `node`.
    pub fn recorded_source(&self, node: u32, entity: usize) -> (Receipt, Undo) {
        self.sources.get(&(node, entity)).cloned().expect("a recorded creation or retirement")
    }

    /// The document's physical branch as of its last readable selection.
    pub fn branch(&self) -> WorthQueryProductBranch {
        self.branch
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
        self.correct_recorded_with(key, source, entity, None, coupled)
    }

    /// [`Document::correct_recorded`] with `undo` as the correction binding
    /// instead of the one the source's stage declares: a mismatched binding
    /// whose head pass refuses, for the single-stage refusal courts.
    pub fn correct_recorded_as(&mut self, key: u32, source: u32, entity: usize, undo: Undo) -> Corrected {
        self.correct_recorded_with(key, source, entity, Some(undo), &[super::super::reference::Stage::Skip; 2])
    }

    /// [`Document::correct_recorded`] (single stage) whose correction request
    /// walks at most `limit` ancestors.
    pub fn correct_recorded_within(&mut self, key: u32, source: u32, entity: usize, limit: std::num::NonZeroUsize) -> Corrected {
        let key = self.key(key);
        self.correct_recorded_via(key, source, entity, Some(limit), false, None)
    }

    /// Record `e1`'s creation by the `u64`-keyed creation under the numeric
    /// key `key`, as a correction source of [`Self::correct_recorded_numeric`].
    pub fn create_numeric(&mut self, key: u64) -> Outcome {
        let document = self.take();
        let outcome = match super::super::fixture::history_correction::numeric_creation(self.host, document, self.branch, key) {
            Executed::Recorded { document, step, receipts } => {
                self.document = Some(document);
                let node = self.number(&step.id());
                for (entity, receipt) in receipts {
                    self.sources.insert((node, entity), (receipt, Undo::Create(entity)));
                }
                Outcome::Recorded { node }
            }
            Executed::AlreadyRecorded { document, step } => {
                self.document = Some(document);
                Outcome::AlreadyRecorded { node: self.number(&step.id()) }
            }
            Executed::Refused { denial, .. } => panic!("the numeric creation must record: {denial:?}"),
            Executed::Unselectable { .. } => unreachable!("an edit envelope selects no correction source"),
        };
        self.follow();
        outcome
    }

    /// [`Document::correct_recorded`] (single stage) through the correction
    /// whose source and inverse bindings key by `u64`, under the numeric key
    /// `key`.
    pub fn correct_recorded_numeric(&mut self, key: u64, source: u32, entity: usize) -> Corrected {
        self.correct_recorded_via(key.to_string(), source, entity, None, true, None)
    }

    /// [`Self::correct_recorded_numeric`] under a request that stops at
    /// consult `stop`.
    pub fn correct_recorded_numeric_stopping(&mut self, key: u64, source: u32, entity: usize, stop: u32) -> Corrected {
        self.correct_recorded_via(key.to_string(), source, entity, None, true, Some(stop))
    }

    fn correct_recorded_via(
        &mut self,
        key: String,
        source: u32,
        entity: usize,
        limit: Option<std::num::NonZeroUsize>,
        numeric: bool,
        stop: Option<u32>,
    ) -> Corrected {
        let (receipt, undo) = self.sources.get(&(source, entity)).cloned().expect("a recorded creation or retirement");
        let document = self.take();
        let corrected = match correction_envelope_limited(
            self.host, document, self.branch, key, &receipt, undo, &[], stop, limit, numeric,
        ) {
            Executed::Recorded { document, step, .. } => {
                self.document = Some(document);
                let node = self.number(&step.id());
                self.reversed.insert(node, entity);
                Corrected::Recorded(node)
            }
            Executed::AlreadyRecorded { document, step } => {
                self.document = Some(document);
                Corrected::AlreadyRecorded(self.number(&step.id()))
            }
            Executed::Refused { document, denial } => {
                self.document = Some(document);
                match classify(&denial, stop.is_some()) {
                    Some(Outcome::Stopped) => Corrected::Stopped,
                    _ => Corrected::Refused(format!("{denial:?}")),
                }
            }
            Executed::Unselectable { document, denial } => {
                self.document = Some(document);
                Corrected::Refused(denial)
            }
        };
        self.follow();
        corrected
    }

    fn correct_recorded_with(
        &mut self,
        key: u32,
        source: u32,
        entity: usize,
        binding: Option<Undo>,
        coupled: &Stages,
    ) -> Corrected {
        let (receipt, declared) = self
            .sources
            .get(&(source, entity))
            .cloned()
            .unwrap_or_else(|| panic!("k{key}: step {source} has no envelope receipt for entity {entity}"));
        let undo = binding.unwrap_or(declared);
        let before = read_entities(self.host, self.branch).map(|entity| entity.is_some());
        let calls = calls(coupled, before);
        let document = self.take();
        let corrected = match correction_envelope(self.host, document, self.branch, self.key(key), &receipt, undo, &calls, None) {
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
    pub(super) fn correct(&mut self, key: u32, source: u32, entity: usize, coupled: &Stages, stop: Option<u32>) -> Outcome {
        // A source stage with no corrective binding (model `Correctable`):
        // a `set` stage is named and refuses at selection; a step with no
        // stage on `entity` has no source member a caller could name, so
        // the request cannot be made and refuses as the model's does.
        let Some((receipt, undo)) = self
            .sources
            .get(&(source, entity))
            .cloned()
            .or_else(|| self.set_receipts.get(&(source, entity)).map(|receipt| (receipt.clone(), Undo::Retire(entity))))
        else {
            return Outcome::SourceRefused;
        };
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
            stop,
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
                classify(&denial, stop.is_some())
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
                Call::Set(..) => {
                    self.set_receipts.insert((node, entity), receipt);
                    continue;
                }
            };
            self.sources.insert((node, entity), (receipt, undo));
        }
    }
}
