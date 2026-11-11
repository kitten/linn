# LLP 0005: Retrieval evaluation

**Type:** Explainer\
**Status:** Active\
**Systems:** Evaluation\
**Related:** LLP 0001, LLP 0002, LLP 0004

The current accepted retrieval page is recorded under Lifecycle correctness
measurement below. Earlier floors and comparisons are historical measurements,
not competing current defaults.

## Corpus and execution

The Rust `linn-eval` workspace package builds one store and scores many cases.
Records and cases are JSONL objects. Records use LLP 0001's fields. Cases have
`id`, `scope`, `query`, optional `alt_queries`, `category` (string array), `gold`
(object array with `record`, optional `equivalent`, `quote`, `heading`), and
`must_not` (record-id array). Case-level equivalent lists are ignored.

`capture <source-directory> <name>` copies records.jsonl and cases.jsonl into
ignored evals/corpus/<name>. Names contain only ASCII alphanumerics, hyphens and
underscores. Refuse tracked destinations, unverifiable Git state, existing files,
and missing source files before copying. Neither corpus nor output is committed.
`--corpus` or LINN_EVAL_CORPUS overrides the default evals/corpus/q4. Use
`--corpus evals/corpus/h2 --mode all` for holdout. No corpus is required by tests.
Shared heading-tree membership is the product default; eval sets no special
membership environment variable. Case metadata never enters the product store.

## Scoring

Eligibility clock is `2026-11-05T00:00:00Z`. Each wording uses case scope, cwd
empty, that clock, session equal to case id, and calls_left 3. Wordings are query
followed by alt_queries, at most three. The store never reads alt_queries or
knows whether a wording is original. SHA-256 of UTF-8 case id, interpreted as
an integer, modulo 10 determines split: residues below 3 are holdout, otherwise
dev. Mode dev selects dev; mode all scores every case.

Gold ids are each gold object's record plus that object's equivalent list.
Gold and cross-scope inspect the first five cards of the first call for each
wording. Any means success on any wording; first means the original wording.
Cross-scope cases have category cross-scope and at least one must_not whose
original nonempty scope differs case-insensitively from case scope. These ids
are cross-scope targets. SQL scope remains null; original scope separately
supplies the penalty of seven. Gold never resides in another repository in the
measured corpus.

Empty and section inspect every card from every call of every wording.
For no-gold cases let me be lowercased case scope, targets be must_not ids whose
original scope is nonempty and differs from me, forbidden be all must_not, and
mn mean some shown forbidden id has missing/empty original scope or equals me.
If targets is nonempty, empty-correct requires a shown target and no mn;
otherwise it requires no cards at all and no mn. Any card on true silence fails.

Section applies to doc-passage cases with gold. Every gold object needs a shown
card of that object's record containing a gold span. A quote supplies all literal
occurrences, restricted to the heading section if also specified. A heading
matches stripped heading case-insensitively, else a case-insensitive suffix of
the heading path joined by ` > `. Without quote, the matched heading section is
the gold span; without either, the whole body is the span. Unknown heading with
no quote fails. Card span must contain an entire gold span and be no longer
than max(3 * gold length, 1500) bytes. Gold length is the matched heading section
length when present, otherwise the longest quote occurrence (or whole body).

Print exactly two score lines:

```
any A/N first B/N xs C/M D/M
empty E/F section G/H
```

## Measured page

These are measurements, never targets for threshold retuning:

| Metric | Dev | Holdout |
|---|---|---|
| Gold any / first | 410/439, 350/439 | 341/354, 294/354 |
| Cross-scope any / first | 30/42, 24/42 | 57/62, 50/62 |
| No-gold | 35/51 | 56/106 |
| Section | 55/73 | 237/271 |

