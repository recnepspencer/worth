//! Qualified names and versioned nominal type names.

use std::fmt;

use crate::expressions::denial::{ExpressionDenial, ExpressionDenialDetail, SyntaxDenial};

const KEYWORDS: [&str; 4] = ["true", "false", "let", "none"];

/// Whether `segment` is an ASCII identifier that is not a keyword.
pub(crate) fn is_identifier(segment: &str) -> bool {
    let mut bytes = segment.bytes();
    bytes
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic() || first == b'_')
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        && !KEYWORDS.contains(&segment)
}

/// Denies text that is not `identifier {"::" identifier}`.
pub(crate) fn check_qualified_name(text: &str) -> Result<(), ExpressionDenial> {
    if text.split("::").all(is_identifier) {
        Ok(())
    } else {
        Err(ExpressionDenial::new(ExpressionDenialDetail::Syntax(
            SyntaxDenial::InvalidIdentifier,
        )))
    }
}

/// A versioned nominal type name, such as `structure::Beam` at version 3.
///
/// Two declarations with the same qualified name but different versions are
/// different types.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExpressionTypeName {
    name: Box<str>,
    version: u32,
}

impl ExpressionTypeName {
    pub fn new(name: &str, version: u32) -> Result<Self, ExpressionDenial> {
        check_qualified_name(name)?;
        Ok(Self {
            name: name.into(),
            version,
        })
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn version(&self) -> u32 {
        self.version
    }
}

impl fmt::Display for ExpressionTypeName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}
