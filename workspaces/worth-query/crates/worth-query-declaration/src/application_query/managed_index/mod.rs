//! Portable index contracts. Executable providers and retained roots belong to runtime owners.
mod descriptor;
mod invariant_source;
pub use descriptor::ApplicationManagedIndexBindingDescriptor;
pub use invariant_source::ApplicationInvariantManagedIndexSource;
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
pub use worth_foundational::facade::{
    ManagedIndexColdSourcePosture as ApplicationManagedIndexColdSource,
    ManagedIndexMembership as ApplicationManagedIndexMembership,
    ManagedIndexSourceStructure as ApplicationManagedIndexSourceStructure,
};

pub use definition::{
    ApplicationManagedIndexDefinition, ApplicationManagedIndexDefinitionDenial,
    ErasedApplicationManagedIndexDefinition, WorthQueryPortableManagedIndexParts,
};
pub use probe::{ApplicationManagedIndexProbeMeaning, WorthQueryPortableManagedIndexProbeParts};
pub use reference::{ApplicationManagedIndexBinding, ApplicationManagedIndexRef};
pub use selection::{
    ApplicationQueryManagedIndexMeaning, WorthQueryPortableQueryManagedIndexParts,
};
