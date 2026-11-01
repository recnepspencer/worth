//! Native identity and structural membership; coordinates stay on Body.
use super::*;
use worth_query_decl::facade::{worth_query_aspect, worth_query_entity, worth_query_field};

worth_query_entity!(pub Scene for S: SceneFaceSchemaBinding);
worth_query_entity!(pub Face for S: SceneFaceSchemaBinding);
worth_query_aspect!(pub SceneFacts for S: SceneFaceSchemaBinding, Scene;
    identity = AspectIdentity(0x9195_1001), revision = AspectContractRevision(1));
worth_query_aspect!(pub FaceFacts for S: SceneFaceSchemaBinding, Face;
    identity = AspectIdentity(0x9195_1002), revision = AspectContractRevision(1));
worth_query_field!(pub SceneKey for S: SceneFaceSchemaBinding, Scene, SceneFacts:
    String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub FaceKey for S: SceneFaceSchemaBinding, Face, FaceFacts:
    String => StringApplicationValueBinding, read_only, equality);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SceneFace;
impl SceneFace {
    pub const fn reference<S: SceneFaceSchemaBinding>(
    ) -> ApplicationRelationRef<S, Self, Scene, Face> {
        ApplicationRelationRef::from_schema_identifiers(
            "SceneFace",
            "Scene",
            "Face",
            ApplicationRelationIntegrity::new(
                ApplicationRelationEndpoints::same_context(false),
                ApplicationRelationCardinality::new(None, None, Some(1), Some(1), None, Some(1)),
                ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
            ),
        )
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FaceAnchor;
impl FaceAnchor {
    pub const fn reference<S: SceneFaceSchemaBinding>(
    ) -> ApplicationRelationRef<S, Self, Face, Body> {
        ApplicationRelationRef::from_schema_identifiers(
            "FaceAnchor",
            "Face",
            "Body",
            ApplicationRelationIntegrity::new(
                ApplicationRelationEndpoints::same_context(false),
                ApplicationRelationCardinality::new(Some(1), Some(1), None, Some(1), None, Some(1)),
                ApplicationRelationDeletionPolicy::RejectDeleteWithLiveRelations,
            ),
        )
    }
}

pub(super) fn declare<S: SceneFaceSchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .entity(Scene::reference())
        .entity(Face::reference())
        .aspect(Scene::reference(), SceneFacts::reference())
        .aspect(Face::reference(), FaceFacts::reference())
        .field(Scene::reference(), SceneKey::reference())
        .field(Face::reference(), FaceKey::reference())
        .relation(
            SceneFace::reference(),
            Scene::reference(),
            Face::reference(),
        )
        .relation(
            FaceAnchor::reference(),
            Face::reference(),
            Body::reference(),
        )
}
