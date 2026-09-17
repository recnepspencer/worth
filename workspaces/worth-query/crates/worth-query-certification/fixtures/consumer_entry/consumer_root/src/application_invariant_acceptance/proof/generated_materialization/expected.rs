use worth_query_consumer_values::PlanarVertex;
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationOutputRole, WorthQueryCreateOutput,
};
use worth_query_topology_entry::{Body, FinalPlanarMutationBinding, PlanarRead};

use super::super::{length, Request};
use crate::ConsumerSchema;

pub(super) fn role(
    vertex: &PlanarVertex,
    index: usize,
) -> WorthQueryApplicationOutputRole<
    FinalPlanarMutationBinding<ConsumerSchema>,
    Body,
    WorthQueryCreateOutput,
> {
    WorthQueryApplicationOutputRole::try_new(if index == 0 {
        "anchor".to_owned()
    } else {
        format!("created.{}", vertex.body_key)
    })
    .expect("fixture keys form valid source-derived output roles")
}

pub(super) fn read(
    request: &Request<'_>,
    key: &str,
) -> worth_query_topology_entry::PlanarReadResult {
    let result = request
        .query(PlanarRead {
            body_key: key.to_owned(),
        })
        .execute()
        .unwrap_or_else(|denial| panic!("the generated body is readable: {denial:?}"));
    assert_eq!(result.rows().len(), 1);
    result.rows()[0].clone()
}

pub(super) fn vertices() -> Vec<PlanarVertex> {
    [(1, 4), (2, 4), (1, 5)]
        .into_iter()
        .enumerate()
        .map(|(index, (x, y))| PlanarVertex {
            body_key: match index {
                0 => "final:anchor-b".to_owned(),
                1 => "final:anchor-b:b".to_owned(),
                _ => "final:anchor-b:c".to_owned(),
            },
            x: length(x),
            y: length(y),
        })
        .collect()
}
