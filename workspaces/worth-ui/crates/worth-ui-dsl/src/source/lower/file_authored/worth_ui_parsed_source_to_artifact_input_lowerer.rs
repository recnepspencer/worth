use std::collections::BTreeMap;

use super::worth_ui_parsed_source_declaration_lowerer::lower_parsed_source_declaration;
use crate::source::{
    WorthUiArtifactInput, WorthUiArtifactInputNormalizer, WorthUiParsedSourcePackage,
};

#[derive(Clone, Debug, Default)]
pub(crate) struct WorthUiParsedSourceToArtifactInputLowerer;

impl WorthUiParsedSourceToArtifactInputLowerer {
    pub(crate) fn lower(
        parsed_source_package: &WorthUiParsedSourcePackage,
    ) -> Result<WorthUiArtifactInput, crate::source::WorthUiDslCompileReport> {
        let mut lowered_modules = BTreeMap::new();
        let canonical_module_order = parsed_source_package.module_ids().to_vec();
        let mut diagnostics = Vec::new();

        for module_id in parsed_source_package.module_ids() {
            let parsed_module = parsed_source_package
                .module(module_id)
                .expect("parsed source package should contain every canonical module");
            let declarations =
                parsed_module
                    .declarations()
                    .iter()
                    .enumerate()
                    .filter_map(|(declaration_index, declaration)| {
                        match lower_parsed_source_declaration(declaration, declaration_index) {
                            Ok(declaration) => Some(declaration),
                            Err(diagnostic) => {
                                diagnostics.push(diagnostic);
                                None
                            }
                        }
                    })
                    .collect();
            lowered_modules.insert(module_id.clone(), declarations);
        }

        if !diagnostics.is_empty() {
            return Err(crate::source::WorthUiDslCompileReport::new(diagnostics));
        }
        let (modules, overlay_declaration_bindings) =
            crate::source::resolve_file_authored_overlay_declarations(lowered_modules)?;
        Ok(WorthUiArtifactInputNormalizer::normalize(
            WorthUiArtifactInput::new(
                modules,
                canonical_module_order,
                overlay_declaration_bindings,
            ),
        ))
    }
}
