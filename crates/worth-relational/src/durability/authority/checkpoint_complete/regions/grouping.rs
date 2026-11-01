//! Group canonical descriptor references without a per-root full-source scan.
use super::*;
use crate::branch::RelationalBranchRootSchemaAuthority;
use crate::history::data::CommitId;
#[derive(Clone)]
pub(super) struct RootGroup {
    pub(super) count: usize,
    pub(super) first_ordinal: usize,
    pub(super) schema: Arc<RelationalBranchRootSchemaAuthority>,
    pub(super) carrier: [u8; 32],
    pub(super) current_main: bool,
}
pub(super) struct Grouped<'a> {
    pub(super) groups: ExecutionOrderedMap<CommitId, RootGroup>,
    pub(super) descriptors:
        ExecutionOrderedMap<(CommitId, usize), &'a RelationalBranchBasisDescriptor>,
    pub(super) empty: ExecutionOrderedMap<usize, (&'a RelationalBranchBasisDescriptor, [u8; 32])>,
    pub(super) mirror: Option<CommitId>,
}
impl<'a> Grouped<'a> {
    pub(super) fn capture<
        I: Iterator<
            Item = (
                &'a RelationalBranchBasisDescriptor,
                &'a Arc<RelationalBranchRoot>,
            ),
        >,
    >(
        observations: impl Fn() -> I,
        main: &BranchId,
        branches: &BranchImages<'_>,
        roots: &FoldedRoots,
        schemas: &CompleteSchemas<'_>,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Denial> {
        let mut result = Self {
            groups: Default::default(),
            descriptors: Default::default(),
            empty: Default::default(),
            mirror: None,
        };
        let mut saw_current_main = false;
        for (ordinal, (descriptor, observed)) in observations().enumerate() {
            crate::execution::check_request(request).map_err(request_stop)?;
            let schema = branches.require_component(descriptor, roots)?;
            let carrier = schemas
                .carrier_for(&schema)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            let observed_schema = observed.retained_schema_authority();
            if schemas.carrier_for(&observed_schema) != Some(carrier) {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let current_main =
                descriptor.branch_id() == main && branches.is_current_component(descriptor, roots);
            if current_main {
                if saw_current_main {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                saw_current_main = true;
            }
            let Some(target) = descriptor.reference().target().as_basis() else {
                if observed.descriptor().is_some() || observed.checkpoint_region_count() != 0 {
                    return Err(Denial::OriginalAuthorityUnavailable);
                }
                result
                    .empty
                    .insert_cloned_native_for_request(&ordinal, &(descriptor, carrier), *request)
                    .map_err(map_stop)?;
                continue;
            };
            let commit = CommitId(target.selected_commit_id());
            let folded = roots
                .get(commit)
                .ok_or(Denial::OriginalAuthorityUnavailable)?;
            if observed.descriptor() != Some(target.roots())
                || observed.descriptor() != folded.root.descriptor()
                || observed.axes() != folded.root.axes()
                || !observed
                    .canonical_envelope()
                    .zip(folded.root.canonical_envelope())
                    .is_some_and(|(observed, derived)| observed.commit == derived.commit)
            {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            if schemas.carrier_for(&folded.root.retained_schema_authority()) != Some(carrier) {
                return Err(Denial::OriginalAuthorityUnavailable);
            }
            let group = match result.groups.get(&commit) {
                Some(previous) => {
                    if previous.carrier != carrier {
                        return Err(Denial::OriginalAuthorityUnavailable);
                    }
                    RootGroup {
                        count: previous.count.checked_add(1).ok_or_else(overflow)?,
                        current_main: previous.current_main || current_main,
                        ..previous.clone()
                    }
                }
                None => RootGroup {
                    count: 1,
                    first_ordinal: ordinal,
                    schema,
                    carrier,
                    current_main,
                },
            };
            if current_main {
                result.mirror = Some(commit);
            }
            result
                .groups
                .insert_cloned_native_for_request(&commit, &group, *request)
                .map_err(map_stop)?;
            result
                .descriptors
                .insert_cloned_native_for_request(&(commit, ordinal), &descriptor, *request)
                .map_err(map_stop)?;
        }
        if !saw_current_main {
            return Err(Denial::OriginalAuthorityUnavailable);
        }
        Ok(result)
    }
    pub(super) fn copy_empty(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<
        (
            Vec<RelationalCheckpointEmptyComponent>,
            Option<ExecutionMemoryReservation>,
        ),
        Denial,
    > {
        let mut bytes = array_bytes::<RelationalCheckpointEmptyComponent>(self.empty.len())?;
        for (_, (descriptor, _)) in self.empty.iter() {
            crate::execution::check_request(request).map_err(request_stop)?;
            bytes = bytes
                .checked_add(descriptor.branch_id().0.len())
                .and_then(|bytes| {
                    bytes.checked_add(descriptor.reference().branch_id().as_str().len())
                })
                .ok_or_else(overflow)?;
        }
        let memory = reserve(bytes, request)?;
        let mut empty = Vec::with_capacity(self.empty.len());
        for (_, (descriptor, carrier)) in self.empty.iter() {
            crate::execution::check_request(request).map_err(request_stop)?;
            empty.push(RelationalCheckpointEmptyComponent {
                descriptor: (**descriptor).clone(),
                schema_carrier_digest: *carrier,
            });
        }
        Ok((empty, memory))
    }
}
