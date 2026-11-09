//! Installs the deployment grant model and its existing Query capability policy.
use super::capability::encoded;
use super::*;
use worth_query_decl::facade::application_capability::*;
pub(crate) fn install(
    schema: ApplicationSchemaDeclarationBuilder<BankSchema>,
) -> ApplicationSchemaDeclarationBuilder<BankSchema> {
    let schema = schema
        .capability_context(ProgramDeploymentContext::reference())
        .capability_provenance(ProgramDeploymentProvenance::reference())
        .operation(
            DeployBankProgramOperation::reference()
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_projection_work_budget(DeployBankProgramOperation::reference(), 512)
        .operation_read_field(
            DeployBankProgramOperation::reference(),
            ProgramDeploymentSourceRevision::at("ProgramDeploymentSourceRevision"),
        )
        .capability(contract());
    super::revocation::install(schema)
}
fn contract() -> ApplicationCapabilityContract<
    BankSchema,
    DeployBankProgram,
    DeployBankProgramOperation,
    ProgramDeploymentInput,
> {
    let path = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(ProgramDeploymentGrantGrantee::reference())
        .forward(ProgramDeploymentGrantResource::reference())
        .allow(ProgramDeploymentScope::reference());
    ApplicationCapabilityContractBuilder::new(
        DeployBankProgram::reference(),
        DeployBankProgramOperation::reference(),
        ProgramDeploymentGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            ProgramDeploymentGrantAction::at("ProgramDeploymentGrantAction"),
            encoded("deploy-bank-program".to_owned()),
        ).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(
            ProgramDeploymentGrantResource::reference(),
        ),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(
            ProgramDeploymentGrantPurpose::at("ProgramDeploymentGrantPurpose"),
            encoded("program-deployment".to_owned()),
        ).unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                ProgramDeploymentGrantStatus::at("ProgramDeploymentGrantStatus"),
                encoded("active".to_owned()),
            ).unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(
                    ProgramDeploymentGrantSourceRevision::at("ProgramDeploymentGrantSourceRevision"),
                ).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(
                    ProgramDeploymentSourceRevision::at("ProgramDeploymentSourceRevision"),
                ).unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(
                    ProgramDeploymentGrantNotBefore::at("ProgramDeploymentGrantNotBefore"),
                ).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(
                    ProgramDeploymentGrantNotAfter::at("ProgramDeploymentGrantNotAfter"),
                ).unwrap(),
            ),
        ),
        ProgramDeploymentContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(
            ProgramDeploymentGrantParent::reference(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            ProgramDeploymentGrantGrantor::reference(),
        ),
        ApplicationCapabilityRelationBinding::from_reference(
            ProgramDeploymentGrantGrantee::reference(),
        ),
        ApplicationCapabilityFieldBinding::from_reference(ProgramDeploymentGrantDepth::at("ProgramDeploymentGrantDepth")).unwrap(),
        ProgramDeploymentProvenance::reference(),
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
