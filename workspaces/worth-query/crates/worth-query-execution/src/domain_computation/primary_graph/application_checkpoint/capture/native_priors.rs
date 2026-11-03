//! Keep authenticated cache payloads only at their selected native family head.
use crate::equality_type_id::EqualityTypeId as TypeId;
use std::collections::BTreeMap;
use std::collections::BTreeSet;

use super::super::super::application_output_demand::AcceptedCheckpointFactSource;
use super::super::super::application_output_demand::CheckpointOutputSlot;
use super::super::super::application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity as Identity;
use super::super::super::output_lineage::NativePriorCheckpointOutput;
use super::super::super::output_lineage::WorthQueryApplicationOutputLineage;
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

        if heads.insert(key, prior).is_some() {
            return Err(capture_denial("checkpoint output family binding is ambiguous").into());
        }
    }
    let retain = |rows: Vec<Accepted>,
                  allow_missing: bool|
     -> Result<Vec<_>, WorthQueryCheckpointCaptureDenial> {
        let mut kept = Vec::new();
        for (identity, source, binding) in rows {
            let Some((family, role_name)) = lineage
                .checkpoint_family_role(binding)
                .map_err(|denial| denial.into_capture_denial())?
            else {
                kept.push((identity, source));
                continue;
            };
            let mut entity = None;
            for role in identity.roles.iter() {
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

            if heads.get(&key).map_or(allow_missing, |head| {
                binding == Some(head.binding)
                    && identity.idempotency_key == head.idempotency_key
                    && head.producing_implementation() == Some(identity.implementation)
                    && head.identity.as_ref().is_none_or(|native| {
                        identity.producer.as_str() == native.producer
                            && identity.source == native.source
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
    let mut accepted = merge_rows(current, recovered)?;
    let accepted_slots = slot_index(&accepted)?;
    let mut native = Vec::new();
    for prior in priors {
        // These are exactly the slots merge_accepted_outputs would discard.
        // Acceptance was checked against the authoritative head above; having
        // a cache row alone never authorizes this omission.
        if let (Some(locator), Some(partition)) = (&prior.identity, prior.partition)
            && accepted_slots.contains(&CheckpointOutputSlot::new(
                locator.producer,
                prior.scope,
                partition,
            ))
        {
            continue;
        }
        if let Some(identity) = prior
            .materialize_identity()
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
) -> Result<Vec<(Identity, Option<AcceptedCheckpointFactSource>)>, WorthQueryCheckpointCaptureDenial>
{
    let slots = slot_index(&current)?;
    let mut unshadowed = Vec::new();
    for row in recovered {
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
) -> Result<BTreeSet<CheckpointOutputSlot<'a>>, WorthQueryCheckpointCaptureDenial> {
    let mut slots = BTreeSet::new();
    for (identity, _) in rows {
        let slot = identity.output_slot();
        if slots.contains(&slot) {
            continue;
        }

        slots.insert(slot);
    }
    Ok(slots)
}

pub(super) fn capture_denial(
    detail: &'static str,
) -> worth_relational::facade::durability::DurabilityError {
    worth_relational::facade::durability::DurabilityError::new(
        worth_relational::facade::durability::RecoveryFailureClass::MissingAuthoritativeParentClosure,
        detail,
    )
}
