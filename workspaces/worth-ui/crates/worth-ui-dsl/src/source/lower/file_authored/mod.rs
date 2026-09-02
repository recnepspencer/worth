mod appearance_declaration_lowerer;
mod backdrop_declaration_lowerer;
mod component_appearance_attachment;
mod worth_ui_parsed_source_declaration_lowerer;
mod worth_ui_parsed_source_to_artifact_input_lowerer;

use crate::source::{WorthUiArtifactInputNode, WorthUiArtifactInputProvenance};

pub(crate) enum WorthUiFileAuthoredLoweredDeclaration {
    Artifact(WorthUiArtifactInputNode),
    Backdrop {
        source: backdrop_declaration_lowerer::WorthUiBackdropSource,
        provenance: WorthUiArtifactInputProvenance,
    },
}

pub(crate) use backdrop_declaration_lowerer::{
    WorthUiBackdropExtentSource, WorthUiBackdropMotionSource, WorthUiBackdropPlacementSource,
    WorthUiBackdropPresenceSource, WorthUiBackdropScopeSource, WorthUiBackdropSource,
};
pub(crate) use worth_ui_parsed_source_to_artifact_input_lowerer::WorthUiParsedSourceToArtifactInputLowerer;
