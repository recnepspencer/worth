//! WORTH capability contracts (`ReadProjectCapability` and `WatchProjectCapability`).

use worth_query_decl::facade::{
    application_capability::*,
    application_schema::{fragment::Registration, *},
    worth_query_capability, worth_query_capability_context, worth_query_capability_provenance,
    worth_query_operation, worth_query_operation_reads, worth_query_structured_value_binding,
};

use super::schema::{
    ExternalIdentityField, ExternalMapping, GrantActionField, GrantActiveField,
    GrantDelegationLimitField, GrantDisclosureField, GrantModeField, GrantNotAfterField,
    GrantNotBeforeField, GrantParent, GrantPurposeField, GrantResource, GrantWorkflowField,
    Grantee, Grantor, MappingStatusField, MappingTarget, Principal, PrincipalIdField, Project,
    ProjectGrant, ProjectKeyField, ProjectVersionField, SharedProjectSchema,
};

pub struct SharedPrincipalBinding;
impl SharedPrincipalBinding {
    pub fn reference() -> ApplicationPrincipalBindingRef<
        SharedProjectSchema,
        Self,
        ExternalMapping,
        Principal,
        u64,
        U64ApplicationValueBinding,
    > {
        ApplicationPrincipalBindingRef::from_requirements(
            "SharedPrincipalBinding",
            ApplicationPrincipalBindingRequirements {
                mapping_identity: ApplicationPrincipalMappingIdentityRequirement::from_field(
                    ExternalIdentityField::at("ExternalIdentityField"),
                ),
                mapping_status: ApplicationPrincipalMappingStatusRequirement::from_field(
                    MappingStatusField::at("MappingStatusField"),
                ),
                target: ApplicationPrincipalTargetRequirement::from_relation(
                    MappingTarget::reference(),
                ),
                principal_identity: ApplicationPrincipalIdentityRequirement::from_field(
                    PrincipalIdField::at("PrincipalIdField"),
                ),
            },
        )
    }
}

impl Registration<SharedProjectSchema> for SharedPrincipalBinding {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(builder.principal_binding(Self::reference()))
    }
}

worth_query_capability_context!(pub SharedAccessContext in SharedProjectSchema);
worth_query_capability_provenance!(pub SharedAccessProvenance in SharedProjectSchema);
worth_query_capability!(pub ReadProjectCapability in SharedProjectSchema);
worth_query_capability!(pub WatchProjectCapability in SharedProjectSchema);
worth_query_structured_value_binding!(pub ProjectCapabilityInputBinding for String {
    identity: "worth.query.court.shared_access.cap_input.v1"
});
worth_query_operation!(pub ReadProjectOperation for SharedProjectSchema, input ProjectCapabilityInputBinding);
worth_query_operation!(pub WatchProjectOperation for SharedProjectSchema, input ProjectCapabilityInputBinding);
worth_query_operation_reads!(ReadProjectOperation => [ProjectKeyField, ProjectVersionField]);
worth_query_operation_reads!(WatchProjectOperation => [ProjectKeyField, ProjectVersionField]);

pub fn text(value: &str) -> ApplicationEncodedScalarValue<StringApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value.to_owned()).unwrap()
}

fn project_capability_request(
    project_key: &str,
    field_token: &str,
) -> Result<
    ApplicationCapabilityRequestProjection<SharedProjectSchema, Project, SharedAccessContext>,
    ApplicationCapabilityRequestProjectionDenial,
> {
    Ok(ApplicationCapabilityRequestProjection::new(
        ApplicationCapabilityEntitySelector::new(
            ProjectKeyField::at("ProjectKey"),
            text(project_key),
        )
        .unwrap(),
        text("access"),
        text("shared"),
        ApplicationCapabilityRequestContext::new(SharedAccessContext::reference()),
    )
    .field(text(field_token)))
}

impl ApplicationCapabilityRequest<SharedProjectSchema, ReadProjectCapability> for String {
    type Scope = Project;
    type Context = SharedAccessContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<SharedProjectSchema, Project, SharedAccessContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        project_capability_request(self, "project_version")
    }
}

impl ApplicationCapabilityRequest<SharedProjectSchema, WatchProjectCapability> for String {
    type Scope = Project;
    type Context = SharedAccessContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<SharedProjectSchema, Project, SharedAccessContext>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        project_capability_request(self, "read_and_watch")
    }
}

impl Registration<SharedProjectSchema> for SharedAccessContext {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability_context(Self::reference()))
    }
}

impl Registration<SharedProjectSchema> for SharedAccessProvenance {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability_provenance(Self::reference()))
    }
}

