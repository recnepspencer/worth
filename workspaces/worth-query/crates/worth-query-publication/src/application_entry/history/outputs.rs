//! A document's output mode owns its tree; a request lends only current admission.
use super::*;
mod binding;
mod forest;
pub(super) use forest::RecordingForest;
pub use forest::WorthQueryHistoryWithOutputTrees;
mod closing;
mod denial;
mod progression;
mod registry;
mod retained;
mod tree;
mod verified;
pub use denial::WorthQueryHistoryOutputDenial;
pub use retained::WorthQueryHistoryWithOutputs;
use worth_query_declaration::facade::application_history::ApplicationHistoryNavigationBinding;
use worth_query_execution::publication_boundary::document_history::{
    WorthQueryDocumentHistoryNavigation, WorthQueryHistoryNavigationCompletion,
    WorthQueryHistoryNavigationIntent, WorthQueryHistorySelection,
};
pub use worth_query_execution::publication_boundary::document_history::{
    WorthQueryHistoryOutputAdvanceDenial, WorthQueryHistoryOutputControlDenial,
    WorthQueryHistoryOutputReadDenial, WorthQueryHistoryRootOutputAdmissionDenial,
};
pub use worth_query_execution::publication_boundary::document_history::{
    WorthQueryHistoryOutputQualificationDenial, WorthQueryRecordingOutputQualificationDenial,
};
pub use worth_query_execution::publication_boundary::original_read_outputs::WorthQueryOriginalReadOutputDenial;

/// A document with no required derived-output tree has no output registry.
#[derive(Debug)]
pub struct WorthQueryHistoryNoOutputs;

/// Sealed output custody preserved by document reads, edits and navigation.
/// Wrappers must preserve the document's output mode.
/// ```compile_fail,E0308
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
/// fn erase<'r, S: ApplicationSchema, H: ApplicationHistoryBinding<S>, O>(
///     document: WorthQueryApplicationHistoryDocument<'r, S, H, OrdinaryHistoryPayload, O>)
///     -> WorthQueryApplicationHistoryDocument<'r, S, H>
/// { document }
/// ```
/// Carry the same output custody through the wrapper's result.
/// ```
/// use worth_query_publication::facade::application_entry::*;
/// use worth_query_declaration::facade::{application_history::ApplicationHistoryBinding, application_schema::ApplicationSchema};
/// fn keep<'r, S: ApplicationSchema, H: ApplicationHistoryBinding<S>, O>(
///     document: WorthQueryApplicationHistoryDocument<'r, S, H, OrdinaryHistoryPayload, O>)
///     -> WorthQueryApplicationHistoryDocument<'r, S, H, OrdinaryHistoryPayload, O>
/// { document }
/// ```
#[allow(private_bounds)]
pub trait WorthQueryHistoryOutputsMode<S: ApplicationSchema, H: ApplicationHistoryBinding<S>>:
    OutputDriver<S, H>
{
}

pub(super) trait OutputDriver<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> {
    /// Data posture only; final qualification still requires the original forest.
    fn records_output_forest(&self) -> bool {
        false
    }
    fn with_recording<T>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        consume: impl FnOnce(Option<&mut RecordingForest<S>>) -> (T, bool),
    ) -> Result<T, WorthQueryHistoryOutputDenial>;
    fn discard_recording(
        &self,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial>;
    fn release_for_branch(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        branch: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial>;
    fn discard(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        navigation: &WorthQueryDocumentHistoryNavigation<'_, S>,
    ) -> Result<(), WorthQueryHistoryNavigationAdvanceDenial>;
    fn close(
        &mut self,
        phase: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial>;
    fn prepare<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
    ) -> Result<
        WorthQueryDocumentHistoryNavigation<'runtime, S>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>;

    fn advance<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
    ) -> Result<
        Option<WorthQueryHistoryNavigationCompletion>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>;
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> WorthQueryHistoryOutputsMode<S, H>
    for WorthQueryHistoryNoOutputs
{
}

impl<S: ApplicationSchema, H: ApplicationHistoryBinding<S>> OutputDriver<S, H>
    for WorthQueryHistoryNoOutputs
{
    fn with_recording<T>(
        &self,
        _: &WorthQueryAdvancementPhase<'_>,
        consume: impl FnOnce(Option<&mut RecordingForest<S>>) -> (T, bool),
    ) -> Result<T, WorthQueryHistoryOutputDenial> {
        Ok(consume(None).0)
    }
    fn discard_recording(
        &self,
        _: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        _: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        Ok(())
    }
    fn release_for_branch(
        &self,
        _: &WorthQueryAdvancementPhase<'_>,
        _: worth_query_execution::facade::product::WorthQueryProductBranch,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        Ok(())
    }
    fn discard(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        navigation: &WorthQueryDocumentHistoryNavigation<'_, S>,
    ) -> Result<(), WorthQueryHistoryNavigationAdvanceDenial> {
        navigation
            .discard(phase, &program_publication_access())
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Navigation)
    }
    fn close(
        &mut self,
        _: &WorthQueryAdvancementPhase<'_>,
    ) -> Result<(), WorthQueryHistoryOutputDenial> {
        Ok(())
    }
    fn prepare<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
    ) -> Result<
        WorthQueryDocumentHistoryNavigation<'runtime, S>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        request
            .application
            .prepare_document_history_navigation::<H, N>(
                phase,
                &program_publication_access(),
                document,
                history_access,
                read,
                expected,
                intent,
                key,
            )
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Navigation)
    }

    fn advance<'runtime, N: ApplicationHistoryNavigationBinding<S, H>>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        request: &WorthQueryApplicationRequest<'_, '_, '_, S>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
    ) -> Result<
        Option<WorthQueryHistoryNavigationCompletion>,
        WorthQueryHistoryNavigationAdvanceDenial,
    >
    where
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        request
            .application
            .advance_document_history_navigation::<H, N>(
                phase,
                &program_publication_access(),
                document,
                history_access,
                read,
                navigation,
                key,
            )
            .map_err(WorthQueryHistoryNavigationAdvanceDenial::Navigation)
    }
}
