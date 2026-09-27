//! Lowers `layout <container> { ... }`: the container's grid, then the grid
//! each `width from <min> [to <max>] { ... }` variant selects instead.
//!
//! This owns the spelling only. Whether the tracks, cells, and intervals make
//! a layout is judged where the declaration lowers into Mosaic meaning.
use super::token_cursor::{Cursor, UiDeclarationWords};
use crate::source::{
    WorthUiArtifactInputLayoutNode, WorthUiArtifactInputNode, WorthUiArtifactInputProvenance,
    WorthUiDslCompileDiagnostic, WorthUiDslCompileDiagnosticCode, WorthUiDslCompileStopClass,
    WorthUiDslSourceSpan, WorthUiParsedBlockDeclaration, WorthUiSourceSpan, WorthUiSourceTokenKind,
};
use crate::{
    UiDslComponentReference, UiLayoutCell, UiLayoutDeclaration, UiLayoutGrid, UiLayoutTrack,
    UiLayoutWidthInterval,
};

type Lowered<T> = Result<T, WorthUiDslCompileDiagnostic>;

pub(super) fn lower_layout(
    declaration: &WorthUiParsedBlockDeclaration,
    declaration_index: usize,
) -> Lowered<WorthUiArtifactInputNode> {
    let container = UiDslComponentReference::new(declaration.name_text()).ok_or_else(|| {
        diagnostic(
            declaration.span(),
            format!(
                "layout container `{}` is not a component identity",
                declaration.name_text()
            ),
        )
    })?;
    let mut cursor = Cursor::new(
        declaration.body(),
        declaration.span(),
        UiDeclarationWords::Identifiers,
        "layout",
    );
    let mut fallback = GridStatements::default();
    let mut variants = Vec::new();
    while !cursor.eof() {
        if cursor.take_word("width") {
            variants.push(variant(&mut cursor)?);
        } else {
            fallback.statement(&mut cursor)?;
        }
    }
    let layout = variants.into_iter().fold(
        UiLayoutDeclaration::new(container, fallback.finish(declaration.span())?),
        |layout, (interval, grid)| layout.with_variant(interval, grid),
    );
    Ok(WorthUiArtifactInputNode::Layout(
        WorthUiArtifactInputLayoutNode::new(
            layout,
            WorthUiArtifactInputProvenance::parsed_source(
                declaration.span().clone(),
                None,
                declaration_index,
            ),
        ),
    ))
}

fn variant(cursor: &mut Cursor<'_>) -> Lowered<(UiLayoutWidthInterval, UiLayoutGrid)> {
    let start = cursor.span();
    expect_word(
        cursor,
        "from",
        "a width variant reads `width from <min> [to <max>] { ... }`",
    )?;
    let min = number(cursor)?;
    let interval = if cursor.take_word("to") {
        UiLayoutWidthInterval::between(min, number(cursor)?)
    } else {
        UiLayoutWidthInterval::at_least(min)
    };
    expect(
        cursor,
        WorthUiSourceTokenKind::LeftBrace,
        "a width variant opens its grid with '{'",
    )?;
    let mut grid = GridStatements::default();
    while !cursor.take_symbol(WorthUiSourceTokenKind::RightBrace) {
        if cursor.eof() {
            return Err(refuse(cursor, "a width variant closes its grid with '}'"));
        }
        grid.statement(cursor)?;
    }
    Ok((interval, grid.finish(start)?))
}

/// One grid's statements as they are read: each clause but `member` at most
/// once, and the tracks of both axes required.
#[derive(Default)]
struct GridStatements {
    columns: Option<Vec<UiLayoutTrack>>,
    rows: Option<Vec<UiLayoutTrack>>,
    gap: Option<(u16, u16)>,
    padding: Option<(u16, u16)>,
    members: Vec<(UiDslComponentReference, UiLayoutCell)>,
}

impl GridStatements {
    fn statement(&mut self, cursor: &mut Cursor<'_>) -> Lowered<()> {
        let span = cursor.span();
        let word = word(cursor)?.to_owned();
        cursor.advance();
        match word.as_str() {
            "columns" => once(&mut self.columns, tracks(cursor)?, "columns", span)?,
            "rows" => once(&mut self.rows, tracks(cursor)?, "rows", span)?,
            "gap" => once(&mut self.gap, pair(cursor)?, "gap", span)?,
            "padding" => once(&mut self.padding, pair(cursor)?, "padding", span)?,
            "member" => self.members.push(member(cursor)?),
            "width" => {
                return Err(diagnostic(
                    span,
                    "a width variant stands at the top of a layout, not inside another variant",
                ))
            }
            other => {
                return Err(diagnostic(
                    span,
                    format!(
                        "layout has no `{other}` statement; use columns, rows, gap, padding, \
                         member, or width"
                    ),
                ))
            }
        }
        expect(
            cursor,
            WorthUiSourceTokenKind::Semicolon,
            "a layout statement ends with ';'",
        )
    }

