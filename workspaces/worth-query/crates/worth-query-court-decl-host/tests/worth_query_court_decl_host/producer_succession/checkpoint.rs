//! Observe and corrupt durable artifacts made by the actual Query producer journey.
use super::*;
use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Weak};
use worth_execution::{ExecutionImmutableBytes, ExecutionRequest, ExecutionRequestArrayBuilder};
use worth_query_testkit_document_retention::facade::directory_home::{
    directory_lease, ClosedDirectorySnapshot,
};
use worth_relational::facade::{
    durability::{
        RelationalDurableColdSource, RelationalDurableMemberCustody,
        RelationalDurableMemberLocator, RelationalDurableMemberRetirementObservation,
        RelationalDurableMemberRetirementRefusal, RelationalDurableOwnedReadRefusal,
        RelationalDurableReadPayload, RelationalNativeCheckpoint,
    },
    publication::PatchStreamPosition,
    storage::RelationalStorageUnavailableCause,
};
const ARTIFACT_MAGIC: &[u8] = b"WRTHRCCA";
const ARTIFACT_HEADER: usize = 66;

pub fn capture(app: &world::App) -> Vec<u8> {
    app.capture_native_checkpoint_frame()
}

pub fn envelopes(app: &world::App) -> Vec<Value> {
    envelopes_in_dir(app.directory())
}

pub fn envelopes_in_dir(root: &Path) -> Vec<Value> {
    let mut files = Vec::new();
    collect_files(root, &mut files);
    files.sort();
    let mut by_commit: BTreeMap<u64, Value> = BTreeMap::new();
    for path in files {
        let Ok(bytes) = std::fs::read(&path) else {
            continue;
        };
        let mut cursor = 0;
        while let Some(pos) = bytes[cursor..]
            .windows(ARTIFACT_MAGIC.len())
            .position(|w| w == ARTIFACT_MAGIC)
        {
            let at = cursor + pos;
            cursor = at + ARTIFACT_MAGIC.len();
            if bytes.len() < at + ARTIFACT_HEADER {
                continue;
            }
            let length = u64::from_le_bytes(bytes[at + 58..at + 66].try_into().unwrap()) as usize;
            let Some(end) = at
                .checked_add(ARTIFACT_HEADER)
                .and_then(|start| start.checked_add(length))
            else {
                continue;
            };
            if end > bytes.len() {
                continue;
            }
            let frame = &bytes[at..end];
            if worth_relational::facade::durability::RelationalCanonicalCommitArtifact::preflight_frame_version(frame).is_err() {
                continue;
            }
            if let Ok(super::checkpoint_value::InspectionValue(value)) =
                rmp_serde::from_slice(&frame[ARTIFACT_HEADER..])
                && let Some(canonical) = value.get("canonical")
                && let Some(commit_id) = canonical["commit"]["commit_id"].as_u64()
            {
                by_commit.insert(commit_id, canonical.clone());
            }
        }
    }
    by_commit.into_values().collect()
}

fn collect_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(root) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_files(&path, out);
        } else {
            out.push(path);
        }
    }
}

fn find(bytes: &[u8], needle: &[u8]) -> usize {
    bytes
        .windows(needle.len())
        .position(|s| s == needle)
        .expect("actual native canonical artifact field")
}

fn value_len(bytes: &[u8]) -> usize {
    let mut decoder = rmp_serde::Deserializer::new(std::io::Cursor::new(bytes));
    serde::de::IgnoredAny::deserialize(&mut decoder).unwrap();
    decoder.position() as usize
}

fn mutate_retirement_artifacts(
    snapshot: &ClosedDirectorySnapshot,
    directory: &Path,
    mut transform: impl FnMut(&[u8]) -> Vec<u8>,
) {
    snapshot.restore_into(directory);
    let mut files = Vec::new();
    collect_files(directory, &mut files);
    let mut mutated = 0usize;
    for path in files {
        let Ok(mut data) = std::fs::read(&path) else {
            continue;
        };
        let mut changed_file = false;
        let mut cursor = 0;
        while let Some(pos) = data[cursor..]
            .windows(ARTIFACT_MAGIC.len())
            .position(|w| w == ARTIFACT_MAGIC)
        {
            let at = cursor + pos;
            cursor = at + ARTIFACT_MAGIC.len();
            if data.len() < at + ARTIFACT_HEADER {
                continue;
            }
            let length = u64::from_le_bytes(data[at + 58..at + 66].try_into().unwrap()) as usize;
            let Some(end) = at
                .checked_add(ARTIFACT_HEADER)
                .and_then(|start| start.checked_add(length))
            else {
                continue;
            };
            if end > data.len() {
                continue;
            }
            let frame = &data[at..end];
            if frame[ARTIFACT_HEADER..]
                .windows(b"producer_cascade_retirement".len())
                .any(|s| s == b"producer_cascade_retirement")
            {
                let mut replacement = transform(frame);
                assert!(
                    replacement.len() <= frame.len(),
                    "tampered frame must fit inside original durable slot"
                );
                let payload_length = (replacement.len() - ARTIFACT_HEADER) as u64;
                replacement[58..66].copy_from_slice(&payload_length.to_le_bytes());
                replacement.resize(frame.len(), 0);
                data[at..end].copy_from_slice(&replacement);
                changed_file = true;
                mutated += 1;
                cursor = end;
            }
        }
        if changed_file {
            std::fs::write(&path, data).unwrap();
        }
    }
    assert!(
        mutated > 0,
        "found durable producer_cascade_retirement artifact to mutate"
    );
}

