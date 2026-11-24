//! The real document as a DST system: each oracle action goes through the
//! public entry point it names, each return is classified into the model's
//! outcome, and each observation is read with fresh requests.
mod observation;
mod writer;

use super::super::*;
use super::fixture::envelope::{self, correct_outside, write_outside, Executed, Push, Receipt, Undo};
use super::fixture::reads::{read_entities, with_request, Forest, Host};
use super::fixture::refusals::{classify, classify_navigation, classify_read, provoke};
use super::fixture::writers::Held;
use super::fixture::program::{self, Shape};
use super::fixture::{calls, Call};
use super::reference::{Action, Environment, Intent, Outcome, Stages, NO_STAGES};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock;
use worth_query_host::facade::product::WorthQueryProductBranch;

/// The environment refusals the owned document can meet. `ForeignModel`
/// needs a document attached to another model's head, `Budget` an exhausted
/// lease the public surface does not give, and `UnsupportedCorrection` the
/// correction envelope (S2).
pub const ENVIRONMENT: [Environment; 3] = [
    Environment::MetadataTooLarge,
    Environment::UnownedOperationScope,
    Environment::OperationNotDeclared,
];

/// Documents attached so far on this process's hosts.
static DOCUMENTS: AtomicU64 = AtomicU64::new(0);

/// A navigation, to a model node.
#[derive(Clone, Copy, Debug)]
enum Move {
    Undo,
    Redo(u32),
    Checkout(u32),
}

/// Navigation ends well inside this many advances.
const ADVANCES: usize = 512;

/// Install the owned world with the DST's program, model `b` beside it, and
/// settle the DST's tree on the world branch.
pub fn install() -> Host {
    install_as(Shape::Members)
}

/// [`install`] settling the tree with the `dst-use` consumer, as public
/// #358's trace did; only its held regression uses it.
pub fn install_consumer() -> Host {
    install_as(Shape::WithConsumer)
}

fn install_as(shape: Shape) -> Host {
    let host: Host = super::super::world::install_program_with_other_model(
        host_limits(WorthQueryAdmissionClock::system()),
        1,
        Some(1),
    );
    with_request(&host, |request| envelope::settle(&host, request, shape));
    host
}

/// One seed's document on the installed world.
pub struct Document<'h> {
    host: &'h Host,
    /// The tree the document binds and each write outside settles.
    shape: Shape,
    /// `None` while a writer holds the move-only document, and after close.
    document: Option<Forest<'h>>,
    writer: Option<Held<'h>>,
    /// The document's physical branch as of its last readable selection: a
    /// move forks a new one and retires the old (traced, seed 0x7b020000).
    branch: WorthQueryProductBranch,
    /// Model node → real step id, the root at 0.
    nodes: Vec<WorthQueryHistoryNodeId>,
    /// Real full-generation ID → dense first-seen number.
    generations: BTreeMap<worth_relational::facade::identity::EntityId, u32>,
    /// Each envelope key's stages and the calls they resolved to: a replay
    /// with the same stages makes the same calls, so its intent matches. A
    /// stopped attempt pins nothing; its retry resolves at the head it runs
    /// on, as the model's does (traced, seed 0x7b02004e). Writers' keys are
    /// another namespace and are never kept here (refinement (v)).
    keyed: BTreeMap<u32, (Stages, Vec<Call>)>,
    /// Each recorded creation or retirement, for today's correction route.
    sources: BTreeMap<(u32, usize), (Receipt, Undo)>,
    /// This document's number on the host: every key it uses carries it, so
    /// the seeds sharing one host never meet each other's keys.
    id: u64,
    navigation: u128,
    head_diverged: bool,
    closed: bool,
}

impl<'h> Document<'h> {
    /// Fork a fresh owned head and attach the history with the DST's tree.
    pub fn attach(host: &'h Host) -> Self {
        Self::attach_as(host, Shape::Members)
    }

    /// [`Document::attach`] binding the tree with the `dst-use` consumer;
    /// only the regression held on public #358 uses it.
    pub fn attach_consumer(host: &'h Host) -> Self {
        Self::attach_as(host, Shape::WithConsumer)
    }

