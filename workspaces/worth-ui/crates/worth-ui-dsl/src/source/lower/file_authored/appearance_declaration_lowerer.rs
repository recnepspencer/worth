use crate::source::{
    WorthUiArtifactInputAppearanceRoleNode, WorthUiArtifactInputNode,
    WorthUiArtifactInputProvenance, WorthUiDslCompileDiagnostic,
    WorthUiParsedAppearanceRoleDeclaration, WorthUiSourceTokenKind,
};
use crate::{
    UiAppearanceAspect, UiAppearanceAxisClass, UiAppearanceAxisDomain, UiAppearanceAxisPredicate,
    UiAppearanceCell, UiAppearanceCellValue, UiAppearancePartitionAuthoring,
    UiAppearanceRoleApplicability, UiAppearanceRoleDeclaration, UiAppearanceRoleIdentity,
    UiAppearanceRoleRevision, UiAppearanceStateAxis, UiDslComponentReference, UiThemeSlotIdentity,
    UiThemeValueKind,
};

#[path = "appearance_diagnostic.rs"]
mod appearance_diagnostic;
#[path = "appearance_value_lowerer.rs"]
mod appearance_value_lowerer;
use appearance_diagnostic::AppearanceLoweringError;
use appearance_value_lowerer::transparent_value;

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
    let mut cursor = Cursor::new(declaration.body().tokens());
    let mut partitions = Vec::new();
    while !cursor.eof() {
        cursor.skip(WorthUiSourceTokenKind::Semicolon);
        if cursor.eof() {
            break;
        }
        let aspect = parse_aspect(cursor.word()?)?;
        cursor.advance();
        let partition = if cursor.take_word("use") {
            simple_partition(aspect, parse_value(&mut cursor, aspect)?)?
        } else if cursor.take_word("over") {
            parse_table(&mut cursor, aspect)?
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
    value: UiAppearanceCellValue,
) -> Result<crate::UiAppearanceDecisionPartition, AppearanceLoweringError> {
    UiAppearancePartitionAuthoring::new([])
        .with_cell(UiAppearanceCell::when([]).uses(value))
        .compile(aspect)
        .map_err(|denial| AppearanceLoweringError::partition(denial, aspect))
}

fn parse_table(
    cursor: &mut Cursor<'_>,
    aspect: UiAppearanceAspect,
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
    while !cursor.take_symbol(WorthUiSourceTokenKind::RightBrace) {
        cursor.skip(WorthUiSourceTokenKind::Semicolon);
        if cursor.take_word("otherwise") {
            authoring = if cursor.take_word("same_as") {
                let name = cursor.word()?.to_owned();
                cursor.advance();
                authoring.otherwise_same_as(name)
            } else {
                cursor.expect_word("use")?;
                authoring.with_otherwise(parse_value(cursor, aspect)?)
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
            authoring = authoring.with_cell(UiAppearanceCell::new(name, predicates, value));
        }
        cursor.skip(WorthUiSourceTokenKind::Semicolon);
    }
    authoring
        .compile(aspect)
        .map_err(|denial| AppearanceLoweringError::partition(denial, aspect))
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
) -> Result<UiAppearanceCellValue, AppearanceLoweringError> {
    if cursor.take_word("same_as") {
        let parenthesized = cursor.take_symbol(WorthUiSourceTokenKind::LeftParen);
        let name = cursor.word()?.to_owned();
        cursor.advance();
        if parenthesized {
            cursor.expect_symbol(WorthUiSourceTokenKind::RightParen)?;
        }
        return Ok(UiAppearanceCellValue::same_as(name));
    }
    if cursor.take_word("token") {
        cursor.expect_symbol(WorthUiSourceTokenKind::LeftParen)?;
        let slot = UiThemeSlotIdentity::new(cursor.word()?.to_owned())
            .ok_or_else(|| "theme slot identity is invalid".to_owned())?;
        cursor.advance();
        cursor.expect_symbol(WorthUiSourceTokenKind::RightParen)?;
        return Ok(UiAppearanceCellValue::theme_slot(slot, aspect.value_kind()));
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
        Ok(UiAppearanceCellValue::literal(transparent_value(aspect)))
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

struct Cursor<'a> {
    tokens: &'a [WorthUiSourceTokenKind],
    index: usize,
}

impl<'a> Cursor<'a> {
    fn new(tokens: &'a [WorthUiSourceTokenKind]) -> Self {
        Self { tokens, index: 0 }
    }

    fn eof(&self) -> bool {
        self.index >= self.tokens.len()
    }

    fn advance(&mut self) {
        self.index += 1;
    }

    fn word(&self) -> Result<&str, String> {
        match self.tokens.get(self.index) {
            Some(WorthUiSourceTokenKind::Identifier(value)) => Ok(value),
            Some(WorthUiSourceTokenKind::KeywordToken) => Ok("token"),
            Some(WorthUiSourceTokenKind::NumberLiteral(value)) => Ok(value),
            _ => Err("appearance declaration expected a word".to_owned()),
        }
    }

    fn peek_word(&self, expected: &str) -> bool {
        self.word().is_ok_and(|word| word == expected)
    }

    fn take_word(&mut self, expected: &str) -> bool {
        if self.peek_word(expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        if self.take_word(expected) {
            Ok(())
        } else {
            Err(format!("appearance declaration expected '{expected}'"))
        }
    }

    fn take_symbol(&mut self, expected: WorthUiSourceTokenKind) -> bool {
        if self.tokens.get(self.index) == Some(&expected) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn expect_symbol(&mut self, expected: WorthUiSourceTokenKind) -> Result<(), String> {
        if self.take_symbol(expected) {
            Ok(())
        } else {
            Err("appearance declaration has malformed punctuation".to_owned())
        }
    }

    fn skip(&mut self, expected: WorthUiSourceTokenKind) {
        while self.take_symbol(expected.clone()) {}
    }
}
