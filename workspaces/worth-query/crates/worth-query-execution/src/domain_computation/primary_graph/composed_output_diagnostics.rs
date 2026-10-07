//! Temporary, opt-in observation of composed output admission and postings.
//! Fixed descriptive copies grant no authority and retain no runtime custody.
use std::{
    any::TypeId,
    backtrace::Backtrace,
    fmt::{self, Write},
    panic::Location,
    sync::{Mutex, OnceLock},
};
use worth_relational::facade::mvcc::CompanionPreflightStop;

use super::output_lineage::RecordedSettlementIdentity;

const EVENTS: usize = 128;
const DUMPS: usize = 4;
const CLASS_DUMPS: [usize; 3] = [1, 1, 2];

#[derive(Clone, Copy)]
enum DumpClass {
    Bytes,
    RequiredWave,
    Missing,
}

static ENABLED: OnceLock<bool> = OnceLock::new();
static TRACE: Mutex<Trace> = Mutex::new(Trace {
    events: [const { None }; EVENTS],
    sequence: 0,
    dumps: 0,
    class_dumps: [0; 3],
});

pub(super) fn enabled() -> bool {
    *ENABLED.get_or_init(|| {
        std::env::var_os("WORTH_QUERY_COMPOSED_DIAGNOSTICS").is_some_and(|v| v == "1")
    })
}

/// All members are fixed-width descriptions; no authored text or Arc is held.
#[derive(Debug)]
pub(super) struct DemandDescription {
    pub(super) family: TypeId,
    pub(super) source: [u8; 32],
    pub(super) generation: u64,
    pub(super) lifecycle: u8,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct RetirementDescription {
    pub(super) interests: usize,
    pub(super) required_interests: usize,
    pub(super) framework_claims: usize,
    pub(super) prepared_claims: usize,
    pub(super) prerequisites: usize,
    pub(super) obligations: usize,
    pub(super) pending_cleanup: bool,
    pub(super) cached_ready: bool,
    pub(super) terminal: bool,
    pub(super) performed_source: bool,
    pub(super) held_successor: bool,
}

#[derive(Debug)]
pub(super) struct Event {
    kind: &'static str,
    identity: RecordedSettlementIdentity,
    demand: Option<DemandDescription>,
    site: &'static Location<'static>,
    retirement: Option<RetirementDescription>,
}

struct Trace {
    events: [Option<(u64, Event)>; EVENTS],
    sequence: u64,
    dumps: usize,
    class_dumps: [usize; 3],
}

/// Prepare before a posting transfers its existing inputs; record after success.
#[track_caller]
pub(super) fn event(
    kind: &'static str,
    identity: &RecordedSettlementIdentity,
    demand: Option<DemandDescription>,
) -> Option<Event> {
    let site = Location::caller();
    enabled().then(|| Event {
        kind,
        identity: identity.clone(),
        demand,
        site,
        retirement: None,
    })
}

#[track_caller]
pub(super) fn retirement_event(
    identity: &RecordedSettlementIdentity,
    demand: DemandDescription,
    retirement: RetirementDescription,
) -> Option<Event> {
    let mut event = event("retire-row-input", identity, Some(demand))?;
    event.retirement = Some(retirement);
    Some(event)
}

pub(super) fn record(event: Option<Event>) {
    let Some(event) = event else { return };
    let mut trace = TRACE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let sequence = trace.sequence;
    trace.events[(sequence % EVENTS as u64) as usize] = Some((sequence, event));
    trace.sequence = sequence.saturating_add(1);
}

#[track_caller]
pub(super) fn missing(
    identity: &RecordedSettlementIdentity,
    downstream: DemandDescription,
    publication: &'static Location<'static>,
) {
    record(event("missing-prerequisite", identity, Some(downstream)));
    dump(
        DumpClass::Missing,
        format_args!("missing exact prerequisite; publication={publication}"),
    );
}

/// Called outside the meter lock; all totals and the original stop are unchanged.
pub(super) fn preparation_denied(
    stop: &CompanionPreflightStop,
    current: u64,
    charge: u64,
    maximum: u64,
) {
    if !enabled() {
        return;
    }
    dump(DumpClass::Bytes, format_args!(
        "preparation stop={stop:?}; current={current}; charge={charge}; required={:?}; maximum={maximum}",
        current.checked_add(charge),
    ));
}

pub(super) fn required_stop(stop: &CompanionPreflightStop) {
    if enabled() {
        dump(
            DumpClass::RequiredWave,
            format_args!("required-wave original admission stop={stop:?}"),
        );
    }
}

fn dump(class: DumpClass, reason: fmt::Arguments<'_>) {
    if !enabled() {
        return;
    }
    let mut trace = TRACE
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let class = class as usize;
    if trace.dumps == DUMPS || trace.class_dumps[class] == CLASS_DUMPS[class] {
        return;
    }
    trace.dumps += 1;
    trace.class_dumps[class] += 1;
    eprintln!(
        "WQ-COMPOSED dump={} events={} overwritten={} {reason}",
        trace.dumps,
        trace.sequence,
        trace.sequence.saturating_sub(EVENTS as u64)
    );
    let start = trace.sequence.saturating_sub(EVENTS as u64);
    for sequence in start..trace.sequence {
        if let Some((recorded, event)) = &trace.events[(sequence % EVENTS as u64) as usize] {
            eprintln!(
                "WQ-COMPOSED event={recorded} kind={} identity={:?} site={}",
                event.kind, event.identity, event.site
            );
            if let Some(demand) = &event.demand {
                eprintln!(
                    "WQ-COMPOSED demand family={:?} source={:?} generation={} lifecycle={}",
                    demand.family, demand.source, demand.generation, demand.lifecycle
                );
            }
            if let Some(retirement) = &event.retirement {
                eprintln!("WQ-COMPOSED retirement {retirement:?}");
            }
        }
    }
    // Capture after refusal only. Formatting stops at the fixed byte/line bound;
    // no unbounded formatted String or per-charge successful output is produced.
    drop(trace);
    let mut buffer = TraceText {
        bytes: [0; 4096],
        used: 0,
        lines: 0,
    };
    let _ = write!(&mut buffer, "{}", Backtrace::force_capture());
    eprintln!(
        "WQ-COMPOSED backtrace (at most 24 lines/4096 bytes):\n{}",
        std::str::from_utf8(&buffer.bytes[..buffer.used]).unwrap_or("non-UTF8 trace")
    );
}

struct TraceText {
    bytes: [u8; 4096],
    used: usize,
    lines: usize,
}

impl Write for TraceText {
    fn write_str(&mut self, text: &str) -> fmt::Result {
        for character in text.chars() {
            if self.lines == 24 || self.used + character.len_utf8() > self.bytes.len() {
                return Err(fmt::Error);
            }
            let mut encoded = [0; 4];
            let bytes = character.encode_utf8(&mut encoded).as_bytes();
            self.bytes[self.used..self.used + bytes.len()].copy_from_slice(bytes);
            self.used += bytes.len();
            self.lines += usize::from(character == '\n');
        }
        Ok(())
    }
}
