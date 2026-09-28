//! The two logs a qualification run correlates: the native host's resize trace
//! and this tool's capture of the client origin. Both timestamp on the Windows
//! performance counter.

use crate::stamp::Reading;

pub const CAPTURE_HEADER: &str = "worth-ui-resize-capture 1";
const HOST_HEADER: &str = "worth-ui-resize-trace 1";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HostKind {
    Observed([u32; 2]),
    Consumed([u32; 2]),
    Submitted {
        frame: u64,
        extent: [u32; 2],
    },
    Accepted(u64),
    /// A render target allocated at an extent.
    Target([u32; 2]),
    /// A frame's text work, counted as [`crate::work::TEXT_WORK`] names.
    Text {
        frame: u64,
        work: [u64; 5],
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostEvent {
    pub counter: i64,
    pub kind: HostKind,
}

pub struct HostTrace {
    pub frequency: i64,
    pub events: Vec<HostEvent>,
    /// The most of each native resource the host retained at once, written
    /// when it closed.
    pub peaks: Vec<(String, u64)>,
    /// The graphics adapter the host presented with.
    pub adapter: Option<String>,
    /// The least client extent the window allows, if the host traced one
    /// that held for the whole trace. A scale change that moved it leaves
    /// none, since the trace cannot say which applied to which movement.
    pub minimum: Option<[u32; 2]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sample {
    /// Counter read before the sample.
    pub before: i64,
    /// Counter read after the pixels were captured.
    pub after: i64,
    /// Client extent in physical pixels.
    pub client: [u32; 2],
    /// Whether the primary button was held when the sample began.
    pub pressed: bool,
    /// Cursor position in physical screen pixels when the sample began.
    pub cursor: [i32; 2],
    pub reading: Reading,
}

/// Which part of the window a capture's drag held, when the capture knows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Grip {
    /// Not known: a person dragged whichever edge or corner they chose.
    #[default]
    Unknown,
    /// The bottom-right corner, which a driven drag holds.
    BottomRight,
}

pub struct CaptureLog {
    pub frequency: i64,
    pub refresh_hz: u32,
    pub dpi: u32,
    /// The Windows build and revision.
    pub windows_build: String,
    pub grip: Grip,
    pub samples: Vec<Sample>,
    /// Whether the capture wrote its closing line.
    pub complete: bool,
}

type Parsed<T> = Result<T, String>;

fn field<T: std::str::FromStr>(words: &[&str], index: usize, line: usize) -> Parsed<T> {
    words
        .get(index)
        .and_then(|word| word.parse().ok())
        .ok_or_else(|| format!("line {line}: field {index} is missing or malformed"))
}

fn header_words(line: Option<&str>, header: &str) -> Parsed<Vec<String>> {
    let line = line.ok_or("the log is empty")?;
    let rest = line
        .strip_prefix(header)
        .ok_or_else(|| format!("the log does not start with `{header}`"))?;
    Ok(rest.split_whitespace().map(str::to_owned).collect())
}

fn named<T: std::str::FromStr>(words: &[String], name: &str) -> Parsed<T> {
    words
        .iter()
        .position(|word| word == name)
        .and_then(|index| words.get(index + 1))
        .and_then(|value| value.parse().ok())
        .ok_or_else(|| format!("the header has no `{name}`"))
}

pub fn parse_host(text: &str) -> Parsed<HostTrace> {
    let mut lines = text.lines();
    let frequency = named(&header_words(lines.next(), HOST_HEADER)?, "frequency")?;
    let (mut events, mut peaks, mut adapter) = (Vec::new(), Vec::new(), None);
    let (mut minimum, mut minimum_moved) = (None, false);
    for (index, text) in lines.enumerate() {
        let line = index + 2;
        let words: Vec<&str> = text.split_whitespace().collect();
        let extent = |at: usize| -> Parsed<[u32; 2]> {
            Ok([field(&words, at, line)?, field(&words, at + 1, line)?])
        };
        let kind = match words.get(1).copied() {
            Some("observed") => HostKind::Observed(extent(2)?),
            Some("consumed") => HostKind::Consumed(extent(2)?),
            Some("submitted") => HostKind::Submitted {
                frame: field(&words, 2, line)?,
                extent: extent(3)?,
            },
            Some("accepted") => HostKind::Accepted(field(&words, 2, line)?),
            Some("target") => HostKind::Target(extent(2)?),
            Some("text") => {
                let mut work = [0; 5];
                for (at, count) in work.iter_mut().enumerate() {
                    *count = field(&words, at + 3, line)?;
                }
                HostKind::Text {
                    frame: field(&words, 2, line)?,
                    work,
                }
            }
            Some("adapter") => {
                adapter = Some(words[2..].join(" "));
                continue;
            }
            Some("minimum") => {
                let least = extent(2)?;
                minimum_moved |= minimum.is_some_and(|earlier| earlier != least);
                minimum = Some(least);
                continue;
            }
            Some("peak") => {
                let name = words
                    .get(2)
                    .ok_or_else(|| format!("line {line}: a peak names no resource"))?;
                peaks.push(((*name).to_owned(), field(&words, 3, line)?));
                continue;
            }
            _ => return Err(format!("line {line}: unknown host event `{text}`")),
        };
        events.push(HostEvent {
            counter: field(&words, 0, line)?,
            kind,
        });
    }
    Ok(HostTrace {
        frequency,
        events,
        peaks,
        adapter,
        minimum: minimum.filter(|_| !minimum_moved),
    })
}

/// The capture log's first line. The grip is written only when known.
pub fn capture_header(
    frequency: i64,
    refresh_hz: u32,
    dpi: u32,
    windows_build: &str,
    grip: Grip,
) -> String {
    let grip = match grip {
        Grip::Unknown => "",
        Grip::BottomRight => " grip bottom-right",
    };
    format!(
        "{CAPTURE_HEADER} frequency {frequency} refresh_hz {refresh_hz} dpi {dpi} windows_build {windows_build}{grip}"
    )
}

fn grip(header: &[String]) -> Parsed<Grip> {
    let Some(index) = header.iter().position(|word| word == "grip") else {
        return Ok(Grip::Unknown);
    };
    match header.get(index + 1).map(String::as_str) {
        Some("bottom-right") => Ok(Grip::BottomRight),
        other => Err(format!(
            "the header's grip `{}` is not one this tool knows",
            other.unwrap_or_default()
        )),
    }
}

/// One capture log line.
pub fn capture_line(sample: &Sample) -> String {
    let reading = match sample.reading {
        Reading::Frame(frame) => format!("frame {frame}"),
        Reading::Stretched([x, y]) => format!("stretched {x} {y}"),
        Reading::Unreadable => "unreadable".to_owned(),
    };
    format!(
        "{} {} {} {} {} {} {} {reading}",
        sample.before,
        sample.after,
        sample.client[0],
        sample.client[1],
        u8::from(sample.pressed),
        sample.cursor[0],
        sample.cursor[1],
    )
}

pub fn parse_capture(text: &str) -> Parsed<CaptureLog> {
    let mut lines = text.lines();
    let header = header_words(lines.next(), CAPTURE_HEADER)?;
    let mut log = CaptureLog {
        frequency: named(&header, "frequency")?,
        refresh_hz: named(&header, "refresh_hz")?,
        dpi: named(&header, "dpi")?,
        windows_build: named(&header, "windows_build")?,
        grip: grip(&header)?,
        samples: Vec::new(),
        complete: false,
    };
    for (index, text) in lines.enumerate() {
        let line = index + 2;
        if log.complete {
            return Err(format!("line {line}: text after the closing line"));
        }
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.first() == Some(&"end") {
            let count: usize = field(&words, 1, line)?;
            if count != log.samples.len() {
                return Err(format!(
                    "line {line}: the capture closed with {count} samples but lists {}",
                    log.samples.len()
                ));
            }
            log.complete = true;
            continue;
        }
        let reading = match words.get(7).copied() {
            Some("frame") => Reading::Frame(field(&words, 8, line)?),
            Some("stretched") => {
                Reading::Stretched([field(&words, 8, line)?, field(&words, 9, line)?])
            }
            Some("unreadable") => Reading::Unreadable,
            _ => return Err(format!("line {line}: unknown reading `{text}`")),
        };
        log.samples.push(Sample {
            before: field(&words, 0, line)?,
            after: field(&words, 1, line)?,
            client: [field(&words, 2, line)?, field(&words, 3, line)?],
            pressed: field::<u8>(&words, 4, line)? == 1,
            cursor: [field(&words, 5, line)?, field(&words, 6, line)?],
            reading,
        });
    }
    Ok(log)
}

#[cfg(test)]
#[path = "tests/logs.rs"]
mod tests;
