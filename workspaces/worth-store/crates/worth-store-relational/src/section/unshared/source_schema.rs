//! Prepare the actual Home Source before the receiving Query compiler binds it.
use super::*;
use crate::section::source_preparation::receiver::HomeReceiverPreparation as State;
use worth_relational::facade::schema::RelationalSchemaRegistry;

impl UnrecoveredRelationalSection {
    pub fn prepare_home_existing_schema_source(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        acknowledged_target: Option<worth_foundational::facade::HomeProgramTransitionTargetMeaning>,
    ) -> Result<
        (
            &mut RelationalRuntime,
            &RelationalSchemaRegistry,
            Option<worth_relational::facade::identity::KindId>,
            Option<worth_foundational::facade::AspectIdentity>,
        ),
        RelationalSectionReadmissionDenial,
    > {
        use RelationalSectionReadmissionDenial as Denial;
        let prepared = self
            .prepared_source
            .as_mut()
            .ok_or(Denial::PreparedHomeSourceRequired)?;
        match &prepared.receiver {
            State::NotRequested => {
                let input = prepared
                    .original
                    .copy_home_input_for_request(request)
                    .map_err(Denial::InitialInputs)?;
                let recovery = request.in_scope(|_| {
                    self.attached.runtime.durability().prepare_home_recovery_source(
                        request, input, worth_relational::facade::durability::RecoveryVerificationMode::NormalRecoveryVerification)
                }).map_err(Denial::Execution)?.map_err(Denial::HomePreparation)?;
                prepared.receiver = State::Prepared(recovery);
            }
            State::Prepared(_) => (),
            State::Consumed => return Err(Denial::PreparedHomeSourceRequired),
        }
        match &prepared.receiver {
            State::Prepared(recovery) => {
                let (maximum_kind, maximum_aspect) = recovery
                    .occupied_identity_maxima(request)
                    .map_err(Denial::HomePreparation)?;
                Ok((
                    &mut self.attached.runtime,
                    match acknowledged_target {
                        Some(target) => {
                            recovery.schema_data(request, target.carrier(), target.schema())
                        }
                        None => recovery.receiver_schema_data(request),
                    }
                    .map_err(Denial::HomePreparation)?,
                    maximum_kind,
                    maximum_aspect,
                ))
            }
            State::NotRequested | State::Consumed => {
                unreachable!("prepared Home Source was retained")
            }
        }
    }
}
