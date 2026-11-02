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
    /// Consume the same preparation on an actual source-input refusal.
    pub fn refuse_home_native_schema(
        self,
        cause: worth_relational::facade::runtime::RelationalInitialSchemaInstallationDenial,
    ) -> QueryHomeNativeInstallationRefused<Schema, Program> {
        QueryHomeNativeInstallationRefused {
            _state: self.state,
            cause:
                super::super::super::initial_schema_denial::map_initial_schema_installation_denial(
                    cause,
                ),
        }
    }
    pub fn initialize_home_native_schema(
        self,
        phase: &WorthQueryBootstrapAdvancementPhase<'_>,
        native: &mut RelationalRuntime,
        initialization: worth_proof::FreshNativeInitialization<
            worth_relational::facade::durability::RelationalFreshInitialization,
        >,
    ) -> Result<
        PreparedHomeNativeMeaning<Schema, Program>,
        QueryHomeNativeInstallationRefused<Schema, Program>,
    > {
        let Self { state, invariants } = self;
        match state.authority.prepare_native_schema_for_fresh_home(
            phase,
            &state.runtime,
            &state.installed,
            native,
            invariants,
            initialization,
        ) {
            Ok(schema) => Ok(PreparedHomeNativeMeaning { state, schema }),
            Err(cause) => Err(QueryHomeNativeInstallationRefused {
                _state: state,
                cause,
            }),
        }
    }

    pub fn prepare_home_native_receiver_schema(
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
        admission: &worth_relational::facade::durability::FreshRelationalSourceAdmission,
    ) -> Result<
        ReadmittedHomeNativeMeaning<Schema, Program>,
        QueryHomeNativeReadmissionRefused<Schema, Program>,
    > {
        let result = (|| {
            let execution = phase.execution_request();
            execution.consult().map_err(|cause| {
                super::super::super::bootstrap::request_denial(
                    super::super::super::application_contribution::advancement_denial(cause),
                )
            })?;
            native.validate_fresh_initial_schema_installation(&execution, admission, &self.schema.receipt)
                .map_err(super::super::super::initial_schema_denial::map_initial_schema_installation_denial)?;
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
                activation_seed: super::super::super::bootstrap::WorthQueryProgramActivationSeed::for_initial_program(
                    &self.state.support.initial_revision, self.state.activation.clone()),
                state: self.state,
                schema: self.schema,
                restore_work: None,
            }),
            Err(cause) => Err(QueryHomeNativeReadmissionRefused {
                _prepared: self,
                cause,
            }),
        }
    }
}
