//! Commit-owner selection of retained prior truth.

use crate::domain_computation::primary_graph::RetainedPreImageExt;
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

use super::FieldRetentionFailure;
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphApplicationDecisionFact;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WorthQueryObservedPreImageCandidate {
    read_scope: WorthQueryOperationGraphReadScope,
    value: AspectValue,
    target_record: RecordRef,
    entity_kind: KindId,
    locator: AspectFieldLocator,
}

pub struct WorthQueryRetainedPreImageFieldSeal {
    locus: InstalledPreImageLocus,
    value: AspectValue,
    encoded_bytes: usize,
    target_record: RecordRef,
    entity_kind: KindId,
    locator: AspectFieldLocator,
}

pub struct WorthQueryRetainedPreImageSeal {
    fields: Vec<WorthQueryRetainedPreImageFieldSeal>,
    absences: Vec<crate::domain_computation::application_aftermath::retained_preimage::RetainedPreImageAbsence>,
    output_fields: Option<crate::domain_computation::application_aftermath::retained_preimage::RetainedOutputFields>,
    total_encoded_bytes: usize,
}

pub struct WorthQueryRetainedPreImageSelection {
    retained: WorthQueryRetainedPreImage,
    candidates_materialized: usize,
}

pub fn retain_from_attempt<'fact>(
    demand: &InstalledPreImageDemand,
    facts: impl Iterator<Item = &'fact WorthQueryPrimaryGraphApplicationDecisionFact>,
    mutation: &ValidatedMutationFootprint,
) -> Result<WorthQueryRetainedPreImageSelection, WorthQueryPreImageRetentionDenial> {
    let facts: Vec<_> = facts.collect();
    retain_from_attempt_with_context(demand, facts.into_iter(), mutation, None)
        .map_err(|failure| failure.denial)
}

/// Retains every demanded locus the attempt changed: its observed value, or,
/// with `layout`, its authenticated absence when the schema declares that
/// field (or aspect) optional. Without `layout` an absent locus refuses.
pub(super) fn retain_from_attempt_with_context<'fact>(
    demand: &InstalledPreImageDemand,
    facts: impl Iterator<Item = &'fact WorthQueryPrimaryGraphApplicationDecisionFact> + Clone,
    mutation: &ValidatedMutationFootprint,
    layout: Option<&crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout>,
) -> Result<WorthQueryRetainedPreImageSelection, FieldRetentionFailure> {
    let candidates = facts
        .clone()
        .filter_map(WorthQueryObservedPreImageCandidate::from_decision_fact)
        .collect::<Vec<_>>();
    let absences = layout.map_or_else(Vec::new, |layout| {
        facts
            .filter_map(absence::ObservedAbsence::from_decision_fact)
            .filter(|absence| mutation.mutates_field(&absence.target_record, absence.locator))
            .filter_map(|absence| {
                let locus = demand.loci().iter().find(|locus| absence.matches(locus))?;
                absence.declared_optional(locus, layout).then_some(absence)
            })
            .collect::<Vec<_>>()
    });
    let retained = retain_matching_with_context(
        demand,
        &candidates,
        &absences,
        mutation.is_empty(),
        |candidate| mutation.mutates_field(candidate.target_record(), candidate.locator()),
    )?;
    Ok(WorthQueryRetainedPreImageSelection {
        retained,
        candidates_materialized: candidates.len() + absences.len(),
    })
}

