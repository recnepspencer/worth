//! A real triangle is born inside an existing Scene's actual Native context.
use super::*;
use worth_query_decl::facade::{
    application_operation::*, worth_query_mutation_binding, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;
mod handler;
pub(super) use handler::CreateTriangleHandler;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct CreateSceneTriangle {
    pub scene_key: String,
    pub face_key: String,
    pub points: [[PositiveLength; 2]; 3],
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CreateSceneTriangleDenial {
    FaceKeyTaken,
    BodyKeyTaken,
}
#[derive(Debug, Eq, PartialEq)]
pub struct CreateSceneTriangleResult;
worth_query_structured_value_binding!(pub SceneTriangleInputBinding for CreateSceneTriangle { identity:"worth.query.scene-face.create-triangle-input.v1" });
worth_query_structured_value_binding!(pub SceneTriangleResultBinding for CreateSceneTriangleResult { identity:"worth.query.scene-face.create-triangle-result.v1" });
worth_query_structured_value_binding!(pub SceneTriangleDenialBinding for CreateSceneTriangleDenial { identity:"worth.query.scene-face.create-triangle-denial.v1" });
worth_query_operation!(pub CreateTriangle for S:SceneFaceSchemaBinding,input SceneTriangleInputBinding);
worth_query_operation_reads!(CreateTriangle => [Scene,SceneKey,Face,FaceKey,Body,BodyKey]);
worth_query_operation_writes!(CreateTriangle => [FaceKey,BodyKey,PositionX,PositionY,Length]);
worth_query_operation_creates!(CreateTriangle => [Face,Body]);
worth_query_operation_links!(CreateTriangle => [SceneFace,FaceAnchor,PlanarSuccessor]);
worth_query_mutation_binding! {
    pub CreateSceneTriangleBinding for CreateSceneTriangle,
    schema generic S:SceneFaceSchemaBinding,
    identity "worth.query.scene-face.create-triangle.v1",
    input SceneTriangleInputBinding,operation CreateTriangle,result SceneTriangleResultBinding,
    idempotency u64,identity "worth.query.scene-face.create-triangle-command.v1",
    decision WorthQueryInvariantMutationTarget<S,Scene>,denial SceneTriangleDenialBinding,
    handler identity "worth.query.scene-face.create-triangle-handler.v1",
    outputs NoApplicationMutationOutputs,source NoApplicationMutationSource,
    principal binding ConsumerPrincipalBinding::reference(),
        binding_type ConsumerPrincipalBinding,mapping ExternalPrincipalMapping,
        principal_entity Principal,principal_identity u64,identity_binding U64ApplicationValueBinding,
    scope Scene,SceneFacts,SceneKey,String,ReadOnly,NoApplicationUnit,
    field SceneKey::reference(),value_field scene_key,
}
pub(super) fn vertices(
    input: &CreateSceneTriangle,
) -> [worth_query_consumer_values::PlanarVertex; 3] {
    std::array::from_fn(|i| worth_query_consumer_values::PlanarVertex {
        body_key: format!("face:{}:vertex:{i}", input.face_key),
        x: input.points[i][0],
        y: input.points[i][1],
    })
}
pub(super) fn declare<S: SceneFaceSchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = CreateTriangle::reference::<S>();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 256)
        .operation_read_entity(op, Scene::reference())
        .operation_read_field(op, SceneKey::reference())
        .operation_read_entity(op, Face::reference())
        .operation_read_field(op, FaceKey::reference())
        .operation_read_entity(op, Body::reference())
        .operation_read_field(op, BodyKey::reference())
        .operation_create(op, Body::reference())
        .operation_create(op, Face::reference())
        .operation_write(op, FaceKey::reference())
        .operation_write(op, BodyKey::reference())
        .operation_write(op, PositionX::reference())
        .operation_write(op, PositionY::reference())
        .operation_write(op, Length::reference())
        .operation_link(op, SceneFace::reference())
        .operation_link(op, FaceAnchor::reference())
        .operation_link(op, PlanarSuccessor::reference())
        .application_mutation_binding::<CreateSceneTriangleBinding<S>>()
}
