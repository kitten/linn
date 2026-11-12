# LLP 0002: Search algorithm

**Type:** Explainer\
**Status:** Active\
**Systems:** Retrieval, Embedding\
**Related:** LLP 0000, LLP 0001, LLP 0002, LLP 0003, LLP 0004, LLP 0005

## Lexical evidence

These rules define one fixed retrieval algorithm. There are no runtime flags,
environment variables or configuration knobs for stages, ranking weights,
thresholds, models or heading-tree membership. POS selection is always enabled
with weight 0.75 and shared heading trees. Query filters and request context are
inputs to this algorithm; diagnostic output settings never change its results.
Research alternatives require isolated source changes, not dormant product switches.

## Governing decisions

The store derives its heading structure and link graph from original bytes and
explicit record relations; agents do not maintain a separate retrieval layout.
Location narrows only through explicit path/heading filters. Cwd is never a
ranking signal; caller scope supplies only the specified within-tier penalty.
Labels and inherited paths may suggest related pointers but cannot satisfy a
lexical query. A BM25 score is a rank within one query, not a portable confidence
cutoff across corpora. Keep tiers before scores and the diagnostic confidence
gate open. Cards point to bytes to read; a topical hint is not an answer.

One static model supplies only the hint, neighbor, beam, form, and sentence
operations below. It does not fuse independent vector rankings into the lexical
page. No author-maintained vocabulary, automatic similarity-based record merge,
model-extracted graph, or second retrieval service is part of this design.

## Search algorithms

### Query language

- **Terms.** Bare words are terms. `"double quotes"` keep a phrase or an identifier together.
- **Filters.** A filter is a lowercase name of letters and hyphens, a colon, and a value, and it
  must start at the beginning of the query or after whitespace:
  `(?<!\S)([a-z][a-z\-]*):("([^"]*)"|\S+)`. A quoted value is the inside of the quotes. An unquoted
  value runs to the next whitespace and keeps any punctuation. `Scope:` is not a filter, because
  the name is not lowercase.
  - `scope:` accepted, and compared with the stored scope. In this store the stored scope is null, so
    the filter matches every row (Eligibility). It does not narrow the search.
  - `writer:` exact, case-sensitive stored writer value; absent/null never matches
    an email. It is a metadata constraint, not a scoring term. Multiple filters
    are ANDed. The CLI also applies it to every card, hint and related pointer,
    including replayed calls. Unfiltered search and corpus statistics are unchanged.
  - `kind:` the record's kind equals the value. The tested values are `note` and `doc`.
  - `state:` `open` and `accepted` match every row. Any other value matches none.
  - `id:` the record id equals the value, or the body hash starts with it. There is no minimum
    length.
  - `under:` the heading path contains the value, case-insensitive
  - `path:` and `cites:` a glob on `source_path`, where `*` stops at `/` and `**` crosses
    directories. Without a leading `/`, the pattern matches a suffix of the path that begins at
    the start or at a `/`. A leading `/` anchors the pattern at the start, and the pattern must
    consume the whole path. A pattern with no `*` or `?` also matches as a substring anywhere.
    A closed double-quoted span in the raw query is a different test. When the inside of the
    quotes equals one record's source-path basename, without regard to case, that record is placed
    first (Ordered fallbacks). Technical file stems have the separate source-backed
    addressing rule there. Neither rule is this glob or requires `path:`.
  - `linked-from:<id>` records the given record links to; `replaces:<id>` the replacement named by a
    close envelope of that record; `version-of:<id>` records whose parent is that record, and its
    parent. All are filtered by eligibility, so a superseded or closed version never appears.
    `sibling-of:<id>` admits only that record itself.
  - `heading:` and `body:` are accepted and have no effect.
- **Errors in a matcher.** An unknown prefix returns no cards and the text `Query error: Unknown prefix`. An
  unclosed quote returns no cards and the text `Query error: Unclosed quote`. A query with filters and no
  text terms returns "Nothing qualified. Supply text terms, or use show/outline navigation."
  These returns do not stop hints, the empty-page rewrites, or the quoted-basename placement (Ordered fallbacks).
- **Words.** After filters are removed, scan `"([^"]+)"|(\S+)`: a non-empty quoted span is a
  phrase; otherwise take a run of non-whitespace. Strip `.,;:!?()[]{}'` and the backtick from
  both ends of a bare run, then take every `[\w-]+` match. Leading hyphens and a run of only
  hyphens survive this parse. Empty quotes produce no term. `a/b` splits into `a` and `b` before
  stop-word removal. Deduplicate `(text, is_phrase)` pairs, case-sensitively, keeping the first.
- **Stop words** are dropped from terms, except `not` and `no`: `a an and are as at be but by can do
  does for from has have how i if in into is it its of on or so that the their then there these
  this to use used using was we what when where which who why will with you your yes`.
- **Hard terms.** A phrase, or a word containing `_` or `-` or a lower-to-upper case change.
- **Content terms.** A phrase, or a word that is not a kind word. The kind words are `command(s)
  flag(s) option(s) variable(s) setting(s) helper function version pinned recommendation replacement
  directory file(s) project repo repository memory note(s) document docs documentation rule(s)
  procedure`. Kind words count toward the score, not toward coverage. A query of kind words only
  never matches.

### Scoring (BM25 over the searched rows)

`N` is the number of index rows whose record passes Eligibility. Query filters do not narrow these
statistics. A section cut into three blocks counts as three. `df(w)` is how many of those rows
contain `w` in any indexed column: heading, body, or parts.
A token that is not in that vocabulary has `df` 0. `avg` is the mean
row length over those rows. Length counts heading and body tokens only, not parts.
`ln` is the natural logarithm. The query side looks up `norm` of each
`[\w][\w\-]*` piece of the term. A piece the tokenizer did not emit adds nothing to that term.

```
idf(w) = ln(1 + (N - df(w) + 0.5) / (df(w) + 0.5))
tf(w)  = tf_body + 3 * tf_heading + 0.3 * tf_parts
```

`tf_parts` is added only for an ordinary word: not a phrase, and not a word with `_`, `-`, or a
lower-to-upper case change. Neither matcher adds an author-term contribution:
legacy `terms` and `section_terms` inputs are unread and no such column exists.

```
length_norm = 0.25 + 0.75 * length / max(1, avg)
c(w) = tf(w) / length_norm + 0.125
score = sum over words w with tf(w) > 0 of
        idf(w) * c(w) * 2.2 / (c(w) + 1.2)
```

This is BM25L with fixed delta 0.125. Absent terms contribute zero, not a
positive floor. Fields still sum before saturation; do not replace the sum
with the strongest field. The name BM25 elsewhere in this document denotes
this fixed primary score.

A query term that splits into several words (a phrase, or a word that `words` splits) contributes
each of those words.

**Other repositories.** A record is far when its original scope is not null and, compared
case-insensitively, differs from the caller's scope. An original scope of null is not far. Far is
not applied inside BM25. It is applied once, when a matcher's rows are cut to five records
(Call-1 page): the sort key is `(tier, -score + (7 if far else 0), arrival)`. `arrival` is how many
other records were already taken while walking the ranking in its stored order, so two rows with
the same tier and the same penalized score keep that order. Tier still beats the penalty. The short-page fills do not apply it.

### Two matchers

Both matchers run over the searched rows (Scoring (BM25 over the searched rows)), and apply the query's filters to
their call-1 matches. Call 3, hints and related pointers ignore the filters (they still apply
eligibility).

**Matcher A, exact columns.**
- A term matches a row if FTS5 finds it in the heading or body. Ordinary words also match in parts.
- A row is a candidate if it holds every hard term and at least
  `need = min(c, max(2, ceil(c / 2)))` of the `c` content terms.

**Matcher B, stemmed. The confidence gate is open.**
- **Body match.** Any term containing `_` or `-`, including a phrase, is an identifier and
  ordinarily matches only through the exact-token body column; the source-witnessed
  nominal-compound exception below applies to eligible bare hyphens. Other phrases and camelCase words match in
  the heading or body. Any other word matches in the heading,
  body or parts. Every non-identifier term also matches in the stemmed table, which holds the
  heading and the body, so "ordering" matches "order".
- **Coverage.** Body matches count 1 toward `need`. Hard terms and `need` work as in matcher A.
- **Gate.** Features and logistic confidence in Confidence diagnostics are computed and cached, but never
  remove a row. The threshold is below every confidence (`THRESHOLD = -1`). The gate was fit on
  a top five from the caller's repository only.
  Other repositories in that five changed the decision and deleted the caller's rows. Opening it
  put those rows back. The weights stay in Confidence diagnostics so the rejected gate can be rebuilt. They
  do not filter this search.

### Source-witnessed nominal compounds

Both matchers augment ordinary term hits with this narrowly defined exception
to exact hyphen matching. The parsed term must be unquoted and contain exactly
two lowercase ASCII alphabetic pieces, each at least two bytes, separated by
one hyphen. Reject it if the original query contains that term wrapped in
backticks. Other identifiers and quoted phrases retain their exact-column rules.

Analyze the two pieces joined by one space with the ordinary POS pipeline.
Require an ordered nominal relation whose left and right token arrays are exactly
the respective pieces. Query the existing stem index for that spaced phrase,
using ordinary matcher eligibility and query filtering. Inspect only those
candidate blocks' current persisted original-body POS relations. A source
relation qualifies when each side is one token and its per-word Porter stem
equals the corresponding query piece's stem. Source order is preserved.

Merge qualifying block ids after the ordinary exact/stem postings through the
same posting-order mechanism. They count as hits for that original term,
including its hard-term requirement. They do not add text, index rows, score
features, synthetic phrases or tier-zero phrase hits. The original term still
supplies its original scoring words. Completeness, generation, resource identity
and body hash must validate for every inspected POS row; a missing/stale row is
an error, never permission to omit a witness silently.

**Tiers and order, for both matchers.** Each matched section gets a tier:
- **0:** the content terms' original texts, joined by one space, match as one phrase in the heading
  or the body. Matcher B also accepts that phrase from the stemmed table, unless some content term
  contains `_` or `-`.
- **1:** every content term is in that matcher's match set. Matcher A's match set includes the
  parts column for an ordinary word. Matcher B's match set is a body match, and a body match
  includes the stemmed table for a non-identifier.
- **2:** every other candidate.

The stored ranking is every matched section, sorted by `(tier, -score, body hash, ordinal, record
id, section id)`, all ascending. It is stored under `(query, caller scope, now)`. Call 1 and calls
2 and 3 read that key. The fallback gate in Ordered fallbacks does not: it scans every stored key whose
query text is the rewritten query and keeps the lowest tier.

