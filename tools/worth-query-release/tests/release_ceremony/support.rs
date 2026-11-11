use std::collections::BTreeMap;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicU64, Ordering};

use worth_query_package_archive::facade::{
    decode_package_release_envelope, WorthQueryPackageEnvelopeLimits,
};

const GOLDEN_ENVELOPE_HEX: &str = include_str!(
    "../../../../workspaces/worth-query/crates/worth-query-testkit-package-archive/src/package_archive/archive_protocol/release_envelope/release_envelope_v1_record_v18.hex"
);
const EXPECTED_IDENTITY: &str = "bc5923afa113ac168ab43d923a50b5e0cdee3002ea50888f4bd1453fc084bffd";
static WORLD_SEQUENCE: AtomicU64 = AtomicU64::new(0);

pub(super) const MISMATCH_CASES: [(&str, &str); 9] = [
    (
        "--expected-package-identity",
        "005923afa113ac168ab43d923a50b5e0cdee3002ea50888f4bd1453fc084bffd",
    ),
    ("--expected-release-name", "foreign-release"),
    ("--expected-release-version", "2099.01.01"),
    (
        "--expected-source-repository",
        "https://github.com/foreign/repository",
    ),
    (
        "--expected-source-revision",
        "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
    ),
    ("--expected-source-reference", "refs/tags/foreign"),
    ("--expected-signer-identity", "foreign-signer"),
    (
        "--expected-signature-protocol-identity",
        "worth.release.foreign",
    ),
    ("--expected-signature-protocol-version", "2"),
];

pub(super) struct ReleaseWorld {
    root: PathBuf,
    signing_payload: PathBuf,
    signature: PathBuf,
}

impl ReleaseWorld {
    pub(super) fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "worth-query-release-{}-{}",
            std::process::id(),
            WORLD_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let signing_payload = root.join("release.signing-payload");
        let signature = root.join("release.signature");
        let golden = golden_envelope();
        let decoded =
            decode_package_release_envelope(&golden, WorthQueryPackageEnvelopeLimits::DEFAULT)
                .unwrap();
        fs::write(&signing_payload, decoded.signing_payload()).unwrap();
        fs::write(&signature, decoded.signature()).unwrap();
        Self {
            root,
            signing_payload,
            signature,
        }
    }

    pub(super) fn output_path(&self, name: &str) -> PathBuf {
        self.root.join(name)
    }

    pub(super) fn replace_signing_payload(&self, bytes: &[u8]) {
        fs::write(&self.signing_payload, bytes).unwrap();
    }

    pub(super) fn replace_signature(&self, bytes: &[u8]) {
        fs::write(&self.signature, bytes).unwrap();
    }

    pub(super) fn signing_payload(&self) -> Vec<u8> {
        fs::read(&self.signing_payload).unwrap()
    }

    #[cfg(target_os = "linux")]
    pub(super) fn signing_payload_path(&self) -> &Path {
        &self.signing_payload
    }

    pub(super) fn run(&self, overrides: &[(&str, &str)], output: &Path, report: &Path) -> Output {
        let mut arguments = vec![
            OsString::from("finalize"),
            OsString::from("--signing-payload"),
            self.signing_payload.as_os_str().to_owned(),
            OsString::from("--signature"),
            self.signature.as_os_str().to_owned(),
        ];
        arguments.extend(expectation_arguments(overrides));
        arguments.push(OsString::from("--output"));
        arguments.push(output.as_os_str().to_owned());
        arguments.push(OsString::from("--report"));
        arguments.push(report.as_os_str().to_owned());
        Command::new(env!("CARGO_BIN_EXE_worth-query-release"))
            .args(arguments)
            .output()
            .unwrap()
    }

    pub(super) fn run_preflight(
        &self,
        overrides: &[(&str, &str)],
        staged_signing_payload: &Path,
    ) -> Output {
        self.run_preflight_with_signature_bytes(overrides, 64, staged_signing_payload)
    }

    pub(super) fn run_preflight_with_signature_bytes(
        &self,
        overrides: &[(&str, &str)],
        expected_signature_bytes: u32,
        staged_signing_payload: &Path,
    ) -> Output {
        let signature_bytes = expected_signature_bytes.to_string();
        let mut combined_overrides = overrides.to_vec();
        combined_overrides.push(("--expected-signature-bytes", &signature_bytes));
        let mut arguments = vec![
            OsString::from("preflight"),
            OsString::from("--signing-payload"),
            self.signing_payload.as_os_str().to_owned(),
        ];
        arguments.extend(expectation_arguments(&combined_overrides));
        arguments.push(OsString::from("--staged-signing-payload"));
        arguments.push(staged_signing_payload.as_os_str().to_owned());
        Command::new(env!("CARGO_BIN_EXE_worth-query-release"))
            .args(arguments)
            .output()
            .unwrap()
    }
}

impl Drop for ReleaseWorld {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

pub(super) fn golden_envelope() -> Vec<u8> {
    decode_hex(GOLDEN_ENVELOPE_HEX.trim())
}

fn default_expectations() -> BTreeMap<&'static str, String> {
    BTreeMap::from([
        ("--expected-package-identity", EXPECTED_IDENTITY.to_owned()),
        ("--expected-release-name", "workflow-editor".to_owned()),
        ("--expected-release-version", "2026.08.26".to_owned()),
        (
            "--expected-source-repository",
            "https://github.com/worth/core".to_owned(),
        ),
        (
            "--expected-source-revision",
            "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_owned(),
        ),
        (
            "--expected-source-reference",
            "refs/tags/query-9.16.2".to_owned(),
        ),
        ("--expected-signer-identity", "release-key-01".to_owned()),
        (
            "--expected-signature-protocol-identity",
            "worth.release.ed25519".to_owned(),
        ),
        ("--expected-signature-protocol-version", "1".to_owned()),
        ("--expected-signature-bytes", "64".to_owned()),
    ])
}

fn expectation_arguments(overrides: &[(&str, &str)]) -> Vec<OsString> {
    let mut values = default_expectations();
    for (field, value) in overrides {
        assert!(values.insert(*field, (*value).to_owned()).is_some());
    }
    values
        .into_iter()
        .flat_map(|(field, value)| [OsString::from(field), OsString::from(value)])
        .collect()
}

fn decode_hex(encoded: &str) -> Vec<u8> {
    encoded
        .as_bytes()
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(text, 16).unwrap()
        })
        .collect()
}
