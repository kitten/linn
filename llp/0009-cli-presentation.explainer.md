# LLP 0009: CLI presentation

**Type:** Explainer\
**Status:** Active\
**Systems:** CLI, Presentation\
**Related:** LLP 0004, LLP 0007, LLP 0008

## Boundaries

The CLI owns presentation, independently of application execution, retrieval and
local IPC. Human terminal output should be easy to scan while captured output
remains compact and precise for agents. Styling never changes eligibility,
ranking, excerpts, original spans, references, command exit codes or consent.
This contract spans parsing/help, ordinary output, diagnostics and mesh progress;
LLP 0004 owns command semantics and LLP 0008 owns synchronization semantics.

## Presentation policy

[The presentation module](../src/cli/presentation.rs) resolves the policy once per
invocation and independently checks stdout and stderr terminal capability.
Non-terminal streams retain the compact plain layout. Any nonempty `NO_COLOR`,
`TERM=dumb`/`unknown`, `CI=1`/`true`, or a zero-valued `CLICOLOR`, `COLOR` or
`FORCE_COLOR` disables all generated styling, extra spacing, human layout and
progress rewrites on both streams. Comparisons are ASCII case-insensitive except
NO_COLOR, whose nonemptiness alone matters. Empty NO_COLOR and false/zero CI do
not disable presentation. Nonzero force-color settings never override a disable
or turn a non-terminal stream into a styled stream. `--json` disables decoration
on both streams, including help and diagnostics. There is no extra plain flag.

These are presentation inputs, not retrieval knobs. A PTY without disable signals
can receive human formatting; terminal capability cannot prove human presence.
An inherited disable environment supplies the explicit plain-output contract.
Prompt consent, token input and terminal capability remain distinct decisions. Enrollment-aware
command discovery modifies only top-level help. A subcommand's `--help` and
`help <subcommand>` retain that subcommand's arguments, options and instructions,
including when global store/JSON options precede it.
No terminal escape queries, subprocesses, animation threads, automatic pagers,
alternate screen or cursor hiding are introduced.

## Visual hierarchy and original bytes

Use one terminal-palette accent for existing search/outline navigation markers,
bold titles, and dim supporting metadata. Related pointers remain secondary;
near-miss headings use warning emphasis and retain the words “not results”.
Successful outcome labels use green, warnings yellow and errors red. Text always
carries the meaning independently of color. Avoid decorative symbols, guide
rails, padded tables or banners that add content without helping a decision.

Search and outline separate entries and indent every excerpt line only in human
presentation. List puts the readable label before its unchanged identifier, kind
and exact timestamps. Keep complete references and suggested commands copyable.
Show styles only its header and truncation notice; original body bytes are not
styled or rewritten. Add a presentation newline when needed on a human terminal
so a notice or shell prompt does not attach to the body. Plain show preserves its
existing byte layout, including missing trailing newlines.

JSON values and raw export bytes remain unchanged. Invitation tokens remain
unstyled, with a heading only in human presentation. Safety notices and essential
diagnostics remain visible in every mode. Do not interpret original content's
control characters as newly generated decoration or claim that plain mode
sanitizes original text.

## Transient progress

[The mesh renderer](../src/cli/mesh.rs) renders explicit sync/first-sync progress
on stderr from typed events. It does not add progress to routine startup, search,
add or rebuild operations. Plain, JSON and unknown-width output remains deduplicated
line-based stderr text, with no generated cursor controls.

For human presentation with known native width, own at most one physical row.
Generated phase/count messages are ASCII; fit within width minus one column to
avoid wrapping. Clear the row with carriage return and erase-line before a redraw,
and flush writes and clears. Deduplicate identical messages, but inspect resize
before deduplication. On an observed width change, stop erasing and fall back to
line output because reflow can move the old status across rows.

Limit same-phase counter redraws to five per second, coalescing to the latest
available event; phase changes remain immediate. This is event-driven, with no
polling or animation timer. Show real phases and counts rather than an invented
overall percentage, ETA or stall assertion. LLP 0008 defines those counters.

Clear and flush before results, diagnostics and prompts, then leave the ordinary
final summary visible. Cleanup is idempotent and only erases an owned active row.
Drop is a best-effort fallback, not a guarantee after forced termination. Never
suppress essential diagnostics behind a hidden progress display or imply that
accepted partial progress was rolled back.

## Validation

[Headless terminal tests](../tests/terminal_presentation.rs) exercise human output,
per-stream capability, environment disables, help/errors, JSON and original-body
preservation. Corpus-free policy and screen-model tests cover disable precedence,
shorter updates, coalescing, resize, unknown/narrow width and idempotent cleanup.
Captured plain output should match the baseline byte-for-byte except corrected
command help. Cursor clocks
remain context-bearing values rather than a presentation difference. Rendering
checks do not establish retrieval quality or a broad agent-latency claim.
