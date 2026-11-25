//! Capability, ability, and policy declarations for the governed board court.
use super::*;
use worth_query_decl::facade::application_capability::*;
use worth_query_decl::facade::application_schema::fragment::Registration;
use worth_query_decl::facade::*;

worth_query_capability!(pub ReadBoard in GovSchema, identity "related-absence.gov.cap.v1");
worth_query_capability_context!(pub BoardCtx in GovSchema);
worth_query_capability_provenance!(pub BoardProv in GovSchema);
worth_query_ability!(pub ReadBoardRows scoped_to Project, in GovSchema);
worth_query_policy!(pub BoardReadPolicy in GovSchema);

#[derive(Clone, Debug, worth_foundational::facade::WorthValue)]
pub struct ReadBoardAccess {
    pub project: String,
}
worth_query_structured_value_binding!(pub AccessInput for ReadBoardAccess { identity: "related-absence.gov.access.v1" });
worth_query_operation!(pub InspectBoard for GovSchema, input AccessInput);
worth_query_operation_reads!(InspectBoard => [ProjectKey, GrantDisclosure]);

pub fn tag(v: u64) -> ApplicationEncodedScalarValue<U64ApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(v).unwrap()
}

impl ApplicationCapabilityRequest<GovSchema, ReadBoard> for ReadBoardAccess {
    type Scope = Project;
    type Context = BoardCtx;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<GovSchema, Project, BoardCtx>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                ProjectKey::at("ProjectKey"),
                ApplicationEncodedScalarValue::<StringApplicationValueBinding>::try_new(
                    self.project.clone(),
                )
                .unwrap(),
            )
            .unwrap(),
            tag(1),
            tag(1),
            ApplicationCapabilityRequestContext::new(BoardCtx::reference()),
        )
        .field(tag(1)))
    }
}

pub struct GovAccess;
impl Registration<GovSchema> for GovAccess {
    fn register(
        builder: ApplicationSchemaDeclarationBuilder<GovSchema>,
    ) -> Result<ApplicationSchemaDeclarationBuilder<GovSchema>, ApplicationSchemaDeclarationDenial>
    {
        let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
            .forward(GrantGrantee::reference())
            .forward(GrantResource::reference())
            .allow(Project::reference());
        let cap = ApplicationCapabilityContractBuilder::new(
            ReadBoard::reference(),
            InspectBoard::reference(),
            BoardGrant::reference(),
        )
        .target(ApplicationCapabilityTargetDefinition::new(
            ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), tag(1)).unwrap(),
            ApplicationCapabilityRelationBinding::from_reference(GrantResource::reference()),
            ApplicationCapabilityRelationDimension::not_applicable(),
            ApplicationCapabilityFieldDimension::bound(GrantDisclosure::at("GrantDisclosure"))
                .unwrap(),
            ApplicationCapabilityValueBinding::new(GrantPurpose::at("GrantPurpose"), tag(1))
                .unwrap(),
        ))
        .constraints(ApplicationCapabilityConstraintDefinition::new(
            ApplicationCapabilityMagnitudeDimension::not_applicable(),
            ApplicationCapabilityCardinalityDimension::One,
            ApplicationCapabilityCurrentnessDefinition::new(
                ApplicationCapabilityValueBinding::new(GrantStatus::at("GrantStatus"), tag(1))
                    .unwrap(),
                ApplicationCapabilityWorkflowDefinition::new(
                    ApplicationCapabilityFieldBinding::from_reference(GrantScope::at("GrantScope"))
                        .unwrap(),
                    ApplicationCapabilityFieldBinding::from_reference(ProjectKey::at("ProjectKey"))
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
            BoardCtx::reference(),
        ))
        .delegation(ApplicationCapabilityDelegationDefinition::new(
            ApplicationCapabilityRelationBinding::from_reference(GrantParent::reference()),
            ApplicationCapabilityRelationBinding::from_reference(GrantGrantor::reference()),
            ApplicationCapabilityRelationBinding::from_reference(GrantGrantee::reference()),
            ApplicationCapabilityFieldBinding::from_reference(GrantDepth::at("GrantDepth"))
                .unwrap(),
            BoardProv::reference(),
        ))
        .composition(ApplicationCapabilityComposition::new(
            ApplicationCapabilityDecisionComposition::new(
                ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                    ApplicationCapabilityGraphClause::new(path.clone()),
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
                        ApplicationCapabilityAcceptedValues::one_of(
                            GrantDisclosure::at("GrantDisclosure"),
                            [tag(1)],
                        )
                        .unwrap(),
                    ]),
                ]),
            ),
        ))
        .elevation(ApplicationCapabilityElevationRule::not_applicable())
        .build();
        Ok(builder
            .capability_context(BoardCtx::reference())
            .capability_provenance(BoardProv::reference())
            .ability(ReadBoardRows::reference())
            .policy(BoardReadPolicy::reference())
            .ability_policy(
                ReadBoardRows::reference(),
                BoardReadPolicy::reference(),
                [path],
            )
            .operation(
                InspectBoard::reference()
                    .definition()
                    .no_external_effect()
                    .no_aftermath()
                    .finish(),
            )
            .operation_projection_work_budget(InspectBoard::reference(), 32)
            .operation_read_field(InspectBoard::reference(), ProjectKey::at("ProjectKey"))
            .capability(cap))
    }
}
