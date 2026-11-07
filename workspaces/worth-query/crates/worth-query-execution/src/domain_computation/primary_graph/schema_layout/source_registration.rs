//! Registration keeps original issuers before typed Source handles are bound.
use worth_relational::facade::schema::{
    DerivedRelationDefinitionParts, DerivedRelationDenial, ProjectedMemberDomainDenial,
    ProjectedMemberDomainParts, RelationalSchemaRegistry,
};

#[derive(Clone, Copy)]
pub(in crate::domain_computation::primary_graph) struct SourceRegistration<'source> {
    original: Option<&'source RelationalSchemaRegistry>,
}
impl<'source> SourceRegistration<'source> {
    pub(super) fn new(original: Option<&'source RelationalSchemaRegistry>) -> Self {
        Self { original }
    }
    pub(super) fn derived(
        self,
        receiving: RelationalSchemaRegistry,
        definitions: Vec<DerivedRelationDefinitionParts>,
    ) -> Result<RelationalSchemaRegistry, DerivedRelationDenial> {
        match self.original {
            Some(original) => {
                receiving.register_derived_relations_from_original(definitions, original)
            }
            None => receiving.register_derived_relations(definitions),
        }
    }
    pub(super) fn domains(
        self,
        receiving: RelationalSchemaRegistry,
        definitions: Vec<ProjectedMemberDomainParts>,
    ) -> Result<RelationalSchemaRegistry, ProjectedMemberDomainDenial> {
        match self.original {
            Some(original) => {
                receiving.register_projected_member_domains_from_original(definitions, original)
            }
            None => receiving.register_projected_member_domains(definitions),
        }
    }
}
