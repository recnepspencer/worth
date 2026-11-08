from pathlib import Path
r=Path.cwd()
def path(s):return Path('\\\\?\\'+str(r/s))
def read(s):return path(s).read_text(encoding='utf8')
def write(s,t):path(s).parent.mkdir(exist_ok=True,parents=True);path(s).write_text(t,encoding='utf8',newline='\n')
def replace(p,a,b):
 s=read(p);assert a in s,(p,a);write(p,s.replace(a,b))
b='crates/worth-runtime-bridge/src/source/async_declaration/'
p=b+'lowering.rs';s=read(p)
a=s.index('#[derive(Debug, Clone, PartialEq, Eq)]\nenum LoweredBridgeAsyncSourceDeclarationBody')
z=s.index('\nimpl LoweredBridgeAsyncSourceDeclaration',a)
body_enum=s[a:z].replace('enum LoweredBridgeAsyncSourceDeclarationBody','pub(super) enum LoweredBridgeAsyncSourceDeclarationBody')
s=s[:a]+s[z:]
s=s.replace('use std::sync::Arc;','mod construction;\nmod storage;\n#[cfg(test)]\nmod storage_tests;\n\npub(crate) use storage::OriginalLoweringStorage;\nuse storage::{LoweredBridgeAsyncSourceDeclarationBody, LoweringBodyStorage};\nuse std::sync::Arc;')
s=s.replace('body: LoweredBridgeAsyncSourceDeclarationBody,','body: Arc<LoweringBodyStorage>,')
a=s.index('        match validated.body()')
s=s[:a]+'''        Self::lower_with_storage(validated, None)
    }

    pub(crate) fn lower_original(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        storage: OriginalLoweringStorage,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
        Self::lower_with_storage(validated, Some(storage))
    }

    fn lower_with_storage(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        storage: Option<OriginalLoweringStorage>,
    ) -> Result<Self, BridgeAsyncSourceDeclarationRejection> {
'''+s[a:]
s=s.replace('Self::lower_request_response(validated, declaration)','Self::lower_request_response(validated, declaration, storage)')
s=s.replace('Self::lower_subscription_backed(validated, declaration)','Self::lower_subscription_backed(validated, declaration, storage)')
s=s.replace('match &self.body {','match &self.body.body {')
s=s.replace('        } = &self.body','        } = &self.body.body')
s=s.replace('            declaration,\n            descriptor,\n        } = &self.body.body','            descriptor, ..\n        } = &self.body.body')
s=s.replace('            self.digest(),','            self.body.template_digest,')
s=s.replace('''            body: LoweredBridgeAsyncSourceDeclarationBody::RequestResponse {
                declaration: declaration.clone(),
                descriptor: descriptor.clone(),
            },''','''            body: Arc::clone(&self.body),''')
s=s.replace('declaration: &ResourceNodeDeclaration,\n    )','declaration: &ResourceNodeDeclaration,\n        storage: Option<OriginalLoweringStorage>,\n    )',1)
s=s.replace('Ok(Self::from_request_response_parts(','Ok(Self::request_response_parts(',1)
s=s.replace('            descriptor.clone(),\n        ))','            descriptor.clone(),\n            storage,\n        ))',1)
s=s.replace('declaration: &AsyncNodeCapabilityDeclaration,\n    )','declaration: &AsyncNodeCapabilityDeclaration,\n        storage: Option<OriginalLoweringStorage>,\n    )',1)
s=s.replace('Self::new_subscription_backed(validated, remapped, bundle)','Self::new_subscription_backed(validated, remapped, bundle, storage)')
a=s.index('    pub(crate) fn from_request_response_parts(');z=s.index('    pub(crate) fn request_response_declaration',a)
constructors=s[a:z]
s=s[:a]+s[z:]
write(p,s)
constructors=constructors.replace('''        let canonical_basis = Arc::<str>::from(format!(''','''        let canonical_basis = Arc::<str>::from(format!(''')
# Keep the valid cold installation entry, with the shared semantic constructor.
a=constructors.index('        let canonical_basis')
constructors=constructors[:a]+'''        Self::request_response_parts(validated, declaration, descriptor, None)
    }

    pub(super) fn request_response_parts(
        validated: &ValidatedBridgeAsyncSourceDeclaration,
        declaration: ResourceNodeDeclaration,
        descriptor: LoweredResourceDescriptor,
        storage: Option<OriginalLoweringStorage>,
    ) -> Self {
'''+constructors[a:]
constructors=constructors.replace('    fn new_subscription_backed(','    pub(super) fn new_subscription_backed(')
constructors=constructors.replace('        bundle: LoweredAsyncNodeCapabilityBundle,\n    )','        bundle: LoweredAsyncNodeCapabilityBundle,\n        storage: Option<OriginalLoweringStorage>,\n    )')
# Each template digest string is allocated once and shared with the payload.
constructors=constructors.replace('''        let digest = Sha256::digest(canonical_basis.as_bytes());

        Self {''','''        let digest = Sha256::digest(canonical_basis.as_bytes());
        let digest: Arc<str> = Arc::from(format!("bridge-async-lowering:sha256:{digest:x}"));
        Self {''')
