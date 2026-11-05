//! A completed current metadata READ lends the historical source to one phase.
use super::super::metadata::CompletedMetadataRead;
use super::*;
use crate::basis::WorthQueryProductBranchReadIdentity;
use crate::domain_computation::primary_graph::product_operation::document_history::HistoryModelIdentity;
use std::sync::Arc;
use worth_foundational::facade::CanonicalDigestId;
use worth_query_installation::facade::{
    WorthQueryInstalledApplicationQueryBindingObservation,
    WorthQueryInstalledApplicationQueryIdentity,
};
use worth_relational::facade::identity::EntityId;
use {
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenial,
    crate::domain_computation::primary_graph::vocabulary::application_query::denial::WorthQueryApplicationQueryAdmissionDenialKind,
};

pub struct WorthQueryHistoryPreviewReadAdmission<'phase, 'scope> {
    pub(super) completed: CompletedMetadataRead<'phase, 'scope>,
    pub(super) model: HistoryModelIdentity,
    pub(super) step: crate::domain_computation::primary_graph::WorthQueryHistoryStep,
    source: Arc<crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation>,
    query: WorthQueryInstalledApplicationQueryIdentity,
    binding: WorthQueryInstalledApplicationQueryBindingObservation,
    parameters: CanonicalDigestId,
    principal: EntityId,
}

impl<'phase, 'scope> WorthQueryHistoryPreviewReadAdmission<'phase, 'scope> {
    pub(super) fn new(
        completed: CompletedMetadataRead<'phase, 'scope>,
        preview: &WorthQueryHistoryPreview,
        query: WorthQueryInstalledApplicationQueryIdentity,
        binding: WorthQueryInstalledApplicationQueryBindingObservation,
        parameters: CanonicalDigestId,
        principal: EntityId,
    ) -> Self {
        Self {
            completed,
            model: preview.model.clone(),
            step: preview.step.clone(),
            source: Arc::clone(&preview.source),
            query,
            binding,
            parameters,
            principal,
        }
    }

    pub(in crate::domain_computation::primary_graph) fn source(
        &self,
    ) -> &crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation {
        &self.source
    }

    pub(in crate::domain_computation::primary_graph) fn validate_selection(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        query: &WorthQueryInstalledApplicationQueryIdentity,
        binding: Option<&WorthQueryInstalledApplicationQueryBindingObservation>,
        current: WorthQueryProductBranchReadIdentity,
        model: HistoryModelIdentity,
    ) -> Result<(), WorthQueryApplicationQueryAdmissionDenial> {
        if !std::ptr::eq(phase, self.completed.phase())
            || query != &self.query
            || binding != Some(&self.binding)
            || current != self.completed.source().identity()
            || model != self.model
        {
            return Err(self.foreign());
        }
        Ok(())
    }

    pub(super) fn validate_plan<S: ApplicationSchema, Q, P, R, A, I, E>(
        &self,
        phase: &WorthQueryAdvancementPhase<'_>,
        plan: &WorthQueryAdmittedApplicationQueryPlan<'_, S, Q, P, R, A, I, E>,
    ) -> Result<(), Denial> {
        self.validate_selection(
            phase,
            plan.query.identity(),
            plan.query.binding_observation(),
            WorthQueryProductBranchReadIdentity::from_observation(
                plan.security_product.observation(),
            ),
            HistoryModelIdentity::from_resolved(plan.scope),
        )
        .map_err(|_| Denial::ForeignHistoryNode)?;
        if plan.parameters.identity() != &self.parameters
            || plan.principal.principal_entity_id() != self.principal
        {
            return Err(Denial::InstalledBindingMismatch);
        }
        Ok(())
    }

    fn foreign(&self) -> WorthQueryApplicationQueryAdmissionDenial {
        WorthQueryApplicationQueryAdmissionDenial::new(
            WorthQueryApplicationQueryAdmissionDenialKind::ForeignBasis,
            "history preview",
        )
    }
}
