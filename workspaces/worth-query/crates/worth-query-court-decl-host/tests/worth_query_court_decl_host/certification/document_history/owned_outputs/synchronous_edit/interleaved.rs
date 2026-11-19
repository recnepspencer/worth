//! One existing cabinet world exercises authored work between real producer waves.
use super::envelope_fixture as fixture;
use super::*;
pub(in super::super) mod component_carriage;
pub(in super::super) use component_carriage::attach;
pub(in super::super) use component_carriage::{
    authored_parent::Binding as AuthoredParentBinding, pending::Input as PendingSeedInput,
    value_observation::Binding as ObserveValueBinding,
};
mod courts;
pub(in super::super) mod created;
pub(in super::super) mod dependent;
mod discovered_envelope;
mod occurrence_topology;
mod occurrences;
pub(in super::super) mod prerequisite_partition;
mod prerequisite_path;
mod program;
mod reading;
mod seed_children;
pub(in super::super) use seed_children::frontier::Binding as SeedFrontierBinding;
pub(in super::super) mod retirement_parent;
pub(in super::super) mod seal;
pub(in super::super) use program::ModelTree;
use program::{ProducedTree, Program};
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    prerequisite_path::declare(component_carriage::declare(
        prerequisite_partition::declare(occurrence_topology::declare(created::declare(
            reading::declare(seal::declare(dependent::declare(
                seed_children::frontier::declare(schema),
            ))),
        ))),
    ))
}
pub fn install(
    setup: &mut WorthQueryApplicationContributionSetup<'_, Schema>,
) -> Result<(), WorthQueryPrimaryGraphInstallationDenial> {
    component_carriage::install(setup)?;
    setup.handler::<prerequisite_partition::expectation::Binding, _>(
        prerequisite_partition::expectation::Handler,
    )?;
    setup.handler::<created::Binding, _>(created::Handler)?;
    setup.handler::<dependent::Binding, _>(dependent::Handler)?;
    setup.handler::<seed_children::frontier::Binding, _>(seed_children::frontier::Handler)?;
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
