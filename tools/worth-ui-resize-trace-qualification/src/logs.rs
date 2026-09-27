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
    Submitted { frame: u64, extent: [u32; 2] },
    Accepted(u64),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct HostEvent {
    pub counter: i64,
    pub kind: HostKind,
}

pub struct HostTrace {
    pub frequency: i64,
    pub events: Vec<HostEvent>,
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

pub struct CaptureLog {
    pub frequency: i64,
    pub refresh_hz: u32,
    pub dpi: u32,
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
    let mut events = Vec::new();
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
            _ => return Err(format!("line {line}: unknown host event `{text}`")),
        };
        events.push(HostEvent {
            counter: field(&words, 0, line)?,
            kind,
        });
    }
    Ok(HostTrace { frequency, events })
}

/// The capture log's first line.
pub fn capture_header(frequency: i64, refresh_hz: u32, dpi: u32) -> String {
    format!("{CAPTURE_HEADER} frequency {frequency} refresh_hz {refresh_hz} dpi {dpi}")
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
mod tests {
    use super::*;

    #[test]
    fn capture_lines_read_back_as_written() {
        let samples = [
            Sample {
                before: 10,
                after: 12,
                client: [1536, 1024],
                pressed: true,
                cursor: [1540, 900],
                reading: Reading::Frame(513),
            },
            Sample {
                before: 13,
                after: 15,
                client: [1500, 1000],
                pressed: false,
                cursor: [-3, 0],
                reading: Reading::Stretched([9, 8]),
            },
            Sample {
                before: 16,
                after: 18,
                client: [1500, 1000],
                pressed: false,
                cursor: [0, 0],
                reading: Reading::Unreadable,
            },
        ];
        let mut text = capture_header(10_000_000, 60, 96);
        for sample in &samples {
            text.push('\n');
            text.push_str(&capture_line(sample));
        }
        text.push_str("\nend 3\n");
        let log = parse_capture(&text).expect("the capture parses");
        assert_eq!(log.samples, samples);
        assert_eq!(
            (log.frequency, log.refresh_hz, log.dpi),
            (10_000_000, 60, 96)
        );
        assert!(log.complete);
    }

    #[test]
    fn a_capture_without_its_closing_line_is_incomplete() {
        let text = format!(
            "{}\n1 2 800 600 1 5 5 unreadable\n",
            capture_header(1, 60, 96)
        );
        assert!(!parse_capture(&text).expect("the capture parses").complete);
    }

    #[test]
    fn host_events_parse_by_kind() {
        let text = "worth-ui-resize-trace 1 frequency 10000000\n\
                    5 observed 800 600\n6 consumed 800 600\n\
                    7 submitted 42 800 600\n8 accepted 42\n";
        let trace = parse_host(text).expect("the trace parses");
        assert_eq!(trace.frequency, 10_000_000);
        let kinds: Vec<HostKind> = trace.events.iter().map(|event| event.kind).collect();
        assert_eq!(
            kinds,
            [
                HostKind::Observed([800, 600]),
                HostKind::Consumed([800, 600]),
                HostKind::Submitted {
                    frame: 42,
                    extent: [800, 600]
                },
                HostKind::Accepted(42),
            ]
        );
        assert!(parse_host("5 observed 1 1").is_err());
        assert!(parse_host("worth-ui-resize-trace 1 frequency 1\n5 moved 1 1").is_err());
    }
}
