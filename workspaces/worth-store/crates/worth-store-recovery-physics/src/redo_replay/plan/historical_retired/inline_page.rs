//! An inline page none of whose records the selected root still routes.
//!
//! Every publication appends derived tree nodes inline, and its directory
//! append retires the nodes it replaced. A page whose every record was retired
//! above the checkpoint is unrouted yet lies below the allocation frontier, so
//! its WAL images are neither materialized nor fresh. They are historical only
//! when the ordered history itself wrote the page and retired all of it:
//! being unrouted and already allocated is never enough.

use super::super::supersession::{inline_image, require_successor};
use super::super::*;
use super::HistoricalRetirements;
use worth_store_physical_format::PersistedRecordIdentity;

impl HistoricalRetirements<'_> {
    /// The selected root identity and the operation of the last retirement
    /// that emptied this page.
    ///
    /// `target` must be the last admitted image of its page. Every image of
    /// the page must continue the one before it and be written by an ordered
    /// edge, so no image lies above the history.
    pub(in super::super) fn retired_page(
        &self,
        target: &PhysicalRedoTarget,
    ) -> Option<([u8; 32], [u8; 32])> {
        let PhysicalRedoTargetIdentity::InlinePage { segment, page, .. } = target.identity() else {
            return None;
        };
        let chain = self
            .claims
            .as_ref()?
            .get(&(segment, page))?
            .values()
            .collect::<Vec<_>>();
        let last = *chain.last()?;
        if last.target != target {
            return None;
        }
        for adjacent in chain.windows(2) {
            require_successor(self.members, adjacent[0], adjacent[1]).ok()?;
        }
        let written = chain
            .iter()
            .map(|claim| self.written[claim.member].unique())
            .collect::<Option<Vec<_>>>()?
            .pop()?;
        let image = inline_image(&self.members[last.member], last.target).ok()?;
        let records = image
            .records
            .iter()
            .map(|(placement, _)| placement.record());
        let (position, selected_root_identity) = self.emptied_at(records, written)?;
        Some((selected_root_identity, self.edges[position].operation))
    }

    /// The position of the last retirement among the records of an image the
    /// edge at `written` wrote, and the selected root that routes none of
    /// them.
    ///
    /// Every record must be unrouted under the anchored selected root and
    /// dropped by exactly one ordinary edge's admitted derived-directory
    /// retirement, ordered no earlier than `written`. An image that holds no
    /// record proves no retirement. A page appended to after one of its
    /// records was retired is denied too: the producer never writes one, and
    /// denying it is the closed side.
    pub(in super::super) fn emptied_at(
        &self,
        records: impl Iterator<Item = PersistedRecordIdentity>,
        written: usize,
    ) -> Option<(usize, [u8; 32])> {
        let mut retired = None;
        for record in records {
            let selected_root_identity = self.unrouted(record)?;
            let position = self.retiring.get(&record)?.unique()?;
            if position < written {
                return None;
            }
            retired = retired.max(Some((position, selected_root_identity)));
        }
        retired
    }
}
