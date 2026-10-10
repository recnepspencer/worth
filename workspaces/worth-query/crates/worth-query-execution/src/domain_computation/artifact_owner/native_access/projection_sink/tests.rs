use stats_alloc::{Region, StatsAlloc, INSTRUMENTED_SYSTEM};
use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey, AspectValue,
    CanonicalBigInt, CanonicalDecimal, CanonicalRational, InternedString, ScalarAspectType,
};
use worth_query_installation::facade::WorthQueryArtifactNativeAlignment;

use super::WorthQueryArtifactProjectionSink;

#[global_allocator]
static TEST_ALLOCATOR: &StatsAlloc<std::alloc::System> = &INSTRUMENTED_SYSTEM;

#[test]
fn allocated_capacity_includes_nested_variable_width_buffers() {
    let capacities = [64, 96, 128, 160, 192];
    let mut sink = WorthQueryArtifactProjectionSink::new(
        fields(),
        1,
        WorthQueryArtifactNativeAlignment::new(1),
    )
    .unwrap();
    sink.push_row([
        AspectValue::String(InternedString::Raw(retained("text", capacities[0]))),
        AspectValue::Decimal(CanonicalDecimal::new(retained("12.5", capacities[1]))),
        AspectValue::BigInt(CanonicalBigInt::new(retained("123", capacities[2]))),
        AspectValue::Rational(
            CanonicalRational::new(
                CanonicalBigInt::new(retained("7", capacities[3])),
                CanonicalBigInt::new(retained("9", capacities[4])),
            )
            .unwrap(),
        ),
    ])
    .unwrap();

    let outer = sink
        .values
        .capacity()
        .saturating_add(sink.pending_row.capacity())
        .saturating_mul(std::mem::size_of::<AspectValue>());
    assert_eq!(
        sink.allocated_capacity_bytes().saturating_sub(outer),
        capacities.into_iter().sum::<usize>()
    );
}

#[test]
fn chunk_width_changes_independently_measured_live_allocation_peak() {
    let output = std::process::Command::new(std::env::current_exe().unwrap())
        .arg("isolated_resident_allocation_probe")
        .env("WORTH_QUERY_RUN_ALLOCATION_PROBE", "1")
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "allocation probe failed:\n{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
}

#[test]
fn isolated_resident_allocation_probe() {
    if std::env::var_os("WORTH_QUERY_RUN_ALLOCATION_PROBE").is_none() {
        return;
    }
    let narrow_peak = measured_sink_peak(1);
    let wide_peak = measured_sink_peak(64);

    assert!(
        narrow_peak < wide_peak,
        "narrow={narrow_peak} wide={wide_peak}"
    );
}

fn measured_sink_peak(max_rows: usize) -> usize {
    let fields = vec![AspectContract::scalar(
        AspectKey::new("resident-field").unwrap(),
        AspectIdentity(9_199),
        AspectContractRevision(1),
        ScalarAspectType::Int64,
    )];
    let region = Region::new(TEST_ALLOCATOR);
    let mut sink = WorthQueryArtifactProjectionSink::new(
        fields,
        max_rows,
        WorthQueryArtifactNativeAlignment::new(1),
    )
    .unwrap();
    for row in 0..max_rows {
        sink.push_row([AspectValue::Int64(row as i64)]).unwrap();
    }
    std::hint::black_box(sink.row_count());
    let stats = region.change();
    stats
        .bytes_allocated
        .saturating_sub(stats.bytes_deallocated)
}

fn fields() -> Vec<AspectContract> {
    [
        ScalarAspectType::String,
        ScalarAspectType::Decimal,
        ScalarAspectType::BigInt,
        ScalarAspectType::Rational,
    ]
    .into_iter()
    .enumerate()
    .map(|(index, scalar)| {
        AspectContract::scalar(
            AspectKey::new(format!("field-{index}")).unwrap(),
            AspectIdentity(9_150 + index as u64),
            AspectContractRevision(1),
            scalar,
        )
    })
    .collect()
}

fn retained(value: &str, capacity: usize) -> String {
    let mut retained = String::with_capacity(capacity);
    retained.push_str(value);
    assert_eq!(retained.capacity(), capacity);
    retained
}
