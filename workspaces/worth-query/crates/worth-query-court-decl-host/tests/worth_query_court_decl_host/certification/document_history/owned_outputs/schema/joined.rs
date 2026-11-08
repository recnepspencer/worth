//! A distinct publication relation keeps join outputs outside its input traversal.
use super::*;
pub struct JoinedOf;
impl Relation for JoinedOf {
    const NAME: &'static str = "JoinedOf";
    type To = Member;
}
impl JoinedOf {
    pub fn reference() -> ApplicationRelationRef<Schema, Self, FirstOutput, Member> {
        relation(Storage, FirstOutput, Self)
    }
}
