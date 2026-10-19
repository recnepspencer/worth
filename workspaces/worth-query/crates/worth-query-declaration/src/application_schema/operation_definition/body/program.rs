//! Ordered, portable native values for one operation's scope-bound body.

mod creation;
mod query_context;

pub use super::created_field::ApplicationOperationBodyCreatedField;
pub use super::scope_field::ApplicationOperationBodyScopeField;

/// Schema-owned operation meaning, not an executable permit or caller payload.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct ApplicationOperationBodyProgram {
    pub(super) explicit_scope: Option<String>,
    pub(super) scope_fields: Vec<ApplicationOperationBodyScopeField>,
    pub(super) creation: Option<ApplicationOperationBodyCreation>,
    pub(super) requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    pub(super) outputs: Vec<super::ApplicationOperationBodyOutput>,
    pub(super) succession: Option<super::ApplicationOperationBodySuccession>,
    pub(super) external_emission: Option<super::ApplicationOperationBodyExternalEmission>,
    pub(super) values: Vec<super::ApplicationOperationBodyValueProgram>,
    pub(super) retirements: Vec<super::ApplicationOperationBodyOwnedSelectionRetirement>,
    pub(super) path_input_expectations: Vec<super::ApplicationOperationBodyPathInputExpectation>,
    pub(super) path_value_assignments: Vec<super::ApplicationOperationBodyPathValueAssignment>,
    pub(super) reference_inputs: Vec<super::ApplicationOperationBodyReferenceInput>,
}

pub use creation::ApplicationOperationBodyCreation;

