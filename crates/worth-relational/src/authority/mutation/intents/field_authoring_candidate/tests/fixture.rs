//! Pure Foundation/Native schema values, no runtime or request construction.
use super::*;
pub(super) fn text(value: &str) -> AspectValue {
    AspectValue::String(InternedString::Raw(value.into()))
}
pub(super) fn key(value: &str) -> AspectKey {
    AspectKey::new(value).unwrap()
}
pub(super) fn field(value: &str) -> FieldKey {
    FieldKey::new(value).unwrap()
}
pub(super) fn plan(wide: bool) -> LoweredAspectContractPlan {
    let scalar = AspectContract::scalar(
        key("z.scalar"),
        AspectIdentity(1),
        AspectContractRevision(1),
        ScalarAspectType::String,
    );
    let mut shape = aspects()
        .struct_fields()
        .optional("title", ScalarAspectType::String)
        .optional("status", ScalarAspectType::String);
    if wide {
        for i in 0..128 {
            shape = shape.optional(
                format!("unused-{i}-{}", "wide".repeat(32)),
                ScalarAspectType::String,
            );
        }
    }
    let structure = AspectContract::struct_aspect(
        key("a.struct"),
        AspectIdentity(2),
        AspectContractRevision(1),
        shape.finish().unwrap(),
    );
    LoweredAspectContractPlan {
        kind_id: KindId(1),
        plan_revision: AspectContractPlanRevision(1),
        executable_bindings: smallvec::smallvec![
            LoweredAspectContractBinding {
                contract: scalar,
                target: AspectBinding::EntityField {
                    field: field("value")
                }
            },
            LoweredAspectContractBinding {
                contract: structure,
                target: AspectBinding::EntityField {
                    field: field("summary")
                }
            },
        ],
    }
}
pub(super) fn target() -> RecordAspectPatchTarget {
    RecordAspectPatchTarget::EntityCreation { kind_id: KindId(1) }
}
pub(super) fn fields() -> AspectFieldPatch {
    AspectFieldPatch::new(BTreeMap::from([
        (
            AspectFieldLocator::new(
                LocatorAuthority::Authoritative,
                key("z.scalar"),
                CanonicalFieldPath::single(field("value")),
            ),
            text("old scalar"),
        ),
        (
            planned_single_field_locator(key("z.scalar"), field("value")),
            text("winner scalar"),
        ),
        (
            planned_single_field_locator(key("a.struct"), field("status")),
            text("open"),
        ),
        (
            planned_single_field_locator(key("a.struct"), field("title")),
            text("before"),
        ),
        (
            AspectFieldLocator::new(
                LocatorAuthority::ReceiptBearing,
                key("a.struct"),
                CanonicalFieldPath::single(field("title")),
            ),
            text("winner title"),
        ),
    ]))
}
