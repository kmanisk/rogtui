# Contributing

Contributions are welcome. Keep terminal UI and hardware-control changes focused, explicit, and easy to validate.

## Workflow

1. Branch from `main`.
2. Follow the setup and run instructions in `README.md`.
3. Keep each pull request limited to one logical feature, fix, or documentation change.
4. Run the relevant formatter, linter, build, and tests available in the repository.
5. Describe the affected controls, expected behavior, and verification performed.

For hardware-facing changes, document supported devices, required privileges, and safe fallback behavior where relevant. For TUI changes, note keyboard bindings and rendering changes. Do not commit credentials, machine-specific state, generated build output, or local caches.
