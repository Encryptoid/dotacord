# Markdown module

This module provides small, focused utilities for rendering structured Markdown tables and link masking used elsewhere in the project. It is not a general-purpose Markdown library or a usage guide — this README explains the module's structure, goals, and the reasons behind the design.

## Goals

- Produce compact, deterministic Markdown table output suitable for posting to Discord messages.
- Keep rendering logic isolated from business logic and leaderboard aggregation.
- Keep the public API surface minimal; prefer crate-local visibility for helpers that are only used inside the crate.
- Support mixing textual and link columns in tabular output with consistent column-width calculations.

## What belongs here

- Column implementations (text and link) and a small TableBuilder that formats rows and columns into Markdown table lines.
- Lightweight helpers directly related to formatting content for output (for example, masking a URL behind a short symbol used in tables).

What does not belong here

- Leaderboard domain types and aggregation results. Those are owned by `crate::leaderboard` and the module only produces `Section`-shaped output consumed by the leaderboard code.
- High-level business logic (data fetching, stat aggregation, CLI/Discord command handling).

## File layout

- `mod.rs` — module entry point; re-exports the main types for convenient use.
- `text.rs` — `Text` column: header + list of string values, with optional inline-code rendering.
- `link.rs` — `Link` column: header + list of URLs, contains `LINK_SYMBOL` and a link-masking helper used to keep link markup consistent with table widths.
- `column.rs` — `Column` enum that unifies `Text` and `Link` variants for use by the table builder.
- `table.rs` — `TableBuilder`: builder-style API to add columns and produce a `Section` (the leaderboard-facing container of title + lines).

## Design and visibility choices

- Helpers and cell-formatting methods are `pub(crate)` when they need to be shared across the markdown submodules. This keeps the external public API small while allowing internal modules to collaborate without duplication.
- `TableBuilder::build` returns a `Section` type that lives under `crate::leaderboard` to keep leaderboard-related presentation data close to its consumer and disambiguate responsibilities.
- `link.rs` owns `LINK_SYMBOL` and `mask_link` to localize URL formatting behavior with the link column implementation; the module `mod.rs` re-exports those identifiers for convenience.
- Column width calculation, header formatting, and cell formatting are deterministic and designed to make text alignment predictable in monospaced Discord code blocks and tables.

## Reasons for implementing this way

- Separation of concerns: formatting code is easier to maintain and test when it is independent of data collection and ranking logic.
- Minimal, controlled API: exposing only what other parts of the crate need reduces accidental coupling and makes future refactors safer.
- Predictable output: by centralizing width calculations and masking logic, table output remains stable across changes to the rest of the codebase.

## Testing and maintenance notes

- Unit tests (if added) should focus on formatting correctness (width calculation, header/cell rendering, link masking) and not on leaderboard aggregation.
- If a new column type is required, implement it as a new variant of `Column` and add the necessary `pub(crate)` helpers for table building; avoid changing the public contract of `TableBuilder` unless strictly necessary.

## Contributing

- Prefer small, localized changes. If you need to change an exported signature, update the consumers in `crate::leaderboard` at the same time to keep the build green.
- Keep the module small — if formatting needs grow significantly, consider splitting renderers (table vs. other content) into submodules.
