use super::ApplicationRelationAspectDefinition;
use crate::application_schema::{ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn relation_aspect<Relation, From, To, Aspect>(
        mut self,
        definition: ApplicationRelationAspectDefinition<Schema, Relation, From, To, Aspect>,
    ) -> Self {
        for recipe in definition.recipes {
            self.member_provenance.register_field_binding(recipe);
        }
        self.push_member(ApplicationSchemaMember::RelationAspect {
            definition: definition.meaning,
        })
    }
}
