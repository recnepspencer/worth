use worth_store_physical_format::{PhysicalRecordFormatDeclaration, PhysicalRewriteRedo};

use super::{admit_span, SpanAdmissionDenial};

fn format() -> PhysicalRecordFormatDeclaration {
    PhysicalRecordFormatDeclaration::builder().admit().unwrap()
}

fn span(source_offset: u64, destination_offset: u64, pages: u32) -> PhysicalRewriteRedo {
    let page_bytes = format().page_size().bytes();
    PhysicalRewriteRedo::new(
        [1; 32],
        [2; 32],
        7,
        3,
        source_offset,
        pages * page_bytes,
        [4; 32],
        4,
        destination_offset,
        0,
        [5; 32],
        1,
        2,
        8,
    )
    .unwrap()
}

#[test]
fn a_span_rewritten_into_a_compact_generation_admits_its_page_count() {
    let page_bytes = u64::from(format().page_size().bytes());
    assert_eq!(admit_span(span(2 * page_bytes, 0, 4), format()), Ok(4));
}

#[test]
fn a_destination_not_at_frame_zero_is_denied_before_projection() {
    let page_bytes = u64::from(format().page_size().bytes());
    assert_eq!(
        admit_span(span(2 * page_bytes, 2 * page_bytes, 4), format()),
        Err(SpanAdmissionDenial::OffsetMismatch)
    );
}

#[test]
fn a_span_over_the_rewrite_limit_is_denied() {
    let page_bytes = format().page_size().bytes();
    let pages = (256 * 1024) / page_bytes + 1;
    assert_eq!(
        admit_span(span(0, 0, pages), format()),
        Err(SpanAdmissionDenial::OverSpanLimit)
    );
}

#[test]
fn a_span_starting_inside_a_page_is_denied() {
    assert_eq!(
        admit_span(span(512, 0, 2), format()),
        Err(SpanAdmissionDenial::Misaligned)
    );
}
