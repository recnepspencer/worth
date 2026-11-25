//! #254 PR B: the wave keeps base runnable order and outcomes, and its
//! charged work grows with the placements, not with turns times placements.
//!
//! Base order is a recorded golden: every scenario's wave events (each ranked
//! batch with id, each prepare with occurrence identity and outcome, each break
//! and quiescent reclosure), the edit outcome, and the outputs
//! it reads. Goldens are recorded at base with `WORTH_RECORD_WAVE_ORDER=1`.
use super::super::super::synchronous_edit::envelope_fixture as fixture;
use super::*;
use worth_query_host::facade::{
    application_entry::WorthQueryEditEnvelopeOutcome as Outcome,
    primary_graph::WorthQueryApplicationUncommitted as Uncommitted,
    runtime::ExecutionAllocationPolicy,
};

/// One model edit through `$tree` on `$key`: its outcome, the wave events it
/// recorded and the units the wave charged.
macro_rules! edit_stage {
    ($host:expr, $tree:ty, $idempotency:expr, $key:expr, $value:expr) => {{
        let observation = WorthQueryExecutionObservation::install_on_this_thread();
        let scope = request_scope();
        let principal = authenticate_operator($host.installed_schema(), &scope);
        let request = $host.request(&principal, &scope);
        let mut edit = (request.edit_envelope())
            .idempotency($idempotency.to_owned())
            .runtime_stages($host)
            .ordered_authored()
            .unwrap();
        edit.push_stage::<$tree, _>(fixture::model::Intent {
            key: $key.into(),
            value: $value,
            key_derived_target: false,
        })
        .unwrap();
        let outcome = edit
            .execute_performed(ExecutionAllocationPolicy::SystemAllocation)
            .unwrap();
        $crate::certification::document_history::owned_outputs::family::captured_input::wave_order::summarize_outcome(
            outcome,
            observation,
        )
    }};
}

mod pr_c_cost;
mod proptest_diff;
mod r3_shapes;
mod sibling_reopen;
mod work_bounds;

const GOLDEN: &str = "tests/worth_query_court_decl_host/certification/document_history/owned_outputs/family/captured_input/wave_order.golden";

