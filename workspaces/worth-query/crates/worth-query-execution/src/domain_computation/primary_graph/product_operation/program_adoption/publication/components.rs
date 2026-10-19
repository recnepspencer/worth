use crate::domain_computation::execution_runtime::product_world::publication_denial::{
    WorthQueryProductPublicationDenial as PublicationDenial,
    WorthQueryProductPublicationOutcome as PublicationOutcome,
};
use crate::domain_computation::primary_graph::application_contribution::advancement_denial;
use worth_runtime_world::facade::NoEffectCompositePublication;

use super::{WorthQueryPerformedBranchAdoption, WorthQueryUnpublishedBranchAdoption};
use crate::domain_computation::primary_graph::product_operation::program_adoption::preparation::WorthQueryPreparedBranchAdoption;

/// What publishing a prepared branch adoption did. Returned by `publish`.
pub enum WorthQueryBranchAdoptionPublicationOutcome {
    /// Fresh capability admission refused; the native reservation remains owned.
    AuthorizationDenied {
        cause: super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial,
        prepared: WorthQueryPreparedBranchAdoption,
    },
    /// Fresh execution admission refused; the reservation remains owned.
    AdmissionDenied {
        cause: crate::domain_computation::primary_graph::WorthQueryAdvancementDenial,
        prepared: WorthQueryPreparedBranchAdoption,
    },
    /// Native execution stopped after publication was attempted.
    ExecutionDenied(crate::domain_computation::primary_graph::WorthQueryAdvancementDenial),
    /// The adoption is published and the product head moved.
    Performed(WorthQueryPerformedBranchAdoption),
    /// The publication had no effect: no owner and no product reference moved.
    NoEffect(NoEffectCompositePublication),
    /// Some owners moved, but the product head did not. Recover publication.
    ProductUnpublished(WorthQueryUnpublishedBranchAdoption),
}

impl WorthQueryPreparedBranchAdoption {
    pub fn publish<Schema: worth_query_installation::facade::ApplicationSchema>(
        self,
        application: &crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationRuntime<Schema>,
        principal: &worth_query_admission::facade::authenticated_principal::WorthQueryAuthenticatedExternalPrincipal<Schema>,
        scope: &worth_query_admission::facade::authenticated_principal::WorthQueryRequestScope,
    ) -> WorthQueryBranchAdoptionPublicationOutcome {
        let mut retained = Some(self);
        let result = application.with_application_advancement(scope, |phase| {
            let prepared = retained.take().expect("one fresh publication attempt");
            let selected = match application.on_branch(prepared.branch).select(&phase) {
                Ok(selected) => selected,
                Err(cause) => return WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause: super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial::ProductSelection(cause), prepared },
            };
            use super::super::authorization::{AdoptionEffect, AdoptionEffectOutcome};
            let target = prepared.target;
            match selected.with_program_adoption(&phase, principal, scope, &target, AdoptionEffect::Publish(prepared)) {
                Ok(AdoptionEffectOutcome::Published(outcome)) => outcome,
                Err(refusal) => {
                    let AdoptionEffect::Publish(prepared) = refusal.effect else { unreachable!("publication refusal retains its requested native owner") };
                    WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause: refusal.cause, prepared }
                }
                Ok(_) => unreachable!("publication dispatch preserves the requested effect"),
            }
        });
        match result {
            Ok(outcome) => outcome,
            Err(cause) => WorthQueryBranchAdoptionPublicationOutcome::AdmissionDenied {
                cause,
                prepared: retained.expect("root refusal precedes native custody transfer"),
            },
        }
    }

    pub(in crate::domain_computation::primary_graph::product_operation::program_adoption) fn publish_in_advancement<
        Schema: worth_query_installation::facade::ApplicationSchema,
        Operation,
        Input,
        Scope,
    >(
        self,
        phase: &crate::domain_computation::primary_graph::WorthQueryAdvancementPhase<'_>,
        admission: crate::domain_computation::authorization::WorthQueryAdmittedApplicationOperation<
            Schema,
            Operation,
            Input,
            Scope,
        >,
    ) -> WorthQueryBranchAdoptionPublicationOutcome {
        if let Err(cause) = admission.validate_current_authority() {
            return WorthQueryBranchAdoptionPublicationOutcome::AuthorizationDenied { cause: super::super::authorization::WorthQueryProgramAdoptionAuthorizationDenial::Authorization(cause), prepared: self };
        }
        let execution = phase
            .request_for_owner(self.owner_identity)
            .expect("the adoption call opened on its retained owner identity");
        let Self {
            candidate_funding,
            source,
            target,
            selected_entity_count,
            migration,
            custody,
            publication,
            recovery,
            disposition,
            support_custody,
            ..
        } = self;
        match publication.execute(execution) {
            Ok(PublicationOutcome::Performed(performed)) => {
                WorthQueryBranchAdoptionPublicationOutcome::Performed(
                    WorthQueryPerformedBranchAdoption::new(
                        performed.consume(),
                        source,
                        target,
                        selected_entity_count,
                        migration,
                        custody,
                    ),
                )
            }
            Err(PublicationDenial::Stopped(stop)) => {
                WorthQueryBranchAdoptionPublicationOutcome::ExecutionDenied(advancement_denial(
                    stop,
                ))
            }
            Err(PublicationDenial::Refused(no_effect)) => {
                WorthQueryBranchAdoptionPublicationOutcome::NoEffect(no_effect)
            }
            Ok(PublicationOutcome::ProductUnpublished(unpublished)) => {
                WorthQueryBranchAdoptionPublicationOutcome::ProductUnpublished(
                    WorthQueryUnpublishedBranchAdoption::new(
                        source,
                        target,
                        selected_entity_count,
                        migration,
                        custody,
                        candidate_funding,
                        unpublished,
                        recovery,
                        disposition,
                        support_custody,
                    ),
                )
            }
        }
    }
}
