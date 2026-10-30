//! Portable index contracts. Executable providers and retained roots belong to runtime owners.
mod descriptor;
pub use descriptor::ApplicationManagedIndexBindingDescriptor;
mod definition;
mod probe;
mod reference;
mod selection;
mod selection_recipe;
pub use selection_recipe::ApplicationManagedIndexSelectionRecipe;
mod source;
pub use source::{
    ApplicationManagedIndexSourceMeaning, ApplicationManagedIndexSourceParameters,
    ApplicationManagedIndexSourceScope,
};
mod validation;

pub use definition::{
    ApplicationManagedIndexDefinition, ApplicationManagedIndexDefinitionDenial,
    ErasedApplicationManagedIndexDefinition, WorthQueryPortableManagedIndexParts,
};
pub use probe::{ApplicationManagedIndexProbeMeaning, WorthQueryPortableManagedIndexProbeParts};
pub use reference::{ApplicationManagedIndexBinding, ApplicationManagedIndexRef};
pub use selection::{
    ApplicationQueryManagedIndexMeaning, WorthQueryPortableQueryManagedIndexParts,
};
