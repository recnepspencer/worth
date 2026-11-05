//! Each case carries its owner's enum injection and checked row decoder.

use super::super::{Aspect, OneOf, Record};
use sha2::{Digest, Sha256};
use std::any::TypeId;

#[derive(Clone, Debug)]
pub struct CaseDefinition {
    marker: TypeId,
    name: &'static str,
    row: &'static str,
    bindings: &'static str,
}

impl PartialEq for CaseDefinition {
    fn eq(&self, other: &Self) -> bool {
        self.marker == other.marker
            && self.name == other.name
            && self.row == other.row
            && self.bindings == other.bindings
    }
}
impl Eq for CaseDefinition {}

impl CaseDefinition {
    pub(super) fn of<Set: Record<A::Row>, A: Aspect>() -> Self {
        Self {
            marker: TypeId::of::<A>(),
            name: A::NAME,
            row: std::any::type_name::<A::Row>(),
            bindings: std::any::type_name::<Set>(),
        }
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GroupDefinition {
    marker: TypeId,
    name: &'static str,
    row: TypeId,
    optional: bool,
    cases: Vec<CaseDefinition>,
    contract: [u8; 32],
}

impl GroupDefinition {
    pub(super) fn of<G: OneOf>(mut cases: Vec<CaseDefinition>, optional: bool) -> Self {
        cases.sort_by_key(|case| case.name);
        let mut hash = Sha256::new();
        for part in [
            "worth.one-of.v1",
            std::any::type_name::<G>(),
            std::any::type_name::<G::Row>(),
        ] {
            hash.update((part.len() as u64).to_le_bytes());
            hash.update(part.as_bytes());
        }
        hash.update([u8::from(optional)]);
        for case in &cases {
            for part in [case.name, case.row, case.bindings] {
                hash.update((part.len() as u64).to_le_bytes());
                hash.update(part.as_bytes());
            }
        }
        Self {
            marker: TypeId::of::<G>(),
            name: std::any::type_name::<G>(),
            row: TypeId::of::<G::Row>(),
            optional,
            cases,
            contract: hash.finalize().into(),
        }
    }
    pub fn marker(&self) -> TypeId {
        self.marker
    }
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn optional(&self) -> bool {
        self.optional
    }
    pub fn cases(&self) -> &[CaseDefinition] {
        &self.cases
    }
    pub fn contains<A: Aspect>(&self) -> bool {
        self.cases
            .iter()
            .any(|case| case.marker == TypeId::of::<A>())
    }
    pub(crate) fn contract(&self) -> &[u8; 32] {
        &self.contract
    }
}
