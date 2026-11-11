# LLP 0001: Immutable storage and sections

**Type:** Explainer\
**Status:** Active\
**Systems:** Store, Sections\
**Related:** LLP 0000, LLP 0001, LLP 0002, LLP 0003, LLP 0004, LLP 0005

Ingestion is one fixed algorithm, not a configurable pipeline. POS annotation
and shared heading-tree membership are unconditional. No runtime flag, environment
variable or configuration file changes tokenization, derived indexes, model use
or algorithm constants. Record metadata and original text are inputs; diagnostics
and store-location settings do not select alternative ingestion behavior.

## Original bytes

Original bodies and complete serialized envelopes are immutable. Rebuild inserts
new records but does not delete and reserialize existing envelopes; their exact
bytes and hashes survive rebuild and sync (LLP 0006). Before generating index
rows, rebuild compares resident records with the known fields of persisted
envelopes and rejects conflicting content or metadata under an existing ID.
Persisted originals remain authoritative; unknown envelope fields and original
JSON formatting are retained without becoming indexed evidence. This validation
runs on the write/rebuild path, not ordinary startup or querying. Missing state defaults to
accepted; search still requires accepted.

A section is never an independent record. Display grouping and excerpt limits
must not change which bytes form an index row. The heading path is navigation
metadata for display, trees, and `under:`, not another FTS column. Inheriting a
label or ancestor heading into every row would make unrelated passages match;
only the section's own heading belongs in its first block's heading column.

## Data model

### Record

| Field | Type | Meaning |
|---|---|---|
| `id` | string | Stable id |
| `kind` | `note` or `doc` | `doc`: the verbatim bytes of a file that was imported. `note`: text a writer wrote. Set by how the record is added (Writing: `add`, `add:close`, `add:forget`), not by a flag |
| `scope` | `"<owner>/<repo>"` or null | The repository it came from, named by its git remote (Command structure). Kept for the penalty (Scoring (BM25 over the searched rows)). Not printed on a call-1 card. Null: not another repository. The value stored in the index is null (Eligibility) |
| `label` | string | Short human label, shown on cards. Never matched by search; related pointers (Related pointers) use its words. Optional on input: a document defaults to its single top-level heading, else its file name; a note to its first line, cut at 80 characters |
| `body` | string | Original Markdown. A close envelope may have an empty body |
| `writer` | optional string | New CLI records capture effective Git `user.email`, or null when unavailable; preserve existing values or absence in immutable envelopes |
| `source` | object | `type` (repo-file, skill, github-doc, thread, derived-from-thread, derived-from-file), `ref`, `note` |
| `source_path` | string or null | Optional. A document: the imported file's path relative to the checkout (set by `add`). A note: the file the note is about, if any |
| `parent` | id or null | The earlier version this record replaces |
| `close` | object or null | A close envelope: `{target, reason: supersede, correct or expire, replacement}` |
| `stale_after` | timestamp or null | After this time the record is expired |
| `created` | date | Creation date |

`state` is stored and defaults to `accepted` when absent. Candidates are stored and
embedded but excluded from word search. No author evidence, terms, or section_terms
fields participate in retrieval. Supporting files are Markdown links.

Documents must be ingestible directly from their original bytes, without a
preparation step that supplies keywords or another representation to search in
place of the content. Do not manufacture labels, summaries, or per-document
hints to improve retrieval or evaluation. A human label remains optional metadata;
its deterministic default comes from the existing heading, filename, or first
line, not a keyword-writing pass. The generated parts column, structural indexes,
and specified model encodings are derived from the content by the store itself.

### Eligibility

The word search returns a row only when every SQL condition holds:
1. `state` is `accepted`.
2. The record is not a close envelope.
3. No accepted close envelope targets it.
4. No accepted, non-envelope record names it as `parent`.
5. `stale_after` is null, empty, or later than the query's `now`. The comparison is the stored
   text, not a parsed timestamp.
6. `length(body) > 0`. A body of only spaces passes this test. The heading-tree index uses a
   stricter test: after stripping whitespace the body must be non-empty.

Scope is not a SQL condition in this store. At ingest every record is stored with `scope` null, and
the original scope is kept beside the index for the penalty in Scoring (BM25 over the searched rows). The clause
`(scope IS NULL OR scope = ?)` therefore matches every row, and a `scope:` filter does not remove
rows either: a stored null scope is treated as matching any `scope:` value. The original scope
supplies the penalty of 7. Heading-tree membership is always shared and never excludes a record by original scope; see LLP 0002.

### Identifiers and references

- **Body hash:** `sha256` of the body's UTF-8 bytes.
- **Fetch id:** `"<body hash>:v1:<start>:<end>"`, a byte range in one immutable body version.
  Citations use the fetch id. The CLI prints the first 12 hex digits of the hash and accepts any
  unique prefix of at least 8.
- **Reference (`<ref>`).** Wherever a record or a span is named (a command argument, `id:`,
  `--parent`, `--replacement`), any of these is accepted:
  - a fetch id: one span of one body version
  - a record id: the whole record
  - a body-hash prefix of at least 8 hex digits: that body version
  A fetch id or hash prefix shared by byte-identical bodies under several ids (also in different
  repositories) names the current one of the caller's repository, else another current one. A prefix matching two different bodies is an error that lists the candidates.
- **Reference as URI.** Markdown may use the `linn:` scheme followed by a record id or fetch id. CLI ingest warns about unresolved destinations; the path-link index retains them as written.

## Byte spans

### Ingest

For every record, in id order:

1. **Lifecycle sets.** Compute the closed set (targets of accepted close envelopes) and the
   superseded set (parents of accepted, non-envelope records).
