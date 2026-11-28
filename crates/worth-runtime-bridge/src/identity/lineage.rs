use std::borrow::Cow;
use std::fmt;
#[cfg(test)]
use std::sync::Arc;

use worth_relational::facade::identity::LineageId;

use super::record::BridgeRecordIdentity;
use crate::relational_source::identity_parts::{
    fixture_position, RelationalBridgeRecordIdentityParts,
};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BridgeHistoricalResolvedLineageIdentity(LineageCoordinate);

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
enum LineageCoordinate {
    Relational(LineageId),
    #[cfg(test)]
    Fixture(Arc<str>),
}

impl BridgeHistoricalResolvedLineageIdentity {
    pub const fn relational(lineage: LineageId) -> Self {
        Self(LineageCoordinate::Relational(lineage))
    }

    pub const fn from_relational_lineage_id(lineage: u64) -> Self {
        Self::relational(LineageId(lineage))
    }

    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_lineage_id(fixture_position(label))
    }

    #[cfg(test)]
    pub(crate) fn admit_bridge_owned(label: impl Into<Arc<str>>) -> Self {
        Self(LineageCoordinate::Fixture(label.into()))
    }

    pub(crate) fn canonical_label(&self) -> Cow<'_, str> {
        match &self.0 {
            LineageCoordinate::Relational(lineage) => {
                Cow::Owned(format!("relational-lineage:{}", lineage.0))
            }
            #[cfg(test)]
            LineageCoordinate::Fixture(label) => Cow::Borrowed(label),
        }
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BridgeHistoricalResolvedRecordIdentity(BridgeRecordIdentity);

impl BridgeHistoricalResolvedRecordIdentity {
    pub const fn from_relational_record(record: RelationalBridgeRecordIdentityParts) -> Self {
        Self(BridgeRecordIdentity::Relational(record))
    }

    pub fn from_bridge_harness_label(label: impl Into<String>) -> Self {
        Self::from_relational_record(RelationalBridgeRecordIdentityParts::entity(
            1,
            fixture_position(label),
            1,
        ))
    }

    #[cfg(test)]
    pub(crate) fn admit_bridge_owned(label: impl Into<Arc<str>>) -> Self {
        Self(BridgeRecordIdentity::External(label.into()))
    }

    pub(crate) fn record(&self) -> &BridgeRecordIdentity {
        &self.0
    }

    pub(crate) fn canonical_label(&self) -> Cow<'_, str> {
        self.0.reporting_label()
    }
}

impl fmt::Debug for BridgeHistoricalResolvedRecordIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BridgeHistoricalResolvedRecordIdentity")
            .field(&"<opaque>")
            .finish()
    }
}

impl fmt::Debug for BridgeHistoricalResolvedLineageIdentity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("BridgeHistoricalResolvedLineageIdentity")
            .field(&"<opaque>")
            .finish()
    }
}
