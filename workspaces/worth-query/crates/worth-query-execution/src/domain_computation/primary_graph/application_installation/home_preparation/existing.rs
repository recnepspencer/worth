//! Existing Home preparation cannot acquire fresh initialization capabilities.
use super::*;
mod native_families;
mod source;
mod world;
pub use source::{PreparedExistingHomeQueryRuntime, QueryExistingHomeSourceAttachmentRefused};
pub use world::QueryExistingHomeWorldAttachmentRefused;

pub struct ReadmittedExistingHomeNativeMeaning<Schema: ApplicationSchemaComposition, Program> {
    state: HomeApplicationState<Schema, Program>,
    schema: InstalledNativeSchemaMeaning,
    restore_work: Option<worth_relational::facade::durability::CheckpointRestoreWork>,
}
pub struct QueryExistingHomeNativeReadmissionRefused<Schema: ApplicationSchemaComposition, Program>
{
    _prepared: PreparedHomeNativeMeaning<Schema, Program>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryExistingHomeNativeReadmissionRefused<Schema, Program>
{
    pub fn reason(&self) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
    }
}
impl<Schema: ApplicationSchemaComposition, Program> PreparedHomeNativeMeaning<Schema, Program> {
    pub fn finish_existing_home_native_readmission(
        mut self,
        phase: &super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        native: &mut RelationalRuntime,
        recovered: &RecoveredRelationalRuntimeAuthority,
        outcome: &RecoveryOutcome,
    ) -> Result<
        ReadmittedExistingHomeNativeMeaning<Schema, Program>,
        QueryExistingHomeNativeReadmissionRefused<Schema, Program>,
    > {
        let result = (|| {
            let execution = phase.execution_request();
            execution.consult().map_err(|cause| {
                super::super::super::bootstrap::request_denial(
                    super::super::super::application_contribution::advancement_denial(cause),
                )
            })?;
            if recovered.codec_transition_pending() {
                return Err(existing_denial(
                    "existing Home retains an unresolved codec transition",
                ));
            }
            self.schema.receipt = recovered
                .readmit_home_initial_schema_installation(
                    &execution,
                    native,
                    self.schema.receipt.clone(),
                )
                .map_err(super::super::super::initial_schema_denial::map_initial_schema_installation_denial)?;
            use super::super::super::bootstrap::home_publication::readmission::{
                inspect_native_completion, RecoveredBootstrapPublication,
            };
            match inspect_native_completion(
                native,
                &self.schema.layout,
                &self.state.installed,
                &execution,
            )? {
                RecoveredBootstrapPublication::Incomplete => (),
                RecoveredBootstrapPublication::Completed => {
                    super::super::super::root::index_installation::register_primary_graph_indexes(
                        &mut self.schema.layout, native,
                        super::super::super::root::index_installation::IndexInstallationPosture::RequireRecovered,
                    ).map_err(existing_denial)?;
                }
            }
            native_families::readmit(native, &self.schema.layout, &execution)
        })();
        match result {
            Ok(()) => Ok(ReadmittedExistingHomeNativeMeaning {
                state: self.state,
                schema: self.schema.bind_sources(&native.main_branch_identity()),
                restore_work: outcome.checkpoint_restore_work,
            }),
            Err(cause) => Err(QueryExistingHomeNativeReadmissionRefused {
                _prepared: self,
                cause,
            }),
        }
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    ReadmittedExistingHomeNativeMeaning<Schema, Program>
{
    pub(super) fn predecessor_native_meaning(
        &self,
    ) -> (
        &super::super::super::schema_layout::WorthQueryPrimaryGraphLayout,
        &RelationalInitialSchemaInstallationReceipt,
    ) {
        (&self.schema.layout, &self.schema.receipt)
    }
    pub fn home_bridge_definition(
        &self,
    ) -> Result<
        worth_runtime_bridge::facade::UnattachedRelationalBridgeBuilder,
        WorthQueryPrimaryGraphInstallationDenial,
    > {
        super::super::super::managed_bridge::home_definition::prepare_home_bridge_definition(
            &self.state.installed,
            &self.schema.layout,
        )
    }
    pub fn home_world_resources(
        &self,
    ) -> (
        worth_runtime_world::facade::RuntimeWorldBudgets,
        worth_runtime_world::facade::RuntimeWorldClock,
        worth_foundational::ExecutionRequestPolicy,
        Option<std::sync::Arc<worth_execution::ExecutionAuthority>>,
        worth_signal::facade::runtime::SignalConditionalEvaluationBudget,
    ) {
        self.state.home_world_resources()
    }
}
fn existing_denial(subject: impl Into<String>) -> WorthQueryPrimaryGraphInstallationDenial {
    WorthQueryPrimaryGraphInstallationDenial::new(super::super::super::WorthQueryPrimaryGraphInstallationDenialKind::CheckpointRecoveryRejected, subject)
}
