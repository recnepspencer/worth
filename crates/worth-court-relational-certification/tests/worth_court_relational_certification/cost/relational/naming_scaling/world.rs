//! A Relational runtime with real persistent naming composed: a
//! structural-identity member scheme, Anchor group partitions and a forked
//! branch, driven only through public Native transactions.
use worth_execution::{CancellationToken, ExecutionRequest, SerialMemoryBudget, SerialRequest};
use worth_relational::facade::aggregate_identity::{
    AggregateContinuityRule, AggregateGroupScheme, AggregateGroupSchemeId, AggregateMemberRole,
};
use worth_relational::facade::branch::AdmittedRelationalBranchBasis;
use worth_relational::facade::history::BranchId;
use worth_relational::facade::identity::{EntityId, KindId, PartitionId};
use worth_relational::facade::lineage::{
    CommitNamingPlan, InstalledPersistentNameScheme, InstalledPersistentNameSchemeRegistry,
    LineagePostureClaim, LineageRecordTarget, PendingBirthTarget, PendingCommitBirth,
    PendingSplitChild, PendingSplitClaim, PersistentNameDiscriminatorReference,
    PersistentNameIdentity, PersistentNameKey, PersistentNameReferenceBasis,
    PersistentNameResolution, PersistentNameSelection, PersistentNameSubjectKind,
    PersistentNameTarget,
};
use worth_relational::facade::mvcc::RelationalTransactionIntent;
use worth_relational::facade::runtime::{RelationalRuntime, RelationalRuntimeApi};
use worth_relational::facade::schema::*;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::*;
use worth_testkit_relational::aspect_storage::AspectSchemaFixture;

pub const SCHEME: &str = "worth.probe.member.v1";
const MEMBER: KindId = KindId(3);
pub const PIECE: &str = "piece";

pub struct World {
    pub runtime: RelationalRuntime,
    host: SerialRequest,
}

/// A bounded cut discriminator: the 32-byte cutter name and a `u64` step,
/// encoded exactly as the Query discriminator encoder writes them.
pub fn cut(cutter: &PersistentNameIdentity, step: u64) -> Vec<u8> {
    let mut bytes = b"worth.query.name-discriminator.v1\0".to_vec();
    let field = |bytes: &mut Vec<u8>, name: &str, tag: u8| {
        bytes.push(0x02);
        bytes.extend_from_slice(&(name.len() as u64).to_be_bytes());
        bytes.extend_from_slice(name.as_bytes());
        bytes.push(tag);
    };
    let identity = b"worth.probe.cut.v1";
    bytes.extend_from_slice(&(identity.len() as u64).to_be_bytes());
    bytes.extend_from_slice(identity);
    field(&mut bytes, "cutter", 0x20);
    bytes.extend_from_slice(cutter.as_bytes());
    field(&mut bytes, "step", 0x18);
    bytes.extend_from_slice(&step.to_be_bytes());
    bytes
}

fn created(key: &str) -> CreatedEntityRef {
    CreatedEntityRef {
        partition_id: PartitionId::main(),
        kind_id: MEMBER,
        client_key: ClientKey::raw(key),
    }
}

fn create(key: &str) -> MutationIntent {
    MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: MEMBER,
        client_key: ClientKey::raw(key),
        fields: AspectFieldPatch::default(),
    }))
}

fn delete(entity_id: EntityId) -> MutationIntent {
    MutationIntent::Entity(EntityMutationIntent::Delete(DeleteEntityIntent {
        entity_id,
    }))
}

