# AI Collaboration Workflow

This repository uses GitHub as the single source of truth for DIAMANDIS ONE.

## Roles

### Claude Free (analysis and implementation proposals)
- Use Claude's GitHub integration to read selected repository files and keep a Project knowledge base synchronized.
- Review architecture, explain code, identify bugs, propose designs, and draft complete file changes or unified diffs.
- Do not claim to have cloned the repository, executed tests, pushed commits, created branches, or opened pull requests unless those actions were actually performed in a separate authorized environment.
- The standard Claude Free GitHub integration provides repository file context for selected files/branches. It does not provide commit history, pull request metadata, or a writable development workspace.

### ChatGPT (GitHub-connected maintainer)
- Inspect the actual repository state and relevant files through the connected GitHub tools.
- Translate accepted proposals into changes on a dedicated branch when the available GitHub actions permit it.
- Open pull requests, inspect diffs and CI results, and merge only after the validation policy below is satisfied.
- Never claim that local builds, Windows installation tests, or hardware tests passed unless there is direct evidence.

### Repository owner
- Approves scope and destructive changes.
- Reviews and merges changes when desired.
- Performs physical Windows testing when required.

## Source of truth and branch policy

- `main` is the only official, integrated baseline.
- Never develop directly on `main`.
- Each task gets one focused branch: `fix/<short-name>`, `feat/<short-name>`, `docs/<short-name>`, or `chore/<short-name>`.
- Do not create parallel branches for the same task.
- Before starting, compare the task branch with the current `main`.
- Open one pull request per coherent task.
- Do not merge a branch that is behind or has unresolved conflicts; update it from the latest `main` first.
- Close superseded pull requests only after confirming their unique changes are included elsewhere.
- After merge, use the new `main` as the base for every subsequent task.

## Standard task handoff

1. The owner describes one task and its acceptance criteria.
2. Claude receives the current project context and performs a read-only audit. It returns:
   - affected files and why;
   - root cause and evidence;
   - proposed implementation;
   - complete replacement contents or a unified diff;
   - tests to run and risks/limitations.
3. ChatGPT independently checks the relevant current files and proposal against the repository.
4. Accepted changes are applied to a dedicated branch, not directly to `main`.
5. Run the available CI and inspect every job. Record any test that could not run.
6. Review the complete diff for scope, security, regressions, and documentation.
7. Merge only when required checks pass and no unresolved high-risk issue remains.
8. Verify that `main` points to the merge commit. Start the next task from that updated `main`.

## Validation policy

For code changes, use the repository's current scripts and workflows rather than inventing commands. At minimum, inspect:
- frontend type-check/build;
- Rust formatting/checks/tests;
- Tauri packaging and NSIS installer workflow;
- dependency lockfiles and reproducibility;
- Windows-specific behavior when the change touches Windows APIs, drives, paths, launchers, or installation.

A green CI is not the same as a successful real-world Windows installation. Report these separately:
- CI passed;
- installer artifact generated;
- installer opened and installed on a clean Windows machine;
- application launch/update/uninstall tested;
- physical drive/game/save behavior tested.

Never label a pre-alpha build stable without the corresponding real Windows validation.

## Safety rules

- Preserve user data. Backups and restore operations must be verified.
- Do not delete, overwrite, or move user files without explicit safeguards and confirmation.
- Do not weaken path validation, symlink protections, or least-privilege behavior.
- Do not change WindowsApps permissions or take ownership of protected folders.
- Do not add telemetry, ads, mandatory cloud services, subscriptions, or external code execution without explicit approval.
- Do not commit secrets, tokens, credentials, private user data, or machine-specific paths.
- Keep dependencies and lockfiles consistent; explain any intentional dependency changes.

## Current project baseline

At the time this workflow document was added, the repository's `main` was verified at merge commit `c6c6f00bf62e9d0a2516e99090a591c87e70ffd1` (PR #7). The README describes the project as pre-alpha 0.2.0. This is a historical checkpoint, not a promise that it remains the latest commit. Always fetch the current `main` before starting work.

## First Claude task

Read the current repository context and perform a read-only audit of the current `main`, prioritizing:
1. Windows drive/volume enumeration and Dashboard storage display;
2. Cargo/npm lockfiles and reproducible CI;
3. Tauri permissions, filesystem path validation, and destructive file operations;
4. actual CI status and installer workflow.

Do not modify files or claim to have run tests from Claude Free chat. Return findings with exact file paths, relevant code excerpts, proposed diffs, and test commands. ChatGPT will independently verify and apply accepted changes through GitHub.
