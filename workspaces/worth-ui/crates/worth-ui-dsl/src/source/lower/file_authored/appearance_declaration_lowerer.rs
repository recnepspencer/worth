use crate::source::{
    WorthUiArtifactInputAppearanceRoleNode, WorthUiArtifactInputNode,
    WorthUiArtifactInputProvenance, WorthUiDslCompileDiagnostic,
    WorthUiParsedAppearanceRoleDeclaration, WorthUiSourceSpan, WorthUiSourceTokenKind,
};
use crate::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceCell, UiAppearanceCellReferenceOrigin, UiAppearanceCellValue,
    UiAppearancePartitionAuthoring, UiAppearanceRoleApplicability, UiAppearanceRoleDeclaration,
    UiAppearanceRoleIdentity, UiAppearanceRoleRevision, UiAppearanceStateAxis,
    UiDslComponentReference, UiThemeSlotIdentity, UiThemeValueKind,
};

#[path = "appearance_diagnostic.rs"]
mod appearance_diagnostic;
#[path = "appearance_value_lowerer.rs"]
mod appearance_value_lowerer;
use appearance_diagnostic::AppearanceLoweringError;
use appearance_value_lowerer::transparent_value;

#[path = "appearance_cursor.rs"]
mod appearance_cursor;
use appearance_cursor::Cursor;

pub(super) fn lower_role(
    declaration: &WorthUiParsedAppearanceRoleDeclaration,
    declaration_index: usize,
) -> Result<WorthUiArtifactInputNode, WorthUiDslCompileDiagnostic> {
    let role =
        parse_role(declaration).map_err(|error| error.into_diagnostic(declaration.span()))?;
    Ok(WorthUiArtifactInputNode::AppearanceRole(
        WorthUiArtifactInputAppearanceRoleNode::new(
            role,
            WorthUiArtifactInputProvenance::parsed_source(
                declaration.span().clone(),
                None,
                declaration_index,
            ),
        ),
    ))
}

fn parse_role(
    declaration: &WorthUiParsedAppearanceRoleDeclaration,
) -> Result<UiAppearanceRoleDeclaration, AppearanceLoweringError> {
    let identity = UiAppearanceRoleIdentity::new(declaration.name_text().to_owned())
        .ok_or_else(|| "appearance role identity must be non-empty ASCII text".to_owned())?;
    let revision = UiAppearanceRoleRevision::new(declaration.revision())
        .ok_or_else(|| "appearance role revision must be positive".to_owned())?;
    let applicability = if declaration.applies_to() == "backdrop" {
        UiAppearanceRoleApplicability::Backdrop
    } else {
        UiAppearanceRoleApplicability::Component(
            UiDslComponentReference::new(declaration.applies_to())
                .ok_or_else(|| "appearance role component target is invalid".to_owned())?,
        )
    };
    let mut cursor = Cursor::with_spans(
        declaration.body().tokens(),
        declaration.body().token_spans(),
    );
    let mut partitions = Vec::new();
    while !cursor.eof() {
        cursor.skip(WorthUiSourceTokenKind::Semicolon);
        if cursor.eof() {
            break;
        }
        let aspect_span = cursor.current_span().cloned();
        let aspect = parse_aspect(cursor.word()?)?;
        cursor.advance();
        let partition = if cursor.take_word("use") {
            let value = parse_value(&mut cursor, aspect)?;
            simple_partition(
                aspect,
                &identity,
                value.value,
                aspect_span,
                value.reference_span,
            )?
        } else if cursor.take_word("over") {
            parse_table(&mut cursor, aspect, &identity, aspect_span)?
        } else {
            return Err(AppearanceLoweringError::invalid(format!(
                "aspect {aspect:?} requires 'use' or 'over'"
            )));
        };
        partitions.push((aspect, partition));
    }
    let contract = match applicability {
        UiAppearanceRoleApplicability::Backdrop => crate::UiAppearanceAspectContract::backdrop(),
        UiAppearanceRoleApplicability::AnyComponent
        | UiAppearanceRoleApplicability::Component(_) => {
            crate::UiAppearanceAspectContract::component(
                partitions.iter().map(|(aspect, _)| *aspect),
                [],
            )
            .map_err(|_| {
                AppearanceLoweringError::duplicate("appearance role contains duplicate aspects")
            })?
        }
    };
    UiAppearanceRoleDeclaration::admit(identity, revision, applicability, &contract, partitions)
        .map_err(AppearanceLoweringError::role)
}

fn simple_partition(
    aspect: UiAppearanceAspect,
    role: &UiAppearanceRoleIdentity,
    value: UiAppearanceCellValue,
    aspect_span: Option<WorthUiSourceSpan>,
    reference_span: Option<WorthUiSourceSpan>,
) -> Result<crate::UiAppearanceDecisionPartition, AppearanceLoweringError> {
    UiAppearancePartitionAuthoring::new([])
        .with_cell(UiAppearanceCell::when([]).uses(value))
        .compile(aspect)
        .map_err(|denial| {
            AppearanceLoweringError::partition(
                denial,
                role.clone(),
                aspect,
                aspect_span,
                reference_span,
            )
        })
}