impl World {
    pub fn new() -> Self {
        let fixture = AspectSchemaFixture::default();
        let registry = fixture
            .build_registry()
            .register_entity_kind(EntityKindRegistration {
                kind_id: MEMBER,
                kind_name: "Member".into(),
                schema_id: fixture.schema_id.clone(),
                schema_version_id: fixture.schema_version_id,
                aspect_contract_declarations: KindAspectContractDeclarations::default(),
            })
            .unwrap();
        let scheme = InstalledPersistentNameScheme::try_from_ordered_roles_for_subject(
            SCHEME,
            PersistentNameSubjectKind::Entity,
            "Member",
            ["authored", PIECE],
            true,
            false,
            true,
        )
        .unwrap();
        let mut schemes = InstalledPersistentNameSchemeRegistry::new();
        schemes.register(scheme).unwrap();
        let binding = (
            (PersistentNameSubjectKind::Entity, MEMBER),
            SCHEME.to_owned(),
        );
        let registry = registry
            .install_persistent_name_schemes(schemes, [binding].into())
            .unwrap();
        let mut runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry)
            .build()
            .unwrap();
        runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install_with_custom_invariants(RelationalSchemaRegistry::new(), Vec::new())
            .unwrap();
        let host = SerialRequest::from_memory(
            SerialMemoryBudget::new(None),
            CancellationToken::new(),
            None,
        );
        Self { runtime, host }
    }

    pub fn main(&self) -> AdmittedRelationalBranchBasis {
        let identity = self.runtime.main_branch_identity();
        self.runtime.observe_branch(&identity).unwrap().1
    }

    /// Forks `side` off main's head: the composed branch tree.
    pub fn fork(&self, side: &str) -> AdmittedRelationalBranchBasis {
        let execution = ExecutionRequest::serial(&self.host);
        let main = self.runtime.main_branch_identity();
        let (_, source) = self.runtime.observe_fork_source(main.branch_id()).unwrap();
        let fork = self
            .runtime
            .fork_branch(&execution, BranchId(side.into()), source)
            .unwrap();
        self.runtime
            .observe_branch(fork.target_identity())
            .unwrap()
            .1
    }

    /// Commits `intents` under `plan` on `basis`.
    pub fn commit(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        intents: Vec<MutationIntent>,
        plan: CommitNamingPlan,
    ) {
        let execution = ExecutionRequest::serial(&self.host);
        let mut transaction = self
            .runtime
            .begin_branch_transaction(basis, RelationalTransactionIntent::ordinary())
            .unwrap();
        let batch = intents
            .into_iter()
            .fold(WorkerIntentBatch::new("probe"), WorkerIntentBatch::push);
        transaction
            .push_batch(batch, execution.allocation_policy())
            .unwrap();
        transaction.attach_naming_plan(plan);
        let committed = transaction.commit(&execution, &self.runtime).unwrap();
        self.runtime
            .snapshots()
            .release_snapshot(&committed.snapshot)
            .unwrap();
    }

    /// One commit creating `keys`, each born authored in its own occurrence.
    pub fn authored(&self, basis: &AdmittedRelationalBranchBasis, keys: &[String]) {
        let births = keys.iter().map(|key| PendingCommitBirth {
            target: PendingBirthTarget::CreatedEntity(created(key)),
            scheme_id: SCHEME.to_owned(),
            occurrence_key: format!("probe.{key}"),
            role_path: "authored".to_owned(),
            parent_identities: Vec::new(),
            referenced_names: Vec::new(),
            canonical_discriminator: Vec::new(),
        });
        let plan = CommitNamingPlan {
            pending_births: births.collect(),
            ..CommitNamingPlan::default()
        };
        self.commit(basis, keys.iter().map(|key| create(key)).collect(), plan);
    }

    /// The name an authored `key` was born with.
    pub fn authored_name(key: &str) -> PersistentNameIdentity {
        PersistentNameIdentity::of_pending_birth(
            SCHEME,
            &format!("probe.{key}"),
            "authored",
            &[],
            &[],
            true,
        )
    }

    /// The live entity `name` designates at `basis`.
    pub fn entity(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        name: PersistentNameIdentity,
    ) -> EntityId {
        let execution = ExecutionRequest::serial(&self.host);
        let key = PersistentNameKey::Entity(name);
        self.runtime
            .with_persistent_names_at_observation(&execution, &basis.observation(), |view| {
                match view.resolve(key, &PersistentNameSelection::Exact).result {
                    Ok(PersistentNameResolution::Exact(PersistentNameTarget::Entity {
                        entity,
                        ..
                    })) => entity,
                    other => panic!("{name:?} designates one live member: {other:?}"),
                }
            })
            .unwrap()
    }

    /// The encoded bytes of `name`'s recorded birth.
    pub fn birth_bytes(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        name: PersistentNameIdentity,
    ) -> usize {
        let execution = ExecutionRequest::serial(&self.host);
        let key = PersistentNameKey::Entity(name);
        self.runtime
            .with_persistent_names_at_observation(&execution, &basis.observation(), |view| {
                rmp_serde::to_vec(view.birth_of(key).expect("a recorded birth"))
                    .unwrap()
                    .len()
            })
            .unwrap()
    }

    /// Deletes `name`'s member and retires the name.
    pub fn retire(&self, basis: &AdmittedRelationalBranchBasis, name: PersistentNameIdentity) {
        let entity = self.entity(basis, name);
        let plan = CommitNamingPlan {
            postures: vec![LineagePostureClaim::Retire {
                target: LineageRecordTarget::Entity(entity),
                name,
            }],
            ..CommitNamingPlan::default()
        };
        self.commit(basis, vec![delete(entity)], plan);
    }

    /// The split plan and intents cutting `parent` into `(key, step)`
    /// pieces told apart by `cutter` and their step; returns their names.
    pub fn split(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        parent: PersistentNameIdentity,
        cutter: PersistentNameIdentity,
        pieces: &[(String, u64)],
    ) -> (
        Vec<MutationIntent>,
        CommitNamingPlan,
        Vec<PersistentNameIdentity>,
    ) {
        let parent_entity = self.entity(basis, parent);
        let mut intents = vec![delete(parent_entity)];
        let mut children = Vec::new();
        let mut names = Vec::new();
        for (key, step) in pieces {
            intents.push(create(key));
            let discriminator = cut(&cutter, *step);
            let occurrence = "probe.cut".to_owned();
            names.push(PersistentNameIdentity::of_pending_birth(
                SCHEME,
                &occurrence,
                PIECE,
                &[parent],
                &discriminator,
                true,
            ));
            children.push(PendingSplitChild {
                target: PendingBirthTarget::CreatedEntity(created(key)),
                scheme_id: SCHEME.to_owned(),
                occurrence_key: occurrence,
                role_path: PIECE.to_owned(),
                referenced_names: vec![PersistentNameDiscriminatorReference {
                    identity: cutter,
                    basis: PersistentNameReferenceBasis::Basis,
                }],
                canonical_discriminator: discriminator,
            });
        }
        let plan = CommitNamingPlan {
            pending_splits: vec![PendingSplitClaim {
                parent_target: LineageRecordTarget::Entity(parent_entity),
                parent_name: parent,
                children,
            }],
            ..CommitNamingPlan::default()
        };
        (intents, plan, names)
    }

    /// Declares Anchor groups of `members` in chunks of `width`.
    pub fn group(
        &self,
        basis: &AdmittedRelationalBranchBasis,
        members: &[PersistentNameIdentity],
        width: usize,
    ) {
        let mut plan = CommitNamingPlan::default();
        let groups = members
            .chunks(width)
            .map(|chunk| {
                let role = |index| match index {
                    0 => AggregateMemberRole::Anchor,
                    _ => AggregateMemberRole::Ordinary,
                };
                chunk
                    .iter()
                    .enumerate()
                    .map(|(i, name)| (*name, role(i)))
                    .collect()
            })
            .collect();
        let scheme = AggregateGroupScheme::new(
            AggregateGroupSchemeId::new("probe.loop").unwrap(),
            AggregateContinuityRule::Anchor,
        );
        plan.declare_group_partition(scheme, Vec::new(), groups);
        self.commit(basis, Vec::new(), plan);
    }
}
