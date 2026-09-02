use crate::{
    WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass,
    WorthUiDslSourceSpan,
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
