//! One real test-binary service process; reopening always kills its predecessor.
use super::protocol::{Request, Response};
use super::*;
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
pub(super) const CHILD: &str =
    "process::application::host_oracle::session::host_oracle_service_child";
pub(super) struct Session {
    pub child: Child,
    pub channel: PathBuf,
}
impl Session {
    pub fn start(path: &Path, channel: &Path, seed: u64) -> (Self, Response) {
        Self::start_with_build(path, channel, seed, None)
    }
    pub fn start_with_build(
        path: &Path,
        channel: &Path,
        seed: u64,
        build: Option<&str>,
    ) -> (Self, Response) {
        Self::start_with_build_and_wal(path, channel, seed, build, None)
    }
    pub fn start_with_build_and_wal(
        path: &Path,
        channel: &Path,
        seed: u64,
        build: Option<&str>,
        wal_bytes: Option<u64>,
    ) -> (Self, Response) {
        std::fs::create_dir_all(channel).unwrap();
        let mut command = Command::new(std::env::current_exe().unwrap());
        command
            .args(["--exact", CHILD, "--nocapture"])
            .env("WORTH_HOST_ORACLE_CHANNEL", channel)
            .env("WORTH_HOST_ORACLE_HOME", path)
            .env("WORTH_HOST_ORACLE_SEED", seed.to_string())
            .stdout(Stdio::from(
                std::fs::File::create(channel.join("stdout.log")).unwrap(),
            ))
            .stderr(Stdio::from(
                std::fs::File::create(channel.join("stderr.log")).unwrap(),
            ));
        if let Some(build) = build {
            command.env("WORTH_HOST_ORACLE_BUILD", build);
        }
        if let Some(bytes) = wal_bytes {
            command.env("WORTH_HOST_ORACLE_WAL_BYTES", bytes.to_string());
        }
        let child = command.spawn().unwrap();
        let mut session = Self {
            child,
            channel: channel.into(),
        };
        let opening = session.receive();
        (session, opening)
    }
    pub fn send(&mut self, request: Request) -> Response {
        self.dispatch(request);
        self.receive()
    }
    pub fn dispatch(&self, request: Request) {
        crate::killed_child::announce(
            &self.channel.join("request"),
            &serde_json::to_vec(&request).unwrap(),
        );
    }
    fn receive(&mut self) -> Response {
        let deadline = Instant::now() + Duration::from_secs(240);
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
                "Host child stopped or did not respond: {ended:?}\n{}\n{}",
                std::fs::read_to_string(self.channel.join("stdout.log")).unwrap(),
                std::fs::read_to_string(self.channel.join("stderr.log")).unwrap()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    pub fn kill(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            self.child.kill().unwrap();
        }
        self.child.wait().unwrap();
    }
    /// Explicitly terminate a STILL-LIVE child process with a real OS kill,
    /// carrying the caller's replay context on every failure assertion.
    pub fn kill_live_with_ctx(&mut self, ctx: &str) {
        let exited = self
            .child
            .try_wait()
            .unwrap_or_else(|e| panic!("{ctx}: poll Host child before OS kill: {e:?}"));
        assert!(
            exited.is_none(),
            "{ctx}: Host child exited before parent-induced OS kill: {exited:?}"
        );
        self.child
            .kill()
            .unwrap_or_else(|e| panic!("{ctx}: parent OS kill of live Host child failed: {e:?}"));
        let status = self
            .child
            .wait()
            .unwrap_or_else(|e| panic!("{ctx}: reap OS-killed Host child: {e:?}"));
        crate::killed_child::assert_os_killed(status, ctx);
    }
    pub fn finish(&mut self) {
        assert!(self.child.wait().unwrap().success(), "graceful child exit");
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.kill();
    }
}
#[test]
fn host_oracle_service_child() {
    let Some(channel) = std::env::var_os("WORTH_HOST_ORACLE_CHANNEL").map(PathBuf::from) else {
        return;
    };
    let wal_profile = std::env::var_os("WORTH_HOST_ORACLE_WAL_BYTES").map(|raw| {
        let bytes = raw
            .to_str()
            .unwrap()
            .parse::<std::num::NonZeroU64>()
            .unwrap();
        worth_store_application::ApplicationHomeCertificationProfile::with_wal_segment_bytes(bytes)
    });
    let run = || run_host_oracle_service_child(channel);
    match wal_profile {
        Some(profile) => profile.run(run),
        None => run(),
    }
}

