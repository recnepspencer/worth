use worth_query_declaration::facade::{
    application_operation::ApplicationMutationDescription,
    application_query::ErasedApplicationQueryDefinition,
    application_schema::{ApplicationOperationProgramTarget, ApplicationSchemaMember},
};

/// Installation posture; neither variant grants permission to execute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthQueryApplicationCallablePosture {
    Declared,
    /// A typed request binding exists. Each call still requires fresh admission.
    InstalledRequestBinding,
}

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryApplicationMutationDescription<'a> {
    pub(super) declaration: &'a ApplicationMutationDescription,
    pub(super) members: &'a [ApplicationSchemaMember],
    pub(super) availability: WorthQueryApplicationCallablePosture,
}

impl<'a> WorthQueryApplicationMutationDescription<'a> {
    pub const fn declaration(&self) -> &'a ApplicationMutationDescription {
        self.declaration
    }

    pub const fn availability(&self) -> WorthQueryApplicationCallablePosture {
        self.availability
    }

    pub fn effects(&self) -> impl Iterator<Item = &'a ApplicationOperationProgramTarget> + 'a {
        let operation = self.declaration.operation();
        self.members.iter().filter_map(move |member| match member {
            ApplicationSchemaMember::OperationProgram {
                operation: name,
                target,
            } if name == operation => Some(target),
            _ => None,
        })
    }

    /// Declared authorization, preconditions, external effects, and aftermath.
    pub fn requirements(&self) -> impl Iterator<Item = &'a ApplicationSchemaMember> + 'a {
        let operation = self.declaration.operation();
        self.members.iter().filter(move |member| match member {
            ApplicationSchemaMember::OperationAbility {
                operation: name, ..
            }
            | ApplicationSchemaMember::OperationMutationPrecondition {
                operation: name, ..
            }
            | ApplicationSchemaMember::OperationExternalEffect {
                operation: name, ..
            }
            | ApplicationSchemaMember::OperationAftermath {
                operation: name, ..
            } => name == operation,
            _ => false,
        })
    }
}

#[derive(Clone, Copy, Debug)]
pub struct WorthQueryApplicationQueryDescription<'a> {
    pub(super) definition: &'a ErasedApplicationQueryDefinition,
    pub(super) availability: WorthQueryApplicationCallablePosture,
}

impl<'a> WorthQueryApplicationQueryDescription<'a> {
    /// Parameters, result shape, scope, basis support, lanes, and governance.
    pub const fn definition(&self) -> &'a ErasedApplicationQueryDefinition {
        self.definition
    }

    pub const fn availability(&self) -> WorthQueryApplicationCallablePosture {
        self.availability
    }
}

/// Field dimensions and portable value identity borrowed from the declaration.
#[derive(Clone, Copy, Debug)]
pub struct WorthQueryApplicationFieldDescription<'a> {
    pub entity: &'a str,
    pub aspect: &'a str,
    pub field: &'a str,
    pub value_type: &'a str,
    pub unit: Option<&'a str>,
    pub frame: Option<&'a str>,
}