impl Registration<SharedProjectSchema> for ReadProjectOperation {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .operation(
                Self::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(Self::reference(), 64)
            .operation_read_field(Self::reference(), ProjectKeyField::at("ProjectKey"))
            .operation_read_field(Self::reference(), ProjectVersionField::at("ProjectVersion")))
    }
}

impl Registration<SharedProjectSchema> for WatchProjectOperation {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema
            .operation(
                Self::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(Self::reference(), 64)
            .operation_read_field(Self::reference(), ProjectKeyField::at("ProjectKey"))
            .operation_read_field(Self::reference(), ProjectVersionField::at("ProjectVersion")))
    }
}

impl Registration<SharedProjectSchema> for ReadProjectCapability {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability(build_capability_contract(
            ReadProjectCapability::reference(),
            ReadProjectOperation::reference(),
            GrantDisclosureField::at("GrantDisclosure"),
            vec![text("project_version")],
        )))
    }
}

impl Registration<SharedProjectSchema> for WatchProjectCapability {
    fn register(
        schema: ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
    ) -> Result<
        ApplicationSchemaDeclarationBuilder<SharedProjectSchema>,
        ApplicationSchemaDeclarationDenial,
    > {
        Ok(schema.capability(build_capability_contract(
            WatchProjectCapability::reference(),
            WatchProjectOperation::reference(),
            GrantModeField::at("GrantMode"),
            vec![text("read_and_watch")],
        )))
    }
}

fn build_capability_contract<Cap, Op>(
    capability: ApplicationCapabilityRef<SharedProjectSchema, Cap>,
    operation: ApplicationOperationRef<SharedProjectSchema, Op, String>,
    field_dimension: GrantDisclosureField,
    accepted_values: Vec<ApplicationEncodedScalarValue<StringApplicationValueBinding>>,
) -> ApplicationCapabilityContract<SharedProjectSchema, Cap, Op, String>
where
    Cap: ApplicationCapabilityMarkerIdentity + 'static,
    Op: ApplicationOperationMarkerIdentity<SharedProjectSchema> + 'static,
    Op::InputBinding: ApplicationStructuredValueBinding<Value = String>,
    String: ApplicationCapabilityRequest<SharedProjectSchema, Cap>,
{
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(Grantee::reference())
        .forward(GrantResource::reference())
        .allow(Project::reference());
    ApplicationCapabilityContractBuilder::new(capability, operation, ProjectGrant::reference())
        .target(capability_target(field_dimension))
        .constraints(capability_constraints())
        .delegation(capability_delegation())
        .composition(capability_composition(
            allow,
            field_dimension,
            accepted_values,
        ))
        .elevation(ApplicationCapabilityElevationRule::not_applicable())
        .build()
}

fn capability_target(
    field_dimension: GrantDisclosureField,
) -> ApplicationCapabilityTargetDefinition {
    ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(GrantActionField::at("GrantAction"), text("access"))
            .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(field_dimension).unwrap(),
        ApplicationCapabilityValueBinding::new(
            GrantPurposeField::at("GrantPurpose"),
            text("shared"),
        )
        .unwrap(),
    )
}

fn capability_constraints() -> ApplicationCapabilityConstraintDefinition {
    ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                GrantActiveField::at("GrantActive"),
                ApplicationEncodedScalarValue::<BoolApplicationValueBinding>::try_new(true)
                    .unwrap(),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantWorkflowField::at(
                    "GrantWorkflow",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(ProjectKeyField::at(
                    "ProjectKey",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(GrantNotBeforeField::at(
                    "GrantNotBefore",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(GrantNotAfterField::at(
                    "GrantNotAfter",
                ))
                .unwrap(),
            ),
        ),
        SharedAccessContext::reference(),
    )
}

fn capability_delegation() -> ApplicationCapabilityDelegationDefinition {
    ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(Grantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(GrantDelegationLimitField::at(
            "GrantDelegationLimit",
        ))
        .unwrap(),
        SharedAccessProvenance::reference(),
    )
}

fn capability_composition(
    allow: ApplicationAuthorizationPath,
    field_dimension: GrantDisclosureField,
    accepted_values: Vec<ApplicationEncodedScalarValue<StringApplicationValueBinding>>,
) -> ApplicationCapabilityComposition {
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
            ApplicationCapabilityDistinctActorRule::not_applicable(),
            ApplicationCapabilityRecordedActorRule::not_applicable(),
        ),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(field_dimension, accepted_values)
                        .unwrap(),
                ]),
            ]),
        ),
    )
}