```python
tier = 0 if sid in phrase_ids else (1 if all(sid in hits[t] for t in content) else 2)
row = (tier, -score, body_hash, ordinal, record_id, sid)
```

It can hold more than one section of one record. The cut to five
records happens in Call-1 page, after the extra rows are appended. With no cards,
the result says "Nothing qualified." and may carry the hints defined below.
No relaxation-count list is exposed by the native store.

**Calls 2 and 3 are built from each matcher's own penalty-sorted top five, before the admissions
of Call-1 page.** Call 1 is replaced later. Calls 2 and 3 are not.
- **Which matcher.** If matcher A returned no cards, calls 2 and 3 are matcher B's. Otherwise they
  are matcher A's, including when both matchers returned cards.
- **Call 2.** For each record in that matcher's penalty-sorted top five, the next section of that
  record in the stored ranking that was not already a card of that matcher. One pass. A section
  the other matcher put on the fused page can appear again here.
- **Call 3.** Records that link to or from one of those top-five records, or that share a
  `source_path` with one of them. Because stored scopes are null, "same scope" is true for every
  pair, so a shared source path matches across original repositories. An outlink can name a record
  already in that top five. A backlink, and a shared source path, cannot: those edges are kept
  only for a record that is not already in the top five. Among eligible sections of those records
  that this matcher has not already shown on call 1 or call 2, keep sections that match at least
  one query term, using that matcher's term matching and no filters. Sort by that count
  descending, then body hash, ordinal, record id, and section id, all ascending. One section per
  record. At most 5. Each card carries every applicable edge, sorted: `linked-from:<id>`,
  `backlink:<id>`, or `evidence:<id>`. The last name means a shared source path.

An empty additional-sections call is not returned, and neither is an empty navigation call. When
the additional-sections call is empty, navigation is the second call. At most three calls are
returned, sliced to `calls_left` (default 3). Matcher B also truncates its own calls before fusion;
matcher A does not. With `calls_left = 0`, that layer returns no calls. An empty-page step in
Ordered fallbacks that finds a card, and a quoted basename that names one eligible record, still return one
call. `search` then has that call.

### Fusion

Before Call-1 page replaces call 1, interleave the matchers' penalty-sorted top fives, A first, dropping
overlapping spans of the same record and keeping five, exactly as in Call-1 page, step 2. If A is
silent and B answers, use B's calls. Otherwise keep A's later calls. The hint layer turns each
non-empty call's card text into the centred line of Card excerpt and rebuilds its call text as
`[i] heading: text` lines numbered from 1 (use the record id when the heading is empty).
Call 1 is then replaced by the page built in Call-1 page. Calls 2 and 3 retain that hint-layer form.

1. **Both matchers silent:** matcher A's silent call is returned, and hints are added (Hints on silence).
   The page of Call-1 page is empty and does not replace that call. The empty-page rewrites of Ordered fallbacks
   may still replace it.
2. **Otherwise:** Call-1 page builds call 1 and replaces whatever call 1 the fusion had. Related
   pointers, if any were attached, stay attached to that call. They were chosen from the
   penalty-sorted fusion, before the replacement, so a pointer can name a record the new page also
   shows, or a record the new page dropped.

After that page, including a page that stayed empty, Ordered fallbacks may place one quoted basename first
on call 1. Calls 2 and 3 are not changed by that placement. Text, hints, and related pointers on
an existing call stay.

### Card excerpt

A tested call-1 card carries `record`, `span`, `heading`, `section`, and `text`. It does not carry
the record's repository, a label, or a fetch id. The short-page tree and one-word cards also carry
`labels` (empty) and `fetch`. Calls 2 and 3 and related pointers use the centred line below, at
most 160 characters. A word hint uses the first line of its span that contains a matched word,
whitespace collapsed, cut at 160 characters. A vector hint uses the centred line. Call 1 does not.
Its `text` is specified in Call-1 page. The centred line is built from the card's span:

1. Take the non-empty lines of the span.
2. Choose the line with the most distinct query content words. Content words here are the query
   text's words (filters removed) without stop words (the full list, including `not` and `no`)
   and without words under 3 letters. Ties go to the earlier line. If no
   line holds one, take the first line.
3. Collapse whitespace.
4. If the line is over 160 characters, find the first position of any query word in it (lower
   case). Start the excerpt 40 characters before that position, but no later than 159 characters
   before the end of the line. If the start is after 0, the excerpt is `…` plus 159 characters;
   otherwise it is the first 160.

### Hints on silence

If no call has cards, the last call may carry up to 3 hints, from two lists.

**Word hints.**
1. **Words.** The query's content words, as in Card excerpt. With fewer than 2, there are no word hints.
2. **Candidates.** Eligible sections that hold at least `max(2, ceil(n / 2))` of the `n` words.
3. **Order.** Most words matched first, then the shorter section, then record id and offset.
4. **Selection and span.** Take the first 3 candidate sections, without deduplicating records.
   Use the section if it is at most 1,500 bytes. Otherwise use the hint block that holds the most
   matched words. A tie keeps the earlier block. Matched and missing words are counted on the
   whole section, even when the span is a block of it.

Hint blocks use a separate byte cutter, also used by related pointers and outlines. Starting at
`s`, set `e = min(end, s + 1400)`. If `e < end`, find the last `\n` in `[s + 1, e)`; when one
exists, end just after it. Otherwise cut at `e`, even inside a line, fence or UTF-8 character.
Repeat from that end. Decode with replacement for malformed UTF-8. This is not the fenced/list
block cutter of Ingest.

**Vector hints.**
1. **Query vector.** Embed the query text, with filters removed (Embeddings). An empty text gives none.
2. **Candidates.** Index rows (blocks) by cosine similarity to the query vector, highest first;
   ties keep the lower section id. Skip a record that is already a word hint, or that the hint
   index rejects. That check uses the caller’s clock and a whitespace-stripped body, and it
   sees the stored record, whose scope is null, so another repository is not a reason to skip.
   Keep the first row of each record, and stop at 3 records. The shared-word and IDF tests below
   run after those three are chosen. A rejected neighbour is not replaced.
3. **Shared word.** Keep a candidate only if its block (heading and text) holds at least one of the
   query's content words. Without this rule, an empty search always shows 3 vector neighbours.
4. **Rare shared word.** Keep it only if at least one of those shared words is not common: its IDF
   over all index rows, `ln(N / df)`, is at least 2.5 (it is in fewer than 8.2% of rows). `df`
   comes from the statistics of Ingest, step 5, and changes as the store grows; there is no word list.
   With rules 3 and 4, the 32M model reaches more gold than the 8M model with rule 3 alone, and
   shows fewer near misses on empty searches .
5. **Span.** The block.

**Merge.** Word hint 1, vector hint 1, word hint 2, vector hint 2, and so on, at most 3 in total. A
list that runs out leaves its turns to the other.

The `hints` field holds this merged list. The call's `text` is not rebuilt for vector hints.
If word hints exist, append a newline and `Nothing qualified. Near misses (not results):`, then
one line per word hint, numbered from 1:
`[i] label — heading — matched: words; missing: words — excerpt`. Use the record id for an empty
label, `(document start)` for an empty heading, `-` for no missing words, and omit the last
` — excerpt` when empty. Join word lists with `, `. With only vector hints the original silent
text stays. Later empty-page attempts can therefore retain text that describes a different hint
list from the final `hints` field.

**Fields**, for both: record, span, heading path, label, and the matched and missing query words.
A vector hint also has `via: "vector"`. A word hint has no `via` field. A word hint's excerpt is
the first line of the span that contains a matched word, whitespace collapsed, at most 160
characters. A vector hint's excerpt is the centred line of Card excerpt. The printed lines do not include
`via`, so the printed output does not say which list a hint came from.

Hints are never answers, and a vector hint never breaks a silence. The same nearest-block walk can
add one card when the page is already non-empty and still has room (Call-1 page). On dev the cosine
does not separate the answer from the rest (gold median 0.56, best non-answer median 0.58), which
is why a vector does not reorder the five. The cosine neighbor does not fill an empty page
(Call-1 page). The beam in Ordered fallbacks can, and only through a word search. After that beam leaves the page
empty, one surface-form block can, and only when the block shares a query word with `ln(N / df)`
at least 5. If that block is not shown, one section can, by the mean of its rare surface forms,
under the same shared-word rule (Ordered fallbacks). The hint, related-pointer and
outline layer checks eligibility against the query's `now`. A body of only whitespace
counts as empty there. The record fill and rare-neighbor walk apply lifecycle
checks before choosing a winner; the record fill retains word-search’s spaces-only body rule. Every returned card, hint and related pointer is checked
again before publication; removing one also removes its text from the response. Scope
does not exclude a record. This is lifecycle enforcement, not a score or threshold change.

### Related pointers

When call 1 has cards, up to 2 related pointers are added beside them. They never rank, and they
never count as an answer.

1. **Words.** At least 2 query content words.
2. **Candidates.** Sections of records that are not on the fusion's call 1 from before Call-1 page
   replaces it, and that are eligible. A record the replacement added can still be a pointer. A
   record the replacement dropped is still excluded. The section's heading-plus-text surface
   token set must be non-empty.
   Each must hold `max(2, ceil(n / 2))` of the words across its text, heading path and label. At
   least one of those words must be only in the heading path or label.
3. **Order.** Most words first, then most heading-only words, then the shorter section, record and
   offset. One per record.
4. **Content.** The span is the section if it is at most 1,500 bytes, else its first block. The
   excerpt is built as in Card excerpt.

### Outline

An outline lists a record's sections down to one level below its top heading level, at most 20
entries. Each entry has:
- a heading path
- a span: the section, or its first block if the section is over 1,500 bytes
- the first non-heading line, at most 160 characters

Entries include a non-empty preamble, if present. For the excerpt skip empty lines and lines
whose left-stripped text starts with `#`, then collapse whitespace in the first remaining line;
use an empty excerpt if none remains. With no headings, use top level 1. An ineligible or unknown
record, or no entries, returns no outline. Entries can be opened like cards.

## Static vectors

Replacement-encoder research is permitted when explicitly authorized by the
operator. It must retain one embedded model, offline native execution and
original-text ingest, measure fresh CLI and resident latency as well as encoding
cost, and use separate model-identified stores. A cheaper model or better raw
vector recall does not establish improved retrieval. Accepted model semantics
below stay in force unless LLP 0005's adoption requirements pass.

