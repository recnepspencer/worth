//! Names choose identity; a collision never silently identifies two meanings.

use super::super::ApplicationSchemaDeclarationDenial as Denial;
use std::collections::BTreeMap;

pub const fn name_identity(name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    let mut index = 0;
    while index < bytes.len() {
        hash = (hash ^ bytes[index] as u64).wrapping_mul(0x100_0000_01b3);
        index += 1;
    }
    hash
}

/// Physical keys have a fixed, const-derived spelling. Selectors can borrow
/// them without allocating while the catalog retains the full logical path.
pub(super) const fn native_aspect_identity(entity: &str, aspect: &str) -> u64 {
    let mut hash = name_identity(entity);
    hash = hash.wrapping_mul(0x100_0000_01b3);
    let bytes = aspect.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        hash = (hash ^ bytes[index] as u64).wrapping_mul(0x100_0000_01b3);
        index += 1;
    }
    hash
}
const fn append_name(mut hash: u64, name: &str) -> u64 {
    let bytes = name.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        hash = (hash ^ bytes[index] as u64).wrapping_mul(0x100_0000_01b3);
        index += 1;
    }
    hash
}
pub(super) const fn native_relation_aspect_identity(relation: &str, aspect: &str) -> u64 {
    let owner = append_name(
        name_identity("relation").wrapping_mul(0x100_0000_01b3),
        relation,
    );
    append_name(owner.wrapping_mul(0x100_0000_01b3), aspect)
}
struct NativeRelationAspectName<R, A>(std::marker::PhantomData<fn() -> (R, A)>);
impl<R: super::Relation, A: super::Aspect> NativeRelationAspectName<R, A> {
    const BYTES: [u8; 17] = encoded_aspect_name(native_relation_aspect_identity(R::NAME, A::NAME));
    const TEXT: &'static str = match std::str::from_utf8(&Self::BYTES) {
        Ok(value) => value,
        Err(_) => panic!("the SDK key alphabet is ASCII"),
    };
}
pub(super) fn native_relation_aspect_key<R: super::Relation, A: super::Aspect>() -> &'static str {
    NativeRelationAspectName::<R, A>::TEXT
}
const fn encoded_aspect_name(identity: u64) -> [u8; 17] {
    let digits = b"0123456789abcdef";
    let mut output = [b'a'; 17];
    let mut index = 0;
    while index < 16 {
        output[index + 1] = digits[((identity >> (4 * (15 - index))) & 15) as usize];
        index += 1;
    }
    output
}
pub fn native_aspect_name(entity: &str, aspect: &str) -> String {
    String::from_utf8(encoded_aspect_name(native_aspect_identity(entity, aspect)).to_vec())
        .expect("the SDK key alphabet is ASCII")
}
struct NativeAspectName<E, A>(std::marker::PhantomData<fn() -> (E, A)>);
impl<E: super::Entity, A: super::Aspect> NativeAspectName<E, A> {
    const BYTES: [u8; 17] = encoded_aspect_name(native_aspect_identity(E::NAME, A::NAME));
    const TEXT: &'static str = match std::str::from_utf8(&Self::BYTES) {
        Ok(value) => value,
        Err(_) => panic!("the SDK key alphabet is ASCII"),
    };
}
pub(super) fn native_aspect_key<E: super::Entity, A: super::Aspect>() -> &'static str {
    NativeAspectName::<E, A>::TEXT
}

#[derive(Default)]
pub(super) struct Names(BTreeMap<u64, String>);

impl Names {
    pub(super) fn insert(&mut self, name: &str) -> Result<u64, Denial> {
        let identity = name_identity(name);
        if let Some(first) = self.0.get(&identity) {
            return Err(if first == name {
                Denial::DuplicateFragmentMember {
                    path: name.to_owned(),
                }
            } else {
                Denial::FragmentIdentityCollision {
                    identity,
                    first: first.clone(),
                    second: name.to_owned(),
                }
            });
        }
        self.0.insert(identity, name.to_owned());
        Ok(identity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_collision_refuses_instead_of_reusing_an_identity() {
        let mut names = Names::default();
        names
            .0
            .insert(name_identity("fixture.item"), "another.meaning".to_owned());
        assert!(
            matches!(names.insert("fixture.item"), Err(Denial::FragmentIdentityCollision { first, second, .. }) if first == "another.meaning" && second == "fixture.item")
        );
    }
}
