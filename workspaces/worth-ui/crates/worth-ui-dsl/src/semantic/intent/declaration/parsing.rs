use crate::source::WorthUiArtifactInputBodyAtom;
use crate::{
    WorthUiIntentMutabilitySourceSpec, WorthUiIntentPolicySourceSpec,
    WorthUiIntentReadinessSourceSpec,
};

use super::{
    WorthUiIntentConcurrencyScope, WorthUiIntentConfirmationContractSpec,
    WorthUiIntentDeclarationParseError, WorthUiIntentDeclarationSpec,
    WorthUiIntentInteractionFamily, WorthUiIntentOperabilityContractSpec,
    WorthUiIntentPayloadSourceSpec,
};

pub(super) fn parse(
    identity: &str,
    body: &[WorthUiArtifactInputBodyAtom],
) -> Result<WorthUiIntentDeclarationSpec, WorthUiIntentDeclarationParseError> {
    let mut cursor = IntentDeclarationCursor::new(body);
    let mut definition = None;
    let mut interaction = None;
    let mut payload_sources = Vec::new();
    let mut operability = None;
    let mut confirmation = None;
    let mut concurrency = None;
    let mut consequences = None;
    while !cursor.is_eof() {
        match cursor.take_identifier()?.as_str() {
            "definition" => set_once(&mut definition, cursor.take_identifier()?, "definition")?,
            "interaction" => {
                let authored = cursor.take_identifier()?;
                let family = WorthUiIntentInteractionFamily::parse(&authored)
                    .ok_or_else(|| error(format!("unknown interaction family `{authored}`")))?;
                set_once(&mut interaction, family, "interaction")?;
            }
            "payload" => payload_sources.push(parse_payload_source(&mut cursor)?),
            "operability" => set_once(
                &mut operability,
                parse_operability_contract(&mut cursor)?,
                "operability",
            )?,
            "confirmation" => set_once(
                &mut confirmation,
                parse_confirmation_contract(&mut cursor)?,
                "confirmation",
            )?,
            "concurrency" => {
                let authored = cursor.take_identifier()?;
                let scope = WorthUiIntentConcurrencyScope::parse(&authored).ok_or_else(|| {
                    error(format!("unknown intent concurrency scope `{authored}`"))
                })?;
                set_once(&mut concurrency, scope, "concurrency")?;
            }
            "consequences" => set_once(
                &mut consequences,
                parse_consequence_contract(&mut cursor)?,
                "consequences",
            )?,
            clause => {
                return Err(error(format!(
                    "unknown intent declaration clause `{clause}`"
                )));
            }
        }
        cursor.take_optional_semicolon();
    }
    let mut declaration = WorthUiIntentDeclarationSpec::new(
        identity,
        definition.ok_or_else(|| error("intent declaration requires `definition`"))?,
        interaction.ok_or_else(|| error("intent declaration requires `interaction`"))?,
        operability.ok_or_else(|| error("intent declaration requires `operability`"))?,
        confirmation.ok_or_else(|| error("intent declaration requires `confirmation`"))?,
        concurrency.ok_or_else(|| error("intent declaration requires `concurrency`"))?,
        consequences.ok_or_else(|| error("intent declaration requires `consequences`"))?,
    );
    declaration.payload_sources = payload_sources;
    Ok(declaration)
}

fn parse_consequence_contract(
    cursor: &mut IntentDeclarationCursor<'_>,
) -> Result<crate::WorthUiIntentConsequenceContractSpec, WorthUiIntentDeclarationParseError> {
    let shape = cursor.take_identifier()?;
    Ok(match shape.as_str() {
        "none" => crate::WorthUiIntentConsequenceContractSpec::none(),
        "mounted-posture" => crate::WorthUiIntentConsequenceContractSpec::mounted_posture(),
        "query-collection-change" => {
            crate::WorthUiIntentConsequenceContractSpec::query_collection_change(
                cursor.take_identifier()?,
            )
        }
        "mounted-posture-and-query" => {
            crate::WorthUiIntentConsequenceContractSpec::mounted_posture_and_query(
                cursor.take_identifier()?,
            )
        }
        _ => {
            return Err(error(format!(
                "unknown intent consequence contract `{shape}`"
            )))
        }
    })
}

