use std::fmt;

use super::WorthQueryName;

/// Successful resolution of a held persistent name at a reader's basis.
///
/// `H` is the live handle type a reader returns and `C` the commit basis
/// coordinate (a Relational commit sequence by default).
///
/// - `Exact(handle)`: exactly one live answer under the requested selection.
/// - `Plural(handles)`: two or more live descendants under
///   [`super::NameSelection::AllDescendants`], in canonical name order.
/// - `Retired { at_commit }`: the held name has no live descendant; it lost
///   its last live descendant at `at_commit`.
///
/// ```
/// use worth_query_declaration::facade::application_naming::ResolvedName;
///
/// fn describe(resolved: ResolvedName<&str>) -> String {
///     match resolved {
///         ResolvedName::Exact(face) => format!("one {face}"),
///         ResolvedName::Plural(faces) => format!("{} pieces", faces.len()),
///         ResolvedName::Retired { at_commit } => format!("retired at {at_commit}"),
///     }
/// }
///
/// assert_eq!(describe(ResolvedName::Retired { at_commit: 7 }), "retired at 7");
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum ResolvedName<H, C = u64> {
    Exact(H),
    Plural(Vec<H>),
    Retired { at_commit: C },
}

/// Typed refusal to resolve a held `WorthQueryName<K>`.
///
/// Resolution never silently rebinds: a plural answer that the selection
/// cannot narrow refuses with every candidate in canonical name order.
///
/// - `Ambiguous { candidates }`: two or more live candidates (never fewer)
///   under [`super::NameSelection::Exact`] or a filter matching several.
/// - `NoMatchingCandidate { live_candidates }`: a carrier or side filter
///   matched none of the live descendants listed here.
/// - `BasisPruned`: the reader's basis lies below the history retention floor.
/// - `UnknownName`: the name has no birth visible at the reader's basis.
///
/// Candidates keep the held name's kind:
///
/// ```compile_fail,E0308
/// use worth_query_declaration::facade::application_naming::{
///     NameResolutionDenial, WorthQueryName,
/// };
///
/// struct Face;
/// struct Edge;
///
/// fn misuse(edges: Vec<WorthQueryName<Edge>>) -> NameResolutionDenial<Face> {
///     NameResolutionDenial::Ambiguous { candidates: edges }
/// }
/// ```
///
/// ```
/// use worth_query_declaration::facade::application_naming::{
///     NameResolutionDenial, WorthQueryName,
/// };
///
/// struct Face;
///
/// fn candidates(denial: &NameResolutionDenial<Face>) -> &[WorthQueryName<Face>] {
///     match denial {
///         NameResolutionDenial::Ambiguous { candidates } => candidates,
///         NameResolutionDenial::NoMatchingCandidate { live_candidates } => live_candidates,
///         NameResolutionDenial::BasisPruned | NameResolutionDenial::UnknownName => &[],
///     }
/// }
///
/// assert!(candidates(&NameResolutionDenial::<Face>::UnknownName).is_empty());
/// ```
pub enum NameResolutionDenial<K> {
    Ambiguous {
        candidates: Vec<WorthQueryName<K>>,
    },
    NoMatchingCandidate {
        live_candidates: Vec<WorthQueryName<K>>,
    },
    BasisPruned,
    UnknownName,
}

impl<K> Clone for NameResolutionDenial<K> {
    fn clone(&self) -> Self {
        match self {
            Self::Ambiguous { candidates } => Self::Ambiguous {
                candidates: candidates.clone(),
            },
            Self::NoMatchingCandidate { live_candidates } => Self::NoMatchingCandidate {
                live_candidates: live_candidates.clone(),
            },
            Self::BasisPruned => Self::BasisPruned,
            Self::UnknownName => Self::UnknownName,
        }
    }
}

impl<K> PartialEq for NameResolutionDenial<K> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Ambiguous { candidates: a }, Self::Ambiguous { candidates: b }) => a == b,
            (
                Self::NoMatchingCandidate { live_candidates: a },
                Self::NoMatchingCandidate { live_candidates: b },
            ) => a == b,
            (Self::BasisPruned, Self::BasisPruned) | (Self::UnknownName, Self::UnknownName) => true,
            _ => false,
        }
    }
}

impl<K> Eq for NameResolutionDenial<K> {}

impl<K> fmt::Debug for NameResolutionDenial<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ambiguous { candidates } => formatter
                .debug_struct("Ambiguous")
                .field("candidates", candidates)
                .finish(),
            Self::NoMatchingCandidate { live_candidates } => formatter
                .debug_struct("NoMatchingCandidate")
                .field("live_candidates", live_candidates)
                .finish(),
            Self::BasisPruned => formatter.write_str("BasisPruned"),
            Self::UnknownName => formatter.write_str("UnknownName"),
        }
    }
}

impl<K> fmt::Display for NameResolutionDenial<K> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ambiguous { candidates } => write!(
                formatter,
                "held name is ambiguous among {} live candidates",
                candidates.len()
            ),
            Self::NoMatchingCandidate { live_candidates } => write!(
                formatter,
                "selection matched none of {} live candidates",
                live_candidates.len()
            ),
            Self::BasisPruned => formatter.write_str("reader basis is below the retention floor"),
            Self::UnknownName => formatter.write_str("held name has no birth at the reader basis"),
        }
    }
}

impl<K> std::error::Error for NameResolutionDenial<K> {}
