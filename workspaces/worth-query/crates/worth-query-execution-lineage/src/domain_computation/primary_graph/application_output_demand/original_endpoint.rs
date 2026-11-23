//! Exact original output movements, without ordinal inference or head selection.
use super::WorthQueryOutputDemandSettlement;
use crate::domain_computation::primary_graph::{
    output_reuse::read_denial, structural_walk::memory,
    WorthQueryApplicationReadObservation as Read, WorthQueryOutputDemandDenial as Denial,
    WorthQueryOutputDemandDenialKind as Kind,
};
use std::{collections::BTreeMap, sync::Arc};
use worth_execution::{ExecutionRequest, MapKernelContext, WorkCeilingDenial};
use worth_runtime_world::facade::{
    CompositeCommitIdentity, CompositePublicationAttemptIdentity, ProductBranchReferenceGeneration,
};

type Key = (CompositeCommitIdentity, ProductBranchReferenceGeneration);
type MovementEdge<'a> = (
    crate::basis::WorthQueryProductBranchReadIdentity,
    crate::basis::WorthQueryProductBranchReadIdentity,
    &'a CompositePublicationAttemptIdentity,
);

fn key(identity: &crate::basis::WorthQueryProductBranchReadIdentity) -> Key {
    (
        identity.selected_commit().clone(),
        identity.reference_generation(),
    )
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
pub fn derive<'a>(
    execution: &ExecutionRequest<'_, '_>,
    source: &Arc<Read>,
    settlements: &mut dyn Iterator<Item = &'a WorthQueryOutputDemandSettlement>,
    reads: &mut dyn Iterator<Item = &'a WorthQueryOutputDemandSettlement>,
) -> Result<Arc<Read>, Denial> {
    execution
        .run_native_with_checkpoint(|context| {
            let mut memory = None;
            let mut bytes = 0;
            let mut edges: BTreeMap<Key, MovementEdge<'_>> = BTreeMap::new();
            let origin = source.product_observation();
            for settlement in settlements {
                checkpoint(context, edges.len())?;
                if !source.same_application_occurrence(settlement.retained_read_ref()) {
                    return Err(refused(
                        Kind::ForeignSettlement,
                        "original output occurrence",
                    ));
                }
                let Some(receipt) = settlement.publication_view() else {
                    continue;
                };
                let publication = receipt.committed_product_publication();
                // This member never moved World independently. Its nominal
                // group origin proves zero distance at this exact endpoint.
                if publication.co_committed_at(origin) {
                    continue;
                }
                let (before, after, attempt) =
                    publication.original_movement_endpoints().ok_or_else(|| {
                        refused(
                            Kind::RetainedBasisUnavailable,
                            "original output movement is unavailable",
                        )
                    })?;
                for endpoint in [&before, &after] {
                    if endpoint.branch_identity().owner_identity() != origin.owner_identity()
                        || endpoint.branch_identity() != origin.branch_identity()
                        || endpoint.lifecycle_incarnation() != origin.lifecycle_incarnation()
                    {
                        return Err(refused(
                            Kind::ForeignSettlement,
                            "original output movement occurrence",
                        ));
                    }
                }
                let edge_key = key(&before);
                if let Some((prior_before, prior_after, prior_attempt)) = edges.get(&edge_key) {
                    if prior_before != &before
                        || prior_after != &after
                        || *prior_attempt != attempt
                    {
                        return Err(refused(
                            Kind::ForeignSettlement,
                            "conflicting original output movements",
                        ));
                    }
                    continue;
                }
                memory::grow_tree::<(Key, MovementEdge<'_>)>(
                    &mut memory,
                    &mut bytes,
                    edges.len(),
                    execution,
                    context.has_accounting(),
                )
                .map_err(|cause| {
                    read_denial(cause).with_subject("original output endpoint index")
                })?;
                edges.insert(edge_key, (before, after, attempt));
            }
            let mut endpoint: Option<&crate::basis::WorthQueryProductBranchReadIdentity> = None;
            let mut next = (
                origin.selected_commit().clone(),
                origin.reference_generation(),
            );
            let mut followed = 0;
            while let Some((before, after, _)) = edges.get(&next) {
                checkpoint(context, edges.len())?;
                if followed == edges.len() {
                    return Err(refused(
                        Kind::ForeignSettlement,
                        "cyclic original output movements",
                    ));
                }
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
                endpoint = Some(after);
                next = key(after);
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
