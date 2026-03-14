#!/usr/bin/env python3
"""Validate Axon-Bend plan XML documents for well-formedness and structural correctness."""

import sys
import os
from pathlib import Path

try:
    from lxml import etree
    HAS_LXML = True
except ImportError:
    import xml.etree.ElementTree as etree
    HAS_LXML = False


REQUIRED_DOCUMENTS = [
    "00-plan-index.xml",
    "01-master-strategy.xml",
    "02-capability-matrix.xml",
    "03-architecture-blueprint.xml",
    "04-workstreams-and-milestones.xml",
    "05-implementation-backlog.xml",
    "06-claude-execution-playbook.xml",
    "07-prompt-library.xml",
    "08-context-governance.xml",
    "09-validation-and-testing.xml",
    "10-gates-scorecard.xml",
    "11-traceability-matrix.xml",
    "12-risk-register.xml",
    "13-benchmark-harness.xml",
    "14-decision-records.xml",
    "15-plan-schema-and-ci.xml",
]


def validate_wellformed(filepath: Path) -> list[str]:
    """Check if XML file is well-formed."""
    errors = []
    try:
        if HAS_LXML:
            etree.parse(str(filepath))
        else:
            etree.parse(str(filepath))
    except Exception as e:
        errors.append(f"  MALFORMED: {e}")
    return errors


def validate_document_exists(plan_dir: Path) -> tuple[list[str], list[str]]:
    """Check that all required documents exist."""
    errors = []
    warnings = []
    for doc in REQUIRED_DOCUMENTS:
        path = plan_dir / doc
        if not path.exists():
            errors.append(f"  MISSING: {doc}")
        elif path.stat().st_size == 0:
            warnings.append(f"  EMPTY: {doc}")
    return errors, warnings


def validate_xref_ids(plan_dir: Path) -> list[str]:
    """Cross-reference validation: check that referenced IDs exist."""
    errors = []

    # Collect all defined epic IDs from backlog
    epic_ids = set()
    story_ids = set()
    backlog = plan_dir / "05-implementation-backlog.xml"
    if backlog.exists():
        try:
            tree = etree.parse(str(backlog))
            root = tree.getroot() if HAS_LXML else tree.getroot()
            for epic in root.iter("epic"):
                eid = epic.get("id")
                if eid:
                    epic_ids.add(eid)
            for story in root.iter("story"):
                sid = story.get("id")
                if sid:
                    story_ids.add(sid)
        except Exception:
            pass

    # Collect all risk IDs
    risk_ids = set()
    risk_file = plan_dir / "12-risk-register.xml"
    if risk_file.exists():
        try:
            tree = etree.parse(str(risk_file))
            root = tree.getroot()
            for risk in root.iter("risk"):
                rid = risk.get("id")
                if rid:
                    risk_ids.add(rid)
        except Exception:
            pass

    # Check traceability matrix references
    trace_file = plan_dir / "11-traceability-matrix.xml"
    if trace_file.exists():
        try:
            tree = etree.parse(str(trace_file))
            root = tree.getroot()
            for chain in root.iter("chain"):
                epic_el = chain.find("epic")
                if epic_el is not None and epic_el.text:
                    # Extract epic ID from text like "E1 — Name"
                    epic_ref = epic_el.text.split(" ")[0] if epic_el.text else ""
                    if epic_ref and epic_ref not in epic_ids and epic_ids:
                        errors.append(f"  ORPHAN_REF: Traceability chain references {epic_ref} not in backlog")
        except Exception:
            pass

    return errors


def main():
    plan_dir = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")

    print(f"=== Axon-Bend Plan Validation ===")
    print(f"Plan directory: {plan_dir.resolve()}\n")

    total_errors = 0
    total_warnings = 0

    # 1. Check all documents exist
    print("--- Document Existence ---")
    errors, warnings = validate_document_exists(plan_dir)
    if errors:
        for e in errors:
            print(e)
        total_errors += len(errors)
    if warnings:
        for w in warnings:
            print(w)
        total_warnings += len(warnings)
    if not errors and not warnings:
        print(f"  ALL {len(REQUIRED_DOCUMENTS)} documents present.")
    print()

    # 2. Validate each document is well-formed XML
    print("--- XML Well-Formedness ---")
    for doc in REQUIRED_DOCUMENTS:
        path = plan_dir / doc
        if path.exists():
            errs = validate_wellformed(path)
            if errs:
                print(f"  FAIL: {doc}")
                for e in errs:
                    print(f"    {e}")
                total_errors += len(errs)
            else:
                print(f"  PASS: {doc}")
    print()

    # 3. Cross-reference validation
    print("--- Cross-Reference Validation ---")
    xref_errors = validate_xref_ids(plan_dir)
    if xref_errors:
        for e in xref_errors:
            print(e)
        total_errors += len(xref_errors)
    else:
        print("  All cross-references valid.")
    print()

    # Summary
    print(f"=== Summary ===")
    print(f"Errors:   {total_errors}")
    print(f"Warnings: {total_warnings}")

    if total_errors > 0:
        print("\nVALIDATION FAILED")
        return 1
    else:
        print("\nVALIDATION PASSED")
        return 0


if __name__ == "__main__":
    sys.exit(main())
