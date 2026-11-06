//! One record model owns field lowering and reusable scalar bindings.
use super::*;
use worth_query_decl::facade::application_schema::{
    fragment::{self, *},
    *,
};
pub struct DeploymentRecords;
impl fragment::Fragment for DeploymentRecords {
    const NAME: &'static str = "deployment_records";
    type Bindings = DeploymentRecordsBindings;
    type Members = (ProgramDeploymentScope, ProgramDeploymentGrant);
}
pub struct DeploymentRecordsBindings;
impl fragment::BindingSet for DeploymentRecordsBindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl fragment::Scalar<String> for DeploymentRecordsBindings {
    type Binding = StringApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = true;
}
impl fragment::Scalar<u64> for DeploymentRecordsBindings {
    type Binding = U64ApplicationValueBinding;
    const EQUALITY_QUERYABLE: bool = true;
    const WRITABLE: bool = false;
}
pub struct ProgramDeploymentScope;
impl fragment::Entity for ProgramDeploymentScope {
    const NAME: &'static str = "ProgramDeploymentScope";
    type Aspects = (ProgramDeploymentScopeFacts,);
    type Relations = ();
}
impl ProgramDeploymentScope {
    pub fn reference() -> ApplicationEntityRef<ConsumerSchema, Self> {
        entity(DeploymentRecords, Self)
    }
}
pub struct ProgramDeploymentGrant;
impl fragment::Entity for ProgramDeploymentGrant {
    const NAME: &'static str = "ProgramDeploymentGrant";
    type Aspects = (ProgramDeploymentGrantFacts,);
    type Relations = (ProgramDeploymentGrantResource, ProgramDeploymentGrantParent);
}
impl ProgramDeploymentGrant {
    pub fn reference() -> ApplicationEntityRef<ConsumerSchema, Self> {
        entity(DeploymentRecords, Self)
    }
}
pub struct ProgramDeploymentScopeFacts;
impl fragment::Aspect for ProgramDeploymentScopeFacts {
    const NAME: &'static str = "ProgramDeploymentScopeFacts";
    type Row = ProgramDeploymentScopeFactsValue;
}
impl ProgramDeploymentScopeFacts {
    pub fn reference() -> AspectSelection<DeploymentRecords, ProgramDeploymentScope, Self> {
        aspect(DeploymentRecords, ProgramDeploymentScope, Self)
    }
}
pub struct ProgramDeploymentScopeFactsValue {
    pub program_deployment_source_revision: String,
}
impl fragment::Record<ProgramDeploymentScopeFactsValue> for DeploymentRecordsBindings {
    fn record() -> impl fragment::Shape<ProgramDeploymentScopeFactsValue, Self> {
        fragment::field(
            "ProgramDeploymentSourceRevision",
            |row: &ProgramDeploymentScopeFactsValue| row.program_deployment_source_revision.clone(),
        )
        .unique()
        .read_only()
        .build(
            |program_deployment_source_revision, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProgramDeploymentScopeFactsValue {
                    program_deployment_source_revision,
                })
            },
        )
    }
}
pub struct ProgramDeploymentGrantFacts;
impl fragment::Aspect for ProgramDeploymentGrantFacts {
    const NAME: &'static str = "ProgramDeploymentGrantFacts";
    type Row = ProgramDeploymentGrantFactsValue;
}
impl ProgramDeploymentGrantFacts {
    pub fn reference() -> AspectSelection<DeploymentRecords, ProgramDeploymentGrant, Self> {
        aspect(DeploymentRecords, ProgramDeploymentGrant, Self)
    }
}
pub struct ProgramDeploymentGrantFactsValue {
    pub program_deployment_grant_identity: String,
    pub program_deployment_grant_source_revision: String,
    pub program_deployment_grant_action: String,
    pub program_deployment_grant_purpose: String,
    pub program_deployment_grant_status: String,
    pub program_deployment_grant_not_before: u64,
    pub program_deployment_grant_not_after: u64,
    pub program_deployment_grant_depth: u64,
}
impl fragment::Record<ProgramDeploymentGrantFactsValue> for DeploymentRecordsBindings {
    fn record() -> impl fragment::Shape<ProgramDeploymentGrantFactsValue, Self> {
        fragment::field(
            "ProgramDeploymentGrantIdentity",
            |row: &ProgramDeploymentGrantFactsValue| row.program_deployment_grant_identity.clone(),
        )
        .unique()
        .read_only()
        .and(
            "ProgramDeploymentGrantSourceRevision",
            |row: &ProgramDeploymentGrantFactsValue| {
                row.program_deployment_grant_source_revision.clone()
            },
        )
        .read_only()
        .and(
            "ProgramDeploymentGrantAction",
            |row: &ProgramDeploymentGrantFactsValue| row.program_deployment_grant_action.clone(),
        )
        .read_only()
        .and(
            "ProgramDeploymentGrantPurpose",
            |row: &ProgramDeploymentGrantFactsValue| row.program_deployment_grant_purpose.clone(),
        )
        .read_only()
        .and(
            "ProgramDeploymentGrantStatus",
            |row: &ProgramDeploymentGrantFactsValue| row.program_deployment_grant_status.clone(),
        )
        .and(
            "ProgramDeploymentGrantNotBefore",
            |row: &ProgramDeploymentGrantFactsValue| row.program_deployment_grant_not_before,
        )
        .and(
            "ProgramDeploymentGrantNotAfter",
            |row: &ProgramDeploymentGrantFactsValue| row.program_deployment_grant_not_after,
        )
        .and(
            "ProgramDeploymentGrantDepth",
            |row: &ProgramDeploymentGrantFactsValue| row.program_deployment_grant_depth,
        )
        .build(
            |program_deployment_grant_identity,
             program_deployment_grant_source_revision,
             program_deployment_grant_action,
             program_deployment_grant_purpose,
             program_deployment_grant_status,
             program_deployment_grant_not_before,
             program_deployment_grant_not_after,
             program_deployment_grant_depth,
             _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(ProgramDeploymentGrantFactsValue {
                    program_deployment_grant_identity,
                    program_deployment_grant_source_revision,
                    program_deployment_grant_action,
                    program_deployment_grant_purpose,
                    program_deployment_grant_status,
                    program_deployment_grant_not_before,
                    program_deployment_grant_not_after,
                    program_deployment_grant_depth,
                })
            },
        )
    }
}
pub type ProgramDeploymentSourceRevision =
    FieldSelection<DeploymentRecords, ProgramDeploymentScope, ProgramDeploymentScopeFacts, String>;
