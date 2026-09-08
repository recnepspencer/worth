//! Literal checksum-reference coordinates in the immutable production world.
//! This graph is fixture evidence, not an integrity verdict or runtime authority.
use super::{ArtifactInventory, frame_checksum};
use std::{collections::BTreeSet, path::Path};
use worth_store_physical_format::{PhysicalArtifactReadTarget, RecordArtifactFile};

#[derive(Clone, Copy)]
pub(super) struct Edge { pub(super) parent:usize, pub(super) child:usize, pub(super) checksum:usize }

pub(super) fn graph(inventory:&ArtifactInventory, baseline:&Path)->Vec<Edge> {
    let mut edges=Vec::new();
    for (parent,granule) in inventory.granules.iter().enumerate() {
        let bytes=std::fs::read(baseline.join(&granule.path)).unwrap();
        match granule.family {
            "root_manifest" => {
                for (flag,offset,family) in [(88,96,"root_routing_block"),(208,216,"segment_membership_block"),(280,288,"free_space_membership_block")] {
                    if bytes[flag]==1 { add_reference(&mut edges,inventory,parent,&bytes,offset,family); }
                }
                let generation=u64_at(&bytes,48);
                let child=inventory.granules.iter().position(|candidate|candidate.target==PhysicalArtifactReadTarget::Record(RecordArtifactFile::FreeSpaceManifest{generation})).unwrap();
                edges.push(Edge{parent,child,checksum:200});
            }
            "free_space_header" => if bytes[112]==1 {add_reference(&mut edges,inventory,parent,&bytes,120,"free_space_membership_block");},
            "root_routing_block" | "segment_membership_block" | "free_space_membership_block" if bytes[68]==2 => {
                let width=if granule.family=="root_routing_block"{72}else{56};
                let count=u16::from_le_bytes(bytes[66..68].try_into().unwrap()) as usize;
                for index in 0..count {add_reference(&mut edges,inventory,parent,&bytes,88+index*width,granule.family);}
            }
            _=>{}
        }
    }
    edges
}
fn add_reference(edges:&mut Vec<Edge>,inventory:&ArtifactInventory,parent:usize,bytes:&[u8],offset:usize,family:&str) {
    let generation=u64_at(bytes,offset);let block=u64_at(bytes,offset+8);
    let child=inventory.granules.iter().position(|candidate| {
        if candidate.family!=family{return false;}
        match candidate.target {
            PhysicalArtifactReadTarget::Record(RecordArtifactFile::RootRoutingBlock{generation:g,block:b}|RecordArtifactFile::SegmentMembershipBlock{generation:g,block:b}|RecordArtifactFile::FreeSpaceMembershipBlock{generation:g,block:b}) => (g,b)==(generation,block),
            _=>false
        }
    }).unwrap();
    edges.push(Edge{parent,child,checksum:offset+20});
}

pub(super) fn affected(edges:&[Edge],index:usize)->BTreeSet<usize> {
    let mut result=BTreeSet::from([index]);
    loop {let old=result.len(); for edge in edges {if result.contains(&edge.child){result.insert(edge.parent);}}
        if old==result.len(){return result;}}
}
pub(super) fn refresh(root:&Path,inventory:&ArtifactInventory,edges:&[Edge],child:usize) {
    let bytes=std::fs::read(root.join(&inventory.granules[child].path)).unwrap();
    let checksum=complete_checksum(&bytes);
    for edge in edges.iter().filter(|edge|edge.child==child) {
        let path=root.join(&inventory.granules[edge.parent].path);
        let mut parent=std::fs::read(&path).unwrap();
        parent[edge.checksum..edge.checksum+4].copy_from_slice(&checksum.to_le_bytes());
        frame_checksum::refresh_checksum(&mut parent);
        std::fs::write(path,parent).unwrap();
        refresh(root,inventory,edges,edge.parent);
    }
}
pub(super) fn audit(root:&Path,baseline:&Path,inventory:&ArtifactInventory,edges:&[Edge],target:usize) {
    let affected=affected(edges,target);
    for index in affected.iter().copied().filter(|index|*index!=target) {
        let path=&inventory.granules[index].path;
        let before=std::fs::read(baseline.join(path)).unwrap();let after=std::fs::read(root.join(path)).unwrap();
        assert_eq!(before.len(),after.len());
        let mut allowed=BTreeSet::from_iter(44..48);
        for edge in edges.iter().filter(|edge|edge.parent==index && affected.contains(&edge.child)) {
            allowed.extend(edge.checksum..edge.checksum+4);
            let child=std::fs::read(root.join(&inventory.granules[edge.child].path)).unwrap();
            assert_eq!(u32::from_le_bytes(after[edge.checksum..edge.checksum+4].try_into().unwrap()),complete_checksum(&child));
        }
        assert!(before.iter().zip(&after).enumerate().all(|(offset,(a,b))|a==b||allowed.contains(&offset)));
        assert!(frame_checksum::checksum_is_valid(&after));
    }
}
pub(super) fn complete_checksum(bytes:&[u8])->u32 {
    let mut crc=!0u32;for byte in bytes {crc^=u32::from(*byte);for _ in 0..8 {crc=(crc>>1)^(0x82f6_3b78 & (crc&1).wrapping_neg());}}!crc
}
fn u64_at(bytes:&[u8],offset:usize)->u64 {u64::from_le_bytes(bytes[offset..offset+8].try_into().unwrap())}
