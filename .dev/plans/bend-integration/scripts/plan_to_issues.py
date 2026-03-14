#!/usr/bin/env python3
"""Generate GitHub issues from Axon-Bend implementation backlog epics and stories."""

import sys
import subprocess
import json
from pathlib import Path

try:
    from lxml import etree
except ImportError:
    import xml.etree.ElementTree as etree


def parse_backlog(path: Path):
    """Parse the implementation backlog and extract epics/stories."""
    tree = etree.parse(str(path))
    root = tree.getroot()

    epics = []
    for epic_el in root.iter("epic"):
        epic = {
            "id": epic_el.get("id", ""),
            "name": epic_el.get("name", ""),
            "priority": epic_el.get("priority", "P1"),
            "phase": epic_el.get("phase", ""),
            "workstream": epic_el.get("workstream", ""),
            "stories": [],
            "depends_on": [],
            "definition_of_done": [],
        }

        deps_el = epic_el.find("dependsOn")
        if deps_el is not None and deps_el.text:
            epic["depends_on"] = deps_el.text.strip().split()

        dod_el = epic_el.find("definitionOfDone")
        if dod_el is not None:
            for item in dod_el.findall("item"):
                if item.text:
                    epic["definition_of_done"].append(item.text.strip())

        stories_el = epic_el.find("stories")
        if stories_el is not None:
            for story_el in stories_el.findall("story"):
                epic["stories"].append({
                    "id": story_el.get("id", ""),
                    "text": story_el.text.strip() if story_el.text else "",
                })

        epics.append(epic)

    return epics


def format_issue(epic, story=None):
    """Format a GitHub issue from an epic or story."""
    if story:
        title = f"[{story['id']}] {story['text'][:60]}"
        body_lines = [
            f"**Epic:** {epic['id']} — {epic['name']}",
            f"**Story:** {story['id']}",
            f"**Phase:** {epic['phase']}",
            f"**Priority:** {epic['priority']}",
            f"**Workstream:** {epic['workstream']}",
            "",
            "## Description",
            story['text'],
            "",
        ]
    else:
        title = f"[{epic['id']}] {epic['name']}"
        body_lines = [
            f"**Phase:** {epic['phase']}",
            f"**Priority:** {epic['priority']}",
            f"**Workstream:** {epic['workstream']}",
            "",
            "## Stories",
        ]
        for s in epic["stories"]:
            body_lines.append(f"- [ ] **{s['id']}**: {s['text']}")
        body_lines.append("")

    if epic["depends_on"]:
        body_lines.append(f"## Dependencies")
        body_lines.append(f"Depends on: {', '.join(epic['depends_on'])}")
        body_lines.append("")

    if epic["definition_of_done"]:
        body_lines.append("## Definition of Done")
        for item in epic["definition_of_done"]:
            body_lines.append(f"- [ ] {item}")
        body_lines.append("")

    body_lines.append("---")
    body_lines.append("*Auto-generated from Axon-Bend integration plan*")

    labels = [
        f"phase:{epic['phase'].lower()}",
        f"priority:{epic['priority'].lower()}",
        f"workstream:{epic['workstream'].lower()}",
        "bend-integration",
    ]

    return {
        "title": title,
        "body": "\n".join(body_lines),
        "labels": labels,
    }


def create_issue(repo: str, issue: dict, dry_run: bool = True):
    """Create a GitHub issue using gh CLI."""
    if dry_run:
        print(f"  [DRY RUN] Would create: {issue['title']}")
        print(f"            Labels: {', '.join(issue['labels'])}")
        return

    cmd = [
        "gh", "issue", "create",
        "--repo", repo,
        "--title", issue["title"],
        "--body", issue["body"],
    ]
    for label in issue["labels"]:
        cmd.extend(["--label", label])

    result = subprocess.run(cmd, capture_output=True, text=True)
    if result.returncode == 0:
        print(f"  CREATED: {issue['title']}")
        print(f"           {result.stdout.strip()}")
    else:
        print(f"  FAILED:  {issue['title']}")
        print(f"           {result.stderr.strip()}")


def main():
    import argparse
    parser = argparse.ArgumentParser(description="Generate GitHub issues from plan backlog")
    parser.add_argument("backlog", nargs="?", default="05-implementation-backlog.xml",
                        help="Path to backlog XML")
    parser.add_argument("--repo", default="SufficientDaikon/Axon",
                        help="GitHub repository (owner/name)")
    parser.add_argument("--dry-run", action="store_true", default=True,
                        help="Preview without creating (default: true)")
    parser.add_argument("--create", action="store_true",
                        help="Actually create issues")
    parser.add_argument("--epics-only", action="store_true",
                        help="Create one issue per epic (not per story)")
    args = parser.parse_args()

    dry_run = not args.create
    backlog_path = Path(args.backlog)

    if not backlog_path.exists():
        print(f"ERROR: {backlog_path} not found")
        return 1

    print(f"=== Axon-Bend Issue Generator ===")
    print(f"Backlog: {backlog_path}")
    print(f"Repo:    {args.repo}")
    print(f"Mode:    {'DRY RUN' if dry_run else 'CREATING ISSUES'}\n")

    epics = parse_backlog(backlog_path)
    print(f"Found {len(epics)} epics\n")

    issue_count = 0
    for epic in epics:
        if args.epics_only:
            issue = format_issue(epic)
            create_issue(args.repo, issue, dry_run)
            issue_count += 1
        else:
            # Create parent epic issue
            issue = format_issue(epic)
            create_issue(args.repo, issue, dry_run)
            issue_count += 1

    print(f"\n{'Would create' if dry_run else 'Created'} {issue_count} issues")
    if dry_run:
        print("Run with --create to actually create issues on GitHub")

    return 0


if __name__ == "__main__":
    sys.exit(main())