pub fn tamper_anchor_dir(snapshot: &ClosedDirectorySnapshot, directory: &Path) {
    mutate_retirement_artifacts(snapshot, directory, |frame| {
        let mut changed = frame.to_vec();
        let group =
            ARTIFACT_HEADER + find(&changed[ARTIFACT_HEADER..], b"producer_cascade_retirement");
        let anchor = group + find(&changed[group..], b"anchors");
        let slot = anchor + find(&changed[anchor..], b"local_slot") + b"local_slot".len();
        assert!(
            changed[slot] < 128,
            "fixture anchor uses a MessagePack positive fixint"
        );
        changed[slot] ^= 1;
        changed
    });
}

pub fn omit_leaf_publication_dir(snapshot: &ClosedDirectorySnapshot, directory: &Path) {
    mutate_retirement_artifacts(snapshot, directory, |frame| {
        let group =
            ARTIFACT_HEADER + find(&frame[ARTIFACT_HEADER..], b"producer_cascade_retirement");
        let array = group + find(&frame[group..], b"publications") + b"publications".len();
        assert_eq!(
            frame[array], 0x92,
            "two separate retired publication origins"
        );
        let first = array + 1;
        let first_len = value_len(&frame[first..]);
        let second = first + first_len;
        let end = second + value_len(&frame[second..]);
        let mut changed = frame.to_vec();
        changed.drain(second..end);
        changed[array] = 0x91;
        changed
    });
}

pub fn omit_retirement_group_dir(snapshot: &ClosedDirectorySnapshot, directory: &Path) {
    mutate_retirement_artifacts(snapshot, directory, |frame| {
        let key = rmp_serde::to_vec("producer_cascade_retirement").unwrap();
        let field = ARTIFACT_HEADER + find(&frame[ARTIFACT_HEADER..], &key);
        let end = field + key.len() + value_len(&frame[field + key.len()..]);
        let map =
            ARTIFACT_HEADER + find(&frame[ARTIFACT_HEADER..], b"canonical") + b"canonical".len();
        assert_eq!(frame[map], 0xde, "current named envelope uses a map16");
        let count = u16::from_be_bytes(frame[map + 1..map + 3].try_into().unwrap());
        let mut changed = frame.to_vec();
        changed.drain(field..end);
        changed[map + 1..map + 3].copy_from_slice(&(count - 1).to_be_bytes());
        changed
    });
}

#[derive(Debug)]
struct DirectoryMemberColdSource {
    files: Vec<Vec<u8>>,
}

impl DirectoryMemberColdSource {
    fn from_dir(root: &Path) -> Arc<Self> {
        let mut paths = Vec::new();
        collect_files(root, &mut paths);
        paths.sort();
        let files = paths
            .into_iter()
            .filter_map(|path| std::fs::read(&path).ok())
            .collect();
        Arc::new(Self { files })
    }
}

