//! A held name's matched live records before owner restriction, and the one
//! rule that turns them into a resolution or a typed refusal.

use worth_query_declaration::facade::application_naming::{
    NameCandidate, NameResolutionDenial, ResolvedName, WorthQueryName,
};

/// What a held name resolved to at the reader's basis, before any owner
/// restriction: the live records the selection matched, or the outcome that
/// leaves nothing to restrict.
///
/// `H` is the live handle of each matched record. Matched records are in
/// canonical name order.
pub struct NameResolutionCandidates<K, H> {
    state: State<K, H>,
}

enum State<K, H> {
    /// One or more live records the selection matched. `selects_all` is an
    /// `AllDescendants` selection, which answers several as `Plural`.
    Matched {
        matched: Vec<(WorthQueryName<K>, H)>,
        selects_all: bool,
    },
    Retired {
        at_commit: u64,
    },
    Refused(NameResolutionDenial<K>),
}

impl<K, H> NameResolutionCandidates<K, H> {
    pub(super) fn matched(matched: Vec<(WorthQueryName<K>, H)>, selects_all: bool) -> Self {
        Self {
            state: State::Matched {
                matched,
                selects_all,
            },
        }
    }

    pub(super) fn retired(at_commit: u64) -> Self {
        Self {
            state: State::Retired { at_commit },
        }
    }

    pub(super) fn refused(denial: NameResolutionDenial<K>) -> Self {
        Self {
            state: State::Refused(denial),
        }
    }

    /// The live records the selection matched; empty when the name is
    /// retired or refused.
    pub fn handles(&self) -> impl Iterator<Item = &H> {
        let matched = match &self.state {
            State::Matched { matched, .. } => matched.as_slice(),
            State::Retired { .. } | State::Refused(_) => &[],
        };
        matched.iter().map(|(_, handle)| handle)
    }

    /// Whether an unscoped resolution refuses `Ambiguous`, so that each
    /// candidate's owners will be reported.
    pub fn is_ambiguous(&self) -> bool {
        matches!(
            &self.state,
            State::Matched { matched, selects_all: false } if matched.len() > 1
        )
    }

    /// Resolves across every owner. Several matches under a selection that
    /// names one refuse `Ambiguous` with each candidate's owners, read
    /// through `owners` only then.
    ///
    /// Complexity: O(matched) plus one `owners` read per ambiguous candidate.
    pub fn resolve<O, E>(
        self,
        mut owners: impl FnMut(&H) -> Result<Vec<O>, E>,
    ) -> Result<Result<ResolvedName<H>, NameResolutionDenial<K, O>>, E> {
        let ambiguous = self.is_ambiguous();
        self.settle(|matched| {
            if !ambiguous {
                return Ok(Scoped::Kept(matched));
            }
            let candidates = with_owners(matched, &mut owners)?;
            Ok(Scoped::Refused(NameResolutionDenial::Ambiguous {
                candidates: named(candidates),
            }))
        })
    }

    /// Resolves among the matched records `owner` holds. When it holds none
    /// the resolution refuses `NoCandidateInOwner` with every matched
    /// candidate and its owners; it never falls back to another owner.
    ///
    /// Complexity: O(matched * owners per record) plus one `owners` read per
    /// matched record.
    pub fn resolve_in<O: PartialEq, E>(
        self,
        owner: &O,
        mut owners: impl FnMut(&H) -> Result<Vec<O>, E>,
    ) -> Result<Result<ResolvedName<H>, NameResolutionDenial<K, O>>, E> {
        let selects_all = matches!(
            &self.state,
            State::Matched {
                selects_all: true,
                ..
            }
        );
        self.settle(|matched| {
            let candidates = with_owners(matched, &mut owners)?;
            if !candidates
                .iter()
                .any(|(_, _, held_by)| held_by.contains(owner))
            {
                return Ok(Scoped::Refused(NameResolutionDenial::NoCandidateInOwner {
                    candidates: named(candidates),
                }));
            }
            let held: Vec<_> = candidates
                .into_iter()
                .filter(|(_, _, held_by)| held_by.contains(owner))
                .collect();
            if held.len() > 1 && !selects_all {
                return Ok(Scoped::Refused(NameResolutionDenial::Ambiguous {
                    candidates: named(held),
                }));
            }
            Ok(Scoped::Kept(
                held.into_iter()
                    .map(|(name, handle, _)| (name, handle))
                    .collect(),
            ))
        })
    }

    /// Applies `scope` to the matched records and answers one kept record
    /// `Exact` and several `Plural`.
    fn settle<O, E>(
        self,
        scope: impl FnOnce(Vec<(WorthQueryName<K>, H)>) -> Result<Scoped<K, H, O>, E>,
    ) -> Result<Result<ResolvedName<H>, NameResolutionDenial<K, O>>, E> {
        let matched = match self.state {
            State::Matched { matched, .. } => matched,
            State::Retired { at_commit } => return Ok(Ok(ResolvedName::Retired { at_commit })),
            State::Refused(denial) => return Ok(Err(denial.map_owners(|never| match never {}))),
        };
        Ok(match scope(matched)? {
            Scoped::Refused(denial) => Err(denial),
            Scoped::Kept(mut kept) if kept.len() == 1 => Ok(ResolvedName::Exact(kept.remove(0).1)),
            Scoped::Kept(kept) => Ok(ResolvedName::Plural(
                kept.into_iter().map(|(_, handle)| handle).collect(),
            )),
        })
    }
}

enum Scoped<K, H, O> {
    Kept(Vec<(WorthQueryName<K>, H)>),
    Refused(NameResolutionDenial<K, O>),
}

type Owned<K, H, O> = Vec<(WorthQueryName<K>, H, Vec<O>)>;

fn with_owners<K, H, O, E>(
    matched: Vec<(WorthQueryName<K>, H)>,
    owners: &mut impl FnMut(&H) -> Result<Vec<O>, E>,
) -> Result<Owned<K, H, O>, E> {
    matched
        .into_iter()
        .map(|(name, handle)| owners(&handle).map(|held_by| (name, handle, held_by)))
        .collect()
}

fn named<K, H, O>(candidates: Owned<K, H, O>) -> Vec<NameCandidate<K, O>> {
    candidates
        .into_iter()
        .map(|(name, _, held_by)| NameCandidate::new(name, held_by))
        .collect()
}
