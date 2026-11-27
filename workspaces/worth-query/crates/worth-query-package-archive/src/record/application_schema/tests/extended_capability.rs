//! Member tag 59 carries standing tenure, extent status, UTC and local-date
//! validity and the self-grant rule; every other contract keeps tag 7 or 33,
//! and each contract has exactly one accepted encoding.
use super::super::member;
use super::fixture;
use crate::{
    binary_input::BinaryInput, binary_output::BinaryOutput, limits::WorthQueryPackageArchiveLimits,
    record::decode_budget::RecordDecodeAttempt,
};
use worth_query_declaration::archive_decode_authority::ARCHIVE_DECODE;
use worth_query_declaration::facade::application_capability::*;
use worth_query_declaration::facade::application_schema::ApplicationSchemaMember as Member;

const VALIDITY_PROBE: &str = "ValidityProbe";

fn capability() -> ErasedApplicationCapabilityContract {
    fixture::complete_untrusted_schema_record()
        .members()
        .iter()
        .find_map(|member| match member {
            Member::ApplicationCapability { contract } => Some(contract.clone()),
            _ => None,
        })
        .unwrap()
}

fn probe_field(template: &ApplicationCapabilityFieldBinding) -> ApplicationCapabilityFieldBinding {
    let mut parts = template.parts();
    parts.entity = VALIDITY_PROBE.to_owned();
    ApplicationCapabilityFieldBinding::from_untrusted_parts(ARCHIVE_DECODE, parts)
}

/// The fixture contract with probe validity fields on `timeline`, standing or
/// workflow tenure, and the given extent status and self-grant rule.
fn contract(
    timeline: ApplicationCapabilityValidityTimeline,
    standing: bool,
    extent: bool,
) -> ErasedApplicationCapabilityContract {
    let mut parts = capability().parts();
    let current = parts.constraints.currentness().clone();
    let probe = probe_field(current.validity().not_before());
    let validity = ApplicationCapabilityValidityDefinition::new(timeline, probe.clone(), probe);
    let mut currentness = match (standing, current.workflow()) {
        (false, Some(workflow)) => ApplicationCapabilityCurrentnessDefinition::new(
            current.active_status().clone(),
            workflow.clone(),
            validity,
        ),
        _ => ApplicationCapabilityCurrentnessDefinition::standing(
            current.active_status().clone(),
            validity,
        ),
    };
    if extent {
        currentness = currentness.require_active(current.active_status().clone());
    }
    let mut constraints = parts.constraints.parts();
    constraints.currentness = currentness;
    parts.constraints = ApplicationCapabilityConstraintDefinition::from_untrusted_parts(
        ARCHIVE_DECODE,
        constraints,
    );
    if standing {
        let mut delegation = parts.delegation.parts();
        delegation.lineage = None;
        delegation.activation = None;
        delegation.self_grant = ApplicationCapabilitySelfGrantRule::Forbidden;
        parts.delegation = ApplicationCapabilityDelegationDefinition::from_untrusted_parts(
            ARCHIVE_DECODE,
            delegation,
        );
    }
    ErasedApplicationCapabilityContract::from_untrusted_parts(ARCHIVE_DECODE, parts)
}

fn encode(member: &Member) -> Vec<u8> {
    let mut output = BinaryOutput::with_capacity(4096);
    member::write(&mut output, member).unwrap();
    output.into_bytes()
}

fn decode(bytes: &[u8]) -> Result<Member, crate::denial::WorthQueryPackageArchiveDenial> {
    let mut input = BinaryInput::new(bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    let member = member::decode(&mut input, &mut budget)?;
    assert!(input.is_finished());
    Ok(member)
}

/// Rewrites the validity timeline code written just before the probe field.
fn retimed(bytes: &[u8], from: u16, to: u16) -> Vec<u8> {
    let mut needle = from.to_be_bytes().to_vec();
    needle.extend((VALIDITY_PROBE.len() as u32).to_be_bytes());
    needle.extend(VALIDITY_PROBE.as_bytes());
    let at = bytes
        .windows(needle.len())
        .position(|window| window == needle.as_slice())
        .unwrap();
    let mut bytes = bytes.to_vec();
    bytes[at..at + 2].copy_from_slice(&to.to_be_bytes());
    bytes
}

fn unsupported() -> crate::denial::WorthQueryPackageArchiveDenialKind {
    crate::denial::WorthQueryPackageArchiveDenialKind::UnsupportedRecordVariant
}

#[test]
fn extended_contracts_round_trip_under_tag_59() {
    use ApplicationCapabilityValidityTimeline as Timeline;
    for (timeline, standing, extent) in [
        (Timeline::UnixEpochSeconds, true, true),
        (Timeline::UtcTimestamp, true, false),
        (Timeline::LocalDate, true, true),
        (Timeline::LocalDate, false, false),
        (Timeline::UnixEpochMilliseconds, false, true),
    ] {
        let member = Member::ApplicationCapability {
            contract: contract(timeline, standing, extent),
        };
        let bytes = encode(&member);
        assert_eq!(&bytes[..2], &59_u16.to_be_bytes());
        assert_eq!(decode(&bytes).unwrap(), member);
    }
}

#[test]
fn workflow_contracts_on_epoch_timelines_keep_their_legacy_tag() {
    let legacy = Member::ApplicationCapability {
        contract: contract(
            ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
            false,
            false,
        ),
    };
    let bytes = encode(&legacy);
    assert_eq!(&bytes[..2], &7_u16.to_be_bytes());
    assert_eq!(decode(&bytes).unwrap(), legacy);
}

#[test]
fn a_legacy_expressible_contract_under_tag_59_is_refused() {
    let member = Member::ApplicationCapability {
        contract: contract(
            ApplicationCapabilityValidityTimeline::UtcTimestamp,
            false,
            false,
        ),
    };
    let bytes = encode(&member);
    assert_eq!(decode(&bytes).unwrap(), member);
    // The same body on the seconds timeline is a tag 7 contract: refused here.
    let downgraded = retimed(&bytes, 3, 1);
    assert_eq!(decode(&downgraded).unwrap_err().kind(), unsupported());
}

#[test]
fn legacy_tags_refuse_utc_and_local_date_timelines() {
    let legacy = Member::ApplicationCapability {
        contract: contract(
            ApplicationCapabilityValidityTimeline::UnixEpochSeconds,
            false,
            false,
        ),
    };
    let bytes = encode(&legacy);
    for code in [3, 4] {
        assert_eq!(
            decode(&retimed(&bytes, 1, code)).unwrap_err().kind(),
            unsupported()
        );
    }
}
