//! Issue and resolve exact prerequisite loans from the live coupled candidate.
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::inputs::RelationalRequiredOutputInputOwnerPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::producer_posture::RelationalCoupledProducerPostureOwnerPort as _;
use super::*;
pub(crate) use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::input_issuance::admission;
#[path = "inputs/settlement.rs"]
mod settlement;
use super::inputs::{InputIssuance, InputOrigin};
use crate::publication::data::AuthoredRetirementRefusal as Cause;

impl RelationalAuthoredCoupledCandidate<'_> {
    // The inverse prefix has already passed Native's exact payload/endpoint
    // validation. Only its resolved full-generation source can borrow a route.
    pub(super) fn restored_output_input(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        family: &str,
        original: crate::identity::data::EntityId,
        source: crate::identity::data::EntityId,
        partition: &[u8; 32],
    ) -> Result<RelationalRequiredOutputInput, TransactionCommitError> {
        let index = admission::restored_index(
            self.binding,
            execution,
            family,
            original,
            source,
            partition,
        )?;
        Ok(RelationalRequiredOutputInput::from_issued_input(
            self.identity.clone(),
            self.input_issuance(execution)?,
            InputOrigin::Restored { index, source },
        ))
    }

    fn input_issuance(
        &self,
        execution: &ExecutionRequest<'_, '_>,
    ) -> Result<Arc<InputIssuance>, TransactionCommitError> {
        if let Some(issuance) = self.input_issuance.borrow().as_ref() {
            return Ok(Arc::clone(issuance));
        }
        let bytes =
            (2 * std::mem::size_of::<usize>() + std::mem::size_of::<InputIssuance>()) as u64;
        let memory = execution
            .reserve_retained_memory(bytes)
            .map_err(custody::execution_stop)?;
        let issuance = Arc::new(InputIssuance { _memory: memory });
        *self.input_issuance.borrow_mut() = Some(Arc::clone(&issuance));
        Ok(issuance)
    }

    /// Carry a publication already authenticated at this candidate's actual C.
    pub fn current_output_input(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        publication: &RelationalRequiredOutputPublication,
    ) -> Result<RelationalRequiredOutputInput, TransactionCommitError> {
        admission::require_current(self.binding, execution, publication)?;
        Ok(RelationalRequiredOutputInput::from_issued_input(
            self.identity.clone(),
            self.input_issuance(execution)?,
            InputOrigin::Current(publication.clone()),
        ))
    }

    /// Only an actual earlier appended owned publication can supply this input.
    /// Ordinary participants carry no Native-owned publication to retain.
    pub fn applied_output_input(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        meaning: &RequiredOutputPublicationMeaning,
    ) -> Result<Option<RelationalRequiredOutputInput>, TransactionCommitError> {
        execution
            .run_native_with_checkpoint(|context| {
                for (index, part) in self.parts.iter().enumerate() {
                    checkpoint(context, meaning.family.len() as u64 + 65)
                        .map_err(custody::execution_stop)?;
                    if part.publication.producing_meaning() == Some(meaning) {
                        return if part.publication.owned().is_some() {
                            Ok(Some(RelationalRequiredOutputInput::from_issued_input(
                                self.identity.clone(),
                                self.input_issuance(execution)?,
                                InputOrigin::Applied(index),
                            )))
                        } else {
                            Ok(None)
                        };
                    }
                }
                Err(part::reject(
                    Cause::ForeignCoupledOwner,
                    meaning.source_scope,
                ))
            })
            .map_err(custody::execution_stop)?
            .0
    }

    /// Bind the complete closed prerequisite set to its actual performed consumer.
    /// A current read never rewrites the existing publication's recorded inputs.
    pub fn retain_output_inputs(
        &mut self,
        execution: &ExecutionRequest<'_, '_>,
        consumer: &RelationalRequiredOutputInput,
        parents: &[&RelationalRequiredOutputInput],
    ) -> Result<(), TransactionCommitError> {
        let issuance = self.input_issuance.borrow().as_ref().cloned();
        admission::retain(
            self.binding,
            self.identity,
            &mut self.parts,
            execution,
            issuance.as_ref(),
            consumer,
            parents,
        )
    }
}
