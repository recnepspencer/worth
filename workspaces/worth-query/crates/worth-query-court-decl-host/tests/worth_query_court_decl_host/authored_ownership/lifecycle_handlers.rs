//! Real operation handlers author the source; Native owns the derived restore.
use super::{co_roles::*, lifecycle::*, *};
pub struct Handler;
impl<S: Scenario> OperationHandler<S, CreateBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Create,
        reader: &mut DecisionReader<'_, '_, '_, S, CreateBinding<S>>,
    ) -> HandlerResult<Target<S>, ()> {
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
            reader.mutation_target(&scope).map(Some)
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
        target: Target<S>,
        writer: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            let scope = writer.projected_entity(&target)?;
            let created = writer.create_entity_in_context(
                &scope,
                Node::reference(),
                WorthQueryApplicationEntityKey::new(format!("authored-{}", input.id)).unwrap(),
            )?;
            writer.initialize_field(&created, Id::reference(), input.id)?;
            writer.initialize_field(&created, Value::reference(), input.id)?;
            if S::MODE == 43 {
                writer
                    .initialize_content(&created, content::field(), &(input.id * 100))
                    .expect("valid installed word content");
            }
            writer.create_output::<Created<S>>(&created)?;
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
                if S::MODE == 27 && id == 11 {
                    writer.link(
                        SourceLink::reference(),
                        "authored-upstream-source",
                        &created,
                        &co,
                    )?;
                }
                if matches!(S::MODE, 21 | 30 | 39 | 40 | 41) {
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
mod restoration;
impl<S: Scenario> OperationHandler<S, RetireBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Retire,
        reader: &mut DecisionReader<'_, '_, '_, S, RetireBinding<S>>,
    ) -> HandlerResult<RetirementTargets<S>, ()> {
        let result = (|| {
            let mut targets = Vec::new();
            let mut edges = Vec::new();
            let mut stage_edges = Vec::new();
            let mut stage_relinks = Vec::new();
            for (index, id) in std::iter::once(&input.id)
                .chain(&input.co_anchors)
                .chain(&input.descendants)
                .enumerate()
            {
                let entity = reader.resolve_entity(Id::reference(), *id)?;
                reader.entity(&entity, Node::reference())?;
                // The authored inverse demands only its distinguished source's
                // pre-image. Native owns all derived publication payloads.
                if index <= input.co_anchors.len() {
                    reader.field(&entity, Id::reference())?;
                    reader.field(&entity, Value::reference())?;
                }
                if matches!(S::MODE, 5 | 13 | 19 | 20) {
                    for edge in reader
                        .relations_to(Peer::reference(), &entity)
                        .map_err(HandlerExecutionDenial::new)?
                    {
                        if matches!(S::MODE, 19 | 20) {
                            let from = reader
                                .field(edge.from(), Id::reference())?
                                .expect("business peer has its logical key");
                            if !input.descendants.contains(&from) {
                                continue;
                            }
                        }
                        edges.push((
                            reader.mutation_target(edge.from())?,
                            reader.mutation_target(&entity)?,
                        ));
                    }
                }
                if S::MODE == 43 && input.descendants.contains(id) {
                    for edge in reader
                        .relations_from(Peer::reference(), &entity)
                        .map_err(HandlerExecutionDenial::new)?
                    {
                        edges.push((
                            reader.mutation_target(edge.from())?,
                            reader.mutation_target(edge.to())?,
                        ));
                    }
                }
                if matches!(S::MODE, 30 | 39 | 40 | 41) {
                    relation_splice::read_retirement(
                        reader,
                        &entity,
                        &mut stage_edges,
                        &mut stage_relinks,
                        input.relation_fault,
                    )?;
                }
                if S::MODE == 21 {
                    for edge in reader
                        .relations_from(PreviousStage::reference(), &entity)
                        .map_err(HandlerExecutionDenial::new)?
                    {
                        stage_edges.push((
                            reader.mutation_target(edge.from())?,
                            reader.mutation_target(edge.to())?,
                        ));
                    }
                }
                targets.push(reader.mutation_target(&entity)?);
            }
            Ok(RetirementTargets {
                nodes: targets,
                edges,
                stage_edges,
                stage_relinks,
            })
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Retire,
        targets: RetirementTargets<S>,
        writer: &mut CandidateWriter<'_, S, RetireBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            for (from, to) in targets.stage_edges {
                let from = writer.projected_entity(&from)?;
                let to = writer.projected_entity(&to)?;
                writer.unlink(PreviousStage::reference(), &from, &to)?;
            }
            for (index, (from, to)) in targets.stage_relinks.into_iter().enumerate() {
                let from = writer.projected_entity(&from)?;
                let to = writer.projected_entity(&to)?;
                writer.link(
                    PreviousStage::reference(),
                    &format!("splice-{index}"),
                    &from,
                    &to,
                )?;
            }
            for (from, to) in targets.edges {
                let from = writer.projected_entity(&from)?;
                let to = writer.projected_entity(&to)?;
                writer.unlink(Peer::reference(), &from, &to)?;
            }
            for (index, target) in targets.nodes.into_iter().enumerate() {
                let entity = writer.projected_entity(&target)?;
                writer.delete_entity(Node::reference(), &entity)?;
                if index > 0 && index <= input.co_anchors.len() {
                    match index {
                        1 => writer.retire_output::<CoRetired<S, 0>>(&entity)?,
                        2 => writer.retire_output::<CoRetired<S, 1>>(&entity)?,
                        _ => unreachable!(),
                    }
                    continue;
                }
                match index.saturating_sub(input.co_anchors.len()) {
                    0 => writer.retire_output::<Retired<S>>(&entity)?,
                    1 => writer.retire_output::<Descendant<S, 0>>(&entity)?,
                    2 => writer.retire_output::<Descendant<S, 1>>(&entity)?,
                    3 => writer.retire_output::<Descendant<S, 2>>(&entity)?,
                    4 => writer.retire_output::<Descendant<S, 3>>(&entity)?,
                    5 => writer.retire_output::<Descendant<S, 4>>(&entity)?,
                    6 => writer.retire_output::<Descendant<S, 5>>(&entity)?,
                    _ => unreachable!("fixed roster has at most seven rows"),
                }
            }
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
impl<S: Scenario> OperationHandler<S, PlainBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Plain,
        reader: &mut DecisionReader<'_, '_, '_, S, PlainBinding<S>>,
    ) -> HandlerResult<Vec<Target<S>>, ()> {
        let result = (|| {
            let mut targets = Vec::new();
            for (index, id) in std::iter::once(&input.id)
                .chain(&input.descendants)
                .enumerate()
            {
                let entity = reader.resolve_entity(Id::reference(), *id)?;
                reader.entity(&entity, Node::reference())?;
                // The authored inverse demands only its distinguished source's
                // pre-image. Native owns all derived publication payloads.
                if index == 0 {
                    reader.field(&entity, Id::reference())?;
                    reader.field(&entity, Value::reference())?;
                }
                targets.push(reader.mutation_target(&entity)?);
            }
            Ok(targets)
        })();
        match result {
            Ok(targets) => HandlerResult::Completed(targets),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Plain,
        targets: Vec<Target<S>>,
        writer: &mut CandidateWriter<'_, S, PlainBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            for (index, target) in targets.into_iter().enumerate() {
                let entity = writer.projected_entity(&target)?;
                if index == 0 && input.replacement {
                    let created = writer.create_entity_in_context(
                        &entity,
                        Node::reference(),
                        WorthQueryApplicationEntityKey::new("mixed-replacement").unwrap(),
                    )?;
                    writer.initialize_field(&created, Id::reference(), 999)?;
                    writer.initialize_field(&created, Value::reference(), 0)?;
                    writer.create_output::<Extra<S>>(&created)?;
                }
                writer.delete_entity(Node::reference(), &entity)?;
                match index {
                    0 => writer.retire_output::<Retired<S>>(&entity)?,
                    1 => writer.retire_output::<Descendant<S, 0>>(&entity)?,
                    2 => writer.retire_output::<Descendant<S, 1>>(&entity)?,
                    3 => writer.retire_output::<Descendant<S, 2>>(&entity)?,
                    4 => writer.retire_output::<Descendant<S, 3>>(&entity)?,
                    5 => writer.retire_output::<Descendant<S, 4>>(&entity)?,
                    6 => writer.retire_output::<Descendant<S, 5>>(&entity)?,
                    _ => unreachable!("fixed roster has at most seven rows"),
                }
            }
            Ok(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
