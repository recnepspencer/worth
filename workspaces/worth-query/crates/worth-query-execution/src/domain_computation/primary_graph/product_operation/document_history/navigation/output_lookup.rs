//! Actual registered attempt context, inspected only after fresh ingress admission.
use super::*;
use worth_query_declaration::facade::application_history::{
    ApplicationHistoryBinding, ApplicationHistoryModel, ApplicationHistoryNavigationBinding,
};
use worth_query_installation::facade::{ApplicationSchema, WorthQueryInstalledDocumentHistory};
use {
    crate::domain_computation::primary_graph::application_query::CompletedMetadataRead,
    crate::domain_computation::primary_graph::vocabulary::application_attempt::idempotency::WorthQueryApplicationIdempotencyBinding,
    crate::domain_computation::primary_graph::WorthQueryAdmittedApplicationOperation,
    crate::domain_computation::request_execution::WorthQueryAdvancementPhase,
};

pub(in crate::domain_computation::primary_graph) enum HistoryOutputLookupPhase {
    AwaitingOutputs,
    Selecting,
}
pub(in crate::domain_computation::primary_graph) enum HistoryOutputLookup {
    Pending {
        key: [u8; 32],
    },
    Source {
        key: [u8; 32],
        phase: HistoryOutputLookupPhase,
        branch: crate::basis::WorthQueryProductBranch,
        source: Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
        profile: WorthQueryHistoryOutputProfile,
    },
    Selected {
        key: [u8; 32],
        completed: WorthQueryHistoryNavigationCompletion,
    },
}

impl<'runtime, S: ApplicationSchema, H: ApplicationHistoryBinding<S>>
    super::super::WorthQueryDocumentHistory<'runtime, S, H>
{
    pub(in crate::domain_computation::primary_graph) fn inspect_output_navigation<
        N: ApplicationHistoryNavigationBinding<S, H>,
    >(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        read: &CompletedMetadataRead<'_, '_>,
        installed: &WorthQueryInstalledDocumentHistory<S, H>,
        admission: &WorthQueryAdmittedApplicationOperation<
            S,
            N::Operation,
            N::Input,
            ApplicationHistoryModel<S, H>,
        >,
        navigation: &WorthQueryDocumentHistoryNavigation<'runtime, S>,
        key: u128,
        profile: &WorthQueryHistoryOutputProfile,
    ) -> Result<HistoryOutputLookup, WorthQueryHistoryNavigationDenial> {
        use WorthQueryHistoryNavigationDenial as Denial;
        self.validate_navigation::<N>(phase, read, installed, admission)?;
        let meaning =
            WorthQueryApplicationIdempotencyBinding::for_host_commit::<S, N::Operation, _, _>(
                &key,
                N::IDENTITY,
            )
            .map_err(Denial::Value)?
            .bind_admission(admission);
        let (_cell, progress) =
            navigation.registered_progress(phase, self, admission, meaning, Some(profile))?;
        let key = progress.meaning.canonical_key();
        let source_phase = match &progress.phase {
            NavigationPhase::AwaitingOutputs { .. } => HistoryOutputLookupPhase::AwaitingOutputs,
            NavigationPhase::Selecting { .. } => HistoryOutputLookupPhase::Selecting,
            NavigationPhase::Selected(completed) => {
                return Ok(HistoryOutputLookup::Selected {
                    key,
                    completed: completed.clone(),
                })
            }
            _ => return Ok(HistoryOutputLookup::Pending { key }),
        };
        let source = Arc::clone(progress.output_source.as_ref().ok_or(Denial::IntentDrift)?);
        Ok(HistoryOutputLookup::Source {
            key,
            phase: source_phase,
            branch: crate::basis::WorthQueryProductBranch::from_occurrence(
                source.product_observation().lifecycle_incarnation(),
            ),
            source,
            profile: progress
                .output_profile
                .as_ref()
                .ok_or(Denial::IntentDrift)?
                .clone(),
        })
    }
}
