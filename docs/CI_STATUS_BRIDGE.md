# ChatGPT-Readable CI Status Bridge

**Repository:** `ekkus93/screenshot-ocr`  
**Default branch:** `master`  
**Status issue:** [#1 — CI Status: Hosted Quality Gates — master](https://github.com/ekkus93/screenshot-ocr/issues/1)  
**Monitored workflow:** `CI`  
**Publisher workflow:** `.github/workflows/publish-ci-status.yml`

## Purpose

This repository publishes the latest authoritative hosted CI state into a persistent GitHub issue. The issue lets a ChatGPT coding session discover the exact workflow run, tested commit, job IDs, step states, failures, timing, and artifact IDs without requiring local Rust, Node, or GitHub CLI tooling.

The issue is a discovery and indexing bridge. GitHub Actions jobs, logs, check runs, and artifacts remain the underlying evidence.

## Monitored target

| Workflow | Workflow file | Branch | Status issue |
|---|---|---|---|
| `CI` | `.github/workflows/ci.yml` | `master` | `#1` |

Runs for unrelated branches cannot overwrite issue #1. Pull-request CI still runs normally, but its source branch is not accepted by the `master` status publisher.

## Publisher behavior

The publisher listens for `requested`, `in_progress`, and `completed` `workflow_run` events. Before updating issue #1, it:

1. verifies that the triggering run's `head_branch` is exactly `master`;
2. queries the latest run for the same workflow and monitored branch;
3. skips the event if its run ID is no longer current;
4. verifies that issue #1 is open and contains the expected ownership marker;
5. reads all jobs, steps, and completed-run artifacts with explicit pagination;
6. produces a concise Markdown summary and parseable schema-versioned JSON;
7. checks the latest applicable run again immediately before publishing;
8. overwrites issue #1 in place.

The publisher uses only these token permissions:

```yaml
permissions:
  actions: read
  contents: read
  issues: write
```

It does not check out the repository, execute triggering-branch code, execute artifacts, publish raw logs, or expose environment variables.

## Hosted quality gates

The initial `CI` workflow runs on Ubuntu 22.04, the oldest supported production baseline. It always validates repository policy and produces a small metadata artifact so artifact indexing can be validated.

Rust and frontend gates are capability-driven during initial scaffolding:

- when a Rust manifest is present, CI installs Rust, Rustfmt, Clippy, and the Linux packages required for Tauri 2, then runs formatting, Clippy with warnings denied, and tests;
- when `package.json` is present, CI requires a committed lockfile, installs the selected package manager dependencies, verifies mandatory scripts, and runs frontend formatting, linting, typechecking, tests, and production build;
- before those manifests exist, the corresponding job records an explicit not-yet-scaffolded result rather than pretending that source validation occurred.

Once application scaffolding is committed, absence of the required scripts or lockfile becomes a hard CI failure.

## ChatGPT operating procedure

For every implementation candidate:

1. record the exact candidate commit SHA;
2. read issue #1;
3. compare `workflow.head_sha` in the JSON block with the candidate SHA;
4. do not claim success if the issue describes a different SHA;
5. use the published run ID and job IDs to inspect only the relevant failed logs;
6. fix the first meaningful failure and repeat;
7. require all mandatory jobs to report success on the same candidate SHA.

A hosted compile or test success does not substitute for Ubuntu desktop, Wayland/X11, OCR quality, clipboard, multi-monitor, or packaging validation required by the project TODO.

## Ownership marker

Issue #1 must always contain this marker:

```html
<!-- maintained by .github/workflows/publish-ci-status.yml -->
```

The publisher fails closed if the marker is absent or the issue is closed.

## Maintenance rules

- Keep the top-level workflow name exactly `CI`, or update the publisher's `workflow_run.workflows` entry in the same change.
- Keep one issue per monitored workflow/branch pair.
- Never reuse issue #1 for planning or bug discussion.
- Do not weaken real gates to make the published status green.
- Do not add a helper script unless a checked-in workflow uses it and deterministic tests cover it.
- If additional long-lived branches or authoritative workflows are introduced, create separate issues and isolated concurrency groups for them.
