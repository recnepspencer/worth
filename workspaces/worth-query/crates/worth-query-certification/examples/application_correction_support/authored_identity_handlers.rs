//! Lifecycle candidates use ordinary observed targets and native creation effects.
use super::{
    authored_identity_model::*,
    edit::{Edited, MissingValue},
    schema::*,
};
use worth_query_host::facade::primary_graph::*;
pub struct Handler;
macro_rules! creation {
    (@reference CreateBinding, $writer:ident, $parent:ident, $entity:ident, $input:ident) => {
        $writer.link(AuthoredReference::reference(), format!("authored-reference-{}", $input.name), &$parent, &$entity)
    };
    (@reference RestoreBinding, $writer:ident, $parent:ident, $entity:ident, $input:ident) => {
        Ok::<(), WorthQueryApplicationAttemptDenial>(())
    };
    ($Binding:ident, $Input:ty, $image:expr) => {
        impl OperationHandler<NumericSchema, $Binding> for Handler {
            fn decide(
                &self,
                input: &$Input,
                reader: &mut DecisionReader<'_, '_, '_, NumericSchema, $Binding>,
            ) -> HandlerResult<Target, MissingValue> {
                let parent = match reader.resolve_entity(Key::reference(), input.key.clone()) {
                    Ok(entity) => entity,
                    Err(cause) => return HandlerResult::ExecutionDenied(cause),
                };
                match reader.field(&parent, Primary::reference()) {
                    Ok(Some(_)) => (),
                    Ok(None) => return HandlerResult::DomainDenied(MissingValue),
                    Err(cause) => return HandlerResult::ExecutionDenied(cause),
                }
                match reader.mutation_target(&parent) {
                    Ok(target) => HandlerResult::Completed(target),
                    Err(cause) => HandlerResult::ExecutionDenied(cause),
                }
            }
            fn build_candidate(
                &self,
                input: &$Input,
                target: Target,
                writer: &mut CandidateWriter<'_, NumericSchema, $Binding>,
            ) -> HandlerResult<Edited, MissingValue> {
                let image: Restore = ($image)(input);
                let result = (|| {
                    let parent = writer
                        .projected_entity(&target)
                        .map_err(HandlerExecutionDenial::new)?;
                    let entity = writer
                        .create_entity_in_context(
                            &parent,
                            Number::reference(),
                            WorthQueryApplicationEntityKey::new(&image.name)
                                .map_err(HandlerExecutionDenial::new)?,
                        )
                        .map_err(HandlerExecutionDenial::new)?;
                    macro_rules! initialize {
                        ($Field:ty, $value:expr) => {
                            writer
                                .initialize_field(&entity, <$Field>::reference(), $value)
                                .map_err(HandlerExecutionDenial::new)?
                        };
                    }
                    initialize!(Key, image.name);
                    initialize!(Primary, image.primary);
                    initialize!(Secondary, image.secondary);
                    initialize!(ReadEnabled, image.read_enabled);
                    initialize!(Ceiling, image.ceiling);
                    initialize!(Calculated, image.calculated);
                    writer
                        .create_output::<Created>(&entity)
                        .map_err(HandlerExecutionDenial::new)?;
                    creation!(@reference $Binding, writer, parent, entity, input)
                        .map_err(HandlerExecutionDenial::new)
                })();
                match result {
                    Ok(()) => HandlerResult::Completed(Edited),
                    Err(cause) => HandlerResult::ExecutionDenied(cause),
                }
            }
        }
    };
}
creation!(CreateBinding, Create, |input: &Create| Restore {
    key: input.key.clone(),
    name: input.name.clone(),
    primary: input.payload,
    secondary: 3,
    read_enabled: true,
    ceiling: 1000,
    calculated: 0,
});
creation!(RestoreBinding, Restore, |input: &Restore| input.clone());
impl OperationHandler<NumericSchema, RetireBinding> for Handler {
    fn decide(
        &self,
        input: &Retire,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, RetireBinding>,
    ) -> HandlerResult<Target, MissingValue> {
        let entity = match reader.resolve_entity(Key::reference(), input.target.clone()) {
            Ok(entity) => entity,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        macro_rules! observe {
            ($Field:ty) => {
                match reader.field(&entity, <$Field>::reference()) {
                    Ok(Some(_)) => (),
                    Ok(None) => return HandlerResult::DomainDenied(MissingValue),
                    Err(cause) => return HandlerResult::ExecutionDenied(cause),
                }
            };
        }
        observe!(Key);
        observe!(Primary);
        observe!(Secondary);
        observe!(ReadEnabled);
        observe!(Ceiling);
        observe!(Calculated);
        if let Err(cause) = reader.entity(&entity, Number::reference()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&entity) {
            Ok(target) => HandlerResult::Completed(target),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Retire,
        target: Target,
        writer: &mut CandidateWriter<'_, NumericSchema, RetireBinding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let entity = match writer.projected_entity(&target) {
            Ok(entity) => entity,
            Err(cause) => {
                return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause))
            }
        };
        match writer
            .delete_entity(Number::reference(), &entity)
            .and_then(|()| writer.retire_output::<Retired>(&entity))
        {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(cause) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(cause)),
        }
    }
}
