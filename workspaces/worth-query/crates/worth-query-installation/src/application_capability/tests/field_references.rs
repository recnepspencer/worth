//! Capability fixtures select columns from their shared domain records.
use super::*;
use worth_query_declaration::facade::application_capability::ApplicationCapabilityFieldBinding;
use worth_query_declaration::facade::application_schema::{fragment as records, *};
use worth_query_declaration::facade::authentication::*;
pub(super) struct CapabilityFields;
impl records::Fragment for CapabilityFields {
    const NAME: &'static str = "capability_fields";
    type Bindings = Bindings;
    type Members = (Grant, Resource, Principal);
}
pub(super) struct Bindings;
impl records::BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        f: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        f(work)
    }
}
impl records::Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
    const WRITABLE: bool = false;
}
impl records::Scalar<WorthQueryExternalPrincipalIdentity> for Bindings {
    type Binding = WorthQueryExternalPrincipalIdentityBinding;
    const WRITABLE: bool = false;
}
impl records::Scalar<WorthQueryPrincipalMappingStatus> for Bindings {
    type Binding = WorthQueryPrincipalMappingStatusBinding;
}
impl records::Entity for Grant {
    const NAME: &'static str = "Grant";
    type Aspects = (Facts,);
    type Relations = (ResourceRelation, ScopedRelation, Parent, MapsToPrincipal);
}
impl records::Entity for Resource {
    const NAME: &'static str = "Resource";
    type Aspects = (ResourceFacts,);
    type Relations = ();
}
impl records::Entity for Principal {
    const NAME: &'static str = "Principal";
    type Aspects = (PrincipalFacts,);
    type Relations = (PrincipalResource, Grantor, Grantee);
}
impl records::Aspect for Facts {
    const NAME: &'static str = "Facts";
    type Row = GrantState;
}
impl records::Aspect for ResourceFacts {
    const NAME: &'static str = "ResourceFacts";
    type Row = ResourceState;
}
impl records::Aspect for PrincipalFacts {
    const NAME: &'static str = "PrincipalFacts";
    type Row = PrincipalState;
}
pub(super) struct GrantState {
    action: u64,
    purpose: u64,
    field: u64,
    amount: u64,
    workflow: u64,
    status: u64,
    valid_from: u64,
    valid_through: u64,
    delegation_limit: u64,
    changed_workflow: u64,
    changed_valid_from: u64,
    mapping_identity: WorthQueryExternalPrincipalIdentity,
    mapping_status: WorthQueryPrincipalMappingStatus,
}
impl records::Record<GrantState> for Bindings {
    fn record() -> impl records::Shape<GrantState, Self> {
        records::field("Action", |row: &GrantState| row.action)
            .equality_queryable(true)
            .and("Purpose", |row: &GrantState| row.purpose)
            .equality_queryable(true)
            .and("Field", |row: &GrantState| row.field)
            .equality_queryable(true)
            .and("Amount", |row: &GrantState| row.amount)
            .equality_queryable(true)
            .and("Workflow", |row: &GrantState| row.workflow)
            .equality_queryable(true)
            .and("Status", |row: &GrantState| row.status)
            .equality_queryable(true)
            .and("ValidFrom", |row: &GrantState| row.valid_from)
            .equality_queryable(true)
            .and("ValidThrough", |row: &GrantState| row.valid_through)
            .equality_queryable(true)
            .and("DelegationLimit", |row: &GrantState| row.delegation_limit)
            .equality_queryable(true)
            .and("ChangedWorkflow", |row: &GrantState| row.changed_workflow)
            .equality_queryable(true)
            .and("ChangedValidFrom", |row: &GrantState| {
                row.changed_valid_from
            })
            .equality_queryable(true)
            .and("MappingIdentity", |row: &GrantState| {
                row.mapping_identity.clone()
            })
            .equality_queryable(true)
            .and("MappingStatus", |row: &GrantState| row.mapping_status)
            .build(
                |action,
                 purpose,
                 field,
                 amount,
                 workflow,
                 status,
                 valid_from,
                 valid_through,
                 delegation_limit,
                 changed_workflow,
                 changed_valid_from,
                 mapping_identity,
                 mapping_status,
                 _: &mut ValueDecodeAdmission<'_>| {
                    Ok::<_, std::convert::Infallible>(GrantState {
                        action,
                        purpose,
                        field,
                        amount,
                        workflow,
                        status,
                        valid_from,
                        valid_through,
                        delegation_limit,
                        changed_workflow,
                        changed_valid_from,
                        mapping_identity,
                        mapping_status,
                    })
                },
            )
    }
}
pub(super) struct ResourceState {
    resource_workflow: u64,
    changed_resource_workflow: u64,
}
impl records::Record<ResourceState> for Bindings {
    fn record() -> impl records::Shape<ResourceState, Self> {
        records::field("ResourceWorkflow", |row: &ResourceState| {
            row.resource_workflow
        })
        .equality_queryable(true)
        .and("ChangedResourceWorkflow", |row: &ResourceState| {
            row.changed_resource_workflow
        })
        .equality_queryable(true)
        .build(
            |resource_workflow, changed_resource_workflow, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ResourceState {
                    resource_workflow,
                    changed_resource_workflow,
                })
            },
        )
    }
}
pub(super) struct PrincipalState {
    principal_identity: u64,
}
impl records::Record<PrincipalState> for Bindings {
    fn record() -> impl records::Shape<PrincipalState, Self> {
        records::field("PrincipalIdentity", |row: &PrincipalState| {
            row.principal_identity
        })
        .equality_queryable(true)
        .build(|principal_identity, _: &mut ValueDecodeAdmission<'_>| {
            Ok::<_, std::convert::Infallible>(PrincipalState { principal_identity })
        })
    }
}
impl records::Relation for ResourceRelation {
    const NAME: &'static str = "ResourceRelation";
    type To = Resource;
}
impl records::Relation for ScopedRelation {
    const NAME: &'static str = "ScopedRelation";
    type To = Resource;
}
impl records::Relation for Parent {
    const NAME: &'static str = "Parent";
    type To = Grant;
}
impl records::Relation for MapsToPrincipal {
    const NAME: &'static str = "MapsToPrincipal";
    type To = Principal;
}
impl records::Relation for PrincipalResource {
    const NAME: &'static str = "PrincipalResource";
    type To = Resource;
}
impl records::Relation for Grantor {
    const NAME: &'static str = "Grantor";
    type To = Grant;
}
impl records::Relation for Grantee {
    const NAME: &'static str = "Grantee";
    type To = Grant;
}
pub(super) fn field_binding<Field: records::RecordSelection>(
    field: Field,
) -> ApplicationCapabilityFieldBinding {
    ApplicationCapabilityFieldBinding::from_reference(field).unwrap()
}