pub(super) fn install<P>(
    configuration: world::Configuration,
) -> WorthQueryProgramApplicationRuntime<Schema, P>
where
    P: ApplicationProgramDefinition<Schema, Contributions = (Contribution,)>,
    P::Outputs: WorthQueryApplicationProgramRoots<Schema>,
{
    world::install_configured(
        host_limits(
            worth_query_host::facade::admission::authenticated_principal::WorthQueryAdmissionClock::system(),
        ),
        1,
        None,
        world::Configuration {
            envelope_records: true,
            ..configuration
        },
    )
}
pub(super) fn summarize_outcome(
    outcome: Outcome,
    observation: WorthQueryExecutionObservation,
) -> (String, Vec<String>, u64) {
    let outcome = match outcome {
        Outcome::Committed(_) => "committed".to_owned(),
        Outcome::Uncommitted(Uncommitted::Denied(cause)) => {
            if let Some(failure) = cause.invariant_execution_failure() {
                format!("refused {}", stable_debug(&failure.kind()))
            } else if let Some(denial) = cause.coupled_producer_denial() {
                format!("refused {}", stable_debug(&denial.kind()))
            } else {
                let text = format!("{cause:?}");
                let kind = text
                    .split("kind: ")
                    .nth(1)
                    .and_then(|rest| rest.split([',', ' ', '}', '(']).next())
                    .unwrap_or("?");
                format!("refused {kind}")
            }
        }
        other => {
            let text = format!("{other:?}");
            let kind = text
                .split("kind: ")
                .nth(1)
                .and_then(|rest| rest.split([',', ' ', '}', '(']).next())
                .unwrap_or("?");
            format!("refused {kind}")
        }
    };
    (
        outcome,
        observation.wave_events_for_test(),
        observation.wave_charged_units_for_test(),
    )
}
/// A refusal's text without each record id's kind marker: the derived `Debug`
/// of `PhantomData<K>` prints the compiler's unstable type name for `K`.
fn stable_debug(value: &impl std::fmt::Debug) -> String {
    let text = format!("{value:?}");
    let (mut stable, mut rest) = (String::with_capacity(text.len()), text.as_str());
    while let Some(at) = rest.find(", _marker: PhantomData<") {
        stable += &rest[..at];
        rest = &rest[at..];
        rest = &rest[rest.find('>').map_or(rest.len(), |end| end + 1)..];
    }
    stable + rest
}
pub(super) fn summarize_denial(
    denial: &impl std::fmt::Debug,
    observation: WorthQueryExecutionObservation,
) -> (String, Vec<String>, u64) {
    let text = format!("{denial:?}");
    let subject = text
        .split("subject: ")
        .nth(1)
        .and_then(|rest| rest.split(", cause:").next())
        .unwrap_or("?");
    (
        format!("denied {subject}"),
        observation.wave_events_for_test(),
        observation.wave_charged_units_for_test(),
    )
}
pub(super) fn wave_units(observation: &WorthQueryExecutionObservation) -> u64 {
    observation.wave_charged_units_for_test()
}
/// The wave units and the retained reads qualification visited since the
/// last call, read before the observation is released.
pub(super) fn scheduling_measures(observation: &WorthQueryExecutionObservation) -> (u64, u64) {
    (
        observation.wave_charged_units_for_test(),
        observation.retained_reads_visited_for_test(),
    )
}
pub(super) fn producer_contacts<P: ApplicationProgramDefinition<Schema>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
) -> u64 {
    host.producer_contacts_on_this_thread_for_test()
}

pub(super) fn value<P: ApplicationProgramDefinition<Schema>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    key: &str,
) -> String {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    match request
        .query(query::first_output::Read(key.into()))
        .execute()
    {
        Ok(rows) => rows
            .rows()
            .first()
            .map_or("absent".into(), |row| row.value.to_string()),
        Err(_) => "absent".into(),
    }
}
pub(super) fn child_value<P: ApplicationProgramDefinition<Schema>>(
    host: &WorthQueryProgramApplicationRuntime<Schema, P>,
    key: &str,
) -> String {
    let scope = request_scope();
    let principal = authenticate_operator(host.installed_schema(), &scope);
    let request = host.request(&principal, &scope);
    match request
        .query(query::child_output::Read(key.into()))
        .execute()
    {
        Ok(rows) => rows
            .rows()
            .first()
            .map_or("absent".into(), |row| row.value.to_string()),
        Err(_) => "absent".into(),
    }
}
pub(super) struct Trace(pub(super) String);
impl Trace {
    pub(super) fn edit(&mut self, label: &str, (outcome, events, _): (String, Vec<String>, u64)) {
        self.0 += &format!("-- {label}: {outcome}\n");
        for event in events {
            self.0 += &format!("   {event}\n");
        }
    }
    pub(super) fn rows<P: ApplicationProgramDefinition<Schema>>(
        &mut self,
        host: &WorthQueryProgramApplicationRuntime<Schema, P>,
        keys: &[&str],
    ) {
        for key in keys {
            self.0 += &format!("   {key} = {}\n", value(host, key));
        }
    }
}

pub(super) fn compare_or_record(relative: &str, actual: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative);
    if std::env::var_os("WORTH_RECORD_WAVE_ORDER").is_some() {
        std::fs::write(&path, actual).unwrap();
        return;
    }
    let golden = std::fs::read_to_string(&path)
        .unwrap()
        .replace("\r\n", "\n");
    assert_eq!(actual, golden, "the wave left base order ({relative})");
}

