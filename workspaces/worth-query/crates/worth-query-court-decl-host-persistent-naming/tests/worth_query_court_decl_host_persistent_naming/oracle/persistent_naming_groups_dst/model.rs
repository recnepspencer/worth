//! The reference model of `Anchor` group continuity, transcribed from
//! `specs/tla/persistent_naming/PersistentNamingGroups.tla`, and the seeded
//! choice of one action it enables.
//!
//! Members are `m1..mN` by index; groups are labelled `g1..` in the order
//! the model first names them. A group's name is a function of its birth
//! anchor and the prior groups it was split from (`sources`), so re-forming
//! the same `(anchor, sources)` revives that exact name. Member names order
//! exactly as the installed world's do (`rank`), so the model predicts every
//! anchor and keeper.
use std::collections::{BTreeMap, BTreeSet};

use worth_dst::Rng;

/// Group labels the trace model declares.
pub const MAX_GROUPS: usize = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Group {
    pub live: bool,
    pub members: BTreeSet<usize>,
    pub anchor: usize,
    pub sources: BTreeSet<usize>,
}

type Members = BTreeSet<usize>;

#[derive(Clone, Debug)]
pub enum Op {
    Born {
        g: usize,
        members: Members,
        anchor: usize,
    },
    Revive {
        g: usize,
        members: Members,
    },
    Split {
        g: usize,
        piece: Members,
        anchor: usize,
        n: usize,
    },
    SplitAway {
        g: usize,
        parts: [Members; 2],
        anchors: [usize; 2],
        news: [usize; 2],
    },
    Merge {
        keep: usize,
        absorb: usize,
    },
    Dissolve {
        g: usize,
    },
    DeleteFree {
        m: usize,
    },
    DeleteMember {
        g: usize,
        m: usize,
    },
    DeleteAnchor {
        g: usize,
        successor: Option<(usize, usize)>,
    },
    RefuseOverlap {
        m: usize,
    },
}

