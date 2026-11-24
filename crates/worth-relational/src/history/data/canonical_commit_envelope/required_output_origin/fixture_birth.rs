//! Assemble descriptive birth input for the branch owner's readmission tests.
use super::{
    CanonicalOutputMember, CanonicalOutputPublication, CanonicalRequiredOutputOrigin,
    CanonicalRequiredOutputRelation,
};
use crate::branch::basis_descriptor::RelationalBranchBasisDescriptor;
use crate::history::data::RelationalCommitReceipt;
use crate::identity::data::{EntityId, KindId};
use crate::mvcc::publication::application_publication::output_meaning::{
    native_meaning_fixture, RequiredOutputRolePosture,
};
use worth_foundational::CanonicalDigestId;

pub(in crate::history::data) fn describe_birth(
    commit: RelationalCommitReceipt,
    basis: RelationalBranchBasisDescriptor,
    entity: EntityId,
    relations: Vec<CanonicalRequiredOutputRelation>,
) -> CanonicalRequiredOutputOrigin {
    let producer = "native.trimmed.producer";
    let dependency = CanonicalDigestId::new([19; 32]);
    CanonicalRequiredOutputOrigin {
        inputs: Vec::new(),
        _input_memory: None,
        birth: commit.commit_id,
        birth_producer: producer.into(),
        birth_dependency: dependency,
        members: vec![CanonicalOutputMember {
            role: "output-0".into(),
            entity_name: "native-unit-output".into(),
            id: entity,
            kind: KindId(1),
            posture: RequiredOutputRolePosture::Create,
            birth: commit.commit_id,
            birth_producer: producer.into(),
            birth_dependency: dependency,
            before_write: None,
            after_write: commit.version_id,
        }],
        commit,
        basis,
        producer: producer.into(),
        source_scope_owned: false,
        dependency_identity: dependency,
        outputs: vec![entity],
        relations,
        meaning: native_meaning_fixture("native.trimmed.producer"),
        publication: CanonicalOutputPublication::Birth,
        issuer: None,
        _funding: None,
        _cascade_memory: None,
    }
}
