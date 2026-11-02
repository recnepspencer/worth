//! Real operation handlers author the source; Native owns the derived restore.
use super::{lifecycle::*, *};
pub struct Handler;
impl<S: Scenario> OperationHandler<S, CreateBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Create,
        reader: &mut DecisionReader<'_, '_, '_, S, CreateBinding<S>>,
    ) -> HandlerResult<Target<S>, ()> {
        match reader
            .resolve_entity(Id::reference(), input.scope)
            .and_then(|entity| reader.mutation_target(&entity))
        {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Create,
        target: Target<S>,
        writer: &mut CandidateWriter<'_, S, CreateBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result = (|| {
            let scope = writer.projected_entity(&target)?;
            let created = writer.create_entity_in_context(
                &scope,
                Node::reference(),
                WorthQueryApplicationEntityKey::new(format!("authored-{}", input.id)).unwrap(),
            )?;
            writer.initialize_field(&created, Id::reference(), input.id)?;
            writer.initialize_field(&created, Value::reference(), 0)?;
            writer.create_output::<Created<S>>(&created)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
impl<S: Scenario> OperationHandler<S, RestoreBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Restore,
        reader: &mut DecisionReader<'_, '_, '_, S, RestoreBinding<S>>,
    ) -> HandlerResult<Target<S>, ()> {
        match reader
            .resolve_entity(Id::reference(), input.scope)
            .and_then(|entity| reader.mutation_target(&entity))
        {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        input: &Restore,
        target: Target<S>,
        writer: &mut CandidateWriter<'_, S, RestoreBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result = (|| {
            let scope = writer.projected_entity(&target)?;
            let created = writer.create_entity_in_context(
                &scope,
                Node::reference(),
                WorthQueryApplicationEntityKey::new(format!("authored-{}", input.id)).unwrap(),
            )?;
            writer.initialize_field(&created, Id::reference(), input.id)?;
            writer.initialize_field(&created, Value::reference(), input.value)?;
            writer.create_output::<Created<S>>(&created)
        })();
        match result {
            Ok(()) => HandlerResult::Completed(()),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
impl<S: Scenario> OperationHandler<S, RetireBinding<S>> for Handler {
    fn decide(
        &self,
        input: &Retire,
        reader: &mut DecisionReader<'_, '_, '_, S, RetireBinding<S>>,
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
        _: &Retire,
        targets: Vec<Target<S>>,
        writer: &mut CandidateWriter<'_, S, RetireBinding<S>>,
    ) -> HandlerResult<(), ()> {
        let result: Result<(), WorthQueryApplicationAttemptDenial> = (|| {
            for (index, target) in targets.into_iter().enumerate() {
                let entity = writer.projected_entity(&target)?;
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
