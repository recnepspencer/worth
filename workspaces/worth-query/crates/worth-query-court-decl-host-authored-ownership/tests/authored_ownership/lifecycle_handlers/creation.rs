//! Author the primary source and its explicitly declared co-born records.
use super::*;
impl<S: Scenario> OperationHandler<S, CreateBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Create,
        reader: &mut DecisionReader<'_, '_, '_, S, CreateBinding<S>>,
    ) -> HandlerResult<CreationTargets<S>, ()> {
        let result = (|| {
            let scope = reader.resolve_entity(Id::reference(), input.scope)?;
            if S::MODE == 5 && input.scope != 1 {
                let edges = reader
                    .relations_to(Peer::reference(), &scope)
                    .map_err(HandlerExecutionDenial::new)?;
                if edges.len() != 1
                    || reader.field(edges[0].from(), Id::reference())? != Some(input.scope * 10 + 1)
                {
                    return Ok(None);
                }
            }
            let previous = if S::MODE == 60 && input.id == 30 && input.scope == 10 {
                let previous = reader.resolve_entity(Id::reference(), 20)?;
                Some(reader.mutation_target(&previous)?)
            } else {
                None
            };
            Ok(Some(CreationTargets {
                scope: reader.mutation_target(&scope)?,
                previous,
            }))
        })();
        match result {
            Ok(Some(target)) => HandlerResult::Completed(target),
            Ok(None) => HandlerResult::DomainDenied(()),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Create,
        target: CreationTargets<S>,
        writer: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            let scope = writer.projected_entity(&target.scope)?;
            let created = writer.create_entity_in_context(
                &scope,
                Node::reference(),
                WorthQueryApplicationEntityKey::new(format!("authored-{}", input.id)).unwrap(),
            )?;
            writer.initialize_field(&created, Id::reference(), input.id)?;
            writer.initialize_field(
                &created,
                Value::reference(),
                if S::COUPLED && input.scope != 1 {
                    input.scope
                } else {
                    input.id
                },
            )?;
            if S::MODE == 43 {
                writer
                    .initialize_content(&created, content::field(), &(input.id * 100))
                    .expect("valid installed word content");
            }
            writer.create_output::<Created<S>>(&created)?;
            if let Some(previous) = &target.previous {
                let previous = writer.projected_entity(previous)?;
                writer.link(
                    PreviousStage::reference(),
                    "append-previous-stage",
                    &created,
                    &previous,
                )?;
            }
            <S::Heterogeneous as heterogeneous::Profile>::create(input, &scope, writer)?;
            let mut stage_records = Vec::new();
            for (index, &id) in input
                .co_anchors
                .iter()
                .enumerate()
                .filter(|_| input.create_co)
            {
                let co = writer.create_entity_in_context(
                    &scope,
                    Node::reference(),
                    WorthQueryApplicationEntityKey::new(format!("authored-{id}")).unwrap(),
                )?;
                writer.initialize_field(&co, Id::reference(), id)?;
                if S::MODE == 43 {
                    writer
                        .initialize_content(&co, content::field(), &(id * 100))
                        .expect("valid installed stage content");
                }
                writer.initialize_field(
                    &co,
                    Value::reference(),
                    if S::COUPLED { input.id } else { id },
                )?;
                match index {
                    0 => writer.create_output::<CoCreated<S, 0>>(&co)?,
                    1 => writer.create_output::<CoCreated<S, 1>>(&co)?,
                    _ => unreachable!(),
                }
                if S::MODE == 48 {
                    writer.link(Peer::reference(), "authored-feature-body", &co, &created)?;
                }
                if S::MODE == 27 && id == 11 {
                    writer.link(
                        SourceLink::reference(),
                        "authored-upstream-source",
                        &created,
                        &co,
                    )?;
                }
                if matches!(S::MODE, 21 | 30 | 39 | 40 | 41 | 60) {
                    stage_records.push(co);
                }
            }
            if matches!(S::MODE, 30 | 39 | 40 | 41) && input.depth > 0 && stage_records.len() == 2 {
                writer.link(
                    PreviousStage::reference(),
                    "stage20-previous10",
                    &stage_records[0],
                    &created,
                )?;
                writer.link(
                    PreviousStage::reference(),
                    "stage30-previous20",
                    &stage_records[1],
                    &stage_records[0],
                )?;
                if input.depth == 2 {
                    writer.link(
                        PreviousStage::reference(),
                        "foreign-unchanged-stage-link",
                        &created,
                        &stage_records[1],
                    )?;
                }
                writer.link(
                    StageVisible::reference(),
                    "authored-stage-visibility",
                    &created,
                    &stage_records[1],
                )?;
            }
            if S::MODE == 60 && stage_records.len() == 1 {
                writer.link(
                    PreviousStage::reference(),
                    "initial-stage-previous-body",
                    &stage_records[0],
                    &created,
                )?;
            }
            if S::MODE == 21 {
                writer.link(
                    PreviousStage::reference(),
                    "authored-stage-predecessor",
                    &stage_records[0],
                    &stage_records[1],
                )?;
                writer.link(
                    StageVisible::reference(),
                    "authored-stage-visibility",
                    &created,
                    &stage_records[1],
                )?;
            }
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
