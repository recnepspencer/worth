use crate::canonicalization::{
    CanonicalPredicateEntry, CanonicalPredicateFamily, CanonicalPredicateOperand,
};
use crate::schema_view::ScalarAspectType;

use super::contains::NormalizedContainsPredicates;
use super::operand_access::{
    comparison_scalar, membership_values, membership_values_set, scalar_operand, string_scalar,
};
use crate::validation::{
    failure::ValidationFailureArtifact, QueryValidationCounters, QueryValidationError,
    ValidatedPredicateEntry, ValidationRejectionMatrix,
};

pub(super) type LegalPredicate = (CanonicalPredicateEntry, ScalarAspectType, &'static str);

#[derive(Default)]
pub struct FieldPredicateState {
    equality: Option<LegalPredicate>,
    strongest_gt: Option<LegalPredicate>,
    weakest_lt: Option<LegalPredicate>,
    membership: Option<LegalPredicate>,
    presence: Option<LegalPredicate>,
    contains: NormalizedContainsPredicates,
}

impl FieldPredicateState {
    pub fn ingest(
        &mut self,
        entry: LegalPredicate,
        aspect: &str,
        field: &str,
        counters: &mut QueryValidationCounters,
        rejection_matrix: &mut ValidationRejectionMatrix,
    ) -> Result<(), ValidationFailureArtifact> {
        match entry.0.family {
            CanonicalPredicateFamily::Equality => {
                if let Some(existing) = &self.equality {
                    if existing.0.operand != entry.0.operand {
                        return contradictory(
                            aspect,
                            field,
                            "conflicting-equality",
                            counters,
                            rejection_matrix,
                        );
                    }
                } else {
                    self.equality = Some(entry);
                }
            }
            CanonicalPredicateFamily::NativeGreaterThan => {
                let replace = self
                    .strongest_gt
                    .as_ref()
                    .map(|existing| comparison_scalar(&entry.0) > comparison_scalar(&existing.0))
                    .unwrap_or(true);
                if replace {
                    self.strongest_gt = Some(entry);
                }
            }
            CanonicalPredicateFamily::NativeLessThan => {
                let replace = self
                    .weakest_lt
                    .as_ref()
                    .map(|existing| comparison_scalar(&entry.0) < comparison_scalar(&existing.0))
                    .unwrap_or(true);
                if replace {
                    self.weakest_lt = Some(entry);
                }
            }
            CanonicalPredicateFamily::ScalarMembership => {
                let next_values = membership_values_set(&entry.0);
                if let Some(existing) = &self.membership {
                    let intersection = membership_values_set(&existing.0).intersect(next_values);
                    if intersection.is_empty() {
                        return contradictory(
                            aspect,
                            field,
                            "empty-membership-intersection",
                            counters,
                            rejection_matrix,
                        );
                    }
                    self.membership = Some((
                        CanonicalPredicateEntry {
                            field: existing.0.field_key().clone(),
                            family: CanonicalPredicateFamily::ScalarMembership,
                            operand: CanonicalPredicateOperand::ScalarSet(intersection),
                        },
                        existing.1,
                        existing.2,
                    ));
                } else {
                    self.membership = Some(entry);
                }
            }
            CanonicalPredicateFamily::PresenceIsPresent => {
                if self.presence.is_none() {
                    self.presence = Some(entry);
                }
            }
            CanonicalPredicateFamily::StringContains => self.contains.ingest(entry),
        }
        Ok(())
    }

    pub fn into_validated(
        self,
        aspect: &str,
        field: &str,
        counters: &mut QueryValidationCounters,
        rejection_matrix: &mut ValidationRejectionMatrix,
    ) -> Result<Vec<ValidatedPredicateEntry>, ValidationFailureArtifact> {
        if let Some(eq) = &self.equality {
            apply_equality_constraints(
                EqualityConstraints {
                    equality: eq,
                    strongest_gt: self.strongest_gt.as_ref(),
                    weakest_lt: self.weakest_lt.as_ref(),
                    membership: self.membership.as_ref(),
                    contains: self.contains.as_slice(),
                },
                aspect,
                field,
                counters,
                rejection_matrix,
            )?;
            return Ok(vec![ValidatedPredicateEntry::from_canonical(
                &eq.0, eq.1, eq.2,
            )]);
        }

        let membership = apply_range_to_membership(
            self.membership,
            self.strongest_gt.as_ref(),
            self.weakest_lt.as_ref(),
            aspect,
            field,
            counters,
            rejection_matrix,
        )?;

        let mut normalized = Vec::new();

        if let (Some(gt), Some(lt)) = (&self.strongest_gt, &self.weakest_lt)
            && comparison_scalar(&gt.0) >= comparison_scalar(&lt.0)
        {
            contradictory(aspect, field, "empty-range", counters, rejection_matrix)?;
        }

        if let Some(membership) = membership {
            let reduced = membership_values(&membership.0);
            if reduced.len() == 1 {
                let only = reduced
                    .first()
                    .expect("single-value reduced membership must have a first value")
                    .clone();
                let equality = CanonicalPredicateEntry {
                    field: membership.0.field_key().clone(),
                    family: CanonicalPredicateFamily::Equality,
                    operand: CanonicalPredicateOperand::Scalar(only),
                };
                normalized.push(ValidatedPredicateEntry::from_canonical(
                    &equality,
                    membership.1,
                    membership.2,
                ));
                return Ok(normalized);
            }
            normalized.push(ValidatedPredicateEntry::from_canonical(
                &membership.0,
                membership.1,
                membership.2,
            ));
        } else {
            if let Some(gt) = self.strongest_gt {
                normalized.push(ValidatedPredicateEntry::from_canonical(&gt.0, gt.1, gt.2));
            }
            if let Some(lt) = self.weakest_lt {
                normalized.push(ValidatedPredicateEntry::from_canonical(&lt.0, lt.1, lt.2));
            }
        }

        for contains in self.contains.into_entries() {
            normalized.push(ValidatedPredicateEntry::from_canonical(
                &contains.0,
                contains.1,
                contains.2,
            ));
        }

        if normalized.is_empty()
            && let Some(presence) = self.presence
        {
            normalized.push(ValidatedPredicateEntry::from_canonical(
                &presence.0,
                presence.1,
                presence.2,
            ));
        }

        normalized.sort();
        Ok(normalized)
    }
}

struct EqualityConstraints<'a> {
    equality: &'a LegalPredicate,
    strongest_gt: Option<&'a LegalPredicate>,
    weakest_lt: Option<&'a LegalPredicate>,
    membership: Option<&'a LegalPredicate>,
    contains: &'a [LegalPredicate],
}

