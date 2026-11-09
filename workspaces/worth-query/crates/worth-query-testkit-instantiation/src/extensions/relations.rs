//! Extension links and traversal use concrete ports; payload selectors use the record core.
use super::*;
pub fn extension_relations(
    read_first: &mut View<'_, ExtensionFirst, Reading<'_>>,
    read_second: &mut View<'_, ExtensionSecond, Reading<'_>>,
    write_first: &mut View<'_, ExtensionFirst, Writing<'_>>,
    write_second: &mut View<'_, ExtensionSecond, Writing<'_>>,
    observed: &ObservedEntity<ExtensionItem>,
    target: &CandidateEntity<ExtensionItem>,
) {
    write_first
        .link(target, ExtensionLink::<0>, target)
        .unwrap();
    std::hint::black_box(
        read_first
            .relations_from(observed, ExtensionLink::<0>)
            .unwrap(),
    );
    let field: RelationFieldSelection<_, _, _, _, u64> = select_relation_field(
        ExtensionFirst,
        ExtensionItem,
        ExtensionLink::<0>,
        ExtensionPayload::<0>,
        "value",
    );
    std::hint::black_box(field.checked().unwrap());
    write_first
        .link(target, ExtensionLink::<1>, target)
        .unwrap();
    std::hint::black_box(
        read_first
            .relations_from(observed, ExtensionLink::<1>)
            .unwrap(),
    );
    let field: RelationFieldSelection<_, _, _, _, u64> = select_relation_field(
        ExtensionFirst,
        ExtensionItem,
        ExtensionLink::<1>,
        ExtensionPayload::<1>,
        "value",
    );
    std::hint::black_box(field.checked().unwrap());
    write_second
        .link(target, ExtensionLink::<2>, target)
        .unwrap();
    std::hint::black_box(
        read_second
            .relations_from(observed, ExtensionLink::<2>)
            .unwrap(),
    );
    let field: RelationFieldSelection<_, _, _, _, u64> = select_relation_field(
        ExtensionSecond,
        ExtensionItem,
        ExtensionLink::<2>,
        ExtensionPayload::<2>,
        "value",
    );
    std::hint::black_box(field.checked().unwrap());
    write_second
        .link(target, ExtensionLink::<3>, target)
        .unwrap();
    std::hint::black_box(
        read_second
            .relations_from(observed, ExtensionLink::<3>)
            .unwrap(),
    );
    let field: RelationFieldSelection<_, _, _, _, u64> = select_relation_field(
        ExtensionSecond,
        ExtensionItem,
        ExtensionLink::<3>,
        ExtensionPayload::<3>,
        "value",
    );
    std::hint::black_box(field.checked().unwrap());
}
