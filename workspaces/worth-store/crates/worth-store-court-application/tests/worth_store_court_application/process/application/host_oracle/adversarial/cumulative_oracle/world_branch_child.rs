//! Fresh OS child process session & wire types for C4b.4 C2.8 Durable World
//! Branch Creation, Retirement & History Disposal lane.
pub(super) mod ops;
mod runner;

use super::super::assert_barrier_killed;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use worth_store::physical_runtime::certification::MediaOperationRole;

const CHILD: &str =
    "process::application::host_oracle::adversarial::cumulative_oracle::world_branch_child::runner::c4b4_world_branch_child";
const BARRIER_CHILD: &str =
    "process::application::host_oracle::adversarial::cumulative_oracle::world_branch_child::runner::c4b4_world_branch_barrier_child";
pub(self) const ENV_HOME: &str = "WORTH_C4B4_WB_HOME";
pub(self) const ENV_CH: &str = "WORTH_C4B4_WB_CH";
pub(self) const ENV_BUILD: &str = "WORTH_C4B4_WB_BUILD";
pub(self) const ENV_BARRIER_SPEC: &str = "WORTH_C4B4_WB_BSPEC";

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct BranchView {
    pub(super) name: String,
    pub(super) incarnation: u64,
    pub(super) commit_ordinal: u64,
    pub(super) reference_gen: u64,
    pub(super) native_branch: String,
    pub(super) signal_branch: String,
    pub(super) entity_payloads: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub(super) struct WorldSnap {
    pub(super) branches: Vec<BranchView>,
    pub(super) installed_commits: usize,
    pub(super) unique_pins: usize,
    pub(super) pending_custody: usize,
    pub(super) cadence: BTreeMap<String, (u64, u64)>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub(super) enum WbCmd {
    BootstrapRoot,
    CreateBranch {
        parent: String,
        name: String,
        fork_prefix: String,
        native_fork: bool,
        signal_fork: bool,
    },
    Publish {
        branch: String,
        round: usize,
    },
    ContinuePendingCustody,
    RetireBranch {
        name: String,
    },
    DisposeRetiredHistory {
        retired_commit: u64,
    },
    RetireChangedHistory,
    AssertDisposed {
        ordinals: Vec<u64>,
    },
    AssertReadable {
        ordinals: Vec<u64>,
    },
    AssertProtectedInAncestry {
        ordinals: Vec<u64>,
    },
    CheckpointAndDiscard,
    Close {
        expect_discarded: bool,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub(super) struct WbReply {
    pub(super) snap: Option<WorldSnap>,
    pub(super) retired_commit: Option<u64>,
    pub(super) reclaimed_ordinals: Vec<u64>,
    pub(super) custody_continued: Option<bool>,
    pub(super) discard_gen: Option<u64>,
    pub(super) close_gen: Option<u64>,
    pub(super) closed: bool,
}

pub(super) struct WorldBranchSession {
    child: Child,
    channel: PathBuf,
}

impl WorldBranchSession {
    pub(super) fn start(home: &Path, channel: &Path, build: &str) -> (Self, WbReply) {
        std::fs::create_dir_all(channel).unwrap();
        let stdout = std::fs::File::create(channel.join("stdout.log")).unwrap();
        let stderr = std::fs::File::create(channel.join("stderr.log")).unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env(ENV_HOME, home)
            .env(ENV_CH, channel)
            .env(ENV_BUILD, build)
            .stdin(Stdio::null())
            .stdout(Stdio::from(stdout))
            .stderr(Stdio::from(stderr))
            .spawn()
            .unwrap();
        let mut s = Self {
            child,
            channel: channel.to_path_buf(),
        };
        let open = s.recv();
        (s, open)
    }

    pub(super) fn send(&mut self, cmd: WbCmd) -> WbReply {
        crate::killed_child::announce(
            &self.channel.join("request"),
            &serde_json::to_vec(&cmd).unwrap(),
        );
        self.recv()
    }

    fn recv(&mut self) -> WbReply {
        let deadline = Instant::now() + Duration::from_secs(180);
        let file = self.channel.join("response");
        loop {
            if file.is_file() {
                let bytes = std::fs::read(&file).unwrap();
                std::fs::remove_file(&file).unwrap();
                return serde_json::from_slice(&bytes).unwrap();
            }
            let ended = self.child.try_wait().unwrap();
            if ended.is_some() && file.is_file() {
                continue;
            }
            assert!(
                ended.is_none() && Instant::now() < deadline,
                "c4b4_world_branch_child stopped/timed out: {ended:?}\n{}\n{}",
                std::fs::read_to_string(self.channel.join("stdout.log")).unwrap_or_default(),
                std::fs::read_to_string(self.channel.join("stderr.log")).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub(super) fn kill_live(&mut self, ctx: &str) {
        let exited = self.child.try_wait().expect("poll wb child before OS kill");
        assert!(
            exited.is_none(),
            "{ctx}: c4b4_world_branch_child exited before parent-induced kill: {exited:?}"
        );
        self.child.kill().expect("OS kill wb child");
        let status = self.child.wait().expect("reap killed wb child");
        crate::killed_child::assert_os_killed(status, ctx);
    }

    pub(super) fn finish(mut self) {
        let status = self.child.wait().unwrap();
        assert!(status.success(), "wb child exit: {status:?}");
    }
}

impl Drop for WorldBranchSession {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

pub(super) fn kill_wb_publish_barrier(
    home: &Path,
    branch: &str,
    round: usize,
    build: &str,
    nth: u64,
    ctx: &str,
) {
    let markers = tempfile::tempdir().unwrap();
    let spec = PathBuf::from(format!("publish|{branch}|{round}|{nth}"));
    let ready = markers.path().join("ready");
    let end = crate::killed_child::run_and_kill_with_ctx(
        BARRIER_CHILD,
        &[
            (ENV_HOME, home),
            (ENV_CH, markers.path()),
            (ENV_BUILD, Path::new(build)),
            (ENV_BARRIER_SPEC, &spec),
        ],
        &ready,
        &markers.path().join("finished"),
        ctx,
    );
    assert_barrier_killed(
        end,
        &ready,
        MediaOperationRole::SynchronizeFileState,
        nth,
        ctx,
    );
}
