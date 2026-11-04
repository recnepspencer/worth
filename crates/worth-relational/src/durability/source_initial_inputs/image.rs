//! The finite original birth/install chronology has its own recorded coordinates.
use super::{frame::*, recorded::RecordedEmptySchemaObservation};
use crate::durability::data::DurableBranchRootSchemaImage;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) enum InitialInputEntry {
    Birth(RecordedEmptySchemaObservation),
    Installation {
        before: RecordedEmptySchemaObservation,
        after: RecordedEmptySchemaObservation,
        follows_transition: crate::branch::RelationalBranchTransitionSequence,
    },
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InitialInputsImage {
    pub(crate) main: crate::history::data::BranchId,
    pub(crate) revision: u64,
    pub(crate) predecessor: Option<[u8; 32]>,
    #[serde(deserialize_with = "entries")]
    pub(crate) entries: Vec<InitialInputEntry>,
    #[serde(deserialize_with = "schemas")]
    pub(crate) schemas: Vec<DurableBranchRootSchemaImage>,
    #[serde(skip)]
    pub(crate) artifact: Option<RelationalSourceInitialInputsArtifact>,
    #[serde(skip)]
    pub(crate) _memory: Option<worth_execution::ExecutionMemoryReservation>,
}

#[derive(Serialize)]
pub(crate) struct InitialInputsImageRef<'a> {
    pub(crate) main: &'a crate::history::data::BranchId,
    pub(crate) revision: u64,
    pub(crate) predecessor: Option<[u8; 32]>,
    pub(crate) entries: &'a [InitialInputEntry],
    pub(crate) schemas: &'a [DurableBranchRootSchemaImage],
}

impl InitialInputsImage {
    pub(crate) fn validate(&self) -> Result<(), RelationalInitialInputsRefusal> {
        use RelationalInitialInputsRefusal as Refusal;
        let Some(InitialInputEntry::Birth(birth)) = self.entries.first() else {
            return Err(Refusal::Malformed);
        };
        if birth.generation != worth_foundational::FoundationalBranchReferenceGeneration::initial()
            || birth.truth_version != crate::branch::RelationalBranchVersion::initial()
            || birth.lifecycle != crate::branch::RelationalBranchLifecyclePosture::Live
        {
            return Err(Refusal::Malformed);
        }
        let valid = match self.entries.as_slice() {
            [InitialInputEntry::Birth(_)] => self.revision == 1 && self.predecessor.is_none(),
            [InitialInputEntry::Birth(_), InitialInputEntry::Installation { before, after, .. }] => {
                (self.revision == 1 && self.predecessor.is_none()
                    || self.revision == 2 && self.predecessor.is_some())
                    && before.truth_version == after.truth_version
                    && before.truth_version == birth.truth_version
                    && before.schema_carrier == birth.schema_carrier
                    && before.lifecycle == after.lifecycle
                    && before
                        .generation
                        .checked_advance()
                        .is_ok_and(|generation| generation == after.generation)
            }
            [] | [InitialInputEntry::Installation { .. }] | [_, _] => false,
            [_, _, _, ..] => false,
        };
        if !valid {
            return Err(Refusal::Malformed);
        }
        let mut previous = None;
        for schema in &self.schemas {
            if previous.is_some_and(|digest| digest >= schema.carrier_digest()) {
                return Err(Refusal::Malformed);
            }
            previous = Some(schema.carrier_digest());
        }
        for entry in &self.entries {
            match entry {
                InitialInputEntry::Birth(birth) => self.require_schema(birth.schema_carrier)?,
                InitialInputEntry::Installation { before, after, .. } => {
                    self.require_schema(before.schema_carrier)?;
                    self.require_schema(after.schema_carrier)?;
                }
            }
        }
        Ok(())
    }
    fn require_schema(&self, digest: [u8; 32]) -> Result<(), RelationalInitialInputsRefusal> {
        if self
            .schemas
            .iter()
            .any(|image| image.carrier_digest() == digest)
        {
            Ok(())
        } else {
            Err(RelationalInitialInputsRefusal::Malformed)
        }
    }
}

fn entries<'de, D: serde::Deserializer<'de>>(input: D) -> Result<Vec<InitialInputEntry>, D::Error> {
    bounded(input, 2)
}
fn schemas<'de, D: serde::Deserializer<'de>>(
    input: D,
) -> Result<Vec<DurableBranchRootSchemaImage>, D::Error> {
    bounded(input, 3)
}
fn bounded<'de, D: serde::Deserializer<'de>, T: Deserialize<'de>>(
    input: D,
    maximum: usize,
) -> Result<Vec<T>, D::Error> {
    struct Visitor<T>(usize, std::marker::PhantomData<T>);
    impl<'de, T: Deserialize<'de>> serde::de::Visitor<'de> for Visitor<T> {
        type Value = Vec<T>;
        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a bounded original source roster")
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(
            self,
            mut input: A,
        ) -> Result<Self::Value, A::Error> {
            let count = input
                .size_hint()
                .filter(|count| *count <= self.0)
                .ok_or_else(|| {
                    serde::de::Error::custom("original source roster exceeds its bound")
                })?;
            let mut rows = Vec::with_capacity(count);
            while let Some(row) = input.next_element()? {
                if rows.len() == count {
                    return Err(serde::de::Error::custom(
                        "original source roster length changed",
                    ));
                }
                rows.push(row);
            }
            Ok(rows)
        }
    }
    input.deserialize_seq(Visitor(maximum, std::marker::PhantomData))
}