/// One retained field under the demand's shared byte bound.
fn seal_field(
    demand: &InstalledPreImageDemand,
    total: &mut usize,
    locus: &InstalledPreImageLocus,
    candidate: &WorthQueryObservedPreImageCandidate,
) -> Result<WorthQueryRetainedPreImageFieldSeal, WorthQueryPreImageRetentionDenial> {
    let encoded_bytes = candidate.value().semantic_byte_width();
    *total = total
        .checked_add(encoded_bytes)
        .ok_or(WorthQueryPreImageRetentionDenial::ExceedsByteBound)?;
    if *total > demand.maximum_encoded_bytes() {
        return Err(WorthQueryPreImageRetentionDenial::ExceedsByteBound);
    }
    Ok(WorthQueryRetainedPreImageFieldSeal {
        locus: locus.clone(),
        value: candidate.value().clone(),
        encoded_bytes,
        target_record: candidate.target_record().clone(),
        entity_kind: candidate.entity_kind(),
        locator: candidate.locator().clone(),
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
        if !projection_admits(&read_scope, &locator) {
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

pub fn retain_matching(
    demand: &InstalledPreImageDemand,
    candidates: &[WorthQueryObservedPreImageCandidate],
    mutation_is_empty: bool,
    mutates: impl Fn(&WorthQueryObservedPreImageCandidate) -> bool,
) -> Result<WorthQueryRetainedPreImage, WorthQueryPreImageRetentionDenial> {
    retain_matching_with_context(demand, candidates, &[], mutation_is_empty, mutates)
        .map_err(|failure| failure.denial)
}

fn retain_matching_with_context(
    demand: &InstalledPreImageDemand,
    candidates: &[WorthQueryObservedPreImageCandidate],
    absences: &[absence::ObservedAbsence<'_>],
    mutation_is_empty: bool,
    mutates: impl Fn(&WorthQueryObservedPreImageCandidate) -> bool,
) -> Result<WorthQueryRetainedPreImage, FieldRetentionFailure> {
    if demand.is_empty() {
        return Err(WorthQueryPreImageRetentionDenial::EmptyDemand.into());
    }
    if demand.loci().is_empty() {
        return Ok(WorthQueryRetainedPreImage::from_retention_seal(
            WorthQueryRetainedPreImageSeal {
                fields: Vec::new(),
                absences: Vec::new(),
                output_fields: None,
                total_encoded_bytes: 0,
            },
        ));
    }
    if mutation_is_empty {
        return Err(WorthQueryPreImageRetentionDenial::NoMutatedRecord.into());
    }
    let selected = select_common_record_fields(demand.loci(), candidates, absences, mutates)?;
    let mut fields = Vec::with_capacity(demand.loci().len());
    let mut retained_absences = Vec::new();
    let mut total = 0usize;
    for (locus, observation) in demand.loci().iter().zip(selected) {
        match observation {
            Observation::Present(candidate) => {
                fields.push(seal_field(demand, &mut total, locus, candidate)?);
            }
            Observation::Absent(absence) => {
                retained_absences.push(absence.seal(demand, &mut total, locus)?);
            }
        }
    }
    Ok(WorthQueryRetainedPreImage::from_retention_seal(
        WorthQueryRetainedPreImageSeal {
            fields,
            absences: retained_absences,
            output_fields: None,
            total_encoded_bytes: total,
        },
    ))
}

fn candidate_matches_locus(
    candidate: &WorthQueryObservedPreImageCandidate,
    locus: &InstalledPreImageLocus,
) -> bool {
    locus_matches(candidate.read_scope(), candidate.locator(), locus)
}

/// A native projection read whose mask names one demanded field slot.
fn projection_admits(
    read_scope: &WorthQueryOperationGraphReadScope,
    locator: &AspectFieldLocator,
) -> bool {
    let WorthQueryOperationGraphReadScope::NativeProjection(scope) = read_scope else {
        return false;
    };
    demanded_field_slot(locator.field_path()).is_some()
        && scope.aspect() == locator.aspect().aspect_key()
        && scope
            .projection()
            .mask()
            .paths()
            .contains(locator.field_path())
}

fn locus_matches(
    read_scope: &WorthQueryOperationGraphReadScope,
    locator: &AspectFieldLocator,
    locus: &InstalledPreImageLocus,
) -> bool {
    let WorthQueryOperationGraphReadScope::NativeProjection(scope) = read_scope else {
        return false;
    };
    scope.entity().semantic_key() == locus.entity()
        && scope.aspect().as_str() == locus.aspect()
        && demanded_field_slot(locator.field_path())
            .is_some_and(|field| field.as_str() == locus.field())
}

impl WorthQueryRetainedPreImageSeal {
    pub fn into_parts(self) -> (
        Vec<WorthQueryRetainedPreImageFieldSeal>,
        Vec<crate::domain_computation::application_aftermath::retained_preimage::RetainedPreImageAbsence>,
        Option<crate::domain_computation::application_aftermath::retained_preimage::RetainedOutputFields>,
        usize,
    ){
        (
            self.fields,
            self.absences,
            self.output_fields,
            self.total_encoded_bytes,
        )
    }
}

impl WorthQueryRetainedPreImageFieldSeal {
    pub fn into_parts(
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

mod absence;
mod correlation;
use correlation::{select_common_record_fields, Observation};
pub mod anchored;
pub(super) use anchored::retain_from_authored_anchors;
pub mod output_fields;
pub(super) use output_fields::retain_output_fields;
pub mod output_members;
pub(super) use output_members::retain_output_fields_with_members;
pub mod court_doors_for_test;
