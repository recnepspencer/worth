use worth_query_declaration::facade::application_operation::{
    ApplicationCandidateRequirements, ApplicationMutationBindingDescriptor,
    ApplicationMutationScopeContract,
};

pub(crate) struct WorthQueryCompiledApplicationMutationBinding {
    descriptor: ApplicationMutationBindingDescriptor,
}

impl WorthQueryCompiledApplicationMutationBinding {
    pub(crate) const fn new(descriptor: ApplicationMutationBindingDescriptor) -> Self {
        Self { descriptor }
    }

    pub(crate) const fn descriptor(&self) -> &ApplicationMutationBindingDescriptor {
        &self.descriptor
    }

    pub(crate) fn scope(&self) -> &ApplicationMutationScopeContract {
        self.descriptor.scope()
    }

    pub(crate) const fn candidates(&self) -> ApplicationCandidateRequirements {
        self.descriptor.candidates()
    }
}
