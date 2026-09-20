mod command;
mod focus;
mod motion;
mod portal;
mod scroll;
mod selection;

pub use command::{
    WorthUiCommandDeclaration, WorthUiCommandKey, WorthUiCommandModifier, WorthUiCommandScope,
    WorthUiCommandShortcutStrokeSpec,
};
pub use focus::{WorthUiFocusDeclaration, WorthUiFocusScope};
pub use motion::{WorthUiMotionDeclaration, WorthUiReducedMotionPolicy};
pub use portal::{WorthUiPortalDeclaration, WorthUiPortalDismissalSet, WorthUiPortalLayer};
pub use scroll::{
    WorthUiScrollAnchorPolicy, WorthUiScrollChromeAxes, WorthUiScrollChromeDeclaration,
    WorthUiScrollDeclaration, WorthUiScrollWheelPolicy,
};
pub use selection::{WorthUiSelectionDeclaration, WorthUiSelectionMode};

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WorthUiServiceFamily {
    Portal,
    Focus,
    Motion,
    CommandRouting,
    Scroll,
    Selection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WorthUiServiceDeclarationMeaning {
    Portal(WorthUiPortalDeclaration),
    Focus(WorthUiFocusDeclaration),
    Motion(WorthUiMotionDeclaration),
    Command(WorthUiCommandDeclaration),
    Scroll(WorthUiScrollDeclaration),
    Selection(WorthUiSelectionDeclaration),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthUiServiceDeclarationParseError {
    detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum Word {
    Text(String),
    Plus,
}

#[derive(Clone, Copy)]
pub(super) enum ClauseRule {
    Single(&'static str),
    List(&'static str),
    Flag(&'static str),
}

impl ClauseRule {
    const fn name(self) -> &'static str {
        match self {
            Self::Single(name) | Self::List(name) | Self::Flag(name) => name,
        }
    }
}

impl WorthUiServiceDeclarationMeaning {
    pub(crate) fn parse(
        family: WorthUiServiceFamily,
        identity: &str,
        atoms: &[crate::WorthUiArtifactInputBodyAtom],
    ) -> Result<Self, WorthUiServiceDeclarationParseError> {
        let words = atoms
            .iter()
            .filter_map(|atom| match atom {
                crate::WorthUiArtifactInputBodyAtom::Identifier(text) => {
                    Some(Word::Text(text.clone()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordImport => {
                    Some(Word::Text("import".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordComponent => {
                    Some(Word::Text("component".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordControl => {
                    Some(Word::Text("control".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordIntent => {
                    Some(Word::Text("intent".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordSurface => {
                    Some(Word::Text("surface".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordBinding => {
                    Some(Word::Text("binding".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordQueryScalar => {
                    Some(Word::Text("query_scalar".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordQueryCollection => {
                    Some(Word::Text("query_collection".to_owned()))
                }
                crate::WorthUiArtifactInputBodyAtom::KeywordToken => {
                    Some(Word::Text("token".to_owned()))
                }
                // A clause value may be a number, as `line_extent 20` is. The
                // family that owns the clause decides what that number means;
                // a family with no numeric clause still denies it as an
                // unknown clause or an invalid value.
                crate::WorthUiArtifactInputBodyAtom::NumberLiteral(text) => {
                    Some(Word::Text(text.clone()))
                }
                crate::WorthUiArtifactInputBodyAtom::Plus => Some(Word::Plus),
                crate::WorthUiArtifactInputBodyAtom::LeftBrace
                | crate::WorthUiArtifactInputBodyAtom::RightBrace
                | crate::WorthUiArtifactInputBodyAtom::Semicolon => None,
                _ => Some(Word::Text("<invalid-token>".to_owned())),
            })
            .collect::<Vec<_>>();
        Ok(match family {
            WorthUiServiceFamily::Portal => {
                Self::Portal(WorthUiPortalDeclaration::parse(identity, &words)?)
            }
            WorthUiServiceFamily::Focus => {
                Self::Focus(WorthUiFocusDeclaration::parse(identity, &words)?)
            }
            WorthUiServiceFamily::Motion => {
                Self::Motion(WorthUiMotionDeclaration::parse(identity, &words)?)
            }
            WorthUiServiceFamily::CommandRouting => {
                Self::Command(WorthUiCommandDeclaration::parse(identity, &words)?)
            }
            WorthUiServiceFamily::Scroll => {
                Self::Scroll(WorthUiScrollDeclaration::parse(identity, &words)?)
            }
            WorthUiServiceFamily::Selection => {
                Self::Selection(WorthUiSelectionDeclaration::parse(identity, &words)?)
            }
        })
    }

    pub const fn family(&self) -> WorthUiServiceFamily {
        match self {
            Self::Portal(_) => WorthUiServiceFamily::Portal,
            Self::Focus(_) => WorthUiServiceFamily::Focus,
            Self::Motion(_) => WorthUiServiceFamily::Motion,
            Self::Command(_) => WorthUiServiceFamily::CommandRouting,
            Self::Scroll(_) => WorthUiServiceFamily::Scroll,
            Self::Selection(_) => WorthUiServiceFamily::Selection,
        }
    }

    pub(crate) fn canonical_text(&self) -> String {
        match self {
            Self::Portal(value) => value.canonical_text(),
            Self::Focus(value) => value.canonical_text(),
            Self::Motion(value) => value.canonical_text(),
            Self::Command(value) => value.canonical_text(),
            Self::Scroll(value) => value.canonical_text(),
            Self::Selection(value) => value.canonical_text(),
        }
    }
}

impl WorthUiServiceDeclarationParseError {
    pub fn detail(&self) -> &str {
        &self.detail
    }
}

pub(super) fn one_value<'a>(
    words: &'a [Word],
    clause: &str,
) -> Result<&'a str, WorthUiServiceDeclarationParseError> {
    let index =
        word_index(words, clause).ok_or_else(|| missing(clause, "add the required clause"))?;
    match words.get(index + 1) {
        Some(Word::Text(value)) => Ok(value),
        _ => Err(missing(clause, "add one typed value after the clause")),
    }
}

pub(super) fn optional_value<'a>(words: &'a [Word], clause: &str) -> Option<&'a str> {
    let index = word_index(words, clause)?;
    match words.get(index + 1) {
        Some(Word::Text(value)) => Some(value),
        _ => None,
    }
}

pub(super) fn validate_clauses(
    words: &[Word],
    rules: &[ClauseRule],
) -> Result<(), WorthUiServiceDeclarationParseError> {
    let mut observed = std::collections::BTreeSet::new();
    let mut index = 0;
    while index < words.len() {
        let Word::Text(clause) = &words[index] else {
            return Err(invalid(
                "service clause",
                "+",
                "use '+' only inside a shortcut clause",
            ));
        };
        let Some(rule) = rules.iter().copied().find(|rule| rule.name() == clause) else {
            return Err(invalid(
                "service clause",
                clause,
                "remove the unknown clause or use a family-owned clause",
            ));
        };
        if !observed.insert(clause.as_str()) {
            return Err(invalid(
                "service clause",
                clause,
                "declare each clause exactly once",
            ));
        }
        index += 1;
        match rule {
            ClauseRule::Flag(_) => {}
            ClauseRule::Single(name) => {
                let Some(Word::Text(_)) = words.get(index) else {
                    return Err(missing(name, "add one typed value after the clause"));
                };
                index += 1;
                if index < words.len() && !is_clause(&words[index], rules) {
                    return Err(invalid(
                        "service clause",
                        word_text(&words[index]),
                        "a single-value clause accepts exactly one value",
                    ));
                }
            }
            ClauseRule::List(_) => {
                while index < words.len() && !is_clause(&words[index], rules) {
                    index += 1;
                }
            }
        }
    }
    Ok(())
}

fn is_clause(word: &Word, rules: &[ClauseRule]) -> bool {
    matches!(word, Word::Text(value) if rules.iter().any(|rule| rule.name() == value))
}

fn word_text(word: &Word) -> &str {
    match word {
        Word::Text(value) => value,
        Word::Plus => "+",
    }
}

pub(super) fn values_until<'a>(
    words: &'a [Word],
    clause: &str,
    stops: &[&str],
) -> Result<Vec<&'a str>, WorthUiServiceDeclarationParseError> {
    let index =
        word_index(words, clause).ok_or_else(|| missing(clause, "add the required clause"))?;
    Ok(words[index + 1..]
        .iter()
        .take_while(|word| match word {
            Word::Text(value) => !stops.contains(&value.as_str()),
            Word::Plus => true,
        })
        .filter_map(|word| match word {
            Word::Text(value) => Some(value.as_str()),
            Word::Plus => None,
        })
        .collect())
}

pub(super) fn words_until(
    words: &[Word],
    clause: &str,
    stops: &[&str],
) -> Result<Vec<Word>, WorthUiServiceDeclarationParseError> {
    let index =
        word_index(words, clause).ok_or_else(|| missing(clause, "add the required clause"))?;
    Ok(words[index + 1..]
        .iter()
        .take_while(|word| match word {
            Word::Text(value) => !stops.contains(&value.as_str()),
            Word::Plus => true,
        })
        .cloned()
        .collect())
}

pub(super) fn optional_flag(words: &[Word], flag: &str) -> bool {
    word_index(words, flag).is_some()
}
fn word_index(words: &[Word], expected: &str) -> Option<usize> {
    words
        .iter()
        .position(|word| matches!(word, Word::Text(value) if value == expected))
}
pub(super) fn missing(law: &str, repair: &str) -> WorthUiServiceDeclarationParseError {
    WorthUiServiceDeclarationParseError {
        detail: format!("service law '{law}' is missing; lawful repair: {repair}"),
    }
}
pub(super) fn invalid(
    law: &str,
    observed: &str,
    repair: &str,
) -> WorthUiServiceDeclarationParseError {
    WorthUiServiceDeclarationParseError {
        detail: format!("service law '{law}' rejected '{observed}'; lawful repair: {repair}"),
    }
}
