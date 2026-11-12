# Collaboration and project roles

This document records the collaboration arrangement for PROPHET-XII. It does not transfer repository ownership or retroactively assign authorship of existing commits.

## Project roles

- **Repository Owner / Original Project Publisher:** [AdhonisBR](https://github.com/AdhonisBR)
- **Lead Developer / DSP Engineering:** [SalatielBM22](https://github.com/SalatielBM22)
- **UI / Integration Contributor:** [AdhonisBR](https://github.com/AdhonisBR)

These are project responsibilities and acknowledgments, not assertions about the individual authorship of every file or prior commit. Roles may be refined against the actual contributions and commit history.

## Development and integration

- Keep independent feature branches for concurrent development. Target `main` through pull requests after the relevant work is ready.
- Authorized collaborators with write access can propose changes and merge eligible pull requests, subject to the repository's active branch rules and CI requirements. Do not make approval exclusively dependent on the repository owner.
- Preserve the `v1.0.0-experimental` tag and its original baseline commit `b47f2d60c7005cd6c81694c0e17c76f7a4f107dc`.
- Never force-push or delete `main`; avoid rewriting history and destructive operations on working branches.
- Require automated checks only once configured and reliably passing. Do not require a third-person review when the active team cannot provide one.
- Do not merge incomplete agent work. Local uncommitted changes and Codex workspace checkpoints must be preserved by the respective developers.

## Authorship

Git commit authorship remains attached to the original authors of commits. For genuinely joint changes, use Git commit trailers such as `Co-authored-by: Name <authorized-github-commit-email>` only with each participant's approved commit email address. Never invent emails or rewrite historical commits to change attribution.

## Administrative scope

Repository collaborator invitations, effective access levels, branch protections and rulesets are managed through GitHub settings by an authorized administrator. This file describes the intended governance; it does not itself create or enforce repository protections.

## Record of baseline

On 2026-11-12, the published `main` and `v1.0.0-experimental` refs both pointed to `b47f2d60c7005cd6c81694c0e17c76f7a4f107dc` at the time of the collaboration review. This document-only change is proposed on an isolated branch rather than directly modifying that baseline.
