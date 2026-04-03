use anyhow::Result;
use crate::commands::shell_injection::execute_shell_commands_in_prompt;

pub async fn get_commit_push_pr_prompt(args: &str) -> Result<String> {
    let safe_user = std::env::var("USER").unwrap_or_else(|_| "developer".to_string());
    let default_branch = "main"; // Hardcoded for simplicity in port, could be fetched dynamically

    let mut raw_prompt = format!(r#"## Context

- `USER`: {}
- `git status`: !`git status`
- `git diff HEAD`: !`git diff HEAD`
- `git branch --show-current`: !`git branch --show-current`
- `git diff {}...HEAD`: !`git diff {}...HEAD`
- `gh pr view --json number 2>/dev/null || true`: !`gh pr view --json number 2>/dev/null || true`

## Git Safety Protocol

- NEVER update the git config
- NEVER run destructive/irreversible git commands (like push --force, hard reset, etc) unless the user explicitly requests them
- NEVER skip hooks (--no-verify, --no-gpg-sign, etc) unless the user explicitly requests it
- NEVER run force push to main/master, warn the user if they request it
- Do not commit files that likely contain secrets (.env, credentials.json, etc)
- Never use git commands with the -i flag (like git rebase -i or git add -i) since they require interactive input which is not supported

## Your task

Analyze all changes that will be included in the pull request, making sure to look at all relevant commits.

Based on the above changes:
1. Create a new branch if on {} (use USER from context above for the branch name prefix, e.g., `username/feature-name`)
2. Create a single commit with an appropriate message using heredoc syntax:
```
git commit -m "$(cat <<'EOF'
Commit message here.

Co-authored-by: Claude Code <claude-code@anthropic.com>
EOF
)"
```
3. Push the branch to origin
4. If a PR already exists for this branch (check the gh pr view output above), update the PR title and body using `gh pr edit` to reflect the current diff. Otherwise, create a pull request using `gh pr create` with heredoc syntax for the body.
   - IMPORTANT: Keep PR titles short (under 70 characters). Use the body for details.
```
gh pr create --title "Short, descriptive title" --body "$(cat <<'EOF'
## Summary
<1-3 bullet points>

## Test plan
[Bulleted markdown checklist of TODOs for testing the pull request...]

Co-authored-by: Claude Code <claude-code@anthropic.com>
EOF
)"
```

You have the capability to call multiple tools in a single response. You MUST do all of the above in a single message.

Return the PR URL when you're done, so the user can see it."#, safe_user, default_branch, default_branch, default_branch);

    let trimmed_args = args.trim();
    if !trimmed_args.is_empty() {
        raw_prompt.push_str("\n\n## Additional instructions from user\n\n");
        raw_prompt.push_str(trimmed_args);
    }

    execute_shell_commands_in_prompt(&raw_prompt).await
}