fn chains(trace: &mut Trace, width: usize, length: usize) {
    type Host = WorthQueryProgramApplicationRuntime<Schema, chains::Program>;
    let host: Host = install(world::Configuration {
        captured_chains: Some((width, length)),
        ..Default::default()
    });
    trace.0 += &format!("== chains W={width} L={length}\n");
    trace.edit(
        "born",
        edit_stage!(&host, chains::Tree, "chains-born", "carriage:d", 4),
    );
    trace.edit(
        "healthy",
        edit_stage!(&host, chains::Tree, "chains-healthy", "carriage:d", 4),
    );
    let last = format!("first:{}", chains::key(width - 1, length - 1));
    trace.rows(&host, &["first:chain:0:0", &last]);
}
fn composed(trace: &mut Trace, waits: bool) {
    type Host = WorthQueryProgramApplicationRuntime<Schema, composed::Program>;
    let host: Host = install(world::Configuration {
        captured_composed: Some((0, waits)),
        ..Default::default()
    });
    trace.0 += &format!("== composed waits={waits}\n");
    trace.edit(
        "born",
        edit_stage!(&host, composed::Tree, "composed-born", "carriage:d", 4),
    );
    trace.edit(
        "changed",
        edit_stage!(&host, composed::Tree, "composed-changed", "carriage:d", 5),
    );
    trace.rows(
        &host,
        &[
            "first:carriage:d",
            "captured:capture:p",
            "captured:capture:r",
            "first:capture:t",
        ],
    );
}
fn wide(trace: &mut Trace, width: usize) {
    type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;
    let host: Host = install(world::Configuration {
        captured_composed: Some((width, false)),
        ..Default::default()
    });
    trace.0 += &format!("== wide W={width}\n");
    trace.edit(
        "born",
        edit_stage!(&host, supplier::SupplierTree, "wide-born", "carriage:d", 4),
    );
    trace.edit(
        "healthy",
        edit_stage!(
            &host,
            supplier::SupplierTree,
            "wide-healthy",
            "carriage:d",
            4
        ),
    );
    trace.edit(
        "changed",
        edit_stage!(
            &host,
            supplier::SupplierTree,
            "wide-changed",
            "carriage:d",
            5
        ),
    );
    trace.rows(&host, &["captured:capture:w0", "first:carriage:d"]);
}
fn supplied(trace: &mut Trace, undeclared: bool) {
    type Host = WorthQueryProgramApplicationRuntime<Schema, supplier::Program>;
    let host: Host = install(world::Configuration {
        interleaved_records: true,
        captured_inputs: true,
        captured_undeclared_anchor: undeclared,
        ..Default::default()
    });
    trace.0 += &format!("== supplied undeclared={undeclared}\n");
    trace.edit(
        "born",
        edit_stage!(
            &host,
            supplier::SupplierTree,
            "supplier-born",
            "carriage:d",
            4
        ),
    );
    trace.edit(
        "unchanged",
        edit_stage!(
            &host,
            supplier::SupplierTree,
            "supplier-unchanged",
            "carriage:d",
            4
        ),
    );
    trace.edit(
        "changed",
        edit_stage!(
            &host,
            supplier::SupplierTree,
            "supplier-changed",
            "carriage:d",
            5
        ),
    );
    trace.rows(&host, &["captured:capture:child", "first:carriage:d"]);
}

/// Each scenario (core + the three r3 shapes) runs in a fresh world; the whole
/// trace must equal the golden recorded at base.
#[test]
fn the_wave_keeps_base_order_and_outcomes() {
    let mut trace = Trace(String::new());
    chains(&mut trace, 2, 3);
    chains(&mut trace, 3, 1);
    composed(&mut trace, false);
    composed(&mut trace, true);
    wide(&mut trace, 2);
    supplied(&mut trace, false);
    supplied(&mut trace, true);
    r3_shapes::record_all(&mut trace);
    compare_or_record(GOLDEN, &trace.0);
}
