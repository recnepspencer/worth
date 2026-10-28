//! Committing one output source and collecting the custody it prepared.
//!
//! Every performed lane, whichever program it presents, commits its source
//! through the program runtime and then either holds prepared output custody
//! or learns exactly why the committed source could not prepare it.

use std::cell::RefCell;
use std::sync::Arc;

use worth_query_declaration::facade::application_operation::ApplicationMutationIntent;
use worth_query_declaration::facade::application_program::{
    ApplicationOutputGraphShape, ApplicationProgramDefinition,
};
use worth_query_execution::facade::application_contribution::WorthQueryAdvancementPhase;
use worth_query_execution::facade::application_installation::{
    WorthQueryProgramApplicationRuntime, WorthQuerySelectedProgramOwner,
};
use worth_query_execution::facade::primary_graph::WorthQueryPrimaryGraphApplicationRuntime;
use worth_query_execution::facade::primary_graph::WorthQuerySelectedProductOperation;
use worth_query_execution::facade::primary_graph::{
    WorthQueryApplicationCommitOutcome, WorthQueryApplicationCommitReceipt,
    WorthQueryApplicationReadObservation, WorthQueryPreparedRequiredOutputSource,
    WorthQueryRequiredOutputSourcePreparationFailure,
};
use worth_query_installation::facade::ApplicationSchema;

use super::{
    WorthQueryApplicationMutationRequestWithIdempotency,
    WorthQueryPerformedMutationExecutionDenial, WorthQueryRequiredOutputPreparationDenial,
};
use crate::application_entry::WorthQueryApplicationRequestMutationDenial;

mod blocked;
pub(super) mod recovery;
pub use blocked::WorthQueryBlockedProgramSource;
pub use recovery::WorthQueryProgramSourceRecoveryProgress;

type PreparedCustody = (
    WorthQueryPreparedRequiredOutputSource,
    Arc<WorthQueryApplicationReadObservation>,
);
type SourceCommit = Result<
    (WorthQueryApplicationCommitOutcome, Option<PreparedCustody>, Option<worth_query_execution::facade::application_installation::WorthQueryUnpublishedProgramOutputSource>),
    WorthQueryRequiredOutputSourcePreparationFailure,
>;

/// Requires that `application` is the program runtime this request's host
/// installed, and that its output shape declares `Root`.
pub(in crate::application_entry) fn require_program_output_root<Schema, Program, Root>(
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

impl<'application, Schema, Intent, SourcePreparation>
    WorthQueryApplicationMutationRequestWithIdempotency<
        'application,
        '_,
        '_,
        '_,
        Schema,
        Intent,
        SourcePreparation,
    >
where
    Schema: ApplicationSchema,
    Intent: ApplicationMutationIntent<Schema>,
{
    /// The one selection check of every performed lane: the program and its
    /// `Root`, the branch's selection, the selected program's installed owner,
    /// and the compiled action that owner must still declare.
    pub(in crate::application_entry) fn select_performed_owner<Program, Root>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        application: &'application WorthQueryProgramApplicationRuntime<Schema, Program>,
    ) -> Result<
        (
            WorthQuerySelectedProductOperation<'application, Schema>,
            WorthQuerySelectedProgramOwner<'application, Schema>,
        ),
        WorthQueryPerformedMutationExecutionDenial,
    >
    where
        Program: ApplicationProgramDefinition<Schema>,
        Root: ApplicationOutputGraphShape<Schema>,
    {
        require_program_output_root::<Schema, Program, Root>(
            self.request.application,
            application,
        )?;
        let selected = self
            .request
            .application
            .on_branch(self.request.branch)
            .select(phase)
            .map_err(|selection| {
                WorthQueryPerformedMutationExecutionDenial::Mutation(
                    WorthQueryApplicationRequestMutationDenial::ProductSelection(selection),
                )
            })?;
        let owner = application
            .selected_program_owner(&selected)
            .map_err(|denial| {
                WorthQueryPerformedMutationExecutionDenial::Mutation(
                    super::selected_program::map_selected_program_owner_denial(denial),
                )
            })?;
        if let Some(declared) = self.compiled_action {
            owner
                .resolve_declared_action::<Intent::Binding>(declared)
                .map_err(WorthQueryPerformedMutationExecutionDenial::Action)?;
        }
        Ok((selected, owner))
    }
}

/// What one source commit handed back beside its commit outcome.
#[derive(Default)]
pub(in crate::application_entry) struct PerformedSourceCommit {
    failure: RefCell<Option<WorthQueryRequiredOutputSourcePreparationFailure>>,
    prepared: RefCell<Option<PreparedCustody>>,
    unpublished: RefCell<Option<worth_query_execution::facade::application_installation::WorthQueryUnpublishedProgramOutputSource>>,
}

impl PerformedSourceCommit {
    /// Keeps the custody or failure a source commit produced and returns the
    /// commit outcome the mutation lane settles. A source that committed but
    /// could not prepare custody still reports its committed receipt.
    pub(in crate::application_entry) fn record(
        &self,
        commit: SourceCommit,
    ) -> WorthQueryApplicationCommitOutcome {
        match commit {
            Ok((outcome, prepared, unpublished)) => {
                self.unpublished.replace(unpublished);
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
    ) -> Result<
        PreparedCustody,
        (
            WorthQueryRequiredOutputPreparationDenial,
            Option<WorthQueryRequiredOutputSourcePreparationFailure>,
        ),
    > {
        if let Some(failure) = self.failure.into_inner() {
            return Err((
                WorthQueryRequiredOutputPreparationDenial::DemandExecution(
                    failure.denial().clone(),
                ),
                Some(failure),
            ));
        }
        self.prepared.into_inner().ok_or((
            WorthQueryRequiredOutputPreparationDenial::MissingPerformedDelivery,
            None,
        ))
    }

    pub(super) fn take_unpublished(&self) -> Option<worth_query_execution::facade::application_installation::WorthQueryUnpublishedProgramOutputSource>{
        self.unpublished.borrow_mut().take()
    }
}
