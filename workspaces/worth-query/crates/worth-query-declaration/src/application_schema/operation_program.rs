use super::capabilities::{
    OperationCreates, OperationDeletes, OperationEmits, OperationLinks, OperationUnlinks,
};
use super::references::{
    ApplicationEffectRef, ApplicationEntityRef, ApplicationOperationRef, ApplicationRelationRef,
};
use super::{
    ApplicationOperationProgramTarget, ApplicationSchemaDeclarationBuilder, ApplicationSchemaMember,
};
mod aspect_removal;
mod relation_initialization;

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    pub fn operation_create<Operation, Input, Entity>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self
    where
        Entity: OperationCreates<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Create {
                entity: entity.name().to_string(),
            },
        )
    }

    pub fn operation_delete<Operation, Input, Entity>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self
    where
        Entity: OperationDeletes<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Delete {
                entity: entity.name().to_string(),
            },
        )
    }

    #[inline(always)]
    pub fn operation_write<Operation, Input, Field: super::fragment::RecordSelection>(
        mut self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: Field,
    ) -> Self {
        let target = declared_column(
            &mut self.member_provenance.fragments,
            field.expected_recipe(),
        );
        self.program(operation.name(), target)
    }

    pub fn operation_link<Operation, Input, Relation, From, To>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationLinks<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Link {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
        )
    }

    pub fn operation_unlink<Operation, Input, Relation, From, To>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationUnlinks<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Unlink {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
        )
    }

    pub fn operation_emit<Operation, Input, Effect, Payload>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        effect: ApplicationEffectRef<Schema, Effect, Payload>,
    ) -> Self
    where
        Effect: OperationEmits<Operation>,
    {
        self.program(
            operation.name(),
            ApplicationOperationProgramTarget::Emit {
                effect: effect.name().to_string(),
            },
        )
    }

    fn program(self, operation: &str, target: ApplicationOperationProgramTarget) -> Self {
        self.push_member(ApplicationSchemaMember::OperationProgram {
            operation: operation.to_string(),
            target,
        })
    }
}

fn declared_column(
    fragments: &mut super::fragment::provenance::FragmentProvenance,
    recipe: super::ApplicationFieldBindingRecipe,
) -> ApplicationOperationProgramTarget {
    let locus = recipe.locus();
    let target = ApplicationOperationProgramTarget::Write {
        entity: locus.entity().to_owned(),
        aspect: locus.aspect().to_owned(),
        field: locus.field().to_owned(),
    };
    fragments.expect_field(recipe);
    target
}
