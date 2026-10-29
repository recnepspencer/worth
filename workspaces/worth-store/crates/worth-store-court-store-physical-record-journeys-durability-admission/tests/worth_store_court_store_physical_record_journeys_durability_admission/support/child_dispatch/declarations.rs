pub(crate) use {std::collections::BTreeMap};
pub(crate) use {std::path::Path};

pub(crate) const CHILD_TEST: &str = "child_dispatch::journey_child_role";
pub(crate) const REPORT_PREFIX: &str = "C4_REPORT ";

pub(crate) struct ChildReport {
    pub(crate) fields: BTreeMap<String, String>,
}

impl ChildReport {
    pub(crate) fn value(&self, name: &str) -> &str {
        self.fields
            .get(name)
            .unwrap_or_else(|| panic!("child report omitted {name}"))
    }

    pub(crate) fn number(&self, name: &str) -> u64 {
        self.value(name)
            .parse()
            .unwrap_or_else(|_| panic!("child report field {name} is not numeric"))
    }
}

pub(crate) fn run_role(role: &str, root: &Path, environment: &[(&str, &str)]) -> ChildReport {
    let mut command = std::process::Command::new(std::env::current_exe().unwrap());
    command
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env("WORTH_STORE_C4_CHILD_ROLE", role)
        .env("WORTH_STORE_C4_CHILD_ROOT", root);
    for (name, value) in environment {
        command.env(name, value);
    }
    let output = command.output().expect("fresh journey process must start");
    assert!(
        output.status.success(),
        "C.4 child role {role} failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    parse_report(&output.stdout)
}

pub(crate) fn emit(fields: &[(&str, String)]) {
    let encoded = fields
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join(";");
    println!("{REPORT_PREFIX}{encoded}");
}

pub(crate) fn parse_report(stdout: &[u8]) -> ChildReport {
    let stdout = String::from_utf8(stdout.to_vec()).expect("child report must be UTF-8");
    let line = stdout
        .lines()
        .find_map(|line| line.strip_prefix(REPORT_PREFIX))
        .unwrap_or_else(|| panic!("child emitted no C.4 report:\n{stdout}"));
    let mut fields = std::collections::BTreeMap::new();
    for field in line.split(';') {
        let (name, value) = field
            .split_once('=')
            .unwrap_or_else(|| panic!("malformed child report field: {field}"));
        assert!(fields.insert(name.to_owned(), value.to_owned()).is_none());
    }
    ChildReport { fields }
}