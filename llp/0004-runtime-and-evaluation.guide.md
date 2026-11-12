# LLP 0004: CLI and packaging

**Type:** Explainer\
**Status:** Active\
**Systems:** CLI, Paths, Packaging\
**Related:** LLP 0000, LLP 0001, LLP 0002, LLP 0003, LLP 0005, LLP 0006

## Runtime

One native Rust binary uses Clap-derived arguments, typed errors, and separate parsing, execution and presentation. Results go to stdout; warnings, errors and opt-in diagnostics to stderr. Resolve paths in order: --store, LINN_STORE, store in $XDG_CONFIG_HOME/linn/config.toml, then $XDG_DATA_HOME/linn/store.db. Defaults are ~/.config and ~/.local/share. Relative XDG values are invalid, including when an explicit store is supplied. Never default to cwd.

Routine internal debug statistics, including POS projection publication counts
and elapsed time, are not printed by default. Existing explicit timing and trace
diagnostics remain opt-in. Routine indexing, rebuild, sync and export progress
messages are omitted; warnings and errors stay on stderr.

The README agent prompt recommends command help when syntax or behavior is
unclear, rather than a mandatory help call each session. Top-level help supports
command discovery and retains brief workflow and evidence-use guidance. Search
and add help are self-contained for their tasks: search includes shared-store
scope, opening results and distinguishing hints; add includes original findings,
sources and corrections. Search help expands syntax, filters, references and
pagination without requiring fetch-ID citations. Query guidance favors specific source
words and exact identifiers, with distinguishing context as needed, rather than
a prescribed word count.

## Command structure

```
linn <command>[:<subcommand>] [arguments] [--json]
```

Commands are in three groups. An agent that reads memory needs only the first.

| Group | Command | Purpose |
|---|---|---|
| Read | `linn list [options]` | Current record metadata, newest first or nearest expiry first |
| | `linn search '<query>'` | Up to 5 results and related pointers, or "Nothing qualified." with near misses |
| | `linn search:more <cursor>` | The next results of the same search: more sections of the same records, then linked records |
| | `linn show <ref> [--all]` | The original bytes a reference names (a span, or a whole record): at most 1,500 characters, or all with `--all` |
| | `linn show:outline <ref>` | The outline of the document a reference names, as openable entries |
| Write | `linn add <file\|->` | Import a file as a document, or store standard input as a note; searchable at once (Writing: `add`, `add:close`, `add:forget`) |
| | `linn add:close <ref>` | Write a close envelope: `--reason supersede\|correct\|expire [--replacement <ref>]` |
| | `linn add:forget <ref>` | Delete a record and its index rows securely |
| | `linn export [--scope <repository>] [--kind note\|doc] [--since <duration\|timestamp>] [--id <ref>]... [--all-states] [--output <file\|->]` | Export a filtered original-data bundle |
| | `linn import <file\|->` | Merge an original-data bundle; preserve identities |
| Maintain | `linn index:check` | Run the integrity check; exit 0 if clean, 3 if damaged |
| | `linn index:rebuild` | Rebuild sections and the index from stored bodies and envelopes |
| | `linn sync:remote pair\|remove\|list` | Configure trusted SSH peers |
| | `linn sync` | Pull every peer before pushing merged records and deletions |
| | `linn sync:serve [--stdio]` | Serve the SSH protocol without a listener |

- **Scope.** The scope of a directory is computed, never configured:
  1. If the directory is not inside a git repository, it is unscoped.
  2. Otherwise, take the repository's remotes (a worktree shares its repository's). Use `upstream`
     if it exists, else `origin`, else the only remote.
  3. The scope is that remote URL's path without host and `.git`, usually `<owner>/<repo>`. SSH,
     HTTPS and `ssh://` forms of one URL give the same scope; a GitLab subgroup keeps every part.
  4. A repository with no usable remote (none, or several without these names) gets
     `local:<path of the repository>`, so its records stay with that repository on this machine.

  Scopes compare case-insensitively.
  Read remote names and resolved fetch URLs together with `git remote -v` after
  establishing the working-tree root. Preserve Git's URL rewrites, first fetch
  URL, remote precedence and local fallback; push URLs never supply scope.
  Unrecognized verbose output falls back to separate remote enumeration and
  `git remote get-url`. No persistent repository cache is used.

  **What a search sees.** The current records of every repository: knowledge is shared (operator,
  2026-11-07). Search has no `--scope` option. A `scope:` term is parsed and, in this store, does not
  remove rows (Eligibility). The caller's repository names a new record's scope and decides the penalty
  of 7 (Scoring (BM25 over the searched rows)). It is not printed on a call-1 card. The store does not inspect cwd; the CLI derives caller scope from Git as above.
