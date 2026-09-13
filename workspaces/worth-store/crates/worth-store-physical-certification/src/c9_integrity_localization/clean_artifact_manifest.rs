use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::frame_checksum::{
    checksum_is_valid, FRAME_CHECKSUM_RANGE, FRAME_FORMAT_VERSION_RANGE, FRAME_HEADER_BYTES,
    FRAME_LENGTH_RANGE,
};
use super::RootArtifactRole;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct RootArtifactIdentity {
    role: RootArtifactRole,
    store_identity: [u8; 16],
    concrete_identity: u64,
    root_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CleanRootArtifactRecord {
    identity: RootArtifactIdentity,
    relative_path: PathBuf,
    substitution_source_path: PathBuf,
    substitution_source_identity: RootArtifactIdentity,
    duplicate_path: PathBuf,
    exact_length: u64,
    content_sha256: [u8; 32],
    substitution_source_sha256: [u8; 32],
    substitution_changed_ranges: Vec<Range<u64>>,
    covered_ranges: [Range<u64>; 2],
    checksum_range: Range<u64>,
    length_range: Range<u64>,
    version_range: Range<u64>,
    covered_edit_offset: u64,
    pointer_range: Range<u64>,
    expected_reachable_paths: Vec<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CleanRootArtifactManifest {
    records: BTreeMap<RootArtifactIdentity, CleanRootArtifactRecord>,
    supporting_artifacts: BTreeMap<PathBuf, [u8; 32]>,
    manifest_identity: [u8; 32],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum RootArtifactManifestDenial {
    MissingArtifact(PathBuf),
    InvalidArtifact(PathBuf),
    InvalidSubstitutionSource(PathBuf),
    InvalidSubstitutionScope(RootArtifactIdentity),
    DuplicateIdentity(RootArtifactIdentity),
    DuplicatePath(PathBuf),
    MissingRole(RootArtifactRole),
    InvalidReachability(PathBuf),
    IdentityEncoding,
}

#[derive(Debug, Clone)]
pub(crate) struct RootArtifactManifestDeclaration {
    pub(crate) identity: RootArtifactIdentity,
    pub(crate) relative_path: PathBuf,
    pub(crate) substitution_source_path: PathBuf,
    pub(crate) substitution_source_identity: RootArtifactIdentity,
    pub(crate) duplicate_path: PathBuf,
    pub(crate) covered_edit_offset: u64,
    pub(crate) pointer_range: Range<u64>,
    pub(crate) expected_reachable_paths: Vec<PathBuf>,
}

impl RootArtifactIdentity {
    pub(crate) const fn new(
        role: RootArtifactRole,
        store_identity: [u8; 16],
        concrete_identity: u64,
        root_generation: u64,
    ) -> Self {
        Self {
            role,
            store_identity,
            concrete_identity,
            root_generation,
        }
    }

    pub(crate) const fn role(self) -> RootArtifactRole {
        self.role
    }

    pub(crate) const fn store_identity(self) -> [u8; 16] {
        self.store_identity
    }

    pub(crate) const fn concrete_identity(self) -> u64 {
        self.concrete_identity
    }

    pub(crate) const fn root_generation(self) -> u64 {
        self.root_generation
    }
}

impl CleanRootArtifactManifest {
    pub(crate) fn observe(
        store_root: &Path,
        declarations: Vec<RootArtifactManifestDeclaration>,
        supporting_paths: Vec<PathBuf>,
    ) -> Result<Self, RootArtifactManifestDenial> {
        let mut records = BTreeMap::new();
        let mut paths = BTreeSet::new();
        for declaration in declarations {
            if !paths.insert(declaration.relative_path.clone())
                || !paths.insert(declaration.substitution_source_path.clone())
                || !paths.insert(declaration.duplicate_path.clone())
            {
                return Err(RootArtifactManifestDenial::DuplicatePath(
                    declaration.relative_path,
                ));
            }
            let artifact_path = store_root.join(&declaration.relative_path);
            let bytes = read_clean_frame(&artifact_path)?;
            if declaration.substitution_source_identity.role() != declaration.identity.role()
                || declaration.substitution_source_identity == declaration.identity
            {
                return Err(RootArtifactManifestDenial::InvalidSubstitutionScope(
                    declaration.identity,
                ));
            }
            let donor_path = store_root.join(&declaration.substitution_source_path);
            let donor = std::fs::read(&donor_path).map_err(|_| {
                RootArtifactManifestDenial::InvalidSubstitutionSource(donor_path.clone())
            })?;
            if !checksum_is_valid(&donor) || donor.len() != bytes.len() || donor == bytes {
                return Err(RootArtifactManifestDenial::InvalidSubstitutionSource(
                    donor_path,
                ));
            }
            let length = bytes.len() as u64;
            let record = CleanRootArtifactRecord {
                identity: declaration.identity,
                relative_path: declaration.relative_path,
                substitution_source_path: declaration.substitution_source_path,
                substitution_source_identity: declaration.substitution_source_identity,
                duplicate_path: declaration.duplicate_path,
                exact_length: length,
                content_sha256: Sha256::digest(&bytes).into(),
                substitution_source_sha256: Sha256::digest(&donor).into(),
                substitution_changed_ranges: changed_byte_ranges(&bytes, &donor),
                covered_ranges: [0..44, FRAME_HEADER_BYTES as u64..length],
                checksum_range: usize_range_to_u64(FRAME_CHECKSUM_RANGE),
                length_range: usize_range_to_u64(FRAME_LENGTH_RANGE),
                version_range: usize_range_to_u64(FRAME_FORMAT_VERSION_RANGE),
                covered_edit_offset: declaration.covered_edit_offset,
                pointer_range: declaration.pointer_range,
                expected_reachable_paths: declaration.expected_reachable_paths,
            };
            validate_record_shape(&record)?;
            if records.insert(record.identity, record).is_some() {
                return Err(RootArtifactManifestDenial::DuplicateIdentity(
                    declaration.identity,
                ));
            }
        }
        for role in [
            RootArtifactRole::CurrentSelector,
            RootArtifactRole::PreviousSelector,
            RootArtifactRole::AddressedRootManifest,
        ] {
            if !records.keys().any(|identity| identity.role() == role) {
                return Err(RootArtifactManifestDenial::MissingRole(role));
            }
        }
        let mut supporting_artifacts = BTreeMap::new();
        for relative_path in supporting_paths {
            if paths.contains(&relative_path) || supporting_artifacts.contains_key(&relative_path) {
                return Err(RootArtifactManifestDenial::DuplicatePath(relative_path));
            }
            let path = store_root.join(&relative_path);
            let bytes = read_clean_frame(&path)?;
            supporting_artifacts.insert(relative_path, Sha256::digest(bytes).into());
        }
        let known_paths: BTreeSet<_> = records
            .values()
            .flat_map(|record| {
                [
                    record.relative_path.clone(),
                    record.substitution_source_path.clone(),
                ]
            })
            .chain(supporting_artifacts.keys().cloned())
            .collect();
        for path in records
            .values()
            .flat_map(|record| record.expected_reachable_paths.iter())
        {
            if !known_paths.contains(path) {
                return Err(RootArtifactManifestDenial::InvalidReachability(
                    path.clone(),
                ));
            }
        }
        let manifest_identity = bincode::serialize(&(&records, &supporting_artifacts))
            .map(|bytes| Sha256::digest(bytes).into())
            .map_err(|_| RootArtifactManifestDenial::IdentityEncoding)?;
        Ok(Self {
            records,
            supporting_artifacts,
            manifest_identity,
        })
    }

    pub(crate) fn record(
        &self,
        identity: RootArtifactIdentity,
    ) -> Option<&CleanRootArtifactRecord> {
        self.records.get(&identity)
    }

    pub(crate) fn target_for_role(&self, role: RootArtifactRole) -> RootArtifactIdentity {
        *self
            .records
            .keys()
            .find(|identity| identity.role() == role)
            .expect("manifest construction requires every root role")
    }

    pub(crate) fn records(&self) -> impl Iterator<Item = &CleanRootArtifactRecord> {
        self.records.values()
    }

    pub(crate) fn supporting_artifacts(&self) -> impl Iterator<Item = (&Path, [u8; 32])> {
        self.supporting_artifacts
            .iter()
            .map(|(path, digest)| (path.as_path(), *digest))
    }

    pub(crate) const fn identity(&self) -> [u8; 32] {
        self.manifest_identity
    }
}

impl CleanRootArtifactRecord {
    pub(crate) const fn identity(&self) -> RootArtifactIdentity {
        self.identity
    }
    pub(crate) fn relative_path(&self) -> &Path {
        &self.relative_path
    }
    pub(crate) fn substitution_source_path(&self) -> &Path {
        &self.substitution_source_path
    }
    pub(crate) const fn substitution_source_identity(&self) -> RootArtifactIdentity {
        self.substitution_source_identity
    }
    pub(crate) fn duplicate_path(&self) -> &Path {
        &self.duplicate_path
    }
    pub(crate) const fn exact_length(&self) -> u64 {
        self.exact_length
    }
    pub(crate) const fn content_sha256(&self) -> [u8; 32] {
        self.content_sha256
    }
    pub(crate) const fn substitution_source_sha256(&self) -> [u8; 32] {
        self.substitution_source_sha256
    }
    pub(crate) fn substitution_changed_ranges(&self) -> &[Range<u64>] {
        &self.substitution_changed_ranges
    }
    pub(crate) fn covered_ranges(&self) -> &[Range<u64>; 2] {
        &self.covered_ranges
    }
    pub(crate) fn checksum_range(&self) -> Range<u64> {
        self.checksum_range.clone()
    }
    pub(crate) fn length_range(&self) -> Range<u64> {
        self.length_range.clone()
    }
    pub(crate) fn version_range(&self) -> Range<u64> {
        self.version_range.clone()
    }
    pub(crate) const fn covered_edit_offset(&self) -> u64 {
        self.covered_edit_offset
    }
    pub(crate) fn pointer_range(&self) -> Range<u64> {
        self.pointer_range.clone()
    }
    pub(crate) fn expected_reachable_paths(&self) -> &[PathBuf] {
        &self.expected_reachable_paths
    }
}

fn read_clean_frame(path: &Path) -> Result<Vec<u8>, RootArtifactManifestDenial> {
    let bytes = std::fs::read(path)
        .map_err(|_| RootArtifactManifestDenial::MissingArtifact(path.to_path_buf()))?;
    if !checksum_is_valid(&bytes) {
        return Err(RootArtifactManifestDenial::InvalidArtifact(
            path.to_path_buf(),
        ));
    }
    Ok(bytes)
}

fn validate_record_shape(
    record: &CleanRootArtifactRecord,
) -> Result<(), RootArtifactManifestDenial> {
    let length = record.exact_length;
    let covered_offset_is_valid = record
        .covered_ranges
        .iter()
        .any(|range| range.contains(&record.covered_edit_offset));
    let pointer_is_covered = record.covered_ranges.iter().any(|range| {
        range.start <= record.pointer_range.start && range.end >= record.pointer_range.end
    });
    if length < FRAME_HEADER_BYTES as u64
        || !covered_offset_is_valid
        || record.checksum_range.contains(&record.covered_edit_offset)
        || !pointer_is_covered
        || record.pointer_range.end > length
    {
        return Err(RootArtifactManifestDenial::InvalidArtifact(
            record.relative_path.clone(),
        ));
    }
    Ok(())
}

fn usize_range_to_u64(range: Range<usize>) -> Range<u64> {
    range.start as u64..range.end as u64
}

fn changed_byte_ranges(before: &[u8], after: &[u8]) -> Vec<Range<u64>> {
    let mut ranges = Vec::new();
    let mut start = None;
    for index in 0..before.len() {
        if before[index] != after[index] {
            start.get_or_insert(index);
        } else if let Some(range_start) = start.take() {
            ranges.push(range_start as u64..index as u64);
        }
    }
    if let Some(range_start) = start {
        ranges.push(range_start as u64..before.len() as u64);
    }
    ranges
}
