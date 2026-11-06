use super::*;

pub struct ExternalIdentity;
impl fragment::Aspect for ExternalIdentity {
    const NAME: &'static str = "ExternalIdentity";
    type Row = ExternalIdentityValue;
}
impl ExternalIdentity {
    pub fn reference() -> AspectSelection<PublicationRecords, ExternalMapping, Self> {
        aspect(PublicationRecords, ExternalMapping, Self)
    }
}
pub struct ExternalIdentityValue {
    pub external_identity_field:
        worth_query_declaration::facade::authentication::WorthQueryExternalPrincipalIdentity,
    pub mapping_status_field:
        worth_query_declaration::facade::authentication::WorthQueryPrincipalMappingStatus,
}
impl fragment::Record<ExternalIdentityValue> for PublicationRecordsBindings {
    fn record() -> impl fragment::Shape<ExternalIdentityValue, Self> {
        fragment::field("ExternalIdentityField", |row: &ExternalIdentityValue| {
            row.external_identity_field.clone()
        })
        .and("MappingStatusField", |row: &ExternalIdentityValue| {
            row.mapping_status_field
        })
        .build(
            |external_identity_field, mapping_status_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ExternalIdentityValue {
                    external_identity_field,
                    mapping_status_field,
                })
            },
        )
    }
}
pub struct PrincipalFacts;
impl fragment::Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalFactsValue;
}
impl PrincipalFacts {
    pub fn reference() -> AspectSelection<PublicationRecords, Principal, Self> {
        aspect(PublicationRecords, Principal, Self)
    }
}
pub struct PrincipalFactsValue {
    pub principal_identity_field: u64,
}
impl fragment::Record<PrincipalFactsValue> for PublicationRecordsBindings {
    fn record() -> impl fragment::Shape<PrincipalFactsValue, Self> {
        fragment::field("PrincipalIdentityField", |row: &PrincipalFactsValue| {
            row.principal_identity_field
        })
        .read_only()
        .build(
            |principal_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(PrincipalFactsValue {
                    principal_identity_field,
                })
            },
        )
    }
}
pub struct ResourceFacts;
impl fragment::Aspect for ResourceFacts {
    const NAME: &'static str = "ResourceFacts";
    type Row = ResourceFactsValue;
}
impl ResourceFacts {
    pub fn reference() -> AspectSelection<PublicationRecords, Resource, Self> {
        aspect(PublicationRecords, Resource, Self)
    }
}
pub struct ResourceFactsValue {
    pub resource_identity_field: String,
    pub resource_workflow_field: String,
    pub resource_label_field: String,
}
impl fragment::Record<ResourceFactsValue> for PublicationRecordsBindings {
    fn record() -> impl fragment::Shape<ResourceFactsValue, Self> {
        fragment::field("ResourceIdentityField", |row: &ResourceFactsValue| {
            row.resource_identity_field.clone()
        })
        .read_only()
        .and("ResourceWorkflowField", |row: &ResourceFactsValue| {
            row.resource_workflow_field.clone()
        })
        .and("ResourceLabelField", |row: &ResourceFactsValue| {
            row.resource_label_field.clone()
        })
        .build(
            |resource_identity_field,
             resource_workflow_field,
             resource_label_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ResourceFactsValue {
                    resource_identity_field,
                    resource_workflow_field,
                    resource_label_field,
                })
            },
        )
    }
}
pub struct GrantFacts;
impl fragment::Aspect for GrantFacts {
    const NAME: &'static str = "GrantFacts";
    type Row = GrantFactsValue;
}
impl GrantFacts {
    pub fn reference() -> AspectSelection<GrantRecords, CapabilityGrant, Self> {
        aspect(GrantRecords, CapabilityGrant, Self)
    }
}
pub struct GrantFactsValue {
    pub grant_identity_field: GrantKey,
    pub grant_action_field: String,
    pub grant_purpose_field: String,
    pub grant_status_field: String,
    pub grant_workflow_field: String,
    pub grant_not_before_field: u64,
    pub grant_not_after_field: u64,
    pub grant_delegation_limit_field: u64,
}
impl fragment::Record<GrantFactsValue> for GrantBindings {
    fn record() -> impl fragment::Shape<GrantFactsValue, Self> {
        fragment::field("GrantIdentityField", |row: &GrantFactsValue| {
            row.grant_identity_field.clone()
        })
        .read_only()
        .and("GrantActionField", |row: &GrantFactsValue| {
            row.grant_action_field.clone()
        })
        .read_only()
        .and("GrantPurposeField", |row: &GrantFactsValue| {
            row.grant_purpose_field.clone()
        })
        .read_only()
        .and("GrantStatusField", |row: &GrantFactsValue| {
            row.grant_status_field.clone()
        })
        .and("GrantWorkflowField", |row: &GrantFactsValue| {
            row.grant_workflow_field.clone()
        })
        .and("GrantNotBeforeField", |row: &GrantFactsValue| {
            row.grant_not_before_field
        })
        .and("GrantNotAfterField", |row: &GrantFactsValue| {
            row.grant_not_after_field
        })
        .and("GrantDelegationLimitField", |row: &GrantFactsValue| {
            row.grant_delegation_limit_field
        })
        .build(
            |grant_identity_field,
             grant_action_field,
             grant_purpose_field,
             grant_status_field,
             grant_workflow_field,
             grant_not_before_field,
             grant_not_after_field,
             grant_delegation_limit_field,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(GrantFactsValue {
                    grant_identity_field,
                    grant_action_field,
                    grant_purpose_field,
                    grant_status_field,
                    grant_workflow_field,
                    grant_not_before_field,
                    grant_not_after_field,
                    grant_delegation_limit_field,
                })
            },
        )
    }
}
pub struct ActionRecordFacts;
impl fragment::Aspect for ActionRecordFacts {
    const NAME: &'static str = "ActionRecordFacts";
    type Row = ActionRecordFactsValue;
}
impl ActionRecordFacts {
    pub fn reference() -> AspectSelection<PublicationRecords, ActionRecord, Self> {
        aspect(PublicationRecords, ActionRecord, Self)
    }
}
pub struct ActionRecordFactsValue {
    pub action_record_identity_field: String,
}
impl fragment::Record<ActionRecordFactsValue> for PublicationRecordsBindings {
    fn record() -> impl fragment::Shape<ActionRecordFactsValue, Self> {
        fragment::field(
            "ActionRecordIdentityField",
            |row: &ActionRecordFactsValue| row.action_record_identity_field.clone(),
        )
        .read_only()
        .build(
            |action_record_identity_field, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ActionRecordFactsValue {
                    action_record_identity_field,
                })
            },
        )
    }
}