- **Output.** Plain text by default. `--json` prints one JSON object (schema in JSON output (`--json`)).
- **Search is deterministic.** `linn search` prints a cursor, which `search:more` takes. A cursor is
  an opaque, self-contained token that encodes the query, the scope and the next call to print, so
  the store keeps no session state. The same cursor always gives the same output while the store
  is unchanged.
- **What the CLI adds to the store's calls (Search algorithms).** Ranking, matching and excerpts are the store's.
  The CLI:
  - accepts the filters `scope under path kind id writer` only. Any other prefix is a query error,
    including those the store also knows (`cites linked-from version-of replaces sibling-of heading
    body`).
    - `path:` matches the source path, so it covers what `cites:` did.
    - `id:<ref>` takes any reference (Identifiers and references), quoted or bare.
      Resolve it once to a record ID for both execution and final card/pointer filtering.
      Rewrite only parsed filter values, never equal text elsewhere in the query.
      Cursors retain the caller's original query and resolve it again on replay.
      An ID containing a double quote cannot be expressed by this query grammar;
      report an error and direct the caller to `show`.
  - applies the query's filters to every call, to the near misses and to the related pointers
  - never prints a card twice within one search, and `search:more` skips a call left empty
  - marks a document with `(doc)` after its label
  - numbers cards `[1]`, related pointers `[r1]`, near misses `[h1]` and outline entries `[o1]`

## Export and import

Top-level `export` and `import` transfer existing records; `add` creates new ones.
Bundle format, selection, deletion privacy, lifecycle validation and recovery
are specified in LLP 0006#file-export-and-import. Export defaults to stdout;
import accepts a file or stdin (`-`). Full original-data backups use unfiltered
`export --all-states`.

## Synchronization

`sync:remote pair <name> <ssh-origin> [--scope <repository>]`,
`sync:remote remove <name>` and `sync:remote list` manage store-local remotes.
`sync` pulls all remotes before pushing the merged records and tombstones.
`sync:serve [--stdio]` serves SSH sessions over stdin/stdout without a listener.
The protocol, conflicts, scope selection and recovery rules are in LLP 0006.

## Commands in detail

### `linn search '<query>'`

Runs `search` (Search algorithms), applies Command structure's CLI rules, and prints call 1.

- **Cards.** At most 5. Call 1 is the page of Call-1 page. Two non-overlapping blocks of one record can
  both appear. A tested call-1 card shows its heading path and the excerpt from Call-1 page. It does not
  show the repository. Cards from call 3 also show their edge label.
- **Related pointers.** Up to 2, printed after the cards under `Related (heading match, not
  results):`. Each has its fetch id.
- **Silence.** A silent search prints `Nothing qualified.` and up to 3 near misses under `Near
  misses (not results):`. Each shows its label, heading path, `matched:` and `missing:` words, the
  excerpt and a fetch id. A query with filters and no words prints `Nothing qualified. Supply text
  terms, or use show/outline navigation.`
- **`scope:<org/repo>`.** Accepted by the parser. In this store it does not narrow BM25, the
  ranking, near misses, or related pointers, because every record is stored with a null scope
  (Eligibility). The original scope is used only for the penalty of 7 (Scoring (BM25 over the searched rows)).
- **Query errors.** An unknown filter or an unclosed quote prints `Query error: <reason>` and exits
  2.

### `linn search:more <cursor>`

Prints the next call of the search that the cursor names. A call left empty by the CLI's rules is
skipped. After the last call it prints `No more results for this search.` and exits 1.

### `linn show <ref> [--all]`

Prints the record's label and heading path, then the original bytes the reference names: the span
of a fetch id, or the whole body for a record id or a hash.
- **Limit.** At most 1,500 characters. If the span is longer, it ends with `… (N more characters;
  --all to print the rest)`.
- **Current records only.** The record must be accepted and current: not a close envelope, not
  closed, not superseded, not expired. A fetch id comes from a search, so a result found with
  `scope:` stays openable.
- **Old versions.** The body hash in the fetch id pins the version. For a superseded or closed
  record, `show` prints `Not current: superseded by <id>.` (or `closed by <id>`) and exits 1. This
  design never shows history.

### `linn show:outline <ref>`

Prints the outline (Outline) of the record a reference names: numbered entries `[o1]`…, each with its
heading path, first line and fetch id. A record that is not current prints `Nothing qualified.` and
exits 1.

