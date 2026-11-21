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
Use sentence case for human labels, standalone statuses, progress and diagnostics:
capitalize the first word, preserving the remaining casing and exact identifiers.
Values after a label, such as `Relay: connected`, remain lowercase.
Successful outcome labels and reachable peer states use green; unknown and
offline states use yellow; other peer failures use red. Peer IDs remain dim and
neutral, and local-store labels use bold. Resolve diagnostic colors against stderr
capability independently of stdout. Transient progress stays uncolored. Text always
carries the meaning independently of color. Avoid decorative symbols, guide
rails, padded tables or banners that add content without helping a decision.

Search and outline separate entries and indent every excerpt line only in human
presentation. List puts the readable label before its unchanged identifier, kind
and exact timestamps. Keep complete references and suggested commands copyable.
Show styles only its header and truncation notice; original body bytes are not
styled or rewritten. In every text mode, append one presentation newline to a
nonempty displayed span that lacks a final newline, so the truncation notice,
next result or shell prompt cannot attach to the body. Preserve existing trailing
newlines without adding a blank line. This separator also applies to `show --all`
and is the explicit compatibility correction to the previous plain byte layout.
JSON and exports supply exact original text when its final newline matters.

Interactive prompt lines end explicitly on stderr after input and on error or
cancellation; input echo is not evidence that stderr received a newline. Never
attach a diagnostic or follow-up instruction to a partially written prompt.

Human sync progress shows receiving/sharing phases without traversal rounds,
levels or contact queues. Final peer totals exclude this store.
Progress keeps sent/received change counts; completion shows applied/sent counts.
Unreachable peers have one human diagnostic rather than separate phase and failure
lines. Keep other failure reasons and recovery instructions on the same line. The final human
summary leads with completion or partial status, then stores synced and changes
applied locally. It never claims that no local changes proves all stores are up
to date. Partial results retain connection warnings and repair/retry diagnostics.
Plain sync summaries and progress retain their existing detailed format.

Human sync status presents transport, relay and discovery on separate labelled
lines and places peer status before complete peer IDs. Across human command
output, progress and diagnostics, put peer IDs last in parentheses after the
status, action, reason and recovery advice. Keep full IDs copyable; do not
shorten them. For a connection involving two peers, label source and target
inside a trailing parenthesis. Plain status retains its
compact line and ID-first peer format. A human close confirmation identifies
successful closure and labels the returned ID as the close record, not the
target record. Plain close output remains the bare new envelope ID. An empty
list says `No matching records.` so filters do not imply the store is empty.

JSON values and raw export bytes remain unchanged. Invitation tokens remain
unstyled, with a heading only in human presentation. Safety notices and essential
diagnostics remain visible in every mode. Do not interpret original content's
control characters as newly generated decoration or claim that plain mode
sanitizes original text.

## Other command outcomes

Human write confirmations lead with `Saved`, `Closed.` or `Forgotten`, followed
by dim IDs in parentheses. The close confirmation labels its ID `Close record`:
it identifies the new immutable close envelope, not the target. Plain writes
retain their existing output. Human integrity checks say `Integrity check passed.`;
rebuild retains `Index rebuilt.`. Import separates its outcome from applied-change
and already-present counts, with singular/plural wording. Follow with author
counts labelled `Authors:`, including unknown writers.
Show deletion-marker counts separately when nonzero, with no author attribution.

Human list entries put the original label first, then readable kind and labelled
creation/expiry values, then the full record ID last on its own dim line. Preserve
exact timestamps, labels and copyable pagination commands. Append `Author: <email>`
to the existing metadata line, never add a new line. Omit this suffix only when
the exact stored writer matches the current effective Git email resolved by the
CLI; unknown attribution displays `Author: Unknown`. Familiarity or a matching
name is not sufficient to infer a local author. JSON always retains the stored
writer, and filters operate on that original value regardless of display omission.

Human daemon output leads with running/stopped state and an optional PID, followed
by the store path and version. Stopped is neutral; running is green. Human sync
administration uses pairing, connection and leaving outcomes rather than internal
membership revisions; full revisions remain in plain/JSON output. Conflict and
pending-leave warnings remain explicit. Pairing consent and token bytes are unchanged.

A successful named-file export gets a concise confirmation on human stderr only,
after the file is safely persisted. Stdout bundle bytes, plain output and JSON
remain unchanged; a streamed export receives no extra confirmation.

## Transient progress

[The mesh renderer](../src/cli/mesh.rs) renders explicit sync/first-sync progress
on stderr from typed events. It does not add progress to routine startup, search,
add or rebuild operations. Plain, JSON and unknown-width output remains deduplicated
line-based stderr text, with no generated cursor controls.

For human presentation with known native width, own at most one physical row.
Generated phase/count messages use ASCII with a one-column Braille spinner; fit
within width minus one column and truncate only at UTF-8 character boundaries to
avoid wrapping or splitting a frame. Clear the row with carriage return and erase-line before a redraw,
and flush writes and clears. Deduplicate identical messages, but inspect resize
before deduplication. On an observed width change, stop erasing and fall back to
line output because reflow can move the old status across rows.

Limit same-phase redraws to five per second; phase changes remain immediate.
An async 200-millisecond presentation timer animates the Braille frames
`⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` in order while
sync events are pending, even when no new event arrives. It never polls the
store or network and stops before results, diagnostics or prompts. Plain, JSON
and unknown-width streams never animate. Show real phases and counts rather than an invented
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
Captured plain output should match the baseline byte-for-byte except documented
clarity corrections: show/prompt boundaries, empty-list wording and command help.
Cursor clocks
remain context-bearing values rather than a presentation difference. Rendering
checks do not establish retrieval quality or a broad agent-latency claim.
