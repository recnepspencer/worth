//! Read and write loans expose only their original Query phase's verbs.

use super::super::WorthQueryInvariantMutationTarget;
use super::{ErasedCandidate, ErasedObserved, FragmentDenial};
use std::collections::BTreeMap;
use worth_foundational::facade::AspectValue;
use worth_query_declaration::facade::application_schema::fragment::{
    AspectDefinition, GroupDefinition, RelationDefinition,
};

pub struct Reading<'a> {
    pub(super) port: &'a mut dyn ReadPort,
}
pub struct Writing<'a> {
    pub(super) port: &'a mut dyn WritePort,
}

pub(in crate::domain_computation::primary_graph) trait ReadPort {
    fn with_decode_admission(
        &mut self,
        path: &str,
        decode: &mut dyn FnMut(
            &mut worth_query_declaration::facade::application_schema::ValueDecodeAdmission<'_>,
        ) -> Result<(), FragmentDenial>,
    ) -> Result<(), FragmentDenial>;
    fn resolve(
        &mut self,
        field: &worth_query_declaration::facade::application_schema::ApplicationFieldBindingLocus,
        value: AspectValue,
    ) -> Result<ErasedObserved, FragmentDenial>;
    fn record(
        &mut self,
        target: &ErasedObserved,
        aspect: &AspectDefinition,
    ) -> Result<Option<BTreeMap<String, AspectValue>>, FragmentDenial>;
    fn target(
        &self,
        target: &ErasedObserved,
    ) -> Result<WorthQueryInvariantMutationTarget<(), ()>, FragmentDenial>;
}
pub(in crate::domain_computation::primary_graph) trait WritePort {
    fn case_record(
        &mut self,
        target: &ErasedCandidate,
        aspect: &AspectDefinition,
        group: &GroupDefinition,
        cases: &[&AspectDefinition],
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial>;
    fn target(
        &self,
        target: &WorthQueryInvariantMutationTarget<(), ()>,
    ) -> Result<ErasedCandidate, FragmentDenial>;
    fn create(&mut self, entity: &str, key: &str) -> Result<ErasedCandidate, FragmentDenial>;
    fn record(
        &mut self,
        target: &ErasedCandidate,
        aspect: &AspectDefinition,
        fields: BTreeMap<String, AspectValue>,
    ) -> Result<(), FragmentDenial>;
    fn link(
        &mut self,
        from: &ErasedCandidate,
        relation: &RelationDefinition,
        to: &ErasedCandidate,
    ) -> Result<(), FragmentDenial>;
}