    fn finish(self, span: &WorthUiSourceSpan) -> Lowered<UiLayoutGrid> {
        let required = |tracks: Option<Vec<UiLayoutTrack>>, axis: &str| {
            tracks.ok_or_else(|| diagnostic(span, format!("a layout grid declares its {axis}")))
        };
        let columns = required(self.columns, "columns")?;
        let rows = required(self.rows, "rows")?;
        let (column_gap, row_gap) = self.gap.unwrap_or_default();
        let (inline, block) = self.padding.unwrap_or_default();
        Ok(self.members.into_iter().fold(
            UiLayoutGrid::new(columns, rows)
                .with_gaps(column_gap, row_gap)
                .with_padding(inline, block),
            |grid, (component, cell)| grid.with_member(component, cell),
        ))
    }
}

fn once<T>(slot: &mut Option<T>, value: T, clause: &str, span: &WorthUiSourceSpan) -> Lowered<()> {
    if slot.replace(value).is_some() {
        return Err(diagnostic(
            span,
            format!("a layout grid declares `{clause}` once"),
        ));
    }
    Ok(())
}

fn tracks(cursor: &mut Cursor<'_>) -> Lowered<Vec<UiLayoutTrack>> {
    let mut tracks = vec![track(cursor)?];
    while cursor.take_symbol(WorthUiSourceTokenKind::Comma) {
        tracks.push(track(cursor)?);
    }
    Ok(tracks)
}

fn track(cursor: &mut Cursor<'_>) -> Lowered<UiLayoutTrack> {
    if cursor.take_word("fixed") {
        return Ok(UiLayoutTrack::Fixed {
            extent: number(cursor)?,
        });
    }
    if cursor.take_word("flex") {
        let weight = number(cursor)?;
        let min = if cursor.take_word("min") {
            number(cursor)?
        } else {
            0
        };
        let max = if cursor.take_word("max") {
            Some(number(cursor)?)
        } else {
            None
        };
        return Ok(UiLayoutTrack::Flexible { weight, min, max });
    }
    Err(refuse(
        cursor,
        "a track reads `fixed <extent>` or `flex <weight> [min <extent>] [max <extent>]`",
    ))
}

fn pair(cursor: &mut Cursor<'_>) -> Lowered<(u16, u16)> {
    Ok((number(cursor)?, number(cursor)?))
}

fn member(cursor: &mut Cursor<'_>) -> Lowered<(UiDslComponentReference, UiLayoutCell)> {
    const SHAPE: &str =
        "a layout member reads `member <component> at <column> <row> [span <columns> <rows>]`";
    let span = cursor.span();
    let component =
        UiDslComponentReference::new(word(cursor)?).ok_or_else(|| diagnostic(span, SHAPE))?;
    cursor.advance();
    expect_word(cursor, "at", SHAPE)?;
    let (column, row) = pair(cursor)?;
    let (column_span, row_span) = if cursor.take_word("span") {
        pair(cursor)?
    } else {
        (1, 1)
    };
    Ok((
        component,
        UiLayoutCell::spanning(column, row, column_span, row_span),
    ))
}

fn word<'a>(cursor: &Cursor<'a>) -> Lowered<&'a str> {
    cursor
        .word()
        .map_err(|message| diagnostic(cursor.span(), message))
}

fn expect_word(cursor: &mut Cursor<'_>, expected: &str, shape: &str) -> Lowered<()> {
    if cursor.take_word(expected) {
        Ok(())
    } else {
        Err(refuse(cursor, shape))
    }
}

fn expect(cursor: &mut Cursor<'_>, expected: WorthUiSourceTokenKind, shape: &str) -> Lowered<()> {
    if cursor.take_symbol(expected) {
        Ok(())
    } else {
        Err(refuse(cursor, shape))
    }
}

/// A whole number of logical points.
fn number(cursor: &mut Cursor<'_>) -> Lowered<u16> {
    cursor.number().ok_or_else(|| {
        refuse(
            cursor,
            "layout expected a whole number of logical points from 0 to 65535",
        )
    })
}

fn refuse(cursor: &Cursor<'_>, message: &str) -> WorthUiDslCompileDiagnostic {
    diagnostic(cursor.span(), message)
}

fn diagnostic(span: &WorthUiSourceSpan, message: impl Into<String>) -> WorthUiDslCompileDiagnostic {
    WorthUiDslCompileDiagnostic::new(
        WorthUiDslCompileDiagnosticCode::InvalidLayoutDeclaration,
        WorthUiDslCompileStopClass::LanguageLegality,
        message,
        Some(span.module_id().as_str().to_owned()),
        Some(WorthUiDslSourceSpan::new(
            span.module_id().as_str(),
            span.start_byte(),
            span.end_byte(),
        )),
    )
}
