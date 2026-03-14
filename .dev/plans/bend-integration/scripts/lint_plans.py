#!/usr/bin/env python3
"""Lint Axon-Bend plan XML documents for quality, completeness, and consistency."""

import sys
import re
from pathlib import Path

try:
    from lxml import etree
except ImportError:
    import xml.etree.ElementTree as etree


# Tag used to identify xref-related warnings for --strict-xref filtering
XREF_WARNING_PREFIX = "[xref] "


def lint_backlog(plan_dir: Path) -> tuple[list, list, list]:
    """Lint the implementation backlog for completeness."""
    errors, warnings, infos = [], [], []
    path = plan_dir / "05-implementation-backlog.xml"
    if not path.exists():
        errors.append("Backlog (05) missing — cannot lint")
        return errors, warnings, infos

    tree = etree.parse(str(path))
    root = tree.getroot()

    epics = list(root.iter("epic"))
    stories = list(root.iter("story"))

    for epic in epics:
        eid = epic.get("id", "?")
        dod = epic.find("definitionOfDone")
        if dod is None:
            warnings.append(f"Epic {eid} missing definitionOfDone")
        story_list = epic.find("stories")
        if story_list is None or len(list(story_list)) == 0:
            warnings.append(f"Epic {eid} has no stories")

    # Check story ID format
    for story in stories:
        sid = story.get("id", "")
        if not re.match(r"E\d+-S\d+", sid):
            errors.append(f"Story ID '{sid}' does not match E{{N}}-S{{N}} pattern")

    # Check summary consistency
    summary = root.find("summary")
    if summary is not None:
        total_el = summary.find("totalStories")
        if total_el is not None:
            try:
                declared = int(total_el.text) if total_el.text else None
            except (ValueError, TypeError):
                declared = None
            if declared is None:
                warnings.append("Backlog summary totalStories is missing or not a valid integer")
            else:
                actual = len(stories)
                if declared != actual:
                    warnings.append(f"Backlog declares {declared} stories but has {actual}")

    infos.append(f"Backlog: {len(epics)} epics, {len(stories)} stories")
    return errors, warnings, infos


def lint_risks(plan_dir: Path) -> tuple[list, list, list]:
    """Lint risk register for completeness."""
    errors, warnings, infos = [], [], []
    path = plan_dir / "12-risk-register.xml"
    if not path.exists():
        errors.append("Risk register (12) missing — cannot lint")
        return errors, warnings, infos

    tree = etree.parse(str(path))
    root = tree.getroot()

    risks = list(root.iter("risk"))
    for risk in risks:
        rid = risk.get("id", "?")
        mitigation = risk.find("mitigation")
        if mitigation is None or not mitigation.text:
            warnings.append(f"Risk {rid} missing mitigation strategy")
        residual = risk.find("residual")
        if residual is None or not residual.text:
            warnings.append(f"Risk {rid} missing residual risk assessment")

    infos.append(f"Risk register: {len(risks)} risks defined")
    return errors, warnings, infos


def lint_gates(plan_dir: Path) -> tuple[list, list, list]:
    """Lint gates scorecard for completeness."""
    errors, warnings, infos = [], [], []
    path = plan_dir / "10-gates-scorecard.xml"
    if not path.exists():
        errors.append("Gates scorecard (10) missing — cannot lint")
        return errors, warnings, infos

    tree = etree.parse(str(path))
    root = tree.getroot()

    gates = list(root.iter("gate"))
    phases_with_hard_gates = set()

    for gate in gates:
        phase = gate.get("phase", "")
        hard_gates = gate.find("hardGates")
        if hard_gates is not None and len(list(hard_gates)) > 0:
            phases_with_hard_gates.add(phase)

    expected_phases = {"ALPHA", "BETA", "GAMMA", "DELTA", "EPSILON"}
    missing = expected_phases - phases_with_hard_gates
    if missing:
        for p in missing:
            errors.append(f"Phase {p} has no hard gates in scorecard")

    # Count individual gate children, not container elements
    hard_count = 0
    for container in root.iter("hardGates"):
        hard_count += len(list(container))
    soft_count = 0
    for container in root.iter("softGates"):
        soft_count += len(list(container))
    infos.append(f"Gates: {hard_count} hard gates, {soft_count} soft gates")
    return errors, warnings, infos


