# Independent layout-design reviews

The initial SignalRuntime proposal was first approved, then REJECTED on deeper
source trace. Three P1 findings were accepted: it missed the early Query denial,
its runtime allocations would occur on ordinary admission, and the Bridge
exclusivity reservation was not funding. No runtime boxes were implemented.

Fresh reviewer original_funded_layout_design_review: APPROVE revised design,
no blocking findings; read-only, no builds. It independently traced original
ExecutionRequest admission, immutable body sharing, original template digest
retention and capacity<Self>(1) partition funding. Re-instantiation tests must
cover repeated instances against fresh lowering. H10 remains unproven until
three genuine example mains pass on their unchanged 1 MiB stack.
