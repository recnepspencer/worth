use super::*;
use worth_query_host::facade::primary_graph::{
    CandidateWriter, DecisionReader, HandlerExecutionDenial, HandlerResult, OperationHandler,
    WorthQueryApplicationEntityKey,
};
pub struct CreateTriangleHandler;
impl<S: SceneFaceSchemaBinding> OperationHandler<S, CreateSceneTriangleBinding<S>>
    for CreateTriangleHandler
{
    fn decide(
        &self,
        input: &CreateSceneTriangle,
        reader: &mut DecisionReader<'_, '_, '_, S, CreateSceneTriangleBinding<S>>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<S, Scene>, CreateSceneTriangleDenial> {
        let observed = (|| {
            let scene = reader.resolve_entity(SceneKey::reference(), input.scene_key.clone())?;
            if !reader
                .select_entities(FaceKey::reference(), input.face_key.clone(), 2)?
                .is_empty()
            {
                return Ok(Err(CreateSceneTriangleDenial::FaceKeyTaken));
            }
            for vertex in vertices(input) {
                if !reader
                    .select_entities(BodyKey::reference(), vertex.body_key, 2)?
                    .is_empty()
                {
                    return Ok(Err(CreateSceneTriangleDenial::BodyKeyTaken));
                }
            }
            Ok(Ok(reader.mutation_target(&scene)?))
        })();
        match observed {
            Ok(Ok(scene)) => HandlerResult::Completed(scene),
            Ok(Err(cause)) => HandlerResult::DomainDenied(cause),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &CreateSceneTriangle,
        scene: WorthQueryInvariantMutationTarget<S, Scene>,
        writer: &mut CandidateWriter<'_, S, CreateSceneTriangleBinding<S>>,
    ) -> HandlerResult<CreateSceneTriangleResult, CreateSceneTriangleDenial> {
        let written: Result<_, HandlerExecutionDenial> = (|| {
            let scene = writer
                .projected_entity(&scene)
                .map_err(HandlerExecutionDenial::new)?;
            let mut bodies = Vec::new();
            bodies
                .try_reserve_exact(3)
                .map_err(HandlerExecutionDenial::new)?;
            for vertex in vertices(input) {
                let key = WorthQueryApplicationEntityKey::new(&vertex.body_key)
                    .map_err(HandlerExecutionDenial::new)?;
                let body = writer
                    .create_entity_in_context(&scene, Body::reference(), key)
                    .map_err(HandlerExecutionDenial::new)?;
                crate::planar_vertex::initialize_planar_vertex(writer, &body, &vertex)?;
                bodies.push(body);
            }
            for i in 0..3 {
                writer
                    .link(
                        PlanarSuccessor::reference(),
                        format!("face:{}:successor:{i}", input.face_key),
                        &bodies[i],
                        &bodies[(i + 1) % 3],
                    )
                    .map_err(HandlerExecutionDenial::new)?;
            }
            let key = WorthQueryApplicationEntityKey::new(format!("face:{}", input.face_key))
                .map_err(HandlerExecutionDenial::new)?;
            let face = writer
                .create_entity_in_context(&scene, Face::reference(), key)
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
                    &bodies[0],
                )
                .map_err(HandlerExecutionDenial::new)?;
            Ok(CreateSceneTriangleResult)
        })();
        match written {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
