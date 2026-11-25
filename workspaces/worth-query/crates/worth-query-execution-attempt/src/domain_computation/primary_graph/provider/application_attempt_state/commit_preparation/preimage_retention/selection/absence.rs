//! An optional field or aspect the attempt read at one record and found absent.
use super::*;
use crate::domain_computation::application_aftermath::retained_preimage::{
    RetainedPreImageAbsence, RetainedPreImageAbsenceKind,
};
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact, WorthQueryPrimaryGraphLayout,
};

pub(super) struct ObservedAbsence<'fact> {
    pub(super) read_scope: &'fact WorthQueryOperationGraphReadScope,
    pub(super) target_record: RecordRef,
    entity_kind: KindId,
    pub(super) locator: &'fact AspectFieldLocator,
    kind: RetainedPreImageAbsenceKind,
}

impl<'fact> ObservedAbsence<'fact> {
    pub(super) fn from_decision_fact(
        fact: &'fact WorthQueryPrimaryGraphApplicationDecisionFact,
    ) -> Option<Self> {
        let WorthQueryPrimaryGraphApplicationDecisionFact::Application { read_scope, fact } = fact
        else {
            return None;
        };
        let (entity_id, kind, locator, absence) = match fact.as_ref() {
            Fact::AbsentField {
                entity_id,
                kind,
                locator,
            } => (entity_id, kind, locator, RetainedPreImageAbsenceKind::Field),
            Fact::OptionalAspectAbsent {
                entity_id,
                kind,
                locator,
            } => (
                entity_id,
                kind,
                locator,
                RetainedPreImageAbsenceKind::Aspect,
            ),
            _ => return None,
        };
        let read_scope = read_scope.as_ref();
        projection_admits(read_scope, locator).then_some(Self {
            read_scope,
            target_record: RecordRef::Entity(*entity_id),
            entity_kind: *kind,
            locator,
            kind: absence,
        })
    }

    /// Whether the installed schema declares this absent field (or aspect)
    /// optional at `locus`. A required field read as absent never becomes
    /// retained prior truth.
    pub(super) fn declared_optional(
        &self,
        locus: &InstalledPreImageLocus,
        layout: &WorthQueryPrimaryGraphLayout,
    ) -> bool {
        match self.kind {
            RetainedPreImageAbsenceKind::Field => {
                layout.field_is_optional(locus.entity(), self.locator)
            }
            RetainedPreImageAbsenceKind::Aspect => {
                layout.aspect_is_optional(locus.entity(), self.locator)
            }
        }
    }

    pub(super) fn matches(&self, locus: &InstalledPreImageLocus) -> bool {
        locus_matches(self.read_scope, self.locator, locus)
    }

    /// Seals this absence under the demand's shared byte bound. Sparse
    /// absence charges its target, locus and presence metadata.
    pub(super) fn seal(
        &self,
        demand: &InstalledPreImageDemand,
        total: &mut usize,
        locus: &InstalledPreImageLocus,
    ) -> Result<RetainedPreImageAbsence, WorthQueryPreImageRetentionDenial> {
        let bytes = (16usize + 4 + 1)
            .checked_add(locus.entity().len())
            .and_then(|n| n.checked_add(locus.aspect().len()))
            .and_then(|n| n.checked_add(locus.field().len()))
            .ok_or(WorthQueryPreImageRetentionDenial::ExceedsByteBound)?;
        *total = total
            .checked_add(bytes)
            .ok_or(WorthQueryPreImageRetentionDenial::ExceedsByteBound)?;
        if *total > demand.maximum_encoded_bytes() {
            return Err(WorthQueryPreImageRetentionDenial::ExceedsByteBound);
        }
        Ok(RetainedPreImageAbsence {
            locus: locus.clone(),
            target: self.target_record.clone(),
            entity_kind: self.entity_kind,
            locator: self.locator.clone(),
            kind: self.kind,
        })
    }
}
