use worth_query_decl::facade::application_capability::{
    ApplicationCapabilityEntitySelector, ApplicationCapabilityGovernedInputIdentity,
    ApplicationCapabilityRequest, ApplicationCapabilityRequestContext,
    ApplicationCapabilityRequestProjection, ApplicationCapabilityRequestProjectionDenial,
    ApplicationCapabilityRevocationRequest, ApplicationCapabilityRevocationRequestProjection,
    ApplicationCapabilityRevocationRequestProjectionDenial,
};

use crate::{
    estate::EstateAction,
    schema::{
        BankSchema, CapabilityGrantIdentityField, DelegateEstateCapability, EstateActionContext,
        EstateCase, EstateCaseIdentityField, EstateLegalAuthoritySlot, LegalAuthorityIdentityField,
        OpenEstateCaseCapability, RecognizeEstateExecutorCapability, ReleaseEstateCapability,
        RetransmitDeathNoticeEstateCapability, RevokeEstateCapability,
        ViewEstateAdministrationCapability, ViewEstateIdentityVerificationCapability,
        ViewEstateLegalComplianceCapability, ViewEstateMandatoryReviewCapability,
    },
};

#[path = "request_projection/death_notification.rs"]
mod death_notification;
#[path = "request_projection/delegation.rs"]
mod delegation;
#[path = "request_projection/disbursement.rs"]
mod disbursement;
#[path = "request_projection/emergency_elevation.rs"]
mod emergency_elevation;
#[path = "request_projection/emergency_use.rs"]
mod emergency_use;
#[path = "request_projection/freeze.rs"]
mod freeze;

macro_rules! simple_estate_request {
    ($capability:ty, $operation:pat) => {
        impl ApplicationCapabilityRequest<BankSchema, $capability> for EstateAction {
            type Scope = EstateCase;
            type Context = EstateActionContext;

            fn capability_request(
                &self,
            ) -> Result<
                ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
                ApplicationCapabilityRequestProjectionDenial,
            > {
                let $operation = *self else {
                    return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                        "estate operation input",
                    ));
                };
                Ok(estate_request(
                    self,
                    self.estate().expect("matched estate operation"),
                ))
            }
        }
    };
}

simple_estate_request!(
    RetransmitDeathNoticeEstateCapability,
    EstateAction::RetransmitDeathNotice { .. }
);
simple_estate_request!(
    OpenEstateCaseCapability,
    EstateAction::OpenEstateCase { .. }
);
simple_estate_request!(
    DelegateEstateCapability,
    EstateAction::DelegateCapability { .. }
);
simple_estate_request!(
    RevokeEstateCapability,
    EstateAction::RevokeCapability { .. }
);
impl ApplicationCapabilityRequest<BankSchema, ReleaseEstateCapability> for EstateAction {
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn governed_input_identity(&self) -> Option<ApplicationCapabilityGovernedInputIdentity> {
        let EstateAction::ReleaseEstate {
            estate,
            executor,
            authority,
            review,
        } = *self
        else {
            return None;
        };
        Some(ApplicationCapabilityGovernedInputIdentity::four_u64([
            estate.get(),
            executor.get(),
            authority.get(),
            review.get(),
        ]))
    }

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::ReleaseEstate { estate, .. } = *self else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "estate release operation input",
            ));
        };
        Ok(estate_request(self, estate))
    }
}

impl ApplicationCapabilityRevocationRequest<BankSchema, RevokeEstateCapability> for EstateAction {
    fn capability_revocation_target(
        &self,
    ) -> Result<
        ApplicationCapabilityRevocationRequestProjection<BankSchema>,
        ApplicationCapabilityRevocationRequestProjectionDenial,
    > {
        let EstateAction::RevokeCapability { grant, .. } = *self else {
            return Err(
                ApplicationCapabilityRevocationRequestProjectionDenial::input_variant(
                    "RevokeEstateCapabilityOperation",
                ),
            );
        };
        Ok(ApplicationCapabilityRevocationRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                CapabilityGrantIdentityField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::CapabilityGrantIdBinding>(grant),
            ),
        ))
    }
}

