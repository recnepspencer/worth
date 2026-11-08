# Second and final stack round

Round 1 at 97f0fc97d2: real consumer-root 1/0/0; Bank parents 2/0/0;
provider courts 12/0/0. All three real examples still overflow. This round
settles the complete remaining Bridge/Signal sealing path, not another Native
frame. Full compiler unwind records: main-entry-round1-*-frames.json; source
line lookup was unavailable in these PDBs (frame-lines.log records that limit).

All three paths now have Signal's current branch-cell creation frames:
SignalBranchExecutionCell::new 39144 (two paths), reservation::prepare_cell_state
38984, from_partition closure 38600, from_partition 60552 and owner_port_slots
60296. These carry the actual current branch state. They are not late Query
retirement storage, and no Signal representation or allowance changes here.

The overlapping Query ancestors are managed_bridge::seal 138200,
seal_application_graph 61112, route_installation::install 155720, the service
conditional publication return frame 99832, and program construction 80408.
The 99832 frame has an aliased nearest PDB symbol; its unique position between
publish_contributions and service::publish identifies the service helper.
Entry frames are 113784/52392/49784; genuine mains remain untouched.

The Bridge installed value acquired three mutually exclusive inline slots in
74214eff891: ordinary, retiring_ordinary and retired_ordinary. Replace them with
one private exhaustive lifecycle enum. Serving owns ordinary then conditional;
Retiring owns conditional then the retirement task; Retired owns conditional
then retained retired meaning. A fourth Released variant retains that retired
meaning after the final conditional alias release. Preserve those field drop orders exactly. A
transition takes the one original payload, restores it on typed refusal, and
keeps the same consult, pointer check, Advanced/Complete and final alias release.
No allocation, changed close gate, census, Drop-only release or new authority.

Separate route/resource configuration from graph sealing into returning funded
phases. Carry the graph backed into sealing and the sealed graph backed into
runtime assembly. reserve each successor before its effects; do not keep the
raw final runtime or later graph next to Signal's current branch state. A small
non-inlined Query Bridge assembly helper runs only after conditional seal returns.
The same private graph-sealing route serves clock and conditional installation.

Verification: Query owner strict check once, then rebuild the real entries and
the two Bank parents affected by this representation change. The shared normal
entry build can retain consumer-root as a final-source control. One named actual
same-directory Home close/reopen journey covers the changed private retirement
representation. Keep the earlier twelve provider cases as separately pinned
evidence of the unchanged fixture factory, not as a new whole-source run.
One serial build at a time, C: >=25 GiB, no stack override.
Then phase gates and a fresh independent final review with no reviewer rebuilds.

Source-only design review main_stack_design_review APPROVE: explicit Released
state, preserve mutated retirement task on advance refusal, and consult/foreign
owner checks before taking custody. No Signal changes are needed. The shared
graph seal must itself return funded storage without a raw successor local.
