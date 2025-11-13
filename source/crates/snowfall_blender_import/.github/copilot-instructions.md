# Copilot Instructions

### User conversation

- Use a concise, straight to the point conversational style
- Describe things precisely. Do not use metaphors.
- When conversing with the user, do not use overly enthusiastic comments like "Perfect!", "Great!", or "Good!"

### Code comment style

- Avoid adding comments that are obvious statement about what the code is doing
- Prefer high-level comments about the architect to comments about individual lines of code
- Mention when common design patterns are being used

### Code style

- Prefer early returns to deeply nested conditionals
- If you create a backup file during a task, be sure to remove it when the task is done

### Testing style

- Prefer a table-driven testing structure when possible
- Prefer storing test input and output in YAML files or other data files when possible rather than hard-coding data into the Rust source code
