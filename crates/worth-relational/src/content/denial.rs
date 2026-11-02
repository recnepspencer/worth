use crate::execution::RelationalExecutionDenialCause;
#[cfg(test)]
mod tests;

// One semantic vocabulary feeds both admission and nonrecursive cause carriage.
// The latter cannot contain Execution, which already carries content failures.
macro_rules! content_rejections {
    ($($variant:ident $({ $($field:ident: $kind:ty),* $(,)? })?),* $(,)?) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
        pub enum RelationalContentRejection {
            $($variant $({ $($field: $kind),* })?),*
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum RelationalContentDenial {
            $($variant $({ $($field: $kind),* })?),*,
            Execution(RelationalExecutionDenialCause),
        }

        impl RelationalContentDenial {
            pub(crate) fn into_execution_cause(self) -> RelationalExecutionDenialCause {
                match self {
                    Self::Execution(cause) => cause,
                    $(Self::$variant $({ $($field),* })? =>
                        RelationalExecutionDenialCause::Content(
                            RelationalContentRejection::$variant $({ $($field),* })?
                        )),*
                }
            }
        }

        impl From<RelationalContentRejection> for RelationalContentDenial {
            fn from(cause: RelationalContentRejection) -> Self {
                match cause {
                    $(RelationalContentRejection::$variant $({ $($field),* })? =>
                        Self::$variant $({ $($field),* })?),*
                }
            }
        }
    };
}

content_rejections! {
    MissingContent,
    BackingUnavailable {
        identity: worth_foundational::facade::ContentRefId,
        cause: crate::storage::data::RelationalStorageUnavailableCause,
    },
    FormatBindingMissing,
    FormatMismatch,
    UnsupportedVersion,
    InvalidFormat,
    NonCanonicalContent,
    IntegrityMismatch,
    ContentIdentityConflict,
    EncodingInconsistency,
    InputTypeMismatch,
    DuplicateContentFormat,
    SizeLimit {
        observed: usize,
        maximum: usize,
    },
    ArithmeticOverflow,
    HistoryNotResident,
}

impl From<RelationalExecutionDenialCause> for RelationalContentDenial {
    fn from(cause: RelationalExecutionDenialCause) -> Self {
        Self::Execution(cause)
    }
}

impl std::fmt::Display for RelationalContentDenial {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "Native content admission refused: {self:?}")
    }
}
impl std::error::Error for RelationalContentDenial {}
