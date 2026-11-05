# Independent close funding boundary source review

Reviewer: /root/close_funding_boundary_review. Read-only; no edits/builds/tests.
Reviewed dirty source on HEAD3be5798075, 2026-11-04.

1. No cycle or Query alias defect: Store strong Arc -> Resources; Resources has
   no operation/Home/runtime/provider edge. Query HomeMaintenance has no new Weak
   aliases, preserving its existing Arc::get_mut preflight.
2. Host root funds Resources Arc allocation and worker backing; backing is last
   field and remains with the last strong service owner.
3. Purpose::Close uses the actual borrowed operation through the same joined
   worker/root/controlled child/deadline/active guard. Explicit capture retains
   caller funding. Absence/busy/spawn/interruption failures are typed.
4. Exact Store capture callsite changes only new capture dispatch. Prior close
   settlement and final interruption keep the original close request. Registered
   service Arc is cloned under lock, then invoked after lock release. Exact
   factory caller list names only Host maintenance preparation.

Verdict: source boundary passes, no material design/source findings. This is NOT
whole-funding approval; behavioral qualification and enforcement unassessed.
