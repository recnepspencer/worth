#[path = "child_process/admission_probe.rs"]
mod admission_probe;
#[path = "child_process/dispatch.rs"]
mod dispatch;
#[path = "child_process/invocation.rs"]
mod invocation;
#[path = "child_process/locator_codec.rs"]
mod locator_codec;
#[path = "child_process/ownership_probe.rs"]
mod ownership_probe;
#[path = "child_process/record_round_trip.rs"]
mod record_round_trip;
#[path = "child_process/segment_read.rs"]
mod segment_read;

pub(super) use invocation::{
    child_command, run_child, run_courtroom_reopener, run_courtroom_writer,
};
pub(super) use locator_codec::{decode_locator, hex, unhex};

const CHILD_TEST: &str = "child_process::dispatch::c5_child_role";
const ROLE_ENV: &str = "WORTH_STORE_C5_CHILD_ROLE";
const ROOT_ENV: &str = "WORTH_STORE_C5_CHILD_ROOT";
pub(super) const LOCATOR_ENV: &str = "WORTH_STORE_C5_LOCATOR";
const ORACLE_ENV: &str = "WORTH_STORE_C5_ORACLE";
