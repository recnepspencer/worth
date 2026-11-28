# Deterministic simulation testing

Use `worth-dst` as a court dev-dependency. `Spec` declares a `Seed`, starting
Unix time, step count, and any clock faults. `DstRunner::run` applies faults
before their steps and gives the scenario one SplitMix64 stream. Provide
enabled operations in canonical order, and use that stream for scheduling and
domain fault choices. Connect `run.clock()` to Execution's `TimeSampler`.
Each run has a fresh boot timeline and a reproducible logical wall source.

To reproduce a run, start from the same scenario state and call
`DstRunner::run` again with the same spec. Keep the seed with any reported
failure. The trace contains scenario records in execution order. Pass a
single encoded JSON object per step or observation to `Trace::push_json` and
write the result with `Trace::write`. Keep field and set member order stable,
and encode TLC sets as `{"__set__":[...]}`. Each row ends with one LF.

For the persistent-naming port, `Seed::rng` keeps the existing SplitMix64 draws
and modulo choices. Feed the existing action and observation rows to
`Trace::push_json` in their existing order. The writer preserves the row bytes,
including field order, with the same final newline. Validate the emitted file
with `python3 tools/tla/run_tlc.py trace <TraceSpec.tla> <trace.ndjson>`.
