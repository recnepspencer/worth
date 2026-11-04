# Independent Host funding code review

Reviewed dirty source based at fab7d6e943; reviewer
/root/host_checkpoint_funding_code_review. Read-only; no builds/tests.

1. P2: resources.rs production thread site lacks perf-law category/source/count,
   local expectation and applicable constructor gates. Agreed; add the narrow
   Host gates and exact local sanctioned-site declaration.
2. P2: tiny-budget test could pass if maintenance never runs. Agreed; add actual
   automatic progress after latching caller cancellation, preserving exact body
   result. Existing bounded court remains unchanged.

Verdict: REJECT current Host portion pending these fixes and green evidence.
No other demonstrated functional flaw. Persistent memory/shared ledger, fresh
per-job work meter, same Store pending owner, joined worker, panic containment
and exclusive retirement custody were traced. Standard-library thread metadata
is not individually enumerated by tuple/handle Layout; extra64KiB conservative
allowance remains an explicit implementation bound requiring supported-platform
review rather than a proven quota overrun. Native replacement was not reviewed.
