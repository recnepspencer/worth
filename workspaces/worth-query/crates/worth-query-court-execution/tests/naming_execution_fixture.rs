//! Shared CAD schema, installation, lowering, and Relational runtime fixture
//! for end-to-end Slice 1B persistent-naming execution courts.

use std::num::NonZeroUsize;
use std::sync::OnceLock;

use worth_execution::{
    CancellationToken, ExecutionAllocationPolicy, ExecutionAuthority, ExecutionAuthorityConfig,
    ExecutionRequest, ExecutionResourceLease, LeaseRequest,
};
use worth_foundational::{
    DeterminismContract, ExecutionBudget, ExecutionPosture, ExecutionRequestPolicy,
};
use worth_query_declaration::facade::application_naming::{
    ApplicationGroupContinuityRule, ApplicationGroupNameSchemeBuilder, ApplicationNameSchemeBuilder,
    WorthQueryName,
};
use worth_query_declaration::facade::application_schema::{fragment, *};
use worth_query_execution_naming::facade::LoweredExecutionNameSchemeCatalog;
use worth_query_installation::facade::install_schema_name_schemes;
use worth_relational::facade::config::{
    CascadeDeletePolicy, CrossContextPolicy, RelationalRuntimeProfile,
};
use worth_relational::facade::identity::{KindId, PartitionId};
use worth_relational::facade::mvcc::{
    BranchBoundRelationalTransaction, RelationalTransactionIntent,
};
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::{
    EntityKindRegistration, KindAspectContractDeclarations, RelationIntegrityDeclarations,
    RelationKindRegistration, RelationalSchemaRegistry, SchemaId, SchemaVersionId,
};
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{
    AspectFieldPatch, CreateIntent, CreatedEntityRef, CreatedRelationRef, EntityReference,
    EntitySpec, MutationIntent, RelationSpec, WorkerIntentBatch,
};

pub const FACE_KIND_ID: KindId = KindId(1);
pub const SEAM_KIND_ID: KindId = KindId(2);
pub const FACE_SCHEME_ID: &str = "worth.cad.face.v1";
pub const SEAM_SCHEME_ID: &str = "worth.cad.seam.v1";
pub const ROOM_GROUP_SCHEME_ID: &str = "worth.cad.room.v1";

pub struct Face;
pub struct Seam;
pub struct RoomGroup;

impl fragment::Entity for Face {
    const NAME: &'static str = "Face";
    type Aspects = ();
    type Relations = (Seam,);
}

impl fragment::Relation for Seam {
    const NAME: &'static str = "Seam";
    type To = Face;
}

