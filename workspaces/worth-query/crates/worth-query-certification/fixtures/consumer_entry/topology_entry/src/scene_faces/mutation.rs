//! Ordinary authored membership. Native validates the actual anchored contour.
use super::*;
use worth_query_decl::facade::{
    application_operation::*, worth_query_mutation_binding, worth_query_operation,
    worth_query_operation_creates, worth_query_operation_links, worth_query_operation_reads,
    worth_query_operation_writes, worth_query_structured_value_binding,
};
use worth_query_host::facade::primary_graph::WorthQueryInvariantMutationTarget;
mod handler;
pub(super) use handler::AttachFaceHandler;

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct AttachPlanarFace {
    pub scope_key: String,
    pub scene_key: String,
    pub face_key: String,
}
#[derive(Debug, Eq, PartialEq)]
pub struct AttachPlanarFaceResult;
#[derive(Debug, Eq, PartialEq)]
pub enum AttachPlanarFaceDenial {
    FaceAlreadyExists,
    AmbiguousScene,
}
pub struct AttachFaceDecision<S> {
    anchor: WorthQueryInvariantMutationTarget<S, Body>,
    scene: Option<WorthQueryInvariantMutationTarget<S, Scene>>,
}
worth_query_structured_value_binding!(pub AttachFaceInputBinding for AttachPlanarFace { identity: "worth.query.scene-face.attach-input.v1" });
worth_query_structured_value_binding!(pub AttachFaceResultBinding for AttachPlanarFaceResult { identity: "worth.query.scene-face.attach-result.v1" });
worth_query_structured_value_binding!(pub AttachFaceDenialBinding for AttachPlanarFaceDenial { identity: "worth.query.scene-face.attach-denial.v1" });
worth_query_operation!(pub AttachFace for S: SceneFaceSchemaBinding, input AttachFaceInputBinding);
worth_query_operation_reads!(AttachFace => [Body, BodyKey, Scene, SceneKey, Face, FaceKey]);
worth_query_operation_writes!(AttachFace => [SceneKey, FaceKey]);
worth_query_operation_creates!(AttachFace => [Scene, Face]);
worth_query_operation_links!(AttachFace => [SceneFace, FaceAnchor]);
worth_query_mutation_binding! {
    pub AttachPlanarFaceBinding for AttachPlanarFace,
    schema generic S: SceneFaceSchemaBinding,
    identity "worth.query.scene-face.attach.v1",
    input AttachFaceInputBinding, operation AttachFace,
    result AttachFaceResultBinding,
    idempotency u64, identity "worth.query.scene-face.attach-command.v1",
    decision AttachFaceDecision<S>, denial AttachFaceDenialBinding,
    handler identity "worth.query.scene-face.attach-handler.v1",
    outputs NoApplicationMutationOutputs, source NoApplicationMutationSource,
    principal binding ConsumerPrincipalBinding::reference(),
        binding_type ConsumerPrincipalBinding, mapping ExternalPrincipalMapping,
        principal_entity Principal, principal_identity u64,
        identity_binding U64ApplicationValueBinding,
    scope Body, PlanarPosition, BodyKey, String, ReadOnly, NoApplicationUnit,
    field BodyKey::reference(), value_field scope_key,
}
pub(super) fn declare<S: SceneFaceSchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    let op = AttachFace::reference::<S>();
    schema
        .operation(op.definition().no_external_effect().no_aftermath().finish())
        .operation_projection_work_budget(op, 256)
        .operation_read_entity(op, Body::reference())
        .operation_read_entity(op, Scene::reference())
        .operation_read_entity(op, Face::reference())
        .operation_read_field(op, BodyKey::reference())
        .operation_read_field(op, SceneKey::reference())
        .operation_read_field(op, FaceKey::reference())
        .operation_write(op, SceneKey::reference())
        .operation_write(op, FaceKey::reference())
        .operation_create(op, Scene::reference())
        .operation_create(op, Face::reference())
        .operation_link(op, SceneFace::reference())
        .operation_link(op, FaceAnchor::reference())
        .application_mutation_binding::<AttachPlanarFaceBinding<S>>()
}
