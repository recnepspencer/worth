//! Actual grant declarations and request projections; native Query evaluates them.
pub use super::schema::*;
pub use worth_query_host::facade::{
    declaration::application_capability::*, declaration::application_schema::*, *,
};
#[path = "grants/recorded_operation.rs"]
pub mod recorded_operation;

pub struct RetentionCapability;
impl declaration::portable_identity::WorthQueryPortableType for RetentionCapability {
    const PORTABLE_TYPE_NAME: &'static str = "capability-retention.RetentionCapability";
}
impl ApplicationCapabilityMarkerIdentity for RetentionCapability {
    type Schema = CapabilitySchema;
    const IDENTIFIER: &'static str = "RetentionCapability";
}
impl RetentionCapability {
    pub fn reference() -> ApplicationCapabilityRef<CapabilitySchema, Self> {
        ApplicationCapabilityRef::from_declaration()
    }
}
pub struct WorkflowInstanceStartCapability;
impl declaration::portable_identity::WorthQueryPortableType for WorkflowInstanceStartCapability {
    const PORTABLE_TYPE_NAME: &'static str = "capability-retention.WorkflowInstanceStartCapability";
}
impl ApplicationCapabilityMarkerIdentity for WorkflowInstanceStartCapability {
    type Schema = CapabilitySchema;
    const IDENTIFIER: &'static str = "WorkflowInstanceStartCapability";
}
impl WorkflowInstanceStartCapability {
    pub fn reference() -> ApplicationCapabilityRef<CapabilitySchema, Self> {
        ApplicationCapabilityRef::from_declaration()
    }
}
pub struct WorkflowAdvanceCapability;
impl declaration::portable_identity::WorthQueryPortableType for WorkflowAdvanceCapability {
    const PORTABLE_TYPE_NAME: &'static str = "capability-retention.WorkflowAdvanceCapability";
}
impl ApplicationCapabilityMarkerIdentity for WorkflowAdvanceCapability {
    type Schema = CapabilitySchema;
    const IDENTIFIER: &'static str = "WorkflowAdvanceCapability";
}
impl WorkflowAdvanceCapability {
    pub fn reference() -> ApplicationCapabilityRef<CapabilitySchema, Self> {
        ApplicationCapabilityRef::from_declaration()
    }
}
pub struct RevocationCapability;
impl declaration::portable_identity::WorthQueryPortableType for RevocationCapability {
    const PORTABLE_TYPE_NAME: &'static str = "capability-retention.RevocationCapability";
}
impl ApplicationCapabilityMarkerIdentity for RevocationCapability {
    type Schema = CapabilitySchema;
    const IDENTIFIER: &'static str = "RevocationCapability";
}
impl RevocationCapability {
    pub fn reference() -> ApplicationCapabilityRef<CapabilitySchema, Self> {
        ApplicationCapabilityRef::from_declaration()
    }
}
pub struct GrantContext;
impl declaration::portable_identity::WorthQueryPortableType for GrantContext {
    const PORTABLE_TYPE_NAME: &'static str = "capability-retention.GrantContext";
}
impl ApplicationCapabilityContextMarkerIdentity for GrantContext {
    type Schema = CapabilitySchema;
    const IDENTIFIER: &'static str = "GrantContext";
}
impl GrantContext {
    pub fn reference() -> ApplicationCapabilityContextRef<CapabilitySchema, Self> {
        ApplicationCapabilityContextRef::from_declaration()
    }
}
pub struct GrantProvenance;
impl declaration::portable_identity::WorthQueryPortableType for GrantProvenance {
    const PORTABLE_TYPE_NAME: &'static str = "capability-retention.GrantProvenance";
}
impl ApplicationCapabilityProvenanceMarkerIdentity for GrantProvenance {
    type Schema = CapabilitySchema;
    const IDENTIFIER: &'static str = "GrantProvenance";
}
impl GrantProvenance {
    pub fn reference() -> ApplicationCapabilityProvenanceRef<CapabilitySchema, Self> {
        ApplicationCapabilityProvenanceRef::from_declaration()
    }
}
#[derive(Clone, serde::Serialize)]
pub struct RevokeInput {
    pub identity: String,
    pub grant: String,
}
worth_query_structured_value_binding!(pub RevokeInputBinding for RevokeInput { identity:"capability-retention.revoke-input.v1" });
worth_query_operation!(pub RevokeGrant for CapabilitySchema, input RevokeInputBinding);
worth_query_operation_reads!(RevokeGrant => [GrantResource]);
pub fn projection(
    action: u64,
    identity: &str,
) -> ApplicationCapabilityRequestProjection<CapabilitySchema, Document, GrantContext> {
    ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            DocumentIdentityField::at("DocumentIdentityField"),
            ApplicationEncodedScalarValue::try_new(identity.to_owned()).unwrap(),
        )
        .unwrap(),
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(action).unwrap(),
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(action).unwrap(),
        ApplicationCapabilityRequestContext::new(GrantContext::reference()),
    )
}
impl ApplicationCapabilityRequest<CapabilitySchema, RetentionCapability> for SetRetentionInput {
    type Scope = Document;
    type Context = GrantContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<CapabilitySchema, Document, GrantContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(projection(1, &self.identity))
    }
}
impl ApplicationCapabilityRequest<CapabilitySchema, WorkflowInstanceStartCapability>
    for SetRetentionInput
{
    type Scope = Document;
    type Context = GrantContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<CapabilitySchema, Document, GrantContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(projection(1, &self.identity))
    }
}
impl ApplicationCapabilityRequest<CapabilitySchema, WorkflowAdvanceCapability>
    for SetRetentionInput
{
    type Scope = Document;
    type Context = GrantContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<CapabilitySchema, Document, GrantContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(projection(2, &self.identity))
    }
}
impl ApplicationCapabilityRequest<CapabilitySchema, RevocationCapability> for RevokeInput {
    type Scope = Document;
    type Context = GrantContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<CapabilitySchema, Document, GrantContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(projection(2, &self.identity))
    }
}
impl ApplicationCapabilityRevocationRequest<CapabilitySchema, RevocationCapability>
    for RevokeInput
{
    fn capability_revocation_target(
        &self,
    ) -> Result<
        ApplicationCapabilityRevocationRequestProjection<CapabilitySchema>,
        ApplicationCapabilityRevocationRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRevocationRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                GrantIdentity::at("GrantIdentity"),
                ApplicationEncodedScalarValue::try_new(self.grant.clone()).unwrap(),
            )
            .unwrap(),
        ))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
) -> ApplicationSchemaDeclarationBuilder<CapabilitySchema> {
    schema
        .capability_context(GrantContext::reference())
        .capability_provenance(GrantProvenance::reference())
        .operation(
            RevokeGrant::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(RevokeGrant::reference(), 16)
        .capability(grant_contract(
            RetentionCapability::reference(),
            SetRetention::reference(),
            1,
            1,
            ApplicationCapabilityRecordedActorRule::not_applicable(),
        ))
        .capability(grant_contract(
            WorkflowInstanceStartCapability::reference(),
            SetRetention::reference(),
            1,
            1,
            ApplicationCapabilityRecordedActorRule::not_applicable(),
        ))
        .capability(grant_contract(
            WorkflowAdvanceCapability::reference(),
            SetRetention::reference(),
            2,
            2,
            ApplicationCapabilityRecordedActorRule::not_applicable(),
        ))
        .capability(grant_contract(
            RevocationCapability::reference(),
            RevokeGrant::reference(),
            2,
            2,
            ApplicationCapabilityRecordedActorRule::not_applicable(),
        ))
        .capability(recorded_operation::contract())
}
pub fn grant_contract<Capability, Operation, Input>(
    capability: ApplicationCapabilityRef<CapabilitySchema, Capability>,
    operation: ApplicationOperationRef<CapabilitySchema, Operation, Input>,
    action: u64,
    purpose: u64,
    recorded_actor: ApplicationCapabilityRecordedActorRule,
) -> ApplicationCapabilityContract<CapabilitySchema, Capability, Operation, Input>
where
    Operation: ApplicationOperationMarkerIdentity<CapabilitySchema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    grant_contract_for_field(
        capability,
        operation,
        action,
        purpose,
        recorded_actor,
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityDisclosureRule::not_applicable(),
    )
}
pub fn grant_contract_for_field<Capability, Operation, Input>(
    capability: ApplicationCapabilityRef<CapabilitySchema, Capability>,
    operation: ApplicationOperationRef<CapabilitySchema, Operation, Input>,
    action: u64,
    purpose: u64,
    recorded_actor: ApplicationCapabilityRecordedActorRule,
    field: ApplicationCapabilityFieldDimension,
    disclosure: ApplicationCapabilityDisclosureRule,
) -> ApplicationCapabilityContract<CapabilitySchema, Capability, Operation, Input>
where
    Operation: ApplicationOperationMarkerIdentity<CapabilitySchema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    let encoded = |value| {
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(value).unwrap()
    };
    let scalar =
        || ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1).unwrap();
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(GrantGrantee::reference())
        .forward(GrantResource::reference())
        .allow(Document::reference());
    ApplicationCapabilityContractBuilder::new(capability, operation, Grant::reference())
        .target(ApplicationCapabilityTargetDefinition::new(
            ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), encoded(action))
                .unwrap(),
            ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
            ApplicationCapabilityRelationDimension::not_applicable(),
            field,
            ApplicationCapabilityValueBinding::new(
                GrantPurpose::at("GrantPurpose"),
                encoded(purpose),
            )
            .unwrap(),
        ))
        .constraints(ApplicationCapabilityConstraintDefinition::new(
            ApplicationCapabilityMagnitudeDimension::not_applicable(),
            ApplicationCapabilityCardinalityDimension::One,
            ApplicationCapabilityCurrentnessDefinition::new(
                ApplicationCapabilityValueBinding::new(GrantStatus::at("GrantStatus"), scalar())
                    .unwrap(),
                ApplicationCapabilityWorkflowDefinition::new(
                    ApplicationCapabilityFieldBinding::from_reference(GrantScopeKey::at(
                        "GrantScopeKey",
                    ))
                    .unwrap(),
                    ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::at(
                        "DocumentIdentityField",
                    ))
                    .unwrap(),
                ),
                ApplicationCapabilityValidityDefinition::new(
                    ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                    ApplicationCapabilityFieldBinding::from_reference(GrantNotBefore::at(
                        "GrantNotBefore",
                    ))
                    .unwrap(),
                    ApplicationCapabilityFieldBinding::from_reference(GrantNotAfter::at(
                        "GrantNotAfter",
                    ))
                    .unwrap(),
                ),
            ),
            GrantContext::reference(),
        ))
        .delegation(
            ApplicationCapabilityDelegationDefinition::new(
                ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
                ApplicationCapabilityRelationBinding::from_reference(GrantGrantor::reference()),
                ApplicationCapabilityRelationBinding::from_reference(GrantGrantee::reference()),
                ApplicationCapabilityFieldBinding::from_reference(GrantDepth::at("GrantDepth"))
                    .unwrap(),
                GrantProvenance::reference(),
            )
            .with_revocation(ApplicationCapabilityRevocationDefinition::new(
                RevokeGrant::reference(),
                ApplicationCapabilityFieldBinding::from_reference(GrantIdentity::at(
                    "GrantIdentity",
                ))
                .unwrap(),
                ApplicationCapabilityValueBinding::new(GrantStatus::at("GrantStatus"), encoded(0))
                    .unwrap(),
            )),
        )
        .composition(ApplicationCapabilityComposition::new(
            ApplicationCapabilityDecisionComposition::new(
                ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                    ApplicationCapabilityGraphClause::new(path),
                ])),
                ApplicationCapabilityDenyRule::not_applicable(),
                ApplicationCapabilityConflictRule::not_applicable(),
            ),
            ApplicationCapabilityActorComposition::new(
                ApplicationCapabilitySeparationOfDutyRule::not_applicable(),
                ApplicationCapabilityDistinctActorRule::not_applicable(),
                recorded_actor,
            ),
            ApplicationCapabilityPropagationComposition::new(
                ApplicationCapabilityDelegationRule::forbidden(),
                disclosure,
            ),
        ))
        .elevation(ApplicationCapabilityElevationRule::not_applicable())
        .build()
}

pub use crate::facade::certification::application_graph::source_only::capability::schema::storage_model::{Grant, GrantFacts, GrantAction, GrantPurpose, GrantStatus, GrantNotBefore, GrantNotAfter, GrantDepth, GrantIdentity, GrantScopeKey, GrantResource, GrantGrantee, GrantGrantor, GrantParent};
