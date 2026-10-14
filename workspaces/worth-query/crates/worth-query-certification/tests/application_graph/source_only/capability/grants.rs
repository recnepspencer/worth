//! Actual grant declarations and request projections; native Query evaluates them.
use super::schema::*;
use worth_query_host::facade::{
    declaration::application_capability::*, declaration::application_schema::*, *,
};
worth_query_entity!(pub Grant for CapabilitySchema);
worth_query_aspect!(pub GrantFacts for CapabilitySchema, Grant; identity=AspectIdentity(0x91765001), revision=AspectContractRevision(1));
worth_query_field!(pub GrantAction for CapabilitySchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantPurpose for CapabilitySchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantStatus for CapabilitySchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_write, equality);
worth_query_field!(pub GrantNotBefore for CapabilitySchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantNotAfter for CapabilitySchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantDepth for CapabilitySchema, Grant, GrantFacts: u64 => U64ApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantIdentity for CapabilitySchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_field!(pub GrantScopeKey for CapabilitySchema, Grant, GrantFacts: String => StringApplicationValueBinding, read_only, equality);
worth_query_relation!(pub GrantResource in CapabilitySchema, Grant => Document; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantee in CapabilitySchema, Principal => Grant; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantGrantor in CapabilitySchema, Principal => Grant; integrity=same_context_unbounded_retain_dangling);
worth_query_relation!(pub GrantParent in CapabilitySchema, Grant => Grant; integrity=same_context_unbounded_retain_dangling);
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
worth_query_operation_reads!(RevokeGrant => [DocumentIdentityField, GrantIdentity, GrantStatus, GrantResource]);
fn projection(
    action: u64,
    identity: &str,
) -> ApplicationCapabilityRequestProjection<CapabilitySchema, Document, GrantContext> {
    ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            DocumentIdentityField::reference(),
            ApplicationEncodedScalarValue::try_new(identity.to_owned()).unwrap(),
        ),
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
                GrantIdentity::reference(),
                ApplicationEncodedScalarValue::try_new(self.grant.clone()).unwrap(),
            ),
        ))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<CapabilitySchema>,
) -> ApplicationSchemaDeclarationBuilder<CapabilitySchema> {
    let schema = schema
        .entity(Grant::reference())
        .aspect(Grant::reference(), GrantFacts::reference());
    let schema = schema.field(Grant::reference(), GrantIdentity::reference());
    let schema = schema.field(Grant::reference(), GrantAction::reference());
    let schema = schema.field(Grant::reference(), GrantPurpose::reference());
    let schema = schema.field(Grant::reference(), GrantStatus::reference());
    let schema = schema.field(Grant::reference(), GrantScopeKey::reference());
    let schema = schema.field(Grant::reference(), GrantNotBefore::reference());
    let schema = schema.field(Grant::reference(), GrantNotAfter::reference());
    let schema = schema.field(Grant::reference(), GrantDepth::reference());
    let schema = schema.relation(
        GrantResource::reference(),
        Grant::reference(),
        Document::reference(),
    );
    let schema = schema.relation(
        GrantGrantee::reference(),
        Principal::reference(),
        Grant::reference(),
    );
    let schema = schema.relation(
        GrantGrantor::reference(),
        Principal::reference(),
        Grant::reference(),
    );
    let schema = schema.relation(
        GrantParent::reference(),
        Grant::reference(),
        Grant::reference(),
    );
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
        ))
        .capability(grant_contract(
            RevocationCapability::reference(),
            RevokeGrant::reference(),
            2,
            2,
        ))
}
fn grant_contract<Capability, Operation, Input>(
    capability: ApplicationCapabilityRef<CapabilitySchema, Capability>,
    operation: ApplicationOperationRef<CapabilitySchema, Operation, Input>,
    action: u64,
    purpose: u64,
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
    let contract =
        ApplicationCapabilityContractBuilder::new(capability, operation, Grant::reference())
            .target(ApplicationCapabilityTargetDefinition::new(
                ApplicationCapabilityValueBinding::new(GrantAction::reference(), encoded(action)),
                ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
                ApplicationCapabilityRelationDimension::not_applicable(),
                ApplicationCapabilityFieldDimension::not_applicable(),
                ApplicationCapabilityValueBinding::new(GrantPurpose::reference(), encoded(purpose)),
            ))
            .constraints(ApplicationCapabilityConstraintDefinition::new(
                ApplicationCapabilityMagnitudeDimension::not_applicable(),
                ApplicationCapabilityCardinalityDimension::One,
                ApplicationCapabilityCurrentnessDefinition::new(
                    ApplicationCapabilityValueBinding::new(GrantStatus::reference(), scalar()),
                    ApplicationCapabilityWorkflowDefinition::new(
                        ApplicationCapabilityFieldBinding::from_reference(
                            GrantScopeKey::reference(),
                        ),
                        ApplicationCapabilityFieldBinding::from_reference(
                            DocumentIdentityField::reference(),
                        ),
                    ),
                    ApplicationCapabilityValidityDefinition::new(
                        ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                        ApplicationCapabilityFieldBinding::from_reference(
                            GrantNotBefore::reference(),
                        ),
                        ApplicationCapabilityFieldBinding::from_reference(
                            GrantNotAfter::reference(),
                        ),
                    ),
                ),
                GrantContext::reference(),
            ))
            .delegation(
                ApplicationCapabilityDelegationDefinition::new(
                    ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
                    ApplicationCapabilityRelationBinding::from_reference(GrantGrantor::reference()),
                    ApplicationCapabilityRelationBinding::from_reference(GrantGrantee::reference()),
                    ApplicationCapabilityFieldBinding::from_reference(GrantDepth::reference()),
                    GrantProvenance::reference(),
                )
                .with_revocation(ApplicationCapabilityRevocationDefinition::new(
                    RevokeGrant::reference(),
                    ApplicationCapabilityFieldBinding::from_reference(GrantIdentity::reference()),
                    ApplicationCapabilityValueBinding::new(GrantStatus::reference(), encoded(0)),
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
                ),
                ApplicationCapabilityPropagationComposition::new(
                    ApplicationCapabilityDelegationRule::forbidden(),
                    ApplicationCapabilityDisclosureRule::not_applicable(),
                ),
            ))
            .elevation(ApplicationCapabilityElevationRule::not_applicable())
            .build();
    contract
}
