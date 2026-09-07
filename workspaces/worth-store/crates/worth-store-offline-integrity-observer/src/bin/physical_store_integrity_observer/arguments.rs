use std::ffi::OsString;
use std::path::PathBuf;

use worth_store_offline_integrity_observer::{
    OfflineIntegrityObservationLimits, OfflineIntegrityReportDestination,
};

pub(super) const HELP: &str = "\
Independent read-only C.9 physical-integrity observer\n\n\
Usage:\n  physical_store_integrity_observer observe \\\n    --store-root <closed-or-isolated-store-root> \\\n    --report <path-outside-store-root|-> \\\n    --max-entries <n> --max-bytes <n> --max-open-files <n> \\\n    --max-depth <n> --max-symlinks <n> --max-elapsed-ms <n> \\\n    --max-report-bytes <n>\n\n\
The observer inspects every current family through independent bounded readers.\n\
Compare: physical_store_integrity_observer compare --runtime-observation <path> --offline-observation <path> --report <external-path|->\n\
Optional --run and --scenario values bind a caller-declared observation context.\n\
It never repairs, recovers, quarantines, deletes, or accepts damaged bytes.\n";

pub(super) struct ObserveArguments {
    pub(super) store_root: PathBuf,
    pub(super) report_destination: OfflineIntegrityReportDestination,
    pub(super) limits: OfflineIntegrityObservationLimits,
    pub(super) run_identity: String,
    pub(super) scenario_identity: String,
}

pub(super) enum ArgumentOutcome {
    Help(&'static str),
    Denied(String),
}

pub(super) enum OperationArguments {
    Observe(ObserveArguments),
    Compare(super::comparison::CompareArguments),
}

pub(super) fn parse(
    arguments: impl IntoIterator<Item = OsString>,
) -> Result<OperationArguments, ArgumentOutcome> {
    let mut arguments = arguments.into_iter();
    let Some(operation) = arguments.next() else {
        return Err(ArgumentOutcome::Help(HELP));
    };
    if operation == "--help" || operation == "-h" {
        return Err(ArgumentOutcome::Help(HELP));
    }
    if operation == "compare" {
        return super::comparison::parse(arguments).map(OperationArguments::Compare);
    }
    if operation != "observe" {
        return Err(ArgumentOutcome::Denied(
            "expected observe or compare".into(),
        ));
    }
    let mut values = ArgumentValues::default();
    while let Some(flag) = arguments.next() {
        if flag == "--help" || flag == "-h" {
            return Err(ArgumentOutcome::Help(HELP));
        }
        let value = arguments.next().ok_or_else(|| {
            ArgumentOutcome::Denied(format!("{} requires a value", flag.to_string_lossy()))
        })?;
        values.assign(&flag.to_string_lossy(), value)?;
    }
    values.finish().map(OperationArguments::Observe)
}

#[derive(Default)]
struct ArgumentValues {
    store_root: Option<PathBuf>,
    report: Option<OsString>,
    maximum_entries: Option<u64>,
    maximum_bytes: Option<u64>,
    maximum_open_files: Option<u32>,
    maximum_depth: Option<u32>,
    maximum_symlinks: Option<u64>,
    maximum_elapsed_milliseconds: Option<u64>,
    maximum_report_bytes: Option<u64>,
    run_identity: Option<String>,
    scenario_identity: Option<String>,
}

impl ArgumentValues {
    fn assign(&mut self, flag: &str, value: OsString) -> Result<(), ArgumentOutcome> {
        match flag {
            "--store-root" => set_once(&mut self.store_root, PathBuf::from(value), flag),
            "--report" => set_once(&mut self.report, value, flag),
            "--max-entries" => parse_once(&mut self.maximum_entries, value, flag),
            "--max-bytes" => parse_once(&mut self.maximum_bytes, value, flag),
            "--max-open-files" => parse_once(&mut self.maximum_open_files, value, flag),
            "--max-depth" => parse_once(&mut self.maximum_depth, value, flag),
            "--max-symlinks" => parse_once(&mut self.maximum_symlinks, value, flag),
            "--max-elapsed-ms" => parse_once(&mut self.maximum_elapsed_milliseconds, value, flag),
            "--max-report-bytes" => parse_once(&mut self.maximum_report_bytes, value, flag),
            "--run" => parse_identity_once(&mut self.run_identity, value, flag),
            "--scenario" => parse_identity_once(&mut self.scenario_identity, value, flag),
            _ => Err(ArgumentOutcome::Denied(format!("unknown argument {flag}"))),
        }
    }

    fn finish(self) -> Result<ObserveArguments, ArgumentOutcome> {
        let store_root = required(self.store_root, "--store-root")?;
        let report = required(self.report, "--report")?;
        let report_destination = if report == "-" {
            OfflineIntegrityReportDestination::standard_output()
        } else {
            OfflineIntegrityReportDestination::file(PathBuf::from(report)).map_err(|denial| {
                ArgumentOutcome::Denied(format!("invalid --report: {denial:?}"))
            })?
        };
        let limits = OfflineIntegrityObservationLimits::new(
            required(self.maximum_entries, "--max-entries")?,
            required(self.maximum_bytes, "--max-bytes")?,
            required(self.maximum_open_files, "--max-open-files")?,
            required(self.maximum_depth, "--max-depth")?,
            required(self.maximum_symlinks, "--max-symlinks")?,
            required(self.maximum_elapsed_milliseconds, "--max-elapsed-ms")?,
            required(self.maximum_report_bytes, "--max-report-bytes")?,
        )
        .map_err(|denial| {
            ArgumentOutcome::Denied(format!("invalid observation limit: {denial:?}"))
        })?;
        Ok(ObserveArguments {
            store_root,
            report_destination,
            limits,
            run_identity: self
                .run_identity
                .unwrap_or_else(|| format!("offline-{}", std::process::id())),
            scenario_identity: self
                .scenario_identity
                .unwrap_or_else(|| "operator-observe".into()),
        })
    }
}

fn required<T>(value: Option<T>, flag: &str) -> Result<T, ArgumentOutcome> {
    value.ok_or_else(|| ArgumentOutcome::Denied(format!("missing required {flag}")))
}

fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), ArgumentOutcome> {
    if slot.replace(value).is_some() {
        Err(ArgumentOutcome::Denied(format!("duplicate {flag}")))
    } else {
        Ok(())
    }
}

fn parse_identity_once(
    slot: &mut Option<String>,
    value: OsString,
    flag: &str,
) -> Result<(), ArgumentOutcome> {
    let value = value
        .into_string()
        .map_err(|_| ArgumentOutcome::Denied(format!("{flag} requires UTF-8")))?;
    set_once(slot, value, flag)
}

fn parse_once<T: std::str::FromStr>(
    slot: &mut Option<T>,
    value: OsString,
    flag: &str,
) -> Result<(), ArgumentOutcome> {
    let parsed = value
        .to_str()
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| ArgumentOutcome::Denied(format!("{flag} requires an unsigned integer")))?;
    set_once(slot, parsed, flag)
}