impl RelationalDurableColdSource for DirectoryMemberColdSource {
    fn retain_members_for_request(
        &self,
        request: &ExecutionRequest<'_, '_>,
        _members: &[RelationalDurableMemberLocator],
        issuer: Weak<dyn RelationalDurableColdSource>,
    ) -> Result<RelationalDurableMemberCustody, RelationalDurableOwnedReadRefusal> {
        RelationalDurableMemberCustody::try_new_from_source_for_request(issuer, request, || Ok(()))
    }
    fn validate_member_custody_retirement(
        &self,
        original: &(dyn std::any::Any + Send + Sync),
    ) -> Result<(), RelationalDurableMemberRetirementRefusal> {
        original
            .is::<()>()
            .then_some(())
            .ok_or(RelationalDurableMemberRetirementRefusal::ForeignCustody)
    }
    fn advance_member_custody_retirement(
        &self,
        _request: &ExecutionRequest<'_, '_>,
        original: &mut (dyn std::any::Any + Send + Sync),
    ) -> RelationalDurableMemberRetirementObservation {
        match self.validate_member_custody_retirement(original) {
            Ok(()) => RelationalDurableMemberRetirementObservation::Complete,
            Err(cause) => RelationalDurableMemberRetirementObservation::Refused(cause),
        }
    }
    fn checkpoint_selected_read_protection(
        &self,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<RelationalDurableReadPayload, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        Ok(Box::new(()))
    }
    fn read_member_owned(
        &self,
        request: &ExecutionRequest<'_, '_>,
        original: &(dyn std::any::Any + Send + Sync),
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<ExecutionImmutableBytes, RelationalDurableOwnedReadRefusal> {
        request
            .check_live()
            .map_err(|cause| RelationalDurableOwnedReadRefusal::Request(cause.into()))?;
        if !original.is::<()>() {
            return Err(RelationalDurableOwnedReadRefusal::ForeignProtection);
        }
        let bytes = self
            .read_member(locator, limit)
            .map_err(RelationalDurableOwnedReadRefusal::Storage)?;
        let mut out = ExecutionRequestArrayBuilder::<u8>::allocate(*request, bytes.len())
            .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
        out.extend_from_slice(&bytes)
            .map_err(RelationalDurableOwnedReadRefusal::Buffer)?;
        out.seal_immutable_bytes()
            .map_err(RelationalDurableOwnedReadRefusal::Buffer)
    }
    fn retain_members(
        &self,
        _members: &[RelationalDurableMemberLocator],
    ) -> Result<RelationalDurableMemberCustody, RelationalStorageUnavailableCause> {
        Ok(RelationalDurableMemberCustody::new(()))
    }

    fn read_member(
        &self,
        locator: RelationalDurableMemberLocator,
        limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        if locator.length > limit {
            return Err(RelationalStorageUnavailableCause::LimitExceeded {
                length: locator.length,
                limit,
            });
        }
        let len = usize::try_from(locator.length)
            .map_err(|_| RelationalStorageUnavailableCause::MissingMember)?;
        const MAGICS: &[&[u8]] = &[
            b"WORTH-REL-REG8\n",
            b"WORTH-REL-MBR1\n",
            b"WRTHRCCA",
            b"WRTHRBTA",
            b"WRTHRSI1",
            b"WRTHRCS1",
        ];
        for bytes in &self.files {
            if bytes.len() < len {
                continue;
            }
            for magic in MAGICS {
                let mut cursor = 0;
                while let Some(pos) = bytes[cursor..]
                    .windows(magic.len())
                    .position(|window| window == *magic)
                {
                    let at = cursor + pos;
                    cursor = at + 1;
                    let Some(end) = at.checked_add(len) else {
                        continue;
                    };
                    if end > bytes.len() {
                        break;
                    }
                    let slice = &bytes[at..end];
                    if <[u8; 32]>::from(Sha256::digest(slice)) == locator.sha256 {
                        return Ok(slice.to_vec());
                    }
                }
            }
        }
        Err(RelationalStorageUnavailableCause::MissingMember)
    }

    fn read_commit(
        &self,
        _position: PatchStreamPosition,
        _limit: u64,
    ) -> Result<Vec<u8>, RelationalStorageUnavailableCause> {
        Err(RelationalStorageUnavailableCause::SourceUnavailable)
    }
}

pub fn assert_shrink_delta(directory: &Path, before: &[u8], after: &[u8]) {
    let lease = directory_lease();
    let request = worth_execution::ExecutionRequest::leased(&lease);
    let source = DirectoryMemberColdSource::from_dir(directory);
    let before = RelationalNativeCheckpoint::inspect_bytes_with_source(
        &request,
        before,
        Arc::clone(&source) as Arc<dyn RelationalDurableColdSource>,
    )
    .expect("inspect before Native checkpoint");
    let after = RelationalNativeCheckpoint::inspect_bytes_with_source(
        &request,
        after,
        source as Arc<dyn RelationalDurableColdSource>,
    )
    .expect("inspect after Native checkpoint");
    let removed = before
        .mirror_live_records
        .difference(&after.mirror_live_records)
        .filter(|record| !before.current_publication_records.contains(record))
        .collect::<Vec<_>>();
    let entities = removed
        .iter()
        .filter(|r| r.class == worth_relational::facade::durability::RecordAllocationClass::Entity)
        .count();
    // Two generated sources, their two realizations, two GeneratedBy and two Realizes edges.
    assert_eq!(
        entities, 4,
        "all retained roots: before={:?}, after={:?}",
        before.mirror_live_records, after.mirror_live_records
    );
    assert_eq!(removed.len() - entities, 4);
}
