//! Real operations refuse unreadable owner output before changing the selected commit.
use super::super::model::write_admission::{
    self as fixture, BinaryCounter, BrokenBinaryBinding, CanonicalBinding, CanonicalCounter,
    DelegatedBinding, DelegatedCounter, Encoding, Write,
};
use super::*;
mod controls;
use worth_foundational::facade::{InternedString, ValueCodecDenial};
use worth_query_host::facade::{
    declaration::application_schema::{
        ApplicationReadableScalarValueBinding, ApplicationScalarValueBinding,
    },
    runtime::ExecutionAllocationPolicy,
};

#[test]
fn binary_owner_output_that_its_reader_refuses_cannot_publish() {
    let raw = BrokenBinaryBinding::encode(&BinaryCounter(13)).unwrap();
    assert_wire(&raw, &[2, 13, 0, 0, 0, 0, 0, 0, 0]);
    assert!(BrokenBinaryBinding::decode(&raw).is_err());
    refuse_and_retry(
        Encoding::Binary,
        13,
        BrokenBinaryBinding::IDENTITY,
        ValueCodecDenial::UnsupportedVersion {
            expected: 1,
            observed: 2,
        },
    );
}
#[test]
fn canonical_domain_owner_refusal_cannot_publish() {
    canonical_refusal(Encoding::Canonical, 13, ValueCodecDenial::Malformed);
}
#[test]
fn canonical_domain_normalizing_seed_cannot_publish() {
    canonical_refusal(Encoding::Canonical, 15, ValueCodecDenial::NonCanonical);
}
#[test]
fn delegated_domain_owner_refusal_cannot_publish() {
    canonical_refusal(Encoding::Delegated, 13, ValueCodecDenial::Malformed);
}
#[test]
fn delegated_domain_normalizing_seed_cannot_publish() {
    canonical_refusal(Encoding::Delegated, 15, ValueCodecDenial::NonCanonical);
}
fn canonical_refusal(encoding: Encoding, value: u64, cause: ValueCodecDenial) {
    let (raw, identity) = match encoding {
        Encoding::Canonical => (
            CanonicalBinding::encode(&CanonicalCounter(value)).unwrap(),
            CanonicalBinding::IDENTITY,
        ),
        Encoding::Delegated => (
            DelegatedBinding::encode(&DelegatedCounter(value)).unwrap(),
            DelegatedBinding::IDENTITY,
        ),
        Encoding::Binary => panic!("canonical court requires a JSON domain binding"),
    };
    // Frozen decimal spellings are independent of the encoder and admission seed.
    let spelling = match value {
        13 => "13",
        15 => "15",
        _ => panic!("unlisted oracle"),
    };
    assert_eq!(
        raw,
        AspectValue::String(InternedString::Raw(spelling.to_owned()))
    );
    refuse_and_retry(encoding, value, identity, cause);
}
fn refuse_and_retry(
    encoding: Encoding,
    value: u64,
    identity: worth_query_host::facade::declaration::portable_identity::WorthQueryPortableTypeIdentity,
    cause: ValueCodecDenial,
) {
    let (application, authority) = host::install_with_projection();
    let before = super::super::model::required_output_courts::head(&application);
    assert_eq!(read(&application, &authority, &encoding), 5);
    let scope = request_scope();
    let principal = authenticate_operator(application.installed_schema(), &scope);
    let run = |value, key: &str| {
        application
            .runtime()
            .request(&principal, &scope)
            .advanced()
            .mutate(Write {
                key: "numeric".into(),
                value,
                encoding: encoding.clone(),
            })
            .without_source()
            .idempotency(&key.to_owned())
            .execute_in_program(&application, ExecutionAllocationPolicy::SystemAllocation)
            .unwrap()
    };
    let outcome = run(value, "unreadable-domain-write");
    assert_eq!(
        super::super::model::required_output_courts::head(&application),
        before,
        "unreadable domain bytes published: {outcome:?}"
    );
    let WorthQueryApplicationMutationOutcome::Commit(WorthQueryApplicationUncommitted::Denied(
        denial,
    )) = outcome
    else {
        panic!("expected domain readability denial: {outcome:?}")
    };
    assert_eq!(
        denial.kind(),
        WorthQueryApplicationCommitDenialKind::DomainValueRejected
    );
    assert_eq!(denial.domain_value_decode_denial(), Some(&
        worth_query_host::facade::declaration::application_schema::ApplicationValueDecodeDenial::Canonical {
            binding_identity: identity, cause,
        }));
    assert_eq!(read(&application, &authority, &encoding), 5);
    let retry = run(7, "readable-domain-write");
    assert!(
        matches!(
            retry,
            WorthQueryApplicationMutationOutcome::Committed { .. }
        ),
        "{retry:?}"
    );
    assert_ne!(
        super::super::model::required_output_courts::head(&application),
        before
    );
    assert_eq!(read(&application, &authority, &encoding), 7);
}
fn read(
    application: &host::Host,
    authority: &WorthQueryApplicationInvariantProjectionAuthority<NumericSchema>,
    encoding: &Encoding,
) -> u64 {
    let scope = request_scope();
    application
        .runtime()
        .with_application_advancement(&scope, |phase| {
            authority.project(
                &phase,
                |reader| {
                    let entity = reader
                        .resolve_entity(Key::at("Key"), "numeric".to_owned())
                        .unwrap();
                    match encoding {
                        Encoding::Binary => {
                            reader
                                .field(&entity, fixture::Binary::at("Binary"))
                                .unwrap()
                                .0
                        }
                        Encoding::Canonical => {
                            reader
                                .field(&entity, fixture::Canonical::at("Canonical"))
                                .unwrap()
                                .0
                        }
                        Encoding::Delegated => {
                            reader
                                .field(&entity, fixture::Delegated::at("Delegated"))
                                .unwrap()
                                .0
                        }
                    }
                },
                ExecutionAllocationPolicy::SystemAllocation,
            )
        })
        .unwrap()
        .unwrap()
        .into_parts()
        .0
}
