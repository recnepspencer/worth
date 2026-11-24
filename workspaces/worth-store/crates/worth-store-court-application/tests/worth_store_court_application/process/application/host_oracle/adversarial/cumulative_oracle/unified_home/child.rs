//! Fresh OS child session for the C4b.4 Single-Home Unified 4-Family Oracle.
use super::super::super::super::graceful::signal::derived_discard::host::checkpoint_and_discard_signal;
use super::super::super::workflow_correction_fixture::{seed_approved_workflow, WorkflowHome};
use super::ops::{
    advance_doc_wf_historical, bind_unified_wf, continue_doc_custody, exec_set_retention,
    exec_wf_correctable, fork_doc_branch, observe_doc_wf_locators, observe_unified,
    open_unified_raw, raw_cadence, run_unified_adopt_p1, DocRuntime, RawDocHome,
};
use super::second_wf::run_second_wf_to_historical;
use super::wire::{UnifiedCmd, UnifiedReply, UnifiedSnap};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};
use worth_execution::ExecutionRequest;
use worth_query_host::facade::{
    WorthQueryApplicationHomeCloseOutcome as CloseOutcome,
    WorthQueryApplicationHomeLocation as Location,
};
use worth_query_testkit_document_retention::facade::document_retention_model::workflow::CertificationAuthenticationOwner;
use worth_query_testkit_host::facade::host::checkpoint_capture;
use worth_store_application::ApplicationHomeCheckpointDenial;

const CHILD: &str =
    "process::application::host_oracle::adversarial::cumulative_oracle::unified_home::child::c4b4_unified_home_child";
const ENV_HOME: &str = "WORTH_C4B4_UNI_HOME";
const ENV_CH: &str = "WORTH_C4B4_UNI_CH";
const ENV_BUILD: &str = "WORTH_C4B4_UNI_BUILD";

pub(super) struct UnifiedSession {
    child: Child,
    channel: PathBuf,
}

impl UnifiedSession {
    pub(super) fn start(
        home: &Path,
        channel: &Path,
        build: &str,
        wf_locators: &[Vec<u8>],
    ) -> (Self, UnifiedReply) {
        std::fs::create_dir_all(channel).unwrap();
        std::fs::write(
            channel.join("wf_locators.json"),
            serde_json::to_vec(wf_locators).unwrap(),
        )
        .unwrap();
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
        let init = s.recv();
        (s, init)
    }

    pub(super) fn send(&mut self, cmd: UnifiedCmd) -> UnifiedReply {
        crate::killed_child::announce(
            &self.channel.join("request"),
            &serde_json::to_vec(&cmd).unwrap(),
        );
        self.recv()
    }

    fn recv(&mut self) -> UnifiedReply {
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
                "c4b4_unified_home_child stopped/timed out: {ended:?}\n{}\n{}",
                std::fs::read_to_string(self.channel.join("stdout.log")).unwrap_or_default(),
                std::fs::read_to_string(self.channel.join("stderr.log")).unwrap_or_default()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }

    pub(super) fn kill_live(&mut self, ctx: &str) {
        let exited = self
            .child
            .try_wait()
            .unwrap_or_else(|e| panic!("{ctx}: poll unified child: {e:?}"));
        assert!(
            exited.is_none(),
            "{ctx}: c4b4_unified_home_child exited before parent OS kill: {exited:?}"
        );
        self.child
            .kill()
            .unwrap_or_else(|e| panic!("{ctx}: OS kill unified child: {e:?}"));
        let status = self
            .child
            .wait()
            .unwrap_or_else(|e| panic!("{ctx}: wait unified child: {e:?}"));
        crate::killed_child::assert_os_killed(status, ctx);
    }

    pub(super) fn finish(mut self) {
        let status = self.child.wait().unwrap();
        assert!(status.success(), "unified child exit: {status:?}");
    }
}

impl Drop for UnifiedSession {
    fn drop(&mut self) {
        if let Ok(None) = self.child.try_wait() {
            let _ = self.child.kill();
        }
        let _ = self.child.wait();
    }
}

enum OpenSlot {
    Raw(RawDocHome),
    Wf(WorkflowHome, CertificationAuthenticationOwner),
    Closed,
}

impl OpenSlot {
    fn runtime(&self) -> &DocRuntime {
        match self {
            Self::Raw(h) => h.runtime(),
            Self::Wf(w, _) => w.runtime(),
            Self::Closed => panic!("slot closed"),
        }
    }

    fn snap(
        &self,
        wf_locators: &[Vec<u8>],
        open_cadence: &BTreeMap<String, (u64, u64)>,
        exec: &ExecutionRequest<'_, '_>,
    ) -> UnifiedSnap {
        match self {
            Self::Raw(h) => observe_unified(h.runtime(), Vec::new(), raw_cadence(h, exec)),
            Self::Wf(w, _) => observe_unified(
                w.runtime(),
                observe_doc_wf_locators(w, wf_locators),
                open_cadence.clone(),
            ),
            Self::Closed => panic!("slot closed"),
        }
    }
}

