//! Committing one output source and collecting the custody it prepared.
//!
//! Every performed lane, whichever program it presents, commits its source
//! through the program runtime and then either holds prepared output custody
//! or learns exactly why the committed source could not prepare it.

use std::cell::RefCell;
use std::sync::Arc;

use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_execution::facade::application_installation::WorthQueryProgramApplicationRuntime;
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationReadObservation, WorthQueryPreparedRequiredOutputSource,
    WorthQueryRequiredOutputSourcePreparationFailure,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryPerformedMutationExecutionDenial, WorthQueryRequiredOutputPreparationDenial,
};

type PreparedCustody = (
    WorthQueryPreparedRequiredOutputSource,
    Arc<WorthQueryApplicationReadObservation>,
);
type SourceCommit = Result<
    (WorthQueryApplicationCommitOutcome, Option<PreparedCustody>),
    WorthQueryRequiredOutputSourcePreparationFailure,
>;

/// Requires that `application` is the program runtime this request's host
/// installed, and that its output shape declares `Root`.
pub(super) fn require_program_output_root<Schema, Program, Root>(
    request_application: &WorthQueryPrimaryGraphApplicationRuntime<Schema>,
    application: &WorthQueryProgramApplicationRuntime<Schema, Program>,
) -> Result<(), WorthQueryPerformedMutationExecutionDenial>
where
    Schema: ApplicationSchema,
    Program: ApplicationProgramDefinition<Schema>,
    Root: ApplicationOutputGraphShape<Schema>,
{
    if !std::ptr::eq(application.runtime(), request_application)
        || application.installed_program().schema_binding()
            != &request_application.installed_schema().binding_identity()
    {
        return Err(WorthQueryPerformedMutationExecutionDenial::ForeignProgram);
    }
    if !application.contains_output_root::<Root>() {
        return Err(WorthQueryPerformedMutationExecutionDenial::UndeclaredOutputRoot);
    }
    Ok(())
}

/// What one source commit handed back beside its commit outcome.
#[derive(Default)]
pub(super) struct PerformedSourceCommit {
    failure: RefCell<Option<WorthQueryRequiredOutputSourcePreparationFailure>>,
    prepared: RefCell<Option<PreparedCustody>>,
}

impl PerformedSourceCommit {
    /// Keeps the custody or failure a source commit produced and returns the
    /// commit outcome the mutation lane settles. A source that committed but
    /// could not prepare custody still reports its committed receipt.
    pub(super) fn record(&self, commit: SourceCommit) -> WorthQueryApplicationCommitOutcome {
        match commit {
            Ok((outcome, prepared)) => {
                if let Some(prepared) = prepared {
                    self.prepared.replace(Some(prepared));
                }
                outcome
            }
            Err(failure) => {
                let receipt: WorthQueryApplicationCommitReceipt = failure.receipt().clone();
                self.failure.replace(Some(failure));
                WorthQueryApplicationCommitOutcome::Committed(receipt)
            }
        }
    }

    /// The prepared custody of a committed source, or why it has none.
    pub(super) fn into_custody(
        self,
    ) -> Result<PreparedCustody, WorthQueryRequiredOutputPreparationDenial> {
        if let Some(failure) = self.failure.into_inner() {
            return Err(WorthQueryRequiredOutputPreparationDenial::DemandExecution(
                failure.denial().clone(),
            ));
        }
        self.prepared
            .into_inner()
            .ok_or(WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery)
    }
}
