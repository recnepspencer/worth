//! Home-level source security survives fresh verifier configuration.
use super::*;
use worth_foundational::facade::{BoundaryProtocolIdentity, BoundaryProtocolVersion};
use worth_query_host::facade::primary_graph::{
    WorthQueryInboundAdmissionDenial as Denial, WorthQueryInboundOccurrenceClaims,
    WorthQueryInboundOccurrenceVerifier, WorthQueryInboundVerificationDenial,
    WorthQueryInboundVerifierHandle,
};
use worth_query_testkit_host::facade::host::dispatch_home::inbound_source::{
    self as sources, InboundNotice, InboundSibling, OtherSourceNotice,
};

// The host pins one exact source message per route. This proves source admission,
// not a signature algorithm. Verification returns the immutable body meaning.
struct Verifier {
    claims: WorthQueryInboundOccurrenceClaims,
    envelope: Vec<u8>,
}
impl Verifier {
    fn message(source: &'static str, audience: &'static str) -> Self {
        let issued = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut claims = WorthQueryInboundOccurrenceClaims {
            audience: audience.to_owned(),
            source_identity: source.to_owned(),
            key_epoch: 1,
            message_identity: [0x31; 32],
            signed_meaning_digest: [0; 32],
            issued_at_unix_seconds: issued,
            expires_at_unix_seconds: issued.checked_add(30).unwrap(),
            protocol_identity: sources::PROTOCOL,
            protocol_version: BoundaryProtocolVersion::new(1),
            correlation_family: dispatch::notify::CORRELATION_FAMILY.to_owned(),
            correlation_token: [0x51; 32],
            payload: vec![0; 8],
        };
        let body = serde_json::json!([
            claims.audience,
            claims.source_identity,
            claims.key_epoch,
            claims.message_identity.to_vec(),
            claims.issued_at_unix_seconds,
            claims.expires_at_unix_seconds,
            claims.protocol_identity.as_str(),
            claims.protocol_version.get(),
            claims.correlation_family,
            claims.correlation_token.to_vec(),
            claims.payload,
        ])
        .to_string();
        claims.signed_meaning_digest = signed_meaning(&body);
        Self {
            claims,
            envelope: body.into_bytes(),
        }
    }
}
impl WorthQueryInboundOccurrenceVerifier for Verifier {
    fn audience(&self) -> &str {
        &self.claims.audience
    }
    fn source_identity(&self) -> &str {
        &self.claims.source_identity
    }
    fn protocol_identity(&self) -> &BoundaryProtocolIdentity {
        &self.claims.protocol_identity
    }
    fn protocol_version(&self) -> BoundaryProtocolVersion {
        self.claims.protocol_version
    }
    fn verify(
        &self,
        envelope: &[u8],
        now: u64,
        work: std::num::NonZeroU64,
    ) -> Result<WorthQueryInboundOccurrenceClaims, WorthQueryInboundVerificationDenial> {
        // Two complete byte passes bound comparison plus cloning the fixed body meaning.
        if work.get()
            < u64::try_from(self.envelope.len())
                .unwrap()
                .checked_mul(2)
                .unwrap()
        {
            return Err(WorthQueryInboundVerificationDenial::WorkExhausted);
        }
        if envelope != self.envelope {
            return Err(WorthQueryInboundVerificationDenial::AuthenticationFailed);
        }
        if now < self.claims.issued_at_unix_seconds || now > self.claims.expires_at_unix_seconds {
            return Err(WorthQueryInboundVerificationDenial::Expired);
        }
        Ok(self.claims.clone())
    }
}
fn signed_meaning(body: &str) -> [u8; 32] {
    use worth_foundational::facade::*;
    let domain = CanonicalBasisDomain::Future("worth-query.test.inbound-source-probe");
    let prepared = prepare_canonical_basis_sequence(
        CanonicalizationRuleVersion::new("inbound-source-probe-v1").unwrap(),
        domain,
        vec![CanonicalBasisEntry::new(
            domain,
            CanonicalBasisLocus::Named("signed-body".into()),
            CanonicalBasisEntryKind::Identity,
            CanonicalBasisValue::ExactText(body.to_owned().into()),
        )],
    )
    .into_result()
    .unwrap();
    let ready = canonicalization()
        .digest()
        .for_sequence_with_budget(
            prepared,
            CanonicalDigestAlgorithmId::sha256(),
            CanonicalDigestWorkBudget::new(8, 4096).unwrap(),
        )
        .into_result()
        .unwrap();
    *canonicalization().digest().derive(ready).value().bytes()
}
struct SourceRoute {
    handle: WorthQueryInboundVerifierHandle,
    envelope: Vec<u8>,
}
fn install(home: &Home, audience: &'static str) -> (SourceRoute, SourceRoute, SourceRoute) {
    let runtime = home.runtime().runtime();
    let schema = runtime.installed_schema();
    let verifier = Verifier::message(sources::SOURCE, audience);
    let envelope = verifier.envelope.clone();
    let handle = runtime
        .install_inbound_occurrence_verifier(
            &schema
                .installed_operation(InboundNotice::reference())
                .unwrap(),
            Arc::new(verifier),
        )
        .unwrap();
    let first = SourceRoute { handle, envelope };
    let verifier = Verifier::message(sources::SOURCE, audience);
    let envelope = verifier.envelope.clone();
    let handle = runtime
        .install_inbound_occurrence_verifier(
            &schema
                .installed_operation(InboundSibling::reference())
                .unwrap(),
            Arc::new(verifier),
        )
        .unwrap();
    let sibling = SourceRoute { handle, envelope };
    let verifier = Verifier::message(sources::OTHER_SOURCE, audience);
    let envelope = verifier.envelope.clone();
    let handle = runtime
        .install_inbound_occurrence_verifier(
            &schema
                .installed_operation(OtherSourceNotice::reference())
                .unwrap(),
            Arc::new(verifier),
        )
        .unwrap();
    (first, sibling, SourceRoute { handle, envelope })
}
fn probe(home: &Home, route: &SourceRoute) -> Denial {
    let (_, scope) = pending_support::scope();
    home.runtime()
        .runtime()
        .receive_inbound_occurrence(&route.handle, &route.envelope, &scope)
        .err()
        .expect("these probes have no original dispatch")
}

