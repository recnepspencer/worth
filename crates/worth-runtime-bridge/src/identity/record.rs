use std::borrow::Cow;
use std::sync::Arc;

use crate::relational_source::identity_parts::RelationalBridgeRecordIdentityParts;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum BridgeRecordIdentity {
    Relational(RelationalBridgeRecordIdentityParts),
    External(Arc<str>),
}

impl BridgeRecordIdentity {
    pub(crate) fn relational(&self) -> Option<RelationalBridgeRecordIdentityParts> {
        match self {
            Self::Relational(record) => Some(*record),
            Self::External(_) => None,
        }
    }

    pub(crate) fn reporting_label(&self) -> Cow<'_, str> {
        match self {
            Self::Relational(record) => Cow::Owned(record.terminal_projection_for_reporting()),
            Self::External(label) => Cow::Borrowed(label),
        }
    }

    pub(crate) fn coordinate_domain(&self) -> &'static str {
        match self {
            Self::Relational(_) => "relational",
            Self::External(_) => "external",
        }
    }
}

#[cfg(test)]
impl From<&str> for BridgeRecordIdentity {
    fn from(label: &str) -> Self {
        Self::External(Arc::from(label))
    }
}
