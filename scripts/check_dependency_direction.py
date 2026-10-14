#!/usr/bin/env python3
"""Gate 1 structural checker (docs/architecture.md §8).

Runs `cargo metadata`, builds the real workspace dependency graph (resolved
normal + dev dependency edges, package name only — not versions), and fails
with a nonzero exit code and a specific reason if any of the following hold:

  * A Domain Core or Domain Orchestration crate has a dependency edge to
    `jitter-ports`, any `jitter-adapter-*`, `jitter-config`, or
    `jitter-observability`.
  * Any `jitter-adapter-*` appears in the dependency graph of
    `jitter-application` or any domain crate.
  * `jitter-contract-tests` appears outside `[dev-dependencies]` anywhere in
    the workspace.
  * A dependency cycle exists among the 22 planned crates.

This is Gate 1 only: it proves the shape of the architecture, not its
behavior. Gate 2 (semantic acceptance) is the per-crate test suite.
"""
import json
import subprocess
import sys

DOMAIN_CORE_AND_ORCHESTRATION = {
    "jitter-scenario",
    "jitter-workload",
    "jitter-policy",
    "jitter-budget",
    "jitter-scheduler",
    "jitter-outcome",
    "jitter-simulation",
    "jitter-recovery",
}
FORBIDDEN_FOR_DOMAIN = {"jitter-ports", "jitter-config", "jitter-observability"}


def is_adapter(name: str) -> bool:
    return name.startswith("jitter-adapter-")


def main() -> int:
    raw = subprocess.run(
        ["cargo", "metadata", "--format-version=1", "--no-deps"],
        capture_output=True,
        text=True,
        check=True,
    ).stdout
    metadata = json.loads(raw)
    packages = {p["name"]: p for p in metadata["packages"]}

    # name -> (normal_deps, dev_deps)
    normal_edges: dict[str, set[str]] = {}
    dev_edges: dict[str, set[str]] = {}
    for name, pkg in packages.items():
        normal = set()
        dev = set()
        for dep in pkg["dependencies"]:
            if dep["name"] not in packages:
                continue  # external crate, not part of this workspace's own graph
            if dep.get("kind") == "dev":
                dev.add(dep["name"])
            else:
                normal.add(dep["name"])
        normal_edges[name] = normal
        dev_edges[name] = dev

    failures: list[str] = []

    # Rule 1: Domain Core/Orchestration -> forbidden crates or any adapter.
    for name in DOMAIN_CORE_AND_ORCHESTRATION:
        for dep in normal_edges.get(name, set()):
            if dep in FORBIDDEN_FOR_DOMAIN or is_adapter(dep):
                failures.append(f"domain crate {name!r} has a forbidden dependency edge to {dep!r}")

    # Rule 2: no adapter in jitter-application's or any domain crate's graph.
    def transitive_normal(start: str) -> set[str]:
        seen: set[str] = set()
        stack = [start]
        while stack:
            current = stack.pop()
            for dep in normal_edges.get(current, set()):
                if dep not in seen:
                    seen.add(dep)
                    stack.append(dep)
        return seen

    for name in DOMAIN_CORE_AND_ORCHESTRATION | {"jitter-application"}:
        for dep in transitive_normal(name):
            if is_adapter(dep):
                failures.append(f"{name!r} transitively depends on adapter {dep!r}")

    # Rule 3: jitter-contract-tests only under [dev-dependencies] workspace-wide.
    for name, deps in normal_edges.items():
        if "jitter-contract-tests" in deps:
            failures.append(f"{name!r} depends on jitter-contract-tests as a normal (non-dev) dependency")

    # Rule 4: no dependency cycle among the 22 crates (normal edges only;
    # dev-dependency back-edges, e.g. an adapter's dev-dep on
    # jitter-contract-tests, are expected and excluded).
    def has_cycle() -> str | None:
        WHITE, GRAY, BLACK = 0, 1, 2
        color = {name: WHITE for name in packages}

        def visit(node: str, path: list[str]) -> str | None:
            color[node] = GRAY
            path.append(node)
            for dep in normal_edges.get(node, set()):
                if color.get(dep, BLACK) == GRAY:
                    cycle = path[path.index(dep):] + [dep]
                    return " -> ".join(cycle)
                if color.get(dep, BLACK) == WHITE:
                    result = visit(dep, path)
                    if result:
                        return result
            path.pop()
            color[node] = BLACK
            return None

        for name in packages:
            if color[name] == WHITE:
                result = visit(name, [])
                if result:
                    return result
        return None

    cycle = has_cycle()
    if cycle:
        failures.append(f"dependency cycle detected: {cycle}")

    if failures:
        print("GATE 1 FAILED:", file=sys.stderr)
        for failure in failures:
            print(f"  - {failure}", file=sys.stderr)
        return 1

    print(f"GATE 1 PASSED: {len(packages)} workspace crates, dependency direction is clean.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
