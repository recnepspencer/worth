use crate::source::{
    WorthUiAuthoredMode, WorthUiAuthoredSourceInput, WorthUiDslCompileDiagnostic,
    WorthUiDslCompileDiagnosticCode, WorthUiDslCompileReport, WorthUiDslCompileStopClass,
    WorthUiDslSourceSpan, WorthUiParseDiagnosticCode, WorthUiParsedSourceToArtifactInputLowerer,
    WorthUiRustAuthoredArtifactInput, WorthUiRustAuthoredInputLoweringDenial,
    WorthUiRustAuthoredToArtifactInputLowerer, WorthUiSealedSemanticPackage,
    WorthUiSourcePackageDiagnosticCode, WorthUiSourcePackageLoader, WorthUiSourceParser,
};

#[derive(Clone, Debug, Default)]
pub struct WorthUiDslCompiler;

impl WorthUiDslCompiler {
    pub fn compile_source(
        input: WorthUiAuthoredSourceInput,
    ) -> Result<WorthUiSealedSemanticPackage, WorthUiDslCompileReport> {
        let (workspace_root, modules) = input.into_parts();
        let mut loader = WorthUiSourcePackageLoader::from_workspace_root(workspace_root);
        for module in modules {
            let (relative_path, source_text) = module.into_parts();
            loader = loader.register_module_with_source(relative_path, source_text);
        }
        let source_package = loader.compile().map_err(source_package_report)?;
        let parsed = WorthUiSourceParser::parse_package(&source_package).map_err(parse_report)?;
        WorthUiSealedSemanticPackage::seal(
            WorthUiParsedSourceToArtifactInputLowerer::lower(&parsed)?,
            WorthUiAuthoredMode::File,
        )
    }

    pub fn compile_rust_authored(
        input: &WorthUiRustAuthoredArtifactInput,
    ) -> Result<WorthUiSealedSemanticPackage, WorthUiDslCompileReport> {
        WorthUiRustAuthoredToArtifactInputLowerer::try_lower(input)
            .map_err(rust_authored_report)
            .and_then(|input| WorthUiSealedSemanticPackage::seal(input, WorthUiAuthoredMode::Rust))
    }
}

fn source_package_report(
    report: crate::source::WorthUiSourcePackageReport,
) -> WorthUiDslCompileReport {
    WorthUiDslCompileReport::new(
        report
            .into_diagnostics()
            .into_iter()
            .map(|diagnostic| {
                let code = match diagnostic.code() {
                    WorthUiSourcePackageDiagnosticCode::InvalidModulePath => {
                        WorthUiDslCompileDiagnosticCode::InvalidModulePath
                    }
                    WorthUiSourcePackageDiagnosticCode::DuplicateModuleIdentity => {
                        WorthUiDslCompileDiagnosticCode::DuplicateModuleIdentity
                    }
                    WorthUiSourcePackageDiagnosticCode::UnknownImportTarget => {
                        WorthUiDslCompileDiagnosticCode::UnknownImportTarget
                    }
                    WorthUiSourcePackageDiagnosticCode::CyclicModuleImport => {
                        WorthUiDslCompileDiagnosticCode::CyclicModuleImport
                    }
                };
                WorthUiDslCompileDiagnostic::new(
                    code,
                    WorthUiDslCompileStopClass::SourceIdentity,
                    diagnostic.message(),
                    diagnostic.module_id_text().map(str::to_owned),
                    None,
                )
            })
            .collect(),
    )
}

fn parse_report(report: crate::source::WorthUiParseReport) -> WorthUiDslCompileReport {
    WorthUiDslCompileReport::new(
        report
            .into_diagnostics()
            .into_iter()
            .map(|diagnostic| {
                let code = match diagnostic.code() {
                    WorthUiParseDiagnosticCode::InvalidCharacter => {
                        WorthUiDslCompileDiagnosticCode::InvalidCharacter
                    }
                    WorthUiParseDiagnosticCode::UnterminatedStringLiteral => {
                        WorthUiDslCompileDiagnosticCode::UnterminatedStringLiteral
                    }
                    WorthUiParseDiagnosticCode::UnexpectedToken => {
                        WorthUiDslCompileDiagnosticCode::UnexpectedToken
                    }
                    WorthUiParseDiagnosticCode::MissingIdentifier => {
                        WorthUiDslCompileDiagnosticCode::MissingIdentifier
                    }
                    WorthUiParseDiagnosticCode::MissingStringLiteral => {
                        WorthUiDslCompileDiagnosticCode::MissingStringLiteral
                    }
                    WorthUiParseDiagnosticCode::MissingEquals => {
                        WorthUiDslCompileDiagnosticCode::MissingEquals
                    }
                    WorthUiParseDiagnosticCode::MissingSemicolon => {
                        WorthUiDslCompileDiagnosticCode::MissingSemicolon
                    }
                    WorthUiParseDiagnosticCode::MissingBlockStart => {
                        WorthUiDslCompileDiagnosticCode::MissingBlockStart
                    }
                    WorthUiParseDiagnosticCode::UnterminatedBlock => {
                        WorthUiDslCompileDiagnosticCode::UnterminatedBlock
                    }
                };
                let span = diagnostic.span();
                WorthUiDslCompileDiagnostic::new(
                    code,
                    WorthUiDslCompileStopClass::LanguageSyntax,
                    diagnostic.message(),
                    Some(span.module_id().as_str().to_owned()),
                    Some(WorthUiDslSourceSpan::new(
                        span.module_id().as_str(),
                        span.start_byte(),
                        span.end_byte(),
                    )),
                )
            })
            .collect(),
    )
}

fn rust_authored_report(denial: WorthUiRustAuthoredInputLoweringDenial) -> WorthUiDslCompileReport {
    let code = match denial {
        WorthUiRustAuthoredInputLoweringDenial::InvalidModulePath => {
            WorthUiDslCompileDiagnosticCode::InvalidRustAuthoredModulePath
        }
        WorthUiRustAuthoredInputLoweringDenial::DuplicateModuleIdentity => {
            WorthUiDslCompileDiagnosticCode::DuplicateRustAuthoredModuleIdentity
        }
        WorthUiRustAuthoredInputLoweringDenial::DuplicateOverlayDeclaration => {
            WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration
        }
        WorthUiRustAuthoredInputLoweringDenial::OverlayIdentityCapacity => {
            WorthUiDslCompileDiagnosticCode::OverlayCapacityDenied
        }
    };
    WorthUiDslCompileReport::new(vec![WorthUiDslCompileDiagnostic::new(
        code,
        WorthUiDslCompileStopClass::RustAuthoring,
        "Rust-authored input could not be normalized into a sealed DSL package",
        None,
        None,
    )])
}
