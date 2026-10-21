//! Native unwatched source rows for functional marking controls.
use super::*;
use crate::domain_computation::primary_graph::tests::fixture::{
    Account, AccountIdentity, AccountLabel,
};
use crate::domain_computation::primary_graph::WorthQueryPrimaryGraphLayout;
use worth_relational::facade::identity::PartitionId;
use worth_relational::facade::symbols::ClientKey;
use worth_relational::facade::transactions::{CreateIntent, EntitySpec};
pub(super) fn unwatched_accounts(
    layout: &WorthQueryPrimaryGraphLayout,
    count: u64,
    name: &'static str,
) -> WorkerIntentBatch {
    let kind = layout.entity_kind(Account::reference().name()).unwrap();
    let locator =
        |entity, aspect, field| layout.field_locator(entity, aspect, field).unwrap().clone();
    let identity_ref = AccountIdentity::reference();
    let status_ref = AccountStatus::reference();
    let label_ref = AccountLabel::reference();
    let identity = locator(
        identity_ref.entity(),
        identity_ref.aspect(),
        identity_ref.field(),
    );
    let status = locator(status_ref.entity(), status_ref.aspect(), status_ref.field());
    let label = locator(label_ref.entity(), label_ref.aspect(), label_ref.field());
    (0..count).fold(WorkerIntentBatch::new(name), |batch, ordinal| {
        let value = |text: String| AspectValue::String(InternedString::Raw(text));
        batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
            partition_id: PartitionId::main(),
            kind_id: kind,
            client_key: ClientKey::raw(format!("{name}-{ordinal}")),
            fields: AspectFieldPatch::from(BTreeMap::from([
                (identity.clone(), value(format!("{name}-{ordinal}"))),
                (status.clone(), value("neutral".to_owned())),
                (label.clone(), value("neutral".to_owned())),
            ])),
        })))
    })
}
