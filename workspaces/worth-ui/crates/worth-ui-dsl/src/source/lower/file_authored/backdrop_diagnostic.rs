use crate::{
    UiBackdropDeclarationAuthoringDenial, WorthUiDslCompileDiagnostic,
    WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass, WorthUiDslSourceSpan,
};

pub(super) struct BackdropLoweringError {
    code: WorthUiDslCompileDiagnosticCode,
    message: String,
}

impl BackdropLoweringError {
    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: WorthUiDslCompileDiagnosticCode::InvalidBackdropDeclaration,
            message: message.into(),
        }
    }

    pub(super) fn specification(denial: UiBackdropDeclarationAuthoringDenial) -> Self {
        let code = match denial {
            UiBackdropDeclarationAuthoringDenial::MissingScope
            | UiBackdropDeclarationAuthoringDenial::MissingExtent
            | UiBackdropDeclarationAuthoringDenial::MissingPresence
            | UiBackdropDeclarationAuthoringDenial::MissingPlacement => {
                WorthUiDslCompileDiagnosticCode::MissingBackdropDeclaration
            }
            UiBackdropDeclarationAuthoringDenial::ForeignSurfaceExtent => {
                WorthUiDslCompileDiagnosticCode::ForeignOverlaySurface
            }
            UiBackdropDeclarationAuthoringDenial::EmptyIdentity
            | UiBackdropDeclarationAuthoringDenial::EmptySurface
            | UiBackdropDeclarationAuthoringDenial::DuplicateClause
            | UiBackdropDeclarationAuthoringDenial::PerPortalScopeMismatch
            | UiBackdropDeclarationAuthoringDenial::ForeignPortalPlacement => {
                WorthUiDslCompileDiagnosticCode::InvalidBackdropDeclaration
            }
        };
        Self {
            code,
            message: format!("backdrop admission denied: {denial:?}"),
        }
    }

    pub(super) fn into_diagnostic(
        self,
        declaration: &crate::source::WorthUiParsedBlockDeclaration,
    ) -> WorthUiDslCompileDiagnostic {
        let span = declaration.span();
        WorthUiDslCompileDiagnostic::new(
            self.code,
            WorthUiDslCompileStopClass::LanguageLegality,
            self.message,
            Some(span.module_id().as_str().to_owned()),
            Some(WorthUiDslSourceSpan::new(
                span.module_id().as_str(),
                span.start_byte(),
                span.end_byte(),
            )),
        )
    }
}

impl From<String> for BackdropLoweringError {
    fn from(message: String) -> Self {
        Self::invalid(message)
    }
}

impl<'a> From<&'a str> for BackdropLoweringError {
    fn from(message: &'a str) -> Self {
        Self::invalid(message)
    }
}
