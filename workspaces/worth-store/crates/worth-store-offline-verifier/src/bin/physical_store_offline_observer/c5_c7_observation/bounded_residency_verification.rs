use std::path::Path;

#[path = "bounded_residency_verification/configuration.rs"]
mod configuration;
#[path = "bounded_residency_verification/expectation.rs"]
mod expectation;

pub(super) fn run(root: &Path, configuration_path: &Path) {
    let result = verify(root, configuration_path);
    match result {
        Ok((observation, expectation)) => {
            super::hostile_physical_truth::emit(&observation);
            println!(
                "BOUNDED_RESIDENCY_VERIFICATION accepted {} {} {} {}",
                expectation.records(),
                expectation.payload_bytes(),
                super::hex(&expectation.digest()),
                expectation.seed(),
            );
        }
        Err(denial) => {
            eprintln!("BOUNDED_RESIDENCY_VERIFICATION denied {denial}");
            std::process::exit(1);
        }
    }
}

fn verify(
    root: &Path,
    configuration_path: &Path,
) -> Result<
    (
        worth_store_offline_verifier::OfflineHostilePhysicalTruthObservation,
        expectation::ExpectedDurableTruth,
    ),
    String,
> {
    let configuration = configuration::BoundedResidencyConfiguration::read(configuration_path)?;
    let expectation = expectation::derive(configuration)?;
    let observation = super::hostile_physical_truth::observe(root)?;
    let current = observation
        .current()
        .map_err(|denial| format!("durable manifest denied: {denial:?}"))?;
    if current.records() != expectation.records()
        || current.payload_bytes() != expectation.payload_bytes()
    {
        return Err(format!(
            "seed-derived truth disagrees with durable files: expected={expectation:?}, \
             current={current:?}"
        ));
    }
    verify_record_payloads(configuration, observation.record_payloads())?;
    Ok((observation, expectation))
}

fn verify_record_payloads(
    configuration: configuration::BoundedResidencyConfiguration,
    records: &[worth_store_offline_verifier::OfflineRecordPayloadObservation],
) -> Result<(), String> {
    if records.len() != configuration.record_count() {
        return Err(format!(
            "durable payload walk observed {} records; expected {}",
            records.len(),
            configuration.record_count()
        ));
    }
    let mut seen = vec![false; configuration.record_count()];
    for record in records {
        let prefix: [u8; 8] = record
            .prefix()
            .try_into()
            .map_err(|_| "durable record omitted its eight-byte workload ordinal".to_owned())?;
        let ordinal = usize::try_from(u64::from_le_bytes(prefix))
            .map_err(|_| "durable workload ordinal exceeds usize".to_owned())?;
        let expected_bytes = configuration
            .record_bytes(ordinal)
            .ok_or_else(|| format!("durable record declares unknown ordinal {ordinal}"))?;
        if seen[ordinal]
            || record.payload_bytes() != expected_bytes as u64
            || record.digest() != expectation::record_digest(configuration, ordinal)?
        {
            return Err(format!(
                "durable record ordinal {ordinal} is duplicated or disagrees with seed truth"
            ));
        }
        seen[ordinal] = true;
    }
    if seen.iter().any(|seen| !seen) {
        return Err("durable payload walk omitted a workload ordinal".to_owned());
    }
    Ok(())
}