fn parse_operability_contract(
    cursor: &mut IntentDeclarationCursor<'_>,
) -> Result<WorthUiIntentOperabilityContractSpec, WorthUiIntentDeclarationParseError> {
    let identity = cursor.take_identifier()?;
    let mutability = parse_mutability_source(cursor)?;
    let readiness = parse_readiness_source(cursor)?;
    let policy_kind = cursor.take_identifier()?;
    let policy = match policy_kind.as_str() {
        "policy-application-boolean" => {
            WorthUiIntentPolicySourceSpec::application_boolean(cursor.take_identifier()?)
        }
        "policy-condition" => WorthUiIntentPolicySourceSpec::condition(cursor.take_identifier()?),
        _ => {
            return Err(error(format!(
                "unknown intent policy source kind `{policy_kind}`"
            )))
        }
    };
    Ok(WorthUiIntentOperabilityContractSpec::new(
        identity, mutability, readiness, policy,
    ))
}

fn parse_mutability_source(
    cursor: &mut IntentDeclarationCursor<'_>,
) -> Result<WorthUiIntentMutabilitySourceSpec, WorthUiIntentDeclarationParseError> {
    let kind = cursor.take_identifier()?;
    Ok(match kind.as_str() {
        "mutability-application-boolean" => {
            WorthUiIntentMutabilitySourceSpec::application_boolean(cursor.take_identifier()?)
        }
        "mutability-projection-readonly" => {
            WorthUiIntentMutabilitySourceSpec::projection_readonly(cursor.take_identifier()?)
        }
        "mutability-committed-draft" => WorthUiIntentMutabilitySourceSpec::committed_draft(),
        "mutability-condition" => {
            WorthUiIntentMutabilitySourceSpec::condition(cursor.take_identifier()?)
        }
        _ => {
            return Err(error(format!(
                "unknown intent mutability source kind `{kind}`"
            )))
        }
    })
}

fn parse_readiness_source(
    cursor: &mut IntentDeclarationCursor<'_>,
) -> Result<WorthUiIntentReadinessSourceSpec, WorthUiIntentDeclarationParseError> {
    let kind = cursor.take_identifier()?;
    Ok(match kind.as_str() {
        "readiness-application-boolean" => {
            WorthUiIntentReadinessSourceSpec::application_boolean(cursor.take_identifier()?)
        }
        "readiness-projection" => {
            WorthUiIntentReadinessSourceSpec::projection(cursor.take_identifier()?)
        }
        "readiness-committed-draft" => WorthUiIntentReadinessSourceSpec::committed_draft(),
        "readiness-condition" => {
            WorthUiIntentReadinessSourceSpec::condition(cursor.take_identifier()?)
        }
        _ => {
            return Err(error(format!(
                "unknown intent readiness source kind `{kind}`"
            )))
        }
    })
}

fn parse_confirmation_contract(
    cursor: &mut IntentDeclarationCursor<'_>,
) -> Result<WorthUiIntentConfirmationContractSpec, WorthUiIntentDeclarationParseError> {
    let policy_identity = cursor.take_identifier()?;
    let source = cursor.take_identifier()?;
    match source.as_str() {
        "not-required" => Ok(WorthUiIntentConfirmationContractSpec::not_required(
            policy_identity,
        )),
        "application-boolean" => Ok(WorthUiIntentConfirmationContractSpec::application_boolean(
            policy_identity,
            cursor.take_identifier()?,
        )),
        _ => Err(error(format!(
            "unknown intent confirmation source kind `{source}`"
        ))),
    }
}