fn run_host_oracle_service_child(channel: PathBuf) {
    let path = PathBuf::from(std::env::var_os("WORTH_HOST_ORACLE_HOME").unwrap());
    let seed = std::env::var("WORTH_HOST_ORACLE_SEED")
        .unwrap()
        .parse()
        .unwrap();
    let script = HostScript::generate(seed);
    let request = super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let contacts = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let build = std::env::var("WORTH_HOST_ORACLE_BUILD")
        .ok()
        .map(|value| worth_foundational::facade::ApplicationBuildIdentity::parse(&value).unwrap())
        .unwrap_or_else(super::opening::build);
    let mut home = Some(host_observation::opening::open(
        worth_query_host::facade::WorthQueryApplicationHomeLocation::Directory(path),
        build,
        initialized.clone(),
        contacts.clone(),
        None,
        super::execution::limits(),
        &execution,
    ));
    let mut originals = std::collections::HashMap::new();
    let respond = |response: Response| {
        crate::killed_child::announce(
            &channel.join("response"),
            &serde_json::to_vec(&response).unwrap(),
        )
    };
    respond(Response {
        state: Some(host_observation::projection::observe(
            home.as_ref().unwrap().runtime(),
            &execution,
        )),
        mutation: None,
        initialized: initialized.load(std::sync::atomic::Ordering::SeqCst),
        contacts: contacts.load(std::sync::atomic::Ordering::SeqCst),
        closed: false,
        disposition: None,
    });
    loop {
        let input = channel.join("request");
        if !input.is_file() {
            std::thread::sleep(std::time::Duration::from_millis(10));
            continue;
        }
        let command: Request = serde_json::from_slice(&std::fs::read(&input).unwrap()).unwrap();
        std::fs::remove_file(input).unwrap();
        let mut mutation = None;
        let mut disposition = None;
        let mut closed = false;
        match command {
            Request::Mutate { key, value } => {
                mutation = super::execution::mutation(
                    home.as_ref().unwrap(),
                    &HostCommand::Mutate {
                        branch: BranchLabel::Main,
                        key,
                        value,
                    },
                    &mut originals,
                );
            }
            Request::Close => {
                let result = home
                    .take()
                    .unwrap()
                    .close(&execution)
                    .unwrap_or_else(|refused| panic!("graceful oracle close: {refused:?}"));
                let generation = match result.outcome() {
                    worth_query_host::facade::WorthQueryApplicationHomeCloseOutcome::Closed {
                        checkpoint: Ok(checkpoint),
                        ..
                    } => Some(checkpoint.generation()),
                    _ => None,
                };
                closed = result.is_closed();
                result.reclaim();
                disposition =
                    Some(super::protocol::Disposition::ClosedWithCheckpoint { generation });
            }
            Request::Checkpoint => {
                let checkpoint = home.as_ref().unwrap().checkpoint(&execution).unwrap();
                disposition = Some(super::protocol::Disposition::Checkpointed {
                    generation: checkpoint.generation(),
                });
            }
            Request::CheckpointAndDiscard => {
                let home = home.as_ref().unwrap();
                let checkpoint = home.checkpoint(&execution).unwrap();
                let discard_generation = home.discard_derived_for_test(&execution).unwrap();
                assert_eq!(discard_generation, checkpoint.generation() + 1);
                disposition = Some(super::protocol::Disposition::Discarded {
                    checkpoint_generation: checkpoint.generation(),
                    discard_generation,
                });
            }
            Request::CloseExpectingDiscarded => {
                let result = home
                    .take()
                    .unwrap()
                    .close(&execution)
                    .unwrap_or_else(|refused| panic!("discarded oracle close: {refused:?}"));
                assert!(matches!(
                    result.outcome(),
                    worth_query_host::facade::WorthQueryApplicationHomeCloseOutcome::Closed {
                        checkpoint: Err(
                            worth_store_application::ApplicationHomeCheckpointDenial::DerivedStateDiscarded
                        ),
                        ..
                    }
                ));
                closed = result.is_closed();
                result.reclaim();
                disposition =
                    Some(super::protocol::Disposition::ClosedWithCheckpoint { generation: None });
            }
            Request::InspectCadence => {
                let inspection = home
                    .as_ref()
                    .unwrap()
                    .inspect_checkpoint(&execution)
                    .unwrap();
                let journals = inspection
                    .journals
                    .iter()
                    .map(|j| (j.name.to_owned(), j.covered, j.head))
                    .collect();
                disposition = Some(super::protocol::Disposition::Cadence { journals });
            }
            Request::Observe => (),
            Request::ObserveReleased { key, value } => {
                disposition = Some(super::protocol::Disposition::ReleasedOriginal(
                    super::released_original::terminal_projection(
                        home.as_ref().unwrap(),
                        key,
                        value,
                        &execution,
                    ),
                ));
            }
            Request::ObserveReleasedCopyScale { key, value } => {
                disposition = Some(super::protocol::Disposition::OriginalBindingCopyScale(
                    super::released_original::original_copy_scale(
                        home.as_ref().unwrap(),
                        key,
                        value,
                        &execution,
                    ),
                ));
            }
            Request::AbandonOriginal { key, value } => {
                let home = home.as_ref().unwrap();
                let before = host_observation::projection::observe(home.runtime(), &execution);
                let outcome =
                    host_interruption::publication::abandon_after_owner(home.runtime(), key, value);
                if let worth_query_host::facade::application_entry::WorthQueryApplicationMutationOutcome::Commit(
                    worth_query_host::facade::primary_graph::WorthQueryApplicationUncommitted::Denied(cause)
                ) = &outcome {
                    panic!("original Native effect refused before abandonment: {cause:?}");
                }
                mutation = Some(host_observation::receipts::observe(&outcome, &before.head));
            }
            Request::ContinueOriginal {
                key,
                value,
                pause_after_disposition,
            } => {
                disposition = Some(super::recovered_custody::continue_original(
                    home.as_ref().unwrap(),
                    key,
                    value,
                    pause_after_disposition.then(|| channel.clone()),
                    &execution,
                ));
            }
            Request::ReleaseOriginal {
                key,
                value,
                expect_continued,
            } => {
                disposition = Some(super::recovered_custody::release_original(
                    home.as_ref().unwrap(),
                    key,
                    value,
                    expect_continued,
                ));
            }
            Request::Step(index) => {
                let command = &script.commands()[index];
                assert_eq!(command.activation(), OperationActivation::Current);
                if matches!(command, HostCommand::Close) {
                    let result = home
                        .take()
                        .unwrap()
                        .close(&execution)
                        .unwrap_or_else(|refused| panic!("script close: {refused:?}"));
                    closed = result.is_closed();
                    result.reclaim();
                } else {
                    let home = home.as_ref().unwrap();
                    mutation = super::execution::mutation(home, command, &mut originals);
                    disposition = super::execution::disposition(home, command);
                    if let HostCommand::Checkpoint(kind) = command {
                        super::execution::checkpoint(home, *kind, &execution);
                    }
                }
            }
        }
        respond(Response {
            state: home
                .as_ref()
                .map(|h| host_observation::projection::observe(h.runtime(), &execution)),
            mutation,
            disposition,
            initialized: initialized.load(std::sync::atomic::Ordering::SeqCst),
            contacts: contacts.load(std::sync::atomic::Ordering::SeqCst),
            closed,
        });
        if closed {
            return;
        }
    }
}
