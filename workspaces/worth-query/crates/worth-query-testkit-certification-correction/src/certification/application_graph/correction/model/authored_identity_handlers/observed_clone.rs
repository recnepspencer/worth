//! The snapshot payload is read from the same parent later edited by the court.
use super::super::observed_clone::{Binding, CloneNumber, Snapshot};
use super::*;

impl OperationHandler<NumericSchema, Binding> for Handler {
    fn decide(
        &self,
        input: &CloneNumber,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, Binding>,
    ) -> HandlerResult<Snapshot, MissingValue> {
        let parent = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(parent) => parent,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        macro_rules! read {
            ($Field:ty) => {
                match reader.field(&parent, <$Field>::reference()) {
                    Ok(Some(value)) => value,
                    Ok(None) => return HandlerResult::DomainDenied(MissingValue),
                    Err(cause) => return HandlerResult::ExecutionDenied(cause),
                }
            };
        }
        let key = read!(Key);
        if key != input.key || input.name == key {
            return HandlerResult::DomainDenied(MissingValue);
        }
        let image = Restore {
            key,
            name: input.name.clone(),
            primary: read!(Primary),
            secondary: read!(Secondary),
            read_enabled: read!(ReadEnabled),
            ceiling: read!(Ceiling),
            calculated: read!(Calculated),
        };
        if let Err(cause) = reader.entity(&parent, Number::reference()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&parent) {
            Ok(parent) => HandlerResult::Completed(Snapshot { parent, image }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &CloneNumber,
        Snapshot { parent, image }: Snapshot,
        writer: &mut CandidateWriter<'_, NumericSchema, Binding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let result = (|| {
            creation!(@image writer, parent, image);
            Ok::<(), HandlerExecutionDenial>(())
        })();
        match result {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
