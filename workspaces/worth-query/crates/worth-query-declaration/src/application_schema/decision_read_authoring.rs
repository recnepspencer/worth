use super::capabilities::OperationReads;
use super::references::{ApplicationEntityRef, ApplicationOperationRef, ApplicationRelationRef};
use super::{
    ApplicationOperationDecisionReadTarget, ApplicationSchemaDeclarationBuilder,
    ApplicationSchemaMember,
};

impl<Schema> ApplicationSchemaDeclarationBuilder<Schema> {
    /// Declares the exact managed Parent binding available to this operation's decision.
    /// Installation requires its scope mapping to match the mutation binding.
    pub fn operation_managed_index_read<Binding, Operation, Input>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
    ) -> Self
    where
        Schema: super::ApplicationSchema,
        Binding: crate::application_query::ApplicationQueryBinding<Schema>,
    {
        self.decision_read(
            operation.name(),
            ApplicationOperationDecisionReadTarget::ManagedIndex {
                parent: crate::application_query::ApplicationQueryBindingReference::of::<
                    Schema,
                    Binding,
                >(),
            },
        )
    }

    /// Optionally limits decision-read work. Omit this declaration for no operation work ceiling.
    /// Scope, authorization, source facts, cancellation and allocation admission still apply.
    pub fn operation_projection_work_budget<Operation, Input>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        maximum_work_units: usize,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::OperationProjectionWorkBudget {
            operation: operation.name().to_string(),
            maximum_work_units,
        })
    }

    pub fn operation_read_entity<Operation, Input, Entity>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Self
    where
        Entity: OperationReads<Operation>,
    {
        self.decision_read(
            operation.name(),
            ApplicationOperationDecisionReadTarget::Entity {
                entity: entity.name().to_string(),
            },
        )
    }

    #[inline(always)]
    pub fn operation_read_field<Operation, Input, Field: super::fragment::RecordSelection>(
        mut self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        field: Field,
    ) -> Self {
        let target = declared_column(
            &mut self.member_provenance.fragments,
            field.expected_recipe(),
        );
        self.decision_read(operation.name(), target)
    }

    pub fn operation_read_relation<Operation, Input, Relation, From, To>(
        self,
        operation: ApplicationOperationRef<Schema, Operation, Input>,
        relation: ApplicationRelationRef<Schema, Relation, From, To>,
    ) -> Self
    where
        Relation: OperationReads<Operation>,
    {
        self.decision_read(
            operation.name(),
            ApplicationOperationDecisionReadTarget::Relation {
                relation: relation.name().to_string(),
                from: relation.from().to_string(),
                to: relation.to().to_string(),
            },
        )
    }

    fn decision_read(
        self,
        operation: &str,
        target: ApplicationOperationDecisionReadTarget,
    ) -> Self {
        self.push_member(ApplicationSchemaMember::OperationDecisionRead {
            operation: operation.to_string(),
            target,
        })
    }
}

fn declared_column(
    fragments: &mut super::fragment::provenance::FragmentProvenance,
    recipe: super::ApplicationFieldBindingRecipe,
) -> ApplicationOperationDecisionReadTarget {
    let locus = recipe.locus();
    let target = ApplicationOperationDecisionReadTarget::Field {
        entity: locus.entity().to_owned(),
        aspect: locus.aspect().to_owned(),
        field: locus.field().to_owned(),
    };
    fragments.expect_field(recipe);
    target
}
