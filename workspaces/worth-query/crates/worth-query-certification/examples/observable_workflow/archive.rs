//! A local process is the external archive owner. Its owned stdout authenticates
//! completion to this host; this example makes no network signature claim.
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::Mutex;
use worth_foundational::facade::{
    derive_canonical_digest, prepare_canonical_basis_bundle, prepare_canonical_basis_sequence,
    BoundaryProtocolIdentity, BoundaryProtocolVersion, CanonicalBasisDomain, CanonicalBasisEntry,
    CanonicalBasisEntryKind, CanonicalBasisLocus, CanonicalBasisValue, CanonicalDigestAlgorithmId,
    CanonicalDigestFrontDoor, CanonicalizationRuleVersion,
};
use worth_proof::TransitionOutcome;
use worth_query_host::facade::primary_graph::{
    WorthQueryExternalDispatchRequest, WorthQueryExternalEffectTransport,
    WorthQueryExternalTransportOutcome, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial,
};

pub const PROTOCOL: &str = "neutral.archive.v1";
pub const SOURCE: &str = "neutral.archive.child-process";
pub const FAMILY: &str = "neutral-archive";
pub const AUDIENCE: &str = "neutral.observable-workflow";
const MAXIMUM_PAYLOAD: usize = 512;
const MAXIMUM_ENVELOPE: usize = 40 + MAXIMUM_PAYLOAD;

pub struct ArchiveTransport {
    directory: PathBuf,
    protocol: BoundaryProtocolIdentity,
    authenticated: Mutex<Option<Vec<u8>>>,
}

impl ArchiveTransport {
    pub fn new(directory: PathBuf) -> Self {
        Self {
            directory,
            protocol: BoundaryProtocolIdentity::new(PROTOCOL),
            authenticated: Mutex::new(None),
        }
    }

    pub fn envelope(&self) -> Result<Vec<u8>, String> {
        self.authenticated
            .lock()
            .map_err(|_| "archive lock poisoned")?
            .clone()
            .ok_or_else(|| "archive has not completed".to_owned())
    }

    fn dispatch_archive(
        &self,
        request: WorthQueryExternalDispatchRequest<'_>,
    ) -> Result<(), String> {
        if request.protocol_identity() != &self.protocol
            || request.protocol_version() != BoundaryProtocolVersion::new(1)
            || request.correlation_family().as_str() != FAMILY
            || request.payload().len() > MAXIMUM_PAYLOAD
        {
            return Err("archive dispatch contract mismatch".to_owned());
        }
        let mut child = Command::new(std::env::current_exe().map_err(|e| e.to_string())?)
            .arg("--archive-worker")
            .arg(&self.directory)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| e.to_string())?;
        let mut input = child.stdin.take().ok_or("missing archive stdin")?;
        input
            .write_all(request.correlation_token())
            .map_err(|e| e.to_string())?;
        input
            .write_all(request.payload())
            .map_err(|e| e.to_string())?;
        drop(input);
        let output = child.stdout.take().ok_or("missing archive stdout")?;
        let response = read_bounded(output)?;
        if !child.wait().map_err(|e| e.to_string())?.success()
            || response.get(8..40) != Some(request.correlation_token().as_slice())
            || response.get(40..) != Some(request.payload())
        {
            return Err("archive process did not authenticate completion".to_owned());
        }
        let artifact =
            std::fs::File::open(artifact_path(&self.directory, request.correlation_token()))
                .map_err(|e| e.to_string())?;
        if read_bounded(artifact)? != request.payload() {
            return Err("independent archive readback differs".to_owned());
        }
        *self
            .authenticated
            .lock()
            .map_err(|_| "archive lock poisoned")? = Some(response);
        Ok(())
    }
}

impl WorthQueryExternalEffectTransport for ArchiveTransport {
    fn dispatch(
        &self,
        request: WorthQueryExternalDispatchRequest<'_>,
    ) -> WorthQueryExternalTransportOutcome {
        match self.dispatch_archive(request) {
            Ok(()) => WorthQueryExternalTransportOutcome::Acknowledged,
            // A failed child response cannot prove that no external write occurred.
            Err(error) => {
                eprintln!("archive response unavailable: {error}");
                WorthQueryExternalTransportOutcome::LostResponse
            }
        }
    }
}

