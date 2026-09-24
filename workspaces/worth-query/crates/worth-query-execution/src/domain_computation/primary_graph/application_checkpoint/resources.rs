use super::super::application_contribution::WorthQueryProducerDemandResources;

pub(super) fn encode_profile(
    output: &mut Vec<u8>,
    resources: Option<WorthQueryProducerDemandResources>,
) {
    output.push(u8::from(resources.is_some()));
    output.extend_from_slice(
        &u64::try_from(resources.map_or(0, |r| r.work()))
            .expect("producer work fits the checkpoint format")
            .to_be_bytes(),
    );
    output.extend_from_slice(
        &u64::try_from(resources.map_or(0, |r| r.retained_bytes()))
            .expect("producer retained bytes fit the checkpoint format")
            .to_be_bytes(),
    );
}

pub(super) fn decode_profile(
    cursor: &mut super::CheckpointCursor<'_>,
    version: u16,
) -> Result<Option<WorthQueryProducerDemandResources>, String> {
    if version == 3 {
        return Ok(None);
    }
    let posture = cursor.next_byte()?;
    let work = cursor.next_u64()?;
    let bytes = cursor.next_u64()?;
    match posture {
        0 if work == 0 && bytes == 0 => Ok(None),
        1 => Ok(Some(WorthQueryProducerDemandResources::new(
            usize::try_from(work)
                .map_err(|_| "checkpoint producer work exceeds this host".to_owned())?,
            usize::try_from(bytes)
                .map_err(|_| "checkpoint producer bytes exceed this host".to_owned())?,
        ))),
        _ => Err("checkpoint producer resource profile is invalid".to_owned()),
    }
}