### Writing: `add`, `add:close`, `add:forget`

```
linn add <file> [--label <text>] [--unscoped] [--parent <ref>] [--stale-after <time>]
linn add -      [--label <text>] [--unscoped] [--parent <ref>] [--stale-after <time>]
                [--source-path <path>]
```

- **Kind comes from the input; there is no `--kind`.** `add <file>` imports the file's bytes
  unchanged as a `doc`, with `source_path` set to the file's path relative to the checkout. `add -`
  stores standard input as a `note`. Do not infer kind from headings, source type or presence of source path.
- **Label is optional.** Without `--label`, a document gets its single top-level heading, else its
  file name, and a note gets its first line, cut at 80 characters. Top level means
  the shallowest heading level present. Notes may contain headings too; kind is
  determined by input mode, never by presence or absence of headings.
- **Source path is optional.** It is derived for a document. For a note it names the file the note
  is about, if any. The store treats a missing path as empty in link resolution, `path:` and
  linked-record navigation.
- **Scope.** The record gets the scope of the repository you are in (Command structure); `--unscoped` ties it to
  none. Either way it is found from every repository. The original scope is the penalty of 7
  (Scoring (BM25 over the searched rows)). It is not a call-1 card field, and `scope:` does not filter (Eligibility). The short-page
  fills do not apply the penalty (Call-1 page).
- **New records.** A record is searchable at once, with no review
  step. The scored word search still requires `state` accepted (Eligibility). A wrong note is closed
  (`add:close`) or replaced (`--parent`).
- **Versions.** `--parent <ref>` makes it a new version of an earlier record. Once it is accepted, the
  parent is superseded and leaves search.
- **Links in the text.** A relative link to a repository file and a `linn:<ref>` link to another
  record remain in the original text. The path-link index resolves relative
  destinations; `linn:` references are checked for warnings by the CLI and retained
  as written by that index, as specified in LLP 0001. `add` lists unresolved `linn:` links.
- **Embedding.** `add` embeds the record's blocks (Embeddings). Nothing else is asked of the writer.
- **The other commands:**
  - They take any reference to the record. `add:close` records the reason and the configured Git email (or null), like other new records.
  - `add:close` writes an envelope and never edits the target.
  - `add:forget` deletes the record and its index rows with secure delete.

### Maintaining: `index:check`, `index:rebuild`

- `index:check` runs SQLite quick-check, validates original body/envelope hashes and
  their ID/body agreement, checks exact block/vector coverage and vector byte length,
  and runs FTS5 integrity on both tables in one transaction. It diagnoses corruption;
  it does not claim to prove every derived ranking value or repair damaged originals.
- `index:rebuild` derives sections, the index and validity again from stored bodies and envelopes.

### Exit codes

Return the command's exit code from `main` after the async runtime shuts down;
do not terminate the process from inside the running runtime.

| Code | Meaning |
|---|---|
| 0 | Results printed, or the command succeeded |
| 1 | Nothing qualified, no more results, or the record is not current |
| 2 | Usage, query, storage, transport or other command error |
| 3 | Integrity check failed |

## JSON output (`--json`)

```json
{"cursor": "s1.k7Q2…", "call": 1, "status": "results" | "nothing_qualified",
 "cards":   [{"n": 1, "record": "…", "label": "…", "heading": "A > B", "excerpt": "…",
              "fetch": "<hash12>:v1:<start>:<end>", "labels": []}],
 "related": [{"record": "…", "label": "…", "heading": "…", "excerpt": "…", "fetch": "…",
              "matched": ["…"], "missing": ["…"]}],
 "hints":   [{"n": 1, "record": "…", "label": "…", "heading": "…", "excerpt": "…", "fetch": "…",
              "matched": ["…"], "missing": ["…"]}],
 "more": true}
```

`show --json` prints `{"fetch", "record", "label", "heading", "text", "truncated", "superseded_by"}`.


## Cursor contract

The opaque s1 cursor is URL-safe unpadded base64 of JSON (query, caller scope, next call, eligibility clock). Replay reruns the same search with the captured clock, filters all calls and suppresses duplicate cards, skipping empty calls. No persistent session is needed.

## Native build

Use the Nix flake for cargo fmt, cargo clippy --workspace --all-targets --all-features -- -D warnings, cargo test --workspace --all-features, and nix build. The build assets contain config.json, tokenizer.json and model.safetensors for the pinned SSE v2 model in LLP 0002. Fetch weights and tokenizer from revision-pinned, SHA-256-checked upstream URLs at build time; generate the byte-stable Linn model descriptor. Never download on ingest/search or commit weights. No developer-local path is a flake input. The installed binary embeds the model and requires no Nix-store dynamic dependency. See LLP 0005 for local scoring.

