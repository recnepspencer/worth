//! Commit-owner selection of retained prior truth.

#[cfg(test)]
mod tests;

use std::collections::BTreeSet;

use worth_foundational::facade::{AspectFieldLocator, AspectValue};
use worth_query_installation::facade::{
    InstalledPreImageDemand, InstalledPreImageLocus, WorthQueryOperationGraphReadScope,
};
use worth_relational::facade::identity::{EntityId, KindId};
use worth_relational::facade::mvcc::ValidatedMutationFootprint;
use worth_relational::facade::transactions::RecordRef;

use crate::domain_computation::application_aftermath::{
    demanded_field_slot, WorthQueryPreImageRetentionDenial, WorthQueryRetainedPreImage,
};

use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact;

#[derive(Clone, Debug, Eq, PartialEq)]
struct WorthQueryObservedPreImageCandidate {
    read_scope: WorthQueryOperationGraphReadScope,
    value: AspectValue,
    target_record: RecordRef,
    entity_kind: KindId,
    locator: AspectFieldLocator,
}

pub(crate) struct WorthQueryRetainedPreImageFieldSeal {
    locus: InstalledPreImageLocus,
    value: AspectValue,
    encoded_bytes: usize,
    target_record: RecordRef,
    entity_kind: KindId,
    locator: AspectFieldLocator,
}

pub(crate) struct WorthQueryRetainedPreImageSeal {
    fields: Vec<WorthQueryRetainedPreImageFieldSeal>,
    total_encoded_bytes: usize,
}

pub(super) struct WorthQueryRetainedPreImageSelection {
    retained: WorthQueryRetainedPreImage,
    candidates_materialized: usize,
}

pub(super) fn retain_from_attempt<'fact>(
    demand: &InstalledPreImageDemand,
    facts: impl Iterator<Item = &'fact WorthQueryPrimaryGraphApplicationDecisionFact>,
    mutation: &ValidatedMutationFootprint,
) -> Result<WorthQueryRetainedPreImageSelection, WorthQueryPreImageRetentionDenial> {
    let candidates = facts
        .filter_map(WorthQueryObservedPreImageCandidate::from_decision_fact)
        .collect::<Vec<_>>();
    let retained = retain_matching(demand, &candidates, mutation.is_empty(), |candidate| {
        mutation.mutates_field(candidate.target_record(), candidate.locator())
    })?;
    Ok(WorthQueryRetainedPreImageSelection {
        retained,
        candidates_materialized: candidates.len(),
    })
}

impl WorthQueryRetainedPreImageSelection {
    pub(super) fn into_parts(self) -> (WorthQueryRetainedPreImage, usize) {
        (self.retained, self.candidates_materialized)
    }
}

impl WorthQueryObservedPreImageCandidate {
    fn from_decision_fact(fact: &WorthQueryPrimaryGraphApplicationDecisionFact) -> Option<Self> {
        use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

        let WorthQueryPrimaryGraphApplicationDecisionFact::Application { read_scope, fact } = fact
        else {
            return None;
        };
        let WorthQueryApplicationObservedFact::Field {
            locator,
            value,
            entity_id,
            kind,
        } = fact.as_ref()
        else {
            return None;
        };
        Self::from_observed_field(
            read_scope.as_ref().clone(),
            locator.clone(),
            value.clone(),
            *entity_id,
            *kind,
        )
    }

    fn from_observed_field(
        read_scope: WorthQueryOperationGraphReadScope,
        locator: AspectFieldLocator,
        value: AspectValue,
        entity_id: EntityId,
        entity_kind: KindId,
    ) -> Option<Self> {
        demanded_field_slot(locator.field_path())?;
        let WorthQueryOperationGraphReadScope::NativeProjection(scope) = &read_scope else {
            return None;
        };
        if scope.aspect() != locator.aspect().aspect_key()
            || !scope
                .projection()
                .mask()
                .paths()
                .contains(locator.field_path())
        {
            return None;
        }
        Some(Self {
            read_scope,
            value,
            target_record: RecordRef::Entity(entity_id),
            entity_kind,
            locator,
        })
    }

    const fn read_scope(&self) -> &WorthQueryOperationGraphReadScope {
        &self.read_scope
    }

    const fn value(&self) -> &AspectValue {
        &self.value
    }

    const fn target_record(&self) -> &RecordRef {
        &self.target_record
    }

    const fn entity_kind(&self) -> KindId {
        self.entity_kind
    }

    const fn locator(&self) -> &AspectFieldLocator {
        &self.locator
    }
}