Validate dev before freezing retrieval and scoring holdout. Counts alone are
insufficient: `--pages` exports case, wording index and ordered record ids;
`--compare` and `--compare-pages` compare exports, including repeated ids.
`--responses <file>` exports complete calls, query, case id and wording index
as JSONL during the ordinary scorer, without enabling verbose rank traces.
Compare those exports for exact cards, spans, text, hints and related pointers;
first-page ids alone do not cover those fields. Response serialization happens
after the agent-invocation timer stops and never runs another query or changes
scoring. This export conflicts with neutral/document scoring, direct-query,
offline page comparison and vector-maintenance modes.
The validated page has 1,470 dev and 1,380 holdout wording pages, with no ordered
id differences. Dev has 6,096 block vectors, holdout 6,477; their float32 matrices
were bit-identical on the validated native platform. This does not establish
agent answer quality. In particular, case s7-c-0100's original wording returns
five cards; its alternate wordings are empty. Do not special-case it.

## Retrieval change adoption

Select on the dev split. A retrieval change qualifies only when at least one of
gold-any, gold-first, cross-scope-any, and cross-scope-first rises, none of those
four fall, and neither no-gold nor section falls. A rise in section alone does
not adopt a change.

Confirm the frozen change once on h2 with `--mode all`, under the same no-fall
and retrieval-rise requirements. If h2 is flat and nothing fell, the change is
a hold, not an adoption. If h2 falls, reject the change and do not retune it.

### Explicit acceptance of the source-evidence and BM25L combination

The explicitly accepted frozen combination uses source-witnessed nominal
compounds, literal colon recovery, unique qualified-identifier suffix
resolution and BM25L delta 0.125, on the already selected SSE v2 encoder and
technical file addressing. Its algorithm is fully specified in LLP 0002.
This supersedes its earlier rejection under the ordinary no-fall rule. It is
an explicit acceptance of measured tradeoffs, not evidence that the rule passed,
not a threshold retuning, and not a general relaxation for future changes.

| Metric | Current dev | Current h2-all | Scale dev | Scale h2-all |
|---|---|---|---|---|
| Gold any / first | 412/439, 354/439 | 341/354, 298/354 | 388/439, 316/439 | 325/354, 275/354 |
| Cross-scope any / first | 31/42, 25/42 | 56/62, 49/62 | 28/42, 22/42 | 48/62, 41/62 |
| No-gold | 35/51 | 55/106 | 31/51 | 47/106 |
| Section | 55/73 | 237/271 | 51/73 | 225/271 |

Against the prior SSE/file-addressing page, dev gains one gold-any and three
gold-first; h2 gains one gold-first but loses one XS-any, one XS-first, one
no-gold and two section cases. Scale dev gains two gold-any and four gold-first;
scale h2 loses two gold-any, one gold-first and one section, while XS-first rises
one. Dev changes 291/1470 ordered wording pages, h2 297/1380. Historical floor
and earlier selected-page tables remain provenance, not current targets.
H2 has been reused and is not independent validation. Synthetic six-document
metrics remain 31/49 Hit@1, 35/49 Hit@3 and Hit@5, MRR@5 0.6735. The recorded external baseline
still leads gold-any in all three matched slices. Full q4, h2 and scale-q4
neutral tables completed with failing gates; scale-h2 neutral was canceled.
No full current neutral pass or external-baseline gap closure is claimed.

### Explicitly authorized POS recovery study

After the first POS confirmation, the candidate was applied and adjusted to
recover the cross-scope losses. For this study, q4-dev and
h2-all are development measurements. Keep their cases, scoring and no-regression
comparisons unchanged, record every configuration, and use generic mechanisms
rather than case-specific rules. A successful adjustment on these reused cases
is not independent holdout confirmation. This exception does not reopen other
closed experiments or change the usual adoption protocol.

The applied adjustment uses POS-selected nominal pairs with bonus weight 0.75.
Recovery weights 0.5, 0.75 and 0.25 were measured on both corpora; all recover the
original cross-scope counts and retain the dev retrieval gains. Weight 0.75 also
retains the larger section numerator. The POS-only development measurements were
q4-dev: any 411/439, first 351/439, cross-scope 30/42 and 24/42, empty 35/51,
section 55/73; h2-all: any 341/354, first 295/354, cross-scope 57/62 and 50/62,
empty 56/106, section 239/271. Both corpora participated in this selection;
these are not fresh holdout results. The earlier floor table records the legacy
baseline from the earlier implementation. Reproduce that historical measurement
with its recorded binary or revision, not a runtime mode in the current product.

