//! All actual generated roles and retained memberships reconstruct through custody.
use super::*;
pub(super) fn complete(
    world: &host::Host,
    suspended: WorthQuerySuspendedGeneratedOutput,
    end_x: i64,
) -> WorthQueryCompletedGeneratedOutputReconstruction<Catalog, producer::Producer> {
    let mut reconstruction = world
        .with_host_advancement(|phase| {
            world
                .reconstruct_generated_output::<producer::Producer>(&phase, suspended)
                .unwrap_or_else(|failure| {
                    panic!(
                        "actual geometry reconstruction admits: {:?}",
                        failure.denial()
                    )
                })
        })
        .unwrap();
    let WorthQueryReconstructedOutputEntity::Generated(edge) =
        reconstruction.output::<operation::EdgeOutput>().unwrap()
    else {
        panic!("actual Edge was generated")
    };
    let WorthQueryReconstructedOutputEntity::Generated(start) =
        reconstruction.output::<operation::StartOutput>().unwrap()
    else {
        panic!("actual start Vertex was generated")
    };
    let WorthQueryReconstructedOutputEntity::Generated(end) =
        reconstruction.output::<operation::EndOutput>().unwrap()
    else {
        panic!("actual end Vertex was generated")
    };
    reconstruction
        .field(
            &edge,
            EdgeIdentity::at("EdgeIdentity"),
            "generated-edge".to_owned(),
        )
        .unwrap();
    for (vertex, identity, x) in [
        (&start, "generated-start", 0),
        (&end, "generated-end", end_x),
    ] {
        reconstruction
            .field(
                vertex,
                VertexIdentity::at("VertexIdentity"),
                identity.to_owned(),
            )
            .unwrap();
        reconstruction
            .field(
                vertex,
                VertexRegion::at("VertexRegion"),
                Coordinates::new(x, 0, x, 0),
            )
            .unwrap();
    }
    reconstruction
        .relation(Starts::reference(), &edge, &start)
        .unwrap();
    reconstruction
        .relation(Ends::reference(), &edge, &end)
        .unwrap();
    let directories = reconstruction
        .retained_relation_sources(
            ContainsEdge::reference(),
            super::super::super::super::schema::Directory::reference(),
            &edge,
        )
        .unwrap();
    assert_eq!(
        directories.len(),
        2,
        "the genuine manifest retains both Directory memberships"
    );
    for directory in directories {
        reconstruction
            .relation_from_retained(ContainsEdge::reference(), &directory, &edge)
            .unwrap();
    }
    reconstruction.finish().unwrap_or_else(|failure| {
        panic!(
            "actual generated endpoint manifest completes: {:?}",
            failure.denial()
        )
    })
}
