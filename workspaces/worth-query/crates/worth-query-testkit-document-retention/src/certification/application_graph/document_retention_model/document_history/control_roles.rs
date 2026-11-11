//! A real grant record has independent READ and NAV, but cannot be historical payload.
use super::super::record_history::capability::{encoded, text};
use super::super::{schema::*, workflow::declaration::*};
use worth_query_decl::facade::{
    application_capability::*, application_history::*,
    application_query::ApplicationQueryDisclosureContract, application_schema::*,
    worth_query_capability,
};
use worth_query_host::facade::primary_graph::{
    WorthQueryApplicationEntitySeed, WorthQueryApplicationRelationSeed,
    WorthQueryPrimaryGraphBootstrap,
};
#[path = "control_roles/grant.rs"]
mod grant;
#[path = "control_roles/read.rs"]
pub mod read;
pub use grant::seed;
pub use grant::*;
pub const SUBJECT: &str = "record-history-payload-grant";
const GRANT: &str = "control-history-navigation-grant";
const ACTION: &str = "select-control-history";
worth_query_capability!(pub ControlNavigationCapability in DocumentRetentionSchema);
worth_query_operation!(pub SelectControlSnapshot for DocumentRetentionSchema, input read::InputBinding);

pub struct History;
worth_query_portable_type!(History => "worth.query.document-retention.control-history.v1");
impl ApplicationHistoryBinding<DocumentRetentionSchema> for History {
    type Access = read::Input;
    const IDENTITY: &'static str = "GrantRecordHistory";
}
pub struct Navigation;
impl ApplicationHistoryNavigationBinding<DocumentRetentionSchema, History> for Navigation {
    const IDENTITY: &'static str = "GrantSnapshotNavigation";
    type Capability = ControlNavigationCapability;
    type Input = read::Input;
    type InputBinding = read::InputBinding;
    type Operation = SelectControlSnapshot;
    fn input(access: &read::Input) -> read::Input {
        access.clone()
    }
}
impl ApplicationCapabilityRequest<DocumentRetentionSchema, ControlNavigationCapability>
    for read::Input
{
    type Scope = WorkflowAuthoringGrant;
    type Context = super::super::workflow::governed_read::WorkflowReadContext;
    fn capability_request(
        &self,
    ) -> Result<
        ApplicationCapabilityRequestProjection<DocumentRetentionSchema, Self::Scope, Self::Context>,
        ApplicationCapabilityRequestProjectionDenial,
    > {
        Ok(ApplicationCapabilityRequestProjection::new(
            ApplicationCapabilityEntitySelector::new(
                WorkflowGrantIdentityField::at("WorkflowGrantIdentityField"),
                ApplicationEncodedScalarValue::<WorkflowGrantKeyBinding>::try_new(
                    self.identity.clone().into(),
                )
                .expect("owned grant key encodes"),
            )
            .unwrap(),
            text(ACTION),
            encoded(),
            ApplicationCapabilityRequestContext::new(Self::Context::reference()),
        )
        .field(encoded()))
    }
}
pub fn declare(
    schema: ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema>,
) -> ApplicationSchemaDeclarationBuilder<DocumentRetentionSchema> {
    let operation = SelectControlSnapshot::reference();
    let schema = grant::declare(schema);
    schema
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .operation_read_field(
            operation,
            WorkflowGrantIdentityField::at("WorkflowGrantIdentityField"),
        )
        .capability(contract())
        .application_query(read::definition())
        .application_query_binding::<read::Binding>()
        .application_history(
            ApplicationHistoryDefinition::<DocumentRetentionSchema, History>::for_binding(
                ApplicationQueryDisclosureContract::public(),
            )
            .model_domain(ApplicationHistoryModelDomain::ScopeEntityRecord)
            .navigation::<Navigation>()
            .operation::<super::super::workflow::WorkflowGrantStatusBinding>("Change grant status"),
        )
}
fn contract() -> ApplicationCapabilityContract<
    DocumentRetentionSchema,
    ControlNavigationCapability,
    SelectControlSnapshot,
    read::Input,
