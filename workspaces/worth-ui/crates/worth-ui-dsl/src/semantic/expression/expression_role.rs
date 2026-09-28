/// The result type a derived declaration yields. `boolean` is deliberately
/// absent: a condition is the only boolean form.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthUiExpressionResultType {
    Text,
    Integer,
    Decimal,
    Token,
}

impl WorthUiExpressionResultType {
    pub const fn canonical_token(self) -> &'static str {
        match self {
            Self::Text => "text",
            Self::Integer => "integer",
            Self::Decimal => "decimal",
            Self::Token => "token",
        }
    }

    pub(crate) fn from_clause(text: &str) -> Option<Self> {
        Some(match text {
            "text" => Self::Text,
            "integer" => Self::Integer,
            "decimal" => Self::Decimal,
            "token" => Self::Token,
            _ => return None,
        })
    }
}

/// What an expression declaration is: the one boolean form, or a typed value.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthUiExpressionRole {
    Condition,
    Derived(WorthUiExpressionResultType),
}

impl WorthUiExpressionRole {
    pub fn canonical_token(self) -> String {
        match self {
            Self::Condition => "condition".to_owned(),
            Self::Derived(result) => format!("derived:{}", result.canonical_token()),
        }
    }
}
