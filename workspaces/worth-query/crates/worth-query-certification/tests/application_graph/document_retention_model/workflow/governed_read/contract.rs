//! One authored read contract geometry, bound independently to each concrete read capability.
use super::super::declaration::*;
use super::*;
pub(super) fn read<C: ReadCapability, O>(
    cap: ApplicationCapabilityRef<DocumentRetentionSchema, C>,
    op: ApplicationOperationRef<DocumentRetentionSchema, O, WorkflowReadInput>,
) -> ApplicationCapabilityContract<DocumentRetentionSchema, C, O, WorkflowReadInput>
where
    O: ApplicationOperationMarkerIdentity<
        DocumentRetentionSchema,
        InputBinding = WorkflowReadInputBinding,
    >,
{
    ApplicationCapabilityContractBuilder::new(cap,op,WorkflowAuthoringGrant::reference())
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(WorkflowGrantActionField::reference(),encoded(C::ACTION.to_owned())),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(WorkflowGrantPurposeField::reference(),encoded("reviewed-document".to_owned()))))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(WorkflowGrantStatusField::reference(),encoded("active".to_owned())),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantWorkflowField::reference()),
                ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::reference())),
            ApplicationCapabilityValidityDefinition::new(ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotBeforeField::reference()),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotAfterField::reference()))),WorkflowReadContext::reference()))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantDelegationLimitField::reference()),WorkflowReadProvenance::reference()))
    .composition(super::super::contract::composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable()).build()
}