The global options are `--store <path>` and `--json`; `--help` is available on
all commands. There is no search `--scope` or model setup/download command; `sync:remote pair --scope` selects records for exchange only. Both file and stdin ingest
capture Git email attribution; all new records are accepted. File source type is repo-file
with the evidence hash as ref; stdin source type is derived-from-thread with null
ref. Close envelopes also capture Git email attribution and have default source metadata. Ingest warns
on stderr about unresolved linn links while still storing the original body.

Write a note when a decision, gotcha, procedure, or project fact would change a
later task. Keep it specific, with the actual command, identifier, version, or
error text and links to supporting files. This is writing guidance, not an
acceptance workflow: new CLI records are accepted immediately. Retrieval gives
leads, not a guarantee that a stored claim is still true of the current checkout;
open and check the cited bytes before relying on a result.

## Linguistic build input

The POS selector uses Linn's `src/nlp` runtime with the pinned English
nlprule 0.6.4 tokenizer data and the same regex semantics. The flake fetches and decompresses the official
release only at build time and verifies SHA256
`e2b51e506aede495268ee05dee0b97956e768107f5bb5891585144a5ff2fd773`.
`build.rs` independently verifies those bytes and converts them into a
reproducible, lookup-ready `english.nlp` asset. The binary embeds that asset;
dictionary FST entries, word ids, tag order, rules and chunker values are
preserved. Runtime uses one lazy process-local load, with no external asset path or runtime
network dependency. Existing stores need `index:rebuild` to prepare the projection;
normal queries never silently retag the corpus. POS is always enabled with
fixed bonus weight 0.75. There are no runtime algorithm settings for ingest or
search. Store-location, output-format and diagnostic controls do not change the
algorithm. LLP 0005 records
that q4 and h2 were both used to select this adjustment. The unchanged nlprule
crate remains in the eval package for differential checks, not in the product
or its build dependencies.

Bounded show resolves and checks record metadata before reading original bytes.
It reads the requested span rather than hydrating the document, preserving exact
character counts, arbitrary byte references and Rust lossy UTF-8 presentation as
specified by LLP 0001's candidate read boundaries.

## Embedding build input

The SSE build input uses the original pinned `model.safetensors` and
`tokenizer.json`; `build.rs` checks SHA-256 values
`473796430f4a5a042d05fd090ea7ff3986e942385d4cbea92fa82d79711a00c3` and
`d241a60d5e8f04cc1b2b3e9ef7a4921b27bf526d9f6050ab90f9267a1f9e5c66`.
Its `config.json` is build metadata:
`{"architecture":"SSE","dimensions":512,"revision":"928b418b2741b7f7b1b557a53845ac88ad5054c4"}`.
No conversion of the weights is needed. All three files contribute to the
publication fingerprint; an incompatible existing store requires explicit rebuild.


## Agent use for medium-term engineering knowledge

Publish this guidance in CLI help: a short entry in `linn --help` and the writing
and correction instructions in `linn add --help`. CLI users need no repository
instruction file to discover it.

Write a memory automatically after discovering a non-obvious integration constraint,
failure mode or system interaction likely to matter again. Keep one coherent finding
per note. Include the conditions under which it applies, the useful explanation and
source references already available from the work. Supply original text, not keyword
lists or generated retrieval hints. No separate verification pass is required.

Keep plans and task tracking elsewhere. Put enduring API and architecture documentation
in the repository; avoid duplicating documentation that already explains the discovery.
Use retrieved memories as useful context and inspect their original bytes when needed.
When new work disproves a note, add a corrected child with `--parent`, or close it with
`add:close` and a reason. Do not silently rewrite the original. Set `--stale-after` only
when there is a known expiry; age alone is not proof that a memory is wrong.

New writes record the effective Git `user.email`, or null when unavailable.
Existing attribution remains unchanged. Email is unverified metadata, not identity.
There is no writer option or device identifier.


## `linn list`

`--writer <email>` selects the exact, case-sensitive stored writer value and
combines with all other filters. Missing/null writers do not match. The filter
is retained by the list cursor; specifying it alongside a cursor is an error.
Search accepts `writer:<email>` (or a quoted value) with the same comparison;
mention it only in the filter reference, not examples or introductory guidance.