fn retain_matching(
    demand: &InstalledPreImageDemand,
    candidates: &[WorthQueryObservedPreImageCandidate],
    mutation_is_empty: bool,
    mutates: impl Fn(&WorthQueryObservedPreImageCandidate) -> bool,
) -> Result<WorthQueryRetainedPreImage, WorthQueryPreImageRetentionDenial> {
    if demand.loci().is_empty() {
        return Err(WorthQueryPreImageRetentionDenial::EmptyDemand);
    }
    if mutation_is_empty {
        return Err(WorthQueryPreImageRetentionDenial::NoMutatedRecord);
    }
    let selected = select_common_record_fields(demand.loci(), candidates, mutates)?;
    let mut fields = Vec::with_capacity(demand.loci().len());
    let mut total = 0usize;
    for (locus, candidate) in demand.loci().iter().zip(selected) {
        let encoded_bytes = candidate.value().semantic_byte_width();
        total = total
            .checked_add(encoded_bytes)
            .ok_or(WorthQueryPreImageRetentionDenial::ExceedsByteBound)?;
        if total > demand.maximum_encoded_bytes() {
            return Err(WorthQueryPreImageRetentionDenial::ExceedsByteBound);
        }
        fields.push(WorthQueryRetainedPreImageFieldSeal {
            locus: locus.clone(),
            value: candidate.value().clone(),
            encoded_bytes,
            target_record: candidate.target_record().clone(),
            entity_kind: candidate.entity_kind(),
            locator: candidate.locator().clone(),
        });
    }
    Ok(WorthQueryRetainedPreImage::from_retention_seal(
        WorthQueryRetainedPreImageSeal {
            fields,
            total_encoded_bytes: total,
        },
    ))
}

/// Correlation costs O(L * C * (M + log C)) time and O(C + L) scratch for
/// L distinct demanded loci, C candidates and mutation-membership cost M.
/// Record sets establish common identity only; observations remain distinct.
fn select_common_record_fields<'candidate>(
    loci: &[InstalledPreImageLocus],
    candidates: &'candidate [WorthQueryObservedPreImageCandidate],
    mutates: impl Fn(&WorthQueryObservedPreImageCandidate) -> bool,
) -> Result<Vec<&'candidate WorthQueryObservedPreImageCandidate>, WorthQueryPreImageRetentionDenial>
{
    let mut admitted_by_locus = Vec::with_capacity(loci.len());
    let mut common_records: Option<BTreeSet<&RecordRef>> = None;
    for locus in loci {
        let admitted: Vec<_> = candidates
            .iter()
            .filter(|candidate| candidate_matches_locus(candidate, locus) && mutates(candidate))
            .collect();
        if admitted.is_empty() {
            return Err(WorthQueryPreImageRetentionDenial::MissingDemandedField);
        }
        let records: BTreeSet<_> = admitted
            .iter()
            .map(|candidate| candidate.target_record())
            .collect();
        if let Some(common) = &mut common_records {
            common.retain(|record| records.contains(record));
        } else {
            common_records = Some(records);
        }
        admitted_by_locus.push(admitted);
    }
    let mut common = common_records.into_iter().flatten();
    let Some(record) = common.next() else {
        return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField);
    };
    if common.next().is_some() {
        return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField);
    }
    admitted_by_locus
        .into_iter()
        .map(|admitted| {
            let mut matching = admitted
                .into_iter()
                .filter(|candidate| candidate.target_record() == record);
            let candidate = matching
                .next()
                .ok_or(WorthQueryPreImageRetentionDenial::MissingDemandedField)?;
            if matching.next().is_some() {
                return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField);
            }
            Ok(candidate)
        })
        .collect()
}

fn candidate_matches_locus(
    candidate: &WorthQueryObservedPreImageCandidate,
    locus: &InstalledPreImageLocus,
) -> bool {
    let WorthQueryOperationGraphReadScope::NativeProjection(scope) = candidate.read_scope() else {
        return false;
    };
    scope.entity().semantic_key() == locus.entity()
        && scope.aspect().as_str() == locus.aspect()
        && demanded_field_slot(candidate.locator().field_path())
            .is_some_and(|field| field.as_str() == locus.field())
}

impl WorthQueryRetainedPreImageSeal {
    pub(crate) fn into_parts(self) -> (Vec<WorthQueryRetainedPreImageFieldSeal>, usize) {
        (self.fields, self.total_encoded_bytes)
    }
}

impl WorthQueryRetainedPreImageFieldSeal {
    pub(crate) fn into_parts(
        self,
    ) -> (
        InstalledPreImageLocus,
        AspectValue,
        usize,
        RecordRef,
        KindId,
        AspectFieldLocator,
    ) {
        (
            self.locus,
            self.value,
            self.encoded_bytes,
            self.target_record,
            self.entity_kind,
            self.locator,
        )
    }
}
