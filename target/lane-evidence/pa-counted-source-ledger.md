# Counted immutable-source fixture custody

Source scope: storage/residency/{region_fixture.rs,head_probe_tests.rs,
request_stop_tests.rs,member_custody_tests.rs}. No production or assertion edit.
All sources are below crates/worth-relational/src/.

The CountedSource owner funds source-bound selected custody through the actual
request and copies owned member bytes through ExecutionRequestArrayBuilder.
The immutable map stays owned by the existing retained source. Legacy counters,
byte limits and corruption behavior remain. Wrappers forward that protocol while
retaining their history-unavailable, retry-failure or active-owner semantics.
CustodySource funds both its real outer owner and the actual nested source
custody, then reads through that original inner custody/source identity.

Evidence under target/lane-evidence:
- pa-schema-counted-native.log5/0/0 includes the original3 assigned residency
  regressions plus2 retained schema/configuration units.
- pa-counted-dependent-native.log46/0/0 includes every remaining test name in
  the top-level fixture-dependent owner files; excludes the prior3 green names.
  No selected ignored case. Raw paths are listed in each umbrella log.
- pa-counted-dependent-format.log passes. Final scoped gates use pa-counted-final-*
  logs and must be read for their actual exit/statuses.

Execution logs record df2420390e with a dirty tree. While the46-case packet ran,
the already-reviewed schema/history and minor source was committed as71d5c78f13
and85d0486f45. No fixture input changed during the46-case build/run. The current
four-file diff is the same fixture correction; there is no new runtime behavior
or public API in either intervening commit. Clippy/gates record85d0486f45.

The original three tests now reach their owned Head promotion, sticky damage,
and independent content-hash assertions. The additional packet reaches original
head/history boundaries, provisional rounds, retry behavior, prefix custody,
kind commitments, layered/mixed history and corruption refusal assertions.
No budget, assertion, product semantics or fixture population changed.

The initial region_fixture.rs-only diff received APPROVE in
review-schema-counted-minors-final-verdict.md. That verdict predates the explicit
three-wrapper expansion; the separate final fixture review covers all four.

Continuation total before this slice's final gate:0check,2Clippy,5focused builds,
4selected runs. Final fixture Clippy is the third Clippy. No ignored body,
query-scale, exhaustive oracle or scenario. No integration merge or push.

Review correction: review-counted-final-verdict.md REJECT correctly identified
13 boundary findings, including a new in-scope member_custody_tests.rs root.
That file's request-bearing provider implementation now lives with the immutable
counted archive in region_fixture.rs as CountedPlacementSource; its assertion
file imports only that private fixture type. The complete implementation,
original nested custody and two original assertion bodies remain. No baseline,
alias, suppression or public surface changed. The fixture file is304 lines.
pa-counted-reviewfix-boundary.log returns the prior12 findings, none in the four
fixture files. Caps and function scan pass (one unchanged94-line fixture advisory).
pa-counted-reviewfix-native.log reruns only the two placement-custody cases;
pa-counted-reviewfix-clippy.log is the final Native-only Clippy. Read their final
statuses. All other previously passing test bodies are reused.

Final CountedSource source review: review-counted-final2-verdict.md APPROVE,
no findings. Changed fixture owner region_fixture.rs is304 lines. Corrected
boundary12 none changed; review-fix Native2/0/0 and Native-only Clippy exit0 with
zero library/new warnings and the two queued adapter warnings. Prior49 fixture
greens reused. No suppression, baseline allowance or public API.
