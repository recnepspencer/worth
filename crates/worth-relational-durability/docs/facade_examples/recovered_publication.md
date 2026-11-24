Actual custody remains transportable without copying its pins:
```
use worth_proof::RelationalPublicationReadmitted;
use worth_relational::facade::durability::RelationalRecoveredPublicationCustody;
fn transport<T: Send + Sync + 'static>() {}
transport::<RelationalRecoveredPublicationCustody>();
transport::<RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>>();
```

```compile_fail,E0308
use worth_proof::{AuthorityMarker, RelationalPublicationReadmitted};
use worth_relational::facade::durability::RelationalRecoveredPublicationCustody;
struct ReceiptDescription;
impl AuthorityMarker for ReceiptDescription {}
fn adopt(_: RelationalPublicationReadmitted<RelationalRecoveredPublicationCustody>) {}
adopt(RelationalPublicationReadmitted::from_owner(ReceiptDescription));
```