2. **Sections.** Split the body at ATX headings. A heading line matches
   `^(#{1,6})[ \t]+(.*?)[ \t]*#*[ \t]*$`. The heading text is the title with surrounding whitespace
   removed; trailing closing hashes are not part of it. Lines inside a fence do not count. A fence
   matches `^[ ]{0,3}(`{3,}|~{3,})`: at most three leading spaces, then three or more backticks or
   tildes. It closes on a later line of that same shape whose mark is the same character and at
   least as long. A fence indented by four or more spaces is ordinary text. Text before the first
   heading is a preamble section with no heading. Each section has an ordinal, a level, its
   heading, its heading path (the ancestor headings and its own), and start and end byte offsets.
   Section parsing walks lines separated by the byte `\n`, not all Unicode line separators. The
   heading line belongs to the new section's body. Pop ancestors whose level is greater than or
   equal to the new heading's level; skipped heading levels do not create placeholder ancestors.
   An empty body parses as one empty preamble, but produces no index row. There is no empty
   preamble before a heading at byte 0.
3. **Blocks.** A section longer than 1,400 bytes is cut into blocks.
   - Cut points are blank lines, the start of a list item (`-`, `+`, `*`, `1.`, `1)`), and fence
     boundaries. A fenced block is never split.
   - Blocks are packed greedily: a block ends before the cut that would take it over 1,400 bytes.
     One oversized unit stays whole.
   - Each block is one index row and keeps its byte span. Only the first block of a section carries
     the section's heading in the heading column.

```python
def block_spans(data, start, end, limit=1400):
    cuts, pos, fence = [start], start, None
    for line in data[start:end].splitlines(keepends=True):
        text = line.decode("utf-8")
        m = FENCE.match(text)  # ^[ ]{0,3}(`{3,}|~{3,}); closer is at least as long
        if fence:
            if m and m.group(1)[0] == fence[0] and len(m.group(1)) >= len(fence):
                fence = None
                cuts.append(pos + len(line))
        elif m:
            cuts.append(pos)
            fence = m.group(1)
        elif not text.strip():
            cuts.append(pos + len(line))
        elif re.match(r"^\s*(?:[-+*]|\d+[.)])\s+", text):
            cuts.append(pos)
        pos += len(line)
    cuts = sorted(set(cuts + [end]))
    out, a, previous = [], start, start
    for b in cuts[1:]:
        if b - a > limit and previous > a:
            out.append((a, previous))
            a = previous
        previous = b
    if a < end:
        out.append((a, end))
    return out
```

Index line scanning uses bytes.splitlines semantics: LF, CR, and CRLF (the pair is one boundary); not Unicode separators, VT or FF. Positions are byte offsets. A fence is classified on the decoded line and is never cut. A unit
with no earlier cut stays one row even when it is longer than 1,400 bytes.

4. **Rows.** Each row holds the heading, the body text and the identifier parts. The parts column
   is generated. It lets `pinned` match `pinnedVersion` as an ordinary word. It never satisfies an
   exact identifier.

```python
def pieces(body):
    body = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", body)
    return body.replace("_", " ").replace("-", " ")
```
5. **Statistics.** From the index vocabulary: term frequency per row and column, and row length.
   Length counts heading and body tokens only. Parts tokens are stored in
   the frequency map and are not added to the length. There is no author-terms column.
   For the vector hints and the neighbor
   (Hints on silence, Call-1 page): each surface token, lowercased with combining marks kept, counts once per
   index row that holds it in the in-memory heading or the block text. The in-memory heading is
   present on every block of a section, so a heading word is counted once per block.
6. **Links.** Markdown link destinations (`](dest)`) are resolved. Any `#fragment` is dropped, the
   remainder is joined onto the directory of `source_path`, and the result is normalized. An
   absolute destination replaces that directory. A destination that equals a record id names that
   record. A `linn:<ref>` destination is retained unresolved by the path index; CLI ingest warns when its reference cannot resolve. The same-scope test uses
   the stored scope, which is null, so a resolved path matches every repository that has it.

   These give outlinks and backlinks. A link that resolves to no record is kept as written. `add`
   lists unresolved `linn:` links, so the writer sees a mistyped reference at once. The tested
   resolver keeps the record id it resolved. It does not follow a later supersession or a close
   envelope to a replacement. Eligibility can still hide that target.
7. **Hint index.** For every non-zero-byte parsed section (not block), keep surface tokens, not
   `norm` words:
   - the set from its heading and text
   - the set from its heading path and its label
   - a posting list from each heading-and-text token to its sections

   Eligibility is checked at query time, not here.
8. **Vectors.** Every index row (block) of step 3 gets one embedding of its body text (Embeddings),
   stored with the row id, in ascending section id. Full ingest and rebuild embed every row in ascending section id; add rebuilds derived state.
9. **Source-path basenames.** For each record whose `source_path` has a final path component, keep
   that component lowercased and the set of record ids that share it. A record with no such
   component is omitted. This map is not an FTS column and it is not written into the body.
   Search uses it only as in Ordered fallbacks.

**Forget.** Deleting a record removes its record and section rows from both matchers, including
FTS and stem rows, with SQLite `secure_delete` on. Remove its in-memory section rows, frequencies
and lengths; clear matcher result caches, BM25 statistics, eligible-row caches, navigation
indexes, and the one-under coverage cache. Remove its vectors by slicing the existing matrix;
do not re-encode the remaining rows. Recompute vector document frequencies from those rows.
Drop the record from heading-tree membership and rebuild the direct and deep trees, drop it
from the one-word index, and drop the beam's cache key, the surface-form cache key, the
section-mean cache key, the sentence cache key, and the vocabulary cache of the absent-identifier
split, and remove the record from the source-path basename map. When that basename then names no
record, drop the name, so the next search embeds those lists again, rereads the exact vocabulary,
and can treat a remaining record as the only one with that basename (Ordered fallbacks).

The native store removes the record, its parsed sections, and its blocks from
the in-memory maps. It clears FTS-hit, statistics, coverage, surface-form, beam,
and sentence caches. Frequencies, vocabulary, word order, trees, and basenames
are rebuilt from surviving rows. Word hints read the surviving section map;
there is no retained forgotten-record hint or fallback-section lookup cache.
Historical matcher rankings, confidence maps, and per-word Porter caches remain.
A valid query recomputes its ranking, but a fallback tier test can still scan
rankings retained under other contexts. Closed and superseded projections and
resolved links are deliberately retained: forgetting a close envelope or a child
leaves its target hidden until rebuild. Integrity uses FTS5 `integrity-check`.

