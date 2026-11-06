# Returning publication phases after the full main-stack census

Reviewed design: head_closure_design_review, source-only, before compilation.
At9379f46 all3paths overflow:1053056/1070696/1068184resolved staticbytes,
2external frames each. Full current compiler unwind lists are retained in
head-closure-main-round2-*-frames.json, not just selected large frames.
The first41ba25 split left construction publication263928bytes and conditional
publication336152bytes active above publish_application_graph130872bytes.
The standalone entry cutover c43a229 exposed these on Windows main stacks.

Construction now returns from configuration before actual service publication,
then returns from service publication before program/output/lineage completion.
Completion metadata has its OWN funded storage and remains a thin field through
both earlier phases. Plain None/Some time sources retain their original distinct
calls. Actual Plain/Conditional route payloads have original-request funded backing.

Conditional publication now returns from graph publication, route/resource/seal,
runtime assembly/contribution transfer, then reconstruction/reconciliation/cleanup.
Every successor backing is reserved from phase.execution_request before that
phase's effects. Predecessor storage is consumed once. No public API, Native
checkpoint/readmission code, feature, stack override, larger thread or allow changes.
The remaining130872graph producer and80408program wrapper are bounded ancestors
once both dominant future-state frames retire. Verification must run ALL3normal
executables plus Bank exact parents; no assertion or stack workaround is accepted.

The normal consumer duplicate journey uses the current D9 recorded-original
receipt route, as resolve_idempotency exhaustively shows. Its typed outcome is
RecoveredCommitted. The fixture now compares original recorded correspondence,
basis and changed count, while keeping its complete final output equality.

## Observed result (supersedes the design prediction)

At8d567b188c all3normalmains stilloverflow. Fullcurrent lists:
head-closure-main-round3-*-frames.json. Constructionservice142840 and
graph_publication214840 remainactive above graphproducer130872; current
paths nowreach Native selected-source validation94616/proposal preparation.
The three granted verification rounds are spent; this design is NOT qualified
as a complete stack repair. Preserve the directmain entries and original funding.