- **Model.** One static SSE v2 model, `RikkaBotan/stable-static-embedding-fast-retrieval-mrl-en-v2`,
  revision `928b418b2741b7f7b1b557a53845ac88ad5054c4`. It contains 30,522 token vectors
  of 512 float32 coordinates and three learned coordinate arrays. Provenance and
  licenses are in [THIRD_PARTY_NOTICES](../THIRD_PARTY_NOTICES).
- **In the binary.** Embed the tokenizer and safetensors at build time. Checked
  safetensors parsing and aligned little-endian F32 views borrow the table without
  copying it at startup. Inference is native Rust using the tokenizer crate and
  the numerical operations below; no runtime file, model download or second runtime.
- **Encoding contract.** Take the first 3,584 Unicode scalars, tokenize without
  special tokens, then retain the first 512 ids, including unknown-token ids.
  An empty sequence returns 512 zeros. Otherwise sum token vectors in input order
  with float32 arithmetic and divide by the number of ids. For coordinate i,
  compute `beta[i] * tanh(alpha[i] * mean[i] + bias[i])`. Use Accelerate's
  `vvtanhf` on macOS and scalar float32 `tanh` on other targets. Sum squared
  transformed coordinates in order in float32, take the square root, clamp the
  denominator to at least `1e-9`, and divide once. Do not remove unknowns or add
  another normalization. The zero vector stays zero. Batch encoding applies this
  operation independently to each text; an empty member does not alter other
  members' arithmetic. The independent float64 equation fixture allows `2e-6`
  absolute error; it is not a cross-platform bit-identity claim.
  Apply this same encoder to blocks, queries, words, forms and sentences. Section
  means and retrieval similarity retain the separate reduction rules below.
- **What is embedded.** At ingest, each index row's body text (a block, at most about 1,400
  bytes, or larger for an oversized unit). Do not prepend the separate heading field; any
  heading line already inside the block text is included. At query time, the characters left
  after filter prefixes are removed, with whitespace stripped from both ends. Quote characters
  stay. An empty remainder is not embedded.
- **Search.** Exact: the dot product of the query vector with every stored vector. Silence hints
  use the order of Hints on silence. The extra card of Call-1 page uses the same order and stops at the first
  block that shares a rare query word. The scan is exact; no approximate index is used.
- **No derived data.** No term lists, synonym tables or neighbour tables are made from the model.
  Only the model itself is shipped. The beam caches one unit vector per attested index word.
  An empty page also caches one unit vector per tricky surface form, one unit vector per
  section that holds a rare form, and one unit vector per sentence of a block that has at least
  two sentences (Ordered fallbacks). Those caches are encodings of text the index already
  stores, not a table the model invented. `forget` drops the cache keys (Ingest). The stored
  vectors stay until the next search replaces them.

### Call-1 page

This section is the call-1 procedure that was scored. A reader does not need another file to
carry it out. Calls 2 and 3 stay as Two matchers built them. When both matchers stored an empty ranking,
the page built here stays empty: no one-under section, no heading tree, and no neighbor is added.
The empty-page rewrites in Ordered fallbacks may still run. A quoted basename may then be placed first
whether or not this page has a card.

**Words.** `norm` lowercases, applies Unicode NFD decomposition, and removes characters whose
Unicode category is `Mn`. Matcher scoring words are `norm` of each
`[\w][\w\-]*` token. The two stop lists are the full list in Query language, and that list without `not`
and `no`. Matchers drop the shorter list. The neighbor, the beam, the fallback checklist, and
the centred excerpt do not use `norm`. Their tokens are the same pattern, lowercased, with
combining marks kept:

```python
def surface(text):
    return [w.lower() for w in re.findall(r"[\w][\w\-]*", text or "", flags=re.UNICODE)]

def content_words(query):
    text = split_filters(query)[1]          # filters removed, quotes left in the text
    ws = [w for w in surface(text) if w not in FULL_STOP and len(w) > 2]
    return list(dict.fromkeys(ws))          # first occurrence kept
```

**Parse.** Split `name:value` filters off the query. An unclosed quote or an unknown filter name
is a matcher query error; the outer rewrites can still run. Use the phrase/bare-run scan and
punctuation stripping in Query language, then `[\w-]+` for bare pieces, which can start with a hyphen.
This differs from the scoring/surface word pattern above, which must start with a word
character. A bare piece whose `norm` is a matcher stop word is dropped; phrases keep stop
words. Duplicate `(text, is_phrase)` pairs are dropped, keeping the first.

**Content and hard terms.** A term is hard when it is a phrase, or its text contains `_` or `-`,
or an adjacent lowercase letter and uppercase letter. A term is a content term when it is
a phrase, or `norm` of its text is not a kind word (Query language). `c` is the number of content terms.
`need = min(c, max(2, ceil(c / 2)))`. A query with no content term matches nothing.

**Matcher A** does not consult the stemmed table. Neither matcher has an author-terms column.
An ordinary word matches heading, body, or
parts. A hard word or a phrase matches heading or body only. A candidate holds every hard term and
at least `need` content terms. Counts are integers. Quote each term for FTS5. A phrase is the
content terms' original texts joined by one space, matched on `{heading body}` only.

```python
def is_exact(term):
    word, phrase = term
    return phrase or "_" in word or "-" in word or bool(re.search(r"[a-z][A-Z]", word))

def fts_expr(term):
    cols = "{heading body}" if is_exact(term) else "{heading body parts}"
    return cols + ":" + fts_quote(term[0])
```

**Identifier precedence.** Except for source-witnessed nominal compounds above, matcher B requires any term containing `_` or `-`, including a phrase, to match the exact-token body column. Other phrases and camelCase match heading and body. Matcher A uses heading and body for hard terms, including identifiers; neither matcher permits the parts column to satisfy an identifier.

**Matcher B** stems heading and body with Porter into a second table. Any term containing `_`
or `-`, including a phrase, matches only through the exact-token body column before the
source-witnessed nominal-compound augmentation. Any other term
also matches through the Porter table by submitting its entire FTS-quoted text as a `MATCH`
expression. A multi-token term requires a phrase in that stream; do not reduce it to one stem.
The joined phrase of content terms is matched there too, unless some content term contains
`_` or `-`.

```python
def stem_hit(term):
    word, _phrase = term
    if "_" in word or "-" in word:
        return None
    return fts_quote(word)  # MATCH against the porter table
```

Every term hit counts 1 toward coverage, including a permitted parts or stem hit.
There are no half-credit author-term hits. Identifier matches still obey the
exact-column rules above.

**Heading text in the index.** Only the first block of a section stores the heading in the heading
column, and only that first block's heading is copied into the stemmed table. Later blocks store
an empty heading in both. Every in-memory block still carries the section heading and the heading
path. The tree, the neighbor, the checklist, and the beam read that heading, not the heading
column and not the stemmed table.

**Heading trees, built once at ingest.** Group matcher B's blocks by record and by heading path.
The path is the tuple of ancestor headings and the section's own heading. Text before the first
heading has the empty path. A direct tree is one path plus the paths exactly one level deeper. A
deep tree is one path plus every longer path that starts with it. An empty path exists only when
the record has text before its first heading. That empty path's deep tree is the whole record. A
record that starts with a heading has no empty-path tree. Each tree stores the union of `norm`
words from each block's heading and text, and the list of `(section id, that block's words)`.
Union coverage counts distinct normalized term strings, while `need` uses the full parsed
content-term list length. This also applies to the record-word fill.
Structural membership is built from accepted, non-close, non-closed, non-superseded
records whose bodies are non-empty after stripping whitespace. Expiry is deliberately
not applied at ingestion: a historical query must still be able to use those trees.
At query time exclude records expired at the caller's clock before choosing winners.
Membership is always shared: original scope does not exclude a record. There is no
null-scope-only mode. Word-search scope and its penalty remain unchanged.

#### Step 1. Extra rows, then five records

For each matcher, start from the stored ranking of Two matchers. If that ranking is empty, add nothing.

**One section a word under `need`, both matchers.** Consider only sections that appear in the
positive content-match counts. Those counts are the matcher's filtered hits, so a row `under:` or
`path:` already rejected is not here. Keep a section whose count is at least `need - 1` and
strictly below `need`, that contains every hard term, and whose record is not already in the
stored ranking. A section with no
content-term hit is never a candidate, including when `need` is 1. Keep one: the highest BM25 from Scoring (BM25 over the searched rows), then the lower section
ordinal. If those tie, keep the section that was counted first. Append it as tier 2. Its score is
its own BM25. Matcher B's count uses the same 1-and-0.5 weights as its candidacy.

The coverage cache key is `(identity of matcher store, query, caller scope)` and omits `now`.
It stores content and hard terms, filtered hit sets, positive counts, all parsed terms, and the
BM25 `N`, average length and `df`. Reuse those values on a later hit of that key, even when
`now` has changed. Clear this cache on build and forget. The matchers' own result/ranking keys
include `now`, and their statistics and eligible-row keys are `(caller scope, now)`.

**Covering trees, matcher B only, and only when its stored ranking is not empty.** Skip a record
already in that ranking or chosen as the one-under section. A tree qualifies
when its union contains every hard term and at least `need` of the content terms. A hard phrase is
the whole quoted string after `norm`, not its separate words, so it is absent from the union
unless the phrase is a single token. The same is true of a content phrase. Skip a block that fails
the query's filters (Query language); `scope:` still does not reject, because the stored scope is null. The
tree's section is the block with the most content terms in its own words, then the lower section
id. One row per record. Walk direct trees first, then deep trees. Replace the record's row only
when the new block contains strictly more content terms. An equal count keeps the earlier tree, so
a deep tree wins only by having a strictly richer block. The row is tier 2. Its score is that
block's BM25.

Append those rows to the stored ranking. Then, walking that list in its existing order, keep the
first row of each record and give it key `(tier, -score + (7 if the record is far else 0),
arrival)`. Sort by that key ascending. Keep five. The card text is the block's `text` from the
first character, cut at 180 characters. The heading is the path joined by ` > `.

#### Step 2. Interleave

Take the two lists of up to five. Emit A1, B1, A2, B2, and so on. Drop a card whose record equals
an earlier card's record and whose byte span overlaps that card's span by at least one byte. Keep
five. Two non-overlapping blocks of one record can both stay.

```python
def overlaps(a, b):
    return a["record"] == b["record"] and a["span"][0] < b["span"][1] and b["span"][0] < a["span"][1]

