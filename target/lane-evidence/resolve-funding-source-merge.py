"""Compose owner changes in the six Rust conflicts without dropping funding."""
from pathlib import Path
import re
import subprocess

root = Path.cwd()
prefix = 'crates/worth-relational/src/'
def original(ref, path):
    return subprocess.check_output(['git', 'show', ref + ':' + path]).decode('utf-8')
def write(path, value):
    target = root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(value, encoding='utf-8')
def ours_hunks(path):
    text = (root / path).read_text(encoding='utf-8')
    return re.sub(r'^<<<<<<< HEAD\n(.*?)^=======\n.*?^>>>>>>> 14b16fa292\n',
                  lambda match: match[1], text, flags=re.M | re.S)

# Captured same-owner roots replace the historical FoldedRoots engine. Both
# retain content through the actual root; never reintroduce covered replay.
for leaf in ('branches.rs', 'roots.rs'):
    path = prefix + 'durability/authority/checkpoint_complete/' + leaf
    write(path, original('HEAD', path))
orphan = root / (prefix + 'durability/authority/checkpoint_complete/branches/component_admission.rs')
assert orphan.parent == root / (prefix + 'durability/authority/checkpoint_complete/branches')
orphan.unlink()

path = prefix + 'storage/overlay/partition_content/digest_index/append_frontier.rs'
value = ours_hunks(path).replace('        key.checked_add(1).ok_or_else(invalid)?;',
    '        let value = *value.bytes();\n        key.checked_add(1).ok_or_else(invalid)?;')
write(path, value)

path = prefix + 'storage/residency/history_fold.rs'
value = ours_hunks(path).replace(
    'super::kind_commitment::verify(&image.kind_projection, reference.kind_commitment)',
    'super::kind_commitment::verify(&image.kind_projection, worth_foundational::CanonicalDigestId::new(reference.kind_commitment))')
write(path, value)
path = prefix + 'storage/residency/region_load.rs'
value = ours_hunks(path).replace('kind_commitment: cold.kind_authority.history,',
                                'kind_commitment: *cold.kind_authority.history.bytes(),')
write(path, value)

# One validation engine, with allocation-owner-specific installation. The
# publication driver retains its controlled custody; materialization keeps
# request-funded map and bucket backing at their actual allocation owners.
path = prefix + 'storage/residency/kind_projection/reconstruction.rs'
value = original('14b16fa292', path)
start = value.index('fn install_projection(')
end = value.index('pub(in crate::storage::residency) fn canonical_history(', start)
installation = value[start:end]
installation = installation.replace('fn install_projection(', 'pub(super) fn install_controlled(')
value = value[:start] + value[end:]
value = value.replace('    let mut expected = BTreeMap::<RelationId, RegionRelationProjection>::new();',
    '''    rebuild_using(partition, projection, basis, None, checkpoint, retain)
}
fn rebuild_using(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
    request: Option<&request::RequestProjection<'_, '_, '_>>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    let mut expected = BTreeMap::<RelationId, RegionRelationProjection>::new();''', 1)
value = value.replace('install_projection(partition, &expected, basis, checkpoint, retain)?;',
    'installation::install(partition, &expected, basis, request, checkpoint, retain)?;')
value = value.replace('''    let checkpoint: &mut ProjectionCheckpoint<'_> = &mut |_, _| Ok(());''',
    '''    rebuild_head_using(partition, projection, None, &mut |_, _| Ok(()))
}
fn rebuild_head_using(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    request: Option<&request::RequestProjection<'_, '_, '_>>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {''', 1)
value = value.replace('''    install_projection(
        partition,
        &expected,
        AdjacencyKindBasis::Historical,
        checkpoint,''', '''    installation::install(
        partition,
        &expected,
        AdjacencyKindBasis::Historical,
        request,
        checkpoint,''')
