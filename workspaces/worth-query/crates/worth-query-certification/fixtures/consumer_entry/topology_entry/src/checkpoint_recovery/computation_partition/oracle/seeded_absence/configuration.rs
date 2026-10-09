use super::*;

pub(super) fn seed(graph: &mut Graph, model: &Model, boundary: Boundary) {
    model.seed(graph);
    match boundary {
        Boundary::Evict => {}
        Boundary::Observation => {
            for number in 0..OBSERVATION_SETS {
                let name = format!("incoming-{number}");
                facts::seed_set(graph, &name, -0.0);
                facts::seed_member(
                    graph,
                    &name,
                    &format!("oracle-entry-{}", model.heavy_number()),
                );
            }
        }
        Boundary::Several => {}
    }
}

pub(super) fn profile(boundary: Boundary) -> WorthQueryOutputDemandResourceProfile {
    match boundary {
        Boundary::Evict => Default::default(),
        Boundary::Observation => WorthQueryOutputDemandResourceProfile::standard()
            .with_lineage_retained_bytes(NonZeroUsize::new(64 * 1024 * 1024).unwrap()),
        Boundary::Several => Default::default(),
    }
}
