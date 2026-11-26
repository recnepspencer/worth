//! Correlates every demanded locus to one common changed record.
use super::*;

/// One demanded locus's original truth at the selected record.
pub(super) enum Observation<'a, 'fact> {
    Present(&'a WorthQueryObservedPreImageCandidate),
    Absent(&'a absence::ObservedAbsence<'fact>),
}

impl Observation<'_, '_> {
    pub(super) fn target_record(&self) -> &RecordRef {
        match self {
            Self::Present(candidate) => candidate.target_record(),
            Self::Absent(absence) => &absence.target_record,
        }
    }
}

/// Correlation costs O(L * (C + A) * (M + log C)) time and O(C + A + L)
/// scratch for L distinct demanded loci, C candidates, A admitted optional
/// absences and mutation-membership cost M. Record sets establish common
/// identity only; observations remain distinct.
pub(super) fn select_common_record_fields<'a, 'fact>(
    loci: &[InstalledPreImageLocus],
    candidates: &'a [WorthQueryObservedPreImageCandidate],
    absences: &'a [absence::ObservedAbsence<'fact>],
    mutates: impl Fn(&WorthQueryObservedPreImageCandidate) -> bool,
) -> Result<Vec<Observation<'a, 'fact>>, FieldRetentionFailure> {
    let mut admitted_by_locus = Vec::with_capacity(loci.len());
    let mut common_records: Option<BTreeSet<RecordRef>> = None;
    for locus in loci {
        let admitted: Vec<_> = candidates
            .iter()
            .filter(|candidate| candidate_matches_locus(candidate, locus) && mutates(candidate))
            .map(Observation::Present)
            .chain(
                absences
                    .iter()
                    .filter(|absence| absence.matches(locus))
                    .map(Observation::Absent),
            )
            .collect();
        if admitted.is_empty() {
            return Err(FieldRetentionFailure::missing(
                "unobserved-mutated-locus",
                None,
                Some(locus),
            ));
        }
        let records: BTreeSet<_> = admitted
            .iter()
            .map(|observation| observation.target_record().clone())
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
        return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField.into());
    };
    if common.next().is_some() {
        return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField.into());
    }
    admitted_by_locus
        .into_iter()
        .zip(loci)
        .map(|(admitted, locus)| {
            let mut matching = admitted
                .into_iter()
                .filter(|observation| observation.target_record() == &record);
            let observation = matching.next().ok_or_else(|| {
                FieldRetentionFailure::missing("selected-record-field", Some(&record), Some(locus))
            })?;
            if matching.next().is_some() {
                return Err(WorthQueryPreImageRetentionDenial::AmbiguousDemandedField.into());
            }
            Ok(observation)
        })
        .collect()
}
