import type {
  LocalTruthBranchReceipt,
  LocalTruthCommit,
  LocalTruthHistorySegment,
} from "worth-signals-wasm";

import type { GearHistoryLane, GearHistoryNode } from "./gear_scenario_view.ts";
import type { GearTruth } from "./gear_truth.ts";

interface GearHistoryInput {
  mainBranch: LocalTruthBranchReceipt;
  mainHistory: LocalTruthHistorySegment<GearTruth>;
  designBranch: LocalTruthBranchReceipt | null;
  designHistory: LocalTruthHistorySegment<GearTruth> | null;
}

interface CommitEntry {
  commit: LocalTruthCommit;
  order: number;
}

export function buildGearHistory({
  designBranch,
  designHistory,
  mainBranch,
  mainHistory,
}: GearHistoryInput): readonly GearHistoryNode[] {
  const commits = collectCommits(mainHistory, designHistory);
  const nodes = new Map([...commits].map(([id, { commit }]) => [
    id,
    commitNode(commit, mainBranch, designBranch, commits),
  ]));
  addCheckpointNode(nodes, mainHistory, mainBranch, "main");
  if (designBranch && designHistory) {
    addCheckpointNode(nodes, designHistory, designBranch, "design");
  }
  return topologicalOrder(nodes, commits).reverse();
}

function collectCommits(
  mainHistory: LocalTruthHistorySegment<GearTruth>,
  designHistory: LocalTruthHistorySegment<GearTruth> | null,
) {
  const commits = new Map<string, CommitEntry>();
  for (const commit of [...mainHistory.commits, ...(designHistory?.commits ?? [])]) {
    if (!commits.has(commit.id)) commits.set(commit.id, { commit, order: commits.size });
  }
  return commits;
}

function commitNode(
  commit: LocalTruthCommit,
  mainBranch: LocalTruthBranchReceipt,
  designBranch: LocalTruthBranchReceipt | null,
  commits: ReadonlyMap<string, CommitEntry>,
): GearHistoryNode {
  const lane: GearHistoryLane = commit.branchId === designBranch?.id ? "design" : "main";
  const branch = lane === "design" && designBranch ? designBranch : mainBranch;
  const parentIds = commit.parentCommitId && commits.has(commit.parentCommitId)
    ? [commit.parentCommitId]
    : [];
  const headLabels = [
    mainBranch.headCommitId === commit.id ? mainBranch.name : null,
    designBranch?.headCommitId === commit.id ? designBranch.name : null,
  ].filter((label): label is string => Boolean(label));
  return {
    id: commit.id,
    branchId: branch.id,
    lane,
    parentIds,
    kind: commit.kind,
    title: commitTitle(commit),
    detail: `${branch.name} · ${shortIdentity(commit.id)}`,
    headLabels,
    isLiveHead: headLabels.length > 0,
  };
}

function addCheckpointNode(
  nodes: Map<string, GearHistoryNode>,
  history: LocalTruthHistorySegment<GearTruth>,
  branch: LocalTruthBranchReceipt,
  lane: GearHistoryLane,
) {
  const checkpoint = history.checkpoint;
  if (!checkpoint || nodes.has(checkpoint.headCommitId)) return;
  nodes.set(checkpoint.headCommitId, {
    id: checkpoint.headCommitId,
    branchId: branch.id,
    lane,
    parentIds: [],
    kind: "checkpoint",
    title: `Checkpoint · ${checkpoint.compactedCommitCount} compacted changes`,
    detail: `${branch.name} · ${shortIdentity(checkpoint.headCommitId)}`,
    headLabels: branch.headCommitId === checkpoint.headCommitId ? [branch.name] : [],
    isLiveHead: branch.headCommitId === checkpoint.headCommitId,
  });
}

function topologicalOrder(
  nodes: ReadonlyMap<string, GearHistoryNode>,
  commits: ReadonlyMap<string, CommitEntry>,
) {
  const pending = [...nodes.values()].sort((left, right) => (
    (commits.get(left.id)?.order ?? -1) - (commits.get(right.id)?.order ?? -1)
  ));
  const emitted = new Set<string>();
  const ordered: GearHistoryNode[] = [];
  while (pending.length > 0) {
    const readyIndex = pending.findIndex(({ parentIds }) => (
      parentIds.every((parentId) => emitted.has(parentId) || !nodes.has(parentId))
    ));
    if (readyIndex < 0) {
      throw new Error("Local Truth history contains an unresolved parent cycle.");
    }
    const [next] = pending.splice(readyIndex, 1);
    ordered.push(next);
    emitted.add(next.id);
  }
  return ordered;
}

function commitTitle(commit: LocalTruthCommit) {
  if (commit.kind === "genesis") return "Initial gear";
  return commit.operations.map((operation) => {
    const value = operation.after ?? operation.value;
    return `${operation.aspectId} → ${formatValue(value)}`;
  }).join(" · ");
}

function formatValue(value: unknown) {
  return typeof value === "number" ? Number(value.toFixed(2)).toString() : String(value);
}

function shortIdentity(id: string) {
  return id.length > 22 ? `${id.slice(0, 19)}…` : id;
}
