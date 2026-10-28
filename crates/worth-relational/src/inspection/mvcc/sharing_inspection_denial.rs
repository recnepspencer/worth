/// Why a branch structural-sharing inspection refused the requested selection.
///
/// Scope resolution and payload admission return these denials before an
/// observation escapes. No variant reports a partially assembled observation, and no
/// variant is recoverable into a weaker observation over a smaller selection:
/// the caller chooses the exact selection or receives nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelationalBranchSharingInspectionDenial {
    /// Selected payload custody is cold or damaged; no absent-data answer is inferred.
    StorageUnavailable(crate::storage::data::RelationalStorageUnavailable),
    /// The supplied identity carries a different runtime instance id than the
    /// receiving runtime. Branch identities are runtime-affine and are never
    /// rebound to a fork or clone of their issuing runtime.
    ForeignRuntime,
    /// No branch cell is registered under the identity's branch id, or the
    /// registered cell carries a different exact identity than the one
    /// supplied.
    UnknownBranch,
    /// The selected branch cell has no root, its root carries no commit id, the
    /// referenced commit is absent from the catalog, or the root is not a
    /// complete, artifact-linked root with resolved axes. Sharing evidence is
    /// only reported over complete roots.
    RootUnavailable,
    /// The same exact branch identity appears more than once in the selection.
    /// Duplicates are refused rather than silently deduplicated, so that the
    /// selection-lane metrics keep reporting the caller's own selection size.
    DuplicateBranch,
}