pub struct CadNamingFragment;
impl fragment::Fragment for CadNamingFragment {
    const NAME: &'static str = "CadNaming";
    type Bindings = fragment::NoBindings;
    type Members = (Face,);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FaceSplitDisc {
    Side { side: u8 },
    Cutter { cutter: WorthQueryName<Face>, piece: u8 },
}

worth_query_declaration::worth_query_name_discriminator! {
    enum FaceSplitDisc {
        identity: "worth.cad.face.split.v1",
        referenced_roles: ["authored", "cut_piece"],
        variants: {
            Side { side },
            Cutter { cutter, piece },
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SeamSplitDisc {
    Ordinal { index: u8 },
    ByFace { face: WorthQueryName<Face>, index: u8 },
}

worth_query_declaration::worth_query_name_discriminator! {
    enum SeamSplitDisc {
        identity: "worth.cad.seam.split.v1",
        variants: {
            Ordinal { index },
            ByFace { face, index },
        }
    }
}

pub struct CadNamingSchema;
impl ApplicationSchema for CadNamingSchema {
    const OWNER: &'static str = "worth.tests.execution.naming";
    const NAME: &'static str = "CadNaming";

    fn declaration(
    ) -> Result<ApplicationSchemaDeclaration<Self>, ApplicationSchemaDeclarationDenial> {
        let face_scheme = ApplicationNameSchemeBuilder::<Self, Face>::new(FACE_SCHEME_ID)
            .role_order(&["authored", "cut_piece", "seam_patch"])
            .role_depends_on("seam_patch", "cut_piece")
            .split_discriminator::<FaceSplitDisc>()
            .require_authored_occurrence()
            .build()
            .expect("valid face scheme");
        let seam_scheme = ApplicationNameSchemeBuilder::<Self, Seam>::for_relation(
            SEAM_SCHEME_ID,
            fragment::relation(CadNamingFragment, Face, Seam),
        )
        .role_order(&["seam", "seam_piece"])
        .split_discriminator::<SeamSplitDisc>()
        .require_authored_occurrence()
        .build()
        .expect("valid seam scheme");
        let room_scheme = ApplicationGroupNameSchemeBuilder::<Self, RoomGroup>::new(
            ROOM_GROUP_SCHEME_ID,
            "RoomGroup",
            ApplicationGroupContinuityRule::Boundary,
        )
        .role_order(&["wall", "producer", "void"])
        .boundary_role("wall")
        .anchor_role("producer")
        .require_authored_occurrence()
        .build()
        .expect("valid room group scheme");

        ApplicationSchemaDeclarationBuilder::from_fragment(CadNamingFragment)?
            .application_name_scheme::<Face>(face_scheme)?
            .application_name_scheme::<Seam>(seam_scheme)?
            .application_group_name_scheme::<RoomGroup>(room_scheme)?
            .build()
    }
}

pub fn lowered_cad_catalog() -> LoweredExecutionNameSchemeCatalog {
    let decl = CadNamingSchema::declaration().expect("CAD schema builds");
    let installed = install_schema_name_schemes(decl.erased()).expect("CAD schema installs");
    LoweredExecutionNameSchemeCatalog::lower_from_installed(&installed)
        .expect("CAD name schemes lower cleanly")
}

pub fn release_commit_snapshot(
    runtime: &RelationalRuntime,
    committed: &worth_relational::facade::transactions::CommitResult,
) {
    worth_query_testkit_execution_primary_graph_application_attempt::domain_computation::primary_graph::tests::fixture::commit_snapshot_closeout::release_test_commit_snapshot(
        runtime,
        committed,
    );
}

pub fn fresh_cad_runtime() -> RelationalRuntime {
    let registry = RelationalSchemaRegistry::new()
        .register_entity_kind(EntityKindRegistration {
            kind_id: FACE_KIND_ID,
            kind_name: "Face".to_owned(),
            schema_id: SchemaId("worth.tests.execution.naming.CadNaming".to_owned()),
            schema_version_id: SchemaVersionId(1),
            aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
        })
        .and_then(|reg| {
            reg.register_relation_kind(RelationKindRegistration {
                kind_id: SEAM_KIND_ID,
                kind_name: "Seam".to_owned(),
                schema_id: SchemaId("worth.tests.execution.naming.CadNaming".to_owned()),
                schema_version_id: SchemaVersionId(1),
                cross_context_policy: CrossContextPolicy::AllowExplicit,
                cascade_delete_policy: CascadeDeletePolicy::CascadeDeleteRelations,
                aspect_contract_declarations: KindAspectContractDeclarations::new(Vec::new()),
                relation_integrity: RelationIntegrityDeclarations::default(),
            })
        })
        .expect("valid relational schema registry");

    RelationalRuntimeApi::builder()
        .profile(RelationalRuntimeProfile::CertificationCore)
        .schema_registry(registry)
        .build()
        .expect("valid relational runtime")
}

static SHARED_EXECUTION_AUTHORITY: OnceLock<ExecutionAuthority> = OnceLock::new();

pub fn execution_lease_for_workers(workers: usize) -> ExecutionResourceLease<'static> {
    let authority = SHARED_EXECUTION_AUTHORITY.get_or_init(|| {
        ExecutionAuthority::try_construct(ExecutionAuthorityConfig {
            max_workers: NonZeroUsize::new(8).expect("positive max workers"),
            charged_memory_bytes: Some(512 * 1024 * 1024),
        })
        .expect("shared court execution authority")
    });
    authority
        .request_lease(LeaseRequest {
            policy: ExecutionRequestPolicy::new(
                ExecutionPosture::Automatic,
                DeterminismContract::CanonicalBitwise,
                ExecutionBudget::new(
                    NonZeroUsize::new(workers).expect("positive worker count"),
                    None,
                    Some(2_000_000),
                ),
            ),
            deadline: None,
            cancellation: CancellationToken::new(),
        })
        .expect("execution lease admitted")
}

pub fn face_ref(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: FACE_KIND_ID,
        client_key: ClientKey::raw(key),
    }
}

pub fn seam_ref(
    key: &str,
    source: EntityReference,
    target: EntityReference,
) -> CreatedRelationRef {
    CreatedRelationRef {
        partition_id: PartitionId::main(),
        kind_id: SEAM_KIND_ID,
        client_key: ClientKey::raw(key),
        source,
        target,
    }
}

pub fn create_face_intent(key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: FACE_KIND_ID,
        client_key: ClientKey::raw(key),
        fields: AspectFieldPatch::default(),
    }))
}

pub fn create_seam_intent(
    key: &str,
    source: EntityReference,
    target: EntityReference,
) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Relation(RelationSpec {
        partition_id: PartitionId::main(),
        kind_id: SEAM_KIND_ID,
        client_key: ClientKey::raw(key),
        source,
        target,
        fields: AspectFieldPatch::default(),
    }))
}

pub fn begin_staged_tx(
    runtime: &RelationalRuntime,
    worker_id: &str,
    intents: Vec<MutationIntent>,
) -> BranchBoundRelationalTransaction {
    let basis = runtime
        .admit_branch_basis(&runtime.main_branch_identity())
        .expect("main branch basis");
    let mut tx = runtime
        .begin_branch_transaction(&basis, RelationalTransactionIntent::ordinary())
        .expect("begin branch transaction");
    let mut batch = WorkerIntentBatch::new(worker_id);
    for intent in intents {
        batch = batch.push(intent);
    }
    tx.push_batch(batch, ExecutionAllocationPolicy::SystemAllocation)
        .expect("stage intent batch");
    tx
}

pub fn commit_dummy_preallocations(
    runtime: &RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    dummy_entities: usize,
    dummy_relations: usize,
) {
    if dummy_entities == 0 && dummy_relations == 0 {
        return;
    }
    let mut intents = Vec::with_capacity(dummy_entities + dummy_relations);
    let mut refs = Vec::with_capacity(dummy_entities);
    for i in 0..dummy_entities {
        let key = format!("dummy.e.{i}");
        refs.push(face_ref(&key));
        intents.push(create_face_intent(&key));
    }
    for i in 0..dummy_relations {
        let src = EntityReference::Created(refs[i % refs.len()].clone());
        let dst = EntityReference::Created(refs[(i + 1) % refs.len()].clone());
        intents.push(create_seam_intent(&format!("dummy.r.{i}"), src, dst));
    }
    let tx = begin_staged_tx(runtime, "dummy-prealloc", intents);
    let committed = runtime
        .commit_branch_transaction(execution, tx)
        .expect("dummy preallocation commit succeeds");
    release_commit_snapshot(runtime, &committed);
}
