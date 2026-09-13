mod completion;
mod formatting;
mod geometry;
pub(in crate::mounting::projection) use geometry::require_allocation;
mod profile;
mod qualification;
mod qualification_cache;
mod qualified;
mod seed;

pub(super) use completion::{
    complete_node_semantic_text, complete_semantic_text_replacement,
    UiMountedSemanticTextCompletionContext,
};
pub(super) use formatting::{lower_semantic_text_formatting, UiMountedSemanticTextFormattingSeed};
pub(in crate::mounting::projection) use profile::current_text_profile_generation;
pub(super) use qualification_cache::UiMountedTextQualificationCache;
pub(super) use qualified::{
    rebind_semantic_text, UiMountedQualifiedSemanticText, UiMountedSemanticTextRepaintInput,
};
pub(super) use seed::{
    lower_semantic_text_seed, UiMountedCollectionTextKey, UiMountedCollectionTextSource,
    UiMountedSemanticTextSeed, UiMountedSemanticTextSeedContent,
    UiMountedSemanticTextSeedTransition,
};