Browse current record metadata across repositories without search terms. This does
not invoke retrieval, POS or embeddings, load bodies, or infer agent sessions.

- Default: 20 records, creation timestamp descending, then record id descending.
- `--kind note|doc` restricts the kind. `--since 7d` accepts a positive duration
  (`m`, `h`, `d`, `w`) or RFC3339 timestamp; creation at the boundary is included.
- `--expiring-within 30d` accepts a positive duration and selects expiries strictly
  after now and at or before the upper boundary. Sort by expiry then record id,
  both ascending. It combines with kind and creation filters.
- Exclude expired, closed, superseded, non-accepted and close-envelope records,
  using the word-search lifecycle predicate. No automatic repository filter.
- `--limit 1..100` controls page size. `--cursor <token>` continues a page;
  it conflicts with kind/since/expiry filters but permits a different limit.
  The opaque `l1.` cursor retains normalized filters, clock, last sort key/id
  and publication identity. A changed store requires starting a new list.
- Print id, kind, creation, expiry and label. Missing creation displays as unknown
  and sorts last in newest-first order; a since filter excludes it. No expiry
  displays as never and is excluded by the expiry filter. Use `show <record>`
  to open original bytes. A next cursor is printed only when another row exists.
- JSON is `{"records":[{"record","label","kind","created","stale_after"}],"next":...}`:
  creation is the stored string (empty when unknown), expiry and next may be null.
  Exit 0 for a nonempty page, 1 for an empty page, 2 for invalid options/cursors.
  Listing uses the stored timestamp comparison convention; CLI-created dates are UTC.


## Flake consumers and package acceptance

Expose `packages.<system>.linn`, its `default` alias, and `overlays.default`.
The overlay builds against its caller's package set; standalone package outputs
use the locked nixpkgs. Both NixOS and nix-darwin can install either package in
`environment.systemPackages`; no service or daemon module is needed.
Native outputs cover aarch64-darwin, aarch64-linux and x86_64-linux. Linux
packages remain static; macOS uses system libraries. Do not claim cross-platform
execution validation from evaluating a derivation on one machine.

`checks.<system>.package-smoke` checks a copied packaged binary with isolated
home/config/data directories, no asset environment variables, and synthetic data.
It verifies original-byte backup restoration, retries, tombstone precedence,
schema upgrade from the previous version and recovery from an interrupted stdio
sync. Required license files accompany the package. The optional real-SSH mode
uses loopback, temporary host/client keys, strict host verification and a forced
Linn command; it cleans up its process and keys. It is an explicit local test,
not a background service. The same protocol can be checked with a separately
supplied compatible peer binary. Real two-device and other-OS execution remain
separate acceptance measurements.

## CI and tagged releases

GitHub Actions runs CI for pull requests and pushes to main, including docs-only
changes. The reusable CI workflow runs formatting, strict workspace Clippy,
workspace tests, LLP validation, workflow linting and release-packaging tests.
Native build jobs cover aarch64-darwin, x86_64-linux and aarch64-linux. Each builds
the flake package, runs its packaged-binary smoke check and uploads a tar.gz
archive plus an individual SHA-256 file as a short-lived workflow artifact.
No private corpus, credentials, or evaluation output is needed.

Pushing a tag matching `v*` invokes the same checks through the release workflow.
The tag must equal `v` plus the root Cargo.toml package version; Nix uses that
same version. After every job passes, verify that exactly the three expected
platform archives are present and match their checksums. Publish them with
SHA256SUMS and generated release notes. Assemble a draft first; publish only
after every asset upload succeeds. A rerun may resume an existing draft, but
refuses to replace an already published release. Versions with a prerelease
suffix publish as prereleases. Creating a local tag alone does not trigger
GitHub; the tag must reach the remote.

Archives contain only the executable, README and the three required license
notice files under one versioned platform directory. Do not archive a checkout,
store, corpus or build directory. Normalize tar ownership, file modes and
timestamps and gzip metadata. Executables retain mode 0755. The packaging script
tests the allowlist, reproducibility, version checks, missing notices and
incomplete or corrupted asset sets.

Workflows pin third-party actions to commit IDs. CI uses read-only repository
permissions and does not retain checkout credentials; only the publication job
gets contents-write. There is no Developer ID signing, notarization, signing
secret or separate package-manager tap. macOS archives therefore make no
Developer ID trust claim; local smoke tests do not establish Gatekeeper behavior
for a browser-downloaded archive. Test that distribution path separately when
needed. The first hosted runs establish actual Linux/runner compatibility;
local workflow validation is not evidence those jobs have already passed.
