use std::collections::BTreeMap;

use worth_foundational::{
    aspects, validate_aspect_value, AspectContract, AspectContractRevision, AspectIdentity,
    AspectKey as FoundationalAspectKey, AspectMask, AspectValue as FoundationalAspectValue,
    CanonicalFieldPath, InternedString as FoundationalInternedString, MutationMask,
    ScalarAspectType,
};
use worth_proof::TransitionOutcome;

use crate::authority::mutation::MutationWorkspace;
use crate::config::data::{
    AdjacencyBackend, AdjacencyPolicy, CascadeDeletePolicy, CrossContextPolicy,
};
use crate::identity::data::{KindId, VersionId};
use crate::schema::data::{
    AspectBinding, AspectContractPlanCatalog, AspectContractPlanRevision,
    LoweredAspectContractBinding, LoweredAspectContractPlan, RelationalSchemaRegistry,
};
use crate::storage::overlay::WorkingState;
use crate::symbols::data::StringInterner;

pub(super) fn mutation_config() -> crate::config::data::MutationConfig {
    crate::config::data::MutationConfig {
        cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
        adjacency_policy: AdjacencyPolicy {
            backend: AdjacencyBackend::InlineSmallDegreeAdjacency,
            small_degree_inline_capacity: 4,
        },
        cross_context_policy: CrossContextPolicy::AllowExplicit,
    }
}

pub(super) fn empty_workspace<'a>(
    state: &'a mut WorkingState,
    symbols: &'a mut StringInterner,
    aspect_plans: &'a AspectContractPlanCatalog,
    config: &'a crate::config::data::MutationConfig,
    schema: &'a RelationalSchemaRegistry,
) -> MutationWorkspace<'a> {
    MutationWorkspace::new(
        state,
        symbols,
        config,
        schema,
        aspect_plans,
        VersionId(1),
        crate::authority::mutation::BranchLocalDeleteAllowance::default(),
        None,
        None,
    )
}

pub(super) fn catalog_with_entity_binding(
    kind_id: KindId,
    contract: AspectContract,
    binding: AspectBinding,
) -> AspectContractPlanCatalog {
    let mut catalog = AspectContractPlanCatalog::empty();
    catalog.entity_plans.insert(
        kind_id,
        LoweredAspectContractPlan {
            kind_id,
            plan_revision: AspectContractPlanRevision(7),
            executable_bindings: smallvec::smallvec![LoweredAspectContractBinding {
                contract,
                target: binding,
            }],
        },
    );
    catalog
}

pub(super) fn catalog_with_relation_binding(
    kind_id: KindId,
    contract: AspectContract,
    binding: AspectBinding,
) -> AspectContractPlanCatalog {
    let mut catalog = AspectContractPlanCatalog::empty();
    catalog.relation_plans.insert(
        kind_id,
        LoweredAspectContractPlan {
            kind_id,
            plan_revision: AspectContractPlanRevision(3),
            executable_bindings: smallvec::smallvec![LoweredAspectContractBinding {
                contract,
                target: binding,
            }],
        },
    );
    catalog
}

pub(super) fn assert_authoritative_whole_aspect_locator(
    locator: &worth_foundational::facade::AspectValueLocator,
    expected_aspect_key: &str,
) {
    let worth_foundational::facade::AspectValueLocator::WholeAspect(aspect_locator) = locator
    else {
        panic!("expected authoritative whole-aspect value locator");
    };
    assert_eq!(
        aspect_locator.authority(),
        worth_foundational::facade::LocatorAuthority::Authoritative
    );
    assert_eq!(
        aspect_locator.aspect_key(),
        &FoundationalAspectKey::new(expected_aspect_key).expect("expected aspect key")
    );
}

pub(super) fn scalar_string_contract(
    aspect_key: FoundationalAspectKey,
    identity: u64,
    revision: u64,
) -> AspectContract {
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(AspectIdentity(identity))
        .at_revision(AspectContractRevision(revision))
        .scalar(ScalarAspectType::String)
}

pub(super) fn summary_struct_contract(aspect_key: FoundationalAspectKey) -> AspectContract {
    let shape = aspects()
        .struct_fields()
        .required("title", ScalarAspectType::String)
        .finish()
        .expect("valid summary struct shape");
    aspects()
        .contract()
        .for_key(aspect_key)
        .identified_by(AspectIdentity(41))
        .at_revision(AspectContractRevision(7))
        .struct_aspect(shape)
}

pub(super) fn authoritative_string_patch(
    contract: &AspectContract,
    value: &str,
) -> worth_foundational::facade::AuthoritativeRecordAspectPatch {
    let TransitionOutcome::Success(validated) = validate_aspect_value(
        contract,
        FoundationalAspectValue::String(FoundationalInternedString::Raw(value.to_string())).into(),
    ) else {
        panic!("test value should validate against scalar string contract");
    };
    let TransitionOutcome::Success(patch) =
        worth_foundational::facade::AuthoritativeRecordAspectPatch::whole_aspect([validated], [])
    else {
        panic!("test patch should construct one whole-aspect set");
    };
    patch
}

pub(super) fn authoritative_summary_patch(
    contract: &AspectContract,
    title: &str,
) -> worth_foundational::facade::AuthoritativeRecordAspectPatch {
    let value = worth_foundational::facade::StructAspectValue::new([(
        worth_foundational::facade::FieldKey::new("title").expect("valid field"),
        FoundationalAspectValue::String(title.into()),
    )])
    .expect("valid summary struct value");
    let TransitionOutcome::Success(validated) = validate_aspect_value(contract, value.into())
    else {
        panic!("test value should validate against struct contract");
    };
    let TransitionOutcome::Success(patch) =
        worth_foundational::facade::AuthoritativeRecordAspectPatch::whole_aspect([validated], [])
    else {
        panic!("test patch should construct one whole-aspect set");
    };
    patch
}

pub(super) fn authoritative_summary_field_patch(
    contract: &AspectContract,
    title: &str,
) -> worth_foundational::facade::AuthoritativeRecordAspectPatch {
    let field_key = worth_foundational::facade::FieldKey::new("title").expect("valid field");
    let mutation_mask =
        AspectMask::<MutationMask>::new([CanonicalFieldPath::single(field_key.clone())]);
    let TransitionOutcome::Success(patch) =
        worth_foundational::facade::AuthoritativeRecordAspectPatch::field_level(
            contract,
            &mutation_mask,
            [(field_key, FoundationalAspectValue::String(title.into()))],
            [],
        )
    else {
        panic!("test field-level patch should construct one field set");
    };
    patch
}

pub(super) fn empty_working_state(config: &crate::config::data::MutationConfig) -> WorkingState {
    WorkingState::new(BTreeMap::new(), config.adjacency_policy.clone())
}
