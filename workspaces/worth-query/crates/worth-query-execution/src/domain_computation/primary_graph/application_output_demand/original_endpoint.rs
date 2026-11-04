//! Exact original output movements, without ordinal inference or head selection.
use super::WorthQueryOutputDemandSettlement;
use {crate::domain_computation::primary_graph::output_reuse::read_denial, crate::domain_computation::primary_graph::structural_walk::memory, crate::domain_computation::primary_graph::WorthQueryApplicationReadObservation as Read, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenial as Denial, crate::domain_computation::primary_graph::application_output_demand::denial::WorthQueryOutputDemandDenialKind as Kind};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};
use worth_runtime_world::facade::{
    CompositeCommitIdentity, ConsumedCompositePublication, ProductBranchReferenceGeneration,
    ProductBranchReferenceSnapshot,
};

type Key<'a> = (
    &'a CompositeCommitIdentity,
    ProductBranchReferenceGeneration,
);

fn key(snapshot: &ProductBranchReferenceSnapshot) -> Key<'_> {
    (snapshot.selected_commit(), snapshot.reference_generation())
}

fn refused(kind: Kind, subject: &'static str) -> Denial {
    Denial::new(kind, subject)
}

fn checkpoint(context: &mut MapKernelContext<'_, '_>, len: usize) -> Result<(), Denial> {
    let cost = u64::from(usize::BITS - len.leading_zeros()) + 1;
    (if context.has_accounting() {
        context.checkpoint(cost)
    } else {
        context.poll()
    })
    .map_err(|cause| read_denial(WorkCeilingDenial::Stopped(cause).into()))
}

/// Both iterators borrow the same immutable original root/dependent collection.
/// World facts link intermediate references; only the final reference needs a
/// retained read. That read may come from a later-settled original sibling.
pub(super) fn derive<'a>(
    execution: &ExecutionRequest<'_, '_>,
    source: &Arc<Read>,
    settlements: &mut dyn Iterator<Item = &'a WorthQueryOutputDemandSettlement>,
    reads: &mut dyn Iterator<Item = &'a WorthQueryOutputDemandSettlement>,
) -> Result<Arc<Read>, Denial> {
    execution
        .run_native_with_checkpoint(|context| {
            let mut memory = None;
            let mut bytes = 0;
            let mut edges: BTreeMap<Key<'_>, &ConsumedCompositePublication> = BTreeMap::new();
            let origin = source.product_observation();
            for settlement in settlements {
                checkpoint(context, edges.len())?;
                if !source.same_application_occurrence(settlement.retained_read_ref()) {
                    return Err(refused(
                        Kind::ForeignSettlement,
                        "original output occurrence",
                    ));
                }
                let Some(receipt) = settlement.application_commit_receipt() else {
                    continue;
                };
                let movement = receipt
                    .committed_product_publication()
                    .original_movement()
                    .ok_or_else(|| {
                        refused(
                            Kind::RetainedBasisUnavailable,
                            "original output movement is unavailable",
                        )
                    })?;
                let before = movement.old_product_head();
                let after = movement.new_product_head();
                for snapshot in [before, after] {
                    if snapshot.owner_identity() != origin.owner_identity()
                        || snapshot.branch_identity() != origin.branch_identity()
                        || snapshot.lifecycle_incarnation() != origin.lifecycle_incarnation()
                    {
                        return Err(refused(
                            Kind::ForeignSettlement,
                            "original output movement occurrence",
                        ));
                    }
                }
                if let Some(prior) = edges.get(&key(before)) {
                    if prior.old_product_head() != before
                        || prior.new_product_head() != after
                        || prior.attempt_identity() != movement.attempt_identity()
                    {
                        return Err(refused(
                            Kind::ForeignSettlement,
                            "conflicting original output movements",
                        ));
                    }
                    continue;
                }
                memory::grow_tree::<(Key<'_>, &ConsumedCompositePublication)>(
                    &mut memory,
                    &mut bytes,
                    edges.len(),
                    execution,
                    context.has_accounting(),
                )
                .map_err(|cause| {
                    read_denial(cause).with_subject("original output endpoint index")
                })?;
                edges.insert(key(before), movement);
            }
            let mut endpoint: Option<&ProductBranchReferenceSnapshot> = None;
            let mut next = (origin.selected_commit(), origin.reference_generation());
            let mut followed = 0;
            while let Some(movement) = edges.get(&next) {
                checkpoint(context, edges.len())?;
                if followed == edges.len() {
                    return Err(refused(
                        Kind::ForeignSettlement,
                        "cyclic original output movements",
                    ));
                }
                let before = movement.old_product_head();
                let continuous = match endpoint {
                    Some(endpoint) => endpoint == before,
                    None => before.matches_observation(origin),
                };
                if !continuous {
                    return Err(refused(
                        Kind::ForeignSettlement,
                        "disconnected original output movement",
                    ));
                }
                endpoint = Some(movement.new_product_head());
                next = key(endpoint.expect("one original movement supplied its successor"));
                followed += 1;
            }
            // Prefer genuine producer correspondence at the same exact endpoint;
            // a source-only D lease cannot manufacture that optional seed.
            for settlement in reads {
                checkpoint(context, 0)?;
                let read = settlement.retained_read_ref();
                let matches = match endpoint {
                    Some(endpoint) => endpoint.matches_observation(read.product_observation()),
                    None => read.product_observation().compare(origin).is_ok(),
                };
                if matches {
                    return Ok(Arc::clone(read));
                }
            }
            match endpoint {
                None => Ok(Arc::clone(source)),
                Some(_) => Err(refused(
                    Kind::RetainedBasisUnavailable,
                    "original output endpoint read is unavailable",
                )),
            }
        })
        .map_err(|cause| read_denial(cause.into()))?
        .0
}