def interleave(a, b):
    out = []
    for i in range(max(len(a), len(b))):
        for src in (a, b):
            if i < len(src) and not any(overlaps(src[i], kept) for kept in out):
                out.append(src[i])
            if len(out) == 5:
                return out
    return out
```

#### Step 3. Append, and do not rearrange

Run this only when step 2 produced at least one card and fewer than five. Stop at five. None of
these appends applies the penalty. A far record can take the open slot.

1. **Direct trees, at most two.** From the direct trees, among records not already on the page,
   keep a tree only when its word union contains every hard term and at least `need` content
   terms, and at least one block passes the query filters. For each record keep the qualifying
   heading whose union covers the most content terms. If two headings tie, keep the one
   encountered first while the index was built. Order the records by that coverage descending,
   then by record id descending. Append up to two, while the page has room. Choose the card's
   block among those that pass the filters, by most content terms, then lower section id.
2. **One deep tree, if the page still has room.** The same rule over deep trees, excluding records
   already chosen. Append one.
3. **One record a word under `need`, if the page still has room.** Over every matcher B section,
   excluding lifecycle-ineligible records at the caller’s clock. A record qualifies from the union of
   heading-plus-text words across all of its blocks, not from one block: every hard term and at
   least `need - 1` content terms must occur somewhere in that union. The block that is shown need
   not meet that test by itself. This walk applies lifecycle eligibility but not the query filters. Most
   content terms in the union, then the higher record id. The card is that record's block with the
   most content terms, then the lower section id.
   There is no upper bound below `need`: a record with full coverage can qualify too. When
   `need` is 1, a record with zero content-word hits can qualify if it meets the hard-term test.

The tree cards and this last card use one excerpt. Find the earliest case-insensitive match of any
parsed query term in the block text. Start 50 characters before it, or at 0 when there is no
match. Take 180 characters from there.

4. **One rare neighbor, if the page still has room.** Embed the query text with filters removed
   (Embeddings). If that text is empty, or the query has no content word under the full stop list and
   the length rule above, append nothing. Otherwise walk the index rows by descending dot product
   with the query vector. The rows are stored in section-id order, and a tie keeps the lower
   section id (`argsort` of the negated scores, stable). Skip a row whose record is already on the
   page. A row qualifies when the intersection of the query's content words with `words` of its
   heading plus its text contains a word whose `ln(N / df)` is at least 2.5. `N` is the number of
   embedded blocks. `df` is how many of those blocks contain the word in the heading or the text.
   A word missing from `df` is treated as `df = 1`. Apply the rare-word and already-shown-record
   exclusions before loading vectors: when no candidate remains, no vector work is needed.
   Otherwise retain the full matrix's numerical scoring and ordering. Only candidate
   vector payloads need reading: preserve their original positions in the full-sized
   matrix, zero unused rows, discard unused outputs, and use the unchanged kernel.
   Reuse an already-resident full matrix. Return the first qualifying row. This walk does
   apply lifecycle eligibility but not query filters, so the block can be outside
`under:` and `path:`. The card text is the
   block's `text` from the first character, cut at 180 characters. One record.

```python
query_words = set(content_words(query))
for j in np.argsort(-sims, kind="stable"):
    sec = rows[vec_ids[j]]          # rows stored in ascending section id
    if sec["record"] in have or not eligible(sec["record"], now):
        continue
    shared = query_words & set(surface((sec.get("heading") or "") + " " + sec["text"]))
    if shared and max(math.log(N / df.get(w, 1)) for w in shared) >= 2.5:
        return sec
```

The card is `record`, `span` `[start, end]`, `heading` the path joined by ` > `, `section` the
section id, and `text` the block from the first character, cut at 180. It has no `labels` and no
`fetch`. The rows were embedded in ascending section id, so a tied dot product keeps the lower id.

When steps 1–3 before the neighbor produce a page, replace call 1's text with one
`heading: text` line per card, joined by newlines, without numbering. The neighbor appends its
line and strips whitespace from the combined call text. Fields such as `related` are retained
when the cards and text are replaced.

**Left out, after being measured.** The same neighbor on an empty page raises gold first and drops
holdout no-gold from 55/106 to 53/106. Replacing the last card of a full page with it drops gold
first on both corpora. Keywords taken from the model's nearest vocabulary tokens do not beat this
page. Choosing the shown block by those keywords drops section. Mixing BM25 with cosine inside a
tier, at cosine weight 0.2, cleared dev and lost holdout cross-scope and no-gold.

## Qualified identifier suffix resolution

Before the ordinary agent search, parse the original query. If parsing fails,
leave it unchanged. Visit unquoted whitespace-delimited atoms with the existing
quote-preserving rewrite walker. A probe contains at least two underscore-separated
pieces, each at least three uppercase ASCII letters. Keep exact token matches:
if the lowercased probe exists in the index vocabulary, do not resolve it.

For each other probe, intersect its pieces in the existing parts-column FTS index,
restricted to word-search eligibility at the caller's clock. Load only candidate
blocks and apply the parsed record/block filters. Scan their original body text
with the ordinary surface-word regex. A full name must satisfy the same uppercase
piece grammar and end with an underscore followed by the complete case-sensitive
probe. Deduplicate full names across records. Exactly one distinct full name
licenses replacing that bare query atom; zero or multiple names leave it alone.
All probes resolve against the same original parsed filters. Quoted atoms and
filter values are not rewritten. The resulting query enters the ordinary agent
pipeline, including embedding and fallback operations; bodies and indexes never
change. This is query-time addressing, not an author vocabulary or ingest step.

## Ordered fallbacks

After qualified suffix resolution, the page of Call-1 page is built first.
Before that search, the unattested-camel rewrite below has already been applied to the query. If call 1 then has no card, further attempts run, in order, and each stops when it
keeps a card. Those attempts do not run on a page that already has a card. The quote walk below copies
quoted spans it encounters at the current position.
The hyphen search and the wordpiece search each call the page of Call-1 page and do not call this
section again. The beam does call this section, except that it does not start another beam. The
surface-form step does not search and does not start another beam. The section step does not
search either. One later attempt rewrites the query and searches it through this section, and it
does not start another rewrite of the same kind. The rare-form attempt does not search and does not
encode a form as one string. After those attempts, technical file-stem addressing runs, followed by quoted
basename placement, whether
or not call 1 has a card. That step does not search and does not encode the name.

Porter, where a step uses it, is one stem per independent input. Lowercase the
word with the existing Rust Unicode lowercase operation, tokenize with
`porter unicode61` and no extra token characters, then take the lexicographically
first stem. If no tokens are emitted, return the lowercased input. This is the
same result as inserting that input alone into FTS5 and taking its first vocabulary
row; it is not necessarily the first token in source order. `_` and `-` split
inputs here. The index helper instead uses `porter unicode61 tokenchars '_-'`,
takes the original input without that extra lowercase pass, and returns all unique
stems in lexicographic order for confidence.

Both helpers call the existing pinned libSQL tokenizer with document-tokenization
flags through `src/nlp/porter.rs`. Preserve its Unicode61 boundaries, folding and
diacritic behavior, Porter rules and byte-length limits: Porter passes tokens
shorter than three bytes or longer than 64 bytes through without stemming. No
second stemming algorithm is introduced. The persistent FTS stem index remains
unchanged. Per-word caching and prewarming may avoid repeated work, but never
combine independent inputs or substitute one tokenizer configuration for the other.

**Literal colon recovery, after an empty first search.** After the unattested-camel
rewrite and its initial base search, try this before the hyphen and wordpiece
attempts. It applies only when parsing reports `Unknown prefix`. Scan
`(^|\s)([a-z][a-z-]*):([A-Za-z][A-Za-z0-9_-]*)`. The name must be unknown to the
query parser; the atom must be outside double quotes and end at whitespace or
end-of-query. Known filters, punctuation-suffixed atoms and malformed queries
are not silently reinterpreted. Quote each qualifying complete atom, preserving
other bytes, and require the resulting whole query to parse successfully.

Apply qualified suffix resolution to that valid quoted query as well, so an
unknown colon atom cannot block an independent suffix recovery. Run base search
and the existing fallback gate. In every returned call, retain only cards whose
original source block contains every recovered colon atom exactly, case-sensitively.
The characters immediately before and after an occurrence must not be alphanumeric,
underscore, hyphen or colon. Rebuild each call's text from its surviving cards
with the ordinary unnumbered card formatter. If any call retains a card, return
those calls and stop empty-page attempts; otherwise continue with the original
hyphen/wordpiece chain. Existing hints/pointers remain metadata, not qualifying
cards. This never writes the recovered atoms into document content.

**Unattested camelCase, before the first search.** A token matches when it is one letter, then
letters or digits, with a lowercase letter immediately before an uppercase letter somewhere after
the first character. If that token contains no `:`, and `vec_df` has no entry for its lowercased
form, split it the same way `pieces` does. `GitHub` and `TestFlight` stay whole, because the whole
token is stored. `iOS` does not match: the pattern requires a lowercase-to-uppercase pair after
its mandatory first letter, and `iO` starts at that first letter.

```python
CAMEL = re.compile(r"^[A-Za-z][A-Za-z0-9]*[a-z][A-Z][A-Za-z0-9]*$")

def split_unattested_camel(token, vec_df):
    if ":" in token or not CAMEL.match(token):
        return token
    if vec_df.get(token.lower()):
        return token
    return re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", token)
```

Walk the query once. If the current character is `"`, copy through the closing quote, or through
the end when there is none. Otherwise take the next run of non-space and replace it with
`split_unattested_camel`; copy a whitespace character as-is and advance one character.
The pattern is applied to that entire run. A comma, period, or other mark stuck to the token
makes the pattern fail, and the token is left whole. The hyphen and wordpiece rewrites use this
same run, so the same mark blocks a hyphen split and is included in the text the tokenizer sees.
A quote inside a run is consumed as part of that run, not recognized as an opening quote. A
quoted span, including a quoted filter value, is copied whole. An unquoted filter is one
non-space run, and a colon anywhere in that run leaves the run unchanged. Text after that run
is a later token and can be rewritten.

**The fallback gate.** The hyphen search and the wordpiece search keep a card when either test
passes for the rewritten query.

The checklist stems every surface token of the section's in-memory heading and its full block
text, not the card excerpt. A token that contains `-` also contributes the stem of each alphabetic
piece longer than two characters. The checklist does not split `_` or a camelCase boundary before
stemming. The card passes when every content word of the rewritten query has its stem in that
set. A missing section fails the checklist. Content words are `content_words` (Call-1 page).

