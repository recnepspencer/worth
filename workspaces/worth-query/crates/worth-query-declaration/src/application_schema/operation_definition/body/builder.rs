//! Body authoring derives each target's binding from its typed field.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use crate::application_schema::{
    ApplicationEntityRef, ApplicationFieldBindingLocus, ApplicationFieldRef, ApplicationFieldUnit,
    ApplicationOperationRef, ApplicationScalarValueBinding, DeclaredApplicationFieldValue,
    OperationReads, OperationWrites, OptionalApplicationFieldValue, WritableCapability,
};

use super::{
    ApplicationOperationBody, ApplicationOperationBodyAuthoringDenial,
    ApplicationOperationBodyProgram, ApplicationOperationBodyScopeField,
};

pub struct ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    scope: Option<String>,
    pub(super) steps: Vec<ApplicationOperationBodyScopeField>,
    pub(super) values: Vec<super::ApplicationOperationBodyValueProgram>,
    pub(super) retirements: Vec<super::ApplicationOperationBodyOwnedSelectionRetirement>,
    pub(super) path_input_expectations: Vec<super::ApplicationOperationBodyPathInputExpectation>,
    pub(super) path_value_assignments: Vec<super::ApplicationOperationBodyPathValueAssignment>,
    requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    pub(super) outputs: Vec<super::ApplicationOperationBodyOutput>,
    pub(super) succession: Option<super::ApplicationOperationBodySuccession>,
    pub(super) external_emission: Option<super::ApplicationOperationBodyExternalEmission>,
    pub(super) external_emission_recipe:
        Option<super::ApplicationOperationBodyExternalEffectRecipe>,
    marker: PhantomData<fn(Input) -> (Schema, Operation)>,
}

impl<Schema, Operation, Input> ApplicationOperationRef<Schema, Operation, Input> {
    /// Begins scope-bound source expectations and assignments. Membership is
    /// derived when the completed body is attached to this operation definition.
    pub fn body(self) -> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
        ApplicationOperationBodyBuilder {
            scope: None,
            steps: Vec::new(),
            values: Vec::new(),
            retirements: Vec::new(),
            path_input_expectations: Vec::new(),
            path_value_assignments: Vec::new(),
            requirements: Vec::new(),
            outputs: Vec::new(),
            succession: None,
            external_emission: None,
            external_emission_recipe: None,
            marker: PhantomData,
        }
    }
}

impl<Schema, Operation, Input> ApplicationOperationBodyBuilder<Schema, Operation, Input> {
    /// Selects the existing admitted scope without requiring a write to it.
    /// This declaration grants no scope authority; installation binds it exactly.
    pub fn in_scope<Entity>(
        mut self,
        entity: ApplicationEntityRef<Schema, Entity>,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Entity: OperationReads<Operation>,
    {
        self.in_named_scope(entity.name())?;
        Ok(self)
    }
    pub(super) fn in_named_scope(
        &mut self,
        entity: &str,
    ) -> Result<(), ApplicationOperationBodyAuthoringDenial> {
        if self.scope.as_deref().is_some_and(|scope| scope != entity) {
            return Err(ApplicationOperationBodyAuthoringDenial::MultipleScopeEntities);
        }
        self.scope = Some(entity.to_owned());
        Ok(())
    }

    pub fn expect<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        Field::Binding::validate(&value)
            .map_err(crate::application_schema::ApplicationValueEncodeDenial::from)?;
        self.steps.push(ApplicationOperationBodyScopeField::Expect {
            field: ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            binding: Field::Binding::IDENTITY,
            value: Field::Binding::encode(&value)?,
        });
        Ok(self)
    }

