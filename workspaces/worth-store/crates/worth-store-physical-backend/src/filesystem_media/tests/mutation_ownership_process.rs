use super::super::*;

pub(super) const PROCESS_ROLE_ENV: &str = "WORTH_STORE_C4_MUTATION_OWNER_ROLE";
pub(super) const PROCESS_ROOT_ENV: &str = "WORTH_STORE_C4_MUTATION_OWNER_ROOT";

pub(super) struct ProcessContender {
    pub(super) child: std::process::Child,
    pub(super) stdin: Option<std::process::ChildStdin>,
    result: std::sync::mpsc::Receiver<String>,
}

impl ProcessContender {
    pub(super) fn read_result(&mut self) -> String {
        self.result
            .recv_timeout(std::time::Duration::from_secs(20))
            .expect("child did not report ownership outcome")
    }
}

pub(super) fn spawn_contender(root: &std::path::Path, index: usize) -> ProcessContender {
    use std::io::BufRead;
    use std::process::Stdio;

    let mut child = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args([
            "--exact",
            "filesystem_media::tests::mutation_ownership::mutation_ownership_process_role",
            "--nocapture",
            "--test-threads=1",
        ])
        .env(PROCESS_ROLE_ENV, index.to_string())
        .env(PROCESS_ROOT_ENV, root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("spawn contender");
    let stdin = child.stdin.take().expect("child stdin");
    let stdout = child.stdout.take().expect("child stdout");
    let (sender, result) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let mut sender = Some(sender);
        let mut transcript = Vec::new();
        for line in std::io::BufReader::new(stdout)
            .lines()
            .map_while(Result::ok)
        {
            let result = line
                .find("OWNED ")
                .map(|offset| line[offset..].to_owned())
                .or_else(|| line.contains("CONTENDED").then(|| "CONTENDED".to_owned()));
            if let Some(result) = result
                && let Some(channel) = sender.take()
            {
                channel.send(result).expect("send child result");
            }
            transcript.push(line);
        }
        if let Some(channel) = sender {
            let _ = channel.send(format!("NO_RESULT {transcript:?}"));
        }
    });
    ProcessContender {
        child,
        stdin: Some(stdin),
        result,
    }
}

pub(super) fn ensure_stable_identity(owner: &FilesystemMediaOwner) -> [u8; 16] {
    use worth_store_physical_format::store_namespace::{
        StoreNamespaceIdentityRecord, STORE_NAMESPACE_IDENTITY_RECORD_LENGTH,
    };

    let identity_path = owner.identity_record_path();
    match owner.open_existing(&identity_path).into_result() {
        NamespaceFileOpenResult::Opened { handle, .. } => {
            let mut bytes = [0_u8; STORE_NAMESPACE_IDENTITY_RECORD_LENGTH];
            read_exact_positioned(&handle, &mut bytes);
            StoreNamespaceIdentityRecord::decode(&bytes)
                .expect("published identity must decode")
                .proposed_identity()
                .bytes()
        }
        NamespaceFileOpenResult::Failed(failure)
            if failure.context().io_kind() == Some(std::io::ErrorKind::NotFound) =>
        {
            publish_new_identity(owner)
        }
        NamespaceFileOpenResult::Failed(failure) => {
            panic!("identity discovery failed: {failure:?}")
        }
    }
}

fn publish_new_identity(owner: &FilesystemMediaOwner) -> [u8; 16] {
    use worth_store_physical_format::store_namespace::{
        NamespaceInitializationAttempt, ProposedStoreIdentity, StagedNamespaceName,
        StoreNamespaceIdentityRecord, StoreNamespaceVersion,
    };

    let identity = random_nonzero_bytes();
    let proposed = ProposedStoreIdentity::from_nonzero_bytes(identity).expect("nonzero identity");
    let record = StoreNamespaceIdentityRecord::new(StoreNamespaceVersion::CURRENT, proposed);
    let attempt = NamespaceInitializationAttempt::from_nonzero_bytes(random_nonzero_bytes())
        .expect("nonzero attempt");
    let staged_path = owner.staged_identity_path(&StagedNamespaceName::for_identity(attempt));
    let staged = match StagedNamespaceFile::create(owner, staged_path) {
        StagedNamespaceFileOutcome::Created(staged) => staged,
        other => panic!("identity staging failed: {other:?}"),
    };
    let completed = match staged.write_all(&record.encode()) {
        StagedNamespaceWriteOutcome::Completed(completed) => completed,
        other => panic!("identity write failed: {other:?}"),
    };
    let synchronized = match completed.synchronize() {
        StagedNamespaceSynchronizationOutcome::Synchronized(value) => value,
        other => panic!("identity file sync failed: {other:?}"),
    };
    let replaced = match synchronized.replace(owner.identity_publication_target()) {
        AtomicReplacementOutcome::Replaced(value) => value,
        other => panic!("identity rename failed: {other:?}"),
    };
    assert!(matches!(
        replaced.synchronize_publication(),
        DurableNamespacePublicationOutcome::Published(_)
    ));
    identity
}

fn random_nonzero_bytes() -> [u8; 16] {
    loop {
        let mut bytes = [0_u8; 16];
        getrandom::fill(&mut bytes).expect("identity entropy");
        if bytes != [0_u8; 16] {
            return bytes;
        }
    }
}

fn read_exact_positioned<Access>(handle: &NamespaceFileHandle<'_, Access>, target: &mut [u8]) {
    let mut completed = 0_usize;
    while completed < target.len() {
        match handle
            .positioned_read(PositionedReadRequest::new(
                completed as u64,
                &mut target[completed..],
            ))
            .result()
        {
            PositionedReadResult::Transferred(transfer) => completed += transfer.bytes() as usize,
            PositionedReadResult::Failed(failure) => match failure.kind() {
                MediaOperationFailureKind::PartialTransfer(transfer) => {
                    completed += transfer.completed_bytes() as usize;
                }
                other => panic!("identity read failed: {other:?}"),
            },
            PositionedReadResult::EndOfFile { .. } => panic!("identity record truncated"),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub(super) struct OwnedProcessReport {
    pub(super) process: u32,
    pub(super) owner: String,
    pub(super) attempt: String,
    pub(super) stable: String,
}

pub(super) fn parse_owned(result: &str) -> OwnedProcessReport {
    let mut fields = result.split_whitespace();
    assert_eq!(fields.next(), Some("OWNED"), "unexpected result: {result}");
    let process = fields
        .next()
        .expect("process identity")
        .parse()
        .expect("numeric process identity");
    let owner = fields.next().expect("owner identity").to_owned();
    let attempt = fields.next().expect("attempt identity").to_owned();
    let stable = fields.next().expect("stable identity").to_owned();
    assert!(fields.next().is_none());
    OwnedProcessReport {
        process,
        owner,
        attempt,
        stable,
    }
}

pub(super) fn encode_hex(bytes: &[u8]) -> String {
    use std::fmt::Write;

    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("write hex");
    }
    encoded
}
