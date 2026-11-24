```compile_fail,E0277
use worth_relational::facade::durability::RelationalAssociatedEffectSubject;
fn cannot_readmit_a_source_subject_from_data(bytes: &[u8]) {
    let _: RelationalAssociatedEffectSubject = rmp_serde::from_slice(bytes).unwrap();
}
```