constructors=constructors.replace('''            lowering_identity: BridgeAsyncSourceLoweringIdentity::admit_bridge_owned(format!(
                "bridge-async-lowering:sha256:{digest:x}"
            )),''','''            lowering_identity: BridgeAsyncSourceLoweringIdentity::admit_bridge_owned(digest.clone()),''')
constructors=constructors.replace('''            digest: Arc::from(format!("bridge-async-lowering:sha256:{digest:x}")),''','''            digest: Arc::clone(&digest),''')
constructors=constructors.replace('''            body: LoweredBridgeAsyncSourceDeclarationBody::RequestResponse {
                declaration,
                descriptor,
            },''','''            body: LoweringBodyStorage::retain(
                LoweredBridgeAsyncSourceDeclarationBody::RequestResponse { declaration, descriptor },
                digest, storage),''')
constructors=constructors.replace('''            body: LoweredBridgeAsyncSourceDeclarationBody::SubscriptionBacked {
                declaration,
                bundle,
            },''','''            body: LoweringBodyStorage::retain(
                LoweredBridgeAsyncSourceDeclarationBody::SubscriptionBacked { declaration, bundle },
                digest, storage),''')
write(b+'lowering/construction.rs','//! Assemble canonical template meaning and its immutable storage.\nuse super::*;\n\nimpl LoweredBridgeAsyncSourceDeclaration {\n'+constructors+'}\n')
write(b+'lowering/storage.rs','''//! Original-request funding stays with the immutable lowering body.
use super::*;
use std::alloc::Layout;
use worth_execution::{ExecutionMemoryReservation, ExecutionRequest, LeaseDenial, WorkCeilingDenial};

'''+body_enum+'''

pub(crate) struct OriginalLoweringStorage(Option<ExecutionMemoryReservation>);

impl OriginalLoweringStorage {
    pub(crate) fn reserve(execution: ExecutionRequest<'_, '_>) -> Result<Self, WorkCeilingDenial> {
        execution.reserve_retained_memory(Self::bytes()?).map(Self)
    }

    pub(crate) fn bytes() -> Result<u64, WorkCeilingDenial> {
        let overflow = || WorkCeilingDenial::Admission(LeaseDenial::ChargedBytesOverflow);
        let arc = |payload: Layout| {
            Layout::new::<[usize; 2]>().extend(payload)
                .map(|(layout, _)| layout.pad_to_align().size()).map_err(|_| overflow())
        };
        let digest_len = "bridge-async-lowering:sha256:".len()
            .checked_add(<Sha256 as Digest>::output_size().checked_mul(2).ok_or_else(overflow)?)
            .ok_or_else(overflow)?;
        let digest = arc(Layout::array::<u8>(digest_len).map_err(|_| overflow())?)?;
        let bytes = arc(Layout::new::<LoweringBodyStorage>())?
            .checked_add(digest).ok_or_else(overflow)?;
        u64::try_from(bytes).map_err(|_| overflow())
    }
}

pub(super) struct LoweringBodyStorage {
    pub(super) body: LoweredBridgeAsyncSourceDeclarationBody,
    pub(super) template_digest: Arc<str>,
    _memory: Option<ExecutionMemoryReservation>,
}

impl LoweringBodyStorage {
    pub(super) fn retain(
        body: LoweredBridgeAsyncSourceDeclarationBody,
        template_digest: Arc<str>,
        storage: Option<OriginalLoweringStorage>,
    ) -> Arc<Self> {
        Arc::new(Self { body, template_digest, _memory: storage.and_then(|held| held.0) })
    }
}
impl std::fmt::Debug for LoweringBodyStorage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.body.fmt(f) }
}
impl PartialEq for LoweringBodyStorage {
    fn eq(&self, other: &Self) -> bool { self.body == other.body }
}
impl Eq for LoweringBodyStorage {}
''')
# Private runtime owner mount, not a public factory or proof.
p=b+'mod.rs';s=read(p).replace('pub use lowering::{','pub(crate) use lowering::OriginalLoweringStorage;\npub use lowering::{');write(p,s)
p='crates/worth-runtime-bridge/src/source/mod.rs';s=read(p).replace('pub(crate) use async_declaration::validate_lineage_runtime;','pub(crate) use async_declaration::{validate_lineage_runtime, OriginalLoweringStorage};');write(p,s)
p='crates/worth-runtime-bridge/src/execution_basis/managed_declaration.rs';s=read(p)
s=s.replace('    LoweredBridgeAsyncSourceDeclaration, ValidatedBridgeAsyncSourceDeclaration,','    LoweredBridgeAsyncSourceDeclaration, OriginalLoweringStorage, ValidatedBridgeAsyncSourceDeclaration,')
s=s.replace('    deadline_nanos: Option<u64>,\n)', '    deadline_nanos: Option<u64>,\n    storage: OriginalLoweringStorage,\n)',1)
s=s.replace('    instance_identity: &str,\n)', '    instance_identity: &str,\n    storage: OriginalLoweringStorage,\n)',1)
s=s.replace('        "bridge-managed-domain-execution-legacy-v1",','        "bridge-managed-domain-execution-legacy-v1",\n        storage,')
s=s.replace('        "bridge-atomic-domain-execution-legacy-v1",','        "bridge-atomic-domain-execution-legacy-v1",\n        storage,')
s=s.replace('    legacy_identity: &str,\n)', '    legacy_identity: &str,\n    storage: OriginalLoweringStorage,\n)')
s=s.replace('LoweredBridgeAsyncSourceDeclaration::lower(&validated)?','LoweredBridgeAsyncSourceDeclaration::lower_original(&validated, storage)?')
write(p,s)
p='crates/worth-runtime-bridge/src/execution_basis/admission.rs';s=read(p)
s=s.replace('    counters.materialized_truth();','''    let storage = crate::source::OriginalLoweringStorage::reserve(execution).map_err(|cause| {
        denial(BridgeExecutionBasisDenialKind::ExecutionDenied(cause.into()),
            format!("original request refused immutable lowering storage: {cause:?}"), &counters)
    })?;
    counters.materialized_truth();''',1)
