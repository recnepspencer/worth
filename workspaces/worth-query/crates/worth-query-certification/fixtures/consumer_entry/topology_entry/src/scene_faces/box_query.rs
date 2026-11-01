//! Candidate enclosure overlap; this query does not promise exact face contact.
use super::*;
use worth_query_decl::facade::{application_query::*, worth_query_structured_value_binding};
mod binding;
mod probe;
pub use binding::*;
pub use probe::*;

pub struct SceneBoxParameters {
    pub probe: SceneBoxProbe,
}
worth_query_structured_value_binding!(pub SceneBoxParametersBinding for SceneBoxParameters { identity: "worth.query.scene-face.box-parameters.v1" });
pub struct SceneFaceBoxesQuery;
impl<S: SceneFaceSchemaBinding> ApplicationQueryMarkerIdentity<S> for SceneFaceBoxesQuery {
    type ParameterBinding = SceneBoxParametersBinding;
    type ResultBinding = SceneFaceRowBinding;
    type Scope = Scene;
    const IDENTIFIER: &'static str = "SceneFaceBoxes";
    const QUERY_TYPE_NAME: &'static str = "worth.query.scene-face.box-candidates.v1";
    const SCOPE_TYPE_NAME: &'static str = "Scene";
}
pub struct BoxProbeParameter;
fn probe_parameter(
) -> ApplicationQueryParameterRef<SceneFaceBoxesQuery, BoxProbeParameter, SceneBoxProbeBinding> {
    ApplicationQueryParameterRef::from_query_identifier("probe")
}
pub(super) fn definition<S: SceneFaceSchemaBinding>(
) -> ApplicationQueryDefinition<S, SceneFaceBoxesQuery, SceneBoxParameters, SceneFaceRow, Scene> {
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::from_declaration())
        .root(Body::reference())
        .scope(Scene::reference())
        .result_shape(query::shape())
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_only())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .managed_index(
            super::box_provider::SceneFaceBoxIndex::<S>::reference(),
            probe_parameter(),
        )
        .order_by(
            query::selectors::anchor_key(),
            ApplicationQueryOrderingDirection::Ascending,
        )
        .build()
        .expect("scene enclosure candidate query is canonical")
}