## Index schema

Migration 1 uses these tables in one local libSQL file. Row ids are shared by
sections, FTS, stem, and vectors. Metadata records the format version; projections
persist lifecycle sets separately from immutable envelopes.

```sql
CREATE TABLE IF NOT EXISTS metadata(version INTEGER NOT NULL);
INSERT INTO metadata SELECT 1 WHERE NOT EXISTS(SELECT 1 FROM metadata);
CREATE TABLE IF NOT EXISTS records(id TEXT PRIMARY KEY,scope TEXT CHECK(scope IS NULL),original_scope TEXT,state TEXT,body BLOB NOT NULL,body_hash TEXT NOT NULL,envelope BLOB NOT NULL,envelope_hash TEXT NOT NULL);
CREATE TABLE IF NOT EXISTS tombstones(id TEXT PRIMARY KEY);
CREATE TABLE IF NOT EXISTS projections(key TEXT PRIMARY KEY,value BLOB NOT NULL);
CREATE TABLE IF NOT EXISTS sections(id INTEGER PRIMARY KEY,record TEXT NOT NULL,heading TEXT NOT NULL,body TEXT NOT NULL,parts TEXT NOT NULL,start INTEGER,end INTEGER);
CREATE INDEX IF NOT EXISTS sections_record ON sections(record,id);
CREATE VIRTUAL TABLE IF NOT EXISTS fts USING fts5(heading,body,parts,content='sections',content_rowid='id',tokenize="unicode61 tokenchars '_-'");
CREATE VIRTUAL TABLE IF NOT EXISTS stem USING fts5(text,tokenize="porter unicode61 tokenchars '_-'");
CREATE VIRTUAL TABLE IF NOT EXISTS vocab USING fts5vocab(fts,'instance');
CREATE VIRTUAL TABLE IF NOT EXISTS stem_vocab USING fts5vocab(stem,'instance');
CREATE TABLE IF NOT EXISTS vectors(id INTEGER PRIMARY KEY,record TEXT,emb F32_BLOB(512));
CREATE INDEX IF NOT EXISTS vectors_order ON vectors(id);
```

Insert/delete triggers synchronize external-content FTS with sections and delete
matching stem rows. Stem content is inserted in the same write. Enable SQLite
secure_delete and FTS5 secure-delete on both indexes. Parts are derived, not user
input. The per-word Porter helper uses `porter unicode61` with no added token chars;
the index stem helper uses `porter unicode61 tokenchars '_-'`. Their results are
deliberately different. Vocabulary tables expose token instances for frequencies.
Vectors are little-endian float32 blobs, 512 components, one per indexed block.

## Derived lookups

During ingestion, obtain heading-plus-body token lengths from FTS5's existing
`fts_docsize` rows: decode exactly three SQLite big-endian varints and add the
first two column counts, excluding parts. Reject malformed sizes. The lexical
vocabulary is the distinct-term view from a connection-local `fts5vocab(...,
'row')` table. Empty-page checks probe only the requested terms, caching both
presence and absence; they never load the complete vocabulary.
Load stored vectors only on first use of a vector stage or an explicit vector
diagnostic. Rebuild still encodes all blocks in its original order; deferred
loading does not encode them again. Do not reconstruct every token occurrence
or load every vector at startup.

Load exact frequencies and body offsets through term-constrained instance
vocabulary queries only when a parsed query needs them; load stem offsets only
for its confidence features. Retain all original columns and occurrence order.
Cache successfully loaded terms, including absent terms, for that Store
instance; never increment the same term twice. Clear these caches together
with row features on rebuild and forget. Token counts, BM25, confidence and
eligibility remain unchanged. These reads use the existing libSQL FTS index,
not a second data file or a separately maintained copy of its postings.

Keep per-record integer block counts and length totals from the ingestion index.
Materialize block ids in ascending persisted order only for requested records or
posting candidates; a partial in-memory block set is never the corpus count. Load normalized words only for
query candidates, using the persistent structural projections below. Record
fills and trees retain the same unions as full scans. The SQL
`sections_record(record,id)` index supports record-qualified reads and deletion.

Resolve outgoing links by exact record-id lookup and an inverse source-path
lookup, preserving all targets when paths are shared. Path normalization and
literal destination matching are unchanged. Group tree rows by heading-path
prefix in ascending block-id order; retain first-encounter tree order, direct
versus descendant membership, and the exact normalized word unions. Avoid
scanning every record for each link or every block for each heading path.

Keep each parsed section's heading-plus-text surface-token set and its
heading-path-plus-label surface-token set separately, retaining section order.
These are the existing hint and related-pointer tokenizers, not FTS tokens,
Porter stems, normalized tree words, or author fields. Persist these sets and
load only queried words; eligibility stays a query-time check. Persist whether
the complete own-text set is nonempty: an unloaded set must not masquerade as
an empty section during related-pointer selection.

Build the projections after build, add, rebuild and forget from current records,
parsed sections and surviving blocks. An older or invalidated store builds them
once on open, matching persisted rows to parsed blocks by exact
`(record,start,end)` keys and retaining gaps. Normal opens read only publication markers and scalar counts; candidate record, section and block
metadata are fetched for candidates without reparsing or tokenizing bodies. No projection changes
original bytes, embeddings, statistics, row order, tie breaks or validity.

Keep indexed surface-word postings for blocks and sections inside libSQL.
Section flags distinguish own text from path/label. References follow record-id
and section order. A shared dictionary records each surface word and its exact
normalized form; it adds no authored evidence. Normalized block matches project
to records and trees through indexed memberships. Read postings by queried word
and tree rows by candidate id, restoring ascending original ids before selection.
Use these to find candidates meeting the same distinct-word coverage bound,
then apply the existing eligibility, hard-term, filter and ordering tests.
Never use the path/label postings for lexical matches or word hints. A zero-hit
record fill remains possible when its coverage bound is zero and there are no
hard terms; keep nonempty record ids in order so that case needs no full scan.

