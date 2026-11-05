//! Local identity sensitivity; these literals carry no execution authority.
use super::*;
use crate::domain_operation::{
    WorthQueryOperationApplicationProjectionScope, WorthQueryOperationGraphAccess,
    WorthQueryOperationGraphParticipation, WorthQueryOperationGraphReadRole,
    WorthQueryOperationGraphReadScope, WorthQueryOperationNativeProjectionContract,
};
use worth_foundational::facade::{
    AbsenceLaw, AspectBinding, AspectContract, AspectContractRevision, AspectEvolutionPolicy,
    AspectIdentity, AspectKey, AspectMask, CanonicalFieldPath, FieldDeclaration, FieldKey,
    FieldRequirement, ProjectionMask, ScalarAspectType, StructAspectShape,
};

fn projection(revision: u64, fields: &[&str]) -> WorthQueryOperationGraphReadScope {
    let contract = AspectContract::struct_aspect(
        AspectKey::new("Facts").unwrap(),
        AspectIdentity(37),
        AspectContractRevision(revision),
        StructAspectShape::new(["a", "a|b", "b"].map(|name| {
            FieldDeclaration::new(
                FieldKey::new(name).unwrap(),
                ScalarAspectType::String,
                FieldRequirement::Required,
                AbsenceLaw::Required,
                AspectEvolutionPolicy::ExplicitBreakRequired,
            )
            .unwrap()
        }))
        .unwrap(),
    );
    let mask = AspectMask::<ProjectionMask>::new(
        fields
            .iter()
            .map(|name| CanonicalFieldPath::single(FieldKey::new(*name).unwrap())),
    );
    WorthQueryOperationGraphReadScope::NativeProjection(
        WorthQueryOperationApplicationProjectionScope::new(
            binding(),
            "Record".into(),
            contract.key().clone(),
            WorthQueryOperationNativeProjectionContract::new(contract, mask).unwrap(),
            AspectBinding::StructuralRegion,
        ),
    )
}

fn identity(
    scopes: Vec<WorthQueryOperationGraphReadScope>,
) -> WorthQueryInstalledGraphObligationSetIdentity {
    let contracts = [
        WorthQueryInstalledGraphObligationContract::OperationGraphRead {
            role: WorthQueryOperationGraphReadRole::new(
                "primary".into(),
                WorthQueryOperationGraphParticipation::PrimaryLogicalGraph,
                WorthQueryOperationGraphAccess::Project,
                scopes,
            ),
        },
    ];
    derive_set_identity(
        worth_foundational::facade::CanonicalDigestWorkBudget::new(4096, 1024 * 1024).unwrap(),
        &binding(),
        "application-operation",
        "apply-preserve",
        Some("Input"),
        &contracts,
        &WorthQueryInstalledGraphObligationResourcePosture::ApplicationOperation(
            crate::domain_computation::WorthQueryExecutionResourceContract::default(),
        ),
    )
    .unwrap()
    .0
}

fn binding() -> ApplicationSchemaBindingIdentity {
    ApplicationSchemaBindingIdentity::from_installed_parts(
        1,
        1,
        CanonicalDigestId::new([0x11; 32]),
        CanonicalDigestId::new([0x22; 32]),
    )
}

#[test]
fn one_projection_contract_or_mask_change_moves_the_identity() {
    let original = identity(vec![projection(1, &["a"]), projection(1, &["a"])]);
    assert_eq!(
        original,
        identity(vec![projection(1, &["a"]), projection(1, &["a"])])
    );
    assert_ne!(
        original,
        identity(vec![projection(1, &["a"]), projection(2, &["a"])])
    );
    assert_ne!(
        original,
        identity(vec![projection(1, &["a"]), projection(1, &["b"])])
    );
}

#[test]
fn mask_boundaries_and_projection_associations_cannot_alias() {
    assert_ne!(
        identity(vec![projection(1, &["a|b"])]),
        identity(vec![projection(1, &["a", "b"])])
    );
    assert_ne!(
        identity(vec![projection(1, &["a"]), projection(2, &["b"])]),
        identity(vec![projection(2, &["a"]), projection(1, &["b"])]),
    );
    assert_eq!(
        identity(vec![projection(1, &["b", "a"])]),
        identity(vec![projection(1, &["a", "b"])])
    );
}

#[test]
fn complete_field_marker_keeps_partial_and_whole_masks_distinct() {
    let complete = identity(vec![projection(1, &["a", "a|b", "b"])]);
    assert_ne!(complete, identity(vec![projection(1, &["a", "b"])]));
    assert_ne!(complete, identity(vec![projection(1, &[])]));
    assert_ne!(complete, identity(vec![projection(2, &["a", "a|b", "b"])]));
}

#[test]
fn every_touch_kind_and_locus_remain_in_the_identity() {
    use crate::domain_operation::{
        WorthQueryOperationEntityTouchScope, WorthQueryOperationTouchContract,
        WorthQueryOperationTouchScope,
    };
    let identity = |scope| {
        derive_set_identity(
            worth_foundational::facade::CanonicalDigestWorkBudget::new(4096, 1024 * 1024).unwrap(),
            &binding(),
            "application-operation",
            "apply-preserve",
            Some("Input"),
            &[WorthQueryInstalledGraphObligationContract::MutationTouch {
                contract: WorthQueryOperationTouchContract::Declared {
                    graph_roles: vec!["primary".into()],
                    scopes: vec![scope],
                },
            }],
            &WorthQueryInstalledGraphObligationResourcePosture::ApplicationOperation(
                crate::domain_computation::WorthQueryExecutionResourceContract::default(),
            ),
        )
        .unwrap()
        .0
    };
    let record = || WorthQueryOperationEntityTouchScope::new(binding(), "Record".into());
    let original = identity(WorthQueryOperationTouchScope::CreateEntity(record()));
    assert_ne!(
        original,
        identity(WorthQueryOperationTouchScope::DeleteEntity(record()))
    );
    assert_ne!(
        original,
        identity(WorthQueryOperationTouchScope::CreateEntity(
            WorthQueryOperationEntityTouchScope::new(binding(), "Other".into()),
        ))
    );
}
