//! Genuine receiver installation precedes the single prepared source replay.
use super::*;
use crate::durability::authority::{
    DurabilityRecoveryAuthority, RecoveredRelationalRuntimeAuthority,
};
use crate::runtime::{
    RecoveryOutcome, RelationalHomeProgramCatalogReadmissionSource,
    RelationalInitialSchemaInstallationReceipt,
};
use worth_proof::AdmittedHomeProgramTransitionCatalog;

impl DurabilityRecoveryAuthority<'_> {
    pub fn recover_prepared_home_source(
        &mut self,
        request: &ExecutionRequest<'_, '_>,
        mut prepared: PreparedRelationalHomeRecovery,
        expected: &RelationalInitialSchemaInstallationReceipt,
        transition: Option<
            &AdmittedHomeProgramTransitionCatalog<
                RelationalHomeProgramCatalogReadmissionSource<'_>,
            >,
        >,
    ) -> Result<
        (RecoveryOutcome, RecoveredRelationalRuntimeAuthority),
        RefusedPreparedRelationalHomeRecovery,
    > {
        use RefusedPreparedRelationalHomeRecovery as Refusal;
        let ready = (|| {
            request.consult().map_err(Denial::Request)?;
            if prepared.state.receiver != self.runtime.runtime_instance_id() {
                return Err(Denial::ForeignReceiver);
            }
            expected
                .require_same_home_receiver(self.runtime, request)
                .map_err(Denial::ReceiverInstallation)?;
            prepared
                .state
                .graph
                .bind_home_executables(
                    &self.runtime.config().schema.registry.content_formats,
                    request,
                )
                .map_err(Denial::Recovery)?;
            prepared
                .state
                .graph
                .bind_directories(&mut prepared.state.plan, request)
                .map_err(Denial::Recovery)?;
            crate::durability::access::rebind_home_receiver_plan(
                self.runtime,
                &mut prepared.state.plan,
            );
            Ok(())
        })();
        if let Err(cause) = ready {
            return Err(Refusal::Preparation { prepared, cause });
        }
        let HomeRecoveryPreparation {
            plan,
            graph,
            receiver: _,
            _memory,
        } = *prepared.state;
        // Ordinary recovery admission remains authoritative. This is schema
        // DATA reuse, never an "already verified" bypass.
        let admitted = super::super::admit_recovery_or_emit(self.runtime, plan)
            .map_err(|cause| Refusal::Recovery(cause.into()))?
            .with_prepared_home_schemas(graph);
        match transition {
            None => self
                .recover_admitted_with_authority(request, admitted)
                .map_err(Refusal::Recovery),
            Some(transition) => self
                .recover_admitted_home_program_transition_source(request, admitted, transition)
                .map(|readmitted| {
                    readmitted.consume(|source| {
                        let (outcome, authority, _original_predecessor) = source.into_parts();
                        (outcome, authority)
                    })
                })
                .map_err(Refusal::Transition),
        }
    }
}
