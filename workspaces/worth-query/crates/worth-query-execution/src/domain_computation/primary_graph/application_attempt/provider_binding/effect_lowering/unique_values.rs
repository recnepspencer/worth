//! One law for every program write of a unique field: the write lowers only
//! when the sealed facts hold the field's indexed selection of that value,
//! and that selection found no live entity but the one written. A program
//! writes each unique value at most once.

use std::collections::BTreeSet;

use worth_foundational::facade::{
    prepare_aspect_value_identity_basis, AspectFieldLocator, AspectValue,
    CanonicalAspectValueIdentityBasis,
};
use worth_relational::facade::identity::{EntityId, KindId};

use super::ObservedFactIndex;
use crate::domain_computation::primary_graph::application_attempt::{
    WorthQueryApplicationAttemptDenial, WorthQueryApplicationAttemptDenialKind,
};
use crate::domain_computation::primary_graph::schema_layout::{
    WorthQueryUniqueFieldIndex, WorthQueryUniqueFields,
};

pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) struct UniqueValueWrites<
    'layout,
> {
    fields: WorthQueryUniqueFields<'layout>,
    written: BTreeSet<(
        KindId,
        AspectFieldLocator,
        CanonicalAspectValueIdentityBasis,
    )>,
}

impl<'layout> UniqueValueWrites<'layout> {
    pub(in crate::domain_computation::primary_graph::application_attempt::provider_binding) fn new(
        fields: WorthQueryUniqueFields<'layout>,
    ) -> Self {
        Self {
            fields,
            written: BTreeSet::new(),
        }
    }

    /// Admits `writer` (none for a create) setting `locator` to `value`.
    pub(super) fn admit(
        &mut self,
        facts: &ObservedFactIndex<'_>,
        kind: KindId,
        writer: Option<EntityId>,
        locator: &AspectFieldLocator,
        value: &AspectValue,
    ) -> Result<(), WorthQueryApplicationAttemptDenial> {
        let index_id = match self.fields.index(kind, locator) {
            None => return Ok(()),
            Some(WorthQueryUniqueFieldIndex::Unavailable) => {
                return Err(denial(
                    WorthQueryApplicationAttemptDenialKind::UniqueIndexUnavailable,
                    locator,
                ))
            }
            Some(WorthQueryUniqueFieldIndex::Installed(index_id)) => index_id,
        };
        let candidates = facts.selection_candidates(index_id, kind, locator, value)?;
        if candidates
            .iter()
            .any(|candidate| Some(*candidate) != writer)
        {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UniqueValueTaken,
                locator,
            ));
        }
        if !self.written.insert((
            kind,
            locator.clone(),
            prepare_aspect_value_identity_basis(value),
        )) {
            return Err(denial(
                WorthQueryApplicationAttemptDenialKind::UniqueValueTaken,
                locator,
            ));
        }
        Ok(())
    }
}

fn denial(
    kind: WorthQueryApplicationAttemptDenialKind,
    locator: &AspectFieldLocator,
) -> WorthQueryApplicationAttemptDenial {
    WorthQueryApplicationAttemptDenial::new(kind, format!("{locator:?}"))
}