impl WorthQueryInboundOccurrenceVerifier for ArchiveTransport {
    fn audience(&self) -> &str {
        AUDIENCE
    }
    fn source_identity(&self) -> &str {
        SOURCE
    }
    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        &self.protocol
    }
    fn protocol_version(&self) -> BoundaryProtocolVersion {
        BoundaryProtocolVersion::new(1)
    }

    fn verify(
        &self,
        envelope: &[u8],
        now: u64,
        maximum_work: std::num::NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        use WorthQueryInboundVerificationDenial as Denial;
        if envelope.len() > MAXIMUM_ENVELOPE {
            return Err(Denial::Oversized);
        }
        // Covers comparison, hexadecimal canonicalization, hashing and bounded claims copies.
        if maximum_work.get() < 4096 + 32 * envelope.len() as u64 {
            return Err(Denial::WorkExhausted);
        }
        let stored = self
            .authenticated
            .lock()
            .map_err(|_| Denial::SourceUnavailable)?;
        if stored.as_deref() != Some(envelope) {
            return Err(Denial::AuthenticationFailed);
        }
        let issued = u64::from_be_bytes(
            envelope
                .get(..8)
                .ok_or(Denial::Malformed)?
                .try_into()
                .map_err(|_| Denial::Malformed)?,
        );
        let expires = issued.checked_add(60).ok_or(Denial::Malformed)?;
        if issued > now || expires < now {
            return Err(Denial::Expired);
        }
        let correlation_token: [u8; 32] = envelope
            .get(8..40)
            .ok_or(Denial::Malformed)?
            .try_into()
            .map_err(|_| Denial::Malformed)?;
        let digest = meaning_digest(envelope, issued, expires, &correlation_token)?;
        Ok(WorthQueryInboundOccurrenceClaims {
            audience: AUDIENCE.to_owned(),
            source_identity: SOURCE.to_owned(),
            key_epoch: 1,
            message_identity: correlation_token,
            signed_meaning_digest: digest,
            issued_at_unix_seconds: issued,
            expires_at_unix_seconds: expires,
            protocol_identity: self.protocol.clone(),
            protocol_version: self.protocol_version(),
            correlation_family: FAMILY.to_owned(),
            correlation_token,
            payload: envelope[40..].to_vec(),
        })
    }
}

fn meaning_digest(
    envelope: &[u8],
    issued: u64,
    expires: u64,
    message: &[u8; 32],
) -> Result<[u8; 32], WorthQueryInboundVerificationDenial> {
    let domain = CanonicalBasisDomain::Future("neutral.archive.authenticated-completion");
    let body = envelope
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    // Every claim is fixed here or derived from these exact authenticated bytes.
    let message = message
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    let semantics =
        format!("{AUDIENCE}|{SOURCE}|1|{message}|{issued}|{expires}|{PROTOCOL}|1|{FAMILY}|{body}");
    let entry = CanonicalBasisEntry::new(
        domain.clone(),
        CanonicalBasisLocus::Named("claims".into()),
        CanonicalBasisEntryKind::Value,
        CanonicalBasisValue::ExactText(semantics.into()),
    );
    let version = CanonicalizationRuleVersion::new("neutral.archive.v1")
        .ok_or(WorthQueryInboundVerificationDenial::Malformed)?;
    let TransitionOutcome::Success(ready) =
        prepare_canonical_basis_sequence(version.clone(), domain, [entry])
    else {
        return Err(WorthQueryInboundVerificationDenial::Malformed);
    };
    let TransitionOutcome::Success(bundle) = prepare_canonical_basis_bundle(version, [ready])
    else {
        return Err(WorthQueryInboundVerificationDenial::Malformed);
    };
    let TransitionOutcome::Success(ready) =
        CanonicalDigestFrontDoor.for_bundle(bundle, CanonicalDigestAlgorithmId::sha256())
    else {
        return Err(WorthQueryInboundVerificationDenial::Malformed);
    };
    Ok(*derive_canonical_digest(ready).value().bytes())
}

fn read_bounded(reader: impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(MAXIMUM_ENVELOPE as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAXIMUM_ENVELOPE {
        return Err("oversized archive response".to_owned());
    }
    Ok(bytes)
}

fn artifact_path(directory: &Path, correlation: &[u8; 32]) -> PathBuf {
    directory.join(
        correlation
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>(),
    )
}

/// Returns false for the normal workflow invocation, true after a real archive write.
pub fn handle_worker() -> Result<bool, String> {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--archive-worker")) {
        return Ok(false);
    }
    let directory = PathBuf::from(args.next().ok_or("missing archive destination")?);
    if args.next().is_some() {
        return Err("unexpected archive argument".to_owned());
    }
    let envelope = read_bounded(std::io::stdin().lock())?;
    if envelope.len() > 32 + MAXIMUM_PAYLOAD {
        return Err("oversized archive request".to_owned());
    }
    let correlation: [u8; 32] = envelope
        .get(..32)
        .ok_or("missing archive correlation")?
        .try_into()
        .map_err(|_| "invalid archive correlation")?;
    std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
    let mut file = std::fs::File::create(artifact_path(&directory, &correlation))
        .map_err(|e| e.to_string())?;
    file.write_all(&envelope[32..]).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    let issued = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| e.to_string())?
        .as_secs();
    let mut response = std::io::stdout().lock();
    response
        .write_all(&issued.to_be_bytes())
        .map_err(|e| e.to_string())?;
    response.write_all(&envelope).map_err(|e| e.to_string())?;
    Ok(true)
}
