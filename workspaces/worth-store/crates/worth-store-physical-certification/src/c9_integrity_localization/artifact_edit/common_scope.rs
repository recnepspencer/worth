use super::{ArtifactGranule,ArtifactInventory,frame_checksum};
use std::{ops::Range,path::Path};

pub(super) fn substitute(bytes:&mut [u8],baseline:&Path,inventory:&ArtifactInventory,index:usize) {
    let target=&inventory.granules[index];let start=target.offset();let end=start+target.length();
    if target.family=="inline_page" {
        let donor=inventory.granules.iter().find(|candidate|candidate.family=="inline_page" && candidate.scope.page_identity()!=target.scope.page_identity() && candidate.length()==target.length()).unwrap();
        let source=std::fs::read(baseline.join(&donor.path)).unwrap();
        bytes[start..end].copy_from_slice(&source[donor.offset()..donor.offset()+donor.length()]);
        return;
    }
    let frame=&mut bytes[start..end];
    // Re-encode only the concrete identity, retaining valid nonzero coordinates.
    for range in substitution_fields(target) {
        if range.len()==16 {frame[range.start]^=0x80;} else {
            let value=u64::from_le_bytes(frame[range.clone()].try_into().unwrap());
            frame[range].copy_from_slice(&value.checked_add(1).unwrap().to_le_bytes());
        }
    }
    frame_checksum::refresh_checksum(frame);
}
pub(super) fn substitution_fields(target:&ArtifactGranule)->Vec<Range<usize>> {
    match target.family {
        "bootstrap_catalog"|"current_root_selector"|"previous_root_selector"=>vec![48..64],
        "root_manifest"=>vec![28..36,48..56],
        "root_routing_block"|"segment_membership_block"|"free_space_membership_block"=>vec![72..80],
        "free_space_header"=>vec![28..36,48..56],
        "extent_manifest"=>vec![28..36],
        "extent_chunk"=>vec![80..88],
        "inline_page"=>vec![0..target.length()],
        other=>panic!("missing concrete substitution for {other}")
    }
}
pub(super) fn pointer_field(target:&ArtifactGranule)->Range<usize> {
    match target.family {
        "current_root_selector"|"previous_root_selector"=>65..73,
        "root_manifest"=>96..104,
        "root_routing_block"|"segment_membership_block"=>88..96,
        "free_space_header"=>128..136,
        // A leaf names the allocation owner's concrete extent rather than a child block.
        "free_space_membership_block"=>96..104,
        "extent_manifest"=>72..80,
        other=>panic!("no pointer in {other}")
    }
}
pub(super) fn corrupt_pointer(bytes:&mut [u8],target:&ArtifactGranule) {
    let start=target.offset();let end=start+target.length();
    let range=pointer_field(target);
    let frame=&mut bytes[start..end];
    match target.family {
        "root_manifest"|"root_routing_block"|"segment_membership_block"=>{
            let parent_generation=if target.family=="root_manifest" {48}else{72};
            let value=u64::from_le_bytes(frame[parent_generation..parent_generation+8].try_into().unwrap())+1;
            frame[range].copy_from_slice(&value.to_le_bytes());
        }
        "free_space_membership_block"=>{
            let value=u64::from_le_bytes(frame[range.clone()].try_into().unwrap())+1;
            frame[range].copy_from_slice(&value.to_le_bytes());
        }
        _=>frame[range].fill(0),
    }
    frame_checksum::refresh_checksum(&mut bytes[start..end]);
}