    fn attach_as(host: &'h Host, shape: Shape) -> Self {
        let document = super::super::world::attach(host);
        let document = with_request(host, |request| {
            host.with_host_advancement(|phase| {
                let demand = upstream::Demand("a".into());
                let controls = WorthQueryOutputDemandControls::default();
                match shape {
                    Shape::Members => document
                        .with_output_trees::<program::Program, program::Tree>(&phase, request, host, demand, controls),
                    Shape::WithConsumer => document.with_output_trees::<program::Program, program::ConsumerTree>(
                        &phase, request, host, demand, controls,
                    ),
                }
                .unwrap_or_else(|(_, cause)| panic!("the DST tree must bind: {cause:?}"))
            })
            .unwrap()
        });
        let (root, branch) = with_request(host, |request| {
            let root = document.current_step(request).unwrap().id();
            let selection = document.selection(request).unwrap();
            (root, selection.physical_endpoint().product_branch())
        });
        Self {
            host,
            shape,
            document: Some(document),
            writer: None,
            branch,
            nodes: vec![root],
            generations: BTreeMap::new(),
            keyed: BTreeMap::new(),
            sources: BTreeMap::new(),
            id: DOCUMENTS.fetch_add(1, Ordering::Relaxed),
            navigation: 0,
            head_diverged: false,
            closed: false,
        }
    }

    /// The idempotency key of model key `key`.
    fn key(&self, key: u32) -> String {
        format!("d{}:k{key}", self.id)
    }

    fn take(&mut self) -> Forest<'h> {
        self.document.take().expect("the document is held between calls")
    }

    /// Follow the document to its current physical branch. A diverged
    /// selection refuses, and the branch it diverged on stays the one read.
    fn follow(&mut self) {
        let held = self.document.as_ref().or(self.writer.as_ref().map(|held| &held.document));
        let Some(document) = held.filter(|_| !self.closed) else { return };
        if let Ok(selection) = with_request(self.host, |request| document.selection(request)) {
            self.branch = selection.physical_endpoint().product_branch();
        }
    }

    /// The model number of a real step, numbering a new one in order.
    fn number(&mut self, step: &WorthQueryHistoryNodeId) -> u32 {
        match self.nodes.iter().position(|id| id == step) {
            Some(index) => index as u32,
            None => {
                self.nodes.push(step.clone());
                self.nodes.len() as u32 - 1
            }
        }
    }

    /// The calls `stages` make under `key`: the first ones for its key when
    /// the stages match, else resolved against the branch's live members.
    fn calls(&mut self, key: u32, stages: &Stages) -> Vec<Call> {
        if let Some((first, calls)) = self.keyed.get(&key) {
            if first == stages {
                return calls.clone();
            }
        }
        calls(stages, read_entities(self.host, self.branch).map(|entity| entity.is_some()))
    }
}

impl super::driver::System for Document<'_> {
    fn act(&mut self, action: &Action) -> Outcome {
        let outcome = match action {
            Action::Envelope { key, intent: Intent::Edit(stages), stop } => self.envelope(*key, stages, *stop),
            Action::Envelope { key, intent: Intent::Correction { source, entity, stages }, .. } => {
                self.correct(*key, *source, *entity, stages)
            }
            Action::Refused { key, cause } => {
                let document = self.take();
                let (document, outcome) = provoke(self.host, document, self.branch, self.key(*key), *cause);
                self.document = Some(document);
                outcome
            }
            Action::Empty { key } => self.envelope(*key, &NO_STAGES, None),
            Action::Undo => self.navigate(Move::Undo),
            Action::Redo(node) => self.navigate(Move::Redo(*node)),
            Action::Checkout(node) => self.navigate(Move::Checkout(*node)),
            Action::OutOfBand(stages) => {
                let calls = calls(stages, read_entities(self.host, self.branch).map(|entity| entity.is_some()));
                self.navigation += 1;
                write_outside(self.host, self.shape, self.branch, format!("d{}:outside:{}", self.id, self.navigation), &calls);
                Outcome::OutOfBand
            }
            Action::Begin(class) => self.begin(*class),
            Action::Commit { key, stages } => self.commit(*key, stages),
            Action::Discard => self.discard(),
            Action::Maintain => self.maintain(),
            Action::Close => self.close(),
        };
        if outcome == Outcome::HeadDiverged {
            self.head_diverged = true;
        }
        self.follow();
        outcome
    }

    fn observe(&mut self, limit: u32) -> super::reference::Observed {
        self.read(limit)
    }

    fn nodes(&self) -> u32 {
        self.nodes.len() as u32
    }

    fn release(&mut self) {
        if self.writer.is_some() {
            self.discard();
        }
        if self.document.is_some() && !self.closed {
            self.close();
        }
    }
}

