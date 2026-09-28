/// Where one named operand of an expression declaration reads its value.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorthUiExpressionOperandSource {
    QueryScalar { projection: String },
    ApplicationBoolean { fact: String },
    ApplicationUnsigned64 { fact: String },
    ApplicationText { fact: String },
    Condition { identity: String },
    Derived { identity: String },
}

impl WorthUiExpressionOperandSource {
    /// The clause keyword that selects this source kind in a declaration.
    pub const fn clause_keyword(&self) -> &'static str {
        match self {
            Self::QueryScalar { .. } => "query-scalar",
            Self::ApplicationBoolean { .. } => "application-boolean",
            Self::ApplicationUnsigned64 { .. } => "application-unsigned64",
            Self::ApplicationText { .. } => "application-text",
            Self::Condition { .. } => "condition",
            Self::Derived { .. } => "derived",
        }
    }

    /// The projection, fact, or declaration identity the operand reads.
    pub fn reference(&self) -> &str {
        match self {
            Self::QueryScalar { projection } => projection,
            Self::ApplicationBoolean { fact }
            | Self::ApplicationUnsigned64 { fact }
            | Self::ApplicationText { fact } => fact,
            Self::Condition { identity } | Self::Derived { identity } => identity,
        }
    }

    pub(crate) fn from_clause(keyword: &str, reference: &str) -> Option<Self> {
        let reference = reference.to_owned();
        Some(match keyword {
            "query-scalar" => Self::QueryScalar {
                projection: reference,
            },
            "application-boolean" => Self::ApplicationBoolean { fact: reference },
            "application-unsigned64" => Self::ApplicationUnsigned64 { fact: reference },
            "application-text" => Self::ApplicationText { fact: reference },
            "condition" => Self::Condition {
                identity: reference,
            },
            "derived" => Self::Derived {
                identity: reference,
            },
            _ => return None,
        })
    }
}
