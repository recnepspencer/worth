//! Read only original intent meaning, admitting every retained allocation first.
use super::{fields::Read, *};
use crate::descriptor::binding;
use sha2::{Digest, Sha256};
use std::num::NonZeroU64;
use worth_execution::{ExecutionRequest, ExecutionRequestArrayBuilder};
use ApplicationHomeTransitionIntentDenial as Denial;

pub(in crate::open::readmission::transition) fn decode(
    bytes: &[u8],
    home: HomeIdentity,
    position: u64,
    shared_bodies: Option<&std::sync::Arc<super::super::native_bodies::HomeNativeStepBodies>>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<HomeProgramTransitionIntent, Denial> {
    if bytes.len() > MAXIMUM {
        return Err(Denial::SizeLimit);
    }
    request
        .run_native_with_checkpoint(|context| {
            if context.has_accounting() {
                context.checkpoint(bytes.len() as u64)
            } else {
                context.poll()
            }
        })
        .map_err(Denial::Execution)?
        .0
        .map_err(|cause| Denial::Execution(worth_execution::WorkCeilingDenial::Stopped(cause)))?;
    let (payload, checksum) = bytes
        .split_at_checked(bytes.len().checked_sub(32).ok_or(Denial::Malformed)?)
        .ok_or(Denial::Malformed)?;
    if Sha256::digest(payload).as_slice() != checksum {
        return Err(Denial::Malformed);
    }
    let mut read = Read(payload);
    if &read.fixed::<8>()? != MAGIC {
        return Err(Denial::Malformed);
    }
    let version = BoundaryProtocolVersion::try_new(read.u32()?).map_err(|_| Denial::Malformed)?;
    if version != VERSION {
        return Err(Denial::UnsupportedVersion { observed: version });
    }
    let observed =
        HomeIdentity::from_observed_bytes(read.fixed()?).map_err(|_| Denial::Malformed)?;
    if observed != home {
        return Err(Denial::ForeignHome);
    }
    let intent = NonZeroU64::new(read.u64()?).ok_or(Denial::Malformed)?;
    if intent.get() != position {
        return Err(Denial::ForeignIntentPosition);
    }
    let owner = DurableWorldOwnerIdentity::from_recorded(
        home,
        WorldOwnerSlotIdentity::from_recorded(read.u64()?),
    );
    let branch_name = read.text()?;
    let incarnation = DurableProductBranchIncarnation::from_recorded(owner, read.u64()?);
    let generation = DurableProductReferenceGeneration::from_recorded(read.u64()?);
    let commit = DurableCompositeCommitIdentity::from_recorded(owner, read.u64()?);
    let digest = HomeProgramTransitionPlanDigest::from_recorded(read.fixed()?);
    let steps = read.u32()?;
    let target = HomeProgramTransitionTargetMeaning::from_recorded(
        read.u32()?,
        HomeProgramTransitionCatalogDigest::from_recorded(read.fixed()?),
        worth_foundational::facade::HomeProgramTransitionSchemaDigest::from_recorded(read.fixed()?),
        worth_foundational::facade::HomeProgramTransitionSchemaCarrierDigest::from_recorded(
            read.fixed()?,
        ),
    );
    if read.u32()? != target.whole_candidate_validation_version() {
        return Err(Denial::Malformed);
    }
    let native = read.bytes()?;
    let predecessor = read.bytes()?;
    let successor = read.bytes()?;
    if steps as usize != read.0.len() / 40 || read.0.len() % 40 != 0 {
        return Err(Denial::AssignmentCount);
    }
    let metadata_bytes = binding::allocation_bytes(predecessor)
        .map_err(Denial::Binding)?
        .checked_add(binding::allocation_bytes(successor).map_err(Denial::Binding)?)
        .and_then(|n| {
            (u64::from(steps) + 2)
                .checked_mul(branch_name.len() as u64)
                .and_then(|names| n.checked_add(names))
        })
        .and_then(|n| {
            n.checked_add(
                (std::mem::size_of::<HomeProgramTransitionIntent>()
                    + 2 * std::mem::size_of::<usize>()) as u64,
            )
        })
        .ok_or(Denial::SizeLimit)?;
    let metadata = request
        .reserve_retained_memory(metadata_bytes)
        .map_err(Denial::Execution)?;
    let branch = DurableProductBranchIdentity::from_recorded(owner, branch_name.to_owned())
        .map_err(|_| Denial::Malformed)?;
    let source = DurableProductBranchReference::from_recorded(
        branch.clone(),
        incarnation,
        generation,
        commit,
    )
    .map_err(|_| Denial::Malformed)?;
    let plan = HomeProgramTransitionPlanDescription::from_recorded(
        HomeProgramTransitionIdentity::from_recorded(home, intent),
        source,
        digest,
        steps,
        target,
    )
    .map_err(|_| Denial::Malformed)?;
    let predecessor = binding::decode(predecessor).map_err(Denial::Binding)?;
    let successor = binding::decode(successor).map_err(Denial::Binding)?;
    let native = RecordedRelationalComponentReference::from_json_in(request, native)
        .map_err(Denial::Native)?;
    let mut assignments = ExecutionRequestArrayBuilder::allocate(*request, steps as usize)
        .map_err(Denial::Backing)?;
    if shared_bodies.is_some_and(|bodies| bodies.values().len() != steps as usize) {
        return Err(Denial::AssignmentCount);
    }
    let mut bodies = if shared_bodies.is_none() {
        Some(
            ExecutionRequestArrayBuilder::allocate(*request, steps as usize)
                .map_err(Denial::Backing)?,
        )
    } else {
        None
    };
    let mut previous = None;
    for step in 0..steps {
        request.consult().map_err(Denial::Execution)?;
        let ordinal = read.u64()?;
        let body = HomeProgramTransitionBodyDigest::from_recorded(read.fixed()?);
        if let Some(shared) = shared_bodies {
            if shared.values()[step as usize] != body {
                return Err(Denial::TransitionPlanDrift);
            }
        }
        if let Some(bodies) = &mut bodies {
            bodies.push(body).map_err(Denial::Backing)?;
        }
        if previous.is_some_and(|previous| ordinal <= previous) {
            return Err(Denial::AssignmentOrder);
        }
        previous = Some(ordinal);
        let association = CompositeAttemptAssociation::from_recorded(
            DurableCompositePublicationAttemptIdentity::from_recorded(owner, ordinal),
            branch.clone(),
            incarnation,
        )
        .map_err(|_| Denial::Malformed)?;
        assignments.push(association).map_err(Denial::Backing)?;
    }
    let bodies = match (shared_bodies, bodies) {
        (Some(shared), None) => shared.clone(),
        (None, Some(bodies)) => super::super::native_bodies::HomeNativeStepBodies::retain(
            bodies.seal().map_err(Denial::Backing)?,
            request,
        )?,
        (Some(_), Some(_)) | (None, None) => return Err(Denial::Malformed),
    };
    Ok(HomeProgramTransitionIntent {
        bodies,
        plan,
        predecessor,
        successor,
        native,
        assignments: assignments.seal().map_err(Denial::Backing)?,
        _metadata: metadata,
    })
}
