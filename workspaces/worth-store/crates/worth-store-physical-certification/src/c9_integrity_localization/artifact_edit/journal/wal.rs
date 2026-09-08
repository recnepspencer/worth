use super::Op;
use sha2::{Digest, Sha256};

pub(super) fn edit(frame: &mut [u8], operator: Op) {
    let footer = frame.len()-32;
    match operator {
        Op::CoveredByte => { frame[116] ^= 1; return; }
        Op::Checksum => frame[84] ^= 1,
        Op::Length => frame[10..12].copy_from_slice(&117_u16.to_le_bytes()),
        Op::ScopeSubstitution => {
            let segment = u64::from_le_bytes(frame[12..20].try_into().unwrap());
            frame[12..20].copy_from_slice(&(segment+1).to_le_bytes());
        }
        Op::EnvelopeVersion => frame[8..10].copy_from_slice(&2_u16.to_le_bytes()),
        _ => unreachable!("declared WAL frame operator"),
    }
    let checksum = Sha256::digest(&frame[..footer]);
    frame[footer..].copy_from_slice(&checksum);
}

pub(super) fn audit(before: &[u8], after: &[u8], operator: Op) {
    let footer = before.len()-32;
    assert_eq!(before.len(), after.len());
    let allowed = match operator {
        Op::CoveredByte => 116..117,
        Op::Checksum => 84..85,
        Op::Length => 10..12,
        Op::ScopeSubstitution => 12..20,
        Op::EnvelopeVersion => 8..10,
        _ => unreachable!(),
    };
    let changes: Vec<_> = before.iter().zip(after).enumerate()
        .filter_map(|(index,(a,b))| (a!=b).then_some(index)).collect();
    assert!(!changes.is_empty());
    assert!(changes.iter().all(|index| allowed.contains(index)
        || (operator!=Op::CoveredByte && *index>=footer)));
    let payload_valid = Sha256::digest(&after[116..footer]).as_slice() == &after[84..116];
    let footer_valid = Sha256::digest(&after[..footer]).as_slice() == &after[footer..];
    assert_eq!(payload_valid, !matches!(operator,Op::CoveredByte|Op::Checksum));
    assert_eq!(footer_valid, operator!=Op::CoveredByte);
    match operator {
        Op::CoveredByte => assert_eq!(changes, [116]),
        Op::Checksum => assert_eq!(after[84], before[84]^1),
        Op::Length => assert_eq!(u16::from_le_bytes(after[10..12].try_into().unwrap()),117),
        Op::ScopeSubstitution => assert_eq!(u64::from_le_bytes(after[12..20].try_into().unwrap()),
            u64::from_le_bytes(before[12..20].try_into().unwrap())+1),
        Op::EnvelopeVersion => assert_eq!(u16::from_le_bytes(after[8..10].try_into().unwrap()),2),
        _ => unreachable!(),
    }
}