The neutral gate table and the hybrid comparison are separate measurements.
A failing gate or a hybrid lead is not permission to change the ranker. Do not
add a second embedding model, a query-expansion model, or a reranker to chase
either measurement.

## Selected static encoder and file addressing

The subsequent fixed SSE v2 encoder plus technical file-stem addressing in
LLP 0002 was selected jointly. The encoder alone lost one dev gold-first case;
file addressing alone gained one but was flat on h2. Neither alone was adopted.
The declared combination changed no thresholds and used no case-dependent routing.
Its dev page was replayed byte-identically before the one-shot h2 run.

| Metric | Selected dev | Selected h2-all |
|---|---|---|
| Gold any / first | 411/439, 351/439 | 341/354, 297/354 |
| Cross-scope any / first | 31/42, 25/42 | 57/62, 50/62 |
| No-gold | 35/51 | 56/106 |
| Section | 55/73 | 239/271 |

Compared with the POS-only page, 82 of 1,470 dev wording pages and 96 of 1,380
h2 wording pages change. Aggregate no-fall does not mean every case improves:
dev gold-first gains s1b-c-0050 and s6-c-0073, loses s4-c-0020 and s6-c-0035;
cross-scope any and first gain s5-c-0081. H2 gold-first gains h2a1-c-0065,
h2a3-c-0046 and s7-c-0029, loses s7-c-0089. No other case-level gold/cross-scope metrics
change. First-page exports do not establish per-case empty/section parity; the
full scorer establishes their unchanged totals. Historical reuse of h2 limits its independence, even though this frozen
combination was confirmed once without h2 tuning. The synthetic six-document
benchmark remains Hit@1 31/49, Hit@3 and Hit@5 35/49, MRR@5 0.6735.
This selection does not claim to close every matched hybrid gap or pass the
neutral table; those remain separate measurements.

## Scale corpus

On 2026-11-05 the curated records were kept and 15,530 distractor documents were
added. 14,925 are Markdown files from dependency trees (READMEs and docs;
changelogs, licenses, and files over 200 KB skipped; at most 25 files per package
per repository; identical bytes kept once per repository), scoped to the
repository that contains them. 605 are pages split from local llms.txt files.
No cases were added. A distractor that happens to answer a task counts as a miss.

Capture scale-q4 and scale-h2 with the same capture command as curated corpora.
Scale-q4 has 16,906 records; scale-h2 has 17,003. Their cases are byte-identical
to q4 and h2 respectively. These are full regular-file copies, not symlinks.
After capture, full runs read only records.jsonl and cases.jsonl from the chosen
local evals/corpus directory. Body bytes inside JSONL supply ingest and scoring;
source_path is metadata, never a file to open. Extra author-supplied terms and
section_terms fields remain unread. There is no author-terms index or enrichment
step. Scale uses the same product ingestion and search, not a different ranker.

## Neutral fixed loop

`linn-eval --neutral` prints a gate table, separate from the two-line retrieval
measurement. It is a deterministic prepared-query loop with neutral grading,
not a model agent, judged agent or CLI agent. It has no votes, generated queries,
progressive disclosure, disclosure token budget, or early exit after success.
For each case run query then the first two alt_queries, even if earlier queries
returned cards, each with calls_left 3, the fixed clock and case session and
scope. The case cwd is supplied as metadata; G3 varies it separately. The product
receives query and context only, never alternatives or gold metadata. Evaluate
all returned calls; found and Q1 consider the first five cards of every call.
Hints and related pointers are never answers.

