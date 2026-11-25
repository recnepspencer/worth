//! The in-memory reference oracle: `specs/tla/persistent_naming/PersistentNaming.tla`
//! transcribed over names `n1..` with no WORTH code. It holds the canonical
//! log and answers every resolution from it alone; the harness compares each
//! answer with what the installed world returned.
use std::collections::{BTreeMap, BTreeSet};

/// The model's owner of a live record: a model key, or none for an
/// issued-partition record that no model owns.
pub type Owner = Option<&'static str>;

/// The model's owner names (`bodyA`, `bodyB`, or `none`).
pub fn owner_name(owner: Owner) -> &'static str {
    match owner {
        Some("a") => "bodyA",
        Some("b") => "bodyB",
        Some(other) => panic!("unknown owner model `{other}`"),
        None => "none",
    }
}

/// One canonical lineage event, as the model records it.
#[derive(Clone, Debug)]
pub enum Event {
    Create(usize),
    Preserve(usize),
    Split { parent: usize, pieces: [usize; 2] },
    MergeKeep { keeper: usize, other: usize },
    MergeMint { parents: [usize; 2], minted: usize },
    Retire(usize),
    Restore(usize),
}

impl Event {
    fn ends(&self, name: usize) -> bool {
        match self {
            Self::Split { parent, .. } => *parent == name,
            Self::MergeKeep { other, .. } => *other == name,
            Self::MergeMint { parents, .. } => parents.contains(&name),
            Self::Retire(target) => *target == name,
            _ => false,
        }
    }

    fn successors(&self) -> Vec<usize> {
        match self {
            Self::Split { pieces, .. } => pieces.to_vec(),
            Self::MergeKeep { keeper, .. } => vec![*keeper],
            Self::MergeMint { minted, .. } => vec![*minted],
            _ => Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct Holder {
    alive: bool,
    owner: Owner,
}

/// What resolving one held name answers, with targets by name index.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct Answer {
    pub status: &'static str,
    pub targets: BTreeSet<usize>,
}

/// The scopes every ever-minted name is resolved under: unscoped, then
/// within each owner model.
pub const SCOPES: [Option<&str>; 3] = [None, Some("a"), Some("b")];

/// The selections every name is resolved under, `true` for `AllDescendants`.
pub const SELECTIONS: [bool; 2] = [false, true];

/// The reference state: the canonical log and the holders it implies.
#[derive(Clone, Debug, Default)]
pub struct Oracle {
    events: Vec<Event>,
    holders: BTreeMap<usize, Holder>,
}

impl Oracle {
    pub fn events(&self) -> usize {
        self.events.len()
    }

    pub fn is_live(&self, name: usize) -> bool {
        self.holders.get(&name).is_some_and(|holder| holder.alive)
    }

    pub fn is_born(&self, name: usize) -> bool {
        self.holders.contains_key(&name)
    }

    pub fn owner(&self, name: usize) -> Owner {
        self.holders[&name].owner
    }

    pub fn live(&self) -> BTreeSet<usize> {
        let live = self.holders.iter().filter(|(_, holder)| holder.alive);
        live.map(|(name, _)| *name).collect()
    }

    pub fn retired(&self) -> BTreeSet<usize> {
        let ended = self.holders.iter().filter(|(_, holder)| !holder.alive);
        ended.map(|(name, _)| *name).collect()
    }

    pub fn born(&self) -> impl Iterator<Item = usize> + '_ {
        self.holders.keys().copied()
    }

    fn start(&mut self, name: usize, owner: Owner) {
        self.holders.insert(name, Holder { alive: true, owner });
    }

    fn end(&mut self, name: usize) {
        let holder = self.holders.get_mut(&name).expect("only a born name ends");
        assert!(holder.alive, "only a live name ends: n{}", name + 1);
        holder.alive = false;
    }

    /// Appends one committed event and applies it.
    pub fn apply(&mut self, event: Event, owners: &[Owner]) {
        match &event {
            Event::Create(name) => {
                assert!(!self.is_born(*name), "a first birth is new");
                self.start(*name, owners[0]);
            }
            Event::Preserve(name) => {
                assert!(self.is_live(*name), "only a live name regenerates");
                self.start(*name, owners[0]);
            }
            Event::Restore(name) => {
                assert!(self.is_born(*name) && !self.is_live(*name));
                self.start(*name, owners[0]);
            }
            Event::Split { parent, pieces } => {
                self.end(*parent);
                self.start(pieces[0], owners[0]);
                self.start(pieces[1], owners[1]);
            }
            Event::MergeKeep { keeper, other } => {
                assert!(self.is_live(*keeper), "the keeper stays live in place");
                self.end(*other);
            }
            Event::MergeMint { parents, minted } => {
                self.end(parents[0]);
                self.end(parents[1]);
                self.start(*minted, owners[0]);
            }
            Event::Retire(name) => self.end(*name),
        }
        self.events.push(event);
    }

    /// The live names `name` designates: itself when live, else the frontier
    /// of the edges recorded by the event that last ended it.
    fn frontier(&self, name: usize, depth: usize, out: &mut BTreeSet<usize>) {
        if self.is_live(name) {
            out.insert(name);
            return;
        }
        let last_end = self.events.iter().rposition(|event| event.ends(name));
        let Some(index) = last_end.filter(|_| depth > 0) else {
            return;
        };
        for successor in self.events[index].successors() {
            self.frontier(successor, depth - 1, out);
        }
    }

    /// Resolves `name` unscoped (`scope` none) or within one owner model.
    pub fn resolve(&self, name: usize, scope: Option<&str>, all: bool) -> Answer {
        if !self.is_born(name) {
            return answer("UnknownName", BTreeSet::new());
        }
        let mut matched = BTreeSet::new();
        self.frontier(name, self.events.len(), &mut matched);
        let in_scope: BTreeSet<usize> = match scope {
            None => matched.clone(),
            Some(owner) => matched
                .iter()
                .copied()
                .filter(|candidate| self.owner(*candidate) == Some(owner))
                .collect(),
        };
        match (matched.is_empty(), in_scope.len()) {
            (true, _) => answer("Retired", BTreeSet::new()),
            (false, 0) => answer("NoCandidateInOwner", matched),
            (false, 1) => answer("Exact", in_scope),
            (false, _) if all => answer("Plural", in_scope),
            (false, _) => answer("Ambiguous", in_scope),
        }
    }
}

fn answer(status: &'static str, targets: BTreeSet<usize>) -> Answer {
    Answer { status, targets }
}
