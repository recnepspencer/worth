//! Real scene membership and supported contour faces, without retained geometry.
use super::*;
use worth_query_decl::facade::{application_schema::*, worth_query_application_contribution};
use worth_query_host::facade::application_contribution::*;

mod box_provider;
mod box_query;
mod model;
mod mutation;
mod query;
mod triangle;
pub use box_provider::{SceneBoxObservations, SceneFaceBoxIndex, SceneFaceConfiguration};
pub use box_query::*;
pub use model::*;
pub use mutation::*;
pub use query::*;
pub use triangle::TriangleFaceContour;

/// Explicit opt-in to the scene/triangle capability alongside the topology model.
pub trait SceneFaceSchemaBinding: TopologySchemaBinding {}

pub struct SceneFacesFeature;
impl<S: SceneFaceSchemaBinding> worth_query_decl::facade::application_program::ApplicationFeature<S>
    for SceneFacesFeature
{
    type Inputs = worth_query_decl::facade::application_program::ApplicationFeatureInputLeaf;
    const IDENTITY: &'static str = "worth.query.scene-face.feature.v1";
}

worth_query_application_contribution! {
    pub contribution SceneFaceContribution for S: SceneFaceSchemaBinding {
        identity: "worth.query.certification.scene-faces.v1",
        members: |schema| {
            let schema = model::declare(schema);
            let schema = mutation::declare(schema);
            schema.invariant(triangle::definition::<S>())
                .application_query(query::definition::<S>())
                .application_query_binding::<SceneFacesReadBinding<S>>()
                .application_query(box_query::definition::<S>())
                .application_query_binding::<SceneFaceBoxesReadBinding<S>>()
                .managed_index(worth_query_decl::facade::application_query::ApplicationManagedIndexDefinition::<S,SceneFaceBoxIndex<S>>::new())
        }
    }
}

impl<S: SceneFaceSchemaBinding> WorthQueryApplicationContribution<S> for SceneFaceContribution {
    type Configuration = SceneFaceConfiguration;
    fn configure(
        configuration: SceneFaceConfiguration,
        setup: &mut WorthQueryApplicationContributionSetup<'_, S>,
    ) -> Result<(), worth_query_host::facade::primary_graph::WorthQueryPrimaryGraphInstallationDenial>
    {
        setup
            .managed_index_provider::<SceneFaceBoxIndex<S>, _>(box_provider::Provider(
                configuration.observations,
            ))
            .with_predicate_routing()
            .install()?;
        setup.handler::<AttachPlanarFaceBinding<S>, _>(mutation::AttachFaceHandler)?;
        setup.invariant(
            TriangleFaceContour::reference(),
            ApplicationInvariantExecutionPoint::CommitBoundary,
            triangle::resolve::<S>,
        )
    }
}
