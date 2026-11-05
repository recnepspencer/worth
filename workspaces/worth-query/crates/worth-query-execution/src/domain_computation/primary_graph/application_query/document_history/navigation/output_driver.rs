//! One original-attempt lookup and borrowed authority dispatcher for all tree modes.
use super::*;
use crate::domain_computation::primary_graph::{
    product_operation::document_history::{HistoryOutputLookup, HistoryOutputLookupPhase},
    WorthQueryCompletedHistoryOutputs, WorthQueryHistoryOutputNavigationAdvance as Step,
    WorthQueryHistoryOutputNavigationDenial as Denial, WorthQueryHistoryOutputProfile,
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub(super) fn call_history_output_driver<'runtime, 'scope, H, N, E>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        read: &CompletedMetadataRead<'_, 'scope>,
        installed: &worth_query_installation::facade::WorthQueryInstalledDocumentHistory<S, H>,
        admission: &NavigationAdmission<S, H, N>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
        profile: &WorthQueryHistoryOutputProfile,
        callback: impl for<'authority> FnOnce(
            [u8; 32],
            Step<'authority, 'scope>,
        )
            -> Result<Option<WorthQueryCompletedHistoryOutputs>, E>,
    ) -> Result<Result<Option<WorthQueryCompletedHistoryOutputs>, E>, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
    {
        let lookup = document
            .inspect_output_navigation::<N>(
                phase, read, installed, admission, navigation, key, profile,
            )
            .map_err(Denial::Navigation)?;
        let outputs = match lookup {
            HistoryOutputLookup::Pending { key } => callback(key, Step::Pending),
            HistoryOutputLookup::Selected { key, completed } => {
                callback(key, Step::Selected(completed))
            }
            HistoryOutputLookup::Source {
                key,
                phase: state,
                branch,
                source,
                profile,
                selected_graph,
            } => {
                let original = std::sync::Arc::clone(&source);
                document
                    .with_output_authority::<N, _>(
                        phase,
                        read,
                        installed,
                        admission,
                        &profile,
                        &original,
                        selected_graph,
                        |authority| {
                            callback(
                                key,
                                match state {
                                    HistoryOutputLookupPhase::AwaitingOutputs => Step::Outputs {
                                        branch,
                                        source,
                                        authority,
                                    },
                                    HistoryOutputLookupPhase::Selecting => Step::Selecting {
                                        branch,
                                        source,
                                        authority,
                                    },
                                },
                            )
                        },
                    )
                    .map_err(Denial::Navigation)?
            }
        };
        Ok(outputs)
    }
}
