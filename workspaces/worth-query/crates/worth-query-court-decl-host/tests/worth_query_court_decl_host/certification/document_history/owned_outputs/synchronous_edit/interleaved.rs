//! One existing cabinet world exercises authored work between real producer waves.
use super::envelope_fixture as fixture;
use super::*;
mod courts;
pub(in super::super) mod created;
pub(in super::super) mod dependent;
mod occurrence_topology;
mod occurrences;
mod program;
mod reading;
pub(in super::super) mod seal;
use program::{ModelTree, ProducedTree, Program};
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    occurrence_topology::declare(created::declare(reading::declare(seal::declare(
        dependent::declare(schema),
    ))))
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    setup.handler::<created::Binding, _>(created::Handler)?;
    setup.handler::<dependent::Binding, _>(dependent::Handler)?;
    setup.handler::<seal::Binding, _>(seal::Handler)
}
pub fn seed(
    graph: &mut WorthQueryPrimaryGraphBootstrap<Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    occurrences::seed(graph)?;
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
