"""Strongly connected components and a greedy feedback-minimizing layering of a weighted digraph."""


def components(nodes, weights):
    """Tarjan's algorithm, iterative; returns components as sorted lists in sorted order."""
    graph = {n: sorted({b for (a, b) in weights if a == n}) for n in nodes}
    index, low, stack, on_stack, result, counter = {}, {}, [], set(), [], [0]
    for root in sorted(nodes):
        if root in index:
            continue
        work = [(root, 0)]
        while work:
            node, position = work.pop()
            if position == 0:
                index[node] = low[node] = counter[0]
                counter[0] += 1
                stack.append(node)
                on_stack.add(node)
            successors = graph.get(node, [])
            if position < len(successors):
                work.append((node, position + 1))
                nxt = successors[position]
                if nxt not in index:
                    work.append((nxt, 0))
                elif nxt in on_stack:
                    low[node] = min(low[node], index[nxt])
                continue
            if low[node] == index[node]:
                group = []
                while True:
                    member = stack.pop()
                    on_stack.discard(member)
                    group.append(member)
                    if member == node:
                        break
                result.append(sorted(group))
            if work:
                parent = work[-1][0]
                low[parent] = min(low[parent], low[node])
    return sorted(result)


def layering(nodes, weights):
    """Eades-Lin-Smyth ordering, top (depends on others) to bottom (depended upon).

    Sinks go to the bottom, sources to the top, and otherwise the node whose
    outgoing minus incoming weight is largest goes next on top. Ties break by name.
    """
    remaining = set(nodes)
    top, bottom = [], []

    def out_weight(n):
        return sum(w for (a, b), w in weights.items() if a == n and b in remaining and b != n)

    def in_weight(n):
        return sum(w for (a, b), w in weights.items() if b == n and a in remaining and a != n)

    while remaining:
        changed = True
        while changed:
            changed = False
            for n in sorted(remaining):
                if out_weight(n) == 0:
                    bottom.insert(0, n)
                    remaining.discard(n)
                    changed = True
            for n in sorted(remaining):
                if in_weight(n) == 0:
                    top.append(n)
                    remaining.discard(n)
                    changed = True
        if remaining:
            best = max(sorted(remaining), key=lambda n: out_weight(n) - in_weight(n))
            top.append(best)
            remaining.discard(best)
    return top + bottom


def upward_edges(order, weights):
    """Edges whose source sits below its target in the order (a lower module using a higher one)."""
    position = {n: i for i, n in enumerate(order)}
    return sorted((a, b) for (a, b), w in weights.items()
                  if w and a != b and position[a] > position[b])
