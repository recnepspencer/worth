use std::path::{Path, PathBuf};

use worth_store_physical_format::{
    durable_artifact_checksum, DurableFreeSpaceManifestHeader, DurablePhysicalRootManifest,
    RecordArtifactFile,
};

pub(super) fn candidate_topology(root: &Path, generation: u64) -> Vec<(String, Vec<u8>)> {
    let generation = format!("{generation:016x}");
    let records = root.join("families/records");
    let mut files = Vec::new();
    collect_matching(
        &records.join("roots"),
        &format!("root-{generation}"),
        &mut files,
    );
    collect_matching(
        &records.join("segment-manifests"),
        &format!("segments-{generation}"),
        &mut files,
    );
    collect_matching(
        &records.join("free-space"),
        &format!("free-space-{generation}"),
        &mut files,
    );
    files.sort_by(|left, right| left.0.cmp(&right.0));
    assert!(
        !files.is_empty(),
        "writer crash must leave successor topology"
    );
    files
}

fn collect_matching(directory: &Path, prefix: &str, files: &mut Vec<(String, Vec<u8>)>) {
    for entry in std::fs::read_dir(directory).expect("enumerate candidate topology") {
        let path = entry.expect("candidate topology entry").path();
        let name = path.file_name().unwrap().to_string_lossy();
        if path.is_file() && name.starts_with(prefix) {
            files.push((
                name.into_owned(),
                std::fs::read(&path).expect("read candidate topology bytes"),
            ));
        }
    }
}

pub(super) fn selected_root_bytes(root: &Path, generation: u64) -> (Vec<u8>, Vec<u8>) {
    let records = root.join("families/records");
    let selector = std::fs::read(records.join("root-current.selector"))
        .expect("read selected current selector");
    let manifest = std::fs::read(
        records
            .join("roots")
            .join(RecordArtifactFile::RootManifest { generation }.file_name()),
    )
    .expect("read selected root manifest");
    (selector, manifest)
}

pub(super) fn mutate_candidate(root: &Path, generation: u64, hostile: &str) {
    if matches!(
        hostile,
        "root-routing-child" | "segment-membership-child" | "free-space-child"
    ) {
        corrupt_child(root, generation, hostile);
        return;
    }
    let path = candidate_root_path(root, generation);
    let mut bytes = std::fs::read(&path).expect("read successor root candidate");
    if hostile == "malformed" {
        std::fs::write(path, vec![0_u8; bytes.len()]).expect("write malformed candidate");
        return;
    }
    if hostile == "noncanonical-root" {
        mutate_frame_metadata(&mut bytes);
        std::fs::write(path, bytes).expect("write noncanonical root candidate");
        return;
    }
    if hostile == "noncanonical-free-header" {
        mutate_free_header(root, generation, &bytes);
        return;
    }
    let (candidate, format) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX)
        .expect("decode successor root candidate");
    if hostile == "selected-routing-root" {
        let selected_path = candidate_root_path(root, generation - 1);
        let (selected, selected_format) = DurablePhysicalRootManifest::decode(
            &std::fs::read(selected_path).expect("read selected root"),
            u16::MAX,
        )
        .expect("decode selected root");
        assert_eq!(selected_format, format);
        let stale = DurablePhysicalRootManifest::builder(
            candidate.generation(),
            candidate.tree_identity(),
            candidate.node_capacity(),
            candidate.free_space_checksum(),
        )
        .record_count(candidate.record_count())
        .next_block(selected.next_block())
        .next_segment_block(candidate.next_segment_block())
        .routing_root(selected.routing_root())
        .segment_root(candidate.segment_root())
        .free_space_root(candidate.free_space_root())
        .last_inline_record(candidate.last_inline_record())
        .last_inline_segment(candidate.last_inline_segment())
        .admit()
        .expect("stale candidate remains structurally encodable");
        std::fs::write(path, stale.encode(format)).expect("write stale candidate root");
        return;
    }
    let tree_identity = if hostile == "conflicting" {
        candidate.tree_identity() + 1
    } else {
        candidate.tree_identity()
    };
    let next_block = if hostile == "inflated" {
        candidate.next_block() + 1
    } else {
        candidate.next_block()
    };
    let mutated = DurablePhysicalRootManifest::builder(
        candidate.generation(),
        tree_identity,
        candidate.node_capacity(),
        candidate.free_space_checksum(),
    )
    .record_count(candidate.record_count())
    .next_block(next_block)
    .next_segment_block(candidate.next_segment_block())
    .routing_root(candidate.routing_root())
    .segment_root(candidate.segment_root())
    .free_space_root(candidate.free_space_root())
    .last_inline_record(candidate.last_inline_record())
    .last_inline_segment(candidate.last_inline_segment())
    .admit()
    .expect("hostile candidate remains structurally encodable");
    std::fs::write(path, mutated.encode(format)).expect("write hostile candidate");
}

