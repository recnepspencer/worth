//! A coupled world whose kind also declares a content-bearing Struct aspect
//! (`samples`, two `ContentRef` fields bound to one sample-grid format).
use super::*;
use crate::content::fixture::{grid, Codec};
use std::sync::Arc;
use worth_foundational::facade::*;
use worth_relational::facade::content::{RelationalContentAttachments, RelationalContentFrame};

pub const GRID: CanonicalDigestId = CanonicalDigestId::new([31; 32]);
const LEFT: CanonicalDigestId = CanonicalDigestId::new([32; 32]);
const RIGHT: CanonicalDigestId = CanonicalDigestId::new([33; 32]);

impl World {
    /// The owned-source world over the content schema, with its `samples` contract.
    pub fn new_with_content(execution: &ExecutionRequest<'_, '_>) -> (Self, AspectContract) {
        let fields = [("left", LEFT), ("right", RIGHT)].map(|(name, binding)| {
            FieldDeclaration::new(
                FieldKey::new(name).unwrap(),
                ScalarAspectType::ContentRef,
                FieldRequirement::Required,
                AbsenceLaw::Required,
                AspectEvolutionPolicy::ExplicitBreakRequired,
            )
            .unwrap()
            .with_codec(binding)
        });
        let contract = AspectContract::struct_aspect(
            AspectKey::new("samples").unwrap(),
            AspectIdentity(741),
            AspectContractRevision(1),
            StructAspectShape::new(fields).unwrap(),
        );
        let mut fixture = AspectSchemaFixture::with_default_declared_aspects(RETAIN);
        fixture.entity_aspects.push(DeclaredAspectContractBinding {
            binding: AspectBinding::EntityField {
                field: FieldKey::new("samples").unwrap(),
            },
            contract: contract.clone(),
        });
        let codec = Arc::new(Codec {
            descriptor: ContentFormat::new("sample-grid", 1).unwrap(),
            alter: false,
        });
        let registry = register_current_publication_storage(
            fixture.build_registry(),
            &fixture.schema_id,
            fixture.schema_version_id,
        )
        .unwrap()
        .register_content_format(GRID, codec)
        .unwrap()
        .bind_content_field(LEFT, GRID)
        .unwrap()
        .bind_content_field(RIGHT, GRID)
        .unwrap();
        let mut runtime = RelationalRuntimeApi::builder()
            .schema_registry(registry.clone())
            .build()
            .unwrap();
        let receipt = runtime
            .prepare_initial_schema_installation()
            .unwrap()
            .install(RelationalSchemaRegistry::new())
            .unwrap();
        let (owner, installed) = receipt.issue_application_publication_owner();
        let installed = installed.with_source_scope_owned_producers([("owned".into(), KindId(1))]);
        let mut tx = begin_main_transaction(&runtime);
        tx.push_batch(
            create("source").push(create_intent("carrier")),
            execution.allocation_policy(),
        )
        .unwrap();
        let result = tx.commit(execution, &runtime).unwrap();
        let source = result.created_entity(&target("source")).unwrap();
        let carrier = result.created_entity(&target("carrier")).unwrap();
        release_test_commit_snapshot(&runtime, &result);
        let world = Self {
            runtime,
            owner,
            installed,
            source,
            carrier,
            summary: false,
            optional_struct_only: false,
            registry: Some(registry),
        };
        (world, contract)
    }

    /// A content frame of the sample grid with its first sample `first`.
    pub fn frame(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        first: i64,
    ) -> RelationalContentFrame {
        let mut samples = grid();
        samples.samples[0] = first;
        self.basis()
            .encode_content(execution, GRID, &samples)
            .unwrap()
    }

    /// Prepare an edit envelope whose authored prefix also applies `prefix`
    /// with `content` attached.
    pub fn prepare_with_content_prefix(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        prefix: WorkerIntentBatch,
        content: &RelationalContentAttachments,
        assembly: &dyn RelationalAuthoredCoupledProducerAssembly,
    ) -> Result<ValidatedRelationalProposal, TransactionCommitError> {
        let owners = self.runtime.readmit_edit_coupled_owners(
            execution,
            &self.installed,
            &self.basis(),
            self.source,
        )?;
        let mut tx = begin_main_transaction(&self.runtime);
        tx.push_batch(
            update(self.source, "edited-source"),
            execution.allocation_policy(),
        )
        .unwrap();
        tx.push_batch(prefix, execution.allocation_policy())
            .unwrap();
        tx.attach_content(execution, content).unwrap();
        let tx = tx.bind_authored_coupled_producers(&self.installed, owners, None)?;
        self.runtime
            .preparation_port()
            .prepare_authored_coupled_transaction(execution, tx, assembly)?
            .expect("completed real assembly")
            .validate(execution, &self.runtime)
    }
}

/// Attachments holding exactly `frames`.
pub fn attachments(
    execution: &ExecutionRequest<'_, '_>,
    frames: &[&RelationalContentFrame],
) -> RelationalContentAttachments {
    let mut content = RelationalContentAttachments::default();
    for frame in frames {
        content.attach(execution, frame).unwrap();
    }
    content
}

/// A create of `key` whose `samples` both reference `frame`.
pub fn content_create(
    contract: &AspectContract,
    key: &str,
    frame: &RelationalContentFrame,
) -> MutationIntent {
    let value = StructAspectValue::new(["left", "right"].map(|name| {
        (
            FieldKey::new(name).unwrap(),
            AspectValue::ContentRef(frame.identity()),
        )
    }))
    .unwrap();
    let patch = PortableRecordAspectPatch::new([PortableAspectPatchOperation::SetWhole {
        basis: PortableAspectContractBasis::from_contract(contract),
        value: ContractValidationInput::Struct(value),
    }]);
    MutationIntent::Create(CreateIntent::EntityAspects(EntityAspectCreateIntent {
        partition_id: worth_relational::facade::identity::PartitionId::main(),
        kind_id: KindId(1),
        client_key: ClientKey::raw(key),
        aspect_patch: patch,
    }))
}