fn set_once<T>(
    slot: &mut Option<T>,
    value: T,
    clause: &str,
) -> Result<(), WorthUiIntentDeclarationParseError> {
    if slot.replace(value).is_some() {
        return Err(error(format!("intent declaration repeats `{clause}`")));
    }
    Ok(())
}

fn parse_payload_source(
    cursor: &mut IntentDeclarationCursor<'_>,
) -> Result<WorthUiIntentPayloadSourceSpec, WorthUiIntentDeclarationParseError> {
    let field = cursor.take_identifier()?;
    let kind = cursor.take_identifier()?;
    Ok(match kind.as_str() {
        "projection-text" => {
            WorthUiIntentPayloadSourceSpec::projection_text(field, cursor.take_identifier()?)
        }
        "projection-selection" => {
            WorthUiIntentPayloadSourceSpec::projection_selection(field, cursor.take_identifier()?)
        }
        "committed-draft" => WorthUiIntentPayloadSourceSpec::committed_draft(field),
        "constant-text" => {
            WorthUiIntentPayloadSourceSpec::constant_text(field, cursor.take_string_literal()?)
        }
        "constant-boolean" => {
            let value = match cursor.take_identifier()?.as_str() {
                "true" => true,
                "false" => false,
                value => return Err(error(format!("invalid boolean payload constant `{value}`"))),
            };
            WorthUiIntentPayloadSourceSpec::constant_boolean(field, value)
        }
        "constant-unsigned64" => {
            let authored = cursor.take_string_literal()?;
            let value = authored
                .parse::<u64>()
                .map_err(|_| error(format!("invalid unsigned-64 payload constant `{authored}`")))?;
            WorthUiIntentPayloadSourceSpec::constant_unsigned64(field, value)
        }
        "application-text" => {
            WorthUiIntentPayloadSourceSpec::application_text(field, cursor.take_identifier()?)
        }
        "application-boolean" => {
            WorthUiIntentPayloadSourceSpec::application_boolean(field, cursor.take_identifier()?)
        }
        "application-unsigned64" => {
            WorthUiIntentPayloadSourceSpec::application_unsigned64(field, cursor.take_identifier()?)
        }
        _ => return Err(error(format!("unknown payload source kind `{kind}`"))),
    })
}

pub(super) fn error(detail: impl Into<Box<str>>) -> WorthUiIntentDeclarationParseError {
    WorthUiIntentDeclarationParseError {
        detail: detail.into(),
    }
}

pub(super) struct IntentDeclarationCursor<'a> {
    atoms: &'a [WorthUiArtifactInputBodyAtom],
    cursor: usize,
}

impl<'a> IntentDeclarationCursor<'a> {
    pub(super) fn new(atoms: &'a [WorthUiArtifactInputBodyAtom]) -> Self {
        Self { atoms, cursor: 0 }
    }

    pub(super) fn take_identifier(&mut self) -> Result<String, WorthUiIntentDeclarationParseError> {
        match self.atoms.get(self.cursor) {
            Some(WorthUiArtifactInputBodyAtom::Identifier(value)) => {
                self.cursor += 1;
                Ok(value.clone())
            }
            _ => Err(error("intent declaration expected an identifier")),
        }
    }

    fn take_string_literal(&mut self) -> Result<String, WorthUiIntentDeclarationParseError> {
        match self.atoms.get(self.cursor) {
            Some(WorthUiArtifactInputBodyAtom::StringLiteral(value)) => {
                self.cursor += 1;
                Ok(value.clone())
            }
            _ => Err(error("intent declaration expected a string literal")),
        }
    }

    pub(super) fn take_optional_semicolon(&mut self) {
        if matches!(
            self.atoms.get(self.cursor),
            Some(WorthUiArtifactInputBodyAtom::Semicolon)
        ) {
            self.cursor += 1;
        }
    }

    pub(super) fn is_eof(&self) -> bool {
        self.cursor == self.atoms.len()
    }
}
