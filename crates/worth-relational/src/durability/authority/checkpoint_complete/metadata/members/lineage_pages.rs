//! Exact original node pages and event pages with funded routing directories.
use super::super::{encode_lineage, encode_lineage_events, OriginalLineageEvent};
use super::*;
pub(super) fn capture(
    lineage: &OriginalCheckpointLineage<'_>,
    request: &ExecutionRequest<'_, '_>,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
) -> Result<(CheckpointPages, CheckpointLineageEventPages), Denial> {
    let ranges = || {
        lineage.nodes.chunk_by(|a, b| {
            RelationalCheckpointPage::range_of(a.lineage_id().0)
                == RelationalCheckpointPage::range_of(b.lineage_id().0)
        })
    };
    let mut nodes = CheckpointPages::allocate_for_request(ranges().count(), request)
        .map_err(Denial::Request)?;
    for page in ranges() {
        let member = add(
            members,
            encode_lineage(page, request).map_err(stop)?,
            request,
        )?;
        nodes
            .push(
                RelationalCheckpointPage::from_ids(
                    page.iter().map(|node| node.lineage_id().0),
                    member,
                )
                .ok_or(Denial::OriginalAuthorityUnavailable)?,
            )
            .map_err(Denial::Request)?;
    }
    // At most one descriptor per actual event; byte limits can split row pages.
    let mut events =
        CheckpointLineageEventPages::allocate_for_request(lineage.events.len(), request)
            .map_err(Denial::Request)?;
    for page in lineage.events.chunks(LINEAGE_EVENT_PAGE_ROWS) {
        append_events(page, request, members, &mut events)?;
    }
    Ok((nodes.seal(), events.seal()))
}
fn append_events(
    page: &[OriginalLineageEvent<'_>],
    request: &ExecutionRequest<'_, '_>,
    members: &mut ExecutionOrderedMap<RelationalDurableMemberLocator, RelationalCheckpointMember>,
    directory: &mut CheckpointLineageEventPagesBuilder,
) -> Result<(), Denial> {
    let encoded = encode_lineage_events(page, request).map_err(stop)?;
    if encoded.0.length > LINEAGE_EVENT_PAGE_BYTES {
        if page.len() == 1 {
            let mut nodes = page[0]
                .event
                .sources()
                .iter()
                .chain(page[0].event.targets());
            let first = nodes.next().ok_or(Denial::OriginalAuthorityUnavailable)?.0;
            let (first, last) = nodes.fold((first, first), |(lo, hi), node| {
                (lo.min(node.0), hi.max(node.0))
            });
            return Err(crate::storage::data::RelationalStorageUnavailable {
                subject: crate::storage::data::RelationalStorageSubject::Lineage {
                    first: crate::identity::data::LineageId(first),
                    last: crate::identity::data::LineageId(last),
                },
                cause: crate::storage::data::RelationalStorageUnavailableCause::LimitExceeded {
                    length: encoded.0.length,
                    limit: LINEAGE_EVENT_PAGE_BYTES,
                },
            }
            .into());
        }
        drop(encoded);
        let (left, right) = page.split_at(page.len() / 2);
        append_events(left, request, members, directory)?;
        return append_events(right, request, members, directory);
    }
    let member = add(members, encoded, request)?;
    directory.push(member, page.iter().map(|row| row.event), request)
}
