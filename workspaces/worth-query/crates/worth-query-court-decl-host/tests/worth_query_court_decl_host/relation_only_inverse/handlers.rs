//! Spawn links its spawned node; Despawn removes exactly the links it names.
use super::*;
pub struct Handler;
impl OperationHandler<Schema, SpawnBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[
            WorthQueryOperationDurableCapability::RelationalMutation,
            WorthQueryOperationDurableCapability::RetainedCorrectionMaterial,
        ])
    }
    fn decide(
        &self,
        _: &Spawn,
        reader: &mut DecisionReader<'_, '_, '_, Schema, SpawnBinding>,
    ) -> HandlerResult<Target, Refused> {
        let result = (|| {
            let scope = reader.resolve_entity(Id::at("Id"), SCOPE)?;
            // The scope's own links are an ordinary admitted read. They cannot
            // describe the spawned node, which does not exist before this action.
            reader
                .relations_to(Holds::reference(), &scope)
                .map_err(HandlerExecutionDenial::new)?;
            reader.mutation_target(&scope)
        })();
        match result {
            Ok(scope) => HandlerResult::Completed(scope),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Spawn,
        scope: Target,
        writer: &mut CandidateWriter<'_, Schema, SpawnBinding>,
    ) -> HandlerResult<Done, Refused> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            let scope = writer.projected_entity(&scope)?;
            let born = |writer: &mut CandidateWriter<'_, Schema, SpawnBinding>, id: u64| {
                let node = writer.create_entity_in_context(
                    &scope,
                    Node::reference(),
                    WorthQueryApplicationEntityKey::new(format!("node-{id}")).unwrap(),
                )?;
                writer.initialize_field(&node, Id::at("Id"), id)?;
                Ok::<_, WorthQueryApplicationAttemptDenial>(node)
            };
            let spawned = born(writer, input.id)?;
            writer.create_output::<Spawned>(&spawned)?;
            let part = if input.part {
                let part = born(writer, input.id + 1)?;
                writer.create_output::<SpawnedPart>(&part)?;
                Some(part)
            } else {
                None
            };
            if input.link {
                writer.link(
                    Holds::reference(),
                    format!("holds-{}", input.id),
                    &spawned,
                    part.as_ref().unwrap_or(&scope),
                )?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Done),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
impl OperationHandler<Schema, DespawnBinding> for Handler {
    fn declared_durable_capabilities() -> Option<&'static [WorthQueryOperationDurableCapability]> {
        Some(&[WorthQueryOperationDurableCapability::RelationalMutation])
    }
    fn decide(
        &self,
        input: &Despawn,
        reader: &mut DecisionReader<'_, '_, '_, Schema, DespawnBinding>,
    ) -> HandlerResult<DespawnTargets, Refused> {
        let result = (|| {
            let spawned = reader.resolve_entity(Id::at("Id"), input.id)?;
            reader.entity(&spawned, Node::reference())?;
            let foreign = match input.foreign {
                Some(id) => Some(reader.resolve_entity(Id::at("Id"), id)?),
                None => None,
            };
            let mut links = Vec::new();
            for holder in std::iter::once(&spawned).chain(foreign.as_ref()) {
                for edge in reader
                    .relations_from(Holds::reference(), holder)
                    .map_err(HandlerExecutionDenial::new)?
                {
                    links.push((
                        reader.mutation_target(edge.from())?,
                        reader.mutation_target(edge.to())?,
                    ));
                }
            }
            let part = match input.part {
                Some(id) => {
                    let part = reader.resolve_entity(Id::at("Id"), id)?;
                    reader.entity(&part, Node::reference())?;
                    Some(reader.mutation_target(&part)?)
                }
                None => None,
            };
            Ok(DespawnTargets {
                spawned: reader.mutation_target(&spawned)?,
                part,
                links,
            })
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Despawn,
        targets: DespawnTargets,
        writer: &mut CandidateWriter<'_, Schema, DespawnBinding>,
    ) -> HandlerResult<Done, Refused> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            for (from, to) in targets.links {
                let from = writer.projected_entity(&from)?;
                let to = writer.projected_entity(&to)?;
                writer.unlink(Holds::reference(), &from, &to)?;
            }
            if let Some(part) = targets.part {
                let part = writer.projected_entity(&part)?;
                writer.delete_entity(Node::reference(), &part)?;
                writer.retire_output::<DespawnedPart>(&part)?;
            }
            let spawned = writer.projected_entity(&targets.spawned)?;
            writer.delete_entity(Node::reference(), &spawned)?;
            writer.retire_output::<Despawned>(&spawned)?;
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Done),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