/// One resolution answer: `exact`, `plural`, `ambiguous`, `retired`.
pub type Answer = (&'static str, BTreeSet<usize>);

pub struct Model {
    pub live_member: Vec<bool>,
    pub groups: BTreeMap<usize, Group>,
    pub succ: BTreeMap<usize, BTreeSet<usize>>,
    /// Each member's name: anchors and keepers are least by name.
    pub rank: Vec<[u8; 32]>,
}

impl Model {
    pub fn new(rank: Vec<[u8; 32]>) -> Self {
        Self {
            live_member: vec![true; rank.len()],
            groups: BTreeMap::new(),
            succ: BTreeMap::new(),
            rank,
        }
    }

    pub fn live(&self) -> impl Iterator<Item = (&usize, &Group)> {
        self.groups.iter().filter(|(_, group)| group.live)
    }

    pub fn owner(&self, m: usize) -> Option<usize> {
        self.live()
            .find(|(_, group)| group.members.contains(&m))
            .map(|(g, _)| *g)
    }

    fn free(&self) -> Vec<usize> {
        (0..self.rank.len())
            .filter(|m| self.live_member[*m] && self.owner(*m).is_none())
            .collect()
    }

    fn least(&self, members: &Members) -> usize {
        *members
            .iter()
            .min_by_key(|m| self.rank[**m])
            .expect("nonempty")
    }

    /// The label `(anchor, sources)` names: the group already holding that
    /// name, or the next fresh label after `skip` fresh ones.
    fn mint(&self, anchor: usize, sources: &BTreeSet<usize>, skip: usize) -> usize {
        let held = self
            .groups
            .iter()
            .find(|(_, g)| g.anchor == anchor && g.sources == *sources);
        held.map_or(self.groups.len() + skip, |(label, _)| *label)
    }

    fn rest(&self, g: usize) -> Vec<usize> {
        let group = &self.groups[&g];
        group
            .members
            .iter()
            .copied()
            .filter(|m| *m != group.anchor)
            .collect()
    }

    /// One action the model enables, chosen by `rng`; `None` once every
    /// member is deleted and no loop is live.
    pub fn next_op(&self, rng: &mut Rng) -> Option<Op> {
        let room = self.groups.len() + 2 <= MAX_GROUPS;
        let free = self.free();
        let live: Vec<usize> = self.live().map(|(g, _)| *g).collect();
        for _ in 0..32 {
            match rng.below(9) {
                0 | 1 if !free.is_empty() && room => {
                    let anchor = rng.pick(&free);
                    let members = free
                        .iter()
                        .copied()
                        .filter(|m| *m == anchor || rng.below(2) == 0)
                        .collect();
                    let g = self.mint(anchor, &BTreeSet::new(), 0);
                    return Some(match self.groups.get(&g) {
                        Some(_) => Op::Revive { g, members },
                        None => Op::Born { g, members, anchor },
                    });
                }
                2 if !live.is_empty() && room => {
                    let g = rng.pick(&live);
                    let rest = self.rest(g);
                    if !rest.is_empty() {
                        let anchor = rng.pick(&rest);
                        let piece = rest
                            .iter()
                            .copied()
                            .filter(|m| *m == anchor || rng.below(2) == 0)
                            .collect();
                        let n = self.mint(anchor, &BTreeSet::from([g]), 0);
                        return Some(Op::Split {
                            g,
                            piece,
                            anchor,
                            n,
                        });
                    }
                }
                3 if !live.is_empty() && room => {
                    let g = rng.pick(&live);
                    let rest = self.rest(g);
                    if rest.len() >= 2 {
                        let split = 1 + rng.below(rest.len() - 1);
                        let parts: [Members; 2] = [
                            rest[..split].iter().copied().collect(),
                            rest[split..].iter().copied().collect(),
                        ];
                        let anchors = [self.least(&parts[0]), self.least(&parts[1])];
                        let sources = BTreeSet::from([g]);
                        let first = self.mint(anchors[0], &sources, 0);
                        let fresh = usize::from(first >= self.groups.len());
                        let news = [first, self.mint(anchors[1], &sources, fresh)];
                        return Some(Op::SplitAway {
                            g,
                            parts,
                            anchors,
                            news,
                        });
                    }
                }
                4 if live.len() >= 2 => {
                    let (a, b) = (rng.pick(&live), rng.pick(&live));
                    if a != b {
                        let rank = |g: usize| self.rank[self.groups[&g].anchor];
                        let (keep, absorb) = if rank(a) < rank(b) { (a, b) } else { (b, a) };
                        return Some(Op::Merge { keep, absorb });
                    }
                }
                5 if !live.is_empty() => return Some(Op::Dissolve { g: rng.pick(&live) }),
                6 if !free.is_empty() => {
                    let m = rng.pick(&free);
                    return Some(if rng.below(3) == 0 {
                        Op::DeleteFree { m }
                    } else {
                        Op::RefuseOverlap { m }
                    });
                }
                7 if !live.is_empty() => {
                    let g = rng.pick(&live);
                    let rest = self.rest(g);
                    if !rest.is_empty() {
                        return Some(Op::DeleteMember {
                            g,
                            m: rng.pick(&rest),
                        });
                    }
                }
                8 if !live.is_empty() && room => {
                    let g = rng.pick(&live);
                    let rest: Members = self.rest(g).into_iter().collect();
                    let successor = (!rest.is_empty()).then(|| {
                        let anchor = self.least(&rest);
                        (self.mint(anchor, &BTreeSet::from([g]), 0), anchor)
                    });
                    return Some(Op::DeleteAnchor { g, successor });
                }
                _ => {}
            }
        }
        match (free.first(), live.first()) {
            (Some(&m), _) => Some(Op::RefuseOverlap { m }),
            (None, Some(&g)) => Some(Op::Dissolve { g }),
            (None, None) => None,
        }
    }

    /// Starts (or revives) `label` as a live group.
    fn start(&mut self, label: usize, members: Members, anchor: usize, sources: BTreeSet<usize>) {
        self.succ.remove(&label);
        self.groups.insert(
            label,
            Group {
                live: true,
                members,
                anchor,
                sources,
            },
        );
    }

    /// Applies a committed action.
    pub fn apply(&mut self, op: &Op) {
        match op {
            Op::Born { g, members, anchor } => {
                self.start(*g, members.clone(), *anchor, BTreeSet::new())
            }
            Op::Revive { g, members } => {
                let anchor = self.groups[g].anchor;
                self.start(*g, members.clone(), anchor, BTreeSet::new());
            }
            Op::Split {
                g,
                piece,
                anchor,
                n,
            } => {
                self.groups
                    .get_mut(g)
                    .expect("split group")
                    .members
                    .retain(|m| !piece.contains(m));
                self.start(*n, piece.clone(), *anchor, BTreeSet::from([*g]));
            }
            Op::SplitAway {
                g,
                parts,
                anchors,
                news,
            } => {
                let anchor = self.groups[g].anchor;
                self.live_member[anchor] = false;
                self.retire(*g, news.iter().copied().collect());
                for i in 0..2 {
                    self.start(news[i], parts[i].clone(), anchors[i], BTreeSet::from([*g]));
                }
            }
            Op::Merge { keep, absorb } => {
                let moved = self.groups[absorb].members.clone();
                self.groups
                    .get_mut(keep)
                    .expect("keeper")
                    .members
                    .extend(moved);
                self.retire(*absorb, BTreeSet::from([*keep]));
            }
            Op::Dissolve { g } => self.retire(*g, BTreeSet::new()),
            Op::DeleteFree { m } => self.live_member[*m] = false,
            Op::DeleteMember { g, m } => {
                self.live_member[*m] = false;
                self.groups.get_mut(g).expect("shrunk").members.remove(m);
            }
            Op::DeleteAnchor { g, successor } => {
                let group = self.groups[g].clone();
                self.live_member[group.anchor] = false;
                let rest = group
                    .members
                    .iter()
                    .copied()
                    .filter(|m| *m != group.anchor)
                    .collect();
                match successor {
                    Some((n, anchor)) => {
                        self.retire(*g, BTreeSet::from([*n]));
                        self.start(*n, rest, *anchor, BTreeSet::from([*g]));
                    }
                    None => self.retire(*g, BTreeSet::new()),
                }
            }
            Op::RefuseOverlap { .. } => {}
        }
    }

    fn retire(&mut self, g: usize, successors: BTreeSet<usize>) {
        let group = self.groups.get_mut(&g).expect("retired group");
        group.live = false;
        group.members.clear();
        self.succ.insert(g, successors);
    }

    fn frontier(&self, g: usize, seen: &mut BTreeSet<usize>) -> BTreeSet<usize> {
        if !seen.insert(g) {
            return BTreeSet::new();
        }
        if self.groups[&g].live {
            return BTreeSet::from([g]);
        }
        let next = self.succ.get(&g).cloned().unwrap_or_default();
        next.into_iter()
            .flat_map(|s| self.frontier(s, seen))
            .collect()
    }

    /// The model's answer for resolving `g` (`all`: `AllDescendants`).
    pub fn resolve(&self, g: usize, all: bool) -> Answer {
        let frontier = self.frontier(g, &mut BTreeSet::new());
        let status = match frontier.len() {
            0 => "retired",
            1 => "exact",
            _ if all => "plural",
            _ => "ambiguous",
        };
        (status, frontier)
    }
}