```python
def section_of(store, card):
    cached = store.get("fallback_sections")
    if cached is None:
        rows = store["base"]["a"]["sections"]
        seq = rows.values() if isinstance(rows, dict) else rows
        cached = {sec.get("sid"): sec for sec in seq}
        store["fallback_sections"] = cached
    return cached.get(card.get("section"))

def checklist(store, card, query):
    sec = section_of(store, card)
    if not sec:
        return False
    stems = set()
    blob = (sec.get("heading") or "") + " " + (sec.get("text") or "")
    for w in surface(blob):
        stems.add(porter(w))
        if "-" in w:
            for piece in w.split("-"):
                if len(piece) > 2 and piece.isalpha():
                    stems.add(porter(piece))
    need = content_words(query)
    return bool(need) and all(porter(w) in stems for w in need)
```

The tier test reads the stored rankings of both matchers for this rewritten query. A ranking row
is `(tier, -score, body hash, ordinal, record id, section id)`. The card's tier is the minimum
tier of any row whose section id is the card's, or none when the section was not ranked. Tier 0
or tier 1 passes. Tier 2 does not.

```python
def tier_of(store, card, query):
    best = None
    for which in ("a", "b"):
        for key, rows in store["base"][which]["rankings"].items():
            if not key or key[0] != query:
                continue
            for row in rows:
                if row[5] == card.get("section"):
                    best = row[0] if best is None else min(best, row[0])
    return best

def fallback_keeps(store, card, query):
    if checklist(store, card, query):
        return True
    tier = tier_of(store, card, query)
    return tier is not None and tier <= 1
```

**One alphabetic hyphen.** If call 1 has no card, split every unquoted run that consists of
exactly two alphabetic pieces and one hyphen. Several runs can split in the same retry. A digit,
an underscore, a colon, or a second hyphen leaves that run whole.

```python
COMPOUND = re.compile(r"[A-Za-z]{2,}-[A-Za-z]{2,}")

def split_one_hyphen(token):
    if ":" in token or "_" in token or any(c.isdigit() for c in token):
        return token
    if token.count("-") == 1 and COMPOUND.fullmatch(token):
        return token.replace("-", " ")
    return token
```

Walk the query the same way as the camel split, so spaces stay and a quoted span is copied whole.
Search the rewritten query with the page of Call-1 page. Take that page's first five cards and keep
those `fallback_keeps` accepts. Cards the gate drops are not replaced, and the neighbor is not
run again. If the rewrite did not change the query, or no card is kept, the
silent result stays, including any hints on it, and the next rewrite runs. The next rewrite sees
the query from before this hyphen split.

When cards are kept, the returned calls are that second search. Call 1's cards are the kept
cards. Call 1's text is rebuilt as one `heading: text` line per kept card. Calls 2 and 3 of the
rewritten query stay.

**Wordpieces.** If the page is still empty, start again from the query after the camelCase split,
with hyphens still in the tokens. Replace each unquoted token that contains no `:` with the pieces
from the static model's tokenizer. Use that same tokenizer: encode without special
tokens, drop a leading `##`, and keep a piece that is alphabetic and longer than two characters.
The embedding is not read. A token with no such piece stays as written. If no token changes,
skip this retry; the beam can still run.
Otherwise search that query with the page of Call-1 page, take that page's first five cards, keep
those `fallback_keeps` accepts, and return that second search the same way the hyphen search
does. If none are kept, the silent result stays.

```python
def model_pieces(tokenizer, token):
    enc = tokenizer.encode(token, add_special_tokens=False)
    out = []
    for piece in enc.tokens:
        if piece.startswith("##"):
            piece = piece[2:]
        if piece.isalpha() and len(piece) > 2:
            out.append(piece)
    return out

def apply_pieces(token, tokenizer):
    if ":" in token:
        return token
    parts = model_pieces(tokenizer, token)
    return " ".join(parts) if parts else token
```

`apply_pieces` runs inside the same quote walk. A quoted span is copied unchanged.

**Two-word beam.** If the page is still empty, propose two index words and search those two words
through the camelCase split, the hyphen retry, the wordpiece retry, and the page of Call-1 page. That
search can still gain a short-page neighbor. It does not start another beam, and its calls 2 and 3
are discarded. Then keep a card only when the original query's content words meet the term count
in that card's section. The two proposed words are not the count.

The proposal list is every alphabetic word in `vec_df` longer than two characters whose
`ln(N / df)` is at least 2.5. Build `vec_df` by walking blocks in ascending section id and then
iterating the set of surface words in each block's heading plus text. That set yields each
word once, in the order `surface` first produced it, left to right. The proposal list follows
the dictionary's insertion order: a word keeps the place of the first block, in ascending
section id, that contained it. It is not alphabetical. Each word is encoded as in Embeddings. The cache key is the
number of blocks and the sum of the document frequencies. `forget` drops the key
after it rebuilds `vec_df`, so the next search embeds the list again. The target is the unit
vector of the query text with filters removed. If that text is empty, or `content_words` is empty,
or the proposal list is empty, add nothing. The prefix is `content_words` of the original query.
The beam starts at the sum of the prefix words that are in the proposal list. It grows exactly two
words. A word already in the prefix or already chosen is not chosen again. It keeps four partials,
and tries the twenty proposal words with the highest dot product against the residual. A partial's
score is the cosine of its mean with the target. Tied dots follow `argpartition` order. Tied
scores keep the earlier partial. The search query is the two added words in the order they were
grown, joined by one space. The prefix is not part of that query.

For the beam, the original query is the caller's input before the camelCase split. Both its
target vector and its content-word checklist use that input. The hyphen and wordpiece retries
instead receive the query after the camelCase split.

```python
def beam(words, vecs, index, target, prefix, n_extra=2, width=4, topk=20):
    if not len(words) or target is None:
        return []
    acc = np.zeros(vecs.shape[1], np.float32)
    n = 0
    for w in prefix:
        j = index.get(w)
        if j is not None:
            acc = acc + vecs[j]
            n += 1
    beams = [(0.0, acc, n, [])]
    banned = {index[w] for w in prefix if w in index}
    for _ in range(n_extra):
        cand = []
        for _, acc, n, extras in beams:
            if n:
                mean = acc / n
                mean = mean / max(float(np.linalg.norm(mean)), 1e-9)
                residual = target - mean
            else:
                residual = target
            dots = vecs @ residual
            for j in list(banned) + extras:
                dots[j] = -1e9
            top = np.argpartition(-dots, min(topk, len(dots) - 1))[:topk]
            for j in top:
                if dots[j] < -1e8:
                    continue
                new_acc = acc + vecs[j]
                new_n = n + 1
                m = new_acc / new_n
                m = m / max(float(np.linalg.norm(m)), 1e-9)
                cand.append((float(m @ target), new_acc, new_n, extras + [int(j)]))
        cand.sort(key=lambda t: -t[0])
        beams = cand[:width]
        if not beams:
            return []
    return [words[j] for j in beams[0][3]]
```

Let `n` be `len(content_words(query))` and `need = min(n, max(2, ceil(n / 2)))`. A returned card
is kept when its current matcher A section (looked up directly, not through the fallback's cached
`section_of`) contains at least `need` of those words in `surface(heading + " " + text)`.
The comparison is the surface tokens, not stems, and it uses the
full block, not the card excerpt. A card whose section is missing is dropped. Up to five are kept,
in the order that search returned them. If none pass, the silent result stays. When some pass,
those cards replace the silent call's cards. That call's text and any hints already on it stay as
they were. The beam does not run on a page that already has a card. Growing one word instead of
two, or also running the beam on a short page, changes pages and does not raise a floor.

**One surface-form block.** If the page is still empty, a form collected from the blocks may name
one block. A form matches when the heading or the block text contains a hyphenated run (letters,
then one or more groups of a hyphen or an underscore plus letters or digits), a camelCase run (a
letter, then letters or digits, with a lowercase letter immediately before an uppercase letter),
or a run that contains a digit. The matched text is kept when it is at least five characters after
lowercasing. Its pieces are that text with each lowercase-to-uppercase boundary opened, then split
on everything that is not a letter or a digit; a piece is kept when it is alphabetic and longer
than two characters. The same lowercase form keeps the pieces of its first match. Each distinct
form is encoded once, as in Embeddings. Its document frequency is the number of records that contain
it, not the number of blocks.

On an empty page, encode each content word longer than three characters. A form is a candidate
for that word when it is among the fifteen with the highest cosine to the word, that cosine is at
least 0.70, the word is one of the form's pieces, the form is not the word itself, the form has
at least one other piece, and the form occurs in at most eight records. A form such as `windows-2`
has no other piece and is not a candidate. The first query word for which a form passes those
word tests claims that form, before the block test. A later query word cannot claim it and cannot
replace that word's cosine. If the block test then fails, the form stays claimed and is not tried
again. For each form that also passes the block test, take the eligible block that
contains it and whose cosine to the query text, filters removed, is highest. Keep that block when
the cosine is at least 0.35 and the block contains a content word of this query, longer than three
characters, whose `ln(N / df)` is at least 5. A word absent from `vec_df` uses `df` 1, the same
stand-in the other IDF reads use. Several forms can survive. Show the one that occurs in the
fewest records, then the one with the higher cosine to the query word, then the earlier form in
alphabetical order. Tied cosines inside the fifteen follow `argpartition` order.

The card is that block. Its excerpt is the block from the first character, 180 characters. It
replaces the silent call's cards. That call's text and any hints already on it stay. This step
does not run on a page that already has a card, and it does not run a word search.

**One section, from the rare forms inside it.** If the page is still empty, each section that
contains at least one of those surface forms in at most eight records gets one vector: the mean
of one copy of a form's vector for every time that form matches, normalized as in Embeddings. A second
match of the same form, including one in the heading and another in the block text, adds the
vector again. A form in more than eight records is left out of the mean. A section with no such
form is not a candidate. Take the eligible section whose vector
is closest to the query text with filters removed, and show it only when that cosine is at least
0.35 and the section contains a content word of the query, longer than three characters, whose
`ln(N / df)` is at least 5. A tie keeps the section encountered first while the forms were
collected. The card is that section's own block, excerpted from the first character, 180
characters. It replaces the silent call's cards. Text and hints stay. This step does not run on
a page that already has a card. Choosing a different section of a record the page already shows,
because that section contains more specific query words, moves correct sections off the page and
is not done.

