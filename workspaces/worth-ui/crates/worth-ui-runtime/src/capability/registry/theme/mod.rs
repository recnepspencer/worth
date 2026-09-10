mod definition;
mod frozen_entry;
mod identity;
mod registration;
mod registry;
mod slot_catalog;

pub use definition::{UiThemeDefinition, UiThemeDefinitionDenial};
pub use frozen_entry::{
    FrozenAppearanceThemeCapabilities, FrozenAppearanceThemeCapabilitiesDenial,
};
pub use identity::UiThemeDefinitionIdentity;
pub(crate) use registration::AppearanceThemeAcceptedRegistrationProof;
pub(crate) use registry::ThemeRegistry;
pub use slot_catalog::{
    UiThemeSlotCatalog, UiThemeSlotCatalogDenial, UiThemeSlotDeclaration, UiThemeSlotDisclosure,
    UiThemeSlotSuccessorCompatibility,
};
