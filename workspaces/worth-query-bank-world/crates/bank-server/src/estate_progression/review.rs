use bank_domain::{
    estate::EstateAction,
    proposals::BankIdempotencyKey,
    schema::{
        BankSchema, CompleteEstateMandatoryReviewCapability, CompleteEstateMandatoryReviewOperation,
    },
};
use worth_query_host::facade::{
    admission::authenticated_principal::WorthQueryRequestScope,
    declaration::application_schema::TypedMutationPreconditions,
    primary_graph::WorthQueryApplicationIdempotencyBinding,
};

use super::{
    idempotency::{elevation_binding, EstateElevationTransition},
    lifecycle_facts::seal_review_lifecycle_facts,
    BankEstateMandatoryReview, BankEstateMandatoryReviewOutcome, BankEstateProgressionDenial,
    BankEstateProgressionFailure,
};
use crate::{BankAuthenticatedPrincipal, BankIdentityRuntime};

impl BankIdentityRuntime {
    pub fn complete_estate_mandatory_review_with_key(
        &self,
        principal: &BankAuthenticatedPrincipal,
        mandatory: BankEstateMandatoryReview,
        action: EstateAction,
        idempotency_key: &BankIdempotencyKey,
        request: &WorthQueryRequestScope,
    ) -> Result<
        BankEstateMandatoryReviewOutcome,
        BankEstateProgressionFailure<BankEstateMandatoryReview>,
    > {
        let idempotency = match elevation_binding(
            idempotency_key,
            EstateElevationTransition::CompleteReview,
            action,
        ) {
            Ok(idempotency) => idempotency,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(denial, mandatory));
            }
        };
        self.complete_estate_mandatory_review_retaining(
            principal,
            mandatory,
            action,
            idempotency,
            request,
        )
    }

    pub fn complete_estate_mandatory_review(
        &self,
        principal: &BankAuthenticatedPrincipal,
        mandatory: BankEstateMandatoryReview,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<BankEstateMandatoryReviewOutcome, BankEstateProgressionDenial> {
        self.complete_estate_mandatory_review_retaining(
            principal,
            mandatory,
            action,
            idempotency,
            request,
        )
        .map_err(BankEstateProgressionFailure::into_denial)
    }

    fn complete_estate_mandatory_review_retaining(
        &self,
        principal: &BankAuthenticatedPrincipal,
        mandatory: BankEstateMandatoryReview,
        action: EstateAction,
        idempotency: WorthQueryApplicationIdempotencyBinding,
        request: &WorthQueryRequestScope,
    ) -> Result<
        BankEstateMandatoryReviewOutcome,
        BankEstateProgressionFailure<BankEstateMandatoryReview>,
    > {
        let EstateAction::CompleteMandatoryReview { access, review, .. } = action else {
            return Err(BankEstateProgressionFailure::retained(
                BankEstateProgressionDenial::CommandInput("CompleteEstateMandatoryReviewOperation"),
                mandatory,
            ));
        };
        let capability = match self.application_runtime().installed_schema().capability(
            CompleteEstateMandatoryReviewCapability::reference(),
            CompleteEstateMandatoryReviewOperation::reference(),
        ) {
            Ok(capability) => capability,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_capability_installation(denial),
                    mandatory,
                ));
            }
        };
        let selected = match self.select_current_product() {
            Ok(selected) => selected,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_product_selection(denial),
                    mandatory,
                ));
            }
        };
        let access_authority =
            match selected.admit_capability_access(principal.query(), &capability, action, request)
            {
                Ok(access) => access,
                Err(denial) => {
                    return Err(BankEstateProgressionFailure::retained(
                        BankEstateProgressionDenial::from_authorization(denial),
                        mandatory,
                    ));
                }
            };
        let operation = match self
            .application_runtime()
            .installed_schema()
            .installed_operation(CompleteEstateMandatoryReviewOperation::reference())
        {
            Ok(operation) => operation,
            Err(denial) => {
                return Err(BankEstateProgressionFailure::retained(
                    BankEstateProgressionDenial::from_operation_installation(denial),
                    mandatory,
                ));
            }
        };
        let admission = match self.application_runtime().authorize_mandatory_review(
            mandatory.into_query(),
            access_authority,
            &operation,
            TypedMutationPreconditions::<
                BankSchema,
                CompleteEstateMandatoryReviewOperation,
                bank_domain::schema::EstateCase,
            >::default(),
        ) {
            Ok(admission) => admission,
            Err(denial) => {
                let mapped = BankEstateProgressionDenial::from_review_authorization_ref(&denial);
                return Err(BankEstateProgressionFailure::retained(
                    mapped,
                    BankEstateMandatoryReview::from_query(denial.into_mandatory_review()),
                ));
            }
        };
        let projected = self
            .invariant_projection()
            .project_admitted_operation(&admission, |reader, estate| {
                seal_review_lifecycle_facts(reader, access, review, estate)
            })
            .map_err(BankEstateProgressionDenial::from_projection)
            .map_err(BankEstateProgressionFailure::consumed)?;
        let (lifecycle_result, projection, _) = projected.into_parts();
        lifecycle_result
            .map_err(BankEstateProgressionDenial::LifecycleProjection)
            .map_err(BankEstateProgressionFailure::consumed)?;
        let program = self
            .application_runtime()
            .begin_projected_application_read_attempt(admission, projection)
            .map_err(BankEstateProgressionDenial::from_attempt)
            .map_err(BankEstateProgressionFailure::consumed)?
            .complete_projected_dependencies()
            .map_err(BankEstateProgressionDenial::from_attempt)
            .map_err(BankEstateProgressionFailure::consumed)?
            .materialize_mandatory_review_program()
            .map_err(BankEstateProgressionDenial::from_attempt)
            .map_err(BankEstateProgressionFailure::consumed)?;
        Ok(BankEstateMandatoryReviewOutcome::from_query(
            self.application_runtime()
                .compare_and_commit_mandatory_review(program, idempotency),
        ))
    }
}