    /// Requires a present scope field equal to one original canonical input member.
    /// Both required and optional fields are supported; absence never satisfies equality.
    /// The installed scalar binding validates the captured member before comparison.
    pub fn expect_input<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationReads<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        if member.is_empty() || member.trim() != member {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember);
        }
        self.steps
            .push(ApplicationOperationBodyScopeField::ExpectInput {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                member: member.to_owned(),
            });
        Ok(self)
    }

    pub fn assign<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        value: Value,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        Field::Binding::validate(&value)
            .map_err(crate::application_schema::ApplicationValueEncodeDenial::from)?;
        self.steps.push(ApplicationOperationBodyScopeField::Assign {
            field: ApplicationFieldBindingLocus::new(field.entity(), field.aspect(), field.field()),
            binding: Field::Binding::IDENTITY,
            value: Field::Binding::encode(&value)?,
        });
        Ok(self)
    }

    pub fn finish(
        self,
    ) -> Result<
        ApplicationOperationBody<Schema, Operation, Input>,
        ApplicationOperationBodyAuthoringDenial,
    > {
        self.finish_with_creation(None)
    }

    pub(super) fn finish_with_creation(
        self,
        creation: Option<super::ApplicationOperationBodyCreation>,
    ) -> Result<
        ApplicationOperationBody<Schema, Operation, Input>,
        ApplicationOperationBodyAuthoringDenial,
    > {
        let scope = self
            .scope
            .as_deref()
            .or_else(|| self.steps.first().map(|step| step.field().entity()));
        let Some(scope) = scope else {
            return Err(ApplicationOperationBodyAuthoringDenial::Empty);
        };
        let mut writes = BTreeSet::new();
        for step in &self.steps {
            if step.field().entity() != scope {
                return Err(ApplicationOperationBodyAuthoringDenial::MultipleScopeEntities);
            }
            if step.write_target().is_some() && !writes.insert(step.field().clone()) {
                return Err(ApplicationOperationBodyAuthoringDenial::DuplicateAssignment);
            }
        }
        if self
            .requirements
            .iter()
            .flat_map(|requirement| requirement.operands())
            .any(|operand| operand.source().scope_entity() != scope)
        {
            return Err(ApplicationOperationBodyAuthoringDenial::MultipleScopeEntities);
        }
        if writes.is_empty()
            && self.path_value_assignments.is_empty()
            && creation.is_none()
            && self.outputs.is_empty()
            && self.external_emission.is_none()
        {
            return Err(ApplicationOperationBodyAuthoringDenial::Empty);
        }
        let program = ApplicationOperationBodyProgram::from_untrusted_in_scope(
            scope.to_owned(),
            self.steps,
            creation,
            self.requirements,
        )
        .with_untrusted_outputs(self.outputs)
        .with_untrusted_succession(self.succession)
        .with_untrusted_external_emission(self.external_emission);
        let program = program
            .with_untrusted_values(self.values)
            .with_untrusted_retirements(self.retirements);
        let program = program
            .with_untrusted_path_input_expectations(self.path_input_expectations)
            .with_untrusted_path_value_assignments(self.path_value_assignments);
        program.validate_path_bindings()?;
        if !program.has_supported_retiring_replacement()
            || !program.has_supported_creation_reuse()
            || !program.has_supported_child_creations()
        {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidCreationLink);
        }
        let mut paths = BTreeSet::new();
        if program.retirements().iter().any(|rule| {
            !rule.fits_body(&program)
                || !paths.insert((&rule.owners_relation, &rule.selection_relation))
        }) {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidSelectionRetirement);
        }
        if program
            .observation_operands()
            .any(|operand| Some(operand.source().scope_entity()) != program.scope_entity())
        {
            return Err(ApplicationOperationBodyAuthoringDenial::MultipleScopeEntities);
        }
        if !super::succession::valid(&program) {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidWorkflowSuccession);
        }
        if program
            .query_contexts()
            .any(|(_, query)| Some(query.scope_entity()) != program.scope_entity())
        {
            return Err(ApplicationOperationBodyAuthoringDenial::MultipleScopeEntities);
        }
        Ok(ApplicationOperationBody {
            program,
            external_emission_recipe: self.external_emission_recipe,
            marker: PhantomData,
        })
    }

    /// Attaches one complete precondition; its expressions own guidance and admission.
    pub fn requirement(
        mut self,
        requirement: super::ApplicationOperationBodyRequirement<Schema, Operation>,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial> {
        if self
            .requirements
            .iter()
            .any(|existing| existing.identity() == requirement.program.identity())
        {
            return Err(
                super::ApplicationOperationBodyRequirementDenial::DuplicateRequirement.into(),
            );
        }
        self.requirements.push(requirement.program);
        Ok(self)
    }

    /// Assigns a scalar from the original canonical input's serialized member.
    /// Selection and destination validation are native; no getter is installed.
    pub fn assign_input<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + DeclaredApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        if member.is_empty() || member.trim() != member {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember);
        }
        self.steps
            .push(ApplicationOperationBodyScopeField::AssignInput {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                member: member.to_owned(),
            });
        Ok(self)
    }

    /// Sets one optional scalar from a flat member of the canonical input.
    /// The member must be present, including when its value is empty or zero.
    pub fn set_optional_input<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        if member.is_empty() || member.trim() != member {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember);
        }
        self.steps
            .push(ApplicationOperationBodyScopeField::SetOptionalInput {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                member: member.to_owned(),
            });
        Ok(self)
    }

    /// Replaces an optional scalar from a present Option-valued input member.
    /// Some sets (including empty text); None clears. A missing member denies.
    pub fn assign_optional_input<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
        member: &str,
    ) -> Result<Self, ApplicationOperationBodyAuthoringDenial>
    where
        Field: OperationWrites<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        if member.is_empty() || member.trim() != member {
            return Err(ApplicationOperationBodyAuthoringDenial::InvalidInputMember);
        }
        self.steps
            .push(ApplicationOperationBodyScopeField::AssignOptionalInput {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
                member: member.to_owned(),
            });
        Ok(self)
    }

    /// Clears one optional scalar; absence is not an empty value.
    pub fn clear_optional<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: OperationWrites<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Write: WritableCapability,
        Unit: ApplicationFieldUnit,
    {
        self.steps
            .push(ApplicationOperationBodyScopeField::ClearOptional {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
            });
        self
    }

    /// Requires a present optional scalar when this operation is admitted.
    /// An absent draft may still be saved by another operation.
    pub fn require_present<Entity, Aspect, Field, Value, Write, Equality, Unit>(
        mut self,
        field: ApplicationFieldRef<Schema, Entity, Aspect, Field, Value, Write, Equality, Unit>,
    ) -> Self
    where
        Field: OperationReads<Operation> + OptionalApplicationFieldValue<Value = Value>,
        Unit: ApplicationFieldUnit,
    {
        self.steps
            .push(ApplicationOperationBodyScopeField::RequirePresent {
                field: ApplicationFieldBindingLocus::new(
                    field.entity(),
                    field.aspect(),
                    field.field(),
                ),
                binding: Field::Binding::IDENTITY,
                scalar_family: Field::Binding::SCALAR_FAMILY,
            });
        self
    }
}
