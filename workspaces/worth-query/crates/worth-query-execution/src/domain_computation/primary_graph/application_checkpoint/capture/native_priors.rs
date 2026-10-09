//! Keep authenticated cache payloads only at their selected native family head.
use std::{
    any::TypeId,
    collections::{BTreeMap, BTreeSet},
};

use super::super::super::{
    application_output_demand::{
        AcceptedCheckpointFactSource, CheckpointOutputSlot,
        WorthQueryAcceptedOutputCheckpointIdentity as Identity,
    },
    output_lineage::{
        invalidation::InvalidationEditAdmission, tree_insert_bytes, tree_work,
        CheckpointPriorSelectionDenial as SelectionDenial, NativePriorCheckpointOutput,
        WorthQueryApplicationOutputLineage,
    },
};
use super::super::WorthQueryCheckpointCaptureDenial;

type Accepted = (
    Identity,
    Option<AcceptedCheckpointFactSource>,
    Option<TypeId>,
);

pub(in crate::domain_computation::primary_graph) fn merge(
    lineage: &WorthQueryApplicationOutputLineage,
    current: Vec<Accepted>,
    recovered: impl IntoIterator<Item = (Identity, Option<TypeId>)>,
    priors: Vec<NativePriorCheckpointOutput<'_>>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Vec<(Identity, Option<AcceptedCheckpointFactSource>)>, WorthQueryCheckpointCaptureDenial>
{
    let mut heads = BTreeMap::new();
    for prior in &priors {
        let key = (
            prior.family_role.family.as_str(),
            prior.family_role.role.as_str(),
            prior.scope,
            prior.partition,
            prior.entity,
        );
        admit_insert(admission, heads.len(), &(key, prior))?;
        if heads.insert(key, prior).is_some() {
            return Err(capture_denial("checkpoint output family binding is ambiguous").into());
        }
    }
    let mut retain = |rows: Vec<Accepted>,
                      allow_missing: bool|
     -> Result<Vec<_>, WorthQueryCheckpointCaptureDenial> {
        let mut kept = Vec::new();
        for (identity, source, binding) in rows {
            let Some((family, role_name)) = lineage
                .checkpoint_family_role(binding, admission)
                .map_err(|denial| denial.into_capture_denial())?
            else {
                kept.push((identity, source));
                continue;
            };
            let mut entity = None;
            for role in &identity.roles {
                admission.charge_external_work(1).map_err(|stop| {
                    SelectionDenial::Admission {
                        phase: "accepted output role visit",
                        stop,
                    }
                    .into_capture_denial()
                })?;
                if role.role == role_name {
                    entity = Some(role.entity);
                    break;
                }
            }
            let key = (
                family,
                role_name,
                identity.scope,
                Some(identity.source_partition),
                entity,
            );
            admit_lookup(admission, heads.len())?;
            if heads.get(&key).map_or(allow_missing, |head| {
                binding == Some(head.binding)
                    && identity.idempotency_key == head.idempotency_key
                    && head.identity.as_ref().is_none_or(|native| {
                        identity.producer == native.producer && identity.source == native.source
                    })
            }) {
                kept.push((identity, source));
            }
        }
        Ok(kept)
    };
    let current = retain(current, false)?;
    let recovered = retain(
        recovered
            .into_iter()
            .map(|(identity, binding)| (identity, None, binding))
            .collect(),
        true,
    )?;
    drop(heads);
    let mut accepted = merge_rows(current, recovered, admission)?;
    let accepted_slots = slot_index(&accepted, admission)?;
    let mut native = Vec::new();
    for prior in priors {
        // These are exactly the slots merge_accepted_outputs would discard.
        // Acceptance was checked against the authoritative head above; having
        // a cache row alone never authorizes this omission.
        if let (Some(locator), Some(partition)) = (&prior.identity, prior.partition) {
            admit_lookup(admission, accepted_slots.len())?;
            if accepted_slots.contains(&CheckpointOutputSlot::new(
                locator.producer,
                prior.scope,
                partition,
            )) {
                continue;
            }
        }
        if let Some(identity) = prior
            .materialize_identity(admission)
            .map_err(|denial| denial.into_capture_denial())?
        {
            native.push((identity, None));
        }
    }
    // Locator-only rows contain no producer facts or witness. They may choose
    // Preserve after reopening, but cannot establish Ready.
    drop(accepted_slots);
    accepted.extend(native);
    Ok(accepted)
}

fn merge_rows(
    mut current: Vec<(Identity, Option<AcceptedCheckpointFactSource>)>,
    recovered: Vec<(Identity, Option<AcceptedCheckpointFactSource>)>,
    admission: &mut InvalidationEditAdmission,
) -> Result<Vec<(Identity, Option<AcceptedCheckpointFactSource>)>, WorthQueryCheckpointCaptureDenial>
{
    let slots = slot_index(&current, admission)?;
    let mut unshadowed = Vec::new();
    for row in recovered {
        admit_lookup(admission, slots.len())?;
        if !slots.contains(&row.0.output_slot()) {
            unshadowed.push(row);
        }
    }
    drop(slots);
    current.extend(unshadowed);
    current.sort_by(|left, right| left.0.canonical_cmp(&right.0));
    // Keep fact-source associations until optional encoding has completed.
    // Equal locators can still carry different authenticated fact payloads.
    Ok(current)
}

fn slot_index<'a>(
    rows: &'a [(Identity, Option<AcceptedCheckpointFactSource>)],
    admission: &mut InvalidationEditAdmission,
) -> Result<BTreeSet<CheckpointOutputSlot<'a>>, WorthQueryCheckpointCaptureDenial> {
    let mut slots = BTreeSet::new();
    for (identity, _) in rows {
        admit_lookup(admission, slots.len())?;
        let slot = identity.output_slot();
        if slots.contains(&slot) {
            continue;
        }
        admit_insert(admission, slots.len(), &slot)?;
        slots.insert(slot);
    }
    Ok(slots)
}

fn admit_insert<T>(
    admission: &mut InvalidationEditAdmission,
    count: usize,
    _row: &T,
) -> Result<(), WorthQueryCheckpointCaptureDenial> {
    let bytes = tree_insert_bytes::<T, ()>(count)
        .ok_or(SelectionDenial::CheckedArithmetic {
            phase: "checkpoint merge index layout",
        })
        .map_err(|denial| denial.into_capture_denial())?;
    admission.admit_read_scratch(bytes).map_err(|stop| {
        SelectionDenial::Admission {
            phase: "checkpoint merge index scratch",
            stop,
        }
        .into_capture_denial()
    })?;
    admit_lookup(admission, count)
}

fn admit_lookup(
    admission: &mut InvalidationEditAdmission,
    count: usize,
) -> Result<(), WorthQueryCheckpointCaptureDenial> {
    let navigation = tree_work::<()>(count)
        .ok_or(SelectionDenial::CheckedArithmetic {
            phase: "checkpoint merge index navigation",
        })
        .map_err(|denial| denial.into_capture_denial())?;
    admission
        .charge_ordered_operations(1, navigation)
        .map_err(|stop| {
            SelectionDenial::Admission {
                phase: "checkpoint merge index lookup",
                stop,
            }
            .into_capture_denial()
        })?;
    Ok(())
}

pub(super) fn capture_denial(
    detail: &'static str,
) -> worth_relational::facade::durability::DurabilityError {
    worth_relational::facade::durability::DurabilityError::new(
        worth_relational::facade::durability::RecoveryFailureClass::MissingAuthoritativeParentClosure,
        detail,
    )
}