fn apply_equality_constraints(
    constraints: EqualityConstraints<'_>,
    aspect: &str,
    field: &str,
    counters: &mut QueryValidationCounters,
    rejection_matrix: &mut ValidationRejectionMatrix,
) -> Result<(), ValidationFailureArtifact> {
    let equality = constraints.equality;
    if let Some(gt) = constraints.strongest_gt
        && scalar_operand(&equality.0).as_native() <= comparison_scalar(&gt.0)
    {
        return contradictory(
            aspect,
            field,
            "equality-violates-greater-than",
            counters,
            rejection_matrix,
        );
    }

    if let Some(lt) = constraints.weakest_lt
        && scalar_operand(&equality.0).as_native() >= comparison_scalar(&lt.0)
    {
        return contradictory(
            aspect,
            field,
            "equality-violates-less-than",
            counters,
            rejection_matrix,
        );
    }

    if let Some(membership) = constraints.membership
        && !membership_values_set(&membership.0).contains(scalar_operand(&equality.0))
    {
        return contradictory(
            aspect,
            field,
            "equality-outside-membership",
            counters,
            rejection_matrix,
        );
    }

    if let worth_foundational::facade::AspectValue::String(
        worth_foundational::facade::InternedString::Raw(value),
    ) = scalar_operand(&equality.0).as_native()
    {
        for contains_predicate in constraints.contains {
            if !value.contains(string_scalar(&contains_predicate.0)) {
                return contradictory(
                    aspect,
                    field,
                    "equality-violates-contains",
                    counters,
                    rejection_matrix,
                );
            }
        }
    }

    Ok(())
}

fn apply_range_to_membership(
    membership: Option<LegalPredicate>,
    strongest_gt: Option<&LegalPredicate>,
    weakest_lt: Option<&LegalPredicate>,
    aspect: &str,
    field: &str,
    counters: &mut QueryValidationCounters,
    rejection_matrix: &mut ValidationRejectionMatrix,
) -> Result<Option<LegalPredicate>, ValidationFailureArtifact> {
    let Some(membership) = membership else {
        return Ok(None);
    };

    let reduced = membership_values_set(&membership.0).filtered(|value| {
        let passes_gt = strongest_gt
            .map(|gt| value.as_native() > comparison_scalar(&gt.0))
            .unwrap_or(true);
        let passes_lt = weakest_lt
            .map(|lt| value.as_native() < comparison_scalar(&lt.0))
            .unwrap_or(true);
        passes_gt && passes_lt
    });

    if reduced.is_empty() {
        return contradictory(
            aspect,
            field,
            "range-eliminated-membership",
            counters,
            rejection_matrix,
        )
        .map(|_| None);
    }

    Ok(Some((
        CanonicalPredicateEntry {
            field: membership.0.field_key().clone(),
            family: CanonicalPredicateFamily::ScalarMembership,
            operand: CanonicalPredicateOperand::ScalarSet(reduced),
        },
        membership.1,
        membership.2,
    )))
}

fn contradictory(
    aspect: &str,
    field: &str,
    reason: &'static str,
    counters: &mut QueryValidationCounters,
    rejection_matrix: &mut ValidationRejectionMatrix,
) -> Result<(), ValidationFailureArtifact> {
    counters.record_rejection();
    rejection_matrix.record_predicate_rejection();
    Err(ValidationFailureArtifact::new(
        QueryValidationError::ContradictoryPredicateSet {
            aspect: aspect.to_string(),
            field: field.to_string(),
            reason,
        },
        counters.clone(),
        rejection_matrix.clone(),
    ))
}
