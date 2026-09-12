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
    writable: bool,
}

impl ApplicationMutationScopeContract {
    pub(crate) fn new(
        mode: ApplicationMutationScopeResolutionMode,
        field: ApplicationFieldBindingRecipe,
        writable: bool,
    ) -> Self {
        Self {
            mode,
            field,
            writable,
        }
    }

    pub const fn mode(&self) -> ApplicationMutationScopeResolutionMode {
        self.mode
    }

    pub fn field(&self) -> &ApplicationFieldBindingRecipe {
        &self.field
    }

    pub const fn writable(&self) -> bool {
        self.writable
    }
}