fn parse_table(
    cursor: &mut Cursor<'_>,
    aspect: UiAppearanceAspect,
    role: &UiAppearanceRoleIdentity,
    aspect_span: Option<WorthUiSourceSpan>,
) -> Result<crate::UiAppearanceDecisionPartition, AppearanceLoweringError> {
    cursor.expect_symbol(WorthUiSourceTokenKind::LeftBracket)?;
    let mut domains = Vec::new();
    loop {
        let axis = parse_axis(cursor.word()?)?;
        cursor.advance();
        domains.push(UiAppearanceAxisDomain::complete(axis));
        if !cursor.take_symbol(WorthUiSourceTokenKind::Comma) {
            break;
        }
    }
    cursor.expect_symbol(WorthUiSourceTokenKind::RightBracket)?;
    cursor.expect_symbol(WorthUiSourceTokenKind::LeftBrace)?;
    let mut authoring = UiAppearancePartitionAuthoring::new(domains);
    let mut reference_spans = Vec::new();
    while !cursor.take_symbol(WorthUiSourceTokenKind::RightBrace) {
        cursor.skip(WorthUiSourceTokenKind::Semicolon);
        if cursor.take_word("otherwise") {
            authoring = if cursor.take_word("same_as") {
                let reference_span = cursor.current_span().cloned();
                let name = cursor.word()?.to_owned();
                cursor.advance();
                if let Some(span) = reference_span {
                    reference_spans.push((
                        name.clone(),
                        UiAppearanceCellReferenceOrigin::OtherwiseClause,
                        span,
                    ));
                }
                authoring.otherwise_same_as(name)
            } else {
                cursor.expect_word("use")?;
                let value = parse_value(cursor, aspect)?;
                if let Some(span) = value.reference_span {
                    if let UiAppearanceCellValue::SameAs(name) = &value.value {
                        reference_spans.push((
                            name.to_string(),
                            UiAppearanceCellReferenceOrigin::OtherwiseClause,
                            span,
                        ));
                    }
                }
                authoring.with_otherwise(value.value)
            };
        } else {
            let name = if cursor.take_word("cell") {
                let name = cursor.word()?.to_owned();
                cursor.advance();
                Some(name)
            } else {
                None
            };
            cursor.expect_word("when")?;
            let predicates = parse_predicates(cursor)?;
            cursor.expect_word("use")?;
            let value = parse_value(cursor, aspect)?;
            if let Some(span) = value.reference_span {
                if let UiAppearanceCellValue::SameAs(name) = &value.value {
                    reference_spans.push((
                        name.to_string(),
                        UiAppearanceCellReferenceOrigin::NamedCell,
                        span,
                    ));
                }
            }
            authoring = authoring.with_cell(UiAppearanceCell::new(name, predicates, value.value));
        }
        cursor.skip(WorthUiSourceTokenKind::Semicolon);
    }
    authoring.compile(aspect).map_err(|denial| {
        AppearanceLoweringError::partition_with_references(
            denial,
            role.clone(),
            aspect,
            aspect_span,
            &reference_spans,
        )
    })
}

fn parse_predicates(cursor: &mut Cursor<'_>) -> Result<Vec<UiAppearanceAxisPredicate>, String> {
    let mut predicates = Vec::new();
    loop {
        let axis = parse_axis(cursor.word()?)?;
        cursor.advance();
        cursor.expect_symbol(WorthUiSourceTokenKind::Equals)?;
        let class = parse_class(axis, cursor.word()?)?;
        cursor.advance();
        predicates.push(UiAppearanceAxisPredicate::exact(class));
        if !cursor.take_symbol(WorthUiSourceTokenKind::Comma) {
            break;
        }
        if cursor.peek_word("use") {
            break;
        }
    }
    Ok(predicates)
}

fn parse_value(
    cursor: &mut Cursor<'_>,
    aspect: UiAppearanceAspect,
) -> Result<ParsedValue, AppearanceLoweringError> {
    if cursor.take_word("same_as") {
        let parenthesized = cursor.take_symbol(WorthUiSourceTokenKind::LeftParen);
        let reference_span = cursor.current_span().cloned();
        let name = cursor.word()?.to_owned();
        cursor.advance();
        if parenthesized {
            cursor.expect_symbol(WorthUiSourceTokenKind::RightParen)?;
        }
        return Ok(ParsedValue {
            value: UiAppearanceCellValue::same_as(name),
            reference_span,
        });
    }
    if cursor.take_word("token") {
        cursor.expect_symbol(WorthUiSourceTokenKind::LeftParen)?;
        let slot = UiThemeSlotIdentity::new(cursor.word()?.to_owned())
            .ok_or_else(|| "theme slot identity is invalid".to_owned())?;
        cursor.advance();
        cursor.expect_symbol(WorthUiSourceTokenKind::RightParen)?;
        return Ok(ParsedValue {
            value: UiAppearanceCellValue::theme_slot(slot, aspect.value_kind()),
            reference_span: None,
        });
    }
    let literal = cursor.word()?.to_owned();
    cursor.advance();
    let expected_literal = match aspect.value_kind() {
        UiThemeValueKind::Color => "transparent-color",
        UiThemeValueKind::Opacity => "transparent-opacity",
        UiThemeValueKind::LogicalLength => "transparent-length",
        UiThemeValueKind::CornerRadii => "transparent-radius",
        UiThemeValueKind::SolidStroke => "transparent-stroke",
        UiThemeValueKind::SolidOutline => "transparent-outline",
    };
    if literal == "transparent" || literal == expected_literal {
        Ok(ParsedValue {
            value: UiAppearanceCellValue::literal(transparent_value(aspect)),
            reference_span: None,
        })
    } else if matches!(
        literal.as_str(),
        "transparent-color"
            | "transparent-opacity"
            | "transparent-length"
            | "transparent-radius"
            | "transparent-stroke"
            | "transparent-outline"
    ) {
        Err(AppearanceLoweringError::wrong_kind(format!(
            "appearance value '{literal}' has wrong kind for {aspect:?}"
        )))
    } else {
        Err(AppearanceLoweringError::invalid(format!(
            "unsupported appearance value '{literal}'"
        )))
    }
}