Cache packed float32 matrices in the exact existing row order for blocks,
beam proposals, surface forms and sentences. Keep each original matrix-vector
or matrix-matrix kernel shape and reduction order. The beam's word-to-position
lookup and occurrence-weighted normalized rare-form section means are also
derived caches. Construct mean accumulators in first-form-match order, using an
inverse index only to locate an existing accumulator; add every occurrence in
its original order. Clear all these caches on rebuild and forget; no surviving
block is re-encoded by forget. These are memory caches, not new author fields
or a second persistent store.

## Persistent structural projections

The `nav_*` tables are indexed materialized projections in the existing libSQL
file, maintained by Linn rather than automatically refreshed SQL views.

The pinned local libSQL engine rejects `CREATE MATERIALIZED VIEW`; the Rust
startup diagnostic has an in-memory capability probe. These tables therefore
use ordinary transactional SQL, not an assumed automatic view-refresh feature.
The tables are:

- `nav_lexemes(id,surface,norm)` has a unique surface index and `(norm,id)` index.
- `nav_block_words(word,block)` is keyed by that pair.
- `nav_section_words(word,section,flags)` is keyed by word and section; flag 1
  means own text and flag 2 means heading-path/label.
- `nav_sections(id,record,ordinal,data,own_nonempty)` stores parsed section
  metadata; `nav_blocks(id,record,ordinal,heading,path,start,end)` stores cuts.
  Text is sliced from the immutable record body, not copied into these tables.
- `nav_trees(kind,id,record)` and `nav_tree_rows(kind,tree,block)` retain exact
  direct/deep tree order and row membership, with a reverse block membership
  index. Tree and record word unions are obtained from block postings, rather
  than stored as another copy of those words.
- `nav_surface(word,df,ord)` retains surface DF and first-encounter word order.
  `nav_links(source,target)` retains resolved links.

Ordinary opens defer the surface-frequency dictionary. Query surface-posting
reads join `nav_surface` to load exact DF only for requested words; absent words
retain the existing default DF of one. The full ordered dictionary is loaded
only when regenerating missing beam vectors, and writes compute it as before.
Never derive dictionary order from the order of queries or change the IDF formula.

The `projections` structure marker identifies layout, fixed shared heading-tree membership and a
unique publication. Core record/section insert, update and delete triggers
invalidate it, including writes from older binaries. Deleting a record clears
all structural tables in that same transaction, so forget cannot leave a
forgotten heading or token in a stale projection. Private secure-delete rules
apply to these tables too.

After the core write, publish the replacement projection and marker together
in a separate immediate transaction. A crash between the two leaves an invalid
projection that is rebuilt once, never trusted. Construction uses prepared
statements reset between bindings. Check the connection's external data version
under the writer lock before publication; reject a concurrent source change.
Check that external version before core rebuild/forget writes too, so a stale
Store cannot overwrite another process's records.
These are disposable auxiliary tables governed by their marker version, not
new immutable envelopes or a second core schema identity.

Open reads the publication identity and scalar totals from one snapshot. Every
agent invocation takes a read snapshot and checks the exact publication marker
before reading postings. If another process changed the store or heading-tree
projection, return an error requiring reopen; never combine metadata from one
publication with postings from another. Rebuild and forget update the live
store's marker. The deliberately retained live resolved links after forget
remain unchanged; the newly persisted projection uses freshly resolved links
for subsequent opens, as before.

## Query-independent numerical projections

Prepare form, beam-proposal and sentence encodings during writes/rebuilds, or
once when opening a store whose numerical projection is missing or stale.
Use the same model and original encoding arrays as the fallback algorithm:
form metadata keeps every occurrence and first-match order, beam proposals keep
surface-word order, and sentences keep block/sentence order. Persist their
unchanged float32 bits; never normalize again on load. These are the specified
algorithm's derived vectors, not author terms, new hints or another model.

`fallback_artifacts(kind,generation,model,metadata,rows)` contains manifests for
`forms`, `beam`, and `sentences-dictionary-v1`. `fallback_vectors(kind,part,data)` stores each
matrix in row-major little-endian float32 chunks of at most 16,384 rows (32 MiB
at 512 dimensions), below SQLite's per-value size limit. Metadata holds form
fields, proposal strings, or sentence block ids and dictionary ordinals; vectors never become excerpts.
Validate dimensions and metadata counts before use. Empty arrays are valid.

Sentence storage keeps one row per bit-identical vector, in first-occurrence
order, plus an ordinal for every original sentence. Equality includes all float32
bits, including signed zero; hash collisions must still compare complete rows.
Before scoring, expand the dictionary to the full original matrix in original
sentence order. Do not score the smaller dictionary or remove repeated candidates.
Forms and beam arrays retain their ordinary packed representation. A legacy
`sentences` manifest contains the full matrix and block ids; migrate its stored
bits without re-encoding or loading original documents. Publish the replacement
and remove legacy sentence chunks in the same checked transaction. Release
temporary preparation matrices after publication. Healthy opens still read only
manifest metadata; sentence expansion happens only when its stage runs.

The model key is a build-time SHA-256 fingerprint of the packaged configuration,
tokenizer and weights, plus an encoder-contract version. Bump that contract
version whenever encoding or its loader changes. A matrix is reusable only when
both the model key and structural publication marker match. Publish all three
manifests and their chunks in one checked immediate transaction. Record-deletion
triggers clear them in the same transaction as forget; structural replacement
invalidates them. Forget retains original block vectors without re-encoding;
the corpus-derived fallback arrays are rebuilt for the surviving corpus so
batch boundaries and occurrence weights stay correct.

