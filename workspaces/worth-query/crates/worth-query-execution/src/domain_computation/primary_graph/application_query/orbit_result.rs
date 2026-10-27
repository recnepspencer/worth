//! Inspection of a completed disclosed orbit answer never performs a new read.
use super::WorthQueryObservedResultSet;
use crate::domain_computation::primary_graph::aggregate_identity::WorthQueryAggregateIdentity;
use worth_query_declaration::facade::application_query::ApplicationQueryOrbitRef;

/// One complete disclosed orbit: its persistent name and anchor-forward member rows.
pub struct WorthQueryApplicationQueryOrbit<'a, Schema, Relation, Row> {
    identity: WorthQueryAggregateIdentity<Schema, Relation>,
    members: &'a [Row],
    length: usize,
}
impl<Schema, Relation, Row> WorthQueryApplicationQueryOrbit<'_, Schema, Relation, Row> {
    pub const fn identity(&self) -> &WorthQueryAggregateIdentity<Schema, Relation> {
        &self.identity
    }
    pub const fn members(&self) -> &[Row] {
        self.members
    }
    pub const fn len(&self) -> usize {
        self.length
    }
    pub const fn is_empty(&self) -> bool {
        self.length == 0
    }
}

pub(super) fn inspect<'a, Schema, Query, Slot, Relation, Entity, Row>(
    rows: &'a [Row],
    result_set: &WorthQueryObservedResultSet<Query>,
    reference: ApplicationQueryOrbitRef<Schema, Query, Slot, Relation, Entity>,
) -> Option<WorthQueryApplicationQueryOrbit<'a, Schema, Relation, Row>> {
    let selected = result_set.declared_orbit()?;
    if selected.meaning.as_ref() != reference.meaning()
        || selected.aggregate.native().len() != rows.len()
    {
        return None;
    }
    Some(WorthQueryApplicationQueryOrbit {
        identity: WorthQueryAggregateIdentity::from_pinned_read(
            selected.aggregate.native(),
            selected.binding.clone(),
            selected.aggregate.key().relation_kind(),
        ),
        members: rows,
        length: selected.aggregate.native().len(),
    })
}
