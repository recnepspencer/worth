use super::*;
use ApplicationIdentityNamespaceDenial as Denial;
use ApplicationIdentityRole as Role;

const ROOT: &str = "daymark.tasks.add.v1";

fn namespace() -> ApplicationIdentityNamespace {
    ApplicationIdentityNamespace::new(ROOT).unwrap()
}

#[test]
fn every_role_derives_its_documented_segment_under_the_root() {
    let namespace = namespace();
    let derived: Vec<String> = Role::ALL
        .iter()
        .map(|role| namespace.identity(*role).to_string())
        .collect();
    assert_eq!(
        derived,
        [
            "input",
            "request",
            "parameters",
            "binding",
            "query",
            "handler",
            "idempotency",
            "denial",
            "result",
            "body",
            "completion",
            "projection",
        ]
        .map(|segment| format!("{ROOT}.{segment}"))
    );
    assert_eq!(namespace.binding().as_str(), "daymark.tasks.add.v1.binding");
    assert_eq!(
        namespace.slot("title").unwrap().as_str(),
        "daymark.tasks.add.v1.slot.title"
    );
}

#[test]
fn an_override_replaces_only_its_own_role() {
    let legacy = "bank.operation.reverse-journal.mutation-binding.v1";
    let namespace = namespace().with_override(Role::Binding, legacy).unwrap();
    assert_eq!(namespace.binding().as_str(), legacy);
    assert_eq!(namespace.input().as_str(), "daymark.tasks.add.v1.input");
    assert_eq!(namespace.root(), ROOT);
}

#[test]
fn malformed_roots_are_refused() {
    for root in ["", ".a", "a.", "a..b", "a b", "a\tb", "caf\u{e9}"] {
        assert_eq!(
            ApplicationIdentityNamespace::new(root),
            Err(Denial::InvalidRoot),
            "{root:?}"
        );
    }
}

#[test]
fn a_root_too_long_for_its_longest_role_is_refused() {
    let fits = "r".repeat(MAXIMUM_APPLICATION_IDENTITY_BYTES - 1 - "idempotency".len());
    let fits: &'static str = Box::leak(fits.into_boxed_str());
    assert_eq!(
        ApplicationIdentityNamespace::new(fits)
            .unwrap()
            .idempotency()
            .len(),
        MAXIMUM_APPLICATION_IDENTITY_BYTES
    );
    let overlong: &'static str = Box::leak(format!("{fits}r").into_boxed_str());
    assert_eq!(
        ApplicationIdentityNamespace::new(overlong),
        Err(Denial::Overlong {
            required: MAXIMUM_APPLICATION_IDENTITY_BYTES + 1,
            maximum: MAXIMUM_APPLICATION_IDENTITY_BYTES,
        })
    );
}

#[test]
fn slot_fields_are_single_bounded_segments() {
    let namespace = namespace();
    for field in ["", "a.b", "a b"] {
        assert_eq!(
            namespace.slot(field),
            Err(Denial::InvalidSlotField),
            "{field:?}"
        );
    }
    let room = MAXIMUM_APPLICATION_IDENTITY_BYTES - ROOT.len() - ".slot.".len();
    assert!(namespace.slot(&"f".repeat(room)).is_ok());
    assert!(matches!(
        namespace.slot(&"f".repeat(room + 1)),
        Err(Denial::Overlong { .. })
    ));
}

#[test]
fn overrides_that_would_collide_are_refused() {
    let namespace = namespace();
    for identity in [
        ROOT,
        "daymark.tasks.add.v1.input",
        "daymark.tasks.add.v1.slot.title",
    ] {
        assert_eq!(
            namespace.with_override(Role::Binding, identity),
            Err(Denial::OverrideCollision {
                role: Role::Binding
            }),
            "{identity:?}"
        );
    }
    let overridden = namespace
        .with_override(Role::Input, "legacy.input.v1")
        .unwrap();
    assert_eq!(
        overridden.with_override(Role::Result, "legacy.input.v1"),
        Err(Denial::OverrideCollision { role: Role::Result })
    );
    assert!(overridden
        .with_override(Role::Binding, "daymark.tasks.add.v1.input")
        .is_ok());
}

#[test]
fn malformed_and_repeated_overrides_are_refused() {
    let namespace = namespace();
    assert_eq!(
        namespace.with_override(Role::Denial, "a..b"),
        Err(Denial::InvalidOverride { role: Role::Denial })
    );
    let once = namespace
        .with_override(Role::Denial, "legacy.denial.v1")
        .unwrap();
    assert_eq!(
        once.with_override(Role::Denial, "other.denial.v1"),
        Err(Denial::DuplicateOverride { role: Role::Denial })
    );
}

#[test]
fn the_const_macros_match_the_runtime_derivation() {
    const ADD: ApplicationIdentityNamespace = crate::worth_query_identity_namespace!(
        "daymark.tasks.add.v1",
        Handler => "daymark.add.handler.v1",
    );
    const HANDLER: &str = crate::worth_query_identity!(ADD, handler);
    const RESULT: &str = crate::worth_query_identity!(ADD, result);
    const SLOT: &str = crate::worth_query_identity!(ADD, slot "due");
    let runtime = namespace()
        .with_override(Role::Handler, "daymark.add.handler.v1")
        .unwrap();
    assert_eq!(ADD, runtime);
    assert_eq!(HANDLER, runtime.handler().as_str());
    assert_eq!(RESULT, runtime.result().as_str());
    assert_eq!(SLOT, runtime.slot("due").unwrap().as_str());
}