Normal opens verify the manifests without loading arrays. Read a matrix only
when its stage needs it. Query text is still encoded at search time. Preparing
these projections shifts work and disk space to the write/migration path;
report that cost separately from repeated CLI latency. All numerical kernel
shapes, gates, selection order and first-page results remain unchanged.

For block-vector walks, read the existing `vectors` rows directly into the
ordered packed matrix instead of first allocating a second per-block hash map.
Join each vector to its block's record id and retain only the corresponding
record ordinal alongside the matrix. Do not construct block objects, heading
paths or token containers for the full vector population. Global hint selection
uses these record ordinals for eligibility and record deduplication; selected
hints and claimed rare-form occurrences load their block metadata by indexed id.
Rare-form/block comparisons read only the claimed form's block vectors by their
existing primary keys and use the same vector dot kernel and stored float32
bits. They do not need the global matrix. Explicit vector diagnostics can still
load the per-block map.

## Native persistence

### Write failure and publication

A successful write publishes both original changes and derived indexes. Originals
and FTS changes commit atomically with the same durable pending-publication marker
used by sync. Clear it only after structural, numerical, POS and catalog publication
succeeds. An interrupted publication blocks ordinary opens; explicit rebuild or
sync recovers from committed originals. The marker is local operational state,
not replicated evidence or a retrieval setting.

A failed write may have rolled back, or may have committed originals before an
index-publication error. Errors must not claim rollback without proof. After any
rebuild/forget execution error the Store handle refuses further record reads,
searches and writes: discard and reopen it. SQL integrity diagnostics remain
available. This prevents a failed add's resident record from leaking into a later
write. Invalid requests rejected before mutation do not poison the handle.

Local and synced forget use the same transactional erasure: insert a permanent
tombstone, delete record/section/index rows, reset external-content FTS accounting
with `delete-all` if no sections remain, and optimize FTS segments. In-process
forget then publishes surviving derived state without re-encoding block vectors.

### Schema versions and migrations

SQLite `PRAGMA user_version` is the authoritative SQL schema version. Version 0
means an uninitialized store; version 1 is the original schema, version 2 adds
the writer column, and version 3 consolidates the formerly unversioned structural
and catalog DDL. Version 4 adds local sync configuration and publication state
(LLP 0006). Version 5 adds writer-prefixed creation and expiry indexes on
original records. The legacy `metadata.version` identifies the original format,
not migration progress.

Every future SQL schema change must have a numbered forward migration. On open,
reject unsupported newer schemas before accessing product tables. For an older
schema, acquire the write lock, re-read its version, and apply missing migrations
in order. Commit DDL, any data conversion and the corresponding version together;
a failed migration rolls back and leaves the previous version usable. A current
schema needs no persistent migration DDL. Never silently downgrade. The ordered
SQL files and runner live in `src/storage/migrations/`; append a migration rather
than editing an already published step. All pending steps run under one write
transaction, so any failure leaves the original schema version and data intact.
Per-connection temporary tables and runtime PRAGMAs still initialize on each open.

Migrations preserve original bodies, immutable envelopes, hashes, references and
tombstones. Rebuildable index data may be replaced. Model and derived-publication
versions remain separate compatibility checks. Test upgrades from supported old
schemas, unchanged original data, reopening, failed upgrades and concurrent
initialization. Update this specification alongside each migration.

Private store files are created with mode 0600 and new directories with mode 0700.
Explicit paths may point at an existing directory without changing that directory's permissions.

The persisted envelope is the complete serialized record supplied to the store;
its hash names those stored envelope bytes. No invented subset of fields defines
envelope identity. The body hash remains solely SHA-256 of original body bytes.
Stored SQL scope is null, with original scope kept separately. Validity is a
separate persisted projection, including the deliberate rule that forgetting a
close or child does not unhide its target until rebuild.

The parts column is materialized by Rust in the same transaction as the section,
FTS and stem rows. Callers cannot supply it. This preserves generated-column
semantics without registering a SQLite scalar function through an unsafe native
handle. Neither label nor filename enters those lexical columns.

Forget writes the tombstone and removes both indexes before deleting content.
Secure-delete segments are optimized before commit: the pinned local engine's
external-content integrity check otherwise reports an inconsistent pending
segment after deletion. Surviving vectors are retained without re-encoding.
Reopening preserves gaps in section ids, so vectors, spans and postings agree.
Scratch stem tokenizers are connection-local temporary tables kept in memory;
search never writes them into the persistent store.

## Connection policy

Connection teardown must close each native handle only once. The local
libSQL wrapper is pinned with the narrowly scoped fix in vendor/libsql/PATCH.md:
clear the handle after closing it so nested destructors cannot close freed memory.
The SQLite engine and local-only feature set remain unchanged.

Initialize connection settings with `execute_batch`, so every statement runs:
`secure_delete=ON`, `foreign_keys=ON`, `busy_timeout=5000`,
`temp_store=MEMORY`, and `mmap_size=268435456` (a 256 MiB mapping limit).
The read mapping uses SQLite's file-backed pager and locking; it is neither a
separate data file nor a preload of all documents. Scratch tokenization stays
in temporary in-memory tables. Reads keep the engine's default page-cache budget.
Rebuild and forget temporarily request a 32 MiB pager cache (`cache_size=-32768`)
to reduce page churn during bulk index writes. Save the prior connection setting
and restore it after success or error, then request unused pager memory release
with `shrink_memory`. This is a fixed write-time budget, independent of corpus
size; it adds no table or Rust document index. Preserve all transaction boundaries,
publication checks, secure deletion and SQL operations. If restoring the cache
fails after successful writes, report that the write completed rather than
implying it rolled back.
Do not weaken synchronous durability or secure deletion for query latency.
Cache and mapping settings are fixed connection-local performance choices, not
runtime search algorithm controls. Journal-mode changes also affect writes,
checkpoints and file layout and must be evaluated separately from read-only
timing probes.

## Source evidence