**Two sentences for two halves of the query.** If the page is still empty, take the content
words in first-seen order. Fewer than three, and this step does not run. Three words become the
phrase of the first two and the phrase of the last two. Four or more split in half, and each
phrase has to contain at least two words; otherwise this step does not run. A sentence is a
stretch of a block's own text of at least four words, broken on a newline or after `.`, `!`, or
`?`, and cut at 500 characters. A block with fewer than two such sentences is not a candidate.
Encode each sentence, and encode each phrase, as in Embeddings. Take the fifteen sentences nearest
each phrase. Show the eligible block that contains one sentence from each fifteen, on two
different sentences, when both cosines are at least 0.35. The block shown is the one whose lower
cosine is highest. A tie keeps the pair first returned among those fifteen. The card is that
block, excerpted from the first character, 180 characters. It replaces the silent call's cards.
Text and hints stay. No query word has to occur in the block. This step does not run on a page
that already has a card, and it does not replace the two sentences with their mean.

**An identifier the index has never stored.** If the page is still empty, walk the query. Copy a
quoted span whole, including a quoted filter value. An unquoted filter is one non-space run, and
a colon anywhere in that run leaves the run unchanged. Any other unquoted run is opened when it
contains `-`, `_`, a lowercase letter immediately before an uppercase letter, or a digit, when
the exact vocabulary of the first matcher has no row for its lowercased form, and when the
pieces below are at least two. A token the vocabulary already stores stays whole.

Insert a space between a lowercase letter or digit and the following uppercase letter. Split on
every character that is not a letter or a digit. Keep a piece only when it is entirely letters
and longer than two characters. A piece that still contains a digit is dropped, so
`alpha2-beta-gamma` opens to `beta` and `gamma`, not to a piece `alpha`. The rewritten query
replaces the token with those pieces, separated by spaces. Search that query through the page of
this section. Do not
start this rewrite again on that search.

Keep a card from the first call of that search only when the section contains every piece of
every opened token. Stem the piece, and stem each surface word of the section's heading and its
full block text, with the Porter step above. Each piece's stem has to be in that set. Do not
split `_` or a camelCase boundary in the section before stemming, and do not require any other
word of the query. A missing section is read from the card excerpt and fails when the pieces are
not in that excerpt. Keep the first five such cards, in the order that search returned them.
They replace the silent call's cards. Text and hints stay. This step does not run on a page that
already has a card.

**A rare form the query explains.** If the page is still empty, take the content words: the query
after filters are removed, lowercased, outside the stop list, longer than two characters, and
unique in first-seen order. Those words are already lowercased, so a lowercase-to-uppercase
boundary is not visible here. The unattested camelCase split above has already run, on the raw
query, before the first search. A content word the exact vocabulary has never stored, and that
contains `-`, `_`, or a digit, is replaced by the same pieces as the identifier split above:
entirely letters, longer than two characters, with a digit-bearing piece dropped. That replacement
is kept when at least two such pieces remain. Otherwise the word stays. Other words longer than
two characters stay. After that replacement, drop a repeated probe string and keep the first.
`prior-art prior art` is the two probes `prior` and `art`. Fewer than two probes, and this step
does not run.

Stem each probe, and stem each distinct piece of a surface form that occurs in at most eight
records and has at least two such pieces. The pieces are the set from the form's first match, so
a repeated piece string counts once. The query explains the form when each piece's stem consumes one
probe stem that has not already been used. A stem the query has once cannot cover two pieces. At
least one piece must be a surface the probes did not write, so a hyphen joining words the query
already spelled is not a form. The form with the most such new pieces is kept, then the one with
more pieces, then the one in fewer records, then the greater form string. The card is the shortest
eligible section that contains the form. Length is the section text, and the first shortest
section wins. The excerpt is that text from the first character, 180 characters. It replaces the
silent call's cards. Text and hints stay. The form is not encoded as one vector. This step does
not run on a page that already has a card.

**A technical file stem.** After all empty-page attempts, whether or not call 1
has cards, parse the original query. On a parse error do nothing. For each bare,
unquoted term containing `_` or `-`, lowercase it as the candidate filename stem.
Take every other content term with a different lowercase spelling, split with
`surface`, and collect its per-word Porter stems. At least one stem is required.
Seek the basename index in `[stem + ".", stem + "/")`, ordered by name and record.
Keep only names whose final-dot split has exactly that stem and a nonempty
extension; a longer stem or compound extension does not qualify. Check current
eligibility and all parsed filters. In candidate blocks, stem the surface words
of the original block text only. Every required stem must occur there. Neither
separate headings nor filenames supply this evidence. Across all candidate terms,
place a card only if every qualifying block belongs to exactly one record. Use
its earliest start offset, breaking a tie by block id. Prepend the card with the
first 180 characters of its own text, remove other cards of that record, and cap
call 1 at five. Preserve existing call text, hints, pointers and later calls. If
there was no call, create one with the excerpt as its text. The quoted-filename
step below runs afterward and retains final placement priority.

**A quoted filename.** Outside the two addressing rules, free text does not read
`source_path`. `path:` is the glob in Query language. This
step is not that glob, and it runs after every attempt above, whether or not call 1 has a card.

The names are the map from ingest (Ingest): the final component of `source_path`, lowercased. Scan
the raw query for a closed pair of double quotes. The inside of the pair is compared to those
components without regard to case. The whole span has to equal the component. A longer path
inside the quotes does not match, a single quote does not match, and an unclosed quote does not
match. Count only eligible records at the caller’s clock. A component shared by two eligible
records is not placed by this single-record test. A component that names one eligible
record is kept. An ineligible record is skipped and does not
block another quote. One remaining record is placed. Two quotes that name two different eligible
records place nothing. The same record named twice is one record.

A component shared by two or more records is not skipped when the words outside every quoted span
can name one of them. Remove the quoted spans, then take the content words of what remains:
lowercased, outside the stop list, longer than two characters. Stem each one and keep the set.
Fewer than two distinct stems, and the shared name is left alone. A block of one of those records
qualifies when it is eligible and every one of those stems is among the stems of the content
words of that block's own text. The heading is not part of that test. Place a card only when
every qualifying block belongs to the same record. The card is the qualifying block with the
smallest start offset. If qualifying blocks name two records, the shared name is left alone and
the page from the single-record rule above stays as it was.

Either card is placed the same way. For the single-record test, the section is the one with the smallest start offset. The heading is the section
path joined by ` > `. The excerpt is that section's text from the first character, 180
characters. Place the card first on call 1. Earlier cards keep their order, a second card for
the same record is removed, and the sixth card is dropped. Calls 2 and 3 stay. Text, hints, and
related pointers on an existing call stay, including a query-error text. If the search returned
no call, the result is one call whose text is that excerpt. The body is not changed, the
filename is not written into the index, and the filename is not encoded. `forget` updates the
map as in Ingest.


## Constants

| Constant | Value |
|---|---|
| Block size | 1,400 bytes |
| Cards per call | 5 (each matcher one per record; the fused call 1 can hold two blocks of one record) |
| Calls per search | at most 3; an empty additional-sections or navigation call is omitted |
| Heading weight in tf | 3 |
| Parts weight in tf | 0.3 |
| BM25L | k1 1.2, b 0.75 with a 0.25 length floor; delta 0.125 after length normalization, positive TF only |
| Coverage need | min(c, max(2, ceil(c / 2))) |
| Gate threshold | open (`-1`); the fitted 0.77 is not applied |
| Other-repository penalty | 7, inside the tier, before the cut to 5 |
| One-word admission | one section per matcher, count in `[need-1, need)`, tier 2, only if that matcher already returned a row; highest BM25, then lower ordinal |
| Heading-tree admission | matcher B only, full `need`, the section's own BM25, tier 2 |
| Short-page fills | up to 2 direct heading trees, then 1 deep tree, then 1 record at `need-1`, then 1 rare neighbor |
| Rare neighbor | nearest block, shared query word with `ln(N/df) >= 2.5`, one record, short non-empty page only |
| Empty-page rewrites | unattested camelCase split before the first search; then, only while the page is empty: one alphabetic hyphen, then wordpieces, then the two-word beam, then one surface-form block, then one section from the mean of its rare surface forms, then one block where two halves of the query land on two different sentences, then one search that opens an identifier the exact vocabulary has never stored, then one rare form whose every piece the query explains. The first two keep a card the checklist or an FTS5 tier of 0 or 1 accepts (Ordered fallbacks). The beam keeps a card that meets `need` on the original content words. The surface form keeps a block at block cosine at least 0.35. The section keeps a block when that mean's cosine is at least 0.35. Both require a query word with `ln(N/df) >= 5`. The two sentences keep a block when each phrase is at cosine at least 0.35 and no query word is required. The identifier search keeps a card only when the section contains every piece of each opened identifier. A piece that contains a digit is dropped. The surface form is claimed by the first query word that passes the word tests, before the block test. The section mean adds a form's vector once per occurrence. The last form uses the lowercased content words, so a camelCase boundary is not visible there; the unattested camelCase split has already run on the raw query. Probes are deduplicated after a token is opened. Fewer than two probes, and that step does not run. The form's pieces are the distinct pieces of its first match, and each consumes one probe stem. The form is kept only when one piece is a surface the query did not spell. Most such new pieces wins, then more pieces, then fewer records, then the greater form string. The card is the first shortest section by the length of its text |
| Technical file stem | bare `_`/`-` identifier, exact final-extension stem, at least one other content stem, all stems in one original block, exactly one qualifying record; earliest start then block id, 180-character excerpt, before quoted filename |
| Quoted filename | after that page is built, whether or not it has a card: a closed double-quoted span whose inside equals the final component of exactly one record's `source_path`, without regard to case, is placed first. A shared basename is placed first only when at least two distinct stems outside the quotes all occur in the block text of exactly one eligible record; the heading is not tested, and the card is that record's qualifying block with the smallest start. An ineligible record is skipped. Two quotes that name two eligible records place nothing. The card is 180 characters. Calls 2 and 3 stay. The body is not changed. `forget` removes the record from the name |
| Beam | 2 words, width 4, top 20 proposals, alphabetic index words longer than 2 with `ln(N/df) >= 2.5`. Score is cosine of the mean to the query vector |
| Call-1 excerpt | ranked cards and the neighbor: the block from the first character, 180 characters. Short-page tree and one-word appends: 180 characters starting up to 50 before the first parsed-term match. Step-1 admissions use the first 180 |
| Later calls and related pointers | the centred line, at most 160 characters, starting 40 before the first match |
| Word-hint excerpt | the first matched line, whitespace collapsed, at most 160 characters |
| Vector-hint excerpt | the same centred line as later calls |
| Hints | 3 (word and vector hints alternate); word spans at most 1,500 bytes; vector spans are whole index blocks and can exceed that |
| Vector hints | the 3 nearest eligible records not already word hints, kept if the block shares a query content word with IDF >= 2.5 |
| Embedding model | SSE v2 (pinned under Static vectors), 512 dimensions, float32, unit length (zero stays zero), exact dot product; encoding defaults under Static vectors |
| Related pointers | 2 |
| Outline | 20 entries, top level plus one |
| `show` | 1,500 characters (`--all` for the whole span) |
| Tested agent budget | 1,800 tokens (checked before each view), 12 calls, 40 list-view decisions, 2 own queries |

