//! Rebuild the genuine output role and both exact retained membership endpoints.
use super::*;
pub(super) fn complete(
    world: &host::Host,
    suspended: WorthQuerySuspendedGeneratedOutput,
    name: &str,
) -> WorthQueryCompletedGeneratedOutputReconstruction<Catalog, producer::Producer> {
    with_bounds(world, suspended, name, bounds::Coordinates::new(0, 0, 0, 0))
}
pub(super) fn with_bounds(
    world: &host::Host,
    suspended: WorthQuerySuspendedGeneratedOutput,
    name: &str,
    coordinates: bounds::Coordinates,
) -> WorthQueryCompletedGeneratedOutputReconstruction<Catalog, producer::Producer> {
    let mut reconstruction = world
        .with_host_advancement(|phase| {
            world
                .reconstruct_generated_output::<producer::Producer>(&phase, suspended)
                .unwrap_or_else(|failure| {
                    panic!(
                        "the actual producer reconstructs its custody: {:?}",
                        failure.denial()
                    )
                })
        })
        .unwrap();
    let WorthQueryReconstructedOutputEntity::Generated(item) =
        reconstruction.output::<operation::ItemOutput>().unwrap()
    else {
        panic!("this output role was genuinely created")
    };
    reconstruction
        .field(
            &item,
            ItemIdentity::at("ItemIdentity"),
            "generated-item".to_owned(),
        )
        .unwrap();
    reconstruction
        .field(&item, ItemName::at("ItemName"), name.to_owned().into())
        .unwrap();
    reconstruction
        .field(&item, ItemEnabled::at("ItemEnabled"), true)
        .unwrap();
    reconstruction
        .field(&item, bounds::ItemBounds::at("ItemBounds"), coordinates)
        .unwrap();
    let directories = reconstruction
        .retained_relation_sources(Contains::reference(), Directory::reference(), &item)
        .unwrap();
    assert_eq!(
        directories.len(),
        2,
        "the actual manifest retains both membership endpoints"
    );
    for directory in directories {
        reconstruction
            .relation_from_retained(Contains::reference(), &directory, &item)
            .unwrap();
    }
    reconstruction.finish().unwrap_or_else(|failure| {
        panic!(
            "the exact generated Item/relations complete their real manifest: {:?}",
            failure.denial()
        )
    })
}
