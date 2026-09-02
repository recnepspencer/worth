use crate::{
    UiAppearanceAspect, UiAppearanceDecisionPartitionDenial, UiAppearanceRoleDeclarationDenial,
    WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass,
    WorthUiDslSourceSpan,
};

pub(super) struct AppearanceLoweringError {
    code: WorthUiDslCompileDiagnosticCode,
    message: String,
}

impl AppearanceLoweringError {
    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: WorthUiDslCompileDiagnosticCode::InvalidAppearanceDeclaration,
            message: message.into(),
        }
    }

    pub(super) fn duplicate(message: impl Into<String>) -> Self {
        Self {
            code: WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration,
            message: message.into(),
        }
    }

    pub(super) fn wrong_kind(message: impl Into<String>) -> Self {
        Self {
            code: WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind,
            message: message.into(),
        }
    }

    pub(super) fn partition(
        denial: UiAppearanceDecisionPartitionDenial,
        aspect: UiAppearanceAspect,
    ) -> Self {
        let (code, message) = match denial {
            UiAppearanceDecisionPartitionDenial::MissingCell
            | UiAppearanceDecisionPartitionDenial::MissingNamedCell => (
                WorthUiDslCompileDiagnosticCode::MissingAppearanceDeclaration,
                format!("appearance aspect {aspect:?} has incomplete finite coverage"),
            ),
            UiAppearanceDecisionPartitionDenial::AmbiguousCell
            | UiAppearanceDecisionPartitionDenial::DuplicateOtherwise => (
                WorthUiDslCompileDiagnosticCode::AmbiguousAppearanceDeclaration,
                format!("appearance aspect {aspect:?} contains ambiguous coverage"),
            ),
            UiAppearanceDecisionPartitionDenial::OverlappingCell => (
                WorthUiDslCompileDiagnosticCode::OverlappingAppearanceCells,
                format!("appearance aspect {aspect:?} contains overlapping cells"),
            ),
            UiAppearanceDecisionPartitionDenial::ResultValueKindMismatch => (
                WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind,
                format!("appearance aspect {aspect:?} contains a wrong-kind result"),
            ),
            UiAppearanceDecisionPartitionDenial::CyclicCellReference => (
                WorthUiDslCompileDiagnosticCode::CyclicAppearanceCellReference,
                format!("appearance aspect {aspect:?} contains a cyclic cell reference"),
            ),
            UiAppearanceDecisionPartitionDenial::CellCapacityExceeded => (
                WorthUiDslCompileDiagnosticCode::AppearanceCapacityDenied,
                format!("appearance aspect {aspect:?} exceeds finite cell capacity"),
            ),
            UiAppearanceDecisionPartitionDenial::DuplicateCellName => (
                WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration,
                format!("appearance aspect {aspect:?} contains a duplicate cell name"),
            ),
            other => (
                WorthUiDslCompileDiagnosticCode::InvalidAppearanceDeclaration,
                format!("appearance aspect {aspect:?} partition denied: {other:?}"),
            ),
        };
        Self { code, message }
    }

    pub(super) fn role(denial: UiAppearanceRoleDeclarationDenial) -> Self {
        let code = match denial {
            UiAppearanceRoleDeclarationDenial::Empty
            | UiAppearanceRoleDeclarationDenial::MissingRequiredAspect
            | UiAppearanceRoleDeclarationDenial::UnadmittedAspect => {
                WorthUiDslCompileDiagnosticCode::MissingAppearanceDeclaration
            }
            UiAppearanceRoleDeclarationDenial::DuplicateAspect => {
                WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration
            }
            UiAppearanceRoleDeclarationDenial::ResultValueKindMismatch => {
                WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind
            }
            UiAppearanceRoleDeclarationDenial::SlotUseCapacityExceeded => {
                WorthUiDslCompileDiagnosticCode::AppearanceCapacityDenied
            }
            UiAppearanceRoleDeclarationDenial::ApplicabilityContractMismatch => {
                WorthUiDslCompileDiagnosticCode::WrongAppearanceDeclarationKind
            }
        };
        Self {
            code,
            message: format!("appearance role admission denied: {denial:?}"),
        }
    }

    pub(super) fn into_diagnostic(
        self,
        span: &crate::WorthUiSourceSpan,
    ) -> WorthUiDslCompileDiagnostic {
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

impl From<String> for AppearanceLoweringError {
    fn from(message: String) -> Self {
        Self::invalid(message)
    }
}
