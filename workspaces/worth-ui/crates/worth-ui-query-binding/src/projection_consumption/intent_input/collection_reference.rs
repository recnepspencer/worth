use std::sync::Arc;

use worth_query::facade::runtime::WorthQueryEvidenceIdentity;

use super::UiProjectionInputRevision;
use crate::UiCollectionProjectionRowReference;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiProjectionOptionReference {
    owner_revision: UiProjectionInputRevision,
    query_row_identity: Arc<WorthQueryEvidenceIdentity>,
    application_item_key: Option<core::num::NonZeroU64>,
}

/// Opaque stable row correlation admitted for UI-owned selection state. It
/// carries no Query cursor or progression authority.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct UiProjectionOptionStableKey(worth_query::facade::runtime::WorthQueryEvidenceIdentityKey);

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiProjectionInputCollectionRow {
    row: UiCollectionProjectionRowReference,
    selected_values: Box<[Arc<str>]>,
    application_item_key: Option<core::num::NonZeroU64>,
}

impl UiProjectionOptionReference {
    pub(super) fn query_issued(
        owner_revision: UiProjectionInputRevision,
        query_row_identity: WorthQueryEvidenceIdentity,
        application_item_key: Option<core::num::NonZeroU64>,
    ) -> Self {
        Self {
            owner_revision,
            query_row_identity: Arc::new(query_row_identity),
            application_item_key,
        }
    }

    pub fn owner_revision(&self) -> &UiProjectionInputRevision {
        &self.owner_revision
    }

    pub fn query_identity(&self) -> &WorthQueryEvidenceIdentity {
        &self.query_row_identity
    }

    pub fn stable_key(&self) -> UiProjectionOptionStableKey {
        UiProjectionOptionStableKey::from_query_identity(&self.query_row_identity)
    }

    pub fn application_item_key(&self) -> Option<core::num::NonZeroU64> {
        self.application_item_key
    }

    /// Carries the original row correlation for current-catalog lookup. This
    /// reference carries no revision or Selection authority of its own.
    pub fn row_reference(&self) -> UiCollectionProjectionRowReference {
        UiCollectionProjectionRowReference::query_issued(self.query_row_identity.as_ref().clone())
    }
}

impl UiProjectionOptionStableKey {
    pub(in crate::projection_consumption) fn from_query_identity(
        identity: &WorthQueryEvidenceIdentity,
    ) -> Self {
        Self(identity.operational_key())
    }
}

impl UiProjectionInputCollectionRow {
    pub(super) fn query_issued(
        row: UiCollectionProjectionRowReference,
        selected_values: Box<[Arc<str>]>,
        application_item_key: Option<core::num::NonZeroU64>,
    ) -> Self {
        Self {
            row,
            selected_values,
            application_item_key,
        }
    }

    pub fn row(&self) -> &UiCollectionProjectionRowReference {
        &self.row
    }

    pub fn selected_values(&self) -> &[Arc<str>] {
        &self.selected_values
    }

    pub fn application_item_key(&self) -> Option<core::num::NonZeroU64> {
        self.application_item_key
    }
}
