//! Ordinary complete structural Source profile; managed custody comes with its provider.
use super::*;
use worth_query_decl::facade::{application_query::*, worth_query_structured_value_binding};
use worth_query_host::facade::primary_graph::*;
mod binding;
mod projection;
mod selectors;
pub use binding::*;
use selectors::*;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SceneFaceRow {
    pub face: (u32, u64, u32),
    pub anchor: (u32, u64, u32),
    pub face_key: String,
    pub anchor_key: String,
    /// Anchor, successor, predecessor, in Metre/model-local.v1.
    pub points: [[u64; 2]; 3],
}
pub struct SceneFacesParameters;
worth_query_structured_value_binding!(pub SceneFacesParametersBinding for SceneFacesParameters { identity: "worth.query.scene-face.parameters.v1" });
worth_query_structured_value_binding!(pub SceneFaceRowBinding for SceneFaceRow { identity: "worth.query.scene-face.row.v1" });
pub struct SceneFacesQuery;
impl<S: SceneFaceSchemaBinding> ApplicationQueryMarkerIdentity<S> for SceneFacesQuery {
    type ParameterBinding = SceneFacesParametersBinding;
    type ResultBinding = SceneFaceRowBinding;
    type Scope = Scene;
    const IDENTIFIER: &'static str = "SceneFaces";
    const QUERY_TYPE_NAME: &'static str = "worth.query.scene-face.source.v1";
    const SCOPE_TYPE_NAME: &'static str = "Scene";
}
pub(super) fn definition<S: SceneFaceSchemaBinding>(
) -> ApplicationQueryDefinition<S, SceneFacesQuery, SceneFacesParameters, SceneFaceRow, Scene> {
    let shape = ApplicationQueryResultShapeBuilder::<
        S,
        SceneFacesQuery,
        Body,
        SceneFaceRow,
        SceneFaceRowBinding,
    >::new(Body::reference())
    .field(anchor_key())
    .field(root_x())
    .field(root_y())
    .relation(
        next(),
        ApplicationQueryResultShapeBuilder::<
            S,
            SceneFacesQuery,
            Body,
            SceneFaceRow,
            SceneFaceRowBinding,
        >::new(Body::reference())
        .field(next_x())
        .field(next_y()),
    )
    .relation(
        previous(),
        ApplicationQueryResultShapeBuilder::<
            S,
            SceneFacesQuery,
            Body,
            SceneFaceRow,
            SceneFaceRowBinding,
        >::new(Body::reference())
        .field(previous_x())
        .field(previous_y()),
    )
    .relation(
        face(),
        ApplicationQueryResultShapeBuilder::<
            S,
            SceneFacesQuery,
            Face,
            SceneFaceRow,
            SceneFaceRowBinding,
        >::new(Face::reference())
        .field(face_key()),
    )
    .build();
    ApplicationQueryDefinitionBuilder::declare(ApplicationQueryReference::from_declaration())
        .root(Body::reference())
        .scope(Scene::reference())
        .result_shape(shape)
        .cardinality(ApplicationQueryCardinality::Many)
        .disclosure(ApplicationQueryDisclosureContract::public())
        .basis_support(ApplicationQueryBasisSupport::current_and_pinned())
        .lanes(ApplicationQueryLaneEligibility::one_shot())
        .public()
        .root_path(
            ApplicationQueryRootPath::from(Scene::reference())
                .forward(SceneFace::reference())
                .forward(FaceAnchor::reference()),
        )
        .build()
        .expect("the scene face Source profile is canonical")
}
