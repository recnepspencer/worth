//! Typed record expansion observes membership work separately from canonical bytes.
use super::comparison_observation::{Counts, Workload};
use super::{Member, ReadTarget};
use crate::application_schema::{fragment::*, *};
use std::collections::BTreeSet;

struct Schema;
struct Suite<const N: usize>;
struct Item<const N: usize>;
struct Values<const N: usize>;
struct Row<const N: usize> {
    values: [u64; N],
}
struct Bindings;
impl<const N: usize> Fragment for Suite<N> {
    const NAME: &'static str = "insertion";
    type Bindings = Bindings;
    type Members = (Item<N>,);
}
impl<const N: usize> Entity for Item<N> {
    const NAME: &'static str = "item";
    type Aspects = (Values<N>,);
    type Relations = ();
}
impl<const N: usize> Aspect for Values<N> {
    const NAME: &'static str = "values";
    type Row = Row<N>;
}
impl BindingSet for Bindings {
    type Work<'a> = ValueDecodeAdmission<'a>;
    fn with_admission<T>(
        work: &mut Self::Work<'_>,
        run: impl FnOnce(&mut ValueDecodeAdmission<'_>) -> T,
    ) -> T {
        run(work)
    }
}
impl Scalar<u64> for Bindings {
    type Binding = U64ApplicationValueBinding;
}
impl<const N: usize> Record<Row<N>> for Bindings {
    fn record() -> impl Shape<Row<N>, Self> {
        field("values", |row: &Row<N>| row.values).build(
            |values, _: &mut ValueDecodeAdmission<'_>| {
                Ok::<_, std::convert::Infallible>(Row { values })
            },
        )
    }
}
crate::worth_query_structured_value_binding!(Input for () { identity: "worth.tests.insertion-input.v1" });
crate::worth_query_operation!(Read for Schema, input Input);
impl ApplicationSchema for Schema {
    const OWNER: &'static str = "worth.tests.insertion";
    const NAME: &'static str = "insertion";
    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        declaration::<10>()
    }
}
fn declaration<const N: usize>(
) -> Result<ApplicationSchemaDeclaration<Schema>, ApplicationSchemaDeclarationDenial> {
    let operation = Read::reference();
    ApplicationSchemaDeclarationBuilder::from_fragment(Suite::<N>)?
        .operation_read_record(operation, Suite::<N>, Item::<N>, Values::<N>)
        .operation(
            operation
                .definition()
                .no_external_effect()
                .no_aftermath()
                .finish(),
        )
        .build()
}
fn measure<const N: usize>() -> Counts {
    let workload = Workload::start();
    let schema = declaration::<N>().unwrap();
    let counts = workload.snapshot();
    let fields = schema
        .erased()
        .members()
        .iter()
        .filter_map(|member| match member {
            Member::OperationDecisionRead {
                operation,
                target:
                    ReadTarget::Field {
                        entity,
                        aspect,
                        field,
                    },
            } if operation == Read::reference().name() => {
                assert_eq!(entity, Item::<N>::NAME);
                assert_eq!(
                    aspect,
                    &super::super::super::identity::native_aspect_name("item", "values")
                );
                Some(field.clone())
            }
            _ => None,
        })
        .collect::<BTreeSet<_>>();
    let expected = (0..N)
        .map(|index| format!("values[{index}]"))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        fields, expected,
        "real typed expanded field contents at width {N}"
    );
    assert!(
        counts.local > 0 && counts.global > 0,
        "both insertion owners must be observed"
    );
    eprintln!(
        "typed width={N} expanded={} LOCAL={} GLOBAL={}",
        fields.len(),
        counts.local,
        counts.global
    );
    counts
}
#[test]
fn typed_record_membership_comparison_growth() {
    let counts = [measure::<10>(), measure::<100>(), measure::<1000>()];
    for (index, pair) in counts.windows(2).enumerate() {
        assert!(
            pair[1].local <= 40 * (pair[0].local + 1),
            "LOCAL insertion growth {}->{}: {}->{}",
            10_usize.pow(index as u32 + 1),
            10_usize.pow(index as u32 + 2),
            pair[0].local,
            pair[1].local
        );
        assert!(
            pair[1].global <= 40 * (pair[0].global + 1),
            "GLOBAL insertion growth {}->{}: {}->{}",
            10_usize.pow(index as u32 + 1),
            10_usize.pow(index as u32 + 2),
            pair[0].global,
            pair[1].global
        );
    }
}
