use crate::source::{
    WorthUiArtifactInputBackdropNode, WorthUiArtifactInputNode, WorthUiArtifactInputProvenance,
    WorthUiDslCompileDiagnostic, WorthUiParsedBlockDeclaration, WorthUiSourceTokenKind,
};
use crate::{
    UiAppearanceRoleIdentity, UiAppearanceRoleRevision, UiBackdropDeclarationAuthoring,
    UiStaticBackdropExtent, UiStaticBackdropMotion, UiStaticBackdropPlacement,
    UiStaticBackdropPresence, UiStaticBackdropScope,
};

#[path = "backdrop_diagnostic.rs"]
mod backdrop_diagnostic;
use backdrop_diagnostic::BackdropLoweringError;

pub(super) fn lower_backdrop(
    declaration: &WorthUiParsedBlockDeclaration,
    declaration_index: usize,
) -> Result<WorthUiArtifactInputNode, WorthUiDslCompileDiagnostic> {
    let parsed = parse_backdrop(declaration).map_err(|error| error.into_diagnostic(declaration))?;
    let provenance = WorthUiArtifactInputProvenance::parsed_source(
        declaration.span().clone(),
        None,
        declaration_index,
    );
    Ok(WorthUiArtifactInputNode::Backdrop(
        WorthUiArtifactInputBackdropNode::new(parsed, provenance),
    ))
}

fn parse_backdrop(
    declaration: &WorthUiParsedBlockDeclaration,
) -> Result<crate::UiStaticBackdropDeclaration, BackdropLoweringError> {
    let mut cursor = Cursor::new(declaration.body().tokens());
    let mut scope = None;
    let mut extent = None;
    let mut presence = None;
    let mut motion = None;
    let mut placement = None;
    let mut role = None;
    let mut revision = 1;
    while !cursor.eof() {
        cursor.skip(WorthUiSourceTokenKind::Semicolon);
        if cursor.eof() {
            break;
        }
        let clause = cursor.word()?.to_owned();
        cursor.advance();
        match clause.as_str() {
            "scope" => set_once(&mut scope, parse_scope(&mut cursor)?, "scope")?,
            "extent" => set_once(&mut extent, parse_extent(&mut cursor)?, "extent")?,
            "presence" => set_once(&mut presence, parse_presence(&mut cursor)?, "presence")?,
            "motion" => set_once(&mut motion, parse_motion(&mut cursor)?, "motion")?,
            "place" => set_once(&mut placement, parse_placement(&mut cursor)?, "place")?,
            "appearance" => {
                if role.is_some() {
                    return Err("backdrop declares appearance more than once"
                        .to_owned()
                        .into());
                }
                cursor.expect_symbol(WorthUiSourceTokenKind::LeftBrace)?;
                cursor.expect_word("role")?;
                role = Some(cursor.word()?.to_owned());
                cursor.advance();
                if cursor.take_word("revision") {
                    revision = cursor.number()?;
                }
                cursor.expect_symbol(WorthUiSourceTokenKind::RightBrace)?;
            }
            _ => return Err(format!("unknown backdrop clause '{clause}'").into()),
        }
    }
    let role = UiAppearanceRoleIdentity::new(role.ok_or("backdrop is missing appearance role")?)
        .ok_or("backdrop role identity is invalid")?;
    let revision =
        UiAppearanceRoleRevision::new(revision).ok_or("backdrop role revision must be positive")?;
    let extent = extent.ok_or("backdrop is missing extent")?;
    let mut result = UiBackdropDeclarationAuthoring::new(
        declaration.name_text(),
        extent.surface().to_owned(),
        role,
        revision,
    )
    .map_err(|_| "backdrop identity or surface is invalid".to_owned())?
    .with_extent(extent)
    .with_scope(scope.ok_or("backdrop is missing scope")?)
    .with_presence(presence.ok_or("backdrop is missing presence")?)
    .with_placement(placement.ok_or("backdrop is missing placement")?);
    if let Some(motion) = motion {
        result = result.with_motion(motion);
    }
    result.admit().map_err(BackdropLoweringError::specification)
}

fn parse_scope(cursor: &mut Cursor<'_>) -> Result<UiStaticBackdropScope, String> {
    match cursor.word()? {
        "surface_singleton" => {
            cursor.advance();
            Ok(UiStaticBackdropScope::SurfaceSingleton)
        }
        "per_portal_instance" => {
            cursor.advance();
            let portal = cursor.word()?.to_owned();
            cursor.advance();
            Ok(UiStaticBackdropScope::PerPortalInstance(
                portal.into_boxed_str(),
            ))
        }
        value => Err(format!("unknown backdrop scope '{value}'")),
    }
}

