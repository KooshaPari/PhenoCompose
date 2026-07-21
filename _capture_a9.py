#!/usr/bin/env python3
"""Capture pre-deletion state for A9 bulk-close."""
import subprocess

BRANCHES = [
    "chore/CC1-005-sota-2026-06-11",
    "chore/CC2-005-sota-2026-06-11",
    "chore/CC3-005-sota-2026-06-11",
    "chore/CC4-005-sota-2026-06-11",
    "chore/QC1-005-sota-2026-06-11",
    "chore/QC2-005-sota-2026-06-11",
    "chore/QC3-005-sota-2026-06-11",
    "chore/QC4-005-sota-2026-06-11",
    "chore/SD2-004-sota-2026-06-11",
    "chore/SD3-004-sota-2026-06-11",
    "chore/SD4-004-sota-2026-06-11",
]


def run(cmd):
    return subprocess.run(cmd, capture_output=True, text=True).stdout.strip()


def main():
    main_sha = run(["git", "rev-parse", "origin/main"])
    out_lines = [
        "branch|tip_sha|commit_date|subject|merged_into_main|ahead_of_main|behind_main|archive_tag|archive_sha|matches"
    ]
    for b in BRANCHES:
        sha = run(["git", "rev-parse", f"origin/{b}"])
        info = run(["git", "log", "-1", "--format=%cI|%s", sha])
        cdate, subject = info.split("|", 1) if "|" in info else (info, "")
        merged = (
            subprocess.run(
                ["git", "merge-base", "--is-ancestor", sha, main_sha],
                capture_output=True, text=True
            ).returncode == 0
        )
        ahead = run(["git", "rev-list", "--count", f"{main_sha}..origin/{b}"])
        behind = run(["git", "rev-list", "--count", f"origin/{b}..{main_sha}"])
        # Archive tag is e.g. archive/CC1-2026-06-11 (no -005 suffix)
        prefix = b.split("/")[1]
        tag_short = prefix.split("-")[0]
        archive = f"archive/{tag_short}-2026-06-11"
        tag_sha_raw = run(["git", "rev-parse", archive])
        tag_sha = tag_sha_raw[:12] if tag_sha_raw else "MISSING"
        matches = "YES" if tag_sha_raw == sha else "MISMATCH"
        out_lines.append(
            f"{b}|{sha[:12]}|{cdate}|{subject}|{merged}|{ahead}|{behind}|{archive}|{tag_sha}|{matches}"
        )
    with open("audits/branch-clean/evidence/A9-pre-remote-refs.txt", "w") as f:
        f.write("\n".join(out_lines) + "\n")
    print("\n".join(out_lines))


if __name__ == "__main__":
    main()