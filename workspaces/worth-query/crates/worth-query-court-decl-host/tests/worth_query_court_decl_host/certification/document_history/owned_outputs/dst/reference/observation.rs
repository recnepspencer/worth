//! What a fresh request reads after each call, and what the model says it
//! must read. The milestone contract (`tree-based-undo.md:139-142`) checks,
//! after every step: live and retired entities, exact full-generation IDs,
//! ownership edges, every output tree, `redo_choices` and paginated
//! `history_page`.
use super::*;

/// A step's label: the history operation of its stage 0.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Label {
    /// The create-or-set binding of an entity.
    Set(usize),
    /// The non-cascading retire binding of an entity.
    Retire(usize),
    /// The inverse that reverses a creation (`retire_created`, with
    /// `co_retire_created` over the owned child).
    UndoCreate(usize),
    /// The inverse that reverses a retirement (`restore_retired`).
    UndoRetire(usize),
}

impl Label {
    /// Stage 0 of an edit: the first entity with a stage in push order.
    pub fn of_edit(stages: &Stages) -> Option<Self> {
        push_order(stages).into_iter().find_map(|e| match stages[e] {
            Stage::Skip => None,
            Stage::Retire => Some(Self::Retire(e)),
            Stage::Set(_) => Some(Self::Set(e)),
        })
    }
}

/// An active reversal fact on the document branch: step `step` reversed
/// `entity` as recorded by `source` (the aftermath-causality fact a
/// correction posts; model `Fact(step, src, ent)`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Fact {
    pub step: u32,
    pub source: u32,
    pub entity: usize,
}

/// One `history_page` row: the step, its parent boundary (`None` is
/// `Root`) and its label. Metadata is always empty in the harness, and the
/// world asserts so when it reads the row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub node: u32,
    pub parent: Option<u32>,
    pub label: Option<Label>,
}

/// `history_page` walked from the first page until `next` is `None`, at
/// `limit` entries per page. Empty pages are not kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct History {
    pub limit: u32,
    pub pages: Vec<Vec<Row>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RedoChoice {
    NoChild,
    Single(u32),
    ChoiceRequired,
}

/// `redo_choices`: the classification, and its children page walked to the
/// end at `limit` entries per page. Empty pages are not kept.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Redo {
    pub choice: RedoChoice,
    pub limit: u32,
    pub pages: Vec<Vec<u32>>,
}

/// What a fresh request reads after the call. The tree fields are `None`
/// when the document cannot be read: diverged, closed, or held by a writer.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observed {
    pub current: Option<u32>,
    pub history: Option<History>,
    pub redo: Option<Redo>,
    /// The document branch's authored rows. `e2`'s ownership edge to `e1` is
    /// part of a live `e2` row; the world fails the seed if a live `e2` has
    /// no live owner, before this comparison runs. `None` only after close
    /// retired the branch, when nothing can read it.
    pub branch: Option<Snapshot>,
    /// Both output trees on the document branch, `None` with `branch`.
    pub outputs: Option<([u64; 2], [u64; 2])>,
    /// What the reads said about the head; `None` while a writer holds the
    /// document or after close, when nothing can read it.
    pub head_diverged: Option<bool>,
    pub closed: bool,
    /// The active reversal facts on the document branch, each attributed to
    /// its step through the fact IDs the journal records with it. Readable
    /// whatever holds the document, like the rows.
    pub reversals: BTreeSet<Fact>,
}

/// `items` in pages of `limit`.
fn pages<T: Clone>(items: &[T], limit: u32) -> Vec<Vec<T>> {
    items.chunks(limit.max(1) as usize).map(<[T]>::to_vec).collect()
}

impl State {
    fn readable(&self) -> bool {
        !self.head_diverged && self.held()
    }

    /// Nothing else holds the document, so a fresh request can try to read it.
    fn held(&self) -> bool {
        !self.closed && self.writer.is_none()
    }

    /// Every step in the journal's order: ascending journal ordinal, which is
    /// record order with the root first (`journal/paging/entries.rs` walks
    /// the retained ordinals; children walk `(parent, ordinal)`). A page's
    /// limit counts examined ordinals, so an uncompleted reservation would
    /// shorten its page; the comparison fails the seed if one ever does.
    fn rows(&self) -> Vec<Row> {
        (0..self.nodes())
            .map(|node| Row {
                node,
                parent: self.parent[node as usize],
                label: self.labels[node as usize],
            })
            .collect()
    }

    /// The current step's children, in record order.
    fn children(&self) -> Vec<u32> {
        (1..self.nodes())
            .filter(|n| self.parent[*n as usize] == Some(self.current))
            .collect()
    }

    fn redo_choice(&self) -> RedoChoice {
        match self.children().as_slice() {
            [] => RedoChoice::NoChild,
            [child] => RedoChoice::Single(*child),
            _ => RedoChoice::ChoiceRequired,
        }
    }

    /// What this state reads as, paged at `limit`.
    pub fn observed(&self, limit: u32) -> Observed {
        let readable = self.readable();
        Observed {
            current: readable.then_some(self.current),
            history: readable.then(|| History {
                limit,
                pages: pages(&self.rows(), limit),
            }),
            redo: readable.then(|| Redo {
                choice: self.redo_choice(),
                limit,
                pages: pages(&self.children(), limit),
            }),
            branch: Some(self.branch.clone()),
            outputs: Some((self.branch.outputs_a(), self.branch.outputs_b())),
            head_diverged: self.held().then_some(self.head_diverged),
            closed: self.closed,
            reversals: self.facts.clone(),
        }
    }

    pub(super) fn matches(&self, observed: &Observed) -> bool {
        let tree = match (&observed.current, &observed.history, &observed.redo) {
            (None, None, None) => !self.readable(),
            (Some(current), Some(history), Some(redo)) => {
                self.readable()
                    && *current == self.current
                    && history.pages == pages(&self.rows(), history.limit)
                    && redo.choice == self.redo_choice()
                    && redo.pages == pages(&self.children(), redo.limit)
            }
            _ => false,
        };
        let rows = match (&observed.branch, &observed.outputs) {
            (Some(branch), Some(outputs)) => {
                *branch == self.branch && *outputs == (self.branch.outputs_a(), self.branch.outputs_b())
            }
            (None, None) => self.closed,
            _ => false,
        };
        tree && rows
            && observed.head_diverged == self.held().then_some(self.head_diverged)
            && observed.closed == self.closed
            && observed.reversals == self.facts
    }
}
