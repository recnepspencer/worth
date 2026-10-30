pub(crate) use {
    std::io::Write, std::num::NonZeroU32, std::path::Path, std::path::PathBuf, std::process::Child,
    std::process::Stdio, std::time::Duration, std::time::Instant,
};

pub(crate) const GATE_ENV: &str = "WORTH_STORE_C5_PHASE16_A_GATE";

pub(crate) struct FreshReopener {
    pub(crate) child: Child,
    pub(crate) gate: PathBuf,
    pub(crate) process: NonZeroU32,
}

pub(crate) struct FreshReopenObservation {
    pub(crate) process: NonZeroU32,
    pub(crate) root_generation: u64,
    pub(crate) records: Vec<Vec<u8>>,
}

pub(crate) fn spawn(root: &Path) -> FreshReopener {
    let gate = root
        .parent()
        .expect("Phase 16 Store root has a parent")
        .join("phase-16-a-reopen.gate");
    let _ = std::fs::remove_file(&gate);
    let mut command = worth_store_testkit_store::physical_record_journeys::child_process::invocation::child_command("phase16_maelstrom_reopener", root);
    let child = command
        .env(GATE_ENV, &gate)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .unwrap();
    let process = std::num::NonZeroU32::new(child.id()).unwrap();
    FreshReopener {
        child,
        gate,
        process,
    }
}

impl FreshReopener {
    pub(crate) fn open_after_close(self) -> FreshReopenObservation {
        std::fs::write(&self.gate, b"open").unwrap();
        let output = self.child.wait_with_output().unwrap();
        let _ = std::fs::remove_file(&self.gate);
        assert!(
            output.status.success(),
            "fresh Phase 16 reopener failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        decode_observation(self.process, &String::from_utf8(output.stdout).unwrap())
    }
}

pub(crate) fn reopener(root: &Path) {
    let gate = std::path::PathBuf::from(std::env::var_os(GATE_ENV).unwrap());
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    while !gate.is_file() && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    assert!(
        gate.is_file(),
        "Phase 16 reopener never received the post-close gate"
    );
    let serving = worth_store_testkit_store::physical_record_journeys::serving_from_open(root);
    let generation = serving
        .observer()
        .acquisition_snapshot()
        .unwrap()
        .root_generation();
    let records = worth_store_testkit_store::physical_record_journeys::scan_journeys::collect_scan(
        &serving, 8, 64_000,
    );
    println!("C5_PHASE16_A_REOPEN {} {generation}", std::process::id());
    for (_, bytes) in records {
        println!(
            "C5_PHASE16_A_RECORD {}",
            worth_store_testkit_store::child_process::locator_codec::hex(&bytes)
        );
    }
    std::io::stdout().flush().unwrap();
    serving.close();
}

pub(crate) fn decode_observation(process: NonZeroU32, output: &str) -> FreshReopenObservation {
    let (reported_process, root_generation) = output
        .lines()
        .find_map(|line| line.strip_prefix("C5_PHASE16_A_REOPEN "))
        .and_then(|line| line.split_once(' '))
        .map(|(process, generation)| {
            (
                std::num::NonZeroU32::new(process.parse().unwrap()).unwrap(),
                generation.parse().unwrap(),
            )
        })
        .expect("fresh Phase 16 reopener must report its process and root generation");
    assert_eq!(reported_process, process);
    let records = output
        .lines()
        .filter_map(|line| line.strip_prefix("C5_PHASE16_A_RECORD "))
        .map(decode_hex)
        .collect();
    FreshReopenObservation {
        process,
        root_generation,
        records,
    }
}

pub(crate) fn decode_hex(encoded: &str) -> Vec<u8> {
    assert_eq!(encoded.len() % 2, 0);
    encoded
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let pair = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(pair, 16).unwrap()
        })
        .collect()
}
