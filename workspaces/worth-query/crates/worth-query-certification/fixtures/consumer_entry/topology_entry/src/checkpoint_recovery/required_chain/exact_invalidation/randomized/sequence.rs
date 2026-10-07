//! What a random sequence draws: its commits, its demands, and the order the
//! held chain advances in.

use super::*;

pub(super) struct Xorshift(pub(super) u64);

impl Xorshift {
    pub(super) fn below(&mut self, bound: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % bound
    }

    /// A value in `low..=high` other than `current`, which lies in that range.
    fn other(&mut self, low: u64, high: u64, current: u64) -> u64 {
        let pick = low + self.below(high - low);
        pick + u64::from(pick >= current)
    }
}

/// What one step commits before it demands.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Commit {
    Nothing,
    /// The one field the root producer's input carries.
    RootInput,
    /// A field written to the value it already holds.
    EqualWrite,
    /// A field the root's source query fetches and its input omits.
    FetchedOnly,
    /// A field of a body further along the ring, which the root's source
    /// query fetches as well.
    FetchedFar,
    /// The root output, overwritten by a writer that is not its producer.
    RootOutputOverwritten,
    /// A chain output, overwritten likewise: its consumer must decide again.
    ChainOutputOverwritten,
    /// An entity deleted and another created in its place, under a new index
    /// key or the retired one, with both its relations removed and added.
    SuccessorReplaced,
    /// A root input committed after its demands started and before they settle.
    RacingRootInput,
}

pub(super) const COMMITS: [Commit; 9] = [
    Commit::Nothing,
    Commit::RootInput,
    Commit::EqualWrite,
    Commit::FetchedOnly,
    Commit::FetchedFar,
    Commit::RootOutputOverwritten,
    Commit::ChainOutputOverwritten,
    Commit::SuccessorReplaced,
    Commit::RacingRootInput,
];

impl Commit {
    pub(super) fn apply(
        self,
        court: &Court<'_, '_, '_, '_>,
        ring: &mut Ring,
        random: &mut Xorshift,
        at: &str,
    ) {
        match self {
            Self::Nothing => {}
            Self::RootInput | Self::RacingRootInput => {
                ring.a_y = random.other(1, 8, ring.a_y);
                court.write_y(&ring.key("a"), ring.a_y, at);
            }
            Self::EqualWrite => court.write_y(&ring.key("a"), ring.a_y, at),
            Self::FetchedOnly => {
                ring.successor_y = random.other(1, 3, ring.successor_y);
                court.write_y(&ring.successor, ring.successor_y, at);
            }
            Self::FetchedFar => {
                ring.far_y = random.other(9, 12, ring.far_y);
                court.write_y(&ring.key("source-c"), ring.far_y, at);
            }
            Self::RootOutputOverwritten => {
                court.overwrite_length(&ring.key("a"), 90 + random.below(9), at);
            }
            Self::ChainOutputOverwritten => {
                let (role, length) = if random.below(2) == 0 {
                    ("b", &mut ring.b_length)
                } else {
                    ("c", &mut ring.c_length)
                };
                *length = random.other(30, 60, (*length).clamp(30, 60));
                court.overwrite_length(&ring_world::key(ring.index, role), *length, at);
            }
            Self::SuccessorReplaced => {
                let seeded = ring.key("source-b");
                let replacement = if ring.successor == seeded {
                    ring.key("source-b-next")
                } else {
                    seeded
                };
                court.replace_successor(ring, replacement, 1 + random.below(3), at);
            }
        }
    }
}

/// What one step demands after its commit.
#[derive(Clone, Copy, Debug)]
pub(super) enum Demands {
    /// Nothing: the next step's commit lands on unsettled marks.
    Nothing,
    /// The chain ring 0 keeps open, in a drawn order.
    Held,
    /// The step's own ring, afresh.
    Ring,
    /// Every output, and then every output again at no cost.
    Everything,
}

pub(super) const DEMANDS: [Demands; 4] = [
    Demands::Nothing,
    Demands::Held,
    Demands::Ring,
    Demands::Everything,
];

/// Prints the failing seed and every step up to the failing one when anything
/// under it panics, the production equivalence check included.
pub(super) struct Trace<'history>(pub(super) &'history [String]);

impl Drop for Trace<'_> {
    fn drop(&mut self) {
        if std::thread::panicking() {
            eprintln!("the randomized courtroom failed after these steps:");
            for step in self.0 {
                eprintln!("  {step}");
            }
        }
    }
}

/// Every order the three held demands can advance in.
pub(super) const HELD_ORDERS: [[char; 3]; 6] = [
    ['c', 'b', 'a'],
    ['c', 'a', 'b'],
    ['b', 'c', 'a'],
    ['b', 'a', 'c'],
    ['a', 'c', 'b'],
    ['a', 'b', 'c'],
];
