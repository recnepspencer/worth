//! Capture selected original field truth on one authenticated output.
use super::*;
use crate::domain_computation::application_aftermath::retained_preimage::{
    RetainedOutputFields, RetainedPreImageAbsence, RetainedPreImageAbsenceKind,
};
use crate::domain_computation::primary_graph::RetainedPreImageExt;
use crate::domain_computation::primary_graph::{
    application_attempt::WorthQueryApplicationObservedFact as Fact, WorthQueryPrimaryGraphLayout,
};

/// O(L * (F + M)) for eligible loci L, admitted facts F and footprint membership M.
/// A missing changed locus is an error, never inferred original absence.
pub(in super::super) fn retain_output_fields<'a>(
    demand: &InstalledPreImageDemand,
    facts: impl Iterator<Item = &'a WorthQueryPrimaryGraphApplicationDecisionFact>,
    mutation: &ValidatedMutationFootprint,
    target: EntityId,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<WorthQueryRetainedPreImageSelection, FieldRetentionFailure> {
    let captured = capture_output_fields(demand, facts, mutation, target, layout)?;
    captured.seal(demand, Vec::new(), 0)
}

/// The output's own fields and absences, and the bytes charged for them.
pub(super) struct OutputCapture {
    fields: Vec<WorthQueryRetainedPreImageFieldSeal>,
    absences: Vec<RetainedPreImageAbsence>,
    pub(super) total: usize,
    facts: usize,
    target: EntityId,
}

