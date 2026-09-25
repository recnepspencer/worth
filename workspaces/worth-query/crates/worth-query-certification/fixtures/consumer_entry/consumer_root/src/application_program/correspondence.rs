use worth_query_consumer_values::{PlanarAdjustment, PlanarOperation, PositiveLength};
use worth_query_decl::facade::application_program::ApplicationRepeatedOptionalMemberCorrespondence;
use worth_query_topology_entry::{PlanarEditBinding, PlanarMutation};

use crate::ConsumerSchema;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OptionalAdjustmentRow {
    pub(crate) body_key: String,
    pub(crate) replacement_y: Option<PositiveLength>,
}

pub(crate) struct OptionalAdjustmentCorrespondence;

impl
    ApplicationRepeatedOptionalMemberCorrespondence<
        ConsumerSchema,
        PlanarEditBinding<ConsumerSchema>,
    > for OptionalAdjustmentCorrespondence
{
    type Row = OptionalAdjustmentRow;
    type Target = String;
    type Member = PositiveLength;

    const IDENTITY: &'static str = "worth.query.certification.optional-adjustment.v1";

    fn target(row: &Self::Row) -> Self::Target {
        row.body_key.clone()
    }

    fn initial_member(row: &Self::Row) -> Option<Self::Member> {
        row.replacement_y
    }

    fn input(target: &Self::Target, member: Option<Self::Member>) -> PlanarMutation {
        PlanarMutation {
            scope_key: target.clone(),
            operation: PlanarOperation::Adjust(
                member
                    .map(|replacement_y| PlanarAdjustment {
                        body_key: target.clone(),
                        replacement_y,
                    })
                    .into_iter()
                    .collect(),
            ),
            validator_work: 0,
        }
    }
}
