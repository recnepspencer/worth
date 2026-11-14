//! Fresh control and disclosure admission over real seeded grant records.
use super::*;
use worth_query_decl::facade::{
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
};
worth_query_capability_context!(pub ControlContext in Schema);
worth_query_capability_provenance!(pub ControlProvenance in Schema);
worth_query_capability!(pub ControlCapability in Schema);
worth_query_capability!(pub StartCapability in Schema);
worth_query_capability!(pub AdvanceCapability in Schema);
impl<C: ApplicationCapabilityMarkerIdentity<Schema = Schema>>
    ApplicationCapabilityRequest<Schema, C> for SetRetentionInput
{
    type Scope = Document;
    type Context = ControlContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<Schema, Document, ControlContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::at("DocumentIdentityField"),
                encoded(&self.identity),
            )
            .unwrap(),
            encoded("author-workflow-definition"),
            encoded("reviewed-document"),
            ApplicationCapabilityRequestContext::new(ControlContext::reference()),
        )
        .related_entity(ApplicationCapabilityRelatedEntitySelector::new(
            related(),
            ApplicationCapabilityEntitySelector::new(
                DocumentIdentityField::at("DocumentIdentityField"),
                encoded(document::retention_entry::RELATED_DOCUMENT_IDENTITY),
            )
            .unwrap(),
        )))
    }
}
pub fn contract<C: ApplicationCapabilityMarkerIdentity<Schema = Schema>>(
) -> ApplicationCapabilityContract<Schema, C, ControlOperation, SetRetentionInput> {
    ApplicationCapabilityContractBuilder::new(
        ApplicationCapabilityRef::<Schema, C>::from_declaration(),
        ControlOperation::reference(),
        grant(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantActionField::at("WorkflowGrantActionField"),
            encoded("author-workflow-definition"),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(resource()),
        ApplicationCapabilityRelationDimension::bound(related()),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            WorkflowGrantPurposeField::at("WorkflowGrantPurposeField"),
            encoded("reviewed-document"),
        )
        .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                WorkflowGrantStatusField::at("WorkflowGrantStatusField"),
                encoded("active"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantWorkflowField::at(
                    "WorkflowGrantWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::at(
                    "DocumentIdentityField",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotBeforeField::at(
                    "WorkflowGrantNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotAfterField::at(
                    "WorkflowGrantNotAfterField",
                ))
                .unwrap(),
            ),
        ),
        ControlContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(parent()),
        ApplicationCapabilityRelationBinding::from_reference(grantor()),
        ApplicationCapabilityRelationBinding::from_reference(grantee()),
        ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantDelegationLimitField::at(
            "WorkflowGrantDelegationLimitField",
        ))
        .unwrap(),
        ControlProvenance::reference(),
    ))
    .composition(composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}

pub fn composition() -> ApplicationCapabilityComposition {
    let allow = ApplicationAuthorizationPathBuilder::from_principal(principal())
        .forward(owner())
        .allow(record());
    ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(allow),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ApplicationCapabilityActorComposition::new(
            ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_decl::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    )
}

pub fn encoded(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned())
        .expect("workflow capability fixture text must encode")
}
