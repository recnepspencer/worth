//! Keep authenticated cache payloads only at their selected native family head.
use std::{any::TypeId, collections::BTreeMap};

use super::super::super::{
    application_output_demand::WorthQueryAcceptedOutputCheckpointIdentity as Identity,
    output_lineage::{NativePriorCheckpointOutput, WorthQueryApplicationOutputLineage},
};

pub(super) fn merge(
    lineage: &WorthQueryApplicationOutputLineage,
    current: impl IntoIterator<Item = (Identity, Option<TypeId>)>,
    recovered: impl IntoIterator<Item = (Identity, Option<TypeId>)>,
    priors: Vec<NativePriorCheckpointOutput>,
) -> Result<Vec<Identity>, ()> {
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
            return Err(());
        }
    }
    let retain =
        |rows: Vec<(Identity, Option<TypeId>)>, allow_missing: bool| -> Result<Vec<Identity>, ()> {
            let mut kept = Vec::new();
            for (identity, binding) in rows {
                let Some((family, role_name)) = lineage.checkpoint_family_role(binding)? else {
                    kept.push(identity);
                    continue;
                };
                let entity = identity
                    .roles
                    .iter()
                    .find(|role| role.role == role_name)
                    .map(|role| role.entity);
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
                        && head.identity.as_ref().is_none_or(|native| {
                            identity.producer == native.producer && identity.source == native.source
                        })
                }) {
                    kept.push(identity);
                }
            }
            Ok(kept)
        };
    let current = retain(current.into_iter().collect(), false)?;
    let recovered = retain(recovered.into_iter().collect(), true)?;
    drop(heads);
    let accepted = super::merge_accepted_outputs(current, recovered);
    // Locator-only rows deliberately contain no producer facts or witness.
    // They can choose Preserve after reopening, but cannot establish Ready.
    Ok(super::merge_accepted_outputs(
        accepted,
        priors.into_iter().filter_map(|prior| prior.identity),
    ))
}

pub(super) fn capture_denial(
    detail: &'static str,
) -> worth_relational::facade::durability::DurabilityError {
    worth_relational::facade::durability::DurabilityError::new(
        worth_relational::facade::durability::RecoveryFailureClass::MissingAuthoritativeParentClosure,
        detail,
    )
}
