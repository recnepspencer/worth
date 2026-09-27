//! Qualifies live resize against the 3.16.2 timing thresholds.
//!
//! Run the native host with `WORTH_UI_RESIZE_TRACE` naming a trace file, start
//! `capture` while the window is open, and drag a window edge for at least ten
//! seconds within the capture. Then `analyze` the host trace with the capture.
//!
//! `drive` does all of that with no one at the controls: it launches the host,
//! drags a window corner with real input while capturing, closes the window,
//! and grades the run, failing it if the host did not exit cleanly. Launch it
//! from the directory the host runs in.

use std::process::ExitCode;
use std::time::Duration;

mod analysis;
#[cfg(windows)]
mod capture;
mod coverage;
#[cfg(windows)]
mod drive;
mod gaps;
mod logs;
mod report;
mod stamp;
mod work;

const USAGE: &str = "usage:
  worth-ui-resize-trace-qualification capture <capture-log> [--seconds <n>] [--title <window title>]
  worth-ui-resize-trace-qualification analyze <host-trace> <capture-log>
  worth-ui-resize-trace-qualification drive <host-executable> <log-prefix> [--seconds <n>] [--title <window title>]";

const DEFAULT_TITLE: &str = "WORTH UI Platform Pulse";
const DEFAULT_SECONDS: u64 = 25;
/// A driven drag's default length: past the ten seconds a run needs.
const DRIVE_SECONDS: u64 = 12;

fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|error| format!("reading {path}: {error}"))
}

fn analyze(host: &str, capture: &str) -> Result<report::Verdict, String> {
    let host = logs::parse_host(&read(host)?)?;
    let capture = logs::parse_capture(&read(capture)?)?;
    let analysis = analysis::analyze(&host, &capture)?;
    let (text, verdict) = report::render(&analysis, &capture);
    print!("{text}");
    Ok(verdict)
}

fn exit_code(verdict: report::Verdict) -> ExitCode {
    match verdict {
        report::Verdict::Pass => ExitCode::SUCCESS,
        report::Verdict::Fail => ExitCode::from(1),
        report::Verdict::Inconclusive => ExitCode::from(3),
    }
}

/// Reads `--seconds` and `--title`, starting from `seconds` and the default
/// title.
fn options(options: &[String], mut seconds: u64) -> Result<(u64, String), String> {
    let mut title = DEFAULT_TITLE.to_owned();
    let mut rest = options.iter();
    while let Some(option) = rest.next() {
        let value = rest
            .next()
            .ok_or_else(|| format!("`{option}` needs a value\n{USAGE}"))?;
        match option.as_str() {
            "--seconds" => {
                seconds = value
                    .parse()
                    .map_err(|_| format!("`{value}` is not a number of seconds"))?;
            }
            "--title" => title.clone_from(value),
            _ => return Err(format!("unknown option `{option}`\n{USAGE}")),
        }
    }
    Ok((seconds, title))
}

#[cfg(windows)]
fn capture(out: &str, options: &[String]) -> Result<(), String> {
    let (seconds, title) = self::options(options, DEFAULT_SECONDS)?;
    println!(
        "capturing `{title}` for {seconds} s; drag a window edge for at least 10 s, then release"
    );
    capture::run(out.as_ref(), Duration::from_secs(seconds), &title)?;
    println!("wrote {out}");
    Ok(())
}

#[cfg(not(windows))]
fn capture(_: &str, _: &[String]) -> Result<(), String> {
    Err("capture reads the Windows performance counter and runs only on Windows".to_owned())
}

/// Drives one run and grades it. A host that did not exit cleanly fails the
/// run whatever its timing.
#[cfg(windows)]
fn drive(host: &str, prefix: &str, options: &[String]) -> Result<ExitCode, String> {
    let (seconds, title) = self::options(options, DRIVE_SECONDS)?;
    let duration = drive::whole_periods(Duration::from_secs(seconds));
    println!(
        "driving `{title}`: its corner is dragged for {} s with the mouse",
        duration.as_secs()
    );
    let drive = drive::run(host.as_ref(), prefix, duration, &title)?;
    let graded = match &drive.capture_error {
        Some(error) => Err(format!("the capture failed: {error}")),
        None => analyze(
            &drive.host_trace.to_string_lossy(),
            &drive.capture.to_string_lossy(),
        ),
    };
    match drive.exit {
        Some(0) => println!("host exited cleanly"),
        Some(code) => println!("host exited with code {code}"),
        None => println!("host did not close and was ended"),
    }
    if !drive.stderr.trim().is_empty() {
        println!("host stderr:\n{}", drive.stderr.trim_end());
    }
    match (drive.exit, graded) {
        (Some(0), graded) => graded.map(exit_code),
        (_, graded) => {
            if let Err(error) = graded {
                println!("{error}");
            }
            Ok(ExitCode::from(1))
        }
    }
}

#[cfg(not(windows))]
fn drive(_: &str, _: &str, _: &[String]) -> Result<ExitCode, String> {
    Err("drive sends Windows input and runs only on Windows".to_owned())
}

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match arguments.as_slice() {
        [command, out, options @ ..] if command == "capture" => {
            capture(out, options).map(|()| ExitCode::SUCCESS)
        }
        [command, host, capture] if command == "analyze" => analyze(host, capture).map(exit_code),
        [command, host, prefix, options @ ..] if command == "drive" => drive(host, prefix, options),
        _ => Err(USAGE.to_owned()),
    };
    outcome.unwrap_or_else(|error| {
        eprintln!("{error}");
        ExitCode::from(2)
    })
}
