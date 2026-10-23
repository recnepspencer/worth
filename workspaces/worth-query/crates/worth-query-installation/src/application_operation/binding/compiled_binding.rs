use worth_query_declaration::facade::application_operation::{
    ApplicationMutationBindingDescriptor, ApplicationMutationScopeContract,
};

pub(crate) struct WorthQueryCompiledApplicationMutationBinding {
    descriptor: ApplicationMutationBindingDescriptor,
    inverse: Option<std::sync::Arc<crate::application_aftermath::InstalledInverseBinding>>,
}

impl WorthQueryCompiledApplicationMutationBinding {
    pub(crate) fn new(
        descriptor: ApplicationMutationBindingDescriptor,
        inverse: Option<crate::application_aftermath::InstalledInverseBinding>,
    ) -> Self {
        Self {
            descriptor,
            inverse: inverse.map(std::sync::Arc::new),
        }
    }

    pub(crate) const fn descriptor(&self) -> &ApplicationMutationBindingDescriptor {
        &self.descriptor
    }

    pub(crate) fn scope(&self) -> &ApplicationMutationScopeContract {
        self.descriptor.scope()
    }

    pub(crate) fn inverse_binding(
        &self,
    ) -> Option<&crate::application_aftermath::InstalledInverseBinding> {
        self.inverse.as_deref()
    }
    pub(crate) fn retain_inverse_binding(
        &self,
    ) -> Option<std::sync::Arc<crate::application_aftermath::InstalledInverseBinding>> {
        self.inverse.clone()
    }
}
