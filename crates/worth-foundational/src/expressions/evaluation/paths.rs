//! Consumed-path recording.
//!
//! Every held value knows where it came from: an operand path, a constructor
//! over parts with their own origins, or a computation whose reads are
//! already recorded. Field access, binders, and lazy selection pass origins
//! through without reading; an operation records a read only when its result
//! depends on what it inspected. Reads merge per path to the strongest kind.

use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use crate::expressions::denial::ExpressionResult;
use crate::expressions::types::ExpressionType;

use super::meter::EvaluationMeter;
use super::value::{key_order, ExpressionValue, Repr};

pub(super) type PathId = u32;

/// Bytes retained per recorded path node, before any map key it names.
const PATH_NODE_BYTES: u64 = 16;

/// What an evaluation learned about one operand path.
///
/// Kinds are ordered by strength; a stronger read covers a weaker one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ExpressionReadKind {
    /// Whether an optional value, or a map key, is present.
    Presence,
    /// The membership and order of a list's first `n` elements, which
    /// exist: a short-circuited scan examined only that prefix.
    Prefix(u64),
    /// The complete membership and order of a list or a map's key set.
    Length,
    /// The whole value.
    Value,
}

/// One step below an operand.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ExpressionPathStep {
    /// A record field, by index in schema order.
    Field(u32),
    /// A list element, by position.
    Element(u64),
    /// A map entry, by key.
    Key(ExpressionValue),
}

impl ExpressionPathStep {
    fn rank(&self) -> u8 {
        match self {
            Self::Field(_) => 0,
            Self::Element(_) => 1,
            Self::Key(_) => 2,
        }
    }
}

impl Ord for ExpressionPathStep {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self, other) {
            (Self::Field(a), Self::Field(b)) => a.cmp(b),
            (Self::Element(a), Self::Element(b)) => a.cmp(b),
            (Self::Key(a), Self::Key(b)) => key_order(a, b),
            _ => self.rank().cmp(&other.rank()),
        }
    }
}

impl PartialOrd for ExpressionPathStep {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// One consumed operand path and the strongest kind of read it received.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionRead {
    operand: Box<str>,
    path: Box<[ExpressionPathStep]>,
    kind: ExpressionReadKind,
}

impl ExpressionRead {
    pub fn operand(&self) -> &str {
        &self.operand
    }

    pub fn path(&self) -> &[ExpressionPathStep] {
        &self.path
    }

    pub fn kind(&self) -> ExpressionReadKind {
        self.kind
    }
}

/// What one evaluation actually read, sorted by operand then path.
///
/// Descriptive evidence of the reads, not an owner receipt: owners decide
/// what it proves about their snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ExpressionConsumption {
    reads: Box<[ExpressionRead]>,
}

impl ExpressionConsumption {
    pub fn reads(&self) -> &[ExpressionRead] {
        &self.reads
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum Step {
    Operand(u32),
    Field(u32),
    Element(u64),
    Key(ExpressionValue),
}

/// Where a held value came from.
#[derive(Debug, Clone)]
pub(super) enum Origin {
    /// Computed from values whose reads are already recorded.
    Computed,
    Path(PathId),
    /// A constructor's parts, in the value's item order: list and record
    /// literals, `some`, comprehension results, and map entries.
    Parts(Arc<[Origin]>),
}

impl Origin {
    pub(super) fn path(&self) -> Option<PathId> {
        match self {
            Self::Path(path) => Some(*path),
            _ => None,
        }
    }
}

/// A consumed parts list, compared by identity and kept alive so its
/// address is never reused while recorded.
struct Consumed(Arc<[Origin]>);

impl PartialEq for Consumed {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

impl Eq for Consumed {}

impl Hash for Consumed {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).cast::<()>().hash(state);
    }
}

#[derive(Default)]
pub(super) struct Recorder {
    nodes: Vec<(Option<PathId>, Step)>,
    index: HashMap<(Option<PathId>, Step), PathId>,
    kinds: Vec<Option<ExpressionReadKind>>,
    consumed: HashSet<Consumed>,
}

impl std::fmt::Debug for Recorder {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("Recorder")
            .field("paths", &self.nodes.len())
            .finish_non_exhaustive()
    }
}

