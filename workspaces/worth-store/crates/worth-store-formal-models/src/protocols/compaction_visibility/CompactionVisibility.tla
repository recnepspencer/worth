---- MODULE CompactionVisibility ----
EXTENDS Naturals

VARIABLES lifecycle, publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration
vars == <<lifecycle, publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration>>

Init == /\ lifecycle = "Idle" /\ publicationPrepared = FALSE /\ oldRetained = TRUE /\ tombstonePreserved = TRUE /\ readers = 1 /\ visibleGeneration = 0
Plan == /\ lifecycle = "Idle" /\ lifecycle' = "Planned" /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration>>
Write == /\ lifecycle = "Planned" /\ lifecycle' = "Writing" /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration>>
Durable == /\ lifecycle = "Writing" /\ lifecycle' = "Durable" /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration>>
AttemptPublish == /\ lifecycle = "Durable" /\ lifecycle' = "PublishAttempted" /\ publicationPrepared' = TRUE /\ UNCHANGED <<oldRetained, tombstonePreserved, readers, visibleGeneration>>
Publish == /\ lifecycle = "PublishAttempted" /\ publicationPrepared /\ lifecycle' = "Visible" /\ visibleGeneration' = visibleGeneration + 1 /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers>>
CrashToOrphan == /\ lifecycle \in {"Writing", "Durable", "PublishAttempted"} /\ lifecycle' = "Orphan" /\ publicationPrepared' = FALSE /\ UNCHANGED <<oldRetained, tombstonePreserved, readers, visibleGeneration>>
Rollback == /\ lifecycle = "Orphan" /\ lifecycle' = "RolledBack" /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration>>
Retry == /\ lifecycle = "Orphan" /\ lifecycle' = "Planned" /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration>>
ReleaseReader == /\ readers > 0 /\ readers' = readers - 1 /\ UNCHANGED <<lifecycle, publicationPrepared, oldRetained, tombstonePreserved, visibleGeneration>>
Reclaim == /\ lifecycle = "Visible" /\ readers = 0 /\ oldRetained /\ lifecycle' = "ReclaimEligible" /\ oldRetained' = FALSE /\ UNCHANGED <<publicationPrepared, tombstonePreserved, readers, visibleGeneration>>
LowerRewrite == /\ lifecycle \in {"Idle", "Planned"} /\ lifecycle' = "Durable" /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers, visibleGeneration>>
PublishRewrite == /\ lifecycle = "Durable" /\ lifecycle' = "Published" /\ publicationPrepared' = TRUE /\ UNCHANGED <<oldRetained, tombstonePreserved, readers, visibleGeneration>>
ValidateReadPlanCutover == /\ lifecycle = "Published" /\ lifecycle' = "Visible" /\ visibleGeneration' = visibleGeneration + 1 /\ UNCHANGED <<publicationPrepared, oldRetained, tombstonePreserved, readers>>
DeferReclaim == /\ lifecycle = "Visible" /\ readers > 0 /\ oldRetained /\ UNCHANGED vars
DrainReclaimAfterReadRelease == /\ lifecycle = "Visible" /\ oldRetained /\ lifecycle' = "ReclaimEligible" /\ readers' = 0 /\ oldRetained' = FALSE /\ UNCHANGED <<publicationPrepared, tombstonePreserved, visibleGeneration>>
DenyLsmOwnerCase == /\ lifecycle # "ReclaimEligible" /\ UNCHANGED vars
DenyInPlaceOverwrite == /\ oldRetained /\ UNCHANGED vars
DenyEarlyReclaim == /\ readers > 0 /\ oldRetained /\ UNCHANGED vars
DenyStaleEpochReuse == /\ lifecycle # "ReclaimEligible" /\ UNCHANGED vars
DenyLatchHierarchyInversion == /\ lifecycle \in {"Planned", "Writing", "Durable", "Published", "Visible"} /\ UNCHANGED vars
DenyMixedRootRead == /\ lifecycle \in {"PublishAttempted", "Published"} /\ UNCHANGED vars

Next == Plan \/ Write \/ Durable \/ AttemptPublish \/ Publish \/ CrashToOrphan \/ Rollback \/ Retry \/ ReleaseReader \/ Reclaim
        \/ LowerRewrite \/ PublishRewrite \/ ValidateReadPlanCutover \/ DeferReclaim
        \/ DrainReclaimAfterReadRelease \/ DenyLsmOwnerCase \/ DenyInPlaceOverwrite
        \/ DenyEarlyReclaim \/ DenyStaleEpochReuse
        \/ DenyLatchHierarchyInversion \/ DenyMixedRootRead
Spec == Init /\ [][Next]_vars
TypeOK == /\ lifecycle \in {"Idle", "Planned", "Writing", "Durable", "PublishAttempted", "Published", "Visible", "Orphan", "RolledBack", "ReclaimEligible"} /\ publicationPrepared \in BOOLEAN /\ oldRetained \in BOOLEAN /\ tombstonePreserved \in BOOLEAN /\ readers \in 0..1 /\ visibleGeneration \in Nat
VisibilityNeedsPublication == visibleGeneration > 0 => lifecycle \in {"Visible", "ReclaimEligible"}
VisibleGenerationNeedsPreparedCutover == visibleGeneration > 0 => publicationPrepared
ReclaimNeedsRelease == lifecycle = "ReclaimEligible" => readers = 0
TombstonesNeverResurrect == tombstonePreserved
====
