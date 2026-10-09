use std::io;

use super::{MediaOwnerIdentity, MutationOwnershipAttempt, MutationOwnershipDenial};

pub(super) struct MutationLeasePublicationFailure {
    pub(super) denial: MutationOwnershipDenial,
    pub(super) effect_fate: super::owner_admission_effect::MediaOwnerAdmissionEffectFate,
}

impl MutationLeasePublicationFailure {
    const fn before_effect(denial: MutationOwnershipDenial) -> Self {
        Self {
            denial,
            effect_fate:
                super::owner_admission_effect::MediaOwnerAdmissionEffectFate::DeniedBeforeEffect,
        }
    }

    const fn effect_possible(denial: MutationOwnershipDenial) -> Self {
        Self {
            denial,
            effect_fate:
                super::owner_admission_effect::MediaOwnerAdmissionEffectFate::EffectPossible,
        }
    }
}

pub(super) fn publish_owner_observation(
    lock: &std::fs::File,
    owner: MediaOwnerIdentity,
    ownership_attempt: MutationOwnershipAttempt,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<(), MutationLeasePublicationFailure> {
    use std::fmt::Write as _;

    let mut observation = String::with_capacity(128);
    boundary
        .counters()
        .explicit_heap_allocation(observation.capacity());
    writeln!(&mut observation, "version=1").expect("writing to String cannot fail");
    writeln!(&mut observation, "process={:010}", std::process::id())
        .expect("writing to String cannot fail");
    observation.push_str("runtime=");
    append_hex(&mut observation, &owner.bytes());
    observation.push('\n');
    observation.push_str("attempt=");
    append_hex(&mut observation, &ownership_attempt.bytes());
    observation.push('\n');
    publish_observation_bytes(lock, &observation, boundary)
}

fn publish_observation_bytes(
    lock: &std::fs::File,
    observation: &str,
    boundary: &super::fault_interposition::MediaFaultInterposer,
) -> Result<(), MutationLeasePublicationFailure> {
    use std::io::{Seek, Write};

    let attempted_bytes = observation.len() as u64;
    let attempt = boundary.begin(
        super::MediaOperationRole::PublishMutationLeaseObservation,
        attempted_bytes,
    );
    if let Some(error) = attempt.fail_before_error() {
        attempt.denied();
        return Err(MutationLeasePublicationFailure::before_effect(lock_error(
            error,
        )));
    }
    if let Err(error) = lock.set_len(0) {
        attempt.indeterminate(0);
        return Err(effect_possible(error));
    }
    let mut writer = lock;
    if let Err(error) = writer.seek(std::io::SeekFrom::Start(0)) {
        attempt.indeterminate(0);
        return Err(effect_possible(error));
    }
    let transfer_limit = attempt.transfer_limit(attempted_bytes) as usize;
    let mut completed = 0;
    while completed < transfer_limit {
        match writer.write(&observation.as_bytes()[completed..transfer_limit]) {
            Ok(0) => {
                attempt.indeterminate(completed as u64);
                return Err(effect_possible(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "lease observation write made no progress",
                )));
            }
            Ok(bytes) => completed += bytes,
            Err(error) => {
                attempt.indeterminate(completed as u64);
                return Err(effect_possible(error));
            }
        }
    }
    if completed != observation.len() {
        attempt.partial(completed as u64);
        return Err(effect_possible(io::Error::new(
            io::ErrorKind::WriteZero,
            "certification stopped lease observation after a known prefix",
        )));
    }
    if let Some(error) = attempt.barrier_error() {
        attempt.indeterminate(attempted_bytes);
        return Err(effect_possible(error));
    }
    if let Err(error) = super::os_synchronization::synchronize_data(lock) {
        attempt.indeterminate(attempted_bytes);
        return Err(effect_possible(error));
    }
    if attempt.effect_observation_is_indeterminate() {
        attempt.indeterminate(attempted_bytes);
        return Err(effect_possible(io::Error::other(
            "certification interrupted lease-publication observation",
        )));
    }
    attempt.completed(attempted_bytes);
    Ok(())
}

fn effect_possible(error: io::Error) -> MutationLeasePublicationFailure {
    MutationLeasePublicationFailure::effect_possible(lock_error(error))
}

fn lock_error(error: io::Error) -> MutationOwnershipDenial {
    super::mutation_ownership::lock_error(error)
}

fn append_hex(encoded: &mut String, bytes: &[u8]) {
    use std::fmt::Write;

    for byte in bytes {
        write!(encoded, "{byte:02x}").expect("writing to String cannot fail");
    }
}
