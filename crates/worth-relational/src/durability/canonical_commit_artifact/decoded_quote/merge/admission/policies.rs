//! Original policy-row admissions precede the decoder's shared leaf growth.
use super::super::policies as quote;
use super::*;
use crate::merge::data::{
    AspectPolicyResolutionRecord, MergePolicyRowMemory, MergeStrategyMemory,
    RelationalMergeAspectPolicyWitnessRow, RelationalMergeDeletionStrategyWitnessRow,
    RelationalMergeTopologyStrategyWitnessRow, ResolvedAspectMergePolicy,
};

pub(super) fn tree_storage(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64, Refusal> {
    let rows = field(value, "aspect_policy_rows", request)?;
    let mut bytes = super::tree_storage::<MergePolicyRowMemory>(count(rows, request)?)?;
    rows.elements(request, |row| {
        for name in ["aspect_resolutions", "applied_policies"] {
            checked_sum(
                &mut bytes,
                super::tree_storage::<Option<MergeLeafMemory>>(count(
                    field(row, name, request)?,
                    request,
                )?)?,
            )?;
        }
        Ok(())
    })?;
    Ok(bytes)
}

pub(super) fn prepare(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<MergeStrategyMemory, Refusal> {
    let rows = field(value, "aspect_policy_rows", request)?;
    let mut memories = Vec::with_capacity(count(rows, request)?);
    let mut children = 0;
    rows.elements(request, |row| {
        let resolutions = field(row, "aspect_resolutions", request)?;
        let mut resolution_names = Vec::with_capacity(count(resolutions, request)?);
        let mut resolution_children = 0;
        resolutions.elements(request, |resolution| {
            let policy = resolution.optional_field("applied_policy", request)?;
            let memory = if policy.is_none() {
                None
            } else {
                policy_name(policy, request)?
            };
            if !policy.is_none() {
                checked_sum(
                    &mut resolution_children,
                    identities::policy(policy, request)?.retained,
                )?;
            }
            resolution_names.push(memory);
            Ok(())
        })?;
        let resolution_full =
            shared_rows::<AspectPolicyResolutionRecord>(resolutions, request, |value| {
                quote::resolution(value, request)
            })?
            .retained;
        let resolutions_memory = leaf(
            count(resolutions, request)?,
            resolution_full
                .checked_sub(resolution_children)
                .ok_or_else(malformed)?,
            request,
        )?;
        let policies = field(row, "applied_policies", request)?;
        let mut policy_names = Vec::with_capacity(count(policies, request)?);
        let mut policy_children = 0;
        policies.elements(request, |policy| {
            let policy = field(policy, "policy", request)?;
            checked_sum(
                &mut policy_children,
                identities::policy(policy, request)?.retained,
            )?;
            policy_names.push(policy_name(policy, request)?);
            Ok(())
        })?;
        let policies_full = shared_rows::<ResolvedAspectMergePolicy>(policies, request, |value| {
            identities::policy_declaration(value, request)
        })?
        .retained;
        let policies_memory = leaf(
            count(policies, request)?,
            policies_full
                .checked_sub(policy_children)
                .ok_or_else(malformed)?,
            request,
        )?;
        checked_sum(&mut children, resolution_full)?;
        checked_sum(&mut children, policies_full)?;
        memories.push(MergePolicyRowMemory {
            resolutions: resolutions_memory,
            resolution_names,
            policies: policies_memory,
            policy_names,
        });
        Ok(())
    })?;
    let aspects = row_memory::<RelationalMergeAspectPolicyWitnessRow>(
        rows,
        request,
        |value| quote::policy_row(value, request),
        children,
    )?;
    let topology = row_memory::<RelationalMergeTopologyStrategyWitnessRow>(
        field(value, "topology_rows", request)?,
        request,
        |value| quote::posture_row(value, request),
        0,
    )?;
    let deletion = row_memory::<RelationalMergeDeletionStrategyWitnessRow>(
        field(value, "deletion_rows", request)?,
        request,
        |value| quote::posture_row(value, request),
        0,
    )?;
    Ok(MergeStrategyMemory {
        aspects,
        rows: memories,
        topology,
        deletion,
    })
}

pub(super) fn retained_leaf_bytes(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<u64, Refusal> {
    let mut bytes = shared_rows::<RelationalMergeAspectPolicyWitnessRow>(
        field(value, "aspect_policy_rows", request)?,
        request,
        |value| quote::policy_row(value, request),
    )?
    .retained;
    checked_sum(
        &mut bytes,
        shared_rows::<RelationalMergeTopologyStrategyWitnessRow>(
            field(value, "topology_rows", request)?,
            request,
            |value| quote::posture_row(value, request),
        )?
        .retained,
    )?;
    checked_sum(
        &mut bytes,
        shared_rows::<RelationalMergeDeletionStrategyWitnessRow>(
            field(value, "deletion_rows", request)?,
            request,
            |value| quote::posture_row(value, request),
        )?
        .retained,
    )?;
    Ok(bytes)
}

fn policy_name(
    value: BorrowedPayload<'_>,
    request: &ExecutionRequest<'_, '_>,
) -> Result<Option<MergeLeafMemory>, Refusal> {
    let mut memory = None;
    if value.is_map() {
        value.fields(request, |variant, raw| {
            if variant == "Custom" {
                let length = field(raw, "name", request)?.text(request)?.len();
                memory = Some(leaf(
                    length,
                    identities::policy(value, request)?.retained,
                    request,
                )?);
            }
            Ok(())
        })?;
    }
    Ok(memory)
}