value += '\nmod installation;\nmod request;\npub(in crate::storage::residency) use request::{rebuild_for_request, rebuild_head_history_for_request};\n'
write(path, value)
write(path.removesuffix('.rs') + '/installation.rs', '''//! Install validated kind claims through the driver that owns their backing.
use super::*;

pub(super) fn install(
    partition: &mut PartitionState,
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    basis: AdjacencyKindBasis,
    request: Option<&request::RequestProjection<'_, '_, '_>>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
    retain: &mut ProjectionRetention<'_>,
) -> Result<(), Cause> {
    match request {
        Some(request) => install_request(partition, expected, basis, request, checkpoint),
        None => install_controlled(partition, expected, basis, checkpoint, retain),
    }
}

fn install_request(
    partition: &mut PartitionState,
    expected: &BTreeMap<RelationId, RegionRelationProjection>,
    basis: AdjacencyKindBasis,
    funding: &request::RequestProjection<'_, '_, '_>,
    checkpoint: &mut ProjectionCheckpoint<'_>,
) -> Result<(), Cause> {
    let historical = matches!(basis, AdjacencyKindBasis::Historical);
    let request = funding.execution;
    for (incoming, table) in [(false, &mut partition.adjacency), (true, &mut partition.reverse_adjacency)] {
        let original = table.clone();
        for (&slot, _) in original.iter() {
            checkpoint(1, 0)?;
            let adjacency = table.get_mut_for_request(slot, request)
                .map_err(|cause| funding.stop(cause))?
                .ok_or(Cause::IntegrityMismatch)?;
            if historical {
                adjacency.reset_historical_kind_buckets();
            } else {
                adjacency.reset_kind_buckets();
            }
        }
        for entry in expected.values() {
            checkpoint(1, 0)?;
            let slots = if incoming { &entry.incoming } else { &entry.outgoing };
            for &slot in slots {
                checkpoint(1, 0)?;
                let adjacency = table.get_mut_for_request(slot, request)
                    .map_err(|cause| funding.stop(cause))?
                    .ok_or(Cause::IntegrityMismatch)?;
                let result = if historical {
                    adjacency.index_historical_kind_for_request(entry.kind, entry.relation, request)
                } else {
                    adjacency.index_current_kind_for_request(entry.kind, entry.relation, request)
                };
                result.map_err(|cause| funding.stop(cause))?;
            }
        }
        if !historical {
            for (_, adjacency) in table.iter() {
                let ids: BTreeSet<_> = adjacency.current_ids().iter().copied().collect();
                let indexed: BTreeSet<_> = adjacency.checkpoint_kind_memberships(basis)
                    .into_iter().flat_map(|(_, ids)| ids).collect();
                if ids != indexed { return Err(Cause::IntegrityMismatch); }
            }
        }
    }
    Ok(())
}

''' + installation)

# A typed request refusal is carried separately from the storage callback's
# source sentinel; the sentinel never escapes as a false storage diagnosis.
write(path.removesuffix('.rs') + '/request.rs', '''//! Original-request driver for the shared kind validation and backing owners.
use super::*;
use super::super::super::RelationalResidencyDemandFailure as Failure;
use crate::execution::RelationalExecutionDenialCause;
use worth_execution::ExecutionRequest;

pub(in crate::storage::residency) fn rebuild_for_request(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    basis: AdjacencyKindBasis,
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let id = partition.partition_id;
    drive(request, id, |funding, checkpoint| super::rebuild_using(
        partition, projection, basis, Some(funding), checkpoint, &mut |_, _| Ok(None)))
}
pub(in crate::storage::residency) fn rebuild_head_history_for_request(
    partition: &mut PartitionState,
    projection: &[RegionRelationProjection],
    request: &ExecutionRequest<'_, '_>,
) -> Result<(), Failure> {
    let id = partition.partition_id;
    drive(request, id, |funding, checkpoint| super::rebuild_head_using(
        partition, projection, Some(funding), checkpoint))
}

pub(super) struct RequestProjection<'request, 'scope, 'host> {
    pub(super) execution: &'request ExecutionRequest<'scope, 'host>,
    stopped: std::cell::Cell<Option<RelationalExecutionDenialCause>>,
}
impl RequestProjection<'_, '_, '_> {
    pub(super) fn stop(&self, cause: RelationalExecutionDenialCause) -> Cause {
        if self.stopped.get().is_none() { self.stopped.set(Some(cause)); }
        Cause::SourceUnavailable
    }
}

fn drive(
    request: &ExecutionRequest<'_, '_>,
    partition: crate::identity::data::PartitionId,
    operation: impl FnOnce(&RequestProjection<'_, '_, '_>, &mut ProjectionCheckpoint<'_>) -> Result<(), Cause>,
) -> Result<(), Failure> {
    let funding = RequestProjection { execution: request, stopped: std::cell::Cell::new(None) };
    let result = operation(&funding, &mut |_, _| {
        if funding.stopped.get().is_some() { return Err(Cause::SourceUnavailable); }
        match request.consult() {
            Ok(()) => Ok(()),
            Err(cause) => Err(funding.stop(cause.into())),
        }
    });
    if let Some(cause) = funding.stopped.get() { return Err(Failure::Request(cause)); }
    result.map_err(|cause| crate::storage::data::RelationalStorageUnavailable {
        subject: crate::storage::data::RelationalStorageSubject::Region(partition), cause,
    }.into())
}
''')

path = 'tools/boundary-check/config/certification-construction-callers.txt'
value = (root / path).read_text(encoding='utf-8')
value = re.sub(r'^<<<<<<< HEAD\n(.*?)^=======\n(.*?)^>>>>>>> 14b16fa292\n',
               lambda match: match[1] + match[2], value, flags=re.M | re.S)
write(path, value)
print('Resolved source conflicts with original typed request stops and backing-local allocation.')
