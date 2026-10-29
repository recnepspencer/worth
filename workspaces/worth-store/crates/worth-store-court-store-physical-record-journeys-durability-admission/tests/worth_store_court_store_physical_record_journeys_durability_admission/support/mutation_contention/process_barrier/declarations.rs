pub(crate) use {std::io::BufRead, std::io::Write};
pub(crate) use {std::path::Path};
pub(crate) use {std::process::Child, std::process::ChildStdin, std::process::Stdio};
pub(crate) use {std::sync::mpsc, std::sync::mpsc::Receiver};
pub(crate) use {std::time::Duration};

pub(crate) const CHILD_TEST: &str = "child_dispatch::journey_child_role";
pub(crate) const EVENT_PREFIX: &str = "C4_EVENT ";

pub(crate) struct Contender {
    pub(crate) child: Child,
    pub(crate) input: ChildStdin,
    pub(crate) events: Receiver<String>,
}

impl Contender {
    pub(crate) fn spawn(root: &Path, index: usize) -> Self {
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", CHILD_TEST, "--nocapture"])
            .env("WORTH_STORE_C4_CHILD_ROLE", "mutation-contender")
            .env("WORTH_STORE_C4_CHILD_ROOT", root)
            .env("WORTH_STORE_C4_CONTENDER_INDEX", index.to_string())
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::inherit())
            .spawn()
            .unwrap();
        let input = child.stdin.take().unwrap();
        let output = child.stdout.take().unwrap();
        let (sender, events) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            for line in std::io::BufReader::new(output).lines() {
                let line = line.unwrap();
                if let Some(event) = line.strip_prefix(EVENT_PREFIX) {
                    sender.send(event.to_owned()).unwrap();
                }
            }
        });
        Self {
            child,
            input,
            events,
        }
    }

    pub(crate) fn send(&mut self, command: u8) {
        self.input.write_all(&[command]).unwrap();
        self.input.flush().unwrap();
    }

    pub(crate) fn event(&self) -> String {
        self.events
            .recv_timeout(std::time::Duration::from_secs(30))
            .expect("contender failed to reach a process barrier")
    }

    pub(crate) fn kill(&mut self) {
        self.child.kill().unwrap();
    }

    pub(crate) fn wait(&mut self) -> std::process::ExitStatus {
        self.child.wait().unwrap()
    }
}