//! Cross-context relation declarations for the shared-access court schema.

use worth_query_decl::facade::application_schema::{
    fragment::{relation, Relation},
    ApplicationRelationCrossContextPolicy, ApplicationRelationRef,
};

use super::schema::{
    ExternalMapping, Principal, Project, ProjectGrant, SharedModel, SharedProjectSchema,
};

pub struct MappingTarget;
impl Relation for MappingTarget {
    const NAME: &'static str = "MappingTarget";
    type To = Principal;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl MappingTarget {
    pub fn reference(
    ) -> ApplicationRelationRef<SharedProjectSchema, Self, ExternalMapping, Principal> {
        relation(SharedModel, ExternalMapping, Self)
    }
}

pub struct Grantor;
impl Relation for Grantor {
    const NAME: &'static str = "Grantor";
    type To = ProjectGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl Grantor {
    pub fn reference() -> ApplicationRelationRef<SharedProjectSchema, Self, Principal, ProjectGrant>
    {
        relation(SharedModel, Principal, Self)
    }
}

pub struct Grantee;
impl Relation for Grantee {
    const NAME: &'static str = "Grantee";
    type To = ProjectGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl Grantee {
    pub fn reference() -> ApplicationRelationRef<SharedProjectSchema, Self, Principal, ProjectGrant>
    {
        relation(SharedModel, Principal, Self)
    }
}

pub struct GrantResource;
impl Relation for GrantResource {
    const NAME: &'static str = "GrantResource";
    type To = Project;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl GrantResource {
    pub fn reference() -> ApplicationRelationRef<SharedProjectSchema, Self, ProjectGrant, Project> {
        relation(SharedModel, ProjectGrant, Self)
    }
}

pub struct GrantParent;
impl Relation for GrantParent {
    const NAME: &'static str = "GrantParent";
    type To = ProjectGrant;
    const CROSS_CONTEXT: ApplicationRelationCrossContextPolicy =
        ApplicationRelationCrossContextPolicy::AllowExplicit;
}
impl GrantParent {
    pub fn reference(
    ) -> ApplicationRelationRef<SharedProjectSchema, Self, ProjectGrant, ProjectGrant> {
        relation(SharedModel, ProjectGrant, Self)
    }
}
