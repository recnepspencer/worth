//! One authored read contract geometry, bound independently to each concrete read capability.
pub use super::super::declaration::*;
use super::encoded;
pub use super::*;
pub fn read<C: ReadCapability, O>(
    cap: ApplicationCapabilityRef<DocumentRetentionSchema, C>,
    op: ApplicationOperationRef<DocumentRetentionSchema, O, WorkflowReadInput>,
) -> ApplicationCapabilityContract<DocumentRetentionSchema, C, O, WorkflowReadInput>
where
    O: ApplicationOperationMarkerIdentity<
        DocumentRetentionSchema,
        InputBinding = WorkflowReadInputBinding,
    >,
{
    for_input(cap, op, C::ACTION)
}

pub fn for_input<C, O, Input>(
    cap: ApplicationCapabilityRef<DocumentRetentionSchema, C>,
    op: ApplicationOperationRef<DocumentRetentionSchema, O, Input>,
    action: &str,
) -> ApplicationCapabilityContract<DocumentRetentionSchema, C, O, Input>
where
    O: ApplicationOperationMarkerIdentity<DocumentRetentionSchema>,
    O::InputBinding: ApplicationStructuredValueBinding<Value = Input>,
{
    ApplicationCapabilityContractBuilder::new(cap,op,WorkflowAuthoringGrant::reference())
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(WorkflowGrantActionField::at("WorkflowGrantActionField"),encoded(action.to_owned())).unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),ApplicationCapabilityFieldDimension::not_applicable(),
        ApplicationCapabilityValueBinding::new(WorkflowGrantPurposeField::at("WorkflowGrantPurposeField"),encoded("reviewed-document".to_owned())).unwrap()))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(WorkflowGrantStatusField::at("WorkflowGrantStatusField"),encoded("active".to_owned())).unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantWorkflowField::at("WorkflowGrantWorkflowField")).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(DocumentIdentityField::at("DocumentIdentityField")).unwrap()),
            ApplicationCapabilityValidityDefinition::new(ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotBeforeField::at("WorkflowGrantNotBeforeField")).unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantNotAfterField::at("WorkflowGrantNotAfterField")).unwrap())),WorkflowReadContext::reference()))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(WorkflowGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantDelegationLimitField::at("WorkflowGrantDelegationLimitField")).unwrap(),WorkflowReadProvenance::reference()))
    .composition(super::super::contract::composition())
    .elevation(ApplicationCapabilityElevationRule::not_applicable()).build()
}
