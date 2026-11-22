//! Retention at the exact records an authored correction anchors.
use super::*;
use crate::domain_computation::primary_graph::application_attempt::WorthQueryApplicationObservedFact;
use crate::domain_computation::primary_graph::RetainedPreImageExt;

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
        projection_admits(read_scope, locator).then_some(Self {
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
) -> Result<WorthQueryRetainedPreImageSelection, FieldRetentionFailure> {
    let mut total = 0usize;
    let anchored = AnchoredFields {
        demand,
        mutation,
        anchors,
        own_changes,
        admits: &|_| true,
        seal: seal_field,
    };
    let (fields, candidates_materialized) = anchored.retain(facts, &mut total)?;
    Ok(WorthQueryRetainedPreImageSelection {
        retained: WorthQueryRetainedPreImage::from_retention_seal(WorthQueryRetainedPreImageSeal {
            fields,
            absences: Vec::new(),
            output_fields: None,
            total_encoded_bytes: total,
        }),
        candidates_materialized,
    })
}

type Seal = fn(
    &InstalledPreImageDemand,
    &mut usize,
    &InstalledPreImageLocus,
    &WorthQueryObservedPreImageCandidate,
) -> Result<WorthQueryRetainedPreImageFieldSeal, WorthQueryPreImageRetentionDenial>;

/// Anchored retention over the demanded loci `admits` selects, charging each
/// retained field through `seal` against the caller's running total.
pub(super) struct AnchoredFields<'a> {
    pub(super) demand: &'a InstalledPreImageDemand,
    pub(super) mutation: &'a ValidatedMutationFootprint,
    pub(super) anchors: &'a [(RecordRef, &'a str)],
    pub(super) own_changes: bool,
    pub(super) admits: &'a dyn Fn(&InstalledPreImageLocus) -> bool,
    pub(super) seal: Seal,
}

impl AnchoredFields<'_> {
    pub(super) fn retain<'fact>(
        &self,
        facts: impl Iterator<Item = &'fact WorthQueryPrimaryGraphApplicationDecisionFact>,
        total: &mut usize,
    ) -> Result<(Vec<WorthQueryRetainedPreImageFieldSeal>, usize), FieldRetentionFailure> {
        let (demand, mutation) = (self.demand, self.mutation);
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
        for (record, entity) in self.anchors {
            for locus in demand
                .loci()
                .iter()
                .filter(|locus| locus.entity() == *entity && (self.admits)(locus))
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
                    return Err(FieldRetentionFailure::missing(
                        "anchored-locus-observation",
                        Some(record),
                        Some(locus),
                    ));
                };
                if matched.next().is_some() {
                    return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField.into());
                }
                fields.push((self.seal)(demand, total, locus, candidate)?);
            }
        }
        if self.own_changes {
            let anchored: BTreeSet<&RecordRef> =
                self.anchors.iter().map(|(record, _)| record).collect();
            let mut kept = BTreeSet::new();
            for candidate in &candidates {
                let record = candidate.target_record();
                if anchored.contains(record) || !mutation.mutates_field(record, candidate.locator())
                {
                    continue;
                }
                let Some(locus) = demand.loci().iter().position(|locus| {
                    (self.admits)(locus) && candidate_matches_locus(candidate, locus)
                }) else {
                    continue;
                };
                if !kept.insert((record, locus)) {
                    return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField.into());
                }
                fields.push((self.seal)(
                    demand,
                    total,
                    &demand.loci()[locus],
                    candidate,
                )?);
            }
        }
        Ok((fields, candidates.len()))
    }
}
