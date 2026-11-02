# Phase5 Scene-context authoring and contour routing: implementation plan

> Scope update2026-11-01: this is a historical receipt/plan under the superseded
> public CAD scope. The Scene/Face application and its authoring/contact expansion
> are removed. Distinct generic numerical/provider fixes remain. This record does
> not reopen a public CAD prerequisite; see the [cleanup evidence](milestone-9.19-phase5-public-cad-cleanup-evidence.md).

The boundary review and independent ruling are complete before Rust edits.

1. Add the pure PlanarOperation::InsertVertexAfter input and a fresh-key domain
   refusal. Preserve all old serialized variants. Move repeated planar Body field
   initialization into topology_entry/planar_vertex.rs; cut over CreateCycle and
   VertexReplacement without changing their allocation contexts or outputs.
2. Add handler/insertion.rs with tracked actual outgoing/incoming correspondence,
   explicit previous target and new-key absence checks. Candidate atomically inserts
   one real vertex in the predecessor context and creates its actual output member.
   Parent handler dispatches only; keep every touched file below400 lines.
   Bump both affected handler identity versions, preserving existing input/result/
   command identities and original controls.
3. Add scene_faces/creation binding/declaration/handler for CreateSceneTriangle:
   existing Scene scope/target, tracked FaceKey/three generated BodyKey absences,
   actual contextual Body/Face creation and closed successor/membership/anchor writes.
   Reuse the existing Scene authoring projection policy and Native invariants.
   No geometry check duplicated in the handler and no new budget/stop/callback lane.
4. Register its actual action in SceneFacesFeature, configure its handler at the
   existing contribution boundary, move ConsumerSchema to minor1.4, and retain
   existing four configuration elements/observation lifetime. v1 query profiles and
   managed Source/provider meaning stay unchanged.
5. Add finite existing consumer-root courts. Create a second triangle in one actual
   Scene; verify independent full Face/anchor identities, complete structural/candidate
   census and warm maintenance. Reject duplicate keys without publication. Protect
   old input serialization with literal contract expectations in the same harness.
6. Exercise ordinary unanchored positive quad insertion and require the real created
   output member. Repeat the identical edit on an anchored triangle: exact typed
   TriangleFaceContour refusal, unchanged head/Source/candidate rows and no new D.
   Use the actual public insertion receipt and preserved-output identities to
   prove no authoritative entity patch targets old A/B/C. Static Source/handler
   inspection proves unchanged A/succB/predC adjacency and no Face/FaceAnchor
   write; do not mislabel declaration capabilities as the actual edit roster. Wrong-old
   successor and duplicate-new-key cases must refuse atomically at their real owners.
7. Build the existing consumer root once after mechanical changes, run focused Scene
   family then affected consumer family. Keep known cleanup exclusions and unchanged
   large Catalog receipt explicit. Run actual Cargo boundary/context, scoped format/
   caps/function/text checks, update evidence/guidance and obtain all three independent
   QA gates before committing this coherent slice.

Contact/Native collision integration,1000/2000/+50000 face locality, public closure
and Phase6 remain subsequent work. This slice makes actual same-Scene authoring
and Body-only topology routing concrete without pretending those are spatial scale.