impl Document<'_> {
    fn envelope(&mut self, key: u32, stages: &Stages, stop: Option<u32>) -> Outcome {
        let calls = self.calls(key, stages);
        let pushes: Vec<Push> = calls.iter().copied().map(Push::Dst).collect();
        let document = self.take();
        match envelope::envelope(self.host, document, self.branch, self.key(key), &pushes, Vec::new(), stop) {
            Executed::Recorded { document, step, receipts } => {
                self.document = Some(document);
                self.keyed.entry(key).or_insert_with(|| (*stages, calls.clone()));
                let node = self.number(&step.id());
                for ((entity, receipt), call) in receipts.into_iter().zip(&calls) {
                    let undo = match call {
                        Call::Create(..) => Undo::Create(entity),
                        Call::Retire(_) => Undo::Retire(entity),
                        Call::Set(..) => continue,
                    };
                    self.sources.insert((node, entity), (receipt, undo));
                }
                Outcome::Recorded { node }
            }
            Executed::AlreadyRecorded { document, step } => {
                self.document = Some(document);
                Outcome::AlreadyRecorded { node: self.number(&step.id()) }
            }
            Executed::Refused { document, denial } => {
                self.document = Some(document);
                classify(&denial, stop.is_some())
                    .unwrap_or_else(|| panic!("k{key}: a refusal the model does not name: {denial:?}"))
            }
        }
    }

    /// Today's correction route: the source stage is reversed on the
    /// document branch outside the document, with the coupled stages after
    /// it. The model records a step; the real document sees a write outside,
    /// or the inverse binding's refusal.
    fn correct(&mut self, key: u32, source: u32, entity: usize, coupled: &Stages) -> Outcome {
        let (receipt, undo) = self
            .sources
            .get(&(source, entity))
            .cloned()
            .unwrap_or_else(|| panic!("k{key}: step {source} has no envelope receipt for entity {entity}"));
        if !correct_outside(self.host, self.shape, self.branch, self.key(key), &receipt, undo) {
            return Outcome::StageRefused;
        }
        if *coupled != NO_STAGES {
            let calls = calls(coupled, read_entities(self.host, self.branch).map(|entity| entity.is_some()));
            write_outside(self.host, self.shape, self.branch, format!("{}:coupled", self.key(key)), &calls);
        }
        Outcome::OutOfBand
    }

    /// Prepare the move from the current selection and advance it to its end.
    fn navigate(&mut self, to: Move) -> Outcome {
        self.navigation += 1;
        let idempotency = (u128::from(self.id) << 64) | self.navigation;
        let nodes = &self.nodes;
        let document = self.document.as_ref().expect("the document is held between calls");
        let refused = |denial: &WorthQueryHistoryNavigationAdvanceDenial| {
            classify_navigation(denial)
                .unwrap_or_else(|| panic!("{to:?}: a navigation refusal the model does not name: {denial:?}"))
        };
        with_request(self.host, |request| {
            let expected = match document.selection(request) {
                Ok(selection) => selection,
                Err(denial) => return refused(&denial),
            };
            let builder = document.navigate::<navigation::Navigation>();
            let target = match to {
                Move::Undo => builder.undo_step(),
                Move::Redo(node) => builder.redo_to(nodes[node as usize].clone()),
                Move::Checkout(node) => builder.checkout(nodes[node as usize].clone()),
            };
            let prepared = target
                .expected_selection(expected)
                .idempotency(idempotency)
                .prepare(request);
            let mut navigation = match prepared {
                Ok(navigation) => navigation,
                Err(denial) => return refused(&denial),
            };
            for _ in 0..ADVANCES {
                navigation = match navigation.advance(request) {
                    WorthQueryHistoryNavigationAdvance::Pending(next) => next,
                    WorthQueryHistoryNavigationAdvance::Selected(_) => return Outcome::Moved,
                    // A refused navigation's owner is discarded, as the
                    // Phase 1 navigation courts do.
                    WorthQueryHistoryNavigationAdvance::Refused { cause, owner } => {
                        owner
                            .discard(request)
                            .unwrap_or_else(|(_, denial)| panic!("{to:?}: a refused navigation must discard: {denial:?}"));
                        return refused(&cause);
                    }
                };
            }
            panic!("navigation must end within {ADVANCES} advances")
        })
    }

    fn maintain(&mut self) -> Outcome {
        let document = self.document.as_mut().expect("the document is held between calls");
        with_request(self.host, |request| match document.maintain_history(request) {
            Ok(WorthQueryHistoryReleaseOutcome::NoPendingRelease) => Outcome::NoPendingRelease,
            Ok(WorthQueryHistoryReleaseOutcome::Closed) => Outcome::Released,
            Ok(other) => panic!("a maintenance outcome the model does not name: {other:?}"),
            Err(denial) => classify_read(&denial)
                .unwrap_or_else(|| panic!("a maintenance refusal the model does not name: {denial:?}")),
        })
    }

    fn close(&mut self) -> Outcome {
        let document = self.take();
        // A kept commit receipt holds the history it committed on: across
        // close it defers the branch's release (traced, S1 probe).
        self.sources.clear();
        self.host.with_host_advancement(|phase| document.close(&phase).unwrap()).unwrap();
        assert!(self.host.runtime().branches().pending_cleanup().is_empty());
        self.closed = true;
        Outcome::Closed
    }
}
