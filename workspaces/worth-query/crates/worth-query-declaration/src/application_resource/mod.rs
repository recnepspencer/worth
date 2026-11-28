//! Declared process-local resource meaning; no runtime request or provider authority.
mod authorization;
mod binding;
mod definition;
mod disclosure;
mod limits;
mod reference;
mod retry;
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
pub use limits::ApplicationResourceLimits;
pub use reference::ApplicationResourceRef;
pub use retry::ApplicationResourceFixedDelayRetry;
pub use value_binding::{
    ApplicationResourceValueBindingParts, ApplicationResourceValueBindingReference,
};
