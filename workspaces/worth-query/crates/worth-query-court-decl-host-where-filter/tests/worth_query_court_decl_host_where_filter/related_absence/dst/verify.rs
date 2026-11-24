//! RNG, coverage counters, and per-step oracle comparison for the seeded
//! differential query DST.
use super::super::queries::*;
use super::super::world::Host;
use super::reference::GraphSnapshot;
use super::trace::Trace;
use worth_query_host::facade::application_entry::{
    WorthQueryApplicationReadObservation, WorthQueryApplicationRequestExt,
};
use worth_query_testkit_document_retention::facade::document_retention_model::operator_identity::{
    authenticate_operator, request_scope,
};

pub(super) struct Rng(u64);
impl Rng {
    pub fn new(seed: u64) -> Self {
        Self(seed ^ 0x9e37_79b9_7f4a_7c15)
    }
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    pub fn pick(&mut self, bound: usize) -> usize {
        (self.next_u64() as usize) % bound
    }
}

#[derive(Default, Debug)]
pub(super) struct Coverage {
    pub toggles: usize,
    pub edge_adds: usize,
    pub edge_removes: usize,
    pub note_edits: usize,
    pub pinned_checks: usize,
    pub self_loops: usize,
    pub cycles_or_shared: usize,
}

impl Coverage {
    pub fn add(&mut self, other: Self) {
        self.toggles += other.toggles;
        self.edge_adds += other.edge_adds;
        self.edge_removes += other.edge_removes;
        self.note_edits += other.note_edits;
        self.pinned_checks += other.pinned_checks;
        self.self_loops += other.self_loops;
        self.cycles_or_shared += other.cycles_or_shared;
    }
}

macro_rules! run_keys {
    ($host:expr, $intent:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($host.installed_schema(), &scope);
        $host
            .runtime()
            .request(&principal, &scope)
            .query($intent)
            .execute()
            .expect("differential DST read succeeds")
            .rows()
            .iter()
            .map(|r| r.key)
            .collect::<Vec<u64>>()
    }};
    ($host:expr, $intent:expr, $basis:expr) => {{
        let scope = request_scope();
        let principal = authenticate_operator($host.installed_schema(), &scope);
        $host
            .runtime()
            .request(&principal, &scope)
            .at($basis)
            .query($intent)
            .execute()
            .expect("differential DST pinned read succeeds")
            .rows()
            .iter()
            .map(|r| r.key)
            .collect::<Vec<u64>>()
    }};
}

#[allow(clippy::too_many_arguments)]
pub(super) fn verify_step(
    seed: u64,
    step: usize,
    host: &Host,
    oracle: &GraphSnapshot,
    history: &[(WorthQueryApplicationReadObservation, GraphSnapshot)],
    rng: &mut Rng,
    trace: &mut Trace,
    cov: &mut Coverage,
    tracing: bool,
) -> Result<(), String> {
    let check = |label: &str, got: Vec<u64>, exp: Vec<u64>| -> Result<(), String> {
        if got != exp {
            Err(format!(
                "seed {seed} step {step} {label}: got {got:?}, oracle {exp:?}"
            ))
        } else {
            Ok(())
        }
    };
    let projects: &[u64] = if tracing { &[1] } else { &[1, 2] };
    for &project in projects {
        let ready = run_keys!(host, ReadyIntent(ProjectScope { project }));
        check("ready_tasks", ready.clone(), oracle.ready_tasks(project))?;
        if project == 1 {
            trace.observe("Absent", "Forward", true, &ready);
        }

        let blocked = run_keys!(host, BlockedIntent(ProjectScope { project }));
        check(
            "blocked_tasks",
            blocked.clone(),
            oracle.blocked_tasks(project),
        )?;
        if project == 1 {
            trace.observe("Exists", "Forward", true, &blocked);
        }

        let blocking = run_keys!(host, BlockingIntent(ProjectScope { project }));
        check(
            "blocking_tasks",
            blocking.clone(),
            oracle.blocking_tasks(project),
        )?;
        if project == 1 {
            trace.observe("Exists", "Reverse", true, &blocking);
        }

        let unlinked = run_keys!(host, UnlinkedIntent(ProjectScope { project }));
        check(
            "unlinked_tasks",
            unlinked.clone(),
            oracle.unlinked_tasks(project),
        )?;
        if project == 1 {
            trace.observe("Absent", "Forward", false, &unlinked);
        }

        let max_due = [None, Some(100), Some(200)][rng.pick(3)];
        let text = [None, Some("flange"), Some("alpha")][rng.pick(3)];
        let min_rank = [None, Some(5), Some(8)][rng.pick(3)];
        let filtered = run_keys!(
            host,
            ReadyFilteredIntent {
                project,
                max_due_days: max_due,
                text: text.map(str::to_owned),
                blocker_min_rank: min_rank,
            }
        );
        check(
            "ready_filtered",
            filtered,
            oracle.ready_filtered(project, max_due, text, min_rank),
        )?;
    }

    let sample_task = (rng.pick(6) as u64) + 1;
    let expected_rank = if rng.pick(2) == 0 {
        oracle.tasks[&sample_task].rank
    } else {
        oracle.tasks[&sample_task].rank + 1
    };
    let exact = run_keys!(
        host,
        ExactReadyIntent {
            task: sample_task,
            expected_rank,
        }
    );
    check(
        "exact_ready_task",
        exact,
        oracle.exact_ready_task(sample_task, expected_rank),
    )?;

    let pin_idx = rng.pick(history.len());
    let (ref basis, ref snap) = history[pin_idx];
    let pinned_ready = run_keys!(host, ReadyIntent(ProjectScope { project: 1 }), basis);
    check("pinned_ready", pinned_ready.clone(), snap.ready_tasks(1))?;
    trace.pinned_read(pin_idx + 1, "Absent", "Forward", true, &pinned_ready);
    cov.pinned_checks += 1;
    Ok(())
}