impl Recorder {
    fn intern(
        &mut self,
        parent: Option<PathId>,
        step: Step,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<PathId> {
        let key = (parent, step);
        if let Some(path) = self.index.get(&key) {
            return Ok(*path);
        }
        let named = match &key.1 {
            Step::Key(value) => value.logical_bytes(),
            _ => 0,
        };
        meter.retain(PATH_NODE_BYTES.saturating_add(named))?;
        let path = self.nodes.len() as PathId;
        self.nodes.push(key.clone());
        self.kinds.push(None);
        self.index.insert(key, path);
        Ok(path)
    }

    pub(super) fn operand(
        &mut self,
        slot: u32,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<PathId> {
        self.intern(None, Step::Operand(slot), meter)
    }

    pub(super) fn field(
        &mut self,
        parent: PathId,
        index: u32,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<PathId> {
        self.intern(Some(parent), Step::Field(index), meter)
    }

    pub(super) fn element(
        &mut self,
        parent: PathId,
        index: u64,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<PathId> {
        self.intern(Some(parent), Step::Element(index), meter)
    }

    /// The path of `key`'s entry. Hashing the key is linear in its bytes, so
    /// callers pay for it before interning.
    pub(super) fn key(
        &mut self,
        parent: PathId,
        key: ExpressionValue,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<PathId> {
        self.intern(Some(parent), Step::Key(key), meter)
    }

    /// Records a read of `kind` at `path`, charging the input it inspected.
    pub(super) fn read(
        &mut self,
        path: PathId,
        kind: ExpressionReadKind,
        value: &ExpressionValue,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<()> {
        let slot = &mut self.kinds[path as usize];
        if *slot >= Some(kind) {
            return Ok(());
        }
        let bytes = match kind {
            ExpressionReadKind::Value => value.logical_bytes(),
            _ => 8,
        };
        meter.input(bytes)?;
        *slot = Some(kind);
        Ok(())
    }

    /// Records a read of `kind` when `origin` is an operand path.
    pub(super) fn read_origin(
        &mut self,
        origin: &Origin,
        kind: ExpressionReadKind,
        value: &ExpressionValue,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<()> {
        match origin.path() {
            Some(path) => self.read(path, kind, value, meter),
            None => Ok(()),
        }
    }

    /// Records that the whole of `value` was used. Each parts list is
    /// consumed once, so repeated uses of one constructed value cost nothing
    /// further.
    pub(super) fn consume(
        &mut self,
        value: &ExpressionValue,
        origin: &Origin,
        meter: &mut EvaluationMeter,
    ) -> ExpressionResult<()> {
        let mut pending = vec![(value.clone(), origin.clone())];
        while let Some((value, origin)) = pending.pop() {
            match origin {
                Origin::Computed => {}
                Origin::Path(path) => self.read(path, ExpressionReadKind::Value, &value, meter)?,
                Origin::Parts(parts) => {
                    if !self.consumed.insert(Consumed(parts.clone())) {
                        continue;
                    }
                    meter.retain(8)?;
                    let items: &[ExpressionValue] = match &value.0 {
                        Repr::Some(inner) => std::slice::from_ref(&**inner),
                        _ => value.items().unwrap_or_default(),
                    };
                    for (item, part) in items.iter().zip(parts.iter()) {
                        pending.push((item.clone(), part.clone()));
                    }
                }
            }
        }
        Ok(())
    }

    /// The recorded reads, named by `slots` and sorted.
    pub(super) fn finish(&self, slots: &[(Box<str>, ExpressionType)]) -> ExpressionConsumption {
        let mut reads: Vec<ExpressionRead> = self
            .kinds
            .iter()
            .enumerate()
            .filter_map(|(path, kind)| Some((path, (*kind)?)))
            .map(|(path, kind)| {
                let mut steps = Vec::new();
                let mut at = Some(path as PathId);
                let mut operand = 0;
                while let Some(node) = at {
                    let (parent, step) = &self.nodes[node as usize];
                    match step {
                        Step::Operand(slot) => operand = *slot,
                        Step::Field(index) => steps.push(ExpressionPathStep::Field(*index)),
                        Step::Element(index) => steps.push(ExpressionPathStep::Element(*index)),
                        Step::Key(key) => steps.push(ExpressionPathStep::Key(key.clone())),
                    }
                    at = *parent;
                }
                steps.reverse();
                ExpressionRead {
                    operand: slots[operand as usize].0.clone(),
                    path: steps.into_boxed_slice(),
                    kind,
                }
            })
            .collect();
        reads.sort();
        ExpressionConsumption {
            reads: reads.into_boxed_slice(),
        }
    }
}
