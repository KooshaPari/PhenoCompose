# Testing Strategy

- Validate edited JSON and YAML config for parse correctness in-session.
- Validate git diff and file presence for the requested hook/config surface.
- Defer live tool execution until dependencies are installed because this session cannot fetch npm packages.
