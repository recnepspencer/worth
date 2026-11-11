# Independent admission DATA review

Source base3d7ccf16b0, first prerequisite commitb8c3d86727.

1. admission_binary_encoding_review: REJECT, medium. ByteProbe inherited the
   human-readable flag rather than the underlying MessagePack flag. Agreed;
   fixed and covered by fixed_byte_probe_obeys_the_underlying_serializer_mode.
2. admission_selector_data_review: REJECT, medium. Recursive borrowed-field
   scans repeated descendant traversal and did not charge that work. Agreed;
   replaced with streaming consumption and explicit outer traversal charges.
3. admission_data_final_review reviewed3d7ccf16b0..b8c3d86727: REJECT, P2.
   The selector's unrestricted family string could undergo whole-string UTF-8
   validation, comparison and copying between request polls. Agreed; b8c3d86727
   is not approved. The correction in progress preserves the existing string
   wire representation, reads bounded scalar metadata through the existing
   slice cursor, and performs family validation/copy/ordering in bounded chunks.
   A genuine long-family public court adds tight work-stop and cancellation
   observations. Its evidence and a fresh review must precede approval.

The third reviewer verified the supplied historical/corrected lane counts,
the exact precommit dirty-state headers, DATA/authority separation and line
caps. It did not certify full Home admission, protected Store selection,
D13 restoration, BC9002 completion or phase-completion gates.

4. Fresh admission_chunked_data_final_review reviewed the full source range
   3d7ccf16b0..d61b52fe288f7a116530573dc19dc87c78167611: APPROVE THIS COMMIT SLICE,
   no supported severity findings. The reviewer independently verified DATA-only
   authority, typed membership/ordering/AVL refusals, original-request funding,
   retained grants, <=64KiB UTF-8/copy/comparison work intervals, serializer mode
   and allocation-free byte staging. It checked the raw3/0/0,7/0/0,2/0/0,1/0/0
   and final9/0/0 execution evidence and classified the ignored discovery events
   separately from selected execution. All22 changed Rust files are below400
   lines. It ran no builds or tests.

   The approval is expressly not D12/D13 or full-task completion. The reviewer
   states that the decoded backing has no operational consumer, and that actual
   Home selection, root/publication admission, World/Query restoration, bounded
   filesystem acceptance and phase gates are not established. Those are still
   P-A work. Execution logs remain honest development-tree evidence, not claimed
   clean-final-commit executions. No earlier rejected snapshot is called approved.

Resolutions: findings1 and2 were fixed inb8c3d86727; finding3 was fixed
ind61b52fe28 and its long-family control passes. The fresh reviewer found no
additional in-slice defect. All full-task limitations remain explicitly open.
