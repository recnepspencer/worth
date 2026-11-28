//! An original schema decode transfers allocation custody to its shared declarations.
use super::aspect_merge_policy::custom_name::MergePolicyNameOwnerPort as _;
use super::aspect_merge_policy::AspectMergePolicyKind;
use super::identity_declarations::custom_name::IdentityBasisNameOwnerPort as _;
use super::identity_declarations::declared_keys::DeclaredIdentityKeysOwnerPort as _;
use super::identity_declarations::IdentityBasisKind;
use worth_execution::ExecutionMemoryReservation;

pub struct SchemaDeclarationLeafMemory {
    count: usize,
    bytes: u64,
    memory: Option<ExecutionMemoryReservation>,
}
impl SchemaDeclarationLeafMemory {
    pub fn new(count: usize, bytes: u64, memory: Option<ExecutionMemoryReservation>) -> Self {
        Self {
            count,
            bytes,
            memory,
        }
    }
    pub fn install(
        &mut self,
        count: usize,
        install: impl FnOnce(
            Option<ExecutionMemoryReservation>,
        ) -> Result<(), Option<ExecutionMemoryReservation>>,
    ) -> Result<(), &'static str> {
        if count != self.count
            || self
                .memory
                .as_ref()
                .is_some_and(|held| held.bytes() != self.bytes)
        {
            return Err("schema declaration admission does not match its original decoded backing");
        }
        // Only this one-decode private handoff can initialize the cells. Any
        // legitimate internal alias shares an already installed original grant;
        // its redundant, unused admission can be released without losing data.
        if let Err(unused) = install(self.memory.take()) {
            drop(unused);
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct SchemaIdentityDeclarationMemory {
    pub keys: Option<SchemaDeclarationLeafMemory>,
    pub custom_name: Option<SchemaDeclarationLeafMemory>,
}
impl SchemaIdentityDeclarationMemory {
    pub fn attach(&mut self, basis: &IdentityBasisKind) -> Result<(), &'static str> {
        match basis {
            IdentityBasisKind::DeclaredKeySet(keys) if self.custom_name.is_none() => self
                .keys
                .as_mut()
                .ok_or("missing identity key-set admission")?
                .install(keys.len(), |memory| keys.retain_decoded_memory(memory)),
            IdentityBasisKind::Custom(custom) if self.keys.is_none() => self
                .custom_name
                .as_mut()
                .ok_or("missing identity name admission")?
                .install(custom.name.len(), |memory| {
                    custom.name.retain_decoded_memory(memory)
                }),
            IdentityBasisKind::StorageIdentity
            | IdentityBasisKind::LineageIdentity
            | IdentityBasisKind::StructuralFingerprint
                if self.keys.is_none() && self.custom_name.is_none() =>
            {
                Ok(())
            }
            _ => Err("schema identity admission kind does not match decoded meaning"),
        }
    }
}

pub fn attach_policy_name(
    memory: &mut Option<SchemaDeclarationLeafMemory>,
    policy: &AspectMergePolicyKind,
) -> Result<(), &'static str> {
    match (memory.as_mut(), policy) {
        (Some(memory), AspectMergePolicyKind::Custom(custom)) => memory
            .install(custom.name.len(), |memory| {
                custom.name.retain_decoded_memory(memory)
            }),
        (None, AspectMergePolicyKind::Custom(_)) | (Some(_), _) => {
            Err("merge policy name admission mismatch")
        }
        (None, _) => Ok(()),
    }
}
