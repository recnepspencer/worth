//! Install the complete grant contract; no test-issued authorization objects.
use super::super::super::schema::*;
use super::declarations::*;
use worth_query_declaration::facade::{application_capability::*, application_schema::*};
pub(super) fn declare<S: OrbitModel>(
    schema: ApplicationSchemaDeclarationBuilder<S>,
) -> ApplicationSchemaDeclarationBuilder<S> {
    schema
        .capability_context(QueryContext::reference())
        .capability_provenance(QueryProvenance::reference())
        .operation(
            Inspect::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_entity(Inspect::reference(), Node::reference())
        .operation_read_field(Inspect::reference(), NodeKey::at("NodeKey"))
        .capability(contract::<S>())
}
fn contract<S: OrbitModel>() -> ApplicationCapabilityContract<S, ReadOrbit<S>, Inspect, InspectInput>
{
    ApplicationCapabilityContractBuilder::new(
        ReadOrbit::reference(),
        Inspect::reference(),
        Grant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(GrantAction::at("GrantAction"), flag(false))
            .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(Resource::reference::<S>()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(GrantDisclosure::at("GrantDisclosure")).unwrap(),
        ApplicationCapabilityValueBinding::new(GrantPurpose::at("GrantPurpose"), flag(false))
            .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(GrantActive::at("GrantActive"), flag(true))
                .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(GrantWorkflow::at(
                    "GrantWorkflow",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(Marked::at("Marked")).unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(GrantFrom::at("GrantFrom"))
                    .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(GrantUntil::at("GrantUntil"))
                    .unwrap(),
            ),
        ),
        QueryContext::<S>::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(ParentGrant::reference::<S>()),
        ApplicationCapabilityRelationBinding::from_reference(Issuer::reference::<S>()),
        ApplicationCapabilityRelationBinding::from_reference(Holder::reference::<S>()),
        ApplicationCapabilityFieldBinding::from_reference(GrantLimit::at("GrantLimit")).unwrap(),
        QueryProvenance::<S>::reference(),
    ))
    .composition(ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any(
                [
                    ApplicationCapabilityGraphClause::new(
                        ApplicationAuthorizationPathBuilder::from_principal(
                            Principal::reference::<S>(),
                        )
                        .forward(Holder::reference())
                        .forward(Resource::reference())
                        .allow(Node::reference()),
                    ),
                ],
            )),
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
                        [
                            ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(0)
                                .unwrap(),
                        ],
                    )
                    .unwrap(),
                ]),
            ]),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
fn flag(value: bool) -> ApplicationEncodedScalarValue<BoolApplicationValueBinding> {
    ApplicationEncodedScalarValue::try_new(value).unwrap()
}