fn parse_extent(cursor: &mut Cursor<'_>) -> Result<UiStaticBackdropExtent, String> {
    match cursor.word()? {
        "surface_viewport" => {
            cursor.advance();
            let surface = cursor.word()?.to_owned();
            cursor.advance();
            Ok(UiStaticBackdropExtent::SurfaceViewport(
                surface.into_boxed_str(),
            ))
        }
        "presented_mosaic_region" => {
            cursor.advance();
            let surface = cursor.word()?.to_owned();
            cursor.advance();
            let region = cursor.word()?.to_owned();
            cursor.advance();
            Ok(UiStaticBackdropExtent::PresentedMosaicRegion {
                surface: surface.into_boxed_str(),
                region: region.into_boxed_str(),
            })
        }
        value => Err(format!("unknown backdrop extent '{value}'")),
    }
}

fn parse_presence(cursor: &mut Cursor<'_>) -> Result<UiStaticBackdropPresence, String> {
    match cursor.word()? {
        "always" => {
            cursor.advance();
            Ok(UiStaticBackdropPresence::Always)
        }
        "while" => {
            cursor.advance();
            cursor.expect_word("portal")?;
            let portal = cursor.word()?.to_owned();
            cursor.advance();
            cursor.expect_word("presented")?;
            Ok(UiStaticBackdropPresence::WhilePortalPresented(
                portal.into_boxed_str(),
            ))
        }
        value => Err(format!("unknown backdrop presence '{value}'")),
    }
}

fn parse_motion(cursor: &mut Cursor<'_>) -> Result<UiStaticBackdropMotion, String> {
    match cursor.word()? {
        "none" => {
            cursor.advance();
            Ok(UiStaticBackdropMotion::None)
        }
        "follow" => {
            cursor.advance();
            cursor.expect_word("portal")?;
            let portal = cursor.word()?.to_owned();
            cursor.advance();
            cursor.expect_word("presentation")?;
            Ok(UiStaticBackdropMotion::PortalPresentation(
                portal.into_boxed_str(),
            ))
        }
        value => Err(format!("unknown backdrop motion '{value}'")),
    }
}

fn parse_placement(cursor: &mut Cursor<'_>) -> Result<UiStaticBackdropPlacement, String> {
    let direction = cursor.word()?.to_owned();
    cursor.advance();
    match direction.as_str() {
        "above_surface_content" => Ok(UiStaticBackdropPlacement::AboveSurfaceContent),
        "immediately_before" | "immediately_after" => {
            let before = direction == "immediately_before";
            let anchor_kind = cursor.word()?.to_owned();
            cursor.advance();
            let anchor = cursor.word()?.to_owned().into_boxed_str();
            cursor.advance();
            match (before, anchor_kind.as_str()) {
                (true, "portal") => Ok(UiStaticBackdropPlacement::ImmediatelyBeforePortal(anchor)),
                (false, "portal") => Ok(UiStaticBackdropPlacement::ImmediatelyAfterPortal(anchor)),
                (true, "backdrop") => {
                    Ok(UiStaticBackdropPlacement::ImmediatelyBeforeBackdrop(anchor))
                }
                (false, "backdrop") => {
                    Ok(UiStaticBackdropPlacement::ImmediatelyAfterBackdrop(anchor))
                }
                _ => Err("backdrop placement requires portal or backdrop anchor".to_owned()),
            }
        }
        _ => Err(format!("unknown backdrop placement '{direction}'")),
    }
}

fn set_once<T>(slot: &mut Option<T>, value: T, name: &str) -> Result<(), String> {
    if slot.is_some() {
        return Err(format!("backdrop clause '{name}' appears more than once"));
    }
    *slot = Some(value);
    Ok(())
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
        self.index == self.tokens.len()
    }
    fn advance(&mut self) {
        self.index += 1;
    }
    fn word(&self) -> Result<&str, String> {
        match self.tokens.get(self.index) {
            Some(WorthUiSourceTokenKind::Identifier(value)) => Ok(value),
            Some(WorthUiSourceTokenKind::KeywordAppearance) => Ok("appearance"),
            Some(WorthUiSourceTokenKind::KeywordBackdrop) => Ok("backdrop"),
            Some(WorthUiSourceTokenKind::KeywordToken) => Ok("token"),
            Some(WorthUiSourceTokenKind::NumberLiteral(value)) => Ok(value),
            _ => Err("backdrop declaration expected a word".to_owned()),
        }
    }
    fn take_word(&mut self, expected: &str) -> bool {
        if self.word().is_ok_and(|word| word == expected) {
            self.advance();
            true
        } else {
            false
        }
    }
    fn expect_word(&mut self, expected: &str) -> Result<(), String> {
        self.take_word(expected)
            .then_some(())
            .ok_or_else(|| format!("backdrop declaration expected '{expected}'"))
    }
    fn number(&mut self) -> Result<u64, String> {
        let value = self
            .word()?
            .parse()
            .map_err(|_| "invalid revision".to_owned())?;
        self.advance();
        Ok(value)
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
        self.take_symbol(expected)
            .then_some(())
            .ok_or_else(|| "backdrop declaration has malformed punctuation".to_owned())
    }
    fn skip(&mut self, expected: WorthUiSourceTokenKind) {
        while self.take_symbol(expected.clone()) {}
    }
}