File ingest reads the original UTF-8 content and records its Git blob identity
using `git hash-object`; it does not modify the index or commit the file. Outside
the current Git repository, or if Git hashing fails, it falls back to SHA-256
of the original file bytes. This source identity is separate from the SHA-256 body hash and envelope
hash. Source metadata is stored inside the immutable envelope.

## POS publication

The always-enabled POS selector stores selected source relations in
`pos_relations` in this same libSQL file. Each candidate block has a record id,
body hash, structural publication generation, resource/crate/tag-mapping/unit
identity and a serialized list of original unit bases and selected original byte
intervals, public analysis alternatives and sentence ids. A block primary key
and record index support candidate reads and deletion. No source vectors are
needed by lexical occurrence scoring; shadow vectors remain process-local.

Rebuild/add prepares annotations on original units and publishes them
transactionally after structural publication, guarded by the existing source
version check. Within one publication, byte-identical source units may share
their computed relative annotations. Validate each unit against its own original
body and attach its own absolute base/end; retain every block and relation row.
The temporary map is keyed by the complete original unit text, not normalized
words, and is dropped before publication. It is never a startup or persistent
corpus index. During a missing publication readers fail explicitly. Every
record/section insert, update or delete invalidates the projection, including
legacy direct SQL and secure forget. Mutation triggers remove annotations and
pair postings for the affected record or block and invalidate the complete
publication marker. Unchanged annotations may remain stored with their old
generation, but are not a current publication. Writers reuse them only when
record id, body hash, resource/mapping identity and every original unit boundary
match. Recompute missing or incompatible rows, then atomically publish the new
generation for all surviving rows under the source-version guard. Updating only
the generation does not rebuild unchanged pair postings. No extra cache table
or startup materialization is needed. A full rebuild that replaces the source
rows still recomputes annotations; a forget need not retag surviving documents.
Normal open neither tags the corpus nor loads a global POS map. Candidate reads
require matching body, structural generation and exact resource/mapping identity.
Resource or grammar changes require explicit rebuild, never silent query-time
retagging. Stale writers must reopen before publication.

### Indexed POS preflight

`pos_pair_lookup(left_key,right_key,block)` is a same-file materialized lookup
of the existing selected source relations. Keys are the compact JSON arrays of
exact FTS tokens already stored as each relation's left and right sides. Its
primary key supports pair lookup; a block index supports deletion. It adds no
linguistic analysis or body evidence. Source-relation insert/update/delete
triggers maintain it transactionally and invalidate its publication marker.
Record/section invalidation and secure forget therefore clear affected entries too, including
writes through older binaries. Rolled-back writes roll back both lookup and marker.

The `pos-pair-lookup-v1` marker contains the resource/mapping identity and current
structural generation. Enabled POS publication writes this marker after all
source rows in the same transaction. A healthy older store is upgraded once:
verify complete block coverage, source record/body hash, identity and generation,
then populate the lookup from stored relation JSON and publish its marker under
the writer lock. This upgrade performs no tagging. An incomplete or stale source
publication never grants an absence proof; retain the original query path and
its stale-publication errors. Query reads use the existing search snapshot.

## On-demand source and structural reads

### Candidate read boundaries

Separate metadata selection from original-byte presentation. Section metadata reads
do not imply a body read. Hints and related pointers apply exclusions, eligibility,
coverage and the existing sort before loading the selected records' original bytes.
On a published lazy store, select word hints and related pointers by joining
query terms to the existing section-word postings, section metadata and record
eligibility index. Apply the caller’s clock, nonblank-body test and exclusions
in SQL. Return only the three selected hint sections or two related records;
do not load every candidate record's complete section list first. Admission
counts distinct content-word postings, retaining the existing content-word
deduplication and matched-word order. Related selection keeps the best section per record before
the global limit. All coverage, heading-only and length/record/offset ties stay
as specified in LLP 0002. Live fully materialized stores retain their in-memory
path, including forget's existing lifecycle semantics.
Record fills select the winning block from metadata before reading its text.
Unique quoted basenames seek the earliest block; shared basenames inspect only
eligible records. Sentence fallback tests its existing score thresholds and
same-block constraints before fetching candidate headers, then reads the winner's
text. Later-call text reads are batched after the unchanged selection.

Scoring metadata reads may omit heading and path. Track this explicitly and upgrade
selected blocks before presentation or metadata-dependent filtering, preserving
already loaded frequencies, words and positions. Block presentation reuses the
heading, path and hash obtained under the search snapshot; its byte query returns
only id and the authoritative body slice.
Surface reads return each queried word's DF once and its ordered block ids. They
do not hydrate all matching blocks: attach queried surface membership when a block
header is subsequently read. Unfiltered navigation selects tree coverage and member
winners in SQL; hydrate those winners rather than every matching member. These
query-local trees are discarded before a different query's navigation. Filter-aware
navigation retains ascending record ordinals and explicit kind/tree/block order. Forward, reverse and same-path neighbor reads
are batched; exclusions retain the existing direction-specific rules. Before CLI
pointer filtering, fetch the indexed block containing each pointer's start. Pointer
visibility must not depend on an unrelated earlier read having loaded that block.

Vocabulary membership is a batched indexed existence lookup. Scratch occurrence
tokenizer tables are created once per connection; insert independent inputs in
one statement with stable row ids. Sentence-token and whole-sentence reconciliation
remain separate inputs in the same batch. Porter inputs are independent strings
using `porter unicode61`, selecting each lowercased word's first lexicographic
term, or that lowercased input if no token exists. Never concatenate inputs or use
the index tokenizer as a substitute.

The Porter adapter in `src/nlp/porter.rs` invokes the pinned libSQL tokenizers
directly, rather than creating and mutating temporary FTS tables. It lazily owns
both configurations and one ephemeral in-memory connection per calling thread;
this connection holds no documents and is not a persistent store. Native handles
never cross threads or concurrent calls. Copy callback tokens before returning,
contain callback unwinds, check lengths and native error codes, and destroy the
tokenizers before their owning connection. Fail explicitly on errors or reentrant
use. Keep the per-store Porter result cache. This adapter does not replace the
persistent stem index or its tokenizer, and adds no healthy-startup work before
its first use.

