//! Existing source attachment has neither an initializer nor a seed publisher.
use super::super::source::{
    admit_registered_source,
    assembly::{self, HomeQueryPreparation},
};
use super::*;
use std::sync::{Arc, RwLock};
use worth_runtime_bridge::facade::{
    BridgeSealedRuntimeAssembly, RuntimeBridge, RuntimeBridgeRelationalSource,
};
/// Existing preparation carries recovered meaning and has no fresh initializer.
/// Its positive handoff is a move of the genuine carrier:
///
/// ```
/// use worth_query_execution::facade::application_home_integration::PreparedExistingHomeQueryRuntime;
/// use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
/// fn handoff<S: ApplicationSchemaComposition, P>(
///     existing: PreparedExistingHomeQueryRuntime<S, P>,
/// ) -> PreparedExistingHomeQueryRuntime<S, P> { existing }
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::{application_home_integration::PreparedExistingHomeQueryRuntime,
///     application_contribution::WorthQueryBootstrapAdvancementPhase};
/// use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
/// fn cannot_initialize<S: ApplicationSchemaComposition, P>(
///     existing: &mut PreparedExistingHomeQueryRuntime<S, P>,
///     phase: &WorthQueryBootstrapAdvancementPhase<'_>,
/// ) { existing.initialize_home_query_state(phase, |_, _, _| Ok(())); }
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::{application_home_integration::PreparedExistingHomeQueryRuntime,
///     application_contribution::WorthQueryBootstrapAdvancementPhase};
/// use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
/// fn cannot_seed<S: ApplicationSchemaComposition, P>(
///     existing: &mut PreparedExistingHomeQueryRuntime<S, P>,
///     phase: &WorthQueryBootstrapAdvancementPhase<'_>,
/// ) { existing.publish_home_query_seed(phase); }
/// ```
///
/// ```compile_fail,E0599
/// use worth_query_execution::facade::application_home_integration::PreparedExistingHomeQueryRuntime;
/// use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
/// fn cannot_duplicate<S: ApplicationSchemaComposition, P>(existing: PreparedExistingHomeQueryRuntime<S, P>) {
///     let _copy = existing.clone();
/// }
/// ```
///
/// ```compile_fail,E0382
/// use worth_query_execution::facade::application_home_integration::PreparedExistingHomeQueryRuntime;
/// use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
/// fn cannot_handoff_twice<S: ApplicationSchemaComposition, P>(
///     existing: PreparedExistingHomeQueryRuntime<S, P>,
///     first: impl FnOnce(PreparedExistingHomeQueryRuntime<S, P>),
///     second: impl FnOnce(PreparedExistingHomeQueryRuntime<S, P>),
/// ) { first(existing); second(existing); }
/// ```
///
/// ```compile_fail,E0451
/// use worth_query_execution::facade::application_home_integration::PreparedExistingHomeQueryRuntime;
/// use worth_query_declaration::facade::application_schema::ApplicationSchemaComposition;
/// fn cannot_fabricate<S: ApplicationSchemaComposition, P>() -> PreparedExistingHomeQueryRuntime<S, P> {
///     PreparedExistingHomeQueryRuntime { prepared: panic!("not authority") }
/// }
/// ```
pub struct PreparedExistingHomeQueryRuntime<Schema: ApplicationSchemaComposition, Program> {
    pub(super) prepared: HomeQueryPreparation<Schema, Program>,
}
pub struct QueryExistingHomeSourceAttachmentRefused<Schema: ApplicationSchemaComposition, Program> {
    _prepared: ReadmittedExistingHomeNativeMeaning<Schema, Program>,
    _source: RuntimeBridgeRelationalSource,
    _bridge: RuntimeBridge,
    _conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    cause: WorthQueryPrimaryGraphInstallationDenial,
}
impl<Schema: ApplicationSchemaComposition, Program>
    QueryExistingHomeSourceAttachmentRefused<Schema, Program>
{
    pub fn reason(&self) -> &WorthQueryPrimaryGraphInstallationDenial {
        &self.cause
    }
}
impl<Schema: ApplicationSchemaComposition, Program>
    ReadmittedExistingHomeNativeMeaning<Schema, Program>
{
    pub fn attach_existing_home_registered_source(
        self,
        _phase: &super::super::super::super::WorthQueryBootstrapAdvancementPhase<'_>,
        source: RuntimeBridgeRelationalSource,
        bridge: RuntimeBridge,
        conditional: Arc<RwLock<BridgeSealedRuntimeAssembly>>,
    ) -> Result<
        PreparedExistingHomeQueryRuntime<Schema, Program>,
        QueryExistingHomeSourceAttachmentRefused<Schema, Program>,
    > {
        let owner = match admit_registered_source(
            &self.state,
            &self.schema,
            &source,
            &bridge,
            &conditional,
        ) {
            Ok(owner) => owner,
            Err(cause) => {
                return Err(QueryExistingHomeSourceAttachmentRefused {
                    _prepared: self,
                    _source: source,
                    _bridge: bridge,
                    _conditional: conditional,
                    cause,
                })
            }
        };
        Ok(PreparedExistingHomeQueryRuntime {
            prepared: assembly::assemble(
                self.state,
                self.schema,
                self.restore_work,
                owner,
                bridge,
                conditional,
            ),
        })
    }
}
