//! Portable aspect wire views over the original admitted native values.
use super::*;
use worth_execution::ExecutionOrderedMap;
use worth_foundational::facade::{
    AspectContract, AspectContractRevision, AspectIdentity, AspectKey,
    AuthoritativeRecordAspectState, ContractValidatedAspectValue, ContractValidatedAspectValueView,
};
type Basis<'a> = (&'a AspectKey, AspectIdentity, AspectContractRevision);
pub(super) struct Contracts<'a> {
    entries: ExecutionOrderedMap<Basis<'a>, &'a AspectContract>,
}
impl<'a> Contracts<'a> {
    pub(super) fn capture(
        schema: &'a RelationalBranchRootSchemaAuthority,
        request: &ExecutionRequest<'_, '_>,
    ) -> Result<Self, Stop> {
        let mut entries = ExecutionOrderedMap::default();
        for contract in schema.retained_aspect_contracts() {
            request
                .consult()
                .map_err(|cause| Stop::Request(cause.into()))?;
            let key = (contract.key(), contract.identity(), contract.revision());
            if let Some(prior) = entries.get(&key) {
                if *prior != contract {
                    return Err(invalid("original aspect contracts conflict"));
                }
            } else {
                entries
                    .insert_cloned_native_for_request(&key, &contract, *request)
                    .map_err(crate::durability::checkpoint_history::capture::map_stop)?;
            }
        }
        Ok(Self { entries })
    }
    pub(super) fn validate(
        &self,
        state: Option<&AuthoritativeRecordAspectState>,
    ) -> Result<(), Stop> {
        if let Some(state) = state {
            for (_, value) in state.aspects().entries() {
                if self
                    .entries
                    .get(&(
                        value.key(),
                        value.contract_identity(),
                        value.contract_revision(),
                    ))
                    .is_none()
                {
                    return Err(invalid(
                        "original aspect export contract is absent or has drifted",
                    ));
                }
            }
        }
        Ok(())
    }
}
#[derive(Serialize)]
struct BasisView<'a> {
    key: &'a AspectKey,
    identity: AspectIdentity,
    revision: AspectContractRevision,
}
#[derive(Serialize)]
enum Input<'a> {
    Scalar(&'a worth_foundational::facade::AspectValue),
    Struct(&'a worth_foundational::facade::StructAspectValue),
}
struct Value<'a>(&'a ContractValidatedAspectValue);
impl Serialize for Value<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[derive(Serialize)]
        struct Entry<'a> {
            basis: BasisView<'a>,
            value: Input<'a>,
        }
        let value = match self.0.view() {
            ContractValidatedAspectValueView::Scalar(value) => Input::Scalar(value),
            ContractValidatedAspectValueView::Struct(value) => Input::Struct(value),
        };
        Entry {
            basis: BasisView {
                key: self.0.key(),
                identity: self.0.contract_identity(),
                revision: self.0.contract_revision(),
            },
            value,
        }
        .serialize(serializer)
    }
}
pub(super) struct State<'a>(pub(super) &'a AuthoritativeRecordAspectState);
impl Serialize for State<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::{SerializeSeq, SerializeStruct};
        struct Entries<'a>(&'a AuthoritativeRecordAspectState);
        impl Serialize for Entries<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut sequence = serializer.serialize_seq(Some(self.0.aspects().len()))?;
                for (_, value) in self.0.aspects().entries() {
                    sequence.serialize_element(&Value(value))?;
                }
                sequence.end()
            }
        }
        let mut state = serializer.serialize_struct("PortableRecordAspectState", 1)?;
        state.serialize_field("entries", &Entries(self.0))?;
        state.end()
    }
}
