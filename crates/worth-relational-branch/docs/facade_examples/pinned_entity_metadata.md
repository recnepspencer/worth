```compile_fail,E0599
use worth_relational::facade::runtime::RelationalPinnedEntityMetadata;
fn historical(pin: &RelationalPinnedEntityMetadata) { pin.full_view(); }
```
