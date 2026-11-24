//! The least-member keeper rule of a merge: of the merged parents, the one
//! whose name sorts least (`WorthQueryName`'s canonical byte order) keeps.
//!
//! An application expressing "the least member survives" (for example a CAD
//! arrangement merging coplanar faces) names the rule instead of choosing a
//! keeper, so the outcome never depends on the order its parents were found
//! in. The rule only selects; Relational still validates and records the
//! merge as an ordinary keeper merge.

use worth_query_declaration::facade::application_naming::{
    ApplicationNameSubjectKind, WorthQueryName,
};

use super::{WorthQueryOutputMergeKeeper, WorthQueryOutputNameDenial};

/// Where the least parent's name lives on after a least-keeper merge;
/// `Target` is the created-output handle of the merged kind.
#[derive(Clone, Debug)]
pub enum WorthQueryLeastMergeKeeper<Target> {
    /// The least parent's record and name stay in place.
    InPlace,
    /// Every parent's record is retired and the least parent's name moves
    /// onto `Target`, a record the operation created.
    Onto(Target),
}

impl<Target> WorthQueryLeastMergeKeeper<Target> {
    /// The parents in canonical order and the keeper posture naming the
    /// least of them. Any permutation of `parents` yields the same result.
    ///
    /// No parents refuses `EmptyMergeParents`: there is no least parent. A
    /// single or repeated parent is passed on unchanged, so the commit
    /// refuses it exactly as it refuses any merge of fewer than two distinct
    /// parents.
    ///
    /// Complexity: O(P log P) for P parents.
    #[allow(clippy::type_complexity)]
    pub fn resolve<K>(
        self,
        subject: ApplicationNameSubjectKind,
        parents: &[WorthQueryName<K>],
    ) -> Result<
        (
            Vec<WorthQueryName<K>>,
            WorthQueryOutputMergeKeeper<K, Target>,
        ),
        WorthQueryOutputNameDenial,
    > {
        let mut ordered = parents.to_vec();
        ordered.sort_unstable();
        let least = *ordered
            .first()
            .ok_or(WorthQueryOutputNameDenial::EmptyMergeParents { subject })?;
        let keeper = match self {
            Self::InPlace => WorthQueryOutputMergeKeeper::KeepParentInPlace(least),
            Self::Onto(target) => WorthQueryOutputMergeKeeper::KeepParentOnto {
                keeper: least,
                target,
            },
        };
        Ok((ordered, keeper))
    }
}
