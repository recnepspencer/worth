use super::*;

pub(super) type DecodedPublicationFields = (
    Option<IndexedThroughBlobPublication>,
    Option<DerivedFamilyRootDirectoryBinding>,
    Option<PersistedRecordIdentity>,
);

pub(super) fn decode(
    schema: u8,
    payload: &[u8],
) -> Result<DecodedPublicationFields, RootManifestDenial> {
    match schema {
        2 | 3 if payload.len() == LEGACY_ROOT_PAYLOAD_BYTES => Ok((None, None, None)),
        4 | 5 | 6 | 7 | 9 | 10
            if payload.len()
                == if schema == 10 {
                    HEAD_BOUND_ROOT_PAYLOAD_BYTES
                } else if schema == 9 {
                    TIER_ANCHORED_ROOT_PAYLOAD_BYTES
                } else if schema >= 6 {
                    QUARANTINE_BOUND_ROOT_PAYLOAD_BYTES
                } else {
                    DIRECTORY_BOUND_ROOT_PAYLOAD_BYTES
                } =>
        {
            if payload[402..408] != [0; 6] {
                return Err(RootManifestDenial::ReservedFieldNonZero);
            }
            let latest = match payload[401] {
                0 if payload[336..400] == [0; 64] => None,
                1 => Some(
                    super::super::derived_family_directory::decode_publication(&payload[336..400])
                        .ok_or(RootManifestDenial::InvalidRecordIdentity)?,
                ),
                _ => return Err(RootManifestDenial::MalformedPrefix),
            };
            let directory = match payload[400] {
                0 if payload[408..496] == [0; 88] => None,
                1 => Some(
                    super::super::derived_family_directory::decode_root_binding(&payload[408..496])
                        .ok_or(RootManifestDenial::InvalidRecordIdentity)?,
                ),
                _ => return Err(RootManifestDenial::MalformedPrefix),
            };
            let quarantine = if schema >= 6 {
                if payload[497..504] != [0; 7] {
                    return Err(RootManifestDenial::ReservedFieldNonZero);
                }
                match payload[496] {
                    1 => Some(
                        decode_identity(&payload[504..528])
                            .ok_or(RootManifestDenial::InvalidRecordIdentity)?,
                    ),
                    0 if schema >= 9 && payload[504..528] == [0; 24] => None,
                    _ => return Err(RootManifestDenial::MalformedPrefix),
                }
            } else {
                None
            };
            if schema != 9
                && schema != 10
                && latest.is_none()
                && directory.is_none()
                && quarantine.is_none()
            {
                return Err(RootManifestDenial::MalformedPrefix);
            }
            Ok((latest, directory, quarantine))
        }
        _ => Err(RootManifestDenial::MalformedPrefix),
    }
}
