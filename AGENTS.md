# http-server-rs — agent instructions

HTTP server cli utility for development

## Response style

Applies to the top-level agent talking to the user, **not** to a sub-agent.

- Lead with the outcome. No preamble, no restating the task, no conversational filler.
- Use short sections or bullets only when they aid parsing.
- Keep the structure sleek and functional.

## Comments

Do not leave comments unless absolutely necessary. The default is no comment; code carries the meaning. A comment earns its place only when it states something the code cannot — a non-obvious reason, a constraint, or a platform quirk — and nothing else will do.

- Never comment the obvious: no narration of the next line, no restating a name, type or signature, no section banners or `//` separators.
- When in doubt, omit the comment. A missing comment is the normal case; an added one must be defensible.
- Comment intent or a reason, never a description of what the code does — if a comment merely restates the code, delete the comment, not the code.
- Do not add doc comments to code that already reads clearly from its type and name.
- Add a comment only when omitting it would plausibly cause a wrong "fix" later.
- Leave existing comments alone. Removing or rewriting them is a separate, explicit request.

## Validation — delegate to the `verifier` role

Do not run the validation loop yourself. When you make a change, delegate validation to a sub-agent using the `verifier` role.

- Load the `verifier` skill and pass its procedure to the sub-agent verbatim; a sub-agent does not see this conversation's context.
- The sub-agent owns `cargo check --workspace` and `cargo xfmt`. Do not duplicate that work in the main thread.
- Accept only the summary it returns. Never paste raw cargo output into the conversation unless the loop failed and the failure itself is the finding.
- If it reports failure after 5 iterations, surface the error log and what was attempted, then decide the next move.
- While the sub-agent runs, keep working only on steps that do not depend on a compiling tree.

## Commands

| Command | Purpose |
|---|---|
| `cargo check --workspace` | Validation gate for every change. |
| `cargo xfmt` | Format after `cargo check` passes. |
| `cargo run` | Launch the GUI locally. |
