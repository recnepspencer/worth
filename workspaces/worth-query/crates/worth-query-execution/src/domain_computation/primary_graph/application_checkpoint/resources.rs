use super::super::application_contribution::WorthQueryProducerDemandResources;

pub(super) fn encode_profile(resources: Option<WorthQueryProducerDemandResources>) -> [u8; 17] {
    let mut output = [0; 17];
    output[0] = match resources {
        None => 0,
        Some(r) if r.work().is_some() => 1,
        Some(_) => 2,
    };
    output[1..9].copy_from_slice(
        &u64::try_from(resources.and_then(|r| r.work()).unwrap_or(0))
            .expect("producer work fits the checkpoint format")
            .to_be_bytes(),
    );
    output[9..17].copy_from_slice(
        &u64::try_from(resources.map_or(0, |r| r.retained_bytes()))
            .expect("producer retained bytes fit the checkpoint format")
            .to_be_bytes(),
    );
    output
}

pub(super) fn decode_profile(
    cursor: &mut super::CheckpointCursor<'_>,
) -> Result<Option<WorthQueryProducerDemandResources>, String> {
    let posture = cursor.next_byte()?;
    let work = cursor.next_u64()?;
    let bytes = cursor.next_u64()?;
    match posture {
        0 if work == 0 && bytes == 0 => Ok(None),
        1 => Ok(Some(WorthQueryProducerDemandResources::new(
            Some(
                usize::try_from(work)
                    .map_err(|_| "checkpoint producer work exceeds this host".to_owned())?,
            ),
            usize::try_from(bytes)
                .map_err(|_| "checkpoint producer bytes exceed this host".to_owned())?,
        ))),
        2 if work == 0 => Ok(Some(WorthQueryProducerDemandResources::new(
            None,
            usize::try_from(bytes)
                .map_err(|_| "checkpoint producer bytes exceed this host".to_owned())?,
        ))),
        _ => Err("checkpoint producer resource profile is invalid".to_owned()),
    }
}
