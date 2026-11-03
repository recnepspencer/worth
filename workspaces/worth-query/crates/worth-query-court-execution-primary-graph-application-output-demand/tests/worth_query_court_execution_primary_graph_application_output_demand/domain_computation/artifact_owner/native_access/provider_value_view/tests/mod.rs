use worth_foundational::facade::CanonicalBigInt;
use worth_foundational::facade::CanonicalDecimal;
use worth_foundational::facade::CanonicalRational;
use worth_foundational::facade::InternedString;

use worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderFieldSlice;
use worth_query_execution::facade::domain_computation::WorthQueryArtifactProviderValueView;

#[test]
fn scalar_source_bytes_include_variable_width_backing_capacity() {
    let decimal = CanonicalDecimal::new(retained("123.5", 128));
    let rational = CanonicalRational::new(
        CanonicalBigInt::new(retained("7", 96)),
        CanonicalBigInt::new(retained("11", 160)),
    )
    .unwrap();
    let text = InternedString::Raw(retained("candidate", 224));

    assert_eq!(
        WorthQueryArtifactProviderValueView::Decimal(&decimal).physical_bytes(),
        std::mem::size_of_val(&decimal) + 128
    );
    assert_eq!(
        WorthQueryArtifactProviderValueView::Rational(&rational).physical_bytes(),
        std::mem::size_of_val(&rational) + 96 + 160
    );
    assert_eq!(
        WorthQueryArtifactProviderValueView::String(&text).physical_bytes(),
        std::mem::size_of_val(&text) + 224
    );
}

#[test]
fn bulk_source_bytes_equal_inline_slice_plus_every_nested_capacity() {
    let values = [
        InternedString::Raw(retained("a", 64)),
        InternedString::Raw(retained("b", 192)),
    ];

    assert_eq!(
        WorthQueryArtifactProviderFieldSlice::String(&values).physical_bytes(),
        std::mem::size_of_val(values.as_slice()) + 64 + 192
    );
}

pub fn retained(value: &str, capacity: usize) -> String {
    let mut retained = String::with_capacity(capacity);
    retained.push_str(value);
    retained
}