Neutral gold equivalences use the case-level equivalent map keyed by each gold
record (this differs from the retrieval measurement's object-level list).
Found means any gold or such equivalent appears among those cards. Q4 requires
every gold object to have a satisfying card, allowing its equivalents. Use the
ordinary quote/heading target and byte containment bound above. Record-level
gold needs only the record id. For an equivalent, an equivalent_targets span
needs at least half-span overlap and a card no longer than max(3 * span length,
1500); a judged heading supplies that heading's target. Otherwise use the gold's
quote/heading in the equivalent; if absent and the equivalent is at most 2400
bytes, record-level credit applies. A long equivalent without a target has no
section credit. Explicit alt_sections of a gold record may satisfy Q4 with at
least half-span overlap and a card no longer than max(3 * gold length, 1500,
judged alternate length). Unknown or invalid spans cannot satisfy a target.

G1 recomputes accepted state, close envelopes, accepted close targets, accepted
child supersessions, text expiry at the fixed clock, and stripped nonempty body.
Scope never makes a record ineligible. Check every card, up to three hints and
two related pointers per call, and unknown record ids. For text leaks inspect
the three longest distinct stripped body lines of length at least 30 characters
of ineligible records; close envelopes and empty bodies are omitted from leak
scans. A matching line is a leak unless an eligible record also contains it.
G1 is the number of cases with any such violation; it must be zero.

G2 is the fraction of cases showing any must_not whose original scope is empty
or case-insensitively equals the caller's scope, inspecting all cards. A record
in another repository is a result, not a violation. Q1 applies only to no-gold
cases: if any must_not has nonempty original scope differing from the caller,
a shown target and no same-repository forbidden card is required. Otherwise no
cards may be shown. Silence on an other-repository target fails.

G3 compares all original-query card ids in order for cwd empty, case cwd, and
zz-unrelated/sub/dir; cases with path, under or cites filters are excluded.
Required invariance is 1.0. G4 requires clean integrity before evaluation and
after forget, plus a working forget. Select the first five eligible records in
case scopes with a unique alphabetic body word at least five characters long;
use each record's lexicographically first such word. Verify it is found before
forget, absent afterward, and none of its distinctive lines leaks afterward.
If no eligible probe candidate exists, the forget check fails. Forget runs only on an independently
rebuilt scratch store, never the measured or cached input store. G5 builds a
second store from the original JSONL records, runs original queries, compares
ordered first-call top-five ids and first-call hint (record, span) pairs with
the original store. Required determinism is 1.0.

R1 uses body tokens only, lowercase Unicode word characters plus hyphen,
permitting leading hyphens and dropping tokens with no letter or digit. For each
record choose the first section with the most distinct non-stop tokens whose
hyphen/underscore-stripped length is at least four and which are not numeric.
Compute document frequency over whole bodies. Sort the chosen words by ascending
frequency then word, take three, quote words starting with a hyphen, and skip
records with fewer than two words. For each case scope in lexical order (null
first), shuffle eligible records in input order with integer-seeded MT19937
seed 7 and rejection-based getrandbits Fisher-Yates; take at most 120. Self-hit
means its id occurs in the first call's top five. R1 is the share of self-hits.

R2 groups case loops by caller scope, only groups of at least 20 loops. Sum
characters per record across the first five cards of each call, plus excerpt
and heading characters for the first three hints and two related pointers.
Per loop weight is min(1, characters / 1500). Exclude gold, equivalents and
acceptable records for that case. For each scope sum weights per remaining
record and divide by number of loops; R2 is the largest share over all scopes.
It is not the fraction of slots occupied or the number of rejected answers.

Tokens per call are ceil(number of Unicode characters in call.text / 4), exactly
the character-count approximation used for the measured protocol. Latency times
each search invocation and divides elapsed milliseconds equally over its returned
calls; zero-call invocations contribute no samples. Include ordinary loops,
rebuild, location, self-retrieval, alternate-query, exposure diagnostic and forget
searches, excluding ingest and grading work. Median averages the two middle
samples; p95 uses the nearest index to 0.95 * (n-1), ties to even. Rates for G2
and Q gates and the maximum exposure share are rounded to three decimals before
threshold comparison; latency p95 is rounded to two decimals before comparison. An absent Q or R1 population reports NaN, never success.
A gate with an explicit sample size under 10 reports INSUFFICIENT-N, not pass.

| Gate | Requirement |
|---|---|
| G1 | zero violating cases |
| G2 | must-not rate <= 0.02 |
| G3 | location invariance = 1.0 |
| G4 | clean integrity and working forget |
| G5 | determinism after rebuild = 1.0 |
| Q1 | no-gold empty-correct >= 0.90 |
| Q2 | identifier/phrase/doc-passage found >= 0.85 |
| Q3 | found excluding paraphrase and lateral >= 0.75 |
| Q4 | doc-passage section hit >= 0.75 |
| Q5 | lateral found >= 0.60 |
| Q6 | paraphrase found >= 0.35 |
| C1 | tokens/call median <= 600, p95 <= 1500 |
| C2 | latency/call p95 <= 50 ms |
| R1 | self-retrieval at five >= 0.90 |
| R2 | maximum character-weighted exposure share <= 0.10 |

These thresholds are fixed. A failing gate is a measurement, not permission to
retune retrieval. The command prints every gate and exits successfully when the
measurement completes, even when gates fail. Operational errors fail the command.

## Validation boundary and running checks

The two-line retrieval measurement is validated at the recorded floors above.
The neutral grader is runnable and has corpus-free tests for cross-scope Q1,
same-scope G2, ineligible text, ignored author fields, and command execution.
Those tests do not establish full neutral validation: that requires retained full
q4-dev, h2-all, scale-q4-dev, and scale-h2-all gate tables for the implementation
being evaluated. Historical tables describe the version tested, not every later
change. Do not infer complete validation from fixtures or an empty selection.

The corpus default and `--case` filter both apply. A missing id selects zero
cases; that is not a full measurement. In particular s7-c-0060 is in h2, not q4,
and its hash residue 4 selects dev when h2 is supplied. R1 and G4 still use scopes
from all loaded cases, captured before selection. G4 checks record disappearance
and distinctive-line leakage on the scratch store, not the probe word's residual
postings count. Its printed boolean combines integrity and probe success and
does not diagnose which component failed. C2 includes diagnostic searches; a
latency failure is not a retrieval-floor failure.

Quote targets enumerate overlapping occurrences in valid UTF-8. Advancing past
the first Unicode scalar of a match cannot skip another valid occurrence inside
that scalar; a continuation byte cannot begin a valid UTF-8 quote. This is true
for non-ASCII bodies too. The captured corpora are not restricted to ASCII.

Only reuse `--store` with the corpus from which that cache was built. Omit it for
an independent fresh ingest. Export comparisons must cover exactly the same
case/wording keys, reject duplicate keys, and compare ordered record-id arrays
including repeated ids; missing keys must not silently count as agreement.

## Saved-page and historical hybrid comparisons

The saved-page helper `evals/examples/page_changes.rs` reports ordered-page and
paired gold/cross-scope changes without searching. It requires matching export
keys and does not infer all-call empty or section outcomes from first-call ids.

`evals/examples/hybrid_compare.rs` compares current Linn with archived hybrid
predictions on matched 1,243- and 90-record indexes. Verify archived bodies
against captured JSONL, exact prepared query strings, page populations and the
historical gold counts before accepting the comparison. Build fresh stores from
the captured bodies and preserve original corpus scopes for forbidden records,
including those excluded from the matched index. Use the scoring rules above;
MRR@5 averages reciprocal original-query gold rank across gold cases. Archived
hybrid pages supply no section spans or later calls, so no section or neutral
comparison follows. This helper does not execute the hybrid stack. Its
corpus-free scorer tests are part of the workspace suite. See `evals/README.md`
for invocation and archive layout.

## Lookup optimization diagnostic

`LINN_STARTUP_TIMINGS=1` emits JSON phase timings on stderr for database setup,
record decoding, validity, block parsing/tokenization, persisted row-id mapping,
vectors, distinct FTS vocabulary, column lengths, surface DF, derived postings,
and links/trees. This is diagnostic instrumentation, not a retrieval switch.
The Rust `startup_check` example opens an existing store without loading corpus
JSONL; an optional query measures the first agent invocation separately.
Its `--raw` mode measures libSQL preparation, row stepping and value extraction
without building Rust search structures. Optional `--cache-kib` and
`--mmap-mib` settings affect only that raw diagnostic connection. Report actual
PRAGMA values and repeated samples; warm OS pages and concurrent jobs affect
timing. Do not weaken secure deletion or durability to improve a benchmark.
Its `--probe-materialized-views` mode ignores the supplied path and probes only
a fresh in-memory database. It reports the actual pinned engine version and
whether materialized-view syntax works; it never modifies a product store.

Separate one-time structural projection publication from normal repeated opens.
Normal opens load metadata and indexed words on demand. Deferred vector and
vocabulary reads belong to the first query that needs them, so report full CLI
wall time and first-use query time in addition to the open timer. Native query
plans and lifecycle tests should establish indexed reads, lazy construction,
rollback, external-write invalidation and secure deletion of projection content.

The `cli_latency` Rust example runs each selected original wording in a fresh
product CLI process. Select the first/last evenly spaced dev originals per
identifier, phrase, doc-passage, lateral, paraphrase and no-gold group, deduplicate
case ids, and optionally add explicit dev case ids. No alternative wording is
passed implicitly. `--only-cases` restricts the run to explicit dev ids.
Time process creation through exit, including output capture
and teardown; use the CLI's real wall clock and caller scope. This is a latency
diagnostic, not gold scoring or neutral C2. Exit 0 and honest-miss exit 1 are
valid invocations. Save stdout/stderr and JSONL timings under ignored output.
Optional before/after CLI comparisons require equal exit codes and complete
semantic JSON after removing the clock-bearing cursor. Report sample size,
selection, individual times, median, nearest-index ties-even p95 and maximum;
do not claim a small fixed sample is a production latency distribution.

`startup_check --artifact-stats` reports numerical projection row, chunk and byte
counts without loading vector payloads. Record disk growth and one-time numerical
preparation cost alongside query gains. Corpus-independent static model loading
and exact dense vector walks remain first-use/query costs, not omitted warmup.

`--timings <file>` on the ordinary two-line scorer writes a JSONL store-ready
event (store-open/build milliseconds, record count and block count), then case,
wording, agent-invocation milliseconds and returned-call count for each wording.
Timing begins immediately before `agent` and stops before grading and export.
Corpus JSONL loading is outside the store timer. No additional query is run;
the two score lines and page comparisons are unchanged. Stderr summarizes median,
p95 using nearest-index ties-to-even, and maximum invocation time. Include first
use of lazy caches; do not hide it as an unmeasured warmup. These are per-query
diagnostics, distinct from neutral C2's time divided over returned calls. Report
store opening separately and do not claim corpus-independent cost for exact
global vector work. It conflicts with neutral, document, query and offline
comparison/vector-maintenance modes.

The corpus-free tests also cover content-derived lookup invalidation and
persisted row-id gaps. The optional Rust `lookup_check` example compares SQL
and memory record lookups in order, times 100 passes over the first at most
128 record ids, and compares full calls from two saved eval executables on up to
24 evenly spaced original dev queries in corpus order. It saves both call
exports and fails on inequality. These diagnostics do not replace full-corpus
page and score comparisons. Its command wall times include startup and store
open and are not neutral latency-gate samples. Use a copied store, since open
may install the derived record index. See evals/README.md for invocation.

## External document benchmarks

`--document-benchmark` uses the same static records.jsonl and cases.jsonl format
and the same product store, with document-level relevance instead of section
judgments. Every selected case must have gold objects, and every gold or
object-level equivalent id must exist in the records. Empty selections and
missing targets are errors. It supplies only the original query and ordinary
context (fixed clock, empty cwd, case scope/session, calls_left 3) to the store;
no alternatives, intent, explanatory notes or relevance labels enter retrieval.

Score the first five cards of the first call only. Hit@1, Hit@3 and Hit@5 are
shares of queries with a gold document within that many card positions. MRR@5
averages reciprocal rank of the first gold within five, with zero for a miss.
Repeated cards still occupy their actual positions; do not deduplicate or
reorder the product page to improve a result. Optional benchmark_group metadata
partitions the report and, for names with a slash, also reports their suite
prefix. It never affects searching. `--pages` writes the usual case/wording/id
exports (wording 0). Existing split and case selection flags still apply; use
`--mode all` to evaluate a whole external fixture. The command prints measurements
without thresholds, not a new adoption gate or a replacement for the curated
retrieval and neutral measurements.

The synthetic six-document retrieval fixture is a converted published external
retrieval fixture with six synthetic documents and 49 questions:
24 standard questions (also used by its BM25 tests) and 25 deep-research
questions. Its older CLI harness repeats 18 of the standard questions; those
are not additional cases. The deep-research set was explicitly designed around
semantic expansion and reranking, and the standard set includes fusion-focused
questions. It is external to Linn but is not evidence of an unbiased benchmark,
a sealed holdout, or exhaustive relevance judgments. It has no no-gold, lifecycle,
repository-scope, or gold-section cases and cannot validate those behaviors.

Conversion is a one-time preparation step, not an evaluation-time dependency or
a committed converter. Preserve original body bytes, question strings, and all
upstream document-relevance cases. Resolve each upstream filename-substring
answer to exactly one source document; fail on ambiguous or missing mappings.
Labels come from the original heading, never manufactured retrieval hints.
Keep intent, notes, expansion outputs, and author terms out of the converted
retrieval inputs. Provenance records the upstream revision, source hashes,
conversion choices and limitations alongside the ignored static data; include
its license. No raw copies, intermediates, corpus data or results are committed.
Once converted, evaluation needs only this repository, the prepared JSONL, and
the usual build-time model input. See [evals/README.md](../evals/README.md) for
operation; no runtime external system checkout or model stack is required.

## Experimental occurrence audit

The eval-only `pos_groups` example executes nlprule 0.6.4 over raw input bytes,
retaining all public analyses, chunk labels and checked byte spans. It emits
linear singleton/adjacent alternatives and complete begin/end chunks, deduplicated
by original interval with alternative labels retained, without claiming
dependency heads. Whole-NP identity retains modifiers; ordered exact group
witnesses consume separate source occurrences. Neither interface certifies
entailment, record eligibility, section membership or a legal card schedule.
They are shadow research interfaces and do not change product retrieval.

The example accepts a caller-supplied tokenizer asset, JSONL `text` and optional
`source`; it never reads relevance labels. Optional embedding matrices encode
original group slices through the existing Encoder and compare with its existing
float32 dot kernel. These matrices propose comparisons, never synonym licenses.
The standalone example still accepts an explicit resource path. The always-enabled
product relation selector embeds the pinned linguistic asset as described in
LLP 0004 and publishes selected source metadata under LLP 0001. Its ranking
contract is in LLP 0002. The audit and bounded `pos_assignment` executor are
shadow diagnostics, not alternative graders or semantic admission authorities.
The assignment controls retain original occurrence identity, source-unit/sentence
bounds, nonoverlapping alternate partitions, model identity, selected assignments
and unmatched original material. Cosine completions remain unverified. Existing
DEV selection and one-shot parent holdout rules are unchanged.


## Lifecycle correctness measurement

Lifecycle eligibility was enforced throughout retrieval, independently
of ranker selection. At the frozen evaluation clock the corrected q4 dev page measures
gold 412/439 any and 354/439 first, cross-scope 31/42 and 25/42, no-gold 35/51,
section 55/73. All six counts remain unchanged, although 30 wording responses change.

The corrected h2 all page measures gold 341/354 and 298/354, cross-scope 55/62
and 48/62, no-gold 54/106, section 237/271. Relative to the preceding accepted page,
cross-scope-any, cross-scope-first and no-gold each fall by one. The case
`h2a1-c-0102` had credited the closed and superseded `s2b-r-0032`; lifecycle
enforcement removes it. This is a disclosed correctness correction, not a successful
retrieval experiment or a reason to retune. There are 28 changed h2 wording responses.
Across all returned cards, lifecycle-ineligible occurrences fall from 32 to zero on q4
and from 28 to zero on h2. Historical measurements above remain historical.

The full q4 dev neutral loop reports zero G1 eligibility violations (previously 26).
Its G4, Q1 and R2 gates still fail; lifecycle correctness does not establish that every
neutral gate passes.