macro_rules! view_request {
    ($capability:ty) => {
        impl ApplicationCapabilityRequest<BankSchema, $capability> for EstateAction {
            type Scope = EstateCase;
            type Context = EstateActionContext;

            fn capability_request(
                &self,
            ) -> Result<
                ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
                ApplicationCapabilityRequestProjectionDenial,
            > {
                let EstateAction::ViewRestrictedEstate {
                    estate,
                    field,
                    purpose,
                } = *self
                else {
                    return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                        "ViewRestrictedEstateOperation",
                    ));
                };
                Ok(ApplicationCapabilityRequestProjection::new(
                    ApplicationCapabilityEntitySelector::new(
                        EstateCaseIdentityField::reference(),
                        crate::schema::encoded_bank_value::<crate::schema::EstateCaseIdBinding>(
                            estate,
                        ),
                    ),
                    crate::schema::encoded_bank_value::<
                        crate::schema::EstateCapabilityOperationBinding,
                    >(self.operation()),
                    crate::schema::encoded_bank_value::<
                        crate::schema::EstateCapabilityPurposeBinding,
                    >(purpose),
                    ApplicationCapabilityRequestContext::new(EstateActionContext::reference()),
                )
                .field(crate::schema::encoded_bank_value::<
                    crate::schema::RestrictedBankFieldBinding,
                >(field)))
            }
        }
    };
}

view_request!(ViewEstateAdministrationCapability);
view_request!(ViewEstateIdentityVerificationCapability);
view_request!(ViewEstateLegalComplianceCapability);
view_request!(ViewEstateMandatoryReviewCapability);

impl ApplicationCapabilityRequest<BankSchema, RecognizeEstateExecutorCapability> for EstateAction {
    type Scope = EstateCase;
    type Context = EstateActionContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<BankSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let EstateAction::RecognizeExecutor {
            estate, authority, ..
        } = *self
        else {
            return Err(ApplicationCapabilityRequestProjectionDenial::input_variant(
                "RecognizeEstateExecutorOperation",
            ));
        };
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                EstateCaseIdentityField::reference(),
                crate::schema::encoded_bank_value::<crate::schema::EstateCaseIdBinding>(estate),
            ),
            crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityOperationBinding>(
                self.operation(),
            ),
            crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityPurposeBinding>(
                self.purpose(),
            ),
            ApplicationCapabilityRequestContext::new(EstateActionContext::reference()).entity(
                EstateLegalAuthoritySlot::reference(),
                ApplicationCapabilityEntitySelector::new(
                    LegalAuthorityIdentityField::reference(),
                    crate::schema::encoded_bank_value::<crate::schema::LegalAuthorityIdBinding>(
                        authority,
                    ),
                ),
            ),
        ))
    }
}

fn estate_request(
    action: &EstateAction,
    estate: crate::estate::EstateCaseId,
) -> ApplicationCapabilityRequestProjection<BankSchema, EstateCase, EstateActionContext> {
    ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            EstateCaseIdentityField::reference(),
            crate::schema::encoded_bank_value::<crate::schema::EstateCaseIdBinding>(estate),
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityOperationBinding>(
            action.operation(),
        ),
        crate::schema::encoded_bank_value::<crate::schema::EstateCapabilityPurposeBinding>(
            action.purpose(),
        ),
        ApplicationCapabilityRequestContext::new(EstateActionContext::reference()),
    )
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::*;

    #[test]
    fn release_input_identity_covers_every_command_dimension() {
        let identities = [
            release(1, 2, 3, 4),
            release(5, 2, 3, 4),
            release(1, 5, 3, 4),
            release(1, 2, 5, 4),
            release(1, 2, 3, 5),
        ]
        .map(|action| {
            <EstateAction as ApplicationCapabilityRequest<
                BankSchema,
                ReleaseEstateCapability,
            >>::governed_input_identity(&action)
            .unwrap()
            .identity()
        });

        assert_eq!(identities.into_iter().collect::<BTreeSet<_>>().len(), 5);
    }

    fn release(estate: u64, executor: u64, authority: u64, review: u64) -> EstateAction {
        EstateAction::ReleaseEstate {
            estate: crate::estate::EstateCaseId::new(estate).unwrap(),
            executor: crate::model::BankPrincipalId::new(executor).unwrap(),
            authority: crate::estate::LegalAuthorityId::new(authority).unwrap(),
            review: crate::estate::MandatoryReviewId::new(review).unwrap(),
        }
    }
}
