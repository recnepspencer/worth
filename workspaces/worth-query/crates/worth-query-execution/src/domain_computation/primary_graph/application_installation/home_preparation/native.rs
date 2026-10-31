//! Consuming Native installation and fresh readmission retain the most advanced Query state.
use super::super::super::root::index_installation::{
    register_primary_graph_indexes, IndexInstallationPosture,
};
use super::super::super::{
    WorthQueryBootstrapAdvancementPhase, WorthQueryPrimaryGraphInstallationDenialKind,
};
use super::*;

pub struct QueryHomeNativeInstallationRefused<Schema: ApplicationSchemaComposition, Program> {
    _state: HomeApplicationState<Schema, Program>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
pub struct QueryHomeNativeReadmissionRefused<Schema: ApplicationSchemaComposition, Program> {
    _prepared: PreparedHomeNativeMeaning<Schema, Program>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryHomeNativeInstallationRefused<Schema, Program>
{
    pub fn reason(&self) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryHomeNativeReadmissionRefused<Schema, Program>
{
    pub fn reason(&self) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
    }
}

impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeApplication<Schema, Program> {
    pub fn install_home_native_schema(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        native: &mut RelationalRuntime,
    ) -> Result<
        PreparedHomeNativeMeaning<Schema, Program>,
        QueryHomeNativeInstallationRefused<Schema, Program>,
    > {
        let Self { state, invariants } = self;
        match state.authority.prepare_native_schema(
            phase,
            &state.runtime,
            &state.installed,
            native,
            invariants,
        ) {
            Ok(schema) => Ok(PreparedHomeNativeMeaning { state, schema }),
            Err(cause) => Err(QueryHomeNativeInstallationRefused {
                _state: state,
                cause,
            }),
        }
    }
}

impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeNativeMeaning<Schema, Program> {
    /// Sole Host caller selects this fresh-only integration through Store's
    /// actual ProvenAbsent custody. It grants no permission to initialize.
    pub fn finish_fresh_home_native_readmission(
        mut self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        native: &mut RelationalRuntime,
        recovered: &RecoveredRelationalRuntimeAuthority,
        outcome: &RecoveryOutcome,
    ) -> Result<
        ReadmittedHomeNativeMeaning<Schema, Program>,
        QueryHomeNativeReadmissionRefused<Schema, Program>,
    > {
        let result = (|| {
            phase.execution_request().consult().map_err(|cause| {
                super::super::super::bootstrap::request_denial(
                    super::super::super::application_contribution::advancement_denial(cause),
                )
            })?;
            if recovered.codec_transition_pending() {
                return Err(WorthQueryPrimaryGraphInstallationDenial::new(
                    WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                    "fresh Home cannot retain an unresolved codec transition",
                ));
            }
            self.schema.receipt = native.readmit_recovered_initial_schema_installation(self.schema.receipt.clone())
                .map_err(|cause| WorthQueryPrimaryGraphInstallationDenial::new(
                    WorthQueryPrimaryGraphInstallationDenialKind::UnsupportedRecoveredNativeAuthorityLayout,
                    format!("Home recovered schema authority denied: {cause}"),
                ))?;
            register_primary_graph_indexes(
                &mut self.schema.layout,
                native,
                IndexInstallationPosture::Register,
            )
            .map_err(|cause| {
                WorthQueryPrimaryGraphInstallationDenial::new(
                    WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected,
                    cause,
                )
            })?;
            Ok(())
        })();
        match result {
            Ok(()) => Ok(ReadmittedHomeNativeMeaning {
                state: self.state,
                schema: self.schema,
                restore_work: outcome.checkpoint_restore_work,
            }),
            Err(cause) => Err(QueryHomeNativeReadmissionRefused {
                _prepared: self,
                cause,
            }),
        }
    }
}
