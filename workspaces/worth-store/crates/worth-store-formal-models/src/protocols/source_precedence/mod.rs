mod action;
mod selection;
#[cfg(test)]
mod tests;

pub use action::{
    ModeledSourceCandidateRole, SourcePrecedenceAction, SourcePrecedenceActionKind,
    SourcePrecedenceDenial,
};
pub use selection::{require_selectable_source, SourceAuthorityPosture};
