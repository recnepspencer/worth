//! One existing cabinet world exercises authored work between real producer waves.
use super::envelope_fixture as fixture;
use super::*;
mod component_carriage;
mod courts;
pub(in super::super) mod created;
pub(in super::super) mod dependent;
mod occurrence_topology;
mod occurrences;
pub(in super::super) mod prerequisite_partition;
mod prerequisite_path;
mod program;
mod reading;
pub(in super::super) mod seal;
use program::{ModelTree, ProducedTree, Program};
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    prerequisite_path::declare(component_carriage::declare(
        prerequisite_partition::declare(occurrence_topology::declare(created::declare(
            reading::declare(seal::declare(dependent::declare(schema))),
        ))),
    ))
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<prerequisite_partition::expectation::Binding, _>(
        prerequisite_partition::expectation::Handler,
    )?;
    setup.handler::<created::Binding, _>(created::Handler)?;
    setup.handler::<dependent::Binding, _>(dependent::Handler)?;
    setup.handler::<seal::Binding, _>(seal::Handler)
}
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    occurrences::seed(graph)?;
    component_carriage::seed(graph)?;
    prerequisite_partition::seed(graph)?;
    for key in ["m", "envelope-calibration"] {
        // Distinct authored records preserve the existing disjoint-write rule.
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Member::reference(),
                WorthQueryApplicationEntityKey::new(format!("progress:{key}")).unwrap(),
            )
            .field(MemberKey::at("MemberKey"), format!("progress:{key}"))
            .field(MemberValue::at("MemberValue"), 0)
            .field(EnvelopeDependent::at("Dependent"), 0)
            .field(EnvelopeGeneration::at("Generation"), 1)
            .field(EnvelopeFrontier::at("Frontier"), 4),
        )?;
    }
    for ordinal in 0..12 {
        let key = format!("dependent:{ordinal}");
        graph.bind_entity(
            WorthQueryApplicationEntitySeed::new(
                Model::reference(),
                WorthQueryApplicationEntityKey::new(&key).unwrap(),
            )
            .field(ModelKey::at("ModelKey"), key)
            .field(ModelValue::at("ModelValue"), ordinal + 1),
        )?;
    }
    Ok(())
}