pub type ProgramDeploymentGrantIdentity =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, String>;
pub type ProgramDeploymentGrantSourceRevision =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, String>;
pub type ProgramDeploymentGrantAction =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, String>;
pub type ProgramDeploymentGrantPurpose =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, String>;
pub type ProgramDeploymentGrantStatus =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, String>;
pub type ProgramDeploymentGrantNotBefore =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, u64>;
pub type ProgramDeploymentGrantNotAfter =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, u64>;
pub type ProgramDeploymentGrantDepth =
    FieldSelection<DeploymentRecords, ProgramDeploymentGrant, ProgramDeploymentGrantFacts, u64>;
pub struct ProgramDeploymentGrantResource;
impl fragment::Relation for ProgramDeploymentGrantResource {
    const NAME: &'static str = "ProgramDeploymentGrantResource";
    type To = ProgramDeploymentScope;
}
impl ProgramDeploymentGrantResource {
    pub fn reference(
    ) -> ApplicationRelationRef<ConsumerSchema, Self, ProgramDeploymentGrant, ProgramDeploymentScope>
    {
        relation(DeploymentRecords, ProgramDeploymentGrant, Self)
    }
}
pub struct ProgramDeploymentGrantParent;
impl fragment::Relation for ProgramDeploymentGrantParent {
    const NAME: &'static str = "ProgramDeploymentGrantParent";
    type To = ProgramDeploymentGrant;
}
impl ProgramDeploymentGrantParent {
    pub fn reference(
    ) -> ApplicationRelationRef<ConsumerSchema, Self, ProgramDeploymentGrant, ProgramDeploymentGrant>
    {
        relation(DeploymentRecords, ProgramDeploymentGrant, Self)
    }
}
pub struct ProgramDeploymentGrantGrantor;
impl fragment::Relation for ProgramDeploymentGrantGrantor {
    const NAME: &'static str = "ProgramDeploymentGrantGrantor";
    type To = ProgramDeploymentGrant;
}
impl ProgramDeploymentGrantGrantor {
    pub fn reference(
    ) -> ApplicationRelationRef<ConsumerSchema, Self, Principal, ProgramDeploymentGrant> {
        {
            let native: ApplicationRelationRef<
                ConsumerSchema,
                Self,
                ConsumerPrincipal,
                ProgramDeploymentGrant,
            > = relation(ConsumerPrincipalRecords, ConsumerPrincipal, Self);
            ApplicationRelationRef::from_schema_identifiers(
                <Self as fragment::Relation>::NAME,
                <Principal as fragment::Entity>::NAME,
                <ProgramDeploymentGrant as fragment::Entity>::NAME,
                native.integrity(),
            )
        }
    }
}
pub struct ProgramDeploymentGrantGrantee;
impl fragment::Relation for ProgramDeploymentGrantGrantee {
    const NAME: &'static str = "ProgramDeploymentGrantGrantee";
    type To = ProgramDeploymentGrant;
}
impl ProgramDeploymentGrantGrantee {
    pub fn reference(
    ) -> ApplicationRelationRef<ConsumerSchema, Self, Principal, ProgramDeploymentGrant> {
        {
            let native: ApplicationRelationRef<
                ConsumerSchema,
                Self,
                ConsumerPrincipal,
                ProgramDeploymentGrant,
            > = relation(ConsumerPrincipalRecords, ConsumerPrincipal, Self);
            ApplicationRelationRef::from_schema_identifiers(
                <Self as fragment::Relation>::NAME,
                <Principal as fragment::Entity>::NAME,
                <ProgramDeploymentGrant as fragment::Entity>::NAME,
                native.integrity(),
            )
        }
    }
}

/// The final application owns the principal's complete relation roster.
pub struct ConsumerPrincipalRecords;
impl fragment::Fragment for ConsumerPrincipalRecords {
    const NAME: &'static str = "principal_records";
    type Bindings = worth_query_topology_entry::TopologyRecordsBindings;
    type Members = (ConsumerPrincipal,);
}
pub struct ConsumerPrincipal;
impl fragment::Entity for ConsumerPrincipal {
    const NAME: &'static str = <Principal as fragment::Entity>::NAME;
    type Aspects = <Principal as fragment::Entity>::Aspects;
    type Relations = (
        <Principal as fragment::Entity>::Relations,
        ProgramDeploymentGrantGrantor,
        ProgramDeploymentGrantGrantee,
    );
}
