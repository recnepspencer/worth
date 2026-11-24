//! Allocation inventory through the same serde MessagePack vocabulary.
//! No path/element Vec or owned locator String is constructed by this pass.
use super::super::storage::StorageQuote;
use super::super::*;
use serde::{
    de::{IgnoredAny, SeqAccess, Visitor},
    Deserialize, Deserializer,
};
use std::{fmt, marker::PhantomData};

#[derive(Deserialize)]
pub(super) struct Inventory {
    pub(super) version: u8,
    #[serde(rename = "principal")]
    _principal: IgnoredAny,
    #[serde(rename = "scope")]
    _scope: IgnoredAny,
    paths: Paths,
}
impl Inventory {
    pub(super) fn quote(self) -> Result<StorageQuote, RelationalAuthorizationDependencyDenial> {
        let mut quote = self.paths.0?;
        quote.add_terms(2)?;
        Ok(quote)
    }
}

struct Count<T>(usize, PhantomData<T>);
impl<'de, T> Deserialize<'de> for Count<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct Counter<T>(PhantomData<T>);
        impl<'de, T> Visitor<'de> for Counter<T> {
            type Value = Count<T>;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("native dependency array")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut count = 0;
                while seq.next_element::<IgnoredAny>()?.is_some() {
                    count += 1;
                }
                Ok(Count(count, PhantomData))
            }
        }
        deserializer.deserialize_seq(Counter(PhantomData))
    }
}

#[derive(Deserialize)]
struct Path {
    #[serde(rename = "matched")]
    _matched: IgnoredAny,
    witness: Option<Count<EntityId>>,
    entities: Count<EntityDependency>,
    relations: Count<RelationDependency>,
    adjacencies: Count<AdjacencyDependency>,
    fields: Fields,
}
impl Path {
    fn quote(self) -> Result<StorageQuote, RelationalAuthorizationDependencyDenial> {
        let mut quote = StorageQuote::default();
        quote.add_vector::<EntityId>(self.witness.map_or(0, |count| count.0))?;
        quote.add_vector::<EntityDependency>(self.entities.0)?;
        quote.add_vector::<RelationDependency>(self.relations.0)?;
        quote.add_vector::<AdjacencyDependency>(self.adjacencies.0)?;
        quote.add_vector::<FieldDependency>(self.fields.count)?;
        quote.add_bytes(self.fields.text)?;
        Ok(quote)
    }
}

struct Paths(Result<StorageQuote, RelationalAuthorizationDependencyDenial>);
impl<'de> Deserialize<'de> for Paths {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct PathVisitor;
        impl<'de> Visitor<'de> for PathVisitor {
            type Value = Paths;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("native authorization paths")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Paths, A::Error> {
                let mut quote = Ok(StorageQuote::default());
                let mut count = 0;
                while let Some(path) = seq.next_element::<Path>()? {
                    count += 1;
                    quote = quote.and_then(|mut quote| {
                        quote.merge(path.quote()?)?;
                        Ok(quote)
                    });
                }
                Ok(Paths(quote.and_then(|mut quote| {
                    quote.add_vector::<PathDependencies>(count)?;
                    Ok(quote)
                })))
            }
        }
        deserializer.deserialize_seq(PathVisitor)
    }
}

struct FieldBytes(usize);
impl<'de> Deserialize<'de> for FieldBytes {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        struct Field<'a> {
            #[serde(rename = "entity")]
            _entity: IgnoredAny,
            #[serde(rename = "locator_authority")]
            _authority: IgnoredAny,
            #[serde(borrow)]
            aspect: &'a str,
            #[serde(borrow)]
            field: &'a str,
            #[serde(rename = "revision")]
            _revision: IgnoredAny,
        }
        let field = Field::deserialize(deserializer)?;
        Ok(Self(field.aspect.len() + field.field.len()))
    }
}
struct Fields {
    count: usize,
    text: usize,
}
impl<'de> Deserialize<'de> for Fields {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct FieldVisitor;
        impl<'de> Visitor<'de> for FieldVisitor {
            type Value = Fields;
            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("native authorization fields")
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Fields, A::Error> {
                let mut fields = Fields { count: 0, text: 0 };
                while let Some(field) = seq.next_element::<FieldBytes>()? {
                    fields.count += 1;
                    fields.text += field.0;
                }
                Ok(fields)
            }
        }
        deserializer.deserialize_seq(FieldVisitor)
    }
}