s=s.replace('managed_execution_declaration(intent.identity().as_str(), contract.deadline_nanos())','managed_execution_declaration(intent.identity().as_str(), contract.deadline_nanos(), storage)')
s=s.replace('atomic_execution_declaration(intent.identity().as_str())','atomic_execution_declaration(intent.identity().as_str(), storage)')
write(p,s)
p='crates/worth-runtime-bridge/src/execution_basis/readmission/admission.rs';s=read(p)
s=s.replace('use crate::execution_basis::managed_declaration::managed_execution_declaration;\n','')
s=s.replace('''    let lowered = match managed_execution_declaration(
        fresh_intent.identity().as_str(),
        yielded.step_contract().deadline_nanos(),
    ) {''','''    let lowered = match yielded.basis.request.lowered()
        .instantiate_request_response(fresh_intent.identity().as_str()) {''')
write(p,s)
# Existing prepaid partition backing, unique ownership unchanged.
p='crates/worth-signal/src/branch/owner_services/conditional_execution/execution.rs';replace(p,'crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition,','Box<crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition>,')
for p in ['crates/worth-signal/src/branch/owner_services/conditional_execution/execution_kernel.rs','crates/worth-signal/src/branch/owner_services/conditional_execution/evaluation_reuse/readmission.rs']:
 replace(p,'SignalConditionalExecutionSlot { partition }','SignalConditionalExecutionSlot { partition: Box::new(partition) }')
p='crates/worth-signal/src/branch/owner_services/conditional_execution/execution_inline_type_budget.rs';s=read(p)
s=s.replace('            128 * 1024,','            64,',1)
s=s.replace('.field::<crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition>(','.field::<Box<crate::data::graph::storage::evaluation_partition::SignalEvaluationPartition>>(')
write(p,s)
p=b+'mod_inline_type_budget.rs';s=read(p).replace('    super::request_identity::inline_type_budget::collect(census);','''    super::request_identity::inline_type_budget::collect(census);
    let _owner = census.owner::<super::LoweredBridgeAsyncSourceDeclaration>(
        "crates/worth-runtime-bridge/src/source/async_declaration/lowering.rs",
        "LoweredBridgeAsyncSourceDeclaration", 256,
    );''');write(p,s)
print('Representation/caller batch applied; semantic tests still to add before compilation.')