def lint_dependency_cycles(plan_dir: Path) -> tuple[list, list, list]:
    """Check for cycles in epic dependency graph."""
    errors, warnings, infos = [], [], []
    path = plan_dir / "05-implementation-backlog.xml"
    if not path.exists():
        return errors, warnings, infos

    tree = etree.parse(str(path))
    root = tree.getroot()

    # Build adjacency list
    graph = {}
    for epic in root.iter("epic"):
        eid = epic.get("id", "")
        deps_el = epic.find("dependsOn")
        deps = deps_el.text.split() if deps_el is not None and deps_el.text else []
        graph[eid] = deps

    # DFS cycle detection
    visited = set()
    in_stack = set()

    def has_cycle(node):
        if node in in_stack:
            return True
        if node in visited:
            return False
        visited.add(node)
        in_stack.add(node)
        for dep in graph.get(node, []):
            if has_cycle(dep):
                return True
        in_stack.remove(node)
        return False

    for epic_id in graph:
        if has_cycle(epic_id):
            warnings.append(f"{XREF_WARNING_PREFIX}Dependency cycle detected involving {epic_id}")
            break

    if not warnings:
        infos.append("Dependency graph: acyclic (OK)")

    return errors, warnings, infos


def lint_traceability(plan_dir: Path) -> tuple[list, list, list]:
    """Check traceability coverage."""
    errors, warnings, infos = [], [], []
    path = plan_dir / "11-traceability-matrix.xml"
    if not path.exists():
        warnings.append(f"{XREF_WARNING_PREFIX}Traceability matrix (11) missing — cannot check coverage")
        return errors, warnings, infos

    tree = etree.parse(str(path))
    root = tree.getroot()

    chains = list(root.iter("chain"))
    coverage = root.find(".//coverageSummary")
    if coverage is not None:
        recs = coverage.find(".//recommendations")
        if recs is not None:
            cov = recs.get("coverage", "")
            if cov != "100%":
                warnings.append(f"{XREF_WARNING_PREFIX}Traceability coverage is {cov}, target is 100%")

    infos.append(f"Traceability: {len(chains)} chains defined")
    return errors, warnings, infos


def main():
    plan_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
    strict_xref = "--strict-xref" in sys.argv

    print(f"=== Axon-Bend Plan Linter ===")
    print(f"Plan directory: {plan_dir.resolve()}\n")

    all_errors, all_warnings, all_infos = [], [], []

    for name, fn in [
        ("Backlog", lint_backlog),
        ("Risks", lint_risks),
        ("Gates", lint_gates),
        ("Dependencies", lint_dependency_cycles),
        ("Traceability", lint_traceability),
    ]:
        print(f"--- {name} ---")
        errs, warns, info = fn(plan_dir)
        for e in errs:
            print(f"  ERROR: {e}")
        for w in warns:
            print(f"  WARN:  {w}")
        for i in info:
            print(f"  INFO:  {i}")
        all_errors.extend(errs)
        all_warnings.extend(warns)
        all_infos.extend(info)
        print()

    print(f"=== Summary ===")
    print(f"Errors:   {len(all_errors)}")
    print(f"Warnings: {len(all_warnings)}")
    print(f"Info:     {len(all_infos)}")

    if all_errors:
        print("\nLINT FAILED")
        return 1
    elif strict_xref:
        # In --strict-xref mode, only fail on xref-related warnings (tagged with prefix)
        xref_warnings = [w for w in all_warnings if w.startswith(XREF_WARNING_PREFIX)]
        if xref_warnings:
            print(f"\nLINT FAILED (--strict-xref: {len(xref_warnings)} xref warning(s))")
            return 1
    print("\nLINT PASSED")
    return 0


if __name__ == "__main__":
    sys.exit(main())
