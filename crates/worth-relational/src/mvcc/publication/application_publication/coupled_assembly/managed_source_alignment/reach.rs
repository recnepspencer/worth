//! What a probed scope's Source read at one candidate image: every record
//! and adjacency list its membership depended on, including the lists it
//! found empty, and every row it produced. A member whose records touch
//! none of them cannot have changed that Source, whatever their kinds.
use crate::identity::data::{EntityId, KindId};
use std::collections::BTreeSet;

#[derive(Default)]
pub(super) struct Reach {
    /// Membership dependencies and rows.
    pub(super) entities: BTreeSet<EntityId>,
    /// `(anchor, relation kind, outgoing)` of every list membership read.
    pub(super) anchors: BTreeSet<(EntityId, KindId, bool)>,
}