impl ApplicationOperationBodyProgram {
    /// Authority-free decoded description; fresh schema readmission validates it.
    pub fn from_untrusted_scope_fields(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
    ) -> Self {
        Self {
            explicit_scope: None,
            scope_fields,
            creation: None,
            requirements: Vec::new(),
            outputs: Vec::new(),
            succession: None,
            external_emission: None,
            values: Vec::new(),
            retirements: Vec::new(),
            path_input_expectations: Vec::new(),
            path_value_assignments: Vec::new(),
            reference_inputs: Vec::new(),
        }
    }
    pub fn from_untrusted_with_creation(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
    ) -> Self {
        Self {
            explicit_scope: None,
            scope_fields,
            creation,
            requirements: Vec::new(),
            outputs: Vec::new(),
            succession: None,
            external_emission: None,
            values: Vec::new(),
            retirements: Vec::new(),
            path_input_expectations: Vec::new(),
            path_value_assignments: Vec::new(),
            reference_inputs: Vec::new(),
        }
    }
    pub fn from_untrusted_with_requirements(
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
        requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    ) -> Self {
        Self {
            explicit_scope: None,
            scope_fields,
            creation,
            requirements,
            outputs: Vec::new(),
            succession: None,
            external_emission: None,
            values: Vec::new(),
            retirements: Vec::new(),
            path_input_expectations: Vec::new(),
            path_value_assignments: Vec::new(),
            reference_inputs: Vec::new(),
        }
    }
    /// Authority-free scope-bound description. Fresh readmission checks the scope
    /// and its exact decision membership. Redundant field-derived scope normalizes.
    pub fn from_untrusted_in_scope(
        scope: String,
        scope_fields: Vec<ApplicationOperationBodyScopeField>,
        creation: Option<ApplicationOperationBodyCreation>,
        requirements: Vec<super::ApplicationOperationBodyRequirementProgram>,
    ) -> Self {
        let explicit_scope = (scope_fields.first().map(|step| step.field().entity())
            != Some(scope.as_str()))
        .then_some(scope);
        Self {
            explicit_scope,
            scope_fields,
            creation,
            requirements,
            outputs: Vec::new(),
            succession: None,
            external_emission: None,
            values: Vec::new(),
            retirements: Vec::new(),
            path_input_expectations: Vec::new(),
            path_value_assignments: Vec::new(),
            reference_inputs: Vec::new(),
        }
    }
    pub fn explicit_scope_entity(&self) -> Option<&str> {
        self.explicit_scope.as_deref()
    }
    pub fn reference_inputs(&self) -> &[super::ApplicationOperationBodyReferenceInput] {
        &self.reference_inputs
    }
    pub fn with_untrusted_reference_inputs(
        mut self,
        references: Vec<super::ApplicationOperationBodyReferenceInput>,
    ) -> Self {
        self.reference_inputs = references;
        self
    }
    pub fn path_value_assignments(&self) -> &[super::ApplicationOperationBodyPathValueAssignment] {
        &self.path_value_assignments
    }
    /// Authority-free carriage requiring fresh declaration admission and installation.
    pub fn with_untrusted_path_value_assignments(
        mut self,
        assignments: Vec<super::ApplicationOperationBodyPathValueAssignment>,
    ) -> Self {
        self.path_value_assignments = assignments;
        self
    }
    pub fn path_input_expectations(
        &self,
    ) -> &[super::ApplicationOperationBodyPathInputExpectation] {
        &self.path_input_expectations
    }
    /// Authority-free carriage; fresh admission rechecks path, field and exact input binding.
    pub fn with_untrusted_path_input_expectations(
        mut self,
        guards: Vec<super::ApplicationOperationBodyPathInputExpectation>,
    ) -> Self {
        self.path_input_expectations = guards;
        self
    }
    pub fn retirements(&self) -> &[super::ApplicationOperationBodyOwnedSelectionRetirement] {
        &self.retirements
    }
    /// Descriptive transported rules require fresh schema admission and installation.
    pub fn with_untrusted_retirements(
        mut self,
        retirements: Vec<super::ApplicationOperationBodyOwnedSelectionRetirement>,
    ) -> Self {
        self.retirements = retirements;
        self
    }
    /// Untrusted output descriptions must be admitted with their mutation contract.
    pub fn with_untrusted_outputs(
        mut self,
        outputs: Vec<super::ApplicationOperationBodyOutput>,
    ) -> Self {
        self.outputs = outputs;
        self
    }
    pub fn outputs(&self) -> &[super::ApplicationOperationBodyOutput] {
        &self.outputs
    }
    pub fn succession(&self) -> Option<&super::ApplicationOperationBodySuccession> {
        self.succession.as_ref()
    }
    pub fn with_untrusted_succession(
        mut self,
        succession: Option<super::ApplicationOperationBodySuccession>,
    ) -> Self {
        self.succession = succession;
        self
    }
    /// Creation changes no existing scope field; guards and requirements may read it.
    pub fn is_creation_only(&self) -> bool {
        self.creation.is_some()
            && self.reference_inputs.is_empty()
            && self.path_value_assignments.is_empty()
            && self
                .scope_fields
                .iter()
                .all(|step| step.write_target().is_none())
    }
    pub fn requirements(&self) -> &[super::ApplicationOperationBodyRequirementProgram] {
        &self.requirements
    }
    pub fn external_emission(&self) -> Option<&super::ApplicationOperationBodyExternalEmission> {
        self.external_emission.as_ref()
    }
    pub fn with_untrusted_external_emission(
        mut self,
        emission: Option<super::ApplicationOperationBodyExternalEmission>,
    ) -> Self {
        self.external_emission = emission;
        self
    }
    /// All source consumers, including optional effect guards. Only `requirements`
    /// are mandatory operation findings; an emission guard may lawfully be unmet.
    pub fn observation_requirements(
        &self,
    ) -> impl Iterator<Item = &super::ApplicationOperationBodyRequirementProgram> {
        self.requirements.iter().chain(
            self.external_emission
                .iter()
                .filter_map(|emission| emission.condition.as_ref()),
        )
    }
    pub fn has_optional_inputs(&self) -> bool {
        !self.reference_inputs.is_empty()
            || self
                .scope_fields
                .iter()
                .any(ApplicationOperationBodyScopeField::input_is_optional)
            || self
                .creation
                .iter()
                .flat_map(|creation| creation.fields())
                .any(ApplicationOperationBodyCreatedField::input_is_optional)
    }
    pub fn scope_fields(&self) -> &[ApplicationOperationBodyScopeField] {
        &self.scope_fields
    }
    pub fn creation(&self) -> Option<&ApplicationOperationBodyCreation> {
        self.creation.as_ref()
    }

    pub fn scope_entity(&self) -> Option<&str> {
        self.explicit_scope
            .as_deref()
            .or_else(|| self.scope_fields.first().map(|step| step.field().entity()))
    }
}
