use crate::application_schema::ApplicationFieldBindingRecipe;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ApplicationMutationScopeResolutionMode {
    InputField,
    PrincipalIdentity,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApplicationMutationScopeContract {
    mode: ApplicationMutationScopeResolutionMode,
    field: ApplicationFieldBindingRecipe,
}

impl ApplicationMutationScopeContract {
    pub(crate) fn new(
        mode: ApplicationMutationScopeResolutionMode,
        field: ApplicationFieldBindingRecipe,
    ) -> Self {
        Self { mode, field }
    }

    pub const fn mode(&self) -> ApplicationMutationScopeResolutionMode {
        self.mode
    }

    pub fn field(&self) -> &ApplicationFieldBindingRecipe {
        &self.field
    }
}
