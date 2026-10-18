//! Serialize native lineage without constructing a boxed durable copy.
use super::*;
use serde::{Serialize, Serializer};

pub(in crate::domain_computation) struct DurableLineageView<'a>(
    &'a WorthQueryAuthorizationDecisionFact,
);

#[derive(Serialize)]
enum Lineage<'a> {
    Unbound,
    Root {
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
    },
    Delegated {
        grantor: worth_relational::facade::identity::EntityId,
        parent_grant: worth_relational::facade::identity::EntityId,
        status_revision: worth_relational::facade::runtime::RelationalFieldRevision,
        parent: DurableLineageView<'a>,
    },
}

impl WorthQueryAuthorizationDecisionFact {
    pub(in crate::domain_computation) fn durable_lineage_view(&self) -> DurableLineageView<'_> {
        DurableLineageView(self)
    }
}

impl Serialize for DurableLineageView<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use delegation::WorthQueryDelegationDecisionFact as Native;
        match self.0.delegation.as_deref() {
            None => Lineage::Unbound,
            Some(Native::Root {
                status_revision, ..
            }) => Lineage::Root {
                status_revision: *status_revision,
            },
            Some(Native::Delegated {
                grantor,
                parent_grant,
                status_revision,
                parent,
                ..
            }) => Lineage::Delegated {
                grantor: *grantor,
                parent_grant: *parent_grant,
                status_revision: *status_revision,
                parent: DurableLineageView(parent),
            },
        }
        .serialize(serializer)
    }
}

impl WorthQueryDurableCapabilityLineage {
    pub(in crate::domain_computation) fn matches_view(
        &self,
        view: &DurableLineageView<'_>,
    ) -> bool {
        use delegation::WorthQueryDelegationDecisionFact as Native;
        match (self, view.0.delegation.as_deref()) {
            (Self::Unbound, None) => true,
            (
                Self::Root {
                    status_revision: left,
                },
                Some(Native::Root {
                    status_revision: right,
                    ..
                }),
            ) => left == right,
            (
                Self::Delegated {
                    grantor: lg,
                    parent_grant: lp,
                    status_revision: ls,
                    parent: left,
                },
                Some(Native::Delegated {
                    grantor: rg,
                    parent_grant: rp,
                    status_revision: rs,
                    parent: right,
                    ..
                }),
            ) => lg == rg && lp == rp && ls == rs && left.matches_view(&DurableLineageView(right)),
            _ => false,
        }
    }
}
