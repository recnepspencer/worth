//! Portable index contracts. Executable providers and retained roots belong to runtime owners.
mod descriptor;
mod invariant_source;
pub use descriptor::ApplicationManagedIndexBindingDescriptor;
pub use invariant_source::ApplicationInvariantManagedIndexSource;
mod definition;
mod probe;
mod reference;
mod refinement;
pub use refinement::{
    ApplicationManagedIndexMatchPosture, ApplicationManagedIndexRefinement,
    ApplicationManagedIndexRefinementMeaning, WorthQueryPortableManagedIndexRefinementParts,
};
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
    ManagedIndexMaintenance as ApplicationManagedIndexMaintenance,
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

/// Associates a managed-index query intent with the exact installed managed
/// index binding it probes, so Host sessions can watch and prepare the index
/// without repeating the index marker type.
pub trait ApplicationManagedIndexQueryIntent<Schema: crate::application_schema::ApplicationSchema>:
    crate::application_query::ApplicationQueryIntent<Schema>
{
    type Index: ApplicationManagedIndexBinding<Schema>;
}
