//! Independent processes must each obtain their own opaque C8 custody seal.

use std::{path::Path, process::Command};

use super::{
    admitted_blob_scope, assert_absent, read_exact, recover_serving, shared_format, world,
    SCOPE_NAME,
};

const CHILD_TEST: &str =
    "release_reopen::shared_reuse_custody::fresh_child_checks_source_first_reuse";
const ROOT_ENV: &str = "WORTH_C11_SHARED_C8_ROOT";
const SOURCE_ENV: &str = "WORTH_C11_SHARED_C8_SOURCE";
const DEST_ENV: &str = "WORTH_C11_SHARED_C8_DESTINATION";
const RECOVERY_BYTES_ENV: &str = "WORTH_C11_SHARED_C8_RECOVERY_BYTES";
const EXPECT_ENV: &str = "WORTH_C11_SHARED_C8_EXPECT";

#[derive(Clone, Copy)]
pub(super) enum ExpectedReopen {
    SurvivingDestination,
    BothInvisible,
}

impl ExpectedReopen {
    fn as_env(self) -> &'static str {
        match self {
            Self::SurvivingDestination => "surviving-destination",
            Self::BothInvisible => "both-invisible",
        }
    }

    fn from_env(value: &str) -> Self {
        match value {
            "surviving-destination" => Self::SurvivingDestination,
            "both-invisible" => Self::BothInvisible,
            _ => panic!("unrecognized shared-release child expectation"),
        }
    }
}

pub(super) fn assert_reopens(
    root: &Path,
    source: [u8; 16],
    destination: [u8; 16],
    recovery_bytes: u64,
    expectation: ExpectedReopen,
) {
    let output = Command::new(std::env::current_exe().unwrap())
        .args(["--exact", CHILD_TEST, "--nocapture", "--test-threads=1"])
        .env(ROOT_ENV, root)
        .env(SOURCE_ENV, encode_object(source))
        .env(DEST_ENV, encode_object(destination))
        .env(RECOVERY_BYTES_ENV, recovery_bytes.to_string())
        .env(EXPECT_ENV, expectation.as_env())
        .output()
        .expect("fresh C8 child process");
    assert!(
        output.status.success(),
        "fresh C8 shared-release child: {} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    let child_pid = stdout
        .lines()
        .find_map(|line| line.split_once("WORTH_SHARED_C8_CHILD_PID "))
        .map(|(_, value)| {
            value
                .split_whitespace()
                .next()
                .unwrap()
                .parse::<u32>()
                .unwrap()
        })
        .expect("child executed its C8 recovery and read oracle");
    assert_ne!(child_pid, std::process::id());
}

pub(super) fn run_child() {
    let Ok(root) = std::env::var(ROOT_ENV) else {
        return;
    };
    let source = decode_object(&std::env::var(SOURCE_ENV).unwrap());
    let destination = decode_object(&std::env::var(DEST_ENV).unwrap());
    let recovery_bytes = std::env::var(RECOVERY_BYTES_ENV).unwrap().parse().unwrap();
    let expectation = ExpectedReopen::from_env(&std::env::var(EXPECT_ENV).unwrap());
    std::thread::Builder::new()
        .name("fresh-shared-release-c8".to_owned())
        .stack_size(16 << 20)
        .spawn(move || {
            let serving = recover_serving(Path::new(&root), shared_format(), recovery_bytes);
            let scope = admitted_blob_scope(SCOPE_NAME);
            assert_absent(&serving, &scope, source);
            match expectation {
                ExpectedReopen::SurvivingDestination => {
                    let published = serving
                        .blobs()
                        .unwrap()
                        .resolve_publication(
                            destination,
                            1,
                            &scope,
                            worth_store::physical_runtime::BlobReadLimits::new(
                                std::num::NonZeroU64::new(1).unwrap(),
                            ),
                        )
                        .expect("selected reused destination");
                    assert_eq!(
                        read_exact(&serving, &scope, published, &world::payload()),
                        4
                    );
                }
                ExpectedReopen::BothInvisible => assert_absent(&serving, &scope, destination),
            }
            println!("WORTH_SHARED_C8_CHILD_PID {}", std::process::id());
            serving.close();
        })
        .expect("fresh C8 child worker")
        .join()
        .expect("fresh C8 child worker did not panic");
}

fn encode_object(object: [u8; 16]) -> String {
    object
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>()
}

fn decode_object(text: &str) -> [u8; 16] {
    assert_eq!(text.len(), 32);
    std::array::from_fn(|index| u8::from_str_radix(&text[index * 2..index * 2 + 2], 16).unwrap())
}
