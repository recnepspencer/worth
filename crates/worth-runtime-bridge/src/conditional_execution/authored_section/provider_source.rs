//! Complete provider-role sources in each version-2 definition row.
use super::{
    fields::{Reader, Writer},
    BridgeAuthoredSectionDenial as D,
};
use crate::conditional_execution::{
    BridgeAuthoredProviderDescription, BridgeAuthoredProviderDescriptions,
    BridgeConditionalProviderRole,
};
use worth_execution::ExecutionRequest;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};

pub(super) fn write(
    out: &mut Writer<'_>,
    descriptions: &BridgeAuthoredProviderDescriptions,
) -> Result<(), D> {
    for role in BridgeConditionalProviderRole::ALL {
        let source = descriptions.get(role);
        out.byte(u8::from(source.is_some()))?;
        if let Some(source) = source {
            out.text(source.protocol().as_str())?;
            out.number(source.version().get() as u64)?;
            out.blob(source.payload())?;
        }
    }
    Ok(())
}

fn fields<'a>(
    input: &mut Reader<'a>,
) -> Result<Option<(&'a str, BoundaryProtocolVersion, &'a [u8])>, D> {
    match input.byte()? {
        0 => Ok(None),
        1 => {
            let protocol = input.text()?;
            BoundaryProtocolIdentity::validate(protocol).map_err(|_| D::Malformed)?;
            let version = u32::try_from(input.number()?).map_err(|_| D::Malformed)?;
            let version = BoundaryProtocolVersion::try_new(version).map_err(|_| D::Malformed)?;
            Ok(Some((protocol, version, input.blob()?)))
        }
        _ => Err(D::Malformed),
    }
}

pub(super) fn inspect(bytes: &[u8]) -> Result<(), D> {
    let mut input = Reader::new(bytes);
    for _role in BridgeConditionalProviderRole::ALL {
        fields(&mut input)?;
    }
    input.finish()
}

pub(super) struct DecodedProviders {
    pub(super) descriptions: BridgeAuthoredProviderDescriptions,
    _backing: Option<worth_execution::ExecutionMemoryReservation>,
}
pub(super) fn decode(
    bytes: &[u8],
    execution: &ExecutionRequest<'_, '_>,
) -> Result<DecodedProviders, D> {
    // The borrowed inspector allocates nothing. Reserve the actual parsed
    // protocol and payload owners before constructing any decoded field.
    let mut input = Reader::new(bytes);
    let mut storage = 0u64;
    for _role in BridgeConditionalProviderRole::ALL {
        if let Some((protocol, _, payload)) = fields(&mut input)? {
            storage = storage
                .checked_add(
                    super::super::retention::arc_charge::<BoundaryProtocolIdentity>()
                        .map_err(|_| D::SizeLimit)?
                        .checked_add(protocol.len() as u64)
                        .and_then(|n| {
                            n.checked_add(
                                super::super::retention::arc_slice_charge::<u8>(payload.len())
                                    .ok()?,
                            )
                        })
                        .ok_or(D::SizeLimit)?,
                )
                .ok_or(D::SizeLimit)?;
        }
    }
    input.finish()?;
    let backing = execution
        .reserve_retained_memory(storage)
        .map_err(D::Request)?;
    let mut input = Reader::new(bytes);
    let mut descriptions = BridgeAuthoredProviderDescriptions::default();
    for role in BridgeConditionalProviderRole::ALL {
        let source = fields(&mut input)?.map(|(protocol, version, payload)| {
            let protocol =
                BoundaryProtocolIdentity::parse(protocol.to_owned()).expect("inspected family");
            BridgeAuthoredProviderDescription::new(protocol, version, payload)
        });
        match role {
            BridgeConditionalProviderRole::Condition => descriptions.condition = source,
            BridgeConditionalProviderRole::DependencyComparator => {
                descriptions.dependency_comparator = source
            }
            BridgeConditionalProviderRole::OutputComparator => {
                descriptions.output_comparator = source
            }
            BridgeConditionalProviderRole::ArtifactReuseComparator => {
                descriptions.reuse_comparator = source
            }
            BridgeConditionalProviderRole::Trigger => descriptions.trigger = source,
            BridgeConditionalProviderRole::Wake => descriptions.wake = source,
            BridgeConditionalProviderRole::Compute => descriptions.compute = source,
        }
    }
    input.finish()?;
    Ok(DecodedProviders {
        descriptions,
        _backing: backing,
    })
}
