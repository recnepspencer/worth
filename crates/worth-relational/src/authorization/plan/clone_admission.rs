use std::mem::size_of;

use worth_foundational::facade::{AspectFieldLocator, AspectValue, FieldKey, InternedString};

use super::{RelationalAuthorizationEffectTarget, RelationalAuthorizationPlanDenial};
use super::{
    RelationalAuthorizationEntityAnchor, RelationalAuthorizationExactAdjacencyConstraint,
    RelationalAuthorizationFieldConstraint, RelationalAuthorizationObservationPlan,
    RelationalAuthorizationPathPlan, RelationalAuthorizationPredicate,
    RelationalAuthorizationRelatedEntityConstraint, RelationalAuthorizationTraversal,
};
use crate::identity::data::{EntityId, KindId};
use crate::snapshots::data::SnapshotHandle;

/// Refusal while preparing one owned copy of an installed authorization path.
#[derive(Debug)]
pub enum RelationalAuthorizationPathCloneStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
}

/// Plan validation remains the native owner after the caller has admitted its
/// exact temporary kind path and finite declaration walks.
#[derive(Debug)]
pub enum RelationalAuthorizationPlanAdmissionStop<Stop> {
    Admission(Stop),
    AccountingOverflow,
    Plan(RelationalAuthorizationPlanDenial),
}

type CloneResult<T, Stop> = Result<T, RelationalAuthorizationPathCloneStop<Stop>>;

