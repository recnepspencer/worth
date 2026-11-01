use super::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey,
};
pub struct AttachFaceHandler;
impl<S: SceneFaceSchemaBinding> OperationHandler<S, AttachPlanarFaceBinding<S>>
    for AttachFaceHandler
{
    fn decide(
        &self,
        input: &AttachPlanarFace,
        reader: &mut DecisionReader<'_, '_, '_, S, AttachPlanarFaceBinding<S>>,
    ) -> HandlerResult<AttachFaceDecision<S>, AttachPlanarFaceDenial> {
        let observed = (|| {
            let anchor = reader.resolve_entity(BodyKey::reference(), input.scope_key.clone())?;
            let faces = reader.select_entities(FaceKey::reference(), input.face_key.clone(), 2)?;
            if !faces.is_empty() {
                return Ok(Err(AttachPlanarFaceDenial::FaceAlreadyExists));
            }
            let scenes =
                reader.select_entities(SceneKey::reference(), input.scene_key.clone(), 2)?;
            if scenes.len() > 1 {
                return Ok(Err(AttachPlanarFaceDenial::AmbiguousScene));
            }
            let scene = scenes
                .first()
                .map(|scene| reader.mutation_target(scene))
                .transpose()?;
            Ok(Ok(AttachFaceDecision {
                anchor: reader.mutation_target(&anchor)?,
                scene,
            }))
        })();
        match observed {
            Ok(Ok(decision)) => HandlerResult::Completed(decision),
            Ok(Err(denial)) => HandlerResult::DomainDenied(denial),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &AttachPlanarFace,
        decision: AttachFaceDecision<S>,
        writer: &mut CandidateWriter<'_, S, AttachPlanarFaceBinding<S>>,
    ) -> HandlerResult<AttachPlanarFaceResult, AttachPlanarFaceDenial> {
        let written: Result<_, HandlerExecutionDenial> = (|| {
            let anchor = writer
                .projected_entity(&decision.anchor)
                .map_err(HandlerExecutionDenial::new)?;
            let scene = match decision.scene {
                Some(scene) => writer
                    .projected_entity(&scene)
                    .map_err(HandlerExecutionDenial::new)?,
                None => {
                    let key =
                        WorthQueryApplicationEntityKey::new(format!("scene:{}", input.scene_key))
                            .map_err(HandlerExecutionDenial::new)?;
                    let scene = writer
                        .create_entity_in_context(&anchor, Scene::reference(), key)
                        .map_err(HandlerExecutionDenial::new)?;
                    writer
                        .initialize_field(&scene, SceneKey::reference(), input.scene_key.clone())
                        .map_err(HandlerExecutionDenial::new)?;
                    scene
                }
            };
            let key = WorthQueryApplicationEntityKey::new(format!("face:{}", input.face_key))
                .map_err(HandlerExecutionDenial::new)?;
            let face = writer
                .create_entity_in_context(&anchor, Face::reference(), key)
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .initialize_field(&face, FaceKey::reference(), input.face_key.clone())
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    SceneFace::reference(),
                    format!("scene-face:{}", input.face_key),
                    &scene,
                    &face,
                )
                .map_err(HandlerExecutionDenial::new)?;
            writer
                .link(
                    FaceAnchor::reference(),
                    format!("face-anchor:{}", input.face_key),
                    &face,
                    &anchor,
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(AttachPlanarFaceResult)
        })();
        match written {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
