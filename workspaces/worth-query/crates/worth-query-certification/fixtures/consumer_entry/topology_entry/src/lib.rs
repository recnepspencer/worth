#![forbid(unsafe_code)]
// Threaded owner tests derive Send/Sync through nested persistent graph storage.
#![recursion_limit = "256"]

use fixture_values::PositiveLength;
use worth_query_consumer_values as fixture_values;
#[cfg(test)]
use worth_query_decl::facade::application_schema::ApplicationSchema;
use worth_query_host::facade as runtime_api;
use worth_query_host::facade::domain as runtime_domain;
#[cfg(test)]
use worth_query_host::facade::worth_query_conditional_node as conditional_node;

mod alternate_output;
mod application_program;
pub mod candidate_source_changes;
mod contribution;
mod document_history;
mod final_output;
mod handler;
mod initial_adjustment;
pub use document_history::{
    bind_navigation_grant, bind_retention_grants, PlanarDocumentHistory, PlanarHistoryBookmark,
    PlanarHistoryNavigation, PlanarHistoryWindow, PlanarSourceDocumentHistory,
    SetPlanarHistoryStatus, SetPlanarHistoryStatusBinding, SetPlanarHistoryStatusHandler,
};
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
mod owner_discovery;
pub use owner_discovery::*;
mod membership;
pub use membership::*;
mod escaping_output;
mod mutation;
pub use escaping_output::*;
mod owning_entry;
pub use owning_entry::capture_original;
mod island_coupling;
mod planar_edit;
mod planar_invariant;
mod planar_islands;
mod planar_output_read;
mod planar_read;
mod planar_topology;
mod planar_vertex;
mod principal;
mod principal_records;
pub use principal_records::*;
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
pub use island_coupling::*;
pub use mutation::*;
pub use planar_edit::*;
pub use planar_invariant::*;
pub use planar_islands::*;
pub use planar_output_read::*;
pub use planar_read::*;
pub use planar_topology::*;
pub use principal::*;
pub use prior_cycle_adjustment::*;
pub use producer::*;
pub use readiness::InitialPlanarReadiness;
pub use source_adjustment::*;

mod base_model;
#[path = "base_model/records.rs"]
mod storage_model;
pub use base_model::*;
pub mod domain_values;
mod length_binding;
pub use length_binding::TopologyLengthBinding;
mod schema_binding;
pub use schema_binding::TopologySchemaBinding;

