//! Real creation, current readback recording and retirement through ordinary owners.
use super::{
    edit::{Edited, MissingValue},
    schema::*,
    source_output_model::*,
};
use worth_query_host::facade::primary_graph::*;
pub struct IssueHandler;
impl OperationHandler<NumericSchema, IssueBinding> for IssueHandler {
    fn decide(
        &self,
        input: &Issue,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, IssueBinding>,
    ) -> HandlerResult<IssueDecision, MissingValue> {
        let root = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(root) => root,
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        let serial = match reader.field(&root, Secondary::reference()) {
            Ok(Some(value)) => value,
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(cause) => return HandlerResult::ExecutionDenied(cause),
        };
        if serial.checked_add(1).is_none() {
            return HandlerResult::DomainDenied(MissingValue);
        }
        match reader.mutation_target(&root) {
            Ok(target) => HandlerResult::Completed(IssueDecision { target, serial }),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
    fn build_candidate(
        &self,
        _: &Issue,
        decision: IssueDecision,
        writer: &mut CandidateWriter<'_, NumericSchema, IssueBinding>,
    ) -> HandlerResult<Issued, MissingValue> {
        match issue(decision, writer) {
            Ok(result) => HandlerResult::Completed(result),
            Err(cause) => HandlerResult::ExecutionDenied(cause),
        }
    }
}
fn issue(
    decision: IssueDecision,
    writer: &mut CandidateWriter<'_, NumericSchema, IssueBinding>,
) -> Result<Issued, HandlerExecutionDenial> {
    let parent = writer
        .projected_entity(&decision.target)
        .map_err(HandlerExecutionDenial::new)?;
    let key = format!("issued-{}", decision.serial);
    let decoy = format!("decoy-{}", decision.serial);
    // The decoy is allocated first; source material must select the bound Actual role.
    let decoy_entity = create(writer, &parent, &decoy, 777)?;
    writer
        .create_output::<Decoy>(&decoy_entity)
        .map_err(HandlerExecutionDenial::new)?;
    let actual = create(writer, &parent, &key, 17)?;
    writer
        .create_output::<Actual>(&actual)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .write_field(&parent, Secondary::reference(), decision.serial + 1)
        .map_err(HandlerExecutionDenial::new)?;
    Ok(Issued { key, decoy })
}
fn create(
    writer: &mut CandidateWriter<'_, NumericSchema, IssueBinding>,
    parent: &WorthQueryApplicationEffectEntity<NumericSchema, Number>,
    key: &str,
    payload: u64,
) -> Result<WorthQueryApplicationEffectEntity<NumericSchema, Number>, HandlerExecutionDenial> {
    let entity = writer
        .create_entity_in_context(
            parent,
            Number::reference(),
            WorthQueryApplicationEntityKey::new(key).map_err(HandlerExecutionDenial::new)?,
        )
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&entity, Key::reference(), key.to_owned())
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&entity, Primary::reference(), payload)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&entity, Secondary::reference(), 0)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&entity, ReadEnabled::reference(), true)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&entity, Ceiling::reference(), 1000)
        .map_err(HandlerExecutionDenial::new)?;
    writer
        .initialize_field(&entity, Calculated::reference(), 0)
        .map_err(HandlerExecutionDenial::new)?;
    Ok(entity)
}
pub struct RecordHandler;
static RECORDS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub fn record_contacts() -> usize {
    RECORDS.load(std::sync::atomic::Ordering::Relaxed)
}
impl OperationHandler<NumericSchema, RecordBinding> for RecordHandler {
    fn decide(
        &self,
        input: &Record,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, RecordBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        RECORDS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let entity = match reader.resolve_entity(Key::reference(), input.key.clone()) {
            Ok(e) => e,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        match reader.field(&entity, Primary::reference()) {
            Ok(Some(_)) => {}
            Ok(None) => return HandlerResult::DomainDenied(MissingValue),
            Err(e) => return HandlerResult::ExecutionDenied(e),
        }
        match reader.mutation_target(&entity) {
            Ok(t) => HandlerResult::Completed(t),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn build_candidate(
        &self,
        input: &Record,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, RecordBinding>,
    ) -> HandlerResult<Recorded, MissingValue> {
        let entity = match writer.projected_entity(&target) {
            Ok(e) => e,
            Err(e) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        };
        match writer.write_field(&entity, Primary::reference(), input.payload) {
            Ok(()) => HandlerResult::Completed(Recorded {
                original_key: input.original_key.clone(),
                payload: input.payload,
            }),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
pub struct RetireHandler;
impl OperationHandler<NumericSchema, RetireBinding> for RetireHandler {
    fn decide(
        &self,
        input: &Retire,
        reader: &mut DecisionReader<'_, '_, '_, NumericSchema, RetireBinding>,
    ) -> HandlerResult<WorthQueryInvariantMutationTarget<NumericSchema, Number>, MissingValue> {
        let entity = match reader.resolve_entity(Key::reference(), input.output_key.clone()) {
            Ok(e) => e,
            Err(e) => return HandlerResult::ExecutionDenied(e),
        };
        macro_rules! read {
            ($Field:ty) => {
                match reader.field(&entity, <$Field>::reference()) {
                    Ok(Some(_)) => {}
                    Ok(None) => return HandlerResult::DomainDenied(MissingValue),
                    Err(e) => return HandlerResult::ExecutionDenied(e),
                }
            };
        }
        read!(Primary);
        read!(Secondary);
        read!(ReadEnabled);
        read!(Ceiling);
        read!(Calculated);
        if let Err(cause) = reader.entity(&entity, Number::reference()) {
            return HandlerResult::ExecutionDenied(cause);
        }
        match reader.mutation_target(&entity) {
            Ok(t) => HandlerResult::Completed(t),
            Err(e) => HandlerResult::ExecutionDenied(e),
        }
    }
    fn build_candidate(
        &self,
        _: &Retire,
        target: WorthQueryInvariantMutationTarget<NumericSchema, Number>,
        writer: &mut CandidateWriter<'_, NumericSchema, RetireBinding>,
    ) -> HandlerResult<Edited, MissingValue> {
        let entity = match writer.projected_entity(&target) {
            Ok(e) => e,
            Err(e) => return HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        };
        match writer
            .delete_entity(Number::reference(), &entity)
            .and_then(|()| writer.retire_output::<Retired>(&entity))
        {
            Ok(()) => HandlerResult::Completed(Edited),
            Err(e) => HandlerResult::ExecutionDenied(HandlerExecutionDenial::new(e)),
        }
    }
}
