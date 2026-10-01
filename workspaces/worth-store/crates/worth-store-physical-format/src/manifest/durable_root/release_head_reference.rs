use super::{ReleaseCustodyHeadBlockReferenceV1, RootManifestDenial};

pub(super) type DecodedHeadBinding = (
    Option<[u8; 32]>,
    Option<ReleaseCustodyHeadBlockReferenceV1>,
    u64,
    bool,
);

pub(super) fn encode(
    payload: &mut [u8],
    root: Option<ReleaseCustodyHeadBlockReferenceV1>,
    frontier: u64,
    maintenance: bool,
) {
    payload[560] = u8::from(maintenance);
    payload[568..576].copy_from_slice(&frontier.to_le_bytes());
    if let Some(reference) = root {
        payload[561] = 1;
        reference.encode_into((&mut payload[576..680]).try_into().unwrap());
    }
}

pub(super) fn decode(schema: u8, payload: &[u8]) -> Result<DecodedHeadBinding, RootManifestDenial> {
    let tier_anchor = if schema >= 9 {
        let anchor: [u8; 32] = payload[528..560].try_into().unwrap();
        if anchor == [0; 32] && schema == 9 {
            return Err(RootManifestDenial::MalformedPrefix);
        }
        (anchor != [0; 32]).then_some(anchor)
    } else {
        None
    };
    if schema != 10 {
        return Ok((tier_anchor, None, 1, false));
    }
    if payload[562..568] != [0; 6] {
        return Err(RootManifestDenial::ReservedFieldNonZero);
    }
    let maintenance = match payload[560] {
        0 => false,
        1 => true,
        _ => return Err(RootManifestDenial::MalformedPrefix),
    };
    let root = match payload[561] {
        0 if payload[576..680] == [0; 104] => None,
        1 => Some(
            ReleaseCustodyHeadBlockReferenceV1::decode(&payload[576..680])
                .map_err(|_| RootManifestDenial::InvalidPlacement)?,
        ),
        _ => return Err(RootManifestDenial::MalformedPrefix),
    };
    let frontier = u64::from_le_bytes(payload[568..576].try_into().unwrap());
    if (root.is_none() && frontier == 1) || (tier_anchor.is_some() && !maintenance) {
        return Err(RootManifestDenial::MalformedPrefix);
    }
    Ok((tier_anchor, root, frontier, maintenance))
}
