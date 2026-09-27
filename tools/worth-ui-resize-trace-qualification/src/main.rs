//! Qualifies live resize against the 3.16.2 timing thresholds.
//!
//! Run the native host with `WORTH_UI_RESIZE_TRACE` naming a trace file, start
//! `capture` while the window is open, and drag a window edge for at least ten
//! seconds within the capture. Then `analyze` the host trace with the capture.

use std::process::ExitCode;
use std::time::Duration;

mod analysis;
#[cfg(windows)]
mod capture;
mod coverage;
mod gaps;
mod logs;
mod report;
mod stamp;
mod work;

const USAGE: &str = "usage:
  worth-ui-resize-trace-qualification capture <capture-log> [--seconds <n>] [--title <window title>]
  worth-ui-resize-trace-qualification analyze <host-trace> <capture-log>";

const DEFAULT_TITLE: &str = "WORTH UI Platform Pulse";
const DEFAULT_SECONDS: u64 = 25;

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

#[cfg(windows)]
fn capture(out: &str, options: &[String]) -> Result<(), String> {
    let mut seconds = DEFAULT_SECONDS;
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

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match arguments.as_slice() {
        [command, out, options @ ..] if command == "capture" => {
            capture(out, options).map(|()| ExitCode::SUCCESS)
        }
        [command, host, capture] if command == "analyze" => {
            analyze(host, capture).map(|verdict| match verdict {
                report::Verdict::Pass => ExitCode::SUCCESS,
                report::Verdict::Fail => ExitCode::from(1),
                report::Verdict::Inconclusive => ExitCode::from(3),
            })
        }
        _ => Err(USAGE.to_owned()),
    };
    outcome.unwrap_or_else(|error| {
        eprintln!("{error}");
        ExitCode::from(2)
    })
}
