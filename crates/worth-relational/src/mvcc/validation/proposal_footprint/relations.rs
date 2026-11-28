//! Relation lifecycle material comes from the validated actor's real applied draft.
use worth_relational_branch::internal_port::history::data::canonical_commit_envelope::CanonicalCommitEnvelopeBodyOwnerPort as _;
use worth_relational_publication::internal_port::publication::ownership::authored_restoration::RelationalAuthoredRestorationFieldsPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_owners::RelationalAuthoredCoupledOwnersFieldsPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::completion::RequiredOutputPublicationOwnerPort as _;
use worth_relational_publication::internal_port::mvcc::publication::application_publication::coupled_assembly::completion::RelationalCoupledAuthoredPublicationFieldsPort as _;
use super::member_restoration::RecordedMemberRestoration;
use super::relation_mutations::ValidatedRelationMutationFootprint;
use crate::identity::data::KindId;
use crate::mvcc::validation::validated_proposal::ValidatedRelationalProposal;
use crate::transactions::data::outcomes::transaction_commit_error::TransactionCommitError;
use worth_execution::ExecutionRequest;

impl ValidatedRelationalProposal {
    /// Optional relation projection. The field-only footprint does no additional work.
    ///
    /// `members` are the recorded members an inverse must restore under owners
    /// that stay live, and `member_kinds` the entity kinds its declared member
    /// families own: every birth of such a kind must be one recorded member.
    /// The member pass runs only when either is non-empty. It costs one
    /// correspondence probe per birth across the recorded members and their
    /// retained fields, then one owner-edge probe per recorded member; it
    /// never reads a roster or a thread.
    pub fn relation_mutation_footprint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kinds: &[KindId],
        members: &[RecordedMemberRestoration<'_>],
        member_kinds: &[KindId],
    ) -> Result<ValidatedRelationMutationFootprint, TransactionCommitError> {
        self.project_relation_mutations(execution, kinds, members, member_kinds, false)
    }
    /// The actual root writer's inverse material, excluding every later stage
    /// and producer. The range is Native-issued, never a caller-supplied subset.
    /// An uncoupled proposal is its whole root; a genuine empty root stays empty.
    pub fn authored_prefix_relation_mutation_footprint(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kinds: &[KindId],
        members: &[RecordedMemberRestoration<'_>],
        member_kinds: &[KindId],
    ) -> Result<ValidatedRelationMutationFootprint, TransactionCommitError> {
        self.project_relation_mutations(execution, kinds, members, member_kinds, true)
    }
    fn project_relation_mutations(
        &self,
        execution: &ExecutionRequest<'_, '_>,
        kinds: &[KindId],
        members: &[RecordedMemberRestoration<'_>],
        member_kinds: &[KindId],
        prefix_only: bool,
    ) -> Result<ValidatedRelationMutationFootprint, TransactionCommitError> {
        let prefix_records = if prefix_only {
            match self
                .state
                .required_output_publication
                .as_ref()
                .map(|a| &a.publication)
            {
                Some(crate::mvcc::RequiredOutputPublication::CoupledAuthored { group }) => Some(
                    group
                        .authored_records()
                        .get(..group.authored_prefix_records())
                        .ok_or_else(|| {
                            crate::transactions::data::outcomes::transaction_commit_error::origin_denial("coupled authored root range unavailable")
                        })?,
                ),
                _ => None,
            }
        } else {
            None
        };
        let restoration = self
            .state
            .required_output_publication
            .as_ref()
            .and_then(|admission| admission.publication.authored_restoration());
        let corrective_root = if prefix_only {
            self.state
                .required_output_publication
                .as_ref()
                .and_then(|admission| match &admission.publication {
                    crate::mvcc::RequiredOutputPublication::CoupledAuthored { group } => group
                        .binding()
                        .owners
                        .corrective_root()
                        .as_ref()
                        .map(|(_, targets)| targets.elements()),
                    _ => None,
                })
        } else {
            None
        };
        let anchors = corrective_root.map_or_else(
            || {
                restoration.map_or(0, |restoration| {
                    restoration
                        .retirement()
                        .authored_source_retirement()
                        .expect("Native restoration")
                        .co_anchors
                        .len()
                        + 1
                })
            },
            |targets| targets.len(),
        );
        // Both routes enter the same allocation resolution and root-membership
        // projection. The binder already refused conflict with a real plan.
        let anchor_targets =
            corrective_root.or_else(|| restoration.map(|plan| plan.entities().as_slice()));
        let outcome = &self.state.proposed_mutation.outcome;
        let authored_records =
            self.state
                .required_output_publication
                .as_ref()
                .and_then(|admission| match &admission.publication {
                    crate::mvcc::RequiredOutputPublication::CoupledAuthored { group } => {
                        Some(group.authored_records().as_slice())
                    }
                    _ => None,
                });
        super::relation_projection::project_relations(
            execution,
            super::relation_projection::RelationProjectionInputs {
                before: &self.state.prepared.selected_branch_state,
                after: &self.state.proposed_mutation.state,
                proposed_version: self.state.evidence.proposed_version(),
                fragments: &outcome.effect.publication.patch_fragments,
                created_entities: &outcome.created_entities,
                prefix_records,
                authored_records,
                anchors,
                anchor_targets,
            },
            kinds,
            members,
            member_kinds,
        )
    }
}
