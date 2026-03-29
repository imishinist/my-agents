## Coding Standards

### Purpose

Maintain consistent code style and quality across the project.

### Code Style

- Follow the project's linter and formatter configuration
- Match existing code style (do not introduce new patterns without discussion)
- Extract magic numbers and unexplained strings into named constants
- Keep functions and methods single-responsibility

### Naming

- Use names that convey intent
- Only use widely understood abbreviations (`id`, `url`, `api`, etc.)
- Prefix booleans with `is`, `has`, `can`, `should`

### Error Handling

- Never swallow errors silently
- Provide specific, actionable error messages for users
- Log unexpected errors

### Security

- Never hardcode secrets or credentials
- Always validate user input
- Watch for dependency vulnerabilities

### Comments

- Write comments that explain "why", not "what" (let code express the what)
- Include assignee or issue number in TODO comments
- Remove commented-out code that is no longer needed

### Language-Specific Rules

<!-- Customize this section based on the project's language and framework -->
