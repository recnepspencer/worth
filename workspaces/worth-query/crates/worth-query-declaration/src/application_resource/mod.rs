//! Declared process-local resource meaning; no runtime request or provider authority.
mod authorization;
mod binding;
mod definition;
mod disclosure;
mod profile;
mod reference;
mod value_binding;

pub use authorization::{ApplicationResourceAction, ApplicationResourceAuthorization};
pub use binding::{
    ApplicationResourceBinding, ApplicationResourceBindingDescriptor, ApplicationResourceInput,
};
pub use definition::{
    ApplicationResourceDefinition, ApplicationResourceDefinitionDenial,
    ErasedApplicationResourceDefinition, WorthQueryPortableApplicationResourceParts,
};
pub use disclosure::{
    ApplicationResourceDisclosureFacet, ApplicationResourceDisclosureMeaning,
    ApplicationResourceDisclosureRef,
};
pub use profile::{ApplicationResourceLimits, ApplicationResourceRequestResponseProfile};
pub use reference::ApplicationResourceRef;
pub use value_binding::{
    ApplicationResourceValueBindingParts, ApplicationResourceValueBindingReference,
};
