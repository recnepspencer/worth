use crate::source::WorthUiArtifactInputBodyAtom;

use super::cursor::StructuralCursor;
use super::{
    WorthUiAuthoredProjectionContent, WorthUiAuthoredRegion, WorthUiAuthoredStructuralBody,
    WorthUiStructuralLanguageDiagnosticCode, WorthUiStructuralParseFailure,
};

pub(crate) struct WorthUiStructuralBodyParser;

pub(super) struct StructuralParser<'a> {
    pub(super) cursor: StructuralCursor<'a>,
}

struct ParsedRootDeclarations {
    regions: Vec<WorthUiAuthoredRegion>,
    projection_contents: Vec<WorthUiAuthoredProjectionContent>,
    interaction_routes: Vec<crate::WorthUiIntentInteractionRoute>,
}

impl WorthUiStructuralBodyParser {
    pub(crate) fn parse(
        body_atoms: &[WorthUiArtifactInputBodyAtom],
    ) -> Result<WorthUiAuthoredStructuralBody, WorthUiStructuralParseFailure> {
        let mut parser = StructuralParser::new(body_atoms);
        let root = parser.parse_root_declarations()?;
        if !parser.cursor.is_eof() {
            return Err(parser.cursor.syntax_failure("trailing_tokens"));
        }
        Ok(WorthUiAuthoredStructuralBody::new(
            root.regions,
            root.projection_contents,
            root.interaction_routes,
        ))
    }
}

impl<'a> StructuralParser<'a> {
    fn new(atoms: &'a [WorthUiArtifactInputBodyAtom]) -> Self {
        Self {
            cursor: StructuralCursor::new(atoms),
        }
    }

    fn parse_root_declarations(
        &mut self,
    ) -> Result<ParsedRootDeclarations, WorthUiStructuralParseFailure> {
        let mut parsed = ParsedRootDeclarations {
            regions: Vec::new(),
            projection_contents: Vec::new(),
            interaction_routes: Vec::new(),
        };
        let mut projection_identities = std::collections::BTreeSet::new();
        while !self.cursor.is_eof() {
            match self.cursor.peek_identifier().as_deref() {
                Some("region") => parsed.regions.push(self.parse_region("root")?),
                Some("content") => {
                    let content = self.parse_projection_content()?;
                    if !projection_identities.insert(content.projection_identity_text().to_owned())
                    {
                        return Err(WorthUiStructuralParseFailure {
                            code: WorthUiStructuralLanguageDiagnosticCode::InvalidStructuralSyntax,
                            authored_text: content.projection_identity_text().to_owned(),
                            structural_locus: "root/content:projection".to_owned(),
                        });
                    }
                    parsed.projection_contents.push(content);
                }
                Some("interaction") => parsed
                    .interaction_routes
                    .push(self.parse_interaction_route()?),
                _ => return Err(self.illegal_root_statement()),
            }
        }
        Ok(parsed)
    }

    fn parse_interaction_route(
        &mut self,
    ) -> Result<crate::WorthUiIntentInteractionRoute, WorthUiStructuralParseFailure> {
        self.cursor
            .expect_keyword("interaction", "root/interaction")?;
        let family = self.parse_interaction_family()?;
        let kind = self.parse_interaction_route_kind()?;
        if matches!(kind, crate::WorthUiIntentInteractionRouteKind::Confirmation)
            && family != crate::WorthUiIntentInteractionFamily::Activate
        {
            return Err(WorthUiStructuralParseFailure {
                code: WorthUiStructuralLanguageDiagnosticCode::InvalidStructuralSyntax,
                authored_text: family.as_str().to_owned(),
                structural_locus: "root/interaction:confirmation-family".to_owned(),
            });
        }
        let declaration_identity = self
            .cursor
            .expect_identifier("root/interaction:declaration")?;
        let opened_portal_identity = if self.cursor.peek_identifier().as_deref() == Some("opens") {
            self.cursor
                .expect_keyword("opens", "root/interaction:opens")?;
            self.cursor
                .expect_keyword("portal", "root/interaction:opens-kind")?;
            Some(
                self.cursor
                    .expect_identifier("root/interaction:opened-portal")?,
            )
        } else {
            None
        };
        if opened_portal_identity.is_some()
            && matches!(kind, crate::WorthUiIntentInteractionRouteKind::Confirmation)
        {
            return Err(WorthUiStructuralParseFailure {
                code: WorthUiStructuralLanguageDiagnosticCode::InvalidStructuralSyntax,
                authored_text: "opens portal".to_owned(),
                structural_locus: "root/interaction:confirmation-portal".to_owned(),
            });
        }
        self.cursor.consume_optional_semicolon();
        Ok(crate::WorthUiIntentInteractionRoute::from_authored_parts(
            family,
            declaration_identity,
            kind,
            opened_portal_identity,
        ))
    }

    fn parse_interaction_family(
        &mut self,
    ) -> Result<crate::WorthUiIntentInteractionFamily, WorthUiStructuralParseFailure> {
        let authored = self.cursor.expect_identifier("root/interaction:family")?;
        crate::WorthUiIntentInteractionFamily::parse(&authored).ok_or(
            WorthUiStructuralParseFailure {
                code: WorthUiStructuralLanguageDiagnosticCode::InvalidStructuralSyntax,
                authored_text: authored,
                structural_locus: "root/interaction:family".to_owned(),
            },
        )
    }

    fn parse_interaction_route_kind(
        &mut self,
    ) -> Result<crate::WorthUiIntentInteractionRouteKind, WorthUiStructuralParseFailure> {
        let authored = self
            .cursor
            .expect_identifier("root/interaction:route-kind")?;
        match authored.as_str() {
            "routes" => Ok(crate::WorthUiIntentInteractionRouteKind::Product),
            "confirms" => Ok(crate::WorthUiIntentInteractionRouteKind::Confirmation),
            _ => Err(WorthUiStructuralParseFailure {
                code: WorthUiStructuralLanguageDiagnosticCode::InvalidStructuralSyntax,
                authored_text: authored,
                structural_locus: "root/interaction:route-kind".to_owned(),
            }),
        }
    }

    fn parse_projection_content(
        &mut self,
    ) -> Result<WorthUiAuthoredProjectionContent, WorthUiStructuralParseFailure> {
        self.cursor.expect_keyword("content", "root")?;
        self.cursor.expect_keyword("projection", "root/content")?;
        let identity = self.cursor.expect_identifier("root/content:projection")?;
        self.cursor.consume_optional_semicolon();
        Ok(WorthUiAuthoredProjectionContent::new(identity))
    }

    fn illegal_root_statement(&self) -> WorthUiStructuralParseFailure {
        WorthUiStructuralParseFailure {
            code: WorthUiStructuralLanguageDiagnosticCode::IllegalRootStructuralStatement,
            authored_text: self
                .cursor
                .peek_identifier()
                .unwrap_or_else(|| "unexpected_eof".to_owned()),
            structural_locus: "root".to_owned(),
        }
    }
}
