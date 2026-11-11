//! A single current READ/metadata/NAV owns the whole configured graph forest.
use super::*;
use crate::domain_computation::primary_graph::{
    WorthQueryCompletedHistoryOutputs, WorthQueryHistoryOutputNavigationAdvance as Step,
    WorthQueryHistoryOutputNavigationDenial as Denial, WorthQueryHistoryOutputProfile,
};

impl<S: ApplicationSchema> WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn prepare_document_history_navigation_with_output_forest<'runtime, H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
        profile: WorthQueryHistoryOutputProfile,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let installed = read.history;
        let (read, admission) = read
            .admit_navigation::<N>(self, phase, access, document, N::input(history_access))
            .map_err(Denial::Navigation)?;
        require_forest(&profile)?;
        document
            .prepare_navigation::<N>(
                phase,
                read,
                installed,
                admission,
                expected,
                intent,
                key,
                Some(profile),
            )
            .map_err(Denial::Navigation)
    }

    pub fn advance_document_history_navigation_with_output_forest<'runtime, 'scope, H, N, E>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
        profile: &WorthQueryHistoryOutputProfile,
        callback: impl for<'authority> FnOnce(
            [u8; 32],
            Step<'authority, 'scope>,
        )
            -> Result<Option<WorthQueryCompletedHistoryOutputs>, E>,
    ) -> Result<Result<Option<WorthQueryHistoryNavigationCompletion>, E>, Denial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        Rows<S, H>: WorthQueryApplicationProjection<S, Query<S, H>>,
    {
        let installed = read.history;
        let (read, admission) = read
            .admit_navigation::<N>(self, phase, access, document, N::input(history_access))
            .map_err(Denial::Navigation)?;
        require_forest(profile)?;
        let outputs = match self.call_history_output_driver::<H, N, E>(
            phase, document, &read, installed, &admission, navigation, key, profile, callback,
        )? {
            Ok(outputs) => outputs,
            Err(cause) => return Ok(Err(cause)),
        };
        read.confirm(document, ())
            .map_err(|cause| Denial::Navigation(NavigationDenial::Document(cause)))?;
        document
            .advance_navigation::<N>(
                phase,
                &read,
                installed,
                &admission,
                navigation,
                key,
                Some(profile),
                outputs,
            )
            .map(Ok)
            .map_err(Denial::Navigation)
    }
}

fn require_forest(profile: &WorthQueryHistoryOutputProfile) -> Result<(), Denial> {
    if profile.is_output_forest() && profile.uses_owned_model_graph() {
        Ok(())
    } else {
        Err(Denial::Navigation(NavigationDenial::IntentDrift))
    }
}
