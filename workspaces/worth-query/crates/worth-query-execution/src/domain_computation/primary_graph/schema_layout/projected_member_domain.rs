//! Declared finite domains join the genuine Native registry before recipes.
use super::*;
use worth_query_declaration::facade::application_schema::{
    ApplicationGraphMemberMeaning as Member, ApplicationSchemaMember as SchemaMember,
};
use worth_relational::facade::schema::{
    NativeMemberDomain, ProjectedMemberDomainHandle, ProjectedMemberDomainIdentity,
    ProjectedMemberDomainParts, SchemaId, SchemaVersionId,
};

#[derive(Debug)]
pub(in crate::domain_computation::primary_graph) struct ProjectedMemberDomainLayout {
    pub carrier: std::sync::Arc<str>,
    pub handle: ProjectedMemberDomainHandle,
}

pub(super) fn lower(
    mut registry: RelationalSchemaRegistry,
    schema: &ErasedApplicationSchemaDeclaration,
    basis: (&SchemaId, SchemaVersionId),
    entities: &BTreeMap<String, KindId>,
) -> Result<
    (
        RelationalSchemaRegistry,
        BTreeMap<String, ProjectedMemberDomainLayout>,
    ),
    WorthQueryPrimaryGraphInstallationDenial,
> {
    let mut definitions = Vec::new();
    let mut carriers = BTreeMap::new();
    for member in schema.members() {
        let SchemaMember::ProjectedMemberDomain { definition } = member else {
            continue;
        };
        let parts = definition.parts();
        definitions.push(ProjectedMemberDomainParts {
            identity: ProjectedMemberDomainIdentity {
                schema: basis.0.clone(),
                version: basis.1,
                name: parts.domain.clone(),
            },
            carrier: required_kind(entities, &parts.carrier)?,
            cases: parts.cases.clone(),
        });
        carriers.insert(parts.domain.as_str(), parts.carrier.as_str());
    }
    registry = registry
        .register_projected_member_domains(definitions)
        .map_err(|cause| invalid_member(&format!("projected Native registration: {cause:?}")))?;
    let mut layouts = BTreeMap::new();
    for (name, carrier) in carriers {
        let identity = ProjectedMemberDomainIdentity {
            schema: basis.0.clone(),
            version: basis.1,
            name: name.to_owned(),
        };
        layouts.insert(
            name.to_owned(),
            ProjectedMemberDomainLayout {
                carrier: std::sync::Arc::from(carrier),
                handle: registry
                    .projected_member_domain(&identity)
                    .expect("registered domain"),
            },
        );
    }
    Ok((registry, layouts))
}

pub(super) fn member_domain(
    member: &Member,
    entities: &BTreeMap<String, KindId>,
    domains: &BTreeMap<String, ProjectedMemberDomainLayout>,
) -> Result<NativeMemberDomain, WorthQueryPrimaryGraphInstallationDenial> {
    match member {
        Member::Entity(name) => required_kind(entities, name).map(NativeMemberDomain::Entity),
        Member::Projected(name) => domains
            .get(name)
            .map(|layout| NativeMemberDomain::Projected {
                projected: layout.handle.parts().identity.clone(),
            })
            .ok_or_else(|| invalid_member(name)),
    }
}

impl WorthQueryPrimaryGraphLayout {
    pub(in crate::domain_computation::primary_graph) fn projected_domain(
        &self,
        name: &str,
    ) -> Option<&ProjectedMemberDomainLayout> {
        self.projected_domains.get(name)
    }
}