use worth_query_decl::facade::application_schema::{fragment::*, *};
pub(crate) type TopologyCoreMembers<S, R = TopologyRecords> = (
    R,
    <S as TopologySchemaBinding>::PrincipalFragment,
    document_history::HistoryGrantRecords,
    TopologyContracts,
);
#[cfg(test)]
pub(crate) type CheckpointRecords = (checkpoint_recovery::computation_partition::facts::FactsRecords, checkpoint_recovery::computation_partition::oracle::collection::lifecycle::members::SupportRecords, checkpoint_recovery::computation_partition::entry_correction::capability::CorrectionGrantRecords);
#[cfg(test)]
pub(crate) type CheckpointMembers<S> = (
    CheckpointRecords,
    checkpoint_recovery::computation_partition::entry_correction::recorded::RecordedEntryCorrection<
        S,
    >,
);
pub struct TopologyContribution;
impl<S: TopologySchemaBinding> ApplicationSchemaContribution<S> for TopologyContribution {
    const IDENTITY: ApplicationSchemaContributionIdentity =
        ApplicationSchemaContributionIdentity::new("worth.query.certification.topology.v2");
    #[cfg(not(test))]
    type Members = TopologyCoreMembers<S>;
    #[cfg(test)]
    type Members = (TopologyCoreMembers<S>, CheckpointMembers<S>);
}
impl TopologyContribution {
    pub fn reference<S: TopologySchemaBinding>() -> ApplicationSchemaContributionRef<S, Self> {
        ApplicationSchemaContributionRef::from_contribution()
    }
}
pub struct TopologyContracts;
impl<Schema: TopologySchemaBinding> Registration<Schema> for TopologyContracts {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<Schema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<Schema>, ApplicationSchemaDeclarationDenial>
    {
        Ok({
            #[cfg(test)]
            let schema = checkpoint_recovery::required_chain::declare(schema);
            let schema = alternate_output::declare_alternate_output(schema);
            let schema = if Schema::ESCAPING_PLANAR_OUTPUT {
                escaping_output::declare(schema)
            } else {
                schema
            };
            let schema = island_coupling::declare(schema);
            let schema = vertex_replacement::declare_vertex_replacement(schema);
            let schema = prior_cycle_adjustment::declare_prior_cycle_adjustment(schema);
            let schema = <PlanarSourceFeatureDefinition<Schema> as worth_query_decl::facade::application_program::ApplicationFeatureDefinition<Schema>>::declare_members(schema);
            let schema = initial_adjustment::declare_initial_adjustment(schema);
            let schema = document_history::declare(schema);
            let schema = owner_discovery::declare(schema);
            let schema = <PlanarFinalFeatureDefinition<Schema> as worth_query_decl::facade::application_program::ApplicationFeatureDefinition<Schema>>::declare_members(schema);
            let schema = match Schema::PLANAR_EDIT_PROJECTION_WORK {
                Some(work) => {
                    schema.operation_projection_work_budget(EditPlanar::reference::<Schema>(), work)
                }
                None => schema,
            };
            let schema = base_model::declare(schema)
                .operation(
                    MutatePlanar::reference::<Schema>()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_projection_work_budget(MutatePlanar::reference::<Schema>(), 256)
                .operation_read_entity(
                    MutatePlanar::reference::<Schema>(),
                    Body::reference::<Schema>(),
                )
                .operation_read_field(MutatePlanar::reference::<Schema>(), BodyKey::at("BodyKey"))
                .operation_read_field(
                    MutatePlanar::reference::<Schema>(),
                    PositionX::at("PositionX"),
                )
                .operation_read_field(
                    MutatePlanar::reference::<Schema>(),
                    PositionY::at("PositionY"),
                )
                .operation_read_field(MutatePlanar::reference::<Schema>(), Length::at("Length"))
                .operation_read_relation(
                    MutatePlanar::reference::<Schema>(),
                    PlanarSuccessor::reference::<Schema>(),
                )
                .operation_create(
                    MutatePlanar::reference::<Schema>(),
                    Body::reference::<Schema>(),
                )
                .operation_write(MutatePlanar::reference::<Schema>(), BodyKey::at("BodyKey"))
                .operation_write(
                    MutatePlanar::reference::<Schema>(),
                    PositionX::at("PositionX"),
                )
                .operation_write(
                    MutatePlanar::reference::<Schema>(),
                    PositionY::at("PositionY"),
                )
                .operation_write(MutatePlanar::reference::<Schema>(), Length::at("Length"))
                .operation_link(
                    MutatePlanar::reference::<Schema>(),
                    PlanarSuccessor::reference::<Schema>(),
                )
                .operation_unlink(
                    MutatePlanar::reference::<Schema>(),
                    PlanarSuccessor::reference::<Schema>(),
                )
                .application_mutation_binding::<PlanarMutationBinding<Schema>>()
                .operation(
                    EditPlanar::reference::<Schema>()
                        .definition()
                        .no_external_effect()
                        .no_aftermath()
                        .finish(),
                )
                .operation_read_entity(
                    EditPlanar::reference::<Schema>(),
                    Body::reference::<Schema>(),
                )
                .operation_read_field(EditPlanar::reference::<Schema>(), BodyKey::at("BodyKey"))
                .operation_read_field(
                    EditPlanar::reference::<Schema>(),
                    PositionX::at("PositionX"),
                )
                .operation_read_field(
                    EditPlanar::reference::<Schema>(),
                    PositionY::at("PositionY"),
                )
                .operation_read_field(EditPlanar::reference::<Schema>(), Length::at("Length"))
                .operation_read_relation(
                    EditPlanar::reference::<Schema>(),
                    PlanarSuccessor::reference::<Schema>(),
                )
                .operation_create(
                    EditPlanar::reference::<Schema>(),
                    Body::reference::<Schema>(),
                )
                .operation_write(EditPlanar::reference::<Schema>(), BodyKey::at("BodyKey"))
                .operation_write(
                    EditPlanar::reference::<Schema>(),
                    PositionX::at("PositionX"),
                )
                .operation_write(
                    EditPlanar::reference::<Schema>(),
                    PositionY::at("PositionY"),
                )
                .operation_write(EditPlanar::reference::<Schema>(), Length::at("Length"))
                .operation_link(
                    EditPlanar::reference::<Schema>(),
                    PlanarSuccessor::reference::<Schema>(),
                )
                .operation_unlink(
                    EditPlanar::reference::<Schema>(),
                    PlanarSuccessor::reference::<Schema>(),
                )
                .application_mutation_binding::<PlanarEditBinding<Schema>>()
                .application_query(planar_query_definition::<Schema>())
                .application_query_binding::<PlanarReadBinding<Schema>>()
                .application_query(planar_discovery_definition::<Schema>())
                .application_query_binding::<PlanarDiscoveryBinding<Schema>>()
                .application_query(planar_output_query_definition::<Schema>())
                .application_query_binding::<PlanarOutputReadBinding<Schema>>();
            #[cfg(test)]
            let schema = checkpoint_recovery::computation_partition::declare(schema);
            schema
        })
    }
}
