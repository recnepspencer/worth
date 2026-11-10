//! Retention at the exact records an authored correction anchors.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;

/// An optional field or aspect the attempt read at one record and found absent.
struct ObservedAbsence<'fact> {
    read_scope: &'fact WorthQueryOperationGraphReadScope,
    target_record: RecordRef,
    locator: &'fact AspectFieldLocator,
}

impl<'fact> ObservedAbsence<'fact> {
    fn from_decision_fact(
        fact: &'fact WorthQueryPrimaryGraphApplicationDecisionFact,
    ) -> Option<Self> {
        let WorthQueryPrimaryGraphApplicationDecisionFact::Application { read_scope, fact } = fact
        else {
            return None;
        };
        let (WorthQueryApplicationObservedFact::AbsentField {
            entity_id, locator, ..
        }
        | WorthQueryApplicationObservedFact::OptionalAspectAbsent {
            entity_id, locator, ..
        }) = fact.as_ref()
        else {
            return None;
        };
        let read_scope = read_scope.as_ref();
        projection_admits(read_scope, locator).then(|| Self {
            read_scope,
            target_record: RecordRef::Entity(*entity_id),
            locator,
        })
    }
}

/// Co-anchor declarations select exact records; loci never guess a common entity.
///
/// With `own_changes`, a live-owner declaration also keeps every other field
/// the attempt observed and changed at a demanded locus, so a member that
/// stays keeps its prior value beside the members that left. That pass is one
/// mutation-membership probe per field fact the attempt already observed and
/// one locus match per changed field; it reads no roster.
///
/// An anchor's demanded locus the attempt read as absent retains nothing, and
/// the inverse restores that absence by writing nothing there. A locus the
/// attempt did not read still refuses. An absent locus costs one probe per
/// absence the attempt observed.
pub(in super::super) fn retain_from_authored_anchors<'fact>(
    demand: &InstalledPreImageDemand,
    facts: impl Iterator<Item = &'fact WorthQueryPrimaryGraphApplicationDecisionFact>,
    mutation: &ValidatedMutationFootprint,
    anchors: &[(RecordRef, &str)],
    own_changes: bool,
) -> Result<WorthQueryRetainedPreImageSelection, WorthQueryPreImageRetentionDenial> {
    let mut candidates = Vec::new();
    let mut absences = Vec::new();
    for fact in facts {
        if let Some(candidate) = WorthQueryObservedPreImageCandidate::from_decision_fact(fact) {
            candidates.push(candidate);
        } else if let Some(absence) = ObservedAbsence::from_decision_fact(fact) {
            absences.push(absence);
        }
    }
    let mut fields = Vec::new();
    let mut total = 0usize;
    for (record, entity) in anchors {
        for locus in demand
            .loci()
            .iter()
            .filter(|locus| locus.entity() == *entity)
        {
            let mut matched = candidates.iter().filter(|candidate| {
                candidate.target_record() == record
                    && candidate_matches_locus(candidate, locus)
                    && mutation.mutates_field(record, candidate.locator())
            });
            let Some(candidate) = matched.next() else {
                if absences.iter().any(|absence| {
                    absence.target_record == *record
                        && locus_matches(absence.read_scope, absence.locator, locus)
                        && mutation.mutates_field(record, absence.locator)
                }) {
                    continue;
                }
                return Err(WorthQueryPreImageRetentionDenial::MissingDemandedField);
            };
            if matched.next().is_some() {
                return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField);
            }
            fields.push(seal_field(demand, &mut total, locus, candidate)?);
        }
    }
    if own_changes {
        let anchored: BTreeSet<&RecordRef> = anchors.iter().map(|(record, _)| record).collect();
        let mut kept = BTreeSet::new();
        for candidate in &candidates {
            let record = candidate.target_record();
            if anchored.contains(record) || !mutation.mutates_field(record, candidate.locator()) {
                continue;
            }
            let Some(locus) = demand
                .loci()
                .iter()
                .position(|locus| candidate_matches_locus(candidate, locus))
            else {
                continue;
            };
            if !kept.insert((record, locus)) {
                return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField);
            }
            fields.push(seal_field(
                demand,
                &mut total,
                &demand.loci()[locus],
                candidate,
            )?);
        }
    }
    Ok(WorthQueryRetainedPreImageSelection {
        retained: WorthQueryRetainedPreImage::from_retention_seal(WorthQueryRetainedPreImageSeal {
            fields,
            total_encoded_bytes: total,
        }),
        candidates_materialized: candidates.len(),
    })
}
