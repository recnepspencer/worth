use crate::source::WorthUiArtifactInputBodyAtom;

use super::declaration::WorthUiProjectionRequirementParts;
use super::{
    WorthUiProjectionCollectionPolicy, WorthUiProjectionDeclarationError,
    WorthUiProjectionDeclarationErrorKind, WorthUiProjectionLifecycle,
    WorthUiProjectionNativeFamily, WorthUiProjectionRequirement, WorthUiProjectionShape,
};

pub(crate) fn parse_projection_requirement(
    declaration_identity: &str,
    shape: WorthUiProjectionShape,
    atoms: &[WorthUiArtifactInputBodyAtom],
) -> Result<WorthUiProjectionRequirement, WorthUiProjectionDeclarationError> {
    let clauses = Clauses::parse(atoms)?;
    let view = clauses.required_one("view")?;
    let native_family = match clauses.required_one("require")? {
        "text" => WorthUiProjectionNativeFamily::Text,
        "boolean" => WorthUiProjectionNativeFamily::Boolean,
        other => return Err(unsupported("require", other)),
    };
    let lifecycle = match clauses.optional_one("lifecycle")?.unwrap_or("live") {
        "snapshot" => WorthUiProjectionLifecycle::Snapshot,
        "live" => WorthUiProjectionLifecycle::Live,
        other => return Err(unsupported("lifecycle", other)),
    };
    let fields = clauses.required_many("field")?;

    match shape {
        WorthUiProjectionShape::Scalar => {
            WorthUiProjectionRequirement::build(WorthUiProjectionRequirementParts {
                declaration_identity: declaration_identity.to_owned(),
                view_identity: view.to_owned(),
                shape,
                selected_fields: fields.into_iter().map(str::to_owned).collect(),
                row_identity_field: clauses.forbid("row")?,
                native_family,
                lifecycle,
                collection_policy: clauses.forbid_policy()?,
            })
        }
        WorthUiProjectionShape::Collection => {
            let row = clauses.required_one("row")?;
            let completeness = clauses.required_one("completeness")?;
            let continuation = clauses.required_one("continuation")?;
            let policy = WorthUiProjectionCollectionPolicy::new(
                match completeness {
                    "complete" => true,
                    "partial" => false,
                    other => return Err(unsupported("completeness", other)),
                },
                match continuation {
                    "allowed" => true,
                    "forbidden" => false,
                    other => return Err(unsupported("continuation", other)),
                },
            );
            WorthUiProjectionRequirement::build(WorthUiProjectionRequirementParts {
                declaration_identity: declaration_identity.to_owned(),
                view_identity: view.to_owned(),
                shape,
                selected_fields: fields.into_iter().map(str::to_owned).collect(),
                row_identity_field: Some(row.to_owned()),
                native_family,
                lifecycle,
                collection_policy: Some(policy),
            })
        }
    }
}

struct Clauses<'a> {
    entries: Vec<(&'a str, &'a str)>,
}

impl<'a> Clauses<'a> {
    fn parse(
        atoms: &'a [WorthUiArtifactInputBodyAtom],
    ) -> Result<Self, WorthUiProjectionDeclarationError> {
        let words = atoms
            .iter()
            .filter_map(|atom| match atom {
                WorthUiArtifactInputBodyAtom::Identifier(value)
                | WorthUiArtifactInputBodyAtom::StringLiteral(value) => Some(value.as_str()),
                WorthUiArtifactInputBodyAtom::Semicolon => None,
                _ => Some(""),
            })
            .collect::<Vec<_>>();
        if words.iter().any(|word| word.is_empty()) || words.len() % 2 != 0 {
            return Err(WorthUiProjectionDeclarationError::new(
                WorthUiProjectionDeclarationErrorKind::UnknownClause,
                "projection body must contain clause/value pairs",
            ));
        }
        let mut entries = Vec::with_capacity(words.len() / 2);
        let (pairs, _) = words.as_chunks::<2>();
        for pair in pairs {
            if !matches!(
                pair[0],
                "view"
                    | "field"
                    | "row"
                    | "require"
                    | "lifecycle"
                    | "completeness"
                    | "continuation"
            ) {
                return Err(WorthUiProjectionDeclarationError::new(
                    WorthUiProjectionDeclarationErrorKind::UnknownClause,
                    format!("unknown projection clause `{}`", pair[0]),
                ));
            }
            entries.push((pair[0], pair[1]));
        }
        Ok(Self { entries })
    }

    fn required_one(&self, name: &str) -> Result<&'a str, WorthUiProjectionDeclarationError> {
        self.optional_one(name)?.ok_or_else(|| {
            WorthUiProjectionDeclarationError::new(
                WorthUiProjectionDeclarationErrorKind::MissingClause,
                format!("projection declaration requires `{name}`"),
            )
        })
    }

    fn optional_one(
        &self,
        name: &str,
    ) -> Result<Option<&'a str>, WorthUiProjectionDeclarationError> {
        let values = self.values(name).collect::<Vec<_>>();
        if values.len() > 1 {
            return Err(WorthUiProjectionDeclarationError::new(
                WorthUiProjectionDeclarationErrorKind::DuplicateClause,
                format!("projection clause `{name}` appears more than once"),
            ));
        }
        Ok(values.into_iter().next())
    }

    fn required_many(&self, name: &str) -> Result<Vec<&'a str>, WorthUiProjectionDeclarationError> {
        let values = self.values(name).collect::<Vec<_>>();
        if values.is_empty() {
            return Err(WorthUiProjectionDeclarationError::new(
                WorthUiProjectionDeclarationErrorKind::MissingClause,
                format!("projection declaration requires `{name}`"),
            ));
        }
        Ok(values)
    }

    fn forbid(&self, name: &str) -> Result<Option<String>, WorthUiProjectionDeclarationError> {
        if self.values(name).next().is_some() {
            return Err(WorthUiProjectionDeclarationError::new(
                WorthUiProjectionDeclarationErrorKind::ShapeClauseMismatch,
                format!("scalar projection cannot declare `{name}`"),
            ));
        }
        Ok(None)
    }

    fn forbid_policy(
        &self,
    ) -> Result<Option<WorthUiProjectionCollectionPolicy>, WorthUiProjectionDeclarationError> {
        self.forbid("completeness")?;
        self.forbid("continuation")?;
        Ok(None)
    }

    fn values<'borrow>(
        &'borrow self,
        name: &'borrow str,
    ) -> impl Iterator<Item = &'a str> + 'borrow {
        self.entries
            .iter()
            .filter(move |(clause, _)| *clause == name)
            .map(move |(_, value)| *value)
    }
}

fn unsupported(clause: &str, value: &str) -> WorthUiProjectionDeclarationError {
    WorthUiProjectionDeclarationError::new(
        WorthUiProjectionDeclarationErrorKind::UnsupportedValue,
        format!("projection clause `{clause}` does not support `{value}`"),
    )
}