fn mutate_free_header(root: &Path, generation: u64, root_bytes: &[u8]) {
    let (candidate, format) = DurablePhysicalRootManifest::decode(root_bytes, u16::MAX).unwrap();
    let header_path = root
        .join("families/records/free-space")
        .join(RecordArtifactFile::FreeSpaceManifest { generation }.file_name());
    let mut header_bytes = std::fs::read(&header_path).unwrap();
    mutate_frame_metadata(&mut header_bytes);
    std::fs::write(header_path, &header_bytes).unwrap();
    let rebound = DurablePhysicalRootManifest::builder(
        candidate.generation(),
        candidate.tree_identity(),
        candidate.node_capacity(),
        durable_artifact_checksum(&header_bytes),
    )
    .record_count(candidate.record_count())
    .next_block(candidate.next_block())
    .next_segment_block(candidate.next_segment_block())
    .routing_root(candidate.routing_root())
    .segment_root(candidate.segment_root())
    .free_space_root(candidate.free_space_root())
    .last_inline_record(candidate.last_inline_record())
    .last_inline_segment(candidate.last_inline_segment())
    .admit()
    .unwrap();
    std::fs::write(
        candidate_root_path(root, generation),
        rebound.encode(format),
    )
    .unwrap();
}

fn mutate_frame_metadata(bytes: &mut [u8]) {
    bytes[36..44].copy_from_slice(&1_u64.to_le_bytes());
    let checksum = crc32c_parts(&[&bytes[..44], &bytes[48..]]);
    bytes[44..48].copy_from_slice(&checksum.to_le_bytes());
}

fn crc32c_parts(parts: &[&[u8]]) -> u32 {
    let mut value = !0_u32;
    for byte in parts.iter().flat_map(|part| part.iter()) {
        value ^= u32::from(*byte);
        for _ in 0..8 {
            let mask = 0_u32.wrapping_sub(value & 1);
            value = (value >> 1) ^ (0x82f6_3b78 & mask);
        }
    }
    !value
}

fn corrupt_child(root: &Path, generation: u64, hostile: &str) {
    let artifact = hostile_artifact(root, generation, hostile);
    let child = artifact_path(root, artifact);
    let length = std::fs::metadata(&child).unwrap().len() as usize;
    std::fs::write(child, vec![0_u8; length]).expect("corrupt candidate child");
}

pub(super) fn hostile_artifact(root: &Path, generation: u64, hostile: &str) -> RecordArtifactFile {
    let bytes = std::fs::read(candidate_root_path(root, generation)).unwrap();
    let (candidate, format) = DurablePhysicalRootManifest::decode(&bytes, u16::MAX).unwrap();
    match hostile {
        "root-routing-child" => {
            let reference = candidate.routing_root().expect("candidate routing root");
            assert_eq!(reference.generation(), generation);
            RecordArtifactFile::RootRoutingBlock {
                generation,
                block: reference.block(),
            }
        }
        "segment-membership-child" => {
            let reference = candidate.segment_root().expect("candidate segment root");
            assert_eq!(reference.generation(), generation);
            RecordArtifactFile::SegmentMembershipBlock {
                generation,
                block: reference.block(),
            }
        }
        "free-space-child" => {
            let header_path = root
                .join("families/records/free-space")
                .join(RecordArtifactFile::FreeSpaceManifest { generation }.file_name());
            let (header, found_format) = DurableFreeSpaceManifestHeader::decode(
                &std::fs::read(header_path).unwrap(),
                candidate.node_capacity(),
            )
            .unwrap();
            assert_eq!(found_format, format);
            let reference = header.root().expect("candidate free-space root");
            assert_eq!(reference.generation(), generation);
            RecordArtifactFile::FreeSpaceMembershipBlock {
                generation,
                block: reference.block(),
            }
        }
        _ => RecordArtifactFile::RootManifest { generation },
    }
}

fn artifact_path(root: &Path, artifact: RecordArtifactFile) -> PathBuf {
    let records = root.join("families/records");
    match artifact {
        RecordArtifactFile::RootManifest { .. } | RecordArtifactFile::RootRoutingBlock { .. } => {
            records.join("roots").join(artifact.file_name())
        }
        RecordArtifactFile::SegmentMembershipBlock { .. } => {
            records.join("segment-manifests").join(artifact.file_name())
        }
        RecordArtifactFile::FreeSpaceManifest { .. }
        | RecordArtifactFile::FreeSpaceMembershipBlock { .. } => {
            records.join("free-space").join(artifact.file_name())
        }
        _ => unreachable!("successor topology artifact"),
    }
}

pub(super) fn candidate_root_path(root: &Path, generation: u64) -> PathBuf {
    root.join("families/records/roots")
        .join(RecordArtifactFile::RootManifest { generation }.file_name())
}

pub(super) fn remove_candidate_topology(root: &Path, generation: u64) {
    let generation = format!("{generation:016x}");
    let records = root.join("families/records");
    for (directory, prefix) in [
        (records.join("roots"), format!("root-{generation}")),
        (
            records.join("segment-manifests"),
            format!("segments-{generation}"),
        ),
        (
            records.join("free-space"),
            format!("free-space-{generation}"),
        ),
    ] {
        for entry in std::fs::read_dir(directory).expect("enumerate successor topology") {
            let path = entry.expect("successor topology entry").path();
            if path.is_file()
                && path
                    .file_name()
                    .is_some_and(|name| name.to_string_lossy().starts_with(&prefix))
            {
                std::fs::remove_file(path).expect("remove successor candidate artifact");
            }
        }
    }
}
