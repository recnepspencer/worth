use worth_query_installation::facade::WorthQueryPortablePackageRecordFamily as Family;

/// Current deterministic record-frame protocol version.
pub const WORTH_QUERY_PACKAGE_ARCHIVE_RECORD_PROTOCOL_VERSION: u16 = 2;
pub(crate) const RECORD_FRAME_HEADER_BYTES: u64 = 2 + 2 + 4 + 4;

pub(super) const fn family_tag(family: Family) -> u16 {
    match family {
        Family::DomainIdentity => 1,
        Family::CapabilityRequirement => 2,
        Family::ConfigurationRequirement => 3,
        Family::OperatingRequirement => 4,
        Family::Definition => 5,
        Family::DomainOperation => 6,
        Family::ArtifactContract => 7,
        Family::ApplicationSchema => 8,
        Family::ConditionalApplicationOperation => 9,
        Family::ContributionPolicy => 10,
        Family::NativeAspectContract => 11,
        Family::ApplicationOperationContract => 12,
    }
}

pub(super) const fn family_from_tag(tag: u16) -> Option<Family> {
    match tag {
        1 => Some(Family::DomainIdentity),
        2 => Some(Family::CapabilityRequirement),
        3 => Some(Family::ConfigurationRequirement),
        4 => Some(Family::OperatingRequirement),
        5 => Some(Family::Definition),
        6 => Some(Family::DomainOperation),
        7 => Some(Family::ArtifactContract),
        8 => Some(Family::ApplicationSchema),
        9 => Some(Family::ConditionalApplicationOperation),
        10 => Some(Family::ContributionPolicy),
        11 => Some(Family::NativeAspectContract),
        12 => Some(Family::ApplicationOperationContract),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_one_family_tags_are_frozen_for_the_complete_vocabulary() {
        let expected = [1_u16, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];
        for (family, tag) in Family::ALL.into_iter().zip(expected) {
            assert_eq!(family_tag(family), tag);
            assert_eq!(family_from_tag(tag), Some(family));
        }
    }
}
