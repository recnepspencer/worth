//! One typed pair projects program requirements and existing installation work.
mod configuration;
mod contract;
mod definition;
mod definition_macro;
pub use configuration::install;
pub use configuration::{
    WorthQueryFeatureConfiguration, WorthQueryFeaturePairConfiguration,
    WorthQueryFeatureRoleRuntime, WorthQueryFeatureRuntime,
};
pub use contract::{required_producers, validate_pair};
pub use contract::{WorthQueryRequiredFeatureContract, WorthQueryRequiredFeaturePair};
pub use definition::WorthQueryFeatureDefinition;
