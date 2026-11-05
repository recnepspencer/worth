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