## Confidence diagnostics

**Row features**, 16 for each candidate row:
1. constant 1
2. IDF-weighted share of query terms with a body match
3. share of terms with a body match
4. share of content terms with a body match
5. `1 / (1 + max(0, window - matched) / 8)`, where `window` is the smallest token window that holds all matched words
6. the window is at most 16 tokens (0 or 1)
7. share of terms with at least one scoring word in the row's indexed heading
8. `ln(1 + row length) / 8`
9. rare terms (IDF mass >= 3, any column), at most 5, divided by 5
10. `ln(1 + score) / 5`
11. number of terms, at most 10, divided by 10
12. every term has a body match (0 or 1)
13. the query has a phrase (0 or 1)
14. share of terms with no body match in any eligible row
15. share of terms matched only in author terms (always 0: there are no author terms)
16. every term matches in some column, but not every term has a body match (0 or 1)

Features 15 and 16 are always 0, and their weights are 0.0. Retain the weights for diagnostics; they do not reject a result.

**Rank features**, 2, added to each row's features:
- `(s - o) / max(1, s + o)`, where `s` is the row's score and `o` the score of the first row of a
  different record in tier order
- the row is in tier 0 (0 or 1)

**Query features.** The top row's 18 features, then the maximum of each of the 18 features over the
rows of the first five records: 36 values in total.

**Confidence.** Let `z = clamp(w · x, -40, 40)`, then use `1 / (1 + exp(-z))`.
The active threshold is `-1`, so this confidence never triggers filtering. The fitted threshold
was 0.77. The weights are:

```
-1.3855513434691795, 0.9593878241251049, 0.7532728866585706, 0.17664441460611954,
 0.24737742197579676, 0.5919029905332762, 0.021904050941645846, -1.5402801189515525,
 1.0004888367231775, 0.7476384047906712, 0.3635049016679036, 0.2128668102983276,
-0.10219770424770558, -0.569738296022477, 0.0, 0.0, -0.5926065130891044, -0.1464830955386999,
-1.3855513434691795, 0.93621863635448, 0.8070180966375439, 0.3051328046511108,
-0.389265204940308, -0.04685987846064951, -0.5989768934538516, -0.5154316163892562,
 1.0868875274033045, 0.7797442807149891, 0.3635049016679036, 0.09247920198147007,
-0.10219770424770558, -0.569738296022477, 0.0, 0.0, -0.4994360850540691, -0.1464830955386999
```

The weights were fitted by five-fold cross-validation on dev cases (1,318 samples from 491 cases).
0.77 was the balanced threshold; no threshold reached Q1 0.92 out of fold.


## Compatibility ordering and caches

Prepare normalized content words and hard words once per parsed query. Coverage
still uses the full parsed content-term count, even when normalized strings
coincide. Candidate postings count distinct strings and retain the original
traversal order before selection. Indexed tree, section and record candidates
implement the same unions as full scans; apply the lifecycle checks defined above
before selecting winners.
Compile explicit path/cites glob predicates once per query, retaining both the
anchored/suffix regex and the literal substring fallback.

For each eligibility clock, compute N and average length from the indexed integer
expiry suffix totals described in LLP 0001. Resolve DF only for query scoring
words, using exact-term FTS vocabulary lookups, distinct block ids and the same
projected lifecycle and strict expiry predicate. Cache those counts by clock and word. Count a block once
across all indexed columns and occurrences. Deduplicate FTS block ids before
joining lifecycle eligibility, so repeated occurrences do not repeat those lookups.
Query filters never narrow these
statistics. This computes the existing full-vocabulary values on demand.
Load the query's normalized scoring terms' exact FTS frequencies before scoring
any row. Before computing confidence, load each query stem's positions from the
stem instance vocabulary. Heading/body/parts frequencies and offset order are
identical to the full instance scan. Row lengths always come from the existing
FTS heading and body token counts, independent of which terms have been loaded.
Aggregate each term's occurrences by block and column inside libSQL. Return the
integer occurrence count directly for term frequencies. Transfer numerically
ordered offset arrays only for body positions and stem positions, which retrieval
uses; heading and parts frequencies do not require position arrays. Do not transfer
and allocate a Rust tuple per occurrence. This uses the existing term index and preserves exact positions.

BM25 values are defined by the formula above for every row, but evaluate them
only when needed. Share a per-page row-score memo between the two matchers;
positive coverage rows get scores eagerly and an admitted tree's chosen row can
be scored on demand. Preserve the exact query-word arithmetic order. Coverage
snapshots retain their positive-row scores only while the caller's clock remains unchanged. Cache
first-count arrival positions for one-under ties; selecting the minimum full
key is equivalent to sorting and taking the first, since arrival is unique.

Nearest-block kernels still evaluate the full ordered matrix. For a short-page
neighbor, selecting the best qualifying block by cosine then lower id is
equivalent to sorting the whole matrix and taking the first qualifying block.
For vector hints first select each eligible record's best block, then the three
nearest records, resolving ties by lower block id. The full matrix carries
record ordinals for this selection; only selected block details are loaded.
This does not restrict the numerical population or change the matrix shape.
Apply the shared-word and IDF gates only after that selection; a rejected record is not replaced by another
block or record. If no indexed block contains any query content word reaching
the required IDF, vector work cannot produce that neighbor/hint and may be
skipped. This exact impossibility test is not a confidence gate or a candidate
cap. Do not replace global hint selection with a word-filtered nearest search.

Rare-form section means are content-derived and query-independent. Publish them
as the block-keyed ingestion index in LLP 0001, preserving occurrence weights and
first-encounter order. Use block surface postings to locate sections that can pass
the existing rare-shared-word test, and fetch only their stored means in that order. Individual mean/query comparisons remain vector
dot products. Packing cached matrices never changes normalization, batch size,
kernel shape, float32 gates or proposal/sentence introselect ordering.
Form, beam and sentence encodings may be prepared on writes and loaded from
LLP 0001's model- and publication-checked numerical projections. Preserve each
original encoding array and batch boundaries, and reload float32 bits without
renormalizing. Keep form metadata alongside one packed matrix: scoring and
occurrence-weighted section-mean publication read that matrix's rows directly,
without splitting it into per-form vector allocations and packing it again.
Read form metadata independently of that matrix. Before numerical form-block
matching, an empty indexed population of blocks sharing a query word at IDF 5,
or absence of any form satisfying the existing piece, nonidentity and df-at-most-8
conditions for any probe, proves that this stage cannot produce a card. Skip its
vector load and scoring in those cases and continue the unchanged fallback chain.
Otherwise load the complete original form matrix: do not restrict its top-15
population to those metadata witnesses. Metadata and vectors retain the same
model/publication guard and search snapshot. Lexical rare-form matching needs
metadata only; section-mean publication explicitly loads the matrix when needed.
Beam accumulation may likewise read the same packed rows instead of a duplicate
vector array. Moving preparation time never changes which stage runs
or the first-card stopping rule.

Record fills use the derived record-to-block lookup of LLP 0001, retaining the
same ascending block-id traversal and word unions as a full scan. Hints and
related pointers reuse the cached section surface-token sets from the original
heading/text and path/label; their candidate traversal, eligibility, sorting,
span selection and excerpts are unchanged. Recreate these content-derived
projections on writes and read matching words on demand through libSQL. Parsed
section and tree ids preserve the original traversal; lazy word sets contain
only queried members and cannot be used as an emptiness test for the original
section. Eligibility remains separate. A query snapshot and publication check
prevent combining different projection generations; see LLP 0001.

One-under admission breaks a score-and-ordinal tie by the order postings first
entered its count map. The native hit-order module reproduces the deterministic
integer posting-table traversal, including union growth; a random Rust hash
iteration or a section-id sort changes pages despite unchanged aggregate floors.

Cache scope statistics by eligibility clock. When the caller's clock changes, clear
coverage snapshots, rankings and confidence diagnostics before retrieval; no earlier
clock's admission or score may survive into the new query. Rebuild and forget clear
derived statistics and coverage; retained ranking diagnostics remain subject to current
eligibility. Cached model encodings contain only indexed text and are invalidated when
rows change. A block matrix stores lifecycle membership and expiry alongside unchanged
ordered vectors; apply the current clock to its eligibility mask on each use.
The confidence model is retained for diagnostics; its open gate never rejects a row.

Encoding uses the fixed mean-and-transform contract under Static vectors. Rebuild
visits blocks in section-id order. Batch encoding retains every input row in its
original position. Within a batch, byte-identical input text may reuse the first
computed vector by copying its exact float32 values; distinct text still uses
the unchanged per-text encoder. This temporary input-text map is discarded when
the batch returns. It changes neither numerical matrix dimensions nor scoring
batch boundaries. Forget slices surviving vectors without encoding them again.
Packing freshly encoded fallback rows consumes the individual vectors in order,
releasing them as their bits enter the packed allocation. Do not retain a complete
second copy merely to borrow rows during packing. Matrix shape, row order and
float32 operations are unchanged.
The model fingerprint and inference revision identify both block vectors and
query-independent fallback arrays; never mix publications from different models.

Sentence dictionaries in LLP 0001 are storage compression only. Restore every
occurrence to its original matrix row before either phrase is compared. Neither
the global top-fifteen population nor the matrix dimensions may shrink.

The two-sentence comparison is a dense float32 matrix multiplication, using the
system Accelerate implementation through CBLAS on macOS. Top-k probes retain
introselect's unsorted order, rather than sorting equal scores before choosing
the first sentence pair. These are numerical and ordering compatibility rules,
not new scores or thresholds. The selection attribution is in
[THIRD_PARTY_NOTICES](../THIRD_PARTY_NOTICES).

