//! Descriptive canonical fields use the real publication coordinates, never a World mint.
use super::*;
use std::collections::BTreeMap;
use worth_foundational::facade::WorldOwnerCoordinate;
use worth_relational::facade::identity::PartitionId;
#[derive(Clone)]
pub(super) struct Row {
    pub parent: Published,
    pub source: Published,
    pub root: Published,
    pub selected: Option<Published>,
    pub active: Option<Published>,
    pub prior: Option<Published>,
}
impl Row {
    pub(super) fn reverse(
        parent: &Published,
        source: &Published,
        root: &Published,
        prior: Option<&Published>,
    ) -> Self {
        Self {
            parent: parent.clone(),
            source: source.clone(),
            root: root.clone(),
            selected: None,
            active: None,
            prior: prior.cloned(),
        }
    }
    pub(super) fn reapply(
        parent: &Published,
        source: &Published,
        root: &Published,
        selected: &Published,
        active: &Published,
    ) -> Self {
        Self {
            parent: parent.clone(),
            source: source.clone(),
            root: root.clone(),
            selected: Some(selected.clone()),
            active: Some(active.clone()),
            prior: Some(active.clone()),
        }
    }
}
fn text(value: impl Into<String>) -> AspectValue {
    AspectValue::String(value.into().into())
}
fn selector(value: &Published) -> AspectValue {
    let owner = value.world.owner_identity().durable().unwrap();
    let home = owner
        .home()
        .bytes()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<String>();
    text(format!(
        "home-v1:{home}:{}:{}:{}:{}:{}",
        owner.slot().get(),
        value.world.ordinal(),
        value.native.branch_id.0.len(),
        value.native.branch_id.0,
        value.native.commit_id.0
    ))
}
fn fields(row: &Row, ordinal: usize) -> BTreeMap<String, AspectValue> {
    let mut fields = BTreeMap::from([
        (
            "key".to_owned(),
            text(format!("recorded-correction:{ordinal:064x}")),
        ),
        (
            "role".to_owned(),
            AspectValue::UInt64(if row.selected.is_some() { 4 } else { 3 }),
        ),
        (
            "parent-branch".to_owned(),
            text(row.parent.native.branch_id.0.clone()),
        ),
        (
            "parent-commit".to_owned(),
            AspectValue::UInt64(row.parent.native.commit_id.0),
        ),
        // These rows are explicit canonical DATA fixtures, not fabricated Query
        // handler receipts or claims about an operation-admission counter.
        (
            "outcome-identity".to_owned(),
            AspectValue::UInt64(ordinal as u64),
        ),
    ]);
    for (prefix, source) in [
        ("source", Some(&row.source)),
        ("lineage-root", Some(&row.root)),
        ("selected-alternative", row.selected.as_ref()),
        ("active-reversal", row.active.as_ref()),
        ("prior-transition", row.prior.as_ref()),
    ] {
        if let Some(source) = source {
            fields.insert(
                format!("{prefix}-world-owner"),
                AspectValue::UInt64(
                    WorldOwnerCoordinate::Home {
                        owner: source.world.owner_identity().durable().unwrap(),
                    }
                    .row_owner_number(),
                ),
            );
            fields.insert(
                format!("{prefix}-world-commit"),
                AspectValue::UInt64(source.world.ordinal()),
            );
            fields.insert(
                format!("{prefix}-branch"),
                text(source.native.branch_id.0.clone()),
            );
            fields.insert(
                format!("{prefix}-commit"),
                AspectValue::UInt64(source.native.commit_id.0),
            );
        }
    }
    if row.selected.is_none() {
        fields.insert("reversed-root".to_owned(), selector(&row.root));
        fields.insert("reversed-source".to_owned(), selector(&row.source));
    } else {
        let active = row.active.as_ref().unwrap();
        fields.insert(
            "consumed-reversal".to_owned(),
            text(format!(
                "{}:{}:{}",
                active.native.branch_id.0.len(),
                active.native.branch_id.0,
                active.native.commit_id.0
            )),
        );
    }
    fields
}
pub(super) fn append(
    native: &worth_relational::facade::runtime::RelationalRuntime,
    execution: &ExecutionRequest<'_, '_>,
    batch: WorkerIntentBatch,
    ordinal: usize,
    row: &Row,
) -> WorkerIntentBatch {
    let fields = fields(row, ordinal)
        .into_iter()
        .map(|(field, value)| {
            (
                planned_single_field_locator(
                    AspectKey::new("aftermath-causality").unwrap(),
                    FieldKey::new(field).unwrap(),
                ),
                value,
            )
        })
        .collect();
    batch.push(MutationIntent::Create(CreateIntent::Entity(EntitySpec {
        partition_id: PartitionId::main(),
        kind_id: administration::kind(native, execution, "worth-query-aftermath-causality"),
        client_key: worth_relational::facade::symbols::ClientKey::raw(format!(
            "actual-correction-data-{ordinal}"
        )),
        fields: AspectFieldPatch::new(fields),
    })))
}