> {
    use super::super::workflow::governed_read::{WorkflowReadContext, WorkflowReadProvenance};
    let ordinary = super::super::workflow::contract::composition();
    let allow = ApplicationAuthorizationPathBuilder::from_principal(Principal::reference())
        .forward(WorkflowGrantGrantee::reference())
        .allow(WorkflowAuthoringGrant::reference());
    let composition = ApplicationCapabilityComposition::new(
        ApplicationCapabilityDecisionComposition::new(
            ApplicationCapabilityAllowRule::new(ApplicationCapabilityGraphRule::any([
                ApplicationCapabilityGraphClause::new(allow),
            ])),
            ApplicationCapabilityDenyRule::not_applicable(),
            ApplicationCapabilityConflictRule::not_applicable(),
        ),
        ordinary.actors().clone(),
        ApplicationCapabilityPropagationComposition::new(
            ApplicationCapabilityDelegationRule::forbidden(),
            ApplicationCapabilityDisclosureRule::permit([
                ApplicationCapabilityScopeGuard::requiring([
                    ApplicationCapabilityAcceptedValues::one_of(
                        ControlGrantPurposeField::at("ControlGrantPurposeField"),
                        [encoded()],
                    )
                    .unwrap(),
                ]),
            ]),
        ),
    );
    ApplicationCapabilityContractBuilder::new(
        ControlNavigationCapability::reference(),
        SelectControlSnapshot::reference(),
        ControlGrant::reference(),
    )
    .target(ApplicationCapabilityTargetDefinition::new(
        ApplicationCapabilityValueBinding::new(
            ControlGrantActionField::at("ControlGrantActionField"),
            text(ACTION),
        )
        .unwrap(),
        ApplicationCapabilityRelationBinding::from_reference(ControlGrantResource::reference()),
        ApplicationCapabilityRelationDimension::not_applicable(),
        ApplicationCapabilityFieldDimension::bound(ControlGrantPurposeField::at(
            "ControlGrantPurposeField",
        ))
        .unwrap(),
        ApplicationCapabilityValueBinding::new(
            ControlGrantPurposeField::at("ControlGrantPurposeField"),
            encoded(),
        )
        .unwrap(),
    ))
    .constraints(ApplicationCapabilityConstraintDefinition::new(
        ApplicationCapabilityMagnitudeDimension::not_applicable(),
        ApplicationCapabilityCardinalityDimension::One,
        ApplicationCapabilityCurrentnessDefinition::new(
            ApplicationCapabilityValueBinding::new(
                ControlGrantStatusField::at("ControlGrantStatusField"),
                text("active"),
            )
            .unwrap(),
            ApplicationCapabilityWorkflowDefinition::new(
                ApplicationCapabilityFieldBinding::from_reference(ControlGrantWorkflowField::at(
                    "ControlGrantWorkflowField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(WorkflowGrantIdentityField::at(
                    "WorkflowGrantIdentityField",
                ))
                .unwrap(),
            ),
            ApplicationCapabilityValidityDefinition::new(
                ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
                ApplicationCapabilityFieldBinding::from_reference(ControlGrantNotBeforeField::at(
                    "ControlGrantNotBeforeField",
                ))
                .unwrap(),
                ApplicationCapabilityFieldBinding::from_reference(ControlGrantNotAfterField::at(
                    "ControlGrantNotAfterField",
                ))
                .unwrap(),
            ),
        ),
        WorkflowReadContext::reference(),
    ))
    .delegation(ApplicationCapabilityDelegationDefinition::new(
        ApplicationCapabilityRelationBinding::from_reference(ControlGrantParent::reference()),
        ApplicationCapabilityRelationBinding::from_reference(ControlGrantGrantor::reference()),
        ApplicationCapabilityRelationBinding::from_reference(ControlGrantGrantee::reference()),
        ApplicationCapabilityFieldBinding::from_reference(ControlGrantDelegationLimitField::at(
            "ControlGrantDelegationLimitField",
        ))
        .unwrap(),
        WorkflowReadProvenance::reference(),
    ))
    .composition(composition)
    .elevation(ApplicationCapabilityElevationRule::not_applicable())
    .build()
}
