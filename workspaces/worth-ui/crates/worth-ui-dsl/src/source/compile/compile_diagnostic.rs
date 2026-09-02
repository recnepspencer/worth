#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiDslCompileStopClass {
    SourceIdentity,
    LanguageSyntax,
    LanguageLegality,
    SemanticNormalization,
    RustAuthoring,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorthUiDslCompileDiagnosticCode {
    InvalidModulePath,
    DuplicateModuleIdentity,
    UnknownImportTarget,
    CyclicModuleImport,
    InvalidCharacter,
    UnterminatedStringLiteral,
    UnexpectedToken,
    MissingIdentifier,
    MissingStringLiteral,
    MissingEquals,
    MissingSemicolon,
    MissingBlockStart,
    UnterminatedBlock,
    InvalidStructuralSyntax,
    DuplicateRegionSizingDeclaration,
    DuplicateRegionStateDeclaration,
    DuplicateMountPlacementDeclaration,
    DuplicateMountStateDeclaration,
    IllegalRootStructuralStatement,
    InvalidProjectionDeclaration,
    UnknownProjectionContent,
    InvalidIntentDeclaration,
    InvalidServiceDeclaration,
    InvalidRustAuthoredModulePath,
    DuplicateRustAuthoredModuleIdentity,
    InvalidAppearanceDeclaration,
    InvalidAppearanceAttachment,
    MissingAppearanceDeclaration,
    MissingAppearanceCoverage,
    MissingAppearanceCellReference,
    OverlappingAppearanceCells,
    WrongAppearanceValueKind,
    WrongAppearanceDeclarationKind,
    StaleAppearanceRoleRevision,
    CyclicAppearanceCellReference,
    AmbiguousAppearanceDeclaration,
    AppearanceCapacityDenied,
    DuplicateAppearanceDeclaration,
    InvalidBackdropDeclaration,
    MissingBackdropDeclaration,
    MissingOverlayAnchor,
    ForeignOverlaySurface,
    CyclicOverlayRelation,
    AmbiguousOverlayRelation,
    OverlayCapacityDenied,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiDslSourceSpan {
    module_id: String,
    start_byte: usize,
    end_byte: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiDslDiagnosticIdentity {
    code: WorthUiDslCompileDiagnosticCode,
    module_id: Option<String>,
    span: Option<WorthUiDslSourceSpan>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiDslCompileDiagnostic {
    identity: WorthUiDslDiagnosticIdentity,
    stop_class: WorthUiDslCompileStopClass,
    message: String,
    detail: Option<WorthUiDslCompileDiagnosticDetail>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiDslCompileDiagnosticDetail {
    AppearancePartition {
        role: crate::UiAppearanceRoleIdentity,
        aspect: crate::UiAppearanceAspect,
        expected_kind: crate::UiThemeValueKind,
        denial: crate::UiAppearanceDecisionPartitionDenial,
        source_span: Option<WorthUiDslSourceSpan>,
    },
    MissingAppearanceCoverage {
        role: crate::UiAppearanceRoleIdentity,
        aspect: crate::UiAppearanceAspect,
        expected_kind: crate::UiThemeValueKind,
        uncovered_canonical_state_cell: crate::UiAppearanceCanonicalStateCell,
        exact_repair_predicate: crate::UiAppearanceFinitePredicate,
        source_span: Option<WorthUiDslSourceSpan>,
    },
    MissingAppearanceCellReference {
        role: crate::UiAppearanceRoleIdentity,
        aspect: crate::UiAppearanceAspect,
        expected_kind: crate::UiThemeValueKind,
        referenced_cell_name: Box<str>,
        reference_origin: crate::UiAppearanceCellReferenceOrigin,
        repair: crate::UiAppearanceCellReferenceRepair,
        source_span: Option<WorthUiDslSourceSpan>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiDslCompileReport {
    diagnostics: Vec<WorthUiDslCompileDiagnostic>,
}

impl WorthUiDslCompileReport {
    pub(crate) fn new(diagnostics: Vec<WorthUiDslCompileDiagnostic>) -> Self {
        Self { diagnostics }
    }

    pub fn diagnostics(&self) -> &[WorthUiDslCompileDiagnostic] {
        &self.diagnostics
    }
}

impl WorthUiDslCompileDiagnostic {
    pub(crate) fn new(
        code: WorthUiDslCompileDiagnosticCode,
        stop_class: WorthUiDslCompileStopClass,
        message: impl Into<String>,
        module_id: Option<String>,
        span: Option<WorthUiDslSourceSpan>,
    ) -> Self {
        Self {
            identity: WorthUiDslDiagnosticIdentity {
                code,
                module_id,
                span,
            },
            stop_class,
            message: message.into(),
            detail: None,
        }
    }

    pub fn identity(&self) -> &WorthUiDslDiagnosticIdentity {
        &self.identity
    }

    pub fn stop_class(&self) -> WorthUiDslCompileStopClass {
        self.stop_class
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn detail(&self) -> Option<&WorthUiDslCompileDiagnosticDetail> {
        self.detail.as_ref()
    }

    pub(crate) fn with_detail(mut self, detail: WorthUiDslCompileDiagnosticDetail) -> Self {
        self.detail = Some(detail);
        self
    }
}

impl WorthUiDslDiagnosticIdentity {
    pub fn code(&self) -> WorthUiDslCompileDiagnosticCode {
        self.code
    }

    pub fn module_id(&self) -> Option<&str> {
        self.module_id.as_deref()
    }

    pub fn span(&self) -> Option<&WorthUiDslSourceSpan> {
        self.span.as_ref()
    }
}

impl WorthUiDslSourceSpan {
    pub(crate) fn new(module_id: impl Into<String>, start_byte: usize, end_byte: usize) -> Self {
        Self {
            module_id: module_id.into(),
            start_byte,
            end_byte,
        }
    }

    pub fn module_id(&self) -> &str {
        &self.module_id
    }

    pub fn start_byte(&self) -> usize {
        self.start_byte
    }

    pub fn end_byte(&self) -> usize {
        self.end_byte
    }
}
