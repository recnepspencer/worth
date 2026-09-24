use std::collections::{BTreeMap, BTreeSet};

use serde::ser::{SerializeSeq, SerializeStruct};
use serde::{Deserialize, Serialize};

use crate::durability::data::{
    branch_root_partition_image_digest, DurableBranchRootImage, DurableCheckpoint,
    PartitionCheckpointImage, RecoveryFailureClass,
};
use crate::history::data::CommitId;
use crate::identity::data::PartitionId;

use super::PersistedDurableCheckpoint;

pub(super) const PARTITION_ALIAS_FORMAT_VERSION: u16 = 1;
pub(super) const PARTITION_DELTA_FORMAT_VERSION: u16 = 2;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct SharedPartitionAlias {
    pub(super) position: u32,
    pub(super) partition_id: PartitionId,
    pub(super) image_digest: [u8; 32],
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct RootPartitionAliases {
    pub(super) commit_id: CommitId,
    pub(super) shared: Vec<SharedPartitionAlias>,
}

pub(super) struct PartitionAliasPlan {
    pub(super) roots: Vec<RootPartitionAliases>,
}

impl PartitionAliasPlan {
    pub(super) fn for_checkpoint(
        checkpoint: &DurableCheckpoint,
    ) -> Result<Self, rmp_serde::encode::Error> {
        let mirror = checkpoint
            .partition_images
            .iter()
            .map(|image| (image.partition_id, image))
            .collect::<BTreeMap<_, _>>();
        let mut roots = Vec::with_capacity(checkpoint.branch_roots.len());
        for root in &checkpoint.branch_roots {
            let mut shared = Vec::new();
            for (position, image) in root.partition_images.iter().enumerate() {
                if mirror.get(&image.partition_id).copied() == Some(image) {
                    let position = u32::try_from(position).map_err(|_| {
                        rmp_serde::encode::Error::Syntax(
                            "partition alias position exceeds u32".to_string(),
                        )
                    })?;
                    shared.push(SharedPartitionAlias {
                        position,
                        partition_id: image.partition_id,
                        image_digest: partition_digest(image)?,
                    });
                }
            }
            if !shared.is_empty() {
                roots.push(RootPartitionAliases {
                    commit_id: root.commit_id,
                    shared,
                });
            }
        }
        Ok(Self { roots })
    }
}

fn partition_digest(
    image: &PartitionCheckpointImage,
) -> Result<[u8; 32], rmp_serde::encode::Error> {
    branch_root_partition_image_digest(std::slice::from_ref(image))
}

pub(super) struct CheckpointBranchRootRefs<'a> {
    pub(super) roots: &'a [DurableBranchRootImage],
    pub(super) aliases: &'a [RootPartitionAliases],
}

impl Serialize for CheckpointBranchRootRefs<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut sequence = serializer.serialize_seq(Some(self.roots.len()))?;
        for root in self.roots {
            let aliases = self
                .aliases
                .iter()
                .find(|aliases| aliases.commit_id == root.commit_id)
                .map_or(&[][..], |aliases| aliases.shared.as_slice());
            sequence.serialize_element(&CheckpointBranchRootRef { root, aliases })?;
        }
        sequence.end()
    }
}

struct CheckpointBranchRootRef<'a> {
    root: &'a DurableBranchRootImage,
    aliases: &'a [SharedPartitionAlias],
}

impl Serialize for CheckpointBranchRootRef<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let DurableBranchRootImage {
            format_version,
            commit_id,
            partition_images,
            partition_image_digest,
            schema_carrier_digest,
            root_image_digest,
        } = self.root;
        let omitted = self
            .aliases
            .iter()
            .map(|alias| alias.position as usize)
            .collect::<BTreeSet<_>>();
        let inline = partition_images
            .iter()
            .enumerate()
            .filter_map(|(position, image)| (!omitted.contains(&position)).then_some(image))
            .collect::<Vec<_>>();
        let mut fields = serializer.serialize_struct("DurableBranchRootImage", 6)?;
        fields.serialize_field("format_version", format_version)?;
        fields.serialize_field("commit_id", commit_id)?;
        fields.serialize_field("partition_images", &inline)?;
        fields.serialize_field("partition_image_digest", partition_image_digest)?;
        fields.serialize_field("schema_carrier_digest", schema_carrier_digest)?;
        fields.serialize_field("root_image_digest", root_image_digest)?;
        fields.end()
    }
}

