//! Native access retains its original refusal; record decoding adds its path.

use super::super::{
    HandlerExecutionDenial, WorthQueryApplicationAttemptDenial, WorthQueryValueDecodeDenial,
};
use worth_query_declaration::facade::application_schema::fragment::RecordRefusal;

#[derive(Debug)]
pub enum FragmentDenial {
    NotInstalled {
        fragment: &'static str,
    },
    MemberNotInstalled {
        entity: &'static str,
        member: &'static str,
    },
    MissingAspect {
        path: String,
    },
    ReadOnly {
        path: String,
    },
    Record(RecordRefusal),
    Encode {
        path: String,
        cause: worth_query_declaration::facade::application_schema::ApplicationValueEncodeDenial,
    },
    Resolution(super::super::WorthQueryEntityResolutionDenial),
    InvalidField {
        path: String,
    },
    Read(HandlerExecutionDenial),
    Write(WorthQueryApplicationAttemptDenial),
    DecodeControl {
        path: String,
        cause: WorthQueryValueDecodeDenial,
    },
    OneOfTransitionUnavailable,
    OneOfReadUnavailable,
}
impl std::fmt::Display for FragmentDenial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fragment access refused: {self:?}")
    }
}
impl std::error::Error for FragmentDenial {}
impl From<RecordRefusal> for FragmentDenial {
    fn from(value: RecordRefusal) -> Self {
        Self::Record(value)
    }
}
impl From<HandlerExecutionDenial> for FragmentDenial {
    fn from(value: HandlerExecutionDenial) -> Self {
        Self::Read(value)
    }
}
impl From<WorthQueryApplicationAttemptDenial> for FragmentDenial {
    fn from(value: WorthQueryApplicationAttemptDenial) -> Self {
        Self::Write(value)
    }
}