#[test]
fn c4b4_unified_home_child() {
    let Some(home_path) = std::env::var_os(ENV_HOME).map(PathBuf::from) else {
        return;
    };
    let channel = PathBuf::from(std::env::var_os(ENV_CH).unwrap());
    let build = std::env::var(ENV_BUILD).unwrap();
    let mut wf_locators: Vec<Vec<u8>> =
        serde_json::from_slice(&std::fs::read(channel.join("wf_locators.json")).unwrap()).unwrap();
    let lease = checkpoint_capture::lease(256 << 20);
    let exec = ExecutionRequest::leased(&lease);

    let first_raw = open_unified_raw(&Location::Directory(home_path.clone()), &build, None, exec);
    let open_cadence = raw_cadence(&first_raw, &exec);
    let mut slot = if !wf_locators.is_empty() {
        let (w, a) = bind_unified_wf(first_raw);
        OpenSlot::Wf(w, a)
    } else {
        OpenSlot::Raw(first_raw)
    };
    let send = |body: UnifiedReply| {
        crate::killed_child::announce(
            &channel.join("response"),
            &serde_json::to_vec(&body).unwrap(),
        );
    };
    send(UnifiedReply {
        snap: Some(slot.snap(&wf_locators, &open_cadence, &exec)),
        ..Default::default()
    });

    loop {
        let req_path = channel.join("request");
        if !req_path.is_file() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let cmd: UnifiedCmd = serde_json::from_slice(&std::fs::read(&req_path).unwrap()).unwrap();
        std::fs::remove_file(req_path).unwrap();
        let mut reply = UnifiedReply::default();
        match cmd {
            UnifiedCmd::SetRetention {
                branch_idx,
                days,
                key,
            } => {
                let (f, r) = exec_set_retention(slot.runtime(), branch_idx, days, key);
                reply.fate_tag = Some(f);
                reply.receipt_id = r;
            }
            UnifiedCmd::ForkBranch => fork_doc_branch(slot.runtime()),
            UnifiedCmd::SeedWorkflow => {
                if matches!(slot, OpenSlot::Raw(_)) {
                    let OpenSlot::Raw(h) = std::mem::replace(&mut slot, OpenSlot::Closed) else {
                        unreachable!();
                    };
                    let (w, a) = bind_unified_wf(h);
                    slot = OpenSlot::Wf(w, a);
                }
                let OpenSlot::Wf(w, a) = &slot else {
                    unreachable!();
                };
                let orig = seed_approved_workflow(w, a);
                let bytes = orig.to_recorded_bytes().unwrap();
                wf_locators.push(bytes.clone());
                reply.wf_locator = Some(bytes);
            }
            UnifiedCmd::RunCorrectableStep => {
                let OpenSlot::Wf(w, _) = &slot else {
                    panic!("wf must be bound");
                };
                reply.fate_tag = Some(exec_wf_correctable(w));
            }
            UnifiedCmd::RunSecondWfToHistorical => {
                let OpenSlot::Wf(w, a) = &slot else {
                    panic!("wf must be bound");
                };
                let orig2 = run_second_wf_to_historical(w, a);
                let bytes = orig2.to_recorded_bytes().unwrap();
                wf_locators.push(bytes.clone());
                reply.wf_locator = Some(bytes);
            }
            UnifiedCmd::AdvanceWorkflowHistorical => {
                let OpenSlot::Wf(w, _) = &slot else {
                    panic!("wf must be bound");
                };
                advance_doc_wf_historical(w);
            }
            UnifiedCmd::AdoptP1 => {
                run_unified_adopt_p1(slot.runtime());
            }
            UnifiedCmd::ContinueCustody => continue_doc_custody(slot.runtime(), &exec),
            UnifiedCmd::CheckpointAndDiscard => {
                if matches!(slot, OpenSlot::Wf(_, _)) {
                    let OpenSlot::Wf(w, a) = std::mem::replace(&mut slot, OpenSlot::Closed) else {
                        unreachable!();
                    };
                    drop(a);
                    w.close(&exec).unwrap().reclaim();
                    let raw = open_unified_raw(
                        &Location::Directory(home_path.clone()),
                        &build,
                        None,
                        exec,
                    );
                    slot = OpenSlot::Raw(raw);
                }
                let OpenSlot::Raw(h) = &slot else {
                    unreachable!();
                };
                reply.discard_gen = Some(checkpoint_and_discard_signal(h, &exec).1);
                if !wf_locators.is_empty() {
                    let OpenSlot::Raw(h) = std::mem::replace(&mut slot, OpenSlot::Closed) else {
                        unreachable!();
                    };
                    let (w, a) = bind_unified_wf(h);
                    slot = OpenSlot::Wf(w, a);
                }
            }
            UnifiedCmd::Close { expect_discarded } => {
                let check_outcome = |out: &CloseOutcome| match out {
                    CloseOutcome::Closed {
                        checkpoint: Err(ApplicationHomeCheckpointDenial::DerivedStateDiscarded),
                        ..
                    } if expect_discarded => None,
                    CloseOutcome::Closed {
                        checkpoint: Ok(cp), ..
                    } if !expect_discarded => Some(cp.generation()),
                    _ => panic!("unexpected unified close (discarded={expect_discarded})"),
                };
                reply.close_gen = match std::mem::replace(&mut slot, OpenSlot::Closed) {
                    OpenSlot::Wf(w, a) => {
                        drop(a);
                        let c = w.close(&exec).unwrap();
                        assert!(c.is_closed());
                        let g = check_outcome(c.outcome());
                        c.reclaim();
                        g
                    }
                    OpenSlot::Raw(h) => {
                        let c = h.close(&exec).unwrap();
                        assert!(c.is_closed());
                        let g = check_outcome(c.outcome());
                        c.reclaim();
                        g
                    }
                    OpenSlot::Closed => panic!("already closed"),
                };
                reply.closed = true;
                send(reply);
                return;
            }
        }
        reply.snap = Some(slot.snap(&wf_locators, &open_cadence, &exec));
        send(reply);
    }
}
