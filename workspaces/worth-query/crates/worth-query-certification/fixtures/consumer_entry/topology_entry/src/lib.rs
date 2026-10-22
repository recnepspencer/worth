#![forbid(unsafe_code)]

use fixture_values::PositiveLength;
use worth_query_consumer_values as fixture_values;
#[cfg(test)]
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_decl::facade::worth_query_application_contribution;
use worth_query_host::facade as runtime_api;
use worth_query_host::facade::domain as runtime_domain;
#[cfg(test)]
use worth_query_host::facade::worth_query_conditional_node as conditional_node;

mod alternate_output;
mod application_program;
mod contribution;
mod final_output;
mod handler;
mod initial_adjustment;
pub mod optional_aspects;
pub use alternate_output::*;
pub use application_program::*;
pub use contribution::TopologyConfiguration;
pub use final_output::*;
pub use initial_adjustment::*;

#[cfg(test)]
mod checkpoint_recovery;
#[cfg(test)]
mod compiled_example_drift;
mod discovery_connection;
mod discovery_read;
pub use discovery_connection::*;
mod membership;
pub use membership::*;
mod mutation;
mod owning_entry;
pub use owning_entry::capture_original;
mod planar_edit;
mod planar_invariant;
mod planar_output_read;
mod planar_read;
mod planar_topology;
mod principal;
mod prior_cycle_adjustment;
mod producer;
mod readiness;
pub mod sensor_telemetry;
mod source_adjustment;
mod vertex_replacement;
pub mod warehouse_inventory;
pub use vertex_replacement::*;

pub use discovery_read::*;
pub use handler::*;
pub use mutation::*;
pub use planar_edit::*;
pub use planar_invariant::*;
pub use planar_output_read::*;
pub use planar_read::*;
pub use planar_topology::*;
pub use principal::*;
pub use prior_cycle_adjustment::*;
pub use producer::*;
pub use readiness::InitialPlanarReadiness;
pub use source_adjustment::*;

mod base_model;
pub use base_model::*;

worth_query_application_contribution! {
    pub contribution TopologyContribution for Schema: TopologySchemaBinding {
        identity: "worth.query.certification.topology.v2",
        members: |schema| {
            #[cfg(test)]
            let schema = checkpoint_recovery::required_chain::declare(schema);
            #[cfg(test)]
            let schema = checkpoint_recovery::computation_partition::declare(schema);
            let schema = alternate_output::declare_alternate_output(schema);
            let schema = vertex_replacement::declare_vertex_replacement(schema);
            let schema = prior_cycle_adjustment::declare_prior_cycle_adjustment(schema);
            let schema = <PlanarSourceFeatureDefinition<Schema> as worth_query_decl::facade::application_program::ApplicationFeatureDefinition<Schema>>::declare_members(schema);
            let schema = initial_adjustment::declare_initial_adjustment(schema);
            let schema = <PlanarFinalFeatureDefinition<Schema> as worth_query_decl::facade::application_program::ApplicationFeatureDefinition<Schema>>::declare_members(schema);
            let schema = match Schema::PLANAR_EDIT_PROJECTION_WORK {
                Some(work) => schema.operation_projection_work_budget(EditPlanar::reference::<Schema>(), work),
                None => schema,
            };
            base_model::declare(schema)
                .operation(MutatePlanar::reference::<Schema>().definition().no_external_effect().no_aftermath().finish())

                .operation_projection_work_budget(MutatePlanar::reference::<Schema>(), 256)
                .operation_read_entity(MutatePlanar::reference::<Schema>(), Body::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), BodyKey::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), PositionX::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), PositionY::reference::<Schema>())
                .operation_read_field(MutatePlanar::reference::<Schema>(), Length::reference::<Schema>())
                .operation_read_relation(MutatePlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .operation_create(MutatePlanar::reference::<Schema>(), Body::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), BodyKey::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), PositionX::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), PositionY::reference::<Schema>())
                .operation_write(MutatePlanar::reference::<Schema>(), Length::reference::<Schema>())
                .operation_link(MutatePlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .operation_unlink(MutatePlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .application_mutation_binding::<PlanarMutationBinding<Schema>>()
                .operation(EditPlanar::reference::<Schema>().definition().no_external_effect().no_aftermath().finish())

                .operation_read_entity(EditPlanar::reference::<Schema>(), Body::reference::<Schema>())
                .operation_read_field(EditPlanar::reference::<Schema>(), BodyKey::reference::<Schema>())
                .operation_read_field(EditPlanar::reference::<Schema>(), PositionX::reference::<Schema>())
                .operation_read_field(EditPlanar::reference::<Schema>(), PositionY::reference::<Schema>())
                .operation_read_field(EditPlanar::reference::<Schema>(), Length::reference::<Schema>())
                .operation_read_relation(EditPlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .operation_create(EditPlanar::reference::<Schema>(), Body::reference::<Schema>())
                .operation_write(EditPlanar::reference::<Schema>(), BodyKey::reference::<Schema>())
                .operation_write(EditPlanar::reference::<Schema>(), PositionX::reference::<Schema>())
                .operation_write(EditPlanar::reference::<Schema>(), PositionY::reference::<Schema>())
                .operation_write(EditPlanar::reference::<Schema>(), Length::reference::<Schema>())
                .operation_link(EditPlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .operation_unlink(EditPlanar::reference::<Schema>(), PlanarSuccessor::reference::<Schema>())
                .application_mutation_binding::<PlanarEditBinding<Schema>>()
                .application_query(planar_query_definition::<Schema>())
                .application_query_binding::<PlanarReadBinding<Schema>>()
                .application_query(planar_discovery_definition::<Schema>())
                .application_query_binding::<PlanarDiscoveryBinding<Schema>>()
                .application_query(planar_output_query_definition::<Schema>())
                .application_query_binding::<PlanarOutputReadBinding<Schema>>()
        }
    }
}