pub(super) fn readmit_partition_aliases(
    checkpoint: &mut PersistedDurableCheckpoint,
) -> Result<(), crate::durability::data::DurabilityError> {
    match checkpoint.partition_alias_format {
        0 if checkpoint.branch_root_partition_aliases.is_empty()
            && checkpoint.branch_root_partition_aliases_v2.is_empty() =>
        {
            Ok(())
        }
        PARTITION_ALIAS_FORMAT_VERSION
            if checkpoint.branch_root_partition_aliases_v2.is_empty() =>
        {
            readmit_whole_root_aliases(checkpoint)
        }
        PARTITION_DELTA_FORMAT_VERSION if checkpoint.branch_root_partition_aliases.is_empty() => {
            readmit_partition_deltas(checkpoint)
        }
        _ => Err(corrupt("unsupported checkpoint partition alias format")),
    }
}

fn readmit_whole_root_aliases(
    checkpoint: &mut PersistedDurableCheckpoint,
) -> Result<(), crate::durability::data::DurabilityError> {
    let aliases = &checkpoint.branch_root_partition_aliases;
    let mut pending = aliases.iter().copied().collect::<BTreeSet<_>>();
    if pending.len() != aliases.len() {
        return Err(corrupt("duplicate branch-root partition alias"));
    }
    for root in &mut checkpoint.branch_roots {
        if pending.remove(&root.commit_id) {
            if !root.partition_images.is_empty() {
                return Err(corrupt(
                    "branch-root partition alias carries an inline image",
                ));
            }
            root.partition_images = checkpoint.partition_images.clone();
        }
    }
    if !pending.is_empty() {
        return Err(corrupt("branch-root partition alias names missing root"));
    }
    Ok(())
}

fn readmit_partition_deltas(
    checkpoint: &mut PersistedDurableCheckpoint,
) -> Result<(), crate::durability::data::DurabilityError> {
    let mirror = checkpoint
        .partition_images
        .iter()
        .map(|image| (image.partition_id, image))
        .collect::<BTreeMap<_, _>>();
    if mirror.len() != checkpoint.partition_images.len() {
        return Err(corrupt("checkpoint mirror has duplicate partition images"));
    }
    let mut aliases = BTreeMap::new();
    for root in &checkpoint.branch_root_partition_aliases_v2 {
        if aliases
            .insert(root.commit_id, root.shared.as_slice())
            .is_some()
        {
            return Err(corrupt("duplicate branch-root partition delta"));
        }
    }
    for root in &mut checkpoint.branch_roots {
        let Some(shared) = aliases.remove(&root.commit_id) else {
            continue;
        };
        let total = root
            .partition_images
            .len()
            .checked_add(shared.len())
            .ok_or_else(|| corrupt("branch-root partition delta length overflow"))?;
        let mut ordered = std::iter::repeat_with(|| None)
            .take(total)
            .collect::<Vec<Option<PartitionCheckpointImage>>>();
        let mut seen_ids = BTreeSet::new();
        for alias in shared {
            let position = alias.position as usize;
            let slot = ordered
                .get_mut(position)
                .ok_or_else(|| corrupt("branch-root partition alias position is out of range"))?;
            if slot.is_some() || !seen_ids.insert(alias.partition_id) {
                return Err(corrupt(
                    "duplicate branch-root partition alias position or identity",
                ));
            }
            let image = mirror
                .get(&alias.partition_id)
                .ok_or_else(|| corrupt("branch-root partition alias names missing mirror image"))?;
            if partition_digest(image).map_err(|_| corrupt("partition alias digest failed"))?
                != alias.image_digest
            {
                return Err(corrupt("branch-root shared partition digest mismatch"));
            }
            *slot = Some((*image).clone());
        }
        let inline = std::mem::take(&mut root.partition_images);
        let mut inline = inline.into_iter();
        for slot in &mut ordered {
            if slot.is_none() {
                let image = inline
                    .next()
                    .ok_or_else(|| corrupt("branch-root partition delta is incomplete"))?;
                if !seen_ids.insert(image.partition_id) {
                    return Err(corrupt(
                        "branch-root partition alias collides with inline image",
                    ));
                }
                *slot = Some(image);
            }
        }
        if inline.next().is_some() {
            return Err(corrupt(
                "branch-root partition delta has extra inline images",
            ));
        }
        root.partition_images = ordered.into_iter().map(Option::unwrap).collect();
    }
    if !aliases.is_empty() {
        return Err(corrupt("branch-root partition delta names missing root"));
    }
    Ok(())
}

fn corrupt(detail: &str) -> crate::durability::data::DurabilityError {
    crate::durability::data::DurabilityError::new(RecoveryFailureClass::CorruptCheckpoint, detail)
}
