//! The elevated fixture's capability requests: touch, request elevation and approve elevation.
use super::*;

impl ApplicationCapabilityRequest<IdentityExecutionSchema, ElevatedTouchAccountCapability>
    for ElevatedCapabilityTouchInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        let projection = ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            ),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(self.action)
                .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(self.purpose)
                .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()),
        );
        let projection = match (&self.elevation, self.substitute_resource_selector) {
            (Some(_), true) => projection.elevation(ApplicationCapabilityEntitySelector::new(
                AccountIdentity::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            )),
            (Some(elevation), false) => {
                projection.elevation(ApplicationCapabilityEntitySelector::new(
                    CapabilityElevationIdentity::reference(),
                    ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                        elevation.clone(),
                    )
                    .expect("fixture elevation identity must encode"),
                ))
            }
            (None, _) => projection,
        };
        Ok(projection
            .field(
                ApplicationEncodedScalarValue::<CapabilityDisclosureBinding>::try_new(
                    self.disclosure,
                )
                .expect("fixture capability disclosure must encode"),
            )
            .magnitude(
                ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(self.amount)
                    .expect("fixture capability magnitude must encode"),
            ))
    }
}

impl ApplicationCapabilityRequest<IdentityExecutionSchema, RequestElevationCapability>
    for RequestElevationInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(self.ordinary_projection())
    }
}

impl ApplicationCapabilityRequest<IdentityExecutionSchema, ApproveElevationCapability>
    for ApproveElevationInput
{
    type Scope = Account;
    type Context = CapabilityRequestContext;

    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<
            IdentityExecutionSchema,
            Account,
            CapabilityRequestContext,
        >,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                AccountIdentity::reference(),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.account.clone(),
                )
                .expect("fixture account identity must encode"),
            ),
            ApplicationEncodedScalarValue::<CapabilityActionBinding>::try_new(
                CapabilityAction::ApproveElevation,
            )
            .expect("fixture capability action must encode"),
            ApplicationEncodedScalarValue::<CapabilityPurposeBinding>::try_new(
                CapabilityPurpose::AccountMaintenance,
            )
            .expect("fixture capability purpose must encode"),
            ApplicationCapabilityRequestContext::new(CapabilityRequestContext::reference()).entity(
                CapabilityElevationSlot::reference(),
                ApplicationCapabilityEntitySelector::new(
                    CapabilityElevationIdentity::reference(),
                    ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                        self.elevation.clone(),
                    )
                    .expect("fixture elevation identity must encode"),
                ),
            ),
        ))
    }
}
