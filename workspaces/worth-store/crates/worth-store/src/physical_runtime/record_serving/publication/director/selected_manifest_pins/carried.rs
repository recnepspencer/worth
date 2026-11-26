//! Carried selected-manifest rosters (Store M4.3, `G7`).
//!
//! The C.5 routing tree is copy-on-write: a successor root rewrites only the
//! blocks on touched paths and keeps every other child reference verbatim,
//! stamped with the generation that first wrote it. Roots of one running
//! director advance linearly, so a block of generation `g <= floor` reachable
//! from the current root was reachable, unchanged, from the root at `floor`.
//! Its records and therefore its manifest pins equal the predecessor's.
//!
//! [`walk_changed_since`] visits only blocks written after `floor`. Each
//! skipped subtree is reported as one retained key range, each record of a
//! rewritten leaf as one changed record, both in ascending key order, so the
//! merged roster has exactly the order a full rebuild scan produces.

use worth_store_physical_format::RootPublicationCell;

use super::SelectedBlobManifestPin;

/// The verified roster of one selected root, retained for its successor.
#[derive(Debug, Clone)]
pub(in crate::physical_runtime::record_serving::publication::director) struct CarriedManifestRoster
{
    pub(super) root: RootPublicationCell,
    pub(super) tree_identity: u64,
    /// Ascending by record: the order every roster scan produces.
    pub(super) pins: Vec<SelectedBlobManifestPin>,
}

impl CarriedManifestRoster {
    /// Whether `current` may be derived from this roster by a delta walk.
    /// A different tree, a different root at the same generation, or an older
    /// root falls back to the full rebuild.
    pub(super) fn carries_to(&self, current: RootPublicationCell, tree_identity: u64) -> bool {
        self.tree_identity == tree_identity
            && (self.root == current || self.root.generation().get() < current.generation().get())
    }
}

/// One decoded routing block, reduced to what the delta walk needs.
pub(super) enum RoutingNode<R, K> {
    Leaf(Vec<K>),
    Branch(Vec<R>),
}

/// A copy-on-write routing tree whose block references carry their write
/// generation and covered key range.
pub(super) trait RoutingSource {
    type Ref: Copy;
    type Key: Copy + Ord;
    type Error;

    fn generation(&self, reference: Self::Ref) -> u64;
    fn range(&self, reference: Self::Ref) -> (Self::Key, Self::Key);
    fn read(
        &mut self,
        reference: Self::Ref,
    ) -> Result<RoutingNode<Self::Ref, Self::Key>, Self::Error>;
}

/// Receives the delta of the current root against the carried root.
pub(super) trait RoutingDeltaVisitor<K> {
    type Error;

    /// Every record in `[first, last]` is unchanged since the carried root.
    fn retained(&mut self, first: K, last: K) -> Result<(), Self::Error>;
    /// `record` lies in a block written after the carried root.
    fn changed(&mut self, record: K) -> Result<(), Self::Error>;
}

/// Walks only the blocks of `root` written after generation `floor`.
pub(super) fn walk_changed_since<S, V>(
    source: &mut S,
    root: Option<S::Ref>,
    floor: u64,
    visitor: &mut V,
) -> Result<(), S::Error>
where
    S: RoutingSource,
    V: RoutingDeltaVisitor<S::Key, Error = S::Error>,
{
    match root {
        Some(reference) => walk(source, reference, floor, visitor),
        None => Ok(()),
    }
}

fn walk<S, V>(
    source: &mut S,
    reference: S::Ref,
    floor: u64,
    visitor: &mut V,
) -> Result<(), S::Error>
where
    S: RoutingSource,
    V: RoutingDeltaVisitor<S::Key, Error = S::Error>,
{
    if source.generation(reference) <= floor {
        let (first, last) = source.range(reference);
        return visitor.retained(first, last);
    }
    match source.read(reference)? {
        RoutingNode::Leaf(records) => records
            .into_iter()
            .try_for_each(|record| visitor.changed(record)),
        RoutingNode::Branch(children) => children
            .into_iter()
            .try_for_each(|child| walk(source, child, floor, visitor)),
    }
}

/// The carried pins whose records lie in `[first, last]`.
pub(super) fn carried_in_range<P, K: Ord>(
    carried: &[P],
    first: K,
    last: K,
    key_of: impl Fn(&P) -> K,
) -> &[P] {
    let low = carried.partition_point(|pin| key_of(pin) < first);
    let high = carried.partition_point(|pin| key_of(pin) <= last);
    &carried[low..high.max(low)]
}