Use matrix-vector kernels for a nearest-block walk and probe proposals, a
matrix-matrix kernel for the two sentence phrases, and a vector dot product for
individual comparisons. Substituting scalar folds for those shapes can choose
a different near-duplicate passage at the last float32 bit. Form means visit
forms in first-match order, although their encoding array is sorted by form.
Score gates compare the float32 score promoted to float64 against the written
constant, preserving the specified boundary without changing a threshold.

## Internal interface

`Store::build(path, records)` ingests typed records and rebuilds derived state;
`add(record)` rejects an existing or tombstoned id, then rebuilds. `forget(id)`
and `rebuild()` follow LLP 0001's persistence and cache rules. `integrity()`
checks both FTS indexes. These methods return typed errors on operational failure.

`Store::agent(query, context)` returns the calls defined above. Context contains
caller `scope`, `cwd`, eligibility `now`, `session`, and `calls_left` (normally 3).
Only query and context enter retrieval: cases, gold labels, and alternative
wording lists belong to evaluation. A card contains record id, heading path,
byte span, section id, excerpt text, edge labels, and an optional fetch id;
the CLI obtains the human label from the record. Hint and related-pointer
fields are specified under Hints on silence and Related pointers. Vectors are
numeric selectors and never become excerpt text.

`Store::resolve(ref, caller_scope)` returns a record and byte span using LLP
0001's immutable-body addressing and shared-body preference. The CLI implements
`show` and outline presentation over those original bytes as specified in LLP
0004. Neither navigation nor display changes the indexed matching unit.

## Settled non-goals

No reciprocal rank fusion, reranker, or second embedding model. The confidence gate stays open. Empty-page attempts stop as soon as a card is kept. Filenames and query text are never written into record bodies. Approximate vector indexes and quantized weights are not used.

## Ordered occurrence support

POS-selected exact body-only ordered occurrence support contributes before
matcher record cuts, at fixed weight 0.75. There is no untyped product variant
or zero-weight mode. The historical untyped control was not selected.

For each adjacent content-term pair, take the closest nonoverlapping ordered
occurrences within one original paragraph or list unit. Phrases and identifiers
remain indivisible tokenizer sequences. Contribution is the smaller existing
lexical IDF divided by one plus the intervening token count, at most once per
pair. Add the weighted sum to BM25 without changing admission, hardness, tiers,
penalty, ties, card spans or assembly. Both matchers use only exact relations.

Direct calls to the pinned libSQL `unicode61 tokenchars '_-'` tokenizer use
document flags and retain every token in occurrence order, including repetitions,
without an additional lowercase pass. This is the exact stream formerly read
from connection-local FTS scratch tables, without their insert/read round trips.
The thread-owned adapter enters libSQL's process-wide initialization guard before
opening any raw SQLite handle; otherwise a tokenizer-first process can prevent
libSQL from configuring its threading mode. Source
relations persist under LLP 0001's POS publication contract. Structural units
use the original block cutter without packing; heading lines are isolated,
pipe-delimited cells are separate units, and fenced units are excluded. Parts,
separate heading columns, labels and paths never supply relation occurrences.

## POS relation selection

Ranking always includes POS-selected ordered occurrences at fixed weight 0.75.
The selector and its weight cannot be configured or disabled.
Both raw query and candidate original source units must
contain adjacent possible nominal/modifier-to-nominal occurrences within the
same NP chunk. Public analyses are alternatives, not a chosen statistical tag;
NN, NN:U, NN:UN, NNS, NNP, NNPS and JJ/JJR/JJS are mapped explicitly. Unknown
analyses supply no permission. Identifiers and camel-case interiors are excluded.
Protected atoms begin after the complete preceding whitespace scalar, including
multi-byte Unicode whitespace; a byte offset must never bisect that scalar.
The independent NLP byte stream is checked against SQLite unicode61 token
positions before any contribution. Baseline terms, hardness and admission stay
intact. The P1 score equation and downstream cuts remain unchanged.

The pinned linguistic asset is embedded at build time. Before loading it, test
adjacent nonempty content-term token arrays against
LLP 0001's indexed POS preflight. If its publication is complete/current and no
pair occurs anywhere in the stored source relations, return no scoring pairs:
even a positive query analysis could contribute no source bonus. Otherwise run
the original query tagging and scoring unchanged. Ineligible source rows may
cause conservative extra tagging, never a false absence. No relation, threshold,
score arithmetic, admission or card order changes.
If the query has no adjacent nonempty content-term token arrays, tagging can
also be skipped regardless of source-publication availability: no scoring pair
can be formed at all.

The focused runtime in `src/nlp` retains the pinned dictionary, rules and
statistical chunker. The build verifies the original resource digest and
converts it to a versioned embedded asset: dictionary FST, stable word ids,
flat original spellings and ordered tag arrays. Runtime loading does not expand
the dictionary into bidirectional hash maps. Source tag ordering, casing and
unknown-word handling remain unchanged. No runtime resource file is needed.
Serialize lookup collections in canonical order so repeated builds produce the
same asset bytes. Exact word-membership sets use sorted unique word-id arrays
and binary search rather than rebuilding hash sets on each load. Token byte-to-character coordinates advance through each
source sentence once rather than recounting the prefix for every token.

The embedded asset also contains the statistical models' immutable feature
lookup slots and already-expanded float32 parameters. Construct linear-probe
slots from the original hashed feature keys at build time, with at most half
the slots occupied; preserve duplicate-key replacement, zero-length features
and original parameter offsets. Runtime loading does not rebuild these maps
or expand bf16 parameters. Evaluation visits context features, outcomes and
parameters in their original order, preserving float32 additions and softmax.
Outcome ids retain their original byte representation; widen them only for
array indexing rather than expanding the whole outcome array during loading.
Serialize feature slots, outcome ids and float32 parameters as packed byte
arrays. Slots and parameters use explicit little-endian fields without struct
padding; loading decodes the same integer values and float32 bits in bulk,
including signed zero and NaN payloads. Reject incomplete packed elements.
The original wire layout remains a build-time compatibility decoder; changing
the compiled asset layout changes its format marker, not annotation semantics.

Disambiguation visits rules in order and applies the
first rule producing changes before continuing after it; do not speculatively
evaluate later rules in parallel. Rule-group token lookup binary-searches the
ordered source spans, then applies the original containment and nonzero-width
predicate within that slice. Exclusion-pattern ranges are computed lazily once
per rule and immutable sentence; sorted starts and prefix maximum ends preserve
the inclusive overlap test. Once a nonempty match is accepted, positions already
rejected or masked cannot become available again, so continue after that match's
start position. Zero-width matches retain the original one-position cursor
advance, including repeated matches. Preserve chunker float32 operation order,
beam ordering, and tie behavior. These are implementation changes, not different
linguistic rules. Differential validation against the unchanged eval-only engine
is provided by `evals/examples/pos_engine_parity.rs`.

Query work is transient; source relations are prepared on writes and read by candidate block,
guarded by the LLP 0001 publication contract. Normal open does no corpus tagging.
Shadow factored assignments in `evals/examples/pos_assignment.rs` never grant
semantic admission. Nonoverlapping partitions and source-sentence capacities
are external constraints; cosine is not a role or entailment certificate.

### Selection before feature reads

Corpus statistics remain global and clock-qualified, but requesting them does not
hydrate term features. Count matching blocks directly through FTS MATCH when a
normalized term tokenizes to exactly itself; quote it as a literal and apply the
same lifecycle/expiry projection. This avoids enumerating repeated occurrences
just to deduplicate block ids. If tokenization changes the term, retain literal
vocabulary equality so normalization cannot broaden the count. Retain complete
eligible term-hit sets and their posting
arrival order before admission. Read frequencies only for rows that can enter the
raw ranking, the one-under addition, or a qualifying tree's chosen block. Apply
hard-term requirements before these reads. A positive-coverage tree winner retains
its POS bonus even if it fails raw admission; a zero-coverage winner retains its
lexical-only score. These are read boundaries, not new retrieval cutoffs.

Keep counts separate from positions. POS body-presence checks use a positive exact
body frequency. Fetch exact-body and stem position arrays only for the first five
distinct-record raw ranks used by diagnostic confidence. Cache completion by
normalized term set, tokenizer, feature kind and block id, including absent
occurrences. Share a scratch candidate-id table across the term set. The vocabulary
virtual table seeks by term; its candidate-id predicate reduces aggregation and
materialization, but does not provide a document seek within the posting list. Loading
another subset or upgrading counts to positions must neither omit a previously
unseen row nor double a frequency. Retain unchanged scoring arithmetic.

For ordinary unfiltered queries, select qualifying heading trees and their winning
block in SQL after lexical hit collection. Tree coverage counts distinct normalized
query words, includes every hard word, and uses the existing need. Choose the
member with greatest query-word coverage, then smallest block id. Keep kind/tree
order for later per-record ties. Only winners need Rust metadata; query-local trees
must not masquerade as complete navigation caches. Explicit query filters retain
the full filter-aware member selection. Record fills may likewise select their
record and block in SQL: coverage descending, record id descending, then block
coverage descending and block id ascending. Preserve the zero-overlap last-record
case. Linked-call candidates are restricted to linked records before hydration.

For POS bonuses, a current complete pair publication permits indexed membership
queries for each ordered query pair and candidate block. Validate the source
annotation's body hash, generation and resource identity in the same read. Add
weights in the original query-pair order, including duplicate pairs. If publication
is unavailable, retain the stored-annotation path and its stale-source errors.
Diagnostic traces may fetch original witnesses and must verify their score agrees
bit-for-bit with indexed membership; tracing never changes a score.

Eligibility can precede candidate vector and section-mean reads where it already
precedes their comparison. Keep surviving traversal, all matrix populations,
float32 operations, gates and tie rules unchanged. These read optimizations do
not narrow global nearest-vector, beam, form or sentence matrices.


### Lifecycle correction

All retrieval entry points use one caller clock for a request. The default is current UTC
at whole-second precision; evaluation supplies its frozen clock explicitly. Cursors retain
their original clock. Eligibility removes expired, closed, superseded and non-accepted
records from fills, hints, pointers and filename candidates as well as lexical matches.
For basename uniqueness count eligible records, so an expired namesake cannot hide a
unique eligible record. Numerical scoring, tier order, thresholds and scope penalties
are unchanged. The operator authorized this correction even where it changes a page.
