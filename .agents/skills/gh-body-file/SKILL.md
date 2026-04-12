---
name: gh-body-file
description: Use when creating or updating GitHub issue bodies, pull request bodies, or substantial markdown comments with the gh CLI. Prefer a local markdown file and gh --body-file for multiline or markdown-heavy content; keep inline bodies only for short low-risk text.
---

# gh body-file

Use this skill for `gh` workflows that send substantial markdown to GitHub.

## Use This When

- creating a GitHub issue with more than a short one-line body
- editing an existing issue body
- creating or editing a pull request body
- preparing a substantial markdown comment or update where a file-backed workflow is safer and easier to review

## Do Not Use This When

- the body is a short one-liner with no quoting risk
- the task is about general GitHub triage, review, or CI debugging rather than composing a body payload
- the CLI path does not support file-backed body input and the content is simple enough to keep inline safely

## Working Rules

- Write substantial markdown content to a local file first.
- Prefer `--body-file` when the `gh` command supports it.
- Keep inline `--body` only for short low-risk text.
- Avoid shell-interpolated multiline markdown when it contains backticks, `$HOME`-style paths, angle brackets, fenced code blocks, or other content likely to break quoting.
- Prefer a local file because it is reviewable before submission and more deterministic for Codex.

## Common Patterns

```sh
gh issue create --title "..." --body-file /tmp/issue.md
gh issue edit 123 --body-file /tmp/issue.md
gh pr create --title "..." --body-file /tmp/pr.md
gh pr edit 456 --body-file /tmp/pr.md
```

## Expected Outcome

When using this skill, the resulting GitHub body workflow should be:

- file-backed for substantial markdown
- easy to inspect before submission
- resistant to shell quoting and interpolation errors
