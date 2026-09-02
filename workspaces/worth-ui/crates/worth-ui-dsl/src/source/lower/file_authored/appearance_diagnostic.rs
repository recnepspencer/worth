use crate::{
    UiAppearanceAspect, UiAppearanceCellReferenceOrigin, UiAppearanceCellReferenceRepair,
    UiAppearanceDecisionPartitionDenial, UiAppearanceRoleDeclarationDenial,
    UiAppearanceRoleIdentity, WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode,
    WorthUiDslCompileDiagnosticDetail, WorthUiDslCompileStopClass, WorthUiDslSourceSpan,
};

pub(super) struct AppearanceLoweringError {
    code: WorthUiDslCompileDiagnosticCode,
    message: String,
    detail: Option<Box<WorthUiDslCompileDiagnosticDetail>>,
}

impl AppearanceLoweringError {
    pub(super) fn invalid(message: impl Into<String>) -> Self {
        Self {
            code: WorthUiDslCompileDiagnosticCode::InvalidAppearanceDeclaration,
            message: message.into(),
            detail: None,
        }
    }

    pub(super) fn duplicate(message: impl Into<String>) -> Self {
        Self {
            code: WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration,
            message: message.into(),
            detail: None,
        }
    }

    pub(super) fn wrong_kind(message: impl Into<String>) -> Self {
        Self {
            code: WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind,
            message: message.into(),
            detail: None,
        }
    }

    pub(super) fn partition(
        denial: UiAppearanceDecisionPartitionDenial,
        role: UiAppearanceRoleIdentity,
        aspect: UiAppearanceAspect,
        aspect_span: Option<crate::WorthUiSourceSpan>,
        reference_span: Option<crate::WorthUiSourceSpan>,
    ) -> Self {
        Self::partition_inner(denial, role, aspect, aspect_span, reference_span)
    }

    pub(super) fn partition_with_references(
        denial: UiAppearanceDecisionPartitionDenial,
        role: UiAppearanceRoleIdentity,
        aspect: UiAppearanceAspect,
        aspect_span: Option<crate::WorthUiSourceSpan>,
        references: &[(
            String,
            UiAppearanceCellReferenceOrigin,
            crate::WorthUiSourceSpan,
        )],
    ) -> Self {
        let reference_span = match &denial {
            UiAppearanceDecisionPartitionDenial::MissingNamedCell {
                referenced_cell_name,
                reference_origin,
            } => references
                .iter()
                .find(|(name, origin, _)| {
                    name == referenced_cell_name.as_ref() && origin == reference_origin
                })
                .map(|(_, _, span)| span.clone()),
            _ => None,
        };
        Self::partition_inner(denial, role, aspect, aspect_span, reference_span)
    }