/// Only loci on the output's entity are judged here. Any other demanded locus
/// belongs to a member family and is captured on its own member record.
pub(super) fn capture_output_fields<'a>(
    demand: &InstalledPreImageDemand,
    facts: impl Iterator<Item = &'a WorthQueryPrimaryGraphApplicationDecisionFact>,
    mutation: &ValidatedMutationFootprint,
    target: EntityId,
    layout: &WorthQueryPrimaryGraphLayout,
) -> Result<OutputCapture, FieldRetentionFailure> {
    use WorthQueryPreImageRetentionDenial as Denial;
    let facts: Vec<_> = facts.collect();
    let record = RecordRef::Entity(target);
    let mut fields = Vec::new();
    let mut absences = Vec::new();
    let output = demand
        .output_fields()
        .ok_or_else(|| FieldRetentionFailure::missing("output-demand", Some(&record), None))?;
    let mut total = 18usize
        .checked_add(output.role().len())
        .and_then(|n| n.checked_add(output.entity().len()))
        .ok_or(Denial::ExceedsByteBound)?;
    if total > demand.maximum_encoded_bytes() {
        return Err(Denial::ExceedsByteBound.into());
    }
    for locus in demand
        .loci()
        .iter()
        .filter(|locus| locus.entity() == output.entity())
    {
        let field = layout
            .field(locus.entity(), locus.aspect(), locus.field())
            .ok_or_else(|| {
                FieldRetentionFailure::missing("output-locus-layout", Some(&record), Some(locus))
            })?;
        if !mutation.mutates_field(&record, &field.locator) {
            if output.capture() == worth_query_declaration::facade::application_aftermath::PreImageOutputFieldCapture::CompleteRetired {
                return Err(FieldRetentionFailure::missing("retired-output-locus-not-mutated", Some(&record), Some(locus)));
            }
            continue;
        }
        let mut matched = facts.iter().filter_map(|fact| {
            let WorthQueryPrimaryGraphApplicationDecisionFact::Application { read_scope, fact } =
                *fact
            else {
                return None;
            };
            let (id, kind, locator) = match fact.as_ref() {
                Fact::Field {
                    entity_id,
                    kind,
                    locator,
                    ..
                }
                | Fact::AbsentField {
                    entity_id,
                    kind,
                    locator,
                }
                | Fact::OptionalAspectAbsent {
                    entity_id,
                    kind,
                    locator,
                } => (entity_id, kind, locator),
                _ => return None,
            };
            (*id == target
                && *kind == field.entity_kind
                && projection_admits(read_scope.as_ref(), locator)
                && locus_matches(read_scope.as_ref(), locator, locus))
            .then_some((read_scope, fact.as_ref()))
        });
        let (read_scope, fact) = matched.next().ok_or_else(|| {
            FieldRetentionFailure::missing("output-locus-observation", Some(&record), Some(locus))
        })?;
        if matched.next().is_some() {
            return Err(Denial::AmbiguousDemandedField.into());
        }
        // Sparse absence needs target/locus/presence metadata as well as values.
        let bytes = 16usize
            .checked_add(4 + 1)
            .and_then(|n| n.checked_add(locus.entity().len()))
            .and_then(|n| n.checked_add(locus.aspect().len()))
            .and_then(|n| n.checked_add(locus.field().len()))
            .ok_or(Denial::ExceedsByteBound)?;
        total = total.checked_add(bytes).ok_or(Denial::ExceedsByteBound)?;
        if total > demand.maximum_encoded_bytes() {
            return Err(Denial::ExceedsByteBound.into());
        }
        match fact {
            Fact::Field {
                entity_id,
                kind,
                locator,
                value,
            } => {
                let candidate = WorthQueryObservedPreImageCandidate::from_observed_field(
                    read_scope.as_ref().clone(),
                    locator.clone(),
                    value.clone(),
                    *entity_id,
                    *kind,
                )
                .ok_or_else(|| {
                    FieldRetentionFailure::missing(
                        "output-locus-value-capture",
                        Some(&record),
                        Some(locus),
                    )
                })?;
                fields.push(seal_field(demand, &mut total, locus, &candidate)?);
            }
            Fact::AbsentField { kind, locator, .. }
            | Fact::OptionalAspectAbsent { kind, locator, .. } => {
                absences.push(RetainedPreImageAbsence {
                    locus: locus.clone(),
                    target: record.clone(),
                    entity_kind: *kind,
                    locator: locator.clone(),
                    kind: if matches!(fact, Fact::AbsentField { .. }) {
                        RetainedPreImageAbsenceKind::Field
                    } else {
                        RetainedPreImageAbsenceKind::Aspect
                    },
                });
            }
            _ => {
                return Err(FieldRetentionFailure::missing(
                    "output-locus-fact-kind",
                    Some(&record),
                    Some(locus),
                ))
            }
        }
    }
    // An absent aspect cannot simultaneously supply a field value or field absence.
    for absent in &absences {
        if matches!(absent.kind, RetainedPreImageAbsenceKind::Aspect)
            && (fields.iter().any(|field| {
                field.locus.entity() == absent.locus.entity()
                    && field.locus.aspect() == absent.locus.aspect()
            }) || absences.iter().any(|field| {
                matches!(field.kind, RetainedPreImageAbsenceKind::Field)
                    && field.locus.entity() == absent.locus.entity()
                    && field.locus.aspect() == absent.locus.aspect()
            }))
        {
            return Err(Denial::AmbiguousDemandedField.into());
        }
    }
    Ok(OutputCapture {
        fields,
        absences,
        total,
        facts: facts.len(),
        target,
    })
}

impl OutputCapture {
    /// One image: the output's fields and absences, then any member fields
    /// charged under the same byte total.
    pub fn seal(
        mut self,
        demand: &InstalledPreImageDemand,
        members: Vec<WorthQueryRetainedPreImageFieldSeal>,
        member_facts: usize,
    ) -> Result<WorthQueryRetainedPreImageSelection, FieldRetentionFailure> {
        use WorthQueryPreImageRetentionDenial as Denial;
        self.fields.extend(members);
        if self.fields.is_empty() && self.absences.is_empty() && demand.relations().is_empty() {
            return Err(Denial::NoMutatedRecord.into());
        }
        Ok(WorthQueryRetainedPreImageSelection {
            retained: WorthQueryRetainedPreImage::from_retention_seal(
                WorthQueryRetainedPreImageSeal {
                    fields: self.fields,
                    absences: self.absences,
                    total_encoded_bytes: self.total,
                    output_fields: Some(RetainedOutputFields {
                        output: demand
                            .output_fields()
                            .ok_or_else(|| {
                                FieldRetentionFailure::missing(
                                    "output-seal-demand",
                                    Some(&RecordRef::Entity(self.target)),
                                    None,
                                )
                            })?
                            .clone(),
                        target: self.target,
                    }),
                },
            ),
            candidates_materialized: self.facts + member_facts,
        })
    }
}
