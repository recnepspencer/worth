//! One-hop physical traversal and predecessor custody in either authored direction.
use super::super::{
    read_observation::WorthQueryInvariantProjectionWork,
    schema_layout::WorthQueryPrimaryRelationLayout,
};
use super::*;
use worth_relational::facade::runtime::RelationalAdjacencyDirection;
pub(super) fn advance<E>(
    basis: &RootPathReadBasis<'_, '_, '_>,
    layout: &WorthQueryPrimaryRelationLayout,
    located: PathStep<'_>,
    frontier: &mut Frontier,
    ports: &mut RootPathReadPorts<'_, E>,
    set_source: &mut Option<SourceFactsBuilder>,
) -> Result<Frontier, RootPathReadDenial<E>> {
    let subject = located.step.relation();
    let anchors = frontier.keys().copied().collect::<BTreeSet<_>>();
    let direction = match located.direction {
        RelationDirection::Forward => RelationalAdjacencyDirection::Outgoing,
        RelationDirection::Reverse => RelationalAdjacencyDirection::Incoming,
    };
    for (&anchor, source) in &mut *frontier {
        if !source.has_adjacency(anchor, layout.kind, direction) {
            ports.notify(RootPathReadObservation::SourceRead { subject })?;
        }
        let observed = source
            .observe_adjacency(
                basis.execution,
                &basis.projection,
                anchor,
                layout.kind,
                direction,
            )
            .map_err(|cause| RootPathReadDenial::source(subject, cause))?;
        if let Some(set_source) = &mut *set_source
            && set_source.record_adjacency(observed)
        {
            ports.notify(RootPathReadObservation::SourceCopy { units: 1, subject })?;
        }
    }
    ports.check(subject)?;
    let read = match located.direction {
        RelationDirection::Forward => basis.runtime.bounded_outgoing_relations_for_frontier(
            basis.execution,
            &anchors,
            layout.kind,
            None,
        ),
        RelationDirection::Reverse => basis.runtime.bounded_incoming_relations_for_frontier(
            basis.execution,
            &anchors,
            layout.kind,
            None,
        ),
    }
    .map_err(|stop| RootPathReadDenial::frontier(subject, stop))?;
    ports.notify(RootPathReadObservation::Adjacency {
        lists: read.adjacency_lists_read(),
        records: read.relation_records_examined(),
        endpoints: read.endpoint_records_reserved(),
        subject,
    })?;
    let mut reads = WorthQueryInvariantProjectionWork::default();
    reads.record_adjacencies(
        read.adjacency_lists_read(),
        read.relation_records_examined(),
    );
    ports.notify(located.completed(reads))?;
    let mut next = BTreeMap::new();
    for record in read.into_records() {
        ports.check(subject)?;
        let (anchor, endpoint) = match located.direction {
            RelationDirection::Forward => (record.source, record.target),
            RelationDirection::Reverse => (record.target, record.source),
        };
        if next.contains_key(&endpoint) {
            continue;
        }
        let predecessor = frontier
            .get(&anchor)
            .ok_or_else(|| RootPathReadDenial::traversal(subject))?;
        ports.notify(RootPathReadObservation::SourceCopy {
            units: predecessor.source_count(),
            subject,
        })?;
        let mut source = predecessor.clone();
        source.observe_entity(endpoint);
        next.insert(endpoint, source);
    }
    Ok(next)
}
