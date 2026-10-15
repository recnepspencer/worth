//! Distinct rooted profile roundtrips; direct member tag and canonical meaning stay unchanged.
use super::super::member;
use super::fixture;
use crate::{
    binary_input::BinaryInput, binary_output::BinaryOutput, limits::WorthQueryPackageArchiveLimits,
    record::decode_budget::RecordDecodeAttempt,
};
use worth_query_declaration::facade::{
    application_capability::{
        application_capability_canonical_components, ApplicationCapabilityResourcePath,
        ApplicationCapabilityTargetDefinition, ErasedApplicationCapabilityContract,
    },
    application_schema::{
        ApplicationAuthorizationTraversal, ApplicationAuthorizationTraversalDirection as Direction,
        ApplicationSchemaMember as Member,
        WorthQueryPortableApplicationAuthorizationTraversalParts as Step,
    },
};

fn capability() -> ErasedApplicationCapabilityContract {
    fixture::complete_untrusted_schema_record()
        .members()
        .iter()
        .find_map(|member| match member {
            Member::ApplicationCapability { contract } => Some(contract.clone()),
            _ => None,
        })
        .unwrap()
}
fn roundtrip(member: &Member) {
    let mut output = BinaryOutput::with_capacity(4096);
    member::write(&mut output, member).unwrap();
    let bytes = output.into_bytes();
    assert_eq!(
        u16::from_be_bytes(bytes[..2].try_into().unwrap()),
        member::member_tag(member)
    );
    let mut input = BinaryInput::new(&bytes);
    let mut budget = RecordDecodeAttempt::begin(
        Default::default(),
        bytes.len() as u64,
        WorthQueryPackageArchiveLimits::DEFAULT,
    )
    .unwrap();
    assert_eq!(member::decode(&mut input, &mut budget).unwrap(), *member);
    assert!(input.is_finished());
    assert!(budget.finish().nested_entries() > 0);
}
#[test]
fn direct_resource_profile_keeps_tag_seven_and_has_no_new_canonical_components() {
    let contract = capability();
    assert!(contract.target().resource_path().is_direct());
    assert!(!application_capability_canonical_components(&contract)
        .iter()
        .any(|component| component.locus().starts_with("target.resource-path")));
    let member = Member::ApplicationCapability { contract };
    assert_eq!(member::member_tag(&member), 7);
    roundtrip(&member);
}
#[test]
fn rooted_resource_member_preserves_order_direction_and_context_slot_without_normalization() {
    let mut parts = capability().parts();
    let root = parts.composition.decision().allow().graph().requirements()[0].clauses()[0]
        .context_anchors()[0]
        .slot()
        .clone();
    let head = parts.target.resource().clone();
    let path = ApplicationCapabilityResourcePath::from_untrusted_parts(
        head.clone(),
        Some(root),
        vec![
            ApplicationAuthorizationTraversal::from_untrusted_parts(Step {
                relation: "Current".into(),
                from: head.to().into(),
                to: "Leaf".into(),
                direction: Direction::Forward,
            }),
            ApplicationAuthorizationTraversal::from_untrusted_parts(Step {
                relation: "Parent".into(),
                from: "Scope".into(),
                to: "Leaf".into(),
                direction: Direction::Reverse,
            }),
        ],
    );
    parts.target = ApplicationCapabilityTargetDefinition::new(
        parts.target.action().clone(),
        path,
        parts.target.relation().clone(),
        parts.target.field().clone(),
        parts.target.purpose().clone(),
    );
    let contract = ErasedApplicationCapabilityContract::from_untrusted_parts(parts);
    assert_eq!(contract.target().resource_path().scope_entity(), "Scope");
    assert!(application_capability_canonical_components(&contract)
        .iter()
        .any(|component| component.locus() == "target.resource-path.profile"));
    let member = Member::ApplicationCapability { contract };
    assert_eq!(member::member_tag(&member), 33);
    // This intentionally untrusted codec fixture is not a cold-admission oracle.
    roundtrip(&member);
}