fn source_control_survives(revoke: bool) {
    let location = dispatch::location();
    let handlers = Arc::new(AtomicUsize::new(0));
    let lease = checkpoint_capture::lease(128 * 1024 * 1024);
    let execution = worth_execution::ExecutionRequest::leased(&lease);
    let rail = ScriptedRail::answering(&[]);
    let home = open(&location, execution, &handlers, true, &rail);
    let (first, sibling, other) = install(&home, "before-close");
    assert_eq!(probe(&home, &first), Denial::UnknownCorrelation);
    let runtime = home.runtime().runtime();
    if revoke {
        runtime
            .revoke_inbound_occurrence_source(&first.handle)
            .unwrap();
    } else {
        runtime
            .retire_inbound_occurrence_source(&first.handle)
            .unwrap();
    }
    let expected = if revoke {
        Denial::SourceRevoked
    } else {
        Denial::SourceRetired
    };
    assert_eq!(
        probe(&home, &sibling),
        expected,
        "a sibling slot cannot revive the same source"
    );
    assert_eq!(probe(&home, &other), Denial::UnknownCorrelation);
    Arc::into_inner(home)
        .unwrap_or_else(|| panic!("exclusive Home"))
        .close(&execution)
        .unwrap_or_else(|cause| panic!("source facts close durably: {cause:?}"))
        .reclaim();
    let fresh_rail = ScriptedRail::answering(&[]);
    let reopened = open(&location, execution, &handlers, false, &fresh_rail);
    let (fresh, fresh_sibling, fresh_other) = install(&reopened, "after-close");
    assert_eq!(probe(&reopened, &first), Denial::ForeignVerifier);
    assert_eq!(probe(&reopened, &fresh), expected);
    assert_eq!(probe(&reopened, &fresh_sibling), expected);
    assert_eq!(probe(&reopened, &fresh_other), Denial::UnknownCorrelation);
    reopened
        .runtime()
        .runtime()
        .retire_inbound_occurrence_source(&fresh.handle)
        .unwrap();
    assert_eq!(
        probe(&reopened, &fresh),
        expected,
        "retirement cannot downgrade revocation"
    );
    assert_eq!(handlers.load(Ordering::SeqCst), 0);
    assert!(rail.contacts().is_empty() && fresh_rail.contacts().is_empty());
    Arc::into_inner(reopened)
        .unwrap_or_else(|| panic!("exclusive reopened Home"))
        .close(&execution)
        .unwrap_or_else(|cause| panic!("reopened source facts close again: {cause:?}"))
        .reclaim();
}
#[test]
fn a_retired_source_stays_retired_across_fresh_home_verifiers() {
    source_control_survives(false);
}
#[test]
fn a_revoked_source_stays_revoked_across_fresh_home_verifiers() {
    source_control_survives(true);
}
