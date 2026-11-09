//! The correction grant has one scope and admits no delegation.
use super::*;
pub(in super::super::super) fn declare<Schema: TopologySchemaBinding>(
    schema: ApplicationSchemaDeclarationBuilder<Schema>,
) -> ApplicationSchemaDeclarationBuilder<Schema> {
    let contract = grant_contract(
        CorrectEntryCapability::<Schema>::reference(),
        CorrectEntryValue::reference::<Schema>(),
        1,
        1,
    );
    let schema = schema
        .capability_context(CorrectionContext::<Schema>::reference())
        .capability_provenance(CorrectionProvenance::<Schema>::reference())
        .capability(contract);
    super::super::program_adoption::declare(schema)
}

pub(in super::super::super) fn grant_contract<
    Schema: TopologySchemaBinding,
    Capability,
    Operation,
    Input,
>(
    capability: ApplicationCapabilityRef<Schema, Capability>,
    operation: ApplicationOperationRef<Schema, Operation, Input>,
    action: u64,
    purpose: u64,
) -> ApplicationCapabilityContract<Schema, Capability, Operation, Input>
where
    Operation: ApplicationOperationMarkerIdentity<Schema>,
    Operation::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    let encoded = |value| {
        ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(value).unwrap()
    };
    let scalar =
        || ApplicationEncodedScalarValue::<U64ApplicationValueBinding>::try_new(1).unwrap();
    let path =
        ApplicationAuthorizationPathBuilder::from_principal(Principal::reference::<Schema>())
            .forward(CorrectionGrantee::reference::<Schema>())
            .forward(CorrectionResource::reference::<Schema>())
            .allow(Body::reference::<Schema>());
    ApplicationCapabilityContractBuilder::new(
        capability,
        operation,
        CorrectionGrant::reference::<Schema>(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            CorrectionAction::at("CorrectionAction"),
            encoded(action),
        ).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(
            CorrectionResource::reference::<Schema>(),
        ),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            CorrectionPurpose::at("CorrectionPurpose"),
            encoded(purpose),
        ).unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                CorrectionStatus::at("CorrectionStatus"),
                scalar(),
            ).unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(CorrectionScopeKey::at("CorrectionScopeKey")).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(BodyKey::at("BodyKey")).unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    CorrectionNotBefore::at("CorrectionNotBefore"),
                ).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(CorrectionNotAfter::at("CorrectionNotAfter")).unwrap(),
            ),
        ),
        CorrectionContext::<Schema>::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(
            CorrectionParent::reference::<Schema>(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            CorrectionGrantor::reference::<Schema>(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            CorrectionGrantee::reference::<Schema>(),
        ),
        ApplicationCapabilityFieldBinding::from_reference(CorrectionDepth::at("CorrectionDepth")).unwrap(),
        CorrectionProvenance::<Schema>::reference(),
    ))
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
            ApplicationCapabilityDistinctActorRule::not_applicable(), worth_query_decl::facade::application_capability::ApplicationCapabilityRecordedActorRule::not_applicable()),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::not_applicable(),
        ),
    ))
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
