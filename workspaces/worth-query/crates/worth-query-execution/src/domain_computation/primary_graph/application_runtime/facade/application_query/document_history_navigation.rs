//! The runtime's consumer methods over `document_history_navigation` in `domain_computation::primary_graph::application_query`; each only calls its subsystem.
use crate::domain_computation::primary_graph::application_query::HistoryOutputControlPlan;
use crate::domain_computation::primary_graph::application_query::WorthQueryApplicationProjection;
use crate::domain_computation::primary_graph::application_query::WorthQueryDocumentHistoryRead;
use crate::domain_computation::primary_graph::WorthQueryCompletedHistoryOutputs;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistory;
use crate::domain_computation::primary_graph::WorthQueryDocumentHistoryNavigation;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationCompletion;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryNavigationIntent;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputNavigationAdvance;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputNavigationDenial;
use crate::domain_computation::primary_graph::WorthQueryHistoryOutputProfile;
use crate::domain_computation::primary_graph::WorthQueryHistorySelection;
use crate::domain_computation::request_execution::WorthQueryAdvancementPhase;
use crate::publication_boundary::WorthQueryProgramPublicationAccess;
use worth_query_declaration::facade::application_history::ApplicationHistoryAccessBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryBinding;
use worth_query_declaration::facade::application_history::ApplicationHistoryNavigationBinding;
use worth_query_declaration::facade::application_query::ApplicationQueryBinding;
use worth_query_declaration::facade::application_schema::ApplicationStructuredValueBinding;
use worth_query_installation::facade::ApplicationSchema;
use worth_query_installation::facade::WorthQueryInstalledApplicationQueryBinding;

impl<S: ApplicationSchema> super::super::super::WorthQueryPrimaryGraphApplicationRuntime<S> {
    pub fn prepare_document_history_navigation<'runtime, H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        expected: WorthQueryHistorySelection,
        intent: WorthQueryHistoryNavigationIntent,
        key: u128,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::NavigationOps<S>>::prepare_document_history_navigation::<H, N>(self, phase, access, document, history_access, read, expected, intent, key)
    }

    pub fn advance_document_history_navigation<'runtime, H, N>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
    ) -> Result<Option<WorthQueryHistoryNavigationCompletion>, WorthQueryHistoryNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::NavigationOps<S>>::advance_document_history_navigation::<H, N>(self, phase, access, document, history_access, read, navigation, key)
    }

    pub fn prepare_document_history_navigation_with_outputs<'runtime, H, N, B>(
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
        source_binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        source: HistoryOutputControlPlan<'_, S, B>,
    ) -> Result<WorthQueryDocumentHistoryNavigation<'runtime, S>, WorthQueryHistoryOutputNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        B: ApplicationQueryBinding<S>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::OutputsOps<S>>::prepare_document_history_navigation_with_outputs::<H, N, B>(self, phase, access, document, history_access, read, expected, intent, key, profile, source_binding, source)
    }

    pub fn advance_document_history_navigation_with_outputs<'runtime, 'scope, H, N, B, E>(
        &self,
        phase: &WorthQueryAdvancementPhase<'scope>,
        access: &WorthQueryProgramPublicationAccess,
        document: &WorthQueryDocumentHistory<'runtime, S, H>,
        history_access: &H::Access,
        read: WorthQueryDocumentHistoryRead<'_, S, H>,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
        profile: &WorthQueryHistoryOutputProfile,
        source_binding: &WorthQueryInstalledApplicationQueryBinding<S, B>,
        source: HistoryOutputControlPlan<'_, S, B>,
        callback: impl for<'authority> FnOnce(
            [u8; 32],
            WorthQueryHistoryOutputNavigationAdvance<'authority, 'scope>,
        )
            -> Result<Option<WorthQueryCompletedHistoryOutputs>, E>,
    ) -> Result<Result<Option<WorthQueryHistoryNavigationCompletion>, E>, WorthQueryHistoryOutputNavigationDenial>
    where
        H: ApplicationHistoryBinding<S>,
        N: ApplicationHistoryNavigationBinding<S, H>,
        B: ApplicationQueryBinding<S>,
    <<ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::ResultBinding as ApplicationStructuredValueBinding>::Value: WorthQueryApplicationProjection<S, <ApplicationHistoryAccessBinding<S, H> as ApplicationQueryBinding<S>>::Query>,{
        <Self as crate::domain_computation::primary_graph::application_query::OutputsOps<S>>::advance_document_history_navigation_with_outputs::<H, N, B, E>(self, phase, access, document, history_access, read, navigation, key, profile, source_binding, source, callback)
    }
}