Reference metadata resolution does not read a body. Bounded `show` obtains at
most the first 6,000 bytes of the requested span for its 1,500-character output;
libSQL counts the span's Unicode characters. Embedded NULs and references cutting
UTF-8 scalars retain the original full-span Rust lossy decoding, since SQLite
text operations do not implement that contract. `--all` reads the requested span.
Currentness rejection precedes these body reads; exact truncation counts remain.

### Publication and indexed reads

Treat the persisted navigation tables as ingestion indexes, not a whole-corpus
cache to reconstruct in RAM. Ordinary healthy opens retain publication markers and scalar corpus counts only.
They do not allocate corpus-wide record, ordinal or eligibility maps, deserialize source bodies or sections,
allocate block objects, read heading-tree entries, resolve links, build basename
maps, or walk FTS document-size rows. Candidate headers are loaded by indexed record id for lifecycle checks and scope
penalties. Sparse ordinal maps contain only requested records. Global aggregate
arithmetic remains integer before the existing conversion to floating point.

- `record_headers(record,data,body_bytes,body_nonblank,body_hash)` stores the
  original metadata with an explicitly unloaded body, its UTF-8 byte length,
  stripped-nonempty flag, and original hash. The immutable envelope and body in
  `records` remain authoritative. An unloaded body is not an empty document:
  eligibility uses byte length, while heading/hint eligibility uses the separate
  stripped flag. Load original body bytes by record id when a candidate needs
  them. Writers/rebuild/forget materialize originals before indexing or publishing.
  `resolve_loaded` is the body-reading reference API; plain metadata resolution
  does not imply a loaded body. Header serialization never becomes FTS evidence.
- `record_index_totals(record,blocks,length)` stores exact block counts and sums
  of decoded native FTS column lengths. Use these totals for BM25, IDF and POS
  publication completeness, never the number of blocks loaded in memory.
- `record_read_index(record,ordinal,eligible,expires)` stores the original sorted
  record ordinal and projected lifecycle eligibility: bit 0 is word-search eligibility,
  bit 1 additionally requires a stripped-nonempty body. Expiry remains a strict
  query-clock comparison. `record_expiry_totals(expires,blocks,length)` stores
  integer suffix sums of eligible block counts and lengths, including immortal
  records; the empty-expiry row stores immortals alone. Seek the first expiry
  strictly greater than the clock, or use the immortal row. This computes exact
  corpus statistics without reading all records. Distinct-block DF joins only
  the requested FTS term to indexed eligibility and expiry.
- The read-index marker binds scalar record/block counts to the exact structural
  publication. Publish its tables and marker atomically under the source-version
  guard. Record/section writes invalidate it; deletion clears its derived rows
  under secure deletion. Missing or stale indexes require a one-time maintenance
  rebuild. Preserve the existing closed-target projection after forget until an
  explicit rebuild. Normal reads never infer absence from an unloaded header.
- Parent and close-target expression indexes support lifecycle navigation.
  A partial nonempty-record index supports the reverse-record fallback with
  exclusions and `LIMIT 1`; do not build a full nonempty-record list at open.
- Block metadata, including heading paths needed by filters, is read by persisted
  id in bounded batches. Obtain the original body hash from the verified compact
  header row, avoiding a read of the main document row just for its hash. Copy each
  batch's block values, then load missing record headers using the record ids already
  returned; do not issue a second query to rediscover those ids. Prepare compact
  headers before numerical projections during a legacy upgrade.
  FTS, surface and tree postings do not load block text or
  parent bodies. Read text for selected cards and other text-consuming stages
  by slicing the authoritative `records.body` BLOB inside libSQL using byte offsets,
  checking UTF-8 and byte length against the stored span. Do not use text-character
  offsets or treat the derived index text as the authoritative body. Whole-record reads are
  reserved for section excerpts, reference navigation and writes. Every posting's values are copied
  before advancing its libSQL cursor. Record-qualified block and section reads
  use `(record,id)` and `(record,ordinal)` indexes and retain their original order.
- Section postings carry stable section ids, record, ordinal and own-nonempty
  metadata. Only coverage-qualified sections cause their record's section JSON
  to load. Tree postings load only matching tree entries; candidate tree member
  ids stay sorted by original block id. Missing in-memory entries mean unloaded,
  never absent from the store.
- Outgoing links use the primary `(source,target)` key; incoming links use
  `(target,source)`. Same-source-path neighbors use an expression index on the
  original envelope's source_path. No whole-record scan is needed for those
  lookups. Preserve the existing live-instance resolved-link behavior after forget.
- `record_basenames(name,record)` uses Rust's existing lowercase final-component
  rule, with a reverse record index for deletion. Read quoted names and bounded technical-stem prefix ranges under LLP 0002. Do not
  substitute SQLite's ASCII lowercase, add filenames to bodies, or change ties.
- Reference lookup uses the stored body-hash index, preserves case sensitivity,
  ambiguity rejection, currentness/scope/id ordering and span validation, and
  reads only the selected original body.

Headers and basename entries are ordinary derived ingestion rows. Insert/update
triggers invalidate their publication marker; delete triggers also remove the
record's rows in the same secure-delete transaction. A missing marker causes a
one-time checked rebuild, never use of stale metadata. Integer totals publish in
the same transaction as the structural index and share its generation. A legacy
store can derive missing totals once from native FTS lengths; normal opens reuse
the published totals. Do not add another data file.

Complete materialization is reserved for rebuild, ingest/index publication,
forget's surviving-index regeneration and explicit conformance diagnostics.
Global nearest-vector fallback stages still read full ordered numerical matrices
and the block-to-record metadata needed to choose their results; they do not need all
source bodies. Preserve matrix dimensions, batch boundaries, float32 reductions,
occurrence weights and ordering. These global numeric operations are not replaced
by approximate candidate cuts in a latency change.

