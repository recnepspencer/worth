//! Genuine receiver installation precedes the single prepared source replay.
use super::{refusal::RefusedPreparedRelationalHomeRecovery, *};
use crate::durability::authority::recovered_runtime::DurabilityRecoveryAuthority;
use crate::durability::authority::recovered_runtime::RecoveredRelationalRuntimeAuthority;
use crate::{
    durability::data::RecoveryOutcome, runtime::RelationalHomeProgramCatalogReadmissionSource,
    runtime::RelationalInitialSchemaInstallationReceipt,
};
use worth_proof::AdmittedHomeProgramTransitionCatalog;
use worth_relational_contracts::internal_port::schema::data::RelationalSchemaRegistryFieldsPort as _;
use worth_relational_durability::internal_port::durability::authority::recovery::prepared_home::PreparedRelationalHomeRecoveryOwnerPort as _;

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
            if prepared.receiver_identity() != self.runtime.runtime_instance_id() {
                return Err(Denial::ForeignReceiver);
            }
            expected
                .require_same_home_receiver(self.runtime, request)
                .map_err(Denial::ReceiverInstallation)?;
            prepared
                .schema_graph_mut()
                .bind_home_executables(
                    self.runtime
                        .config()
                        .schema
                        .registry
                        .owner_content_formats(),
                    request,
                )
                .map_err(Denial::Recovery)?;
            prepared
                .bind_schema_directories(request)
                .map_err(Denial::Recovery)?;
            crate::runtime::durability_access::rebind_home_receiver_plan(
                self.runtime,
                prepared.recovery_plan_mut(),
            );
            Ok(())
        })();
        if let Err(cause) = ready {
            return Err(Refusal::Preparation { prepared, cause });
        }
        let (plan, graph, _memory) = prepared.into_prepared_recovery();
        // Ordinary recovery admission remains authoritative. This is schema
        // DATA reuse, never an "already verified" bypass.
        let admitted =
            super::super::runtime_installation::admit_recovery_with_counters(self.runtime, plan)
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
