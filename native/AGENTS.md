# Native desktop workflow

- Generate local Windows previews with `native/scripts/package-preview.ps1` from the repository root. The only preview entry point is `artifacts/preview/Rockoon.exe`.
- When the preview is occupied, close the process identified by that exact executable path and replace the same file. Do not create numbered, dated, or feature-named executable copies.
- Keep distributable copies in `artifacts/preview` or `artifacts/releases/<tag>`, temporary diagnostics in `artifacts/tmp` or `artifacts/logs`, and compiler caches in `native/target`. Do not scatter generated files in Downloads or the source tree.
- User-state backups belong in `artifacts/backups` and must be preserved during generated-file cleanup.
- The user tests the interface. Do not run GUI or screenshot tests unless explicitly requested.