For rare-neighbor selection, the existing rare-word and shown-record predicates
identify every row that can survive. Read those vector payloads by primary key.
Retain the full ordered matrix dimensions and selected row positions, padding
unused rows with zeros and discarding their scores. Reuse the full matrix when
already resident. This preserves the numerical kernel and selected float32 scores;
it does not change the candidate predicate or nearest-row comparison. The narrow
native `vectors_order` index supports counts and ordered positions without reading
all embedding blobs. Contiguous ids use subtraction; after deletion, SQL window
positions retain the order of surviving rows. Do not retain a parallel whole-store
id map or publish a partially filled matrix as the complete cached matrix.
Allocation and arithmetic still scale with the full matrix shape; only payload
reads and decoding are restricted to candidates.

`section_means(block,ordinal,emb)` stores query-independent rare-form means by
block id. Compute them during numerical-index publication using the original
form encounter order, repeated occurrences, float32 sums, division and normalization.
Persist the final 512 little-endian float32 values and their first-encounter
ordinal. A model- and structural-generation-bound marker is published atomically
with the table under the source-version guard. Missing means require a one-time
upgrade from the stored form arrays; healthy opening verifies only the marker.
`beam_terms(term,ordinal)` is published with the same marker and stores the
original proposal position. Query words use exact indexed lookups instead of
building a map of every proposal. Record deletion clears this derived vocabulary
under secure deletion before rebuilding surviving proposals.
Deleting a section deletes its mean row under secure deletion. Read only the
rare-word candidate block ids, ordered by the persisted ordinal; never build a
whole-population mean map on the query path.

### Full-population operation boundaries

Writes, rebuild, forget, legacy-index migration and explicit conformance diagnostics
may materialize the corpus. Healthy opening does not. Candidate reads may touch
many matches for a common query, but never preload unrelated records. Exact
numerical fallback populations still require their specified global matrix or
array scans: preserve their arithmetic, batch boundaries and selection order.
The global hint matrix joins the vector row's own record key directly to the read
index for stable ordinals and eligibility bits, without joining or loading all
block or record headers. This is not a claim of
sublinear worst-case retrieval, nor permission to replace exact selection with
approximate nearest neighbors.

## Embedding publication

`projections[block-model-v1]` stores the embedded asset fingerprint and inference
revision as bytes. Healthy opens compare this scalar marker. A populated store
uses a distinct revision for the vForce and scalar tanh kernels. A store
with a missing or different marker refuses ordinary operations and asks for
`linn index:rebuild`; do not silently mix old block vectors with new query vectors.
An empty store may initialize the marker. Only the explicit rebuild command may
open an incompatible publication, and it must rebuild before returning results.
Rebuild publishes the marker in the same transaction as all block vectors.
Fallback and section-mean publications independently check the same model identity
and the structural generation. Changing the model therefore requires a rebuild,
not a startup corpus scan or an implicit lifecycle reprojection.


### Query-time lifecycle and structural upgrades

The navigation index retains otherwise eligible nonblank records regardless of expiry.
Apply expiry at the caller's clock when selecting trees and fills, not when creating
structural membership. The navigation publication version is `navigation-v2-lifecycle`;
opening an older version rebuilds derived structure without altering original envelopes
or section identities. Republish existing POS annotations against the new generation,
reusing unchanged source units. Word search continues to accept a spaces-only body; tree and hint
membership require a nonblank body. The retained close/supersession projection and the
forget-until-rebuild behavior above remain unchanged.


### Metadata listing indexes

`record_created` orders the existing body-free `record_headers` by creation and
record id descending. `record_kind_created` adds kind as the leading equality
key. Both index coalesced JSON expressions directly; no extra metadata table or
Rust index is maintained. `record_expiry_list` indexes expiry and record id on
`record_read_index`, restricted to lifecycle-eligible rows. SQLite maintains these
indexes with the existing header/catalog writes, rebuild and forget operations.

Listing drives an ordered index range and joins metadata by record primary key.
Use a sort-key/id seek cursor rather than OFFSET: first seek remaining ids at
the cursor’s timestamp, then seek beyond that timestamp if needed. These two
scalar ranges let SQLite use expression-index keys, including timestamp ties,
without tuple-comparison scans or sorting. Read at most limit plus one
qualifying rows. Kind and lifecycle filters can require skipping nonmatching
index entries, but never materialize the whole corpus in Rust. Read publication
and page in one database snapshot. Return no body, section or vector payload.

### Writer attribution

For new file, stdin and close records, read `git config --get user.email` in the
invocation directory. Git resolves repository, worktree, global, system and
conditional configuration according to its normal rules. Trim the value; missing,
empty or unreadable configuration (including unavailable Git) yields null.
This describes the configured writer/importer, not verified identity or the
original author of imported file content. It has no authentication, indexing or
ranking role. No device identity or automatic attribution backfill is added.

Schema version 2 exposes `records.writer` as a nullable TEXT VIRTUAL generated
column over the immutable envelope’s `writer` field. It can be selected directly
in SQL without a second stored copy or synchronization triggers. Migration adds
the column without changing envelopes or their hashes. New unknown writers are
explicit JSON null; absent legacy fields remain absent and also read as SQL NULL.
The column is not a retrieval index and is not included in body text or vectors.


### Writer selection indexes

Writer filters use exact, case-sensitive stored values. Missing and null writer
values do not match a supplied email; there is no synthetic unknown-writer value.
Migration 5 creates `records_writer_created(writer, created DESC, id DESC)`
and `records_writer_expiry(writer, expiry, id)`: created and expiry are indexed
JSON expressions over the immutable envelope, with missing/null timestamps mapped
to the empty string. SQLite maintains both indexes on original-record writes and
deletes, including import/sync, without a new table or publication step.
List seeks the writer plus creation/expiry range and uses indexed ID tie order.
Search intersects FTS row IDs with blocks belonging to records selected by the
writer index before loading lexical block metadata. Corpus scoring statistics
remain global; attribution is never lexical evidence.