    fn partition_inner(
        denial: UiAppearanceDecisionPartitionDenial,
        role: UiAppearanceRoleIdentity,
        aspect: UiAppearanceAspect,
        aspect_span: Option<crate::WorthUiSourceSpan>,
        reference_span: Option<crate::WorthUiSourceSpan>,
    ) -> Self {
        let expected_kind = aspect.value_kind();
        let detail_span = reference_span
            .or(aspect_span)
            .map(source_to_diagnostic_span);
        let (code, message, detail) = match &denial {
            UiAppearanceDecisionPartitionDenial::MissingCell {
                uncovered_canonical_state_cell,
                exact_repair_predicate,
            } => (
                WorthUiDslCompileDiagnosticCode::MissingAppearanceCoverage,
                format!("appearance aspect {aspect:?} has an uncovered finite state cell"),
                WorthUiDslCompileDiagnosticDetail::MissingAppearanceCoverage {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    uncovered_canonical_state_cell: uncovered_canonical_state_cell.clone(),
                    exact_repair_predicate: exact_repair_predicate.clone(),
                    source_span: detail_span,
                },
            ),
            UiAppearanceDecisionPartitionDenial::MissingNamedCell {
                referenced_cell_name,
                reference_origin,
            } => {
                let repair = UiAppearanceCellReferenceRepair::DeclareNamedCellOrRetargetReference;
                (
                    WorthUiDslCompileDiagnosticCode::MissingAppearanceCellReference,
                    format!(
                        "appearance aspect {aspect:?} references missing cell \
                         '{referenced_cell_name}'; lawful repair: {}",
                        repair.render()
                    ),
                    WorthUiDslCompileDiagnosticDetail::MissingAppearanceCellReference {
                        role: role.clone(),
                        aspect,
                        expected_kind,
                        referenced_cell_name: referenced_cell_name.clone(),
                        reference_origin: *reference_origin,
                        repair,
                        source_span: detail_span,
                    },
                )
            }
            UiAppearanceDecisionPartitionDenial::AmbiguousCell
            | UiAppearanceDecisionPartitionDenial::DuplicateOtherwise => (
                WorthUiDslCompileDiagnosticCode::AmbiguousAppearanceDeclaration,
                format!("appearance aspect {aspect:?} contains ambiguous coverage"),
                WorthUiDslCompileDiagnosticDetail::AppearancePartition {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    denial: denial.clone(),
                    source_span: detail_span,
                },
            ),
            UiAppearanceDecisionPartitionDenial::OverlappingCell => (
                WorthUiDslCompileDiagnosticCode::OverlappingAppearanceCells,
                format!("appearance aspect {aspect:?} contains overlapping cells"),
                WorthUiDslCompileDiagnosticDetail::AppearancePartition {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    denial: denial.clone(),
                    source_span: detail_span,
                },
            ),
            UiAppearanceDecisionPartitionDenial::ResultValueKindMismatch => (
                WorthUiDslCompileDiagnosticCode::WrongAppearanceValueKind,
                format!("appearance aspect {aspect:?} contains a wrong-kind result"),
                WorthUiDslCompileDiagnosticDetail::AppearancePartition {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    denial: denial.clone(),
                    source_span: detail_span,
                },
            ),
            UiAppearanceDecisionPartitionDenial::CyclicCellReference => (
                WorthUiDslCompileDiagnosticCode::CyclicAppearanceCellReference,
                format!("appearance aspect {aspect:?} contains a cyclic cell reference"),
                WorthUiDslCompileDiagnosticDetail::AppearancePartition {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    denial: denial.clone(),
                    source_span: detail_span,
                },
            ),
            UiAppearanceDecisionPartitionDenial::CellCapacityExceeded => (
                WorthUiDslCompileDiagnosticCode::AppearanceCapacityDenied,
                format!("appearance aspect {aspect:?} exceeds finite cell capacity"),
                WorthUiDslCompileDiagnosticDetail::AppearancePartition {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    denial: denial.clone(),
                    source_span: detail_span,
                },
            ),
            UiAppearanceDecisionPartitionDenial::DuplicateCellName => (
                WorthUiDslCompileDiagnosticCode::DuplicateAppearanceDeclaration,
                format!("appearance aspect {aspect:?} contains a duplicate cell name"),
                WorthUiDslCompileDiagnosticDetail::AppearancePartition {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    denial: denial.clone(),
                    source_span: detail_span,
                },
            ),
            _ => (
                WorthUiDslCompileDiagnosticCode::InvalidAppearanceDeclaration,
                format!("appearance aspect {aspect:?} partition denied: {denial:?}"),
                WorthUiDslCompileDiagnosticDetail::AppearancePartition {
                    role: role.clone(),
                    aspect,
                    expected_kind,
                    denial: denial.clone(),
                    source_span: detail_span,
                },
            ),
        };
        Self {
            code,
            message,
            detail: Some(Box::new(detail)),
        }
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
            detail: None,
        }
    }

    pub(super) fn into_diagnostic(
        self,
        span: &crate::WorthUiSourceSpan,
    ) -> WorthUiDslCompileDiagnostic {
        let diagnostic_span = WorthUiDslSourceSpan::new(
            span.module_id().as_str(),
            span.start_byte(),
            span.end_byte(),
        );
        let diagnostic = WorthUiDslCompileDiagnostic::new(
            self.code,
            WorthUiDslCompileStopClass::LanguageLegality,
            self.message,
            Some(span.module_id().as_str().to_owned()),
            Some(diagnostic_span),
        );
        match self.detail {
            Some(detail) => diagnostic.with_detail(*detail),
            None => diagnostic,
        }
    }
}

fn source_to_diagnostic_span(span: crate::WorthUiSourceSpan) -> WorthUiDslSourceSpan {
    WorthUiDslSourceSpan::new(
        span.module_id().as_str(),
        span.start_byte(),
        span.end_byte(),
    )
}

impl From<String> for AppearanceLoweringError {
    fn from(message: String) -> Self {
        Self::invalid(message)
    }
}