struct ParsedValue {
    value: UiAppearanceCellValue,
    reference_span: Option<WorthUiSourceSpan>,
}

fn parse_aspect(value: &str) -> Result<UiAppearanceAspect, String> {
    match value {
        "background" => Ok(UiAppearanceAspect::Background),
        "foreground" => Ok(UiAppearanceAspect::Foreground),
        "border" => Ok(UiAppearanceAspect::Border),
        "radius" => Ok(UiAppearanceAspect::Radius),
        "opacity" => Ok(UiAppearanceAspect::Opacity),
        "outline" => Ok(UiAppearanceAspect::Outline),
        _ => Err(format!("unknown appearance aspect '{value}'")),
    }
}

fn parse_axis(value: &str) -> Result<UiAppearanceStateAxis, String> {
    match value {
        "operability" => Ok(UiAppearanceStateAxis::Operability),
        "focus" => Ok(UiAppearanceStateAxis::Focus),
        "validation" => Ok(UiAppearanceStateAxis::Validation),
        "selection" => Ok(UiAppearanceStateAxis::Selection),
        "hover" => Ok(UiAppearanceStateAxis::Hover),
        "pressed" => Ok(UiAppearanceStateAxis::Pressed),
        _ => Err(format!("unknown appearance state axis '{value}'")),
    }
}

fn parse_class(axis: UiAppearanceStateAxis, value: &str) -> Result<UiAppearanceAxisClass, String> {
    use UiAppearanceAxisClass::*;
    let class = match (axis, value) {
        (UiAppearanceStateAxis::Operability, "ready") => OperabilityReady,
        (UiAppearanceStateAxis::Operability, "pending") => OperabilityPending,
        (UiAppearanceStateAxis::Operability, "occupied") => OperabilityOccupied,
        (UiAppearanceStateAxis::Operability, "denied") => OperabilityDenied,
        (UiAppearanceStateAxis::Operability, "unsupported") => OperabilityUnsupported,
        (UiAppearanceStateAxis::Operability, "stale") => OperabilityStale,
        (UiAppearanceStateAxis::Focus, "unfocused") => FocusUnfocused,
        (UiAppearanceStateAxis::Focus, "focused") => FocusFocused,
        (UiAppearanceStateAxis::Focus, "focus-visible") => FocusVisible,
        (UiAppearanceStateAxis::Focus, "window-inactive") => FocusedWindowInactive,
        (UiAppearanceStateAxis::Validation, "unspecified") => ValidationUnspecified,
        (UiAppearanceStateAxis::Validation, "valid") => ValidationValid,
        (UiAppearanceStateAxis::Validation, "advisory") => ValidationAdvisory,
        (UiAppearanceStateAxis::Validation, "invalid") => ValidationInvalid,
        (UiAppearanceStateAxis::Validation, "pending") => ValidationPending,
        (UiAppearanceStateAxis::Validation, "stale") => ValidationStale,
        (UiAppearanceStateAxis::Selection, "unselected") => SelectionUnselected,
        (UiAppearanceStateAxis::Selection, "selected") => SelectionSelected,
        (UiAppearanceStateAxis::Selection, "anchor") => SelectionAnchor,
        (UiAppearanceStateAxis::Selection, "cursor") => SelectionCursor,
        (UiAppearanceStateAxis::Selection, "anchor-cursor") => SelectedAnchorCursor,
        (UiAppearanceStateAxis::Hover, "outside") => HoverOutside,
        (UiAppearanceStateAxis::Hover, "hovered") => Hovered,
        (UiAppearanceStateAxis::Pressed, "idle") => PressedIdle,
        (UiAppearanceStateAxis::Pressed, "armed-inside") => PressedArmedInside,
        (UiAppearanceStateAxis::Pressed, "captured-outside") => PressedCapturedOutside,
        _ => return Err(format!("class '{value}' does not belong to axis {axis:?}")),
    };
    Ok(class)
}