impl RelationalAuthorizationPathPlan {
    /// Clone an installed path only after its initialized copy work and new
    /// backing have been admitted by the caller's cumulative request meter.
    /// The admission callback also prices each descriptor visit before the
    /// owner walks variable nested fields to determine their copy cost.
    pub fn try_clone_admitted<Stop>(
        &self,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, RelationalAuthorizationPathCloneStop<Stop>> {
        let lengths = [
            self.traversals.len(),
            self.predicates.len(),
            self.field_constraints.len(),
            self.entity_anchors.len(),
            self.related_entities.len(),
            self.exact_adjacencies.len(),
        ];
        let outer_items = lengths
            .into_iter()
            .try_fold(1_usize, usize::checked_add)
            .ok_or(RelationalAuthorizationPathCloneStop::AccountingOverflow)?;
        admit(to_u64(outer_items)?, 0).map_err(RelationalAuthorizationPathCloneStop::Admission)?;

        let mut copy_work = outer_items;
        let mut backing = 0_usize;
        add_array::<RelationalAuthorizationTraversal, Stop>(&mut backing, self.traversals.len())?;
        add_array::<RelationalAuthorizationPredicate, Stop>(&mut backing, self.predicates.len())?;
        add_array::<RelationalAuthorizationFieldConstraint, Stop>(
            &mut backing,
            self.field_constraints.len(),
        )?;
        add_array::<RelationalAuthorizationEntityAnchor, Stop>(
            &mut backing,
            self.entity_anchors.len(),
        )?;
        add_array::<RelationalAuthorizationRelatedEntityConstraint, Stop>(
            &mut backing,
            self.related_entities.len(),
        )?;
        add_array::<RelationalAuthorizationExactAdjacencyConstraint, Stop>(
            &mut backing,
            self.exact_adjacencies.len(),
        )?;
        for predicate in &self.predicates {
            add_locator(predicate.field(), &mut copy_work, &mut backing, &mut admit)?;
            let payload = value_payload_bytes(predicate.expected())
                .ok_or(RelationalAuthorizationPathCloneStop::AccountingOverflow)?;
            add(&mut copy_work, payload)?;
            add(&mut backing, payload)?;
        }
        for constraint in &self.field_constraints {
            add_locator(
                constraint.left().field(),
                &mut copy_work,
                &mut backing,
                &mut admit,
            )?;
            add_locator(
                constraint.right().field(),
                &mut copy_work,
                &mut backing,
                &mut admit,
            )?;
        }
        for exact in &self.exact_adjacencies {
            let count = exact.expected_entities().len();
            admit(to_u64(count)?, 0).map_err(RelationalAuthorizationPathCloneStop::Admission)?;
            add(&mut copy_work, count)?;
            add_array::<EntityId, Stop>(&mut backing, count)?;
        }
        admit(to_u64(copy_work)?, to_u64(backing)?)
            .map_err(RelationalAuthorizationPathCloneStop::Admission)?;
        Ok(self.clone())
    }
}

impl RelationalAuthorizationObservationPlan {
    pub(crate) fn comparison_at_admitted<Stop>(
        &self,
        snapshot: &SnapshotHandle,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, RelationalAuthorizationPlanAdmissionStop<Stop>> {
        use RelationalAuthorizationPlanAdmissionStop as Denial;

        // Fund both outer vectors and the selected branch copy before their
        // allocations. Each path and effect field then pays its nested copy.
        admit(4, 0).map_err(Denial::Admission)?;
        let path_count = self.paths.len();
        let effect_count = self.proposed_effects.len();
        let branch_bytes = snapshot.branch_id().0.len();
        let outer_bytes = path_count
            .checked_mul(size_of::<RelationalAuthorizationPathPlan>())
            .and_then(|bytes| {
                effect_count
                    .checked_mul(size_of::<RelationalAuthorizationEffectTarget>())
                    .and_then(|effects| bytes.checked_add(effects))
            })
            .ok_or(Denial::AccountingOverflow)?;
        let copy_work = outer_bytes
            .checked_add(branch_bytes)
            .and_then(|work| work.checked_add(1))
            .ok_or(Denial::AccountingOverflow)?;
        let copy_bytes = outer_bytes
            .checked_add(branch_bytes)
            .ok_or(Denial::AccountingOverflow)?;
        admit(
            u64::try_from(copy_work).map_err(|_| Denial::AccountingOverflow)?,
            u64::try_from(copy_bytes).map_err(|_| Denial::AccountingOverflow)?,
        )
        .map_err(Denial::Admission)?;
        let snapshot = snapshot.clone();
        let mut paths = Vec::with_capacity(path_count);
        for path in &self.paths {
            paths.push(
                path.try_clone_admitted(&mut admit)
                    .map_err(|stop| match stop {
                        RelationalAuthorizationPathCloneStop::Admission(stop) => {
                            Denial::Admission(stop)
                        }
                        RelationalAuthorizationPathCloneStop::AccountingOverflow => {
                            Denial::AccountingOverflow
                        }
                    })?,
            );
        }
        let mut proposed_effects = Vec::with_capacity(effect_count);
        for effect in &self.proposed_effects {
            if let Some(locator) = effect.field_locator() {
                let mut nested_work = 0;
                let mut nested_bytes = 0;
                add_locator(locator, &mut nested_work, &mut nested_bytes, &mut admit).map_err(
                    |stop| match stop {
                        RelationalAuthorizationPathCloneStop::Admission(stop) => {
                            Denial::Admission(stop)
                        }
                        RelationalAuthorizationPathCloneStop::AccountingOverflow => {
                            Denial::AccountingOverflow
                        }
                    },
                )?;
                admit(
                    u64::try_from(nested_work).map_err(|_| Denial::AccountingOverflow)?,
                    u64::try_from(nested_bytes).map_err(|_| Denial::AccountingOverflow)?,
                )
                .map_err(Denial::Admission)?;
            }
            proposed_effects.push(effect.clone());
        }
        Self::try_new_owned_admitted(
            snapshot,
            self.principal,
            self.scope,
            self.principal_kind,
            self.scope_kind,
            paths,
            proposed_effects,
            admit,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn try_new_owned_admitted<Stop>(
        snapshot: SnapshotHandle,
        principal: EntityId,
        scope: EntityId,
        principal_kind: KindId,
        scope_kind: KindId,
        paths: Vec<RelationalAuthorizationPathPlan>,
        proposed_effects: Vec<RelationalAuthorizationEffectTarget>,
        mut admit: impl FnMut(u64, u64) -> Result<(), Stop>,
    ) -> Result<Self, RelationalAuthorizationPlanAdmissionStop<Stop>> {
        use RelationalAuthorizationPlanAdmissionStop as Denial;
        admit(
            u64::try_from(paths.len()).map_err(|_| Denial::AccountingOverflow)?,
            0,
        )
        .map_err(Denial::Admission)?;
        // Validation drops each kind path before visiting the next one. Its
        // temporary backing therefore peaks at the longest path, while work
        // still includes every declaration visited below.
        let mut maximum_kind_slots = 0_usize;
        for path in &paths {
            maximum_kind_slots = maximum_kind_slots.max(
                path.traversals()
                    .len()
                    .checked_add(1)
                    .ok_or(Denial::AccountingOverflow)?,
            );
        }
        let kind_backing = maximum_kind_slots
            .checked_mul(size_of::<KindId>())
            .ok_or(Denial::AccountingOverflow)?;
        admit(
            0,
            u64::try_from(kind_backing).map_err(|_| Denial::AccountingOverflow)?,
        )
        .map_err(Denial::Admission)?;
        for path in &paths {
            let kind_slots = path
                .traversals()
                .len()
                .checked_add(1)
                .ok_or(Denial::AccountingOverflow)?;
            let constraint_visits = path
                .field_constraints()
                .len()
                .checked_mul(2)
                .ok_or(Denial::AccountingOverflow)?;
            let visits = kind_slots
                .checked_add(path.predicates().len())
                .and_then(|value| value.checked_add(constraint_visits))
                .and_then(|value| value.checked_add(path.entity_anchors().len()))
                .and_then(|value| value.checked_add(path.related_entities().len()))
                .and_then(|value| value.checked_add(path.exact_adjacencies().len()))
                .ok_or(Denial::AccountingOverflow)?;
            admit(
                u64::try_from(visits).map_err(|_| Denial::AccountingOverflow)?,
                0,
            )
            .map_err(Denial::Admission)?;
        }
        Self::try_new_owned(
            snapshot,
            principal,
            scope,
            principal_kind,
            scope_kind,
            paths,
            proposed_effects,
        )
        .map_err(Denial::Plan)
    }
}

fn add_locator<Stop>(
    locator: &AspectFieldLocator,
    work: &mut usize,
    bytes: &mut usize,
    admit: &mut impl FnMut(u64, u64) -> Result<(), Stop>,
) -> CloneResult<(), Stop> {
    let fields = locator.field_path().fields();
    let visits = fields
        .len()
        .checked_add(1)
        .ok_or(RelationalAuthorizationPathCloneStop::AccountingOverflow)?;
    admit(to_u64(visits)?, 0).map_err(RelationalAuthorizationPathCloneStop::Admission)?;
    let text = fields
        .iter()
        .try_fold(
            locator.aspect().aspect_key().as_str().len(),
            |total, field| total.checked_add(field.as_str().len()),
        )
        .ok_or(RelationalAuthorizationPathCloneStop::AccountingOverflow)?;
    add(work, text)?;
    add(work, fields.len())?;
    add(bytes, text)?;
    add_array::<FieldKey, Stop>(bytes, fields.len())
}

fn value_payload_bytes(value: &AspectValue) -> Option<usize> {
    match value {
        AspectValue::Decimal(value) => Some(value.as_str().len()),
        AspectValue::BigInt(value) => Some(value.as_str().len()),
        AspectValue::Rational(value) => value
            .numerator
            .as_str()
            .len()
            .checked_add(value.denominator.as_str().len()),
        AspectValue::String(InternedString::Raw(value)) => Some(value.len()),
        _ => Some(0),
    }
}

fn add_array<T, Stop>(target: &mut usize, count: usize) -> CloneResult<(), Stop> {
    let bytes = count
        .checked_mul(size_of::<T>())
        .ok_or(RelationalAuthorizationPathCloneStop::AccountingOverflow)?;
    add(target, bytes)
}

fn add<Stop>(target: &mut usize, amount: usize) -> CloneResult<(), Stop> {
    *target = target
        .checked_add(amount)
        .ok_or(RelationalAuthorizationPathCloneStop::AccountingOverflow)?;
    Ok(())
}

fn to_u64<Stop>(value: usize) -> CloneResult<u64, Stop> {
    u64::try_from(value).map_err(|_| RelationalAuthorizationPathCloneStop::AccountingOverflow)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::data::KindId;
    use worth_foundational::facade::{AspectKey, CanonicalFieldPath, FieldKey, LocatorAuthority};

    fn path_with_spare_source_capacity() -> RelationalAuthorizationPathPlan {
        let mut source = String::with_capacity(4096);
        source.push('v');
        let field = AspectFieldLocator::new(
            LocatorAuthority::Authoritative,
            AspectKey::new("Policy").unwrap(),
            CanonicalFieldPath::single(FieldKey::new("gate").unwrap()),
        );
        RelationalAuthorizationPathPlan::new(
            [],
            [RelationalAuthorizationPredicate::new(
                0,
                KindId::new(1),
                field,
                AspectValue::String(InternedString::Raw(source)),
            )],
        )
    }

    #[test]
    fn admitted_clone_preserves_path_without_pricing_unused_string_capacity_as_work() {
        let path = path_with_spare_source_capacity();
        let mut charged_work = 0_u64;
        let mut charged_bytes = 0_u64;
        let cloned = path
            .try_clone_admitted(|work, bytes| {
                charged_work += work;
                charged_bytes += bytes;
                Ok::<_, ()>(())
            })
            .unwrap();
        assert_eq!(cloned, path);
        assert!(charged_work < 4096);
        assert!(charged_bytes < 4096);
        assert!(charged_bytes > 0);
    }

    #[test]
    fn scratch_refusal_keeps_the_original_admission_stop() {
        let path = path_with_spare_source_capacity();
        let denied =
            path.try_clone_admitted(|_, bytes| if bytes > 0 { Err("scratch") } else { Ok(()) });
        assert!(matches!(
            denied,
            Err(RelationalAuthorizationPathCloneStop::Admission("scratch"))
        ));
    }
}
