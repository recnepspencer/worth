use crate::WorthUiArtifactInputBodyAtom;

use super::{
    WorthUiExpressionBody, WorthUiExpressionDeclaration, WorthUiExpressionDeclarationError,
    WorthUiExpressionDeclarationErrorKind as ErrorKind, WorthUiExpressionIntroducer,
    WorthUiExpressionOperand, WorthUiExpressionOperandSource, WorthUiExpressionResultType,
    WorthUiExpressionRole,
};

impl WorthUiExpressionDeclaration {
    /// Parses the body atoms of a `condition` or `derived` block. The
    /// introducer the block kind requires decides the role.
    pub(crate) fn parse(
        identity: &str,
        expected: WorthUiExpressionIntroducer,
        atoms: &[WorthUiArtifactInputBodyAtom],
    ) -> Result<Self, WorthUiExpressionDeclarationError> {
        let mut clauses = Clauses { atoms, position: 0 };
        let mut operands = Vec::new();
        let mut result = None;
        let mut body = None;
        while let Some(atom) = clauses.next() {
            if body.is_some() {
                return Err(error(
                    ErrorKind::MisplacedBody,
                    format!(
                        "`{}` must be the last clause of `{identity}`",
                        expected.keyword()
                    ),
                ));
            }
            match atom {
                WorthUiArtifactInputBodyAtom::ExpressionBody(found) => {
                    body = Some(check_introducer(found, expected)?);
                }
                WorthUiArtifactInputBodyAtom::Identifier(word) if word == "operand" => {
                    operands.push(parse_operand(&mut clauses)?);
                }
                WorthUiArtifactInputBodyAtom::Identifier(word) if word == "result" => {
                    if result.replace(parse_result(&mut clauses)?).is_some() {
                        return Err(error(
                            ErrorKind::DuplicateResult,
                            "`result` is declared more than once",
                        ));
                    }
                }
                other => {
                    return Err(error(
                        ErrorKind::UnknownClause,
                        format!("unknown expression clause {}", describe(other)),
                    ))
                }
            }
        }
        let body = body.ok_or_else(|| {
            error(
                ErrorKind::MissingBody,
                format!("`{identity}` requires a `{}(...)` body", expected.keyword()),
            )
        })?;
        let role = match (expected, result) {
            (WorthUiExpressionIntroducer::When, None) => WorthUiExpressionRole::Condition,
            (WorthUiExpressionIntroducer::When, Some(_)) => {
                return Err(error(
                    ErrorKind::ExtraResult,
                    "a condition is always boolean and declares no `result`",
                ))
            }
            (WorthUiExpressionIntroducer::Value, Some(result)) => {
                WorthUiExpressionRole::Derived(result)
            }
            (WorthUiExpressionIntroducer::Value, None) => {
                return Err(error(
                    ErrorKind::MissingResult,
                    "a derived declaration requires `result <type>;`",
                ))
            }
        };
        Self::build(identity.to_owned(), role, operands, body)
    }
}

struct Clauses<'a> {
    atoms: &'a [WorthUiArtifactInputBodyAtom],
    position: usize,
}

impl<'a> Clauses<'a> {
    fn next(&mut self) -> Option<&'a WorthUiArtifactInputBodyAtom> {
        let atom = self.atoms.get(self.position)?;
        self.position += 1;
        Some(atom)
    }

    fn word(&mut self, expected: &str) -> Result<&'a str, WorthUiExpressionDeclarationError> {
        match self.next() {
            Some(WorthUiArtifactInputBodyAtom::Identifier(text)) => Ok(text),
            Some(WorthUiArtifactInputBodyAtom::KeywordToken) => Ok("token"),
            Some(other) => Err(error(
                ErrorKind::MalformedClause,
                format!("expected {expected}, found {}", describe(other)),
            )),
            None => Err(error(
                ErrorKind::MalformedClause,
                format!("expected {expected}, found the end of the declaration"),
            )),
        }
    }

    fn semicolon(&mut self, clause: &str) -> Result<(), WorthUiExpressionDeclarationError> {
        match self.next() {
            Some(WorthUiArtifactInputBodyAtom::Semicolon) => Ok(()),
            _ => Err(error(
                ErrorKind::MalformedClause,
                format!("`{clause}` clause must end with a semicolon"),
            )),
        }
    }
}

fn parse_operand(
    clauses: &mut Clauses<'_>,
) -> Result<WorthUiExpressionOperand, WorthUiExpressionDeclarationError> {
    let name = clauses.word("an operand name after `operand`")?;
    let keyword = clauses.word("an operand source kind")?;
    let reference = clauses.word("an operand source reference")?;
    clauses.semicolon("operand")?;
    let source =
        WorthUiExpressionOperandSource::from_clause(keyword, reference).ok_or_else(|| {
            error(
                ErrorKind::UnknownOperandSource,
                format!(
                    "operand `{name}` uses unknown source kind `{keyword}`; expected \
                     query-scalar, application-boolean, application-unsigned64, \
                     application-text, condition, or derived"
                ),
            )
        })?;
    Ok(WorthUiExpressionOperand::new(name, source))
}

fn parse_result(
    clauses: &mut Clauses<'_>,
) -> Result<WorthUiExpressionResultType, WorthUiExpressionDeclarationError> {
    let text = clauses.word("a result type after `result`")?;
    clauses.semicolon("result")?;
    if text == "boolean" {
        return Err(error(
            ErrorKind::BooleanResult,
            "a derived declaration cannot yield boolean; declare a `condition` instead",
        ));
    }
    WorthUiExpressionResultType::from_clause(text).ok_or_else(|| {
        error(
            ErrorKind::UnknownResultType,
            format!("unknown result type `{text}`; expected text, integer, decimal, or token"),
        )
    })
}

fn check_introducer(
    found: &WorthUiExpressionBody,
    expected: WorthUiExpressionIntroducer,
) -> Result<WorthUiExpressionBody, WorthUiExpressionDeclarationError> {
    if found.introducer() == expected {
        return Ok(found.clone());
    }
    Err(error(
        ErrorKind::WrongIntroducer,
        format!(
            "expected `{}(...)` for this declaration, found `{}(...)`",
            expected.keyword(),
            found.introducer().keyword()
        ),
    ))
}

fn describe(atom: &WorthUiArtifactInputBodyAtom) -> String {
    match atom {
        WorthUiArtifactInputBodyAtom::Identifier(text) => format!("`{text}`"),
        other => format!("{other:?}"),
    }
}

fn error(kind: ErrorKind, detail: impl Into<String>) -> WorthUiExpressionDeclarationError {
    WorthUiExpressionDeclarationError::new(kind, detail)
}
