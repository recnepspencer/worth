//! Acknowledged observations are quiescent; the parent kills only after response.
use super::{opening,operations,protocol::{Request,Response}};
use std::{path::{Path,PathBuf},process::{Child,Command,Stdio},time::{Duration,Instant},sync::{Arc,atomic::{AtomicUsize,Ordering}}};
use worth_query_host::facade::{WorthQueryApplicationHomeLocation};
use worth_query_testkit_host::facade::host::temporal_conditional_operation::contribution_installation::{ApplicationBuildIdentity,ContactCounters};
const CHILD:&str="process::application::host_oracle::graceful::signal::session::signal_home_oracle_service_child";
pub(super) struct Session {
    child: Child,
    channel: PathBuf,
}
impl Session {
    pub(super) fn start(home: &Path, channel: &Path, build: &str) -> (Self, Response) {
        std::fs::create_dir_all(channel).unwrap();
        let child = Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD, "--nocapture"])
            .env("WORTH_SIGNAL_HOME_CHANNEL", channel)
            .env("WORTH_SIGNAL_HOME_PATH", home)
            .env("WORTH_SIGNAL_HOME_BUILD", build)
            .stdout(Stdio::from(
                std::fs::File::create(channel.join("stdout.log")).unwrap(),
            ))
            .stderr(Stdio::from(
                std::fs::File::create(channel.join("stderr.log")).unwrap(),
            ))
            .spawn()
            .unwrap();
        let mut session = Self {
            child,
            channel: channel.into(),
        };
        let response = session.receive();
        (session, response)
    }
    pub(super) fn send(&mut self, request: Request) -> Response {
        crate::killed_child::announce(
            &self.channel.join("request"),
            &serde_json::to_vec(&request).unwrap(),
        );
        self.receive()
    }
    fn receive(&mut self) -> Response {
        let deadline = Instant::now() + Duration::from_secs(240);
        let path = self.channel.join("response");
        loop {
            if path.is_file() {
                let bytes = std::fs::read(&path).unwrap();
                std::fs::remove_file(path).unwrap();
                return serde_json::from_slice(&bytes).unwrap();
            }
            let ended = self.child.try_wait().unwrap();
            assert!(
                ended.is_none() && Instant::now() < deadline,
                "Signal Home child {ended:?}:\n{}\n{}",
                std::fs::read_to_string(self.channel.join("stdout.log")).unwrap(),
                std::fs::read_to_string(self.channel.join("stderr.log")).unwrap()
            );
            std::thread::sleep(Duration::from_millis(10));
        }
    }
    pub(super) fn kill(&mut self) {
        if self.child.try_wait().unwrap().is_none() {
            self.child.kill().unwrap();
        }
        self.child.wait().unwrap();
    }
    pub(super) fn finish(&mut self) {
        assert!(self.child.wait().unwrap().success());
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        self.kill();
    }
}
#[test]
fn signal_home_oracle_service_child() {
    let Some(channel) = std::env::var_os("WORTH_SIGNAL_HOME_CHANNEL").map(PathBuf::from) else {
        return;
    };
    let path = PathBuf::from(std::env::var_os("WORTH_SIGNAL_HOME_PATH").unwrap());
    let build = ApplicationBuildIdentity::parse(&std::env::var("WORTH_SIGNAL_HOME_BUILD").unwrap())
        .unwrap();
    let request = super::super::super::opening::serial();
    let execution = worth_execution::ExecutionRequest::serial(&request);
    let initialized = Arc::new(AtomicUsize::new(0));
    let contacts = ContactCounters::default();
    let mut home = Some(opening::open(
        WorthQueryApplicationHomeLocation::Directory(path),
        build,
        initialized.clone(),
        contacts.clone(),
        &execution,
    ));
    let mut branches = operations::installed(home.as_ref().unwrap());
    let respond = |response: Response| {
        crate::killed_child::announce(
            &channel.join("response"),
            &serde_json::to_vec(&response).unwrap(),
        )
    };
    respond(Response {
        states: operations::observe(home.as_ref().unwrap(), &branches),
        frontiers: operations::frontiers(home.as_ref().unwrap(), &branches),
        initialized: initialized.load(Ordering::SeqCst),
        contacts: contacts.snapshot(),
        closed: false,
    });
    loop {
        let path = channel.join("request");
        if !path.is_file() {
            std::thread::sleep(Duration::from_millis(10));
            continue;
        }
        let command: Request = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
        std::fs::remove_file(path).unwrap();
        match command {
            Request::Prepare => {
                let home = home.as_ref().unwrap();
                branches = operations::branches(home);
                operations::mixed(home, branches[0], 1);
                operations::definition(home, branches[1]);
                operations::mixed(home, branches[2], 2);
            }
            Request::Continue => operations::mixed(home.as_ref().unwrap(), branches[1], 3),
            Request::Close => {
                let closed = home
                    .take()
                    .unwrap()
                    .close(&execution)
                    .unwrap_or_else(|cause| panic!("authored Home close: {cause:?}"));
                assert!(closed.is_closed());
                closed.reclaim();
                respond(Response {
                    states: Vec::new(),
                    frontiers: Vec::new(),
                    initialized: initialized.load(Ordering::SeqCst),
                    contacts: contacts.snapshot(),
                    closed: true,
                });
                return;
            }
        }
        respond(Response {
            states: operations::observe(home.as_ref().unwrap(), &branches),
            frontiers: operations::frontiers(home.as_ref().unwrap(), &branches),
            initialized: initialized.load(Ordering::SeqCst),
            contacts: contacts.snapshot(),
            closed: false,
        });
    }
}
