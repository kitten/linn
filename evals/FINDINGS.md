# Closed findings and measurements

This is a historical closed-measurement log, not the specification. The LLPs
and AGENTS.md govern current work. Historical experiment numbers identify
measurements, not instructions to restore a design. A not-adopted result is
closed and must not be rerun as if it were new. Proposals and untested ideas
below retain their recorded status; their presence is not authorization to
implement them. Earlier decisions are retained beside their supersessions. Numerical claims retain their recorded context and values; historical reports are
not new Rust validation results. Citations for retained papers and dependencies
remain in place; anonymized external-system comparisons keep their recorded outcomes.

## Current supersessions and scored behavior

- **Author fields — superseded and closed.** The 2026-11-05 entry called author
  terms and section terms open. They were subsequently measured and closed.
  Writers supply original text only. `terms` and `section_terms` stay unread;
  neither an author-terms index nor manufactured labels or keyword hints may be
  added to improve the eval. Earlier results below remain historical evidence.
- **Scope — superseded.** Earlier decisions restricted default search to the
  caller and later said `scope:` narrowed shared search to one repository.
  The current store keeps indexed scope null: `scope:` matches every row.
  Original scope supplies only the within-tier penalty of 7; shared heading-tree
  membership is unconditional.
- **Acceptance — superseded.** Earlier write-bar and separate-actor acceptance
  proposals are historical. New CLI records are searchable immediately; stored
  candidate records still fail word-search eligibility. Corrections use a child
  or close envelope. The accepted SSE v2 encoder and its fixed arithmetic
  supersede earlier model measurements and are specified in LLP 0002.
- **Silence and hints — superseded in stages.** The early no-fallback rule was
  followed by near-miss hints, then the ordered, gated empty-page attempts.
  Only the current LLP algorithm governs; the unsuccessful relaxations below
  remain closed. Historical agent runs, CLI-agent runs, neutral fixed loops,
  and prepared-query retrieval counts are different measurements.
- **s7-c-0060, scored behavior.** This h2 case has no gold and names
  `s7-r-0069` as must_not. All three prepared wordings stay empty in the current
  page. The rejected releases below show why dropping the identifier was unsafe.
- **s7-c-0100, scored behavior.** Its two `blob-util` wordings stay empty.
  The original wording, `resume download range request`, returns five cards in
  the matched scored store. Earlier statements that the whole case must remain
  silent are not a description of that scored page. No case-specific suppression
  was added.
- **Current page and hybrid.** The current page does not pass the external hybrid comparison. The slice
  rates and joint model-stack cost below remain a separate comparison, not
  permission to add those models or retune the ranker. Historical "beats" and
  "selected" labels below describe their original runs; new adoption decisions
  follow LLP 0005, including a hold when h2 is flat.

## Retained research and measurement entries

The following public studies and implementation references informed background
research. Their measurements describe their own systems and collections; they
are not Linn results.

- A published scaling study ([arXiv:2607.26497](https://arxiv.org/html/2607.26497))
  held questions fixed while growing a synthetic enterprise corpus. At 1,144
  documents, file-walking scored 77.4 and BM25 74.7, with overlapping intervals.
  At about 10 million corpus tokens BM25 led; at 511,959 documents, BM25 scored
  50.5, file-walking 30.7, and dense retrieval 29.9. Graph-based systems stopped
  earlier because graph construction cost more than the corpus. A separate
  matched rejudge of 150 questions scored an agent with BM25 at 69.4 versus 36.9
  without an index. These tables use different comparisons and do not form one
  performance ladder.
- Public [blob-transfer](https://docs.iroh.computer/protocols/blobs) and
  [document-reconciliation](https://docs.iroh.computer/protocols/documents)
  protocols illustrate separating immutable body transfer from record-set
  reconciliation. The cited blob protocol verifies streamed bytes against a
  content hash; the document protocol reconciles entry metadata while leaving
  body transfer to the blob layer. The referenced release documentation warned
  that the blob library was not yet production quality.
- An alternative SQLite-compatible engine's [FTS documentation](https://docs.turso.tech/sql-reference/functions/fts)
  describes a Tantivy-backed engine in the same database file. It differs from
  the SQLite FTS5 interface and its snippet/tokenizer behavior; it is background
  context, not a proposed dependency.
- [cr-sqlite](https://github.com/vlcn-io/cr-sqlite) replicates SQLite state,
  including virtual-table shadow rows. SQLite's session extension does not
  capture virtual-table changes, so replication is not a shortcut for shipping
  an FTS index. This is a replication reference, not an adopted dependency.
- Public [SQLite and FTS discussions](https://lobste.rs/s/yev7oa/using_sqlite3_as_notekeeping_document)
  describe choosing SQLite over a plain-file wiki when history, search, and
  edge cases made a bespoke file system costly. A separate
  [FTS5 discussion](https://lobste.rs/s/b3d1ba/structure_fts5_index_sqlite)
  records failures caused by passing raw questions to `MATCH`; quoting each
  term was the reported fix. These are anecdotes, not benchmarks.

## Additional retained background (historical, superseded where noted)

- **Earlier Linn retrieval was rejected.** The predecessor fused a phrase-only
  lexical query with vectors using reciprocal-rank fusion at k=60 and equal
  weights. Event tiers later used weights 4, 3, 2 and 1 against vector weight 1;
  lexical first-place results had appeared at hybrid ranks 7, 13 and 15.
  Heading text was omitted from embedding input while FTS weighted it threefold.
  A four-document/one-session top-three check passed both EmbeddingGemma and
  bge-small and did not detect the ranking problem. CPU/Metal profile changes
  forced reindexing despite measured cosine agreement above 0.9998. Two float8
  DiskANN snapshots per generation and exact rescoring added operational cost.
  Transcript windows were roughly 700 tokens. This stack, canonical-URL
  identity, transcript import and its configuration surface were not carried
  forward. These are predecessor measurements, not the current implementation.
- **Earlier design proposals were superseded.** Original-text file/stdin ingest,
  one local transactional store and source-byte section spans survived. The
  initial proposal to remove embeddings entirely, require separate acceptance,
  and keep observation timestamps outside an envelope hash did not become the
  current contract. Current identity, acceptance and fixed encoding follow the
  LLPs; these historical proposals are not implementation instructions.
- **Database-engine distinction.** The cited
  [Turso 0.5 release](https://turso.tech/blog/turso-0.5.0) and
  [FTS reference](https://docs.turso.tech/sql-reference/functions/fts) described
  Tantivy in the database, transactional visibility after commit, manual
  OPTIMIZE INDEX, fts_highlight rather than bounded snippet(), and tokenizers
  default/raw/simple/whitespace/ngram rather than unicode61 tokenchars.
  The recorded conclusion was to retain SQLite-derived libSQL FTS5 because
  identifier tokenization and bounded snippets were required.
- **Search without an extra embedding stack.** Simon Willison's
  [embeddings discussion](https://lobste.rs/s/lqbeno/embeddings_are_underrated)
  and [search-based retrieval discussion](https://lobste.rs/s/iyq4pf/building_search_based_rag_using_claude)
  described extracting search terms and querying SQLite FTS. The recorded
  anecdotal conclusion was lower cost and useful exact-phrase/name behavior;
  cosine distance supplied no reliable rejection cutoff. Hybrid search remained
  an option, not a requirement. These reports were not benchmark evidence.
- **Keep always-on instructions small.** The
  [discussion of instruction-file contents](https://news.ycombinator.com/item?id=44193056)
  described using the file as a directory into architecture and other notes,
  with personal URLs and test accounts kept uncommitted. The recorded conclusion:
  the always-on file points to knowledge rather than containing the whole store.

## Staleness and irrelevant hits

- Ranking cannot do either job. BM25 orders whatever the query admits.

- A closed decision still contains the words. A note from another repo can share them.

- An any-term query on a small corpus will fill the page, because almost nothing is rare.

- Two different things go stale:

- The index, when the bytes changed. Content hashes and a rebuild handle that.

- The text you get is the text that was stored.

- The claim, when the world changed and the text is still byte-for-byte true as a historical sentence. Search will return it forever unless something other than similarity takes it out of the default set.

### What is eligible, before anything is ranked

- Default search is the intersection of hard filters. A row that fails one of them is not a low score.

- It is absent. Filters apply before ranking, so the page is not the leftovers of a global top ten.

- [A memory-search note](https://aimemoryworks.com/infrastructure/hybrid-search/) makes the same cut for a different stack: a hit from the wrong scope is an incident, not a weaker match, and a time window has to exclude an outdated fact rather than hope rank buries it.

- **Surface.** Accepted memories are the default. An imported document is a different surface, searched when the caller asks for documents.

- A long file must not answer a memory question by containing one shared word. Unaccepted ingests stay in the store and out of the default surface.

- Easy ingest and the write bar stay compatible: capture is allowed, citation is not, until someone accepts the note.

- **Scope.** The caller passes the checkout it is in. A memory with that scope, or with no scope, is eligible.

- A memory scoped to another checkout is not. Scope is not a weight.

- **Validity.** Each record has `valid_from` and `valid_until`. `valid_until` null means still current.

- Default search requires that. Closing a record keeps the bytes.

- History is a separate query.

- **Mandatory tokens.** If the query contains an identifier, that identifier is required. "deploy" plus a miss on `E_INDEX-42` is not a hit. For a short query, every non-stopword term is required.

- The any-term pass exists only when the caller asks to widen. An empty result is a miss.

- Filling the page with one-term overlaps is how irrelevant notes arrive. On a small corpus, inverse document frequency barely separates those overlaps, which is why a score floor does not travel between corpora and is not the filter.

- Negation is not a filter the index can apply. "do not use corepack" matches a query for corepack. The snippet is the guard: it is a window of the record, and the agent reads it before treating the hit as an instruction.

### How a claim stops being current

- A newer note does not close an older one because they share words. "Marco likes tea" leaves "Marco likes coffee" open. Closing is an explicit write, naming the record.

- The useful split, from an external database system and from the four operators in [the healthcare arbitration paper](https://aclanthology.org/2026.knowfm-1.10.pdf), is small enough to do without a second model in the index:

- **Support.** The new text agrees. It adds an evidence link.

- Validity does not change.

- **Refine.** The new text is the same claim, more specific. The old record stays as history.

- The new one is current.

- **Supersede.** The writer says the old claim ended: "no longer", "switched to", a replacement they name. The close is a new envelope.

- A local column, `valid_until`, is derived from that envelope and points at the new record. The old envelope is not rewritten.

- It was true then.

- **Correct.** The old claim was wrong when written. Same close, different reason, so a later reader can tell a fix from a change.

- **Conflict.** Two open claims disagree and nobody has closed either. Both stay in default search, marked as a fork.

- Newer-wins would hide the one that was right. Two machines superseding the same parent are this case, and both children come back.

- Expiry is the close with no replacement. A note whose author already knows it is temporary ("until the flake lands") carries `valid_until` at write time and drops out of default search on that date.

- A forgotten or retracted record is withheld from history queries too. Supersession is not deletion.

- The moment to ask which record this replaces is the same tick list as reflect. If the writer does not name one, the new text is a new open claim, and both can be retrieved.

### Evidence that can be checked without a model

- A memory that depends on a file records the path and the git blob id, taken from `git hash-object` with filters applied. That id matches `HEAD:<path>` on a checkout whose line endings differ.

- `git hash-object --no-filters` and a raw hash of the working-tree bytes do not, which a CRLF attribute confirmed on git in this environment. Files outside a work tree use a raw hash.

- At read time, if that file is in the current checkout and the blob id differs, the hit comes back labeled `evidence-changed`. It is not dropped, and it is not presented as current.

- Dropping it hides the drift. Trusting it repeats the stale fact.

- The label is what recall already wanted, done as a compare instead of a reread. If the file is not in this checkout, the hit says `unchecked`.

- A memory with no evidence pointer is still a claim. Default search can prefer records that have one when the scores tie.

- It should not refuse a decision that was never about a single file.

- Automatic contradiction search, cross-encoders, and age decay stay out. Decay hides a decision that is old because it stayed true.

- A similarity merge closes the coffee note because it looks like the tea note. Both fail in the direction this store cannot afford.

## New mechanisms, with measurements

- These are later finds. They change the stale-claim and bad-hit rules above.

- They do not add a model to the query path.

### A compiled page is not the record

- Andrej Karpathy's [LLM wiki note](https://gist.github.com/karpathy/442a6bf555914893e9891c11519de94f) puts a maintained markdown wiki between the reader and the raw sources. Ingest updates the pages, query reads `index.md` and then the page, and a periodic lint looks for contradictions, superseded claims, orphan pages, and missing links.

- He reports that an index works at about a hundred sources and a few hundred pages, which is the scale of this store, and that it removes any need for an embedding index.

- The failure mode is in the same note: one ingest touches ten or fifteen pages, and the wiki is a paraphrase. A single unpublished comparison of memory designs reported exact recall of 86% for Claude's own memory files and 3% for a wiki, because the wiki had rewritten the fact.

- Treat that number as one person's benchmark, not a paper. The mechanism is enough.

- Search runs on the original bytes. A compiled summary is a separate accepted memory that links to the source.

- It does not replace the source, and ingest does not rewrite a fan of existing pages. Lint is a real third operation, beside reflect and recall: walk evidence hashes, open conflicts, and pages nothing links to, and put the list in front of a person.

- [A later wiki-retrieval paper](https://arxiv.org/html/2605.25480v2) adds an error book, where a recurring construction mistake becomes a constraint on the next compile and a repair of the pages already written. That is the same promotion an external memory workflow already wants, from a repeated lesson to a check.

- [A-MEM](https://arxiv.org/html/2502.12110v5) is the published version of automatic evolution. A new note rewrites the context of older notes it judges related.

- That is the similarity merge, with embeddings. The links are useful when a person or an explicit write creates them.

- The unsolicited rewrite is not.

### Lineage stops the stale hit from ranking first, and does not fix paraphrase

- [Link](https://github.com/gowtham0992/link) is local markdown memory with review-gated writes and no model on the recall path. Its [benchmark table](https://github.com/gowtham0992/link/blob/main/benchmarks/RESULTS.md) is the first measurement that matches this store's supersede rule:

| Store | Query | Current value first | Stale value first | Miss |
| --- | --- | --- | --- | --- |
| With lineage | Direct | 0.94 | 0.00 | 0.06 |
| With lineage | Paraphrase | 0.31 | 0.00 | 0.69 |
| No lineage | Direct | 0.81 | 0.12 | 0.06 |
| No lineage | Paraphrase | 0.25 | 0.06 | 0.69 |

- Naming the replaced record drives stale-first to zero on that slice. It does not create paraphrase recall.

- The miss stays 0.69 either way. Contradiction exposure at 3 after a revision is 0.000 with the lineage and 0.167 without it, on their hygiene set.

- Word rules that compare a new memory to stored memories about the same subject catch 17 of 20 revisions with 0 false alarms on 216 unrelated pairs. An 87 MB NLI model, alone, catches the same 17 and raises 107 false alarms on those 216 pairs.

- With a subject-overlap gate it adds one revision the words missed. Link ships it as an opt-in review note, never as a refusal and never on the recall path.

- That is the empirical case against automatic contradiction detection.

- Link's staleness check has a precision rule worth copying onto the evidence hash. A reference is reported only when git history shows the thing existed.

- Their count is 0 false flags on 345 memory references in their own docs, and 0 false flags on 604 references in `AGENTS.md` and `CLAUDE.md` across eight public repositories, with every planted deletion caught. A path the note invented is not "stale." A path, symbol, script, or version that history shows and the tree no longer has is `evidence-changed`.

- The check also covers the always-on instruction file, which this system generates and which can name a command or a version the repo has since dropped.

- Two further Link practices belong with the write bar. A memory that will constrain a tool call cannot be marked reviewed by the agent that wrote it.

- And the useful memory is the one present at the tool call, not only in the brief the agent agreed with forty minutes earlier. This store does not need their hook runner.

- It does need the rule that acceptance of an enforceable note is a separate actor from the writer.

### A weak hit is worse than no hit

- Google's [sufficient-context paper](https://arxiv.org/abs/2411.06037) separates "topically related" from "enough to answer." On their sets, standard benchmarks contain a large share of insufficient contexts (55.4% of Musique in one figure). With RAG, models abstain less.

- Claude 3.5 Sonnet abstains on 84.1% of questions with no context and on 52% once context is added. Gemini 1.5 Pro drops from 100% abstention to 18.6%.

- The model treats any retrieved paragraph as a reason to answer. Insufficient context raises hallucinations.

- So the tool result for an empty eligible set says that nothing qualified. It does not attach the next-best note "in case it helps."

### Fields that already say whether a note is current

- [OKF v0.2](https://github.com/GoogleCloudPlatform/open-knowledge-format/blob/main/SPEC.md) is markdown plus frontmatter, and the families that matter are small. `generated` records who wrote the bytes and when.

- `verified` is a separate list of who confirmed them, so a rewrite does not silently keep an old sign-off. Trust is derived, not stored as a score: no `verified` means unverified, only machines means machine-confirmed, a `human:` actor means human-reviewed.

- A missing trust block is still readable. OKF forbids consumers from rejecting it.

- That matches a candidate surface, not a hidden row. `status` is `draft`, `stable`, or `deprecated`.

- `stale_after` is an absolute instant, compared with now, not a TTL counted from the moment of reading. `sources[].last_modified` is when the source changed, which is distinct from when the note was written.

- Credibility is not a number on the note, because a number goes stale and does not travel between consumers.

- a shared-fact memory system is the working version of the shared-fact half. Several agents search, read, and follow links.

- Writes are proposals. The model that drafted a proposal cannot approve it.

- Apply fails if the git revision moved. The FTS index is derived and rebuildable.

- Chats, reminders, and credentials stay out. That is the write bar with an actor split, and it is the concurrency rule for two machines accepting different children of one parent.

## What a later pass added

- A Claude Opus research pass read this file first, then went out through the FTS5 manual, a few papers, and the harness docs. The items below were checked again against those sources, or rerun locally on SQLite 3.54.0, before they were written down.



### Index every section, then filter

- An external-content FTS index that only contains the "visible" rows cannot be checked, and it cannot be updated. [FTS5 §6.7](https://www.sqlite.org/fts5.html#the_integrity_check_command) compares the index to the content table only when `integrity-check` is run with rank 1.

- Two local cases failed that check:

- The content row was updated without the trigger firing. A search for the old word still returned the row, and the snippet was the new text, which did not contain that word.

- The check then failed with a checksum mismatch.

- The insert trigger skipped rows with `redacted=1`, and the delete trigger had no such condition. The check failed immediately with a malformed-database error, before any update.

- So the index holds every section. Accepted, scope, and validity are `WHERE` or `JOIN` conditions on the query.

- `integrity-check` with rank 1 belongs in tests and in lint. A filtered trigger is how an index and its content table drift apart while search still returns rows.

### A second column for the pieces of an identifier

- With `unicode61` and `tokenchars '_-'`, the body of `--force-with-lease` is one token, leading dashes included. On SQLite 3.54.0 a body-only index returned nothing for `"force"`, for `"read" "only"` against `read-only`, and for `"local-only"` against `--local-only`.

- `"E_INDEX-42"` still matched as one token.

- A generated column `replace(replace(body,'-',' '),'_',' ')`, indexed beside the body at a lower weight, makes `"force"`, `"INDEX"`, and `"local only"` match. The stored bytes do not change.

- The query builder has to split on the same characters when it targets that column. The phrase `"local-only"` still misses, because the body token is `--local-only` and the parts column holds `local` and `only` as separate words.

- A custom synonym tokenizer is the other way to do this, and it needs an FTS5 extension. No such crate turned up, and the generated column does the job.

### The envelope needs a canonical byte encoding

- Two machines that hash the same fields in different JSON will not converge. [RFC 8785](https://www.rfc-editor.org/rfc/rfc8785) (JCS) sorts object keys and does not Unicode-normalize.

- Every encoder that uses it has to preserve the string as given. A label copied off a macOS filename can arrive in a different normalization form than the same label typed on Linux, and both encodings will keep that difference.

- Normalize the label to NFC before the canonical bytes are hashed. DAG-CBOR is the other candidate, because content-addressed systems already sort map keys and reject indefinite lengths.

- Pick one encoding and use it everywhere. The choice is part of the format version inside the envelope.

### Validity is a local projection of close envelopes

- `valid_until` cannot be a field that later writes poke into an immutable envelope. Supersede, Correct, and a later Expiry are themselves envelopes.

- The local database derives `valid_until` from them. An expiry the author already knows at write time ("until the flake lands") is an absolute instant in the original envelope, which is the OKF `stale_after` shape.

- Two machines that close one record toward different replacements leave both closes in place. That is the Conflict case already specified.

- a temporal database system draws the same line from the other direction. Users do not set system time.

- The database records when a fact was written. Valid time is the time the fact was true, and a correction does not erase the earlier belief.

- This store does not need a temporal database system. It does need to keep "when we wrote the close" off the validity column.

- a database system makes the matching distinction for deletion: retraction fixes an error and leaves history, excision removes bytes and leaves a permanent record that the excision happened. Excision records cannot themselves be excised.

- Forgetting a body is excision. Closing a claim is retraction.

### A forgotten body needs a tombstone the sync can see

- Sync that copies any hash a peer has will resurrect a body someone deleted, as long as one machine still holds the blob. The forget is an envelope that names the body hash and carries no bytes.

- A peer that sees it does not fetch that blob, and does not reinsert the row. [FTS5 secure-delete](https://www.sqlite.org/fts5.html#the_secure_delete_configuration_option) (SQLite 3.42.0) actually removes the old index entries instead of leaving delete-keys that reconstruct the text.

- Pair it with `PRAGMA secure_delete=1`, or the freed pages in the file still hold the bytes. a database system documents the same hole for its own full-text index: excision of `:db/fulltext` attributes is not supported.

- Deleting the FTS row under secure-delete is the step that closes it. Once a row has been deleted that way, FTS5 older than 3.42.0 cannot read the table.

### Terms the author adds, beside the bytes

- [LongMemEval](https://arxiv.org/html/2410.10813v2) Table 9 is chat history, not this store, and the extra keys were extracted by a model. On BM25, adding an extracted fact to the value raised Recall@5 from 0.634 to 0.683 for whole sessions and from 0.472 to 0.554 for rounds.

- A summary used as the key did not help: the same table shows session Recall@5 falling from 0.634 to 0.626. Section 5.3 says a condensed form used on its own does not raise recall.

- Rank-merging a separate key after retrieval scored worse than putting the extra terms in the index entry.

- That fits the paraphrase miss Link measured, without a model on the query path. When a person accepts a note, they can add a few query terms.

- Those terms are a low-weight column next to the body. They are not a summary that replaces it.

- There is no measurement yet of terms a person wrote, as opposed to terms a model extracted from a chat log. [Claude Code path rules](https://code.claude.com/docs/en/memory) are the same idea at harness scope: a rule with a `paths` glob loads when a tool touches a matching file, and an invalid glob matches nothing while the rule's other patterns still apply.

- A record may carry globs inside its checkout. When the caller names the files it is touching, the glob is a hard filter.

- It is not a rank feature.

### Say why nothing came back, without leaking the rest

- `fts5vocab` can count how many indexed rows contain a term. The count is only safe for rows the caller is already allowed to see.

- Forgotten rows are gone, so they cannot be counted. Rows in another checkout are not mentioned.

- Inside this scope, "the term occurs only in closed records" is a useful empty result, and it returns no snippet. A raw vocab count over the whole index would tell the caller that a forgotten or out-of-scope note exists.

- A record whose parent was closed stays in default search, labeled `parent-closed`. [StateMemBench](https://arxiv.org/html/2608.19652) marks dependents `needs_recheck` with no model call, and still returns them.

- Dropping that propagation left their DeepSeek score slightly higher (0.373 against 0.363) and cost one scenario set 12.5 points where the flag over-fired. The label is the part worth keeping.

- Hiding the dependent record is the part that hurt.

### Outside evidence for rules already chosen

- [MINJA](https://arxiv.org/abs/2503.03704) puts records into an agent's memory through ordinary queries and the agent's own replies, with no direct write to the store. A person accepting the note is what blocks that path.

- The agent that produced the text does not get to accept it.

- [MemDev-Bench](https://arxiv.org/html/2610.27524) found that noisier history raised regressions rather than burying the useful part. On already-solved tasks, stuffing the raw trajectory into context moved harm from 0.7% to 4.2%.

- Agents followed the injected memory instead of checking it against the task. The paper also found the raw trajectory used about 34% fewer steps than no memory, because it kept the failed attempts.

- That process detail belongs in a handoff. It is not a reason to search session transcripts.

- [VibeMemBench](https://arxiv.org/html/2609.23570) kept 111 coding tasks only where a known-good prior experience helped a reference run. Retrieval by existing memory systems still failed to beat the matched memory-off baseline in eleven of twelve solver and system pairings.

- The same record helped a weaker solver and hurt a stronger one. A note is not injected because it ranked.

- The caller asks, and an empty eligible set stays empty.

- [An AGENTS.md study](https://arxiv.org/abs/2602.11988) found that context files did not generally raise task success, while inference cost rose by over 20%. LLM-generated files were the ones to drop.

- Developer-written files helped only when they said something the README did not, and the success change was not significant. That is the argument for a small always-on prompt and for a write bar on anything that would be promoted into it.

- Claude Code loads the first 200 lines or 25KB of `MEMORY.md`, and drops the rest on the next load. An always-on index has a size cliff.

- The store stays on demand. The index of it does not grow into the prompt.

- [GitHub Copilot's memory post](https://github.blog/ai-and-ml/github-copilot/building-an-agentic-memory-system-for-github-copilot/) stores a subject, a fact, citations, and a reason, and tells the agent to read the cited lines before using the fact. Planted false memories were corrected in their tests.

- The post gives no count for that test. Their A/B result, a merge rate of 90% against 83%, is their own claim and names no sample size.

- The citation is the evidence pointer this store already wants. The check belongs in the tool result, as `evidence-changed`, rather than in a prompt the agent may skip.

- Their other habit, re-saving a memory to refresh a timestamp and then loading the most recent ones at session start, is recency ranking. It stays out.

### Looked at, and not adopted

- an external search tool mixes a normalized BM25 with a 30-day recency decay, uses a Porter tokenizer, and shows the start of the row instead of a window around the hit. The decay and the snippet both conflict with rules already set.

- Its filtered insert trigger is the shape that failed `integrity-check` above.

- an external note index treats a leading `-` as `NOT` in its query syntax, so a flag becomes a negation. Its token characters include `/`.

- Letta replaces a whole memory block on write, and the last writer wins. Parent hashes already keep both children.

- Codex writes memories in the background after a chat goes idle, on one machine, with no acceptance step.

- Anthropic's memory tool recommends deleting files that have not been accessed in a long time. That is expiry by last read.

- It also edits a file in place.

- StateMem's supersession mark is made by a model at write time. The close stays an explicit write a person accepts.

- `contentless_delete=1` stores no text, so `snippet()` has nothing to return. `detail=column` and `detail=none` drop phrase queries.

## Trees and graphs

- Two later explorations asked whether structure can do what ranking cannot. Neither is a decision yet.

- Proposals are marked as proposals.

### Trees

- "Tree" names four different things in the prior art.

- **A location tree.** Claude Code [loads a subdirectory `CLAUDE.md`](https://code.claude.com/docs/en/memory) only when Claude reads, writes, or edits a file in that subdirectory, and path-scoped rules load only for matching files. The repo's own file tree decides relevance, and no text is ranked.

- Proposal: a note hangs on one node, `<org>/<repo>`, a directory in it, or a file, and is eligible when the caller works at or below that node. That joins the checkout scope and the path globs above into one hard filter.

- It removes wrong notes before ranking. Wrong notes are the harm MemDev-Bench and VibeMemBench measured above.

- Revised and accepted 2026-11-04: eligibility by working directory makes results depend on where the agent stands, so the default scope is the checkout, and a subtree narrows only when the query names it with `path:` or `under:` (see "Location narrows, it does not answer").

- **A heading tree.** Structured-document retrieval measured this before agents. In a [2003 INEX run from the University of Amsterdam](https://e.humanities.uva.nl/publications/2004/sigu_elem04.pdf), on the strict content-and-structure task, element scores alone reached MAP 0.2987, whole documents 0.2314, and elements mixed with their article's score 0.3182.

- Elements alone had the best precision at 5 and 10. The authors say to take the per-class results with a grain of salt, because the topics were few.

- [Contextual retrieval](https://www.anthropic.com/news/contextual-retrieval) prepended 50 to 100 tokens of model-written context to each chunk; with that context in both embeddings and BM25, top-20 failures fell 49% (5.7% to 2.9%). Generic document summaries gave "very limited gains." A plain heading-path prefix was not tested, so the model-free version has no measurement.

- [cAST](https://arxiv.org/abs/2506.15655) split code on its syntax tree instead of line counts and raised Recall@5 by 4.3 points on RepoEval and Pass@1 by 2.67 on SWE-bench.

- Proposal: index each section's heading path as its own low-weight column, give some weight to the whole document's match, and return the section. FTS5's `bm25()` applies column weights to the phrase frequency inside one score, and `|D|` is the token count of the whole row ([FTS5 §bm25](https://www.sqlite.org/fts5.html)), so a long path makes every section count as longer.

- A query that matches only a document's title matches every section of it, so results group by document or one file fills the page.

- **A summary tree.** [RAPTOR](https://arxiv.org/html/2401.18059) clusters chunks with embeddings and has gpt-3.5-turbo summarize each cluster, recursively. Scoring every layer in one pool "consistently performs better" than walking down the tree, and 18.5% to 57% of retrieved nodes came from summary layers (Table 18).

- In [an agent-memory survey](https://arxiv.org/html/2606.24775v1), a deeper MemTree gave small gains over a flat setting (Answer F1 30.7 to 31.2 on LoCoMo, Substring EM 23.0 to 23.3 on LongMemEval), and the survey recommends keeping raw content, because a hierarchy cannot restore detail already removed. a document-navigation system extracts its tree from the layout without a model, then adds model summaries and lets a model search the tree; its 98.7% on FinanceBench is the project's own claim.

- A summary tree needs a model on the write path and searches paraphrase, so it is out. RAPTOR's useful lesson, score all levels in one pool, is what indexing every section already does.

- A person's summary stays a separate accepted note that links to its sources.

- **A storage tree.** A body could be a tree of section hashes, as git stores a directory. An edit would keep the hashes of the sections it did not touch, and sync would move only changed sections.

- This is reasoning, not a measurement, and it does not change which note ranks first.

### Graphs from access

- A graph of what was accessed, where, and what came next has twenty years of prior art in developer tools. It helps when edges come from strong, typed events.

- It hurts when edges come from co-viewing during exploration.

- **Task context.** [Mylyn](https://eclipse.dev/mylyn/publications/2006-11-mylar-cscw.pdf) gives each element a degree-of-interest from its edits and selections during one activated task, and uses that context to filter and highlight the IDE. In a field study of 16 programmers over several weeks, they spent more time editing than navigating, with statistical significance.

- The graph belongs to a task, not to global popularity.

- **Co-change.** [ROSE](https://www.st.uni-trier.de/diehl/pubs/icse04.pdf) mines version history: "Programmers who changed these functions also changed…". After a first change it predicted 26% of further files and 15% of the exact functions or variables; the top three suggestions held a correct location 64% of the time.

- Git already holds this graph for every checkout.

- **Co-view noise.** [CITR](https://letaiw.github.io/pubs/EMSE24.pdf) combines interaction traces from similar past tasks and recommends 73% of the files to edit on average, 31% above the MI baseline. It also states the failure: rules built from elements "frequently edited or viewed together" recommend unrelated elements when developers touched the wrong elements first.

- In one comparison it cites, Team Tracks recommended three methods and none was needed.

- **Typed provenance in agent runs.** [A provenance study](https://arxiv.org/abs/2609.25913) over 1,207 held-out agent trajectories found that units cut at tool-call boundaries raised Full Support@2048 by 19.07 points over flat 512-token windows. Propagating scores over typed provenance edges added 4.55 points, mostly when evidence spanned several events.

- Entity co-occurrence expansion gave no comparable benefit.

- **Graph against reading.** [Codebase-Memory](https://arxiv.org/abs/2603.27277) reached 83% answer quality across 31 repositories against 92% for a file-exploration agent, with ten times fewer tokens and 2.1 times fewer tool calls. Cheaper, and less accurate.

- **Search trails.** [Bilenko and White](https://dblp1.uni-trier.de/rec/bibtex/conf/www/BilenkoW08) treat post-search browsing as implicit relevance. The full text was not available, so there are no numbers here.

- Proposal: record only typed edges from explicit events. A search hit followed by an edit in the same checkout in one task.

- A note the agent quoted or followed. Co-change, read from git.

- Files read before a note was written, which is close to the evidence pointer. Leave out "opened together," because agents explore ([RepoReuse](https://arxiv.org/html/2609.35357v1) measured 27.7 files opened per turn without memory), and leave out visit counts as a rank feature.

- Default search still returns only rows that pass the hard filters, in the same order. The graph adds a short related list beside the hits, scoped to the checkout and labeled by edge type.

- An empty search stays empty. Events are paths, hashes, and times, not transcript text; the store logs its own hits, and Claude Code hooks on tool calls could log edits.

- A forget removes the edges too, or the graph still says the note existed.

- This goes past two lines above: "an explicit link the author wrote, parent or evidence, is enough graph," and the exclusion of access-count promotion. A related list that never changes rank respects the second.

- The first needs a decision. The test is the same set of real cases: when the text query misses, does a co-change or hit-then-edit edge from an earlier task reach the right note?

## Narrowing by derived structure

- The direction in Decisions is search that narrows through structure the store derives, not structure an agent maintains. Older tools already do this from a command line.

- Recent papers say how deep the structure should go.

- **Highest priority to explore:** a document query tool's Markdown selectors, an indexing tool's deterministic section tree and its progressive disclosure, and the draft CLI below, layered with a retrieval tool's filter, rank, inspect, and fetch, and with the other learnings in this file. These are the ideas to focus on.

### Prior art in CLI and query languages

- **GitHub code search** ([syntax](https://docs.github.com/en/search-github/github-code-search/understanding-github-code-search-syntax)) mixes qualifiers and text in one string: `repo:`, `org:`, `path:` with globs (`*` stops at `/`, `**` crosses directories, a leading `/` anchors), `language:`, `symbol:`, `content:`, and `is:`. Spaces are AND, with `OR`, `NOT`, and parentheses.

- `symbol:` comes from Tree-sitter parsing with "no extra setup," and finds definitions only. The structure is derived from the code, and the user narrows by naming it.

- **an email indexer** ([search terms](https://notmuchmail.org/doc/latest/man7/an email indexer-search-terms.html), [count](https://notmuchmail.org/doc/latest/man1/an email indexer-count.html)) derives threads from message headers, which nobody files by hand. Boolean prefixes (`tag:`, `id:`, `thread:`, `folder:`, `path:`, `property:`) match exactly, and free-text terms must all be present.

- Results sort by date, newest first by default, not by relevance (an email indexer). `an email indexer count` prints the count of messages or threads for a query, and `--batch` takes one query per line and prints one count per line.

- Counting before fetching is a cheap narrowing step.

- **a document query tool** ([README](https://github.com/yshavit/a document query tool)) is "like jq but for Markdown." Its selectors mirror Markdown: `# title text` for sections, `- item` for list items, `[text](url)` for links, fenced code, tables, and front matter. Pipes chain them, so `# usage | -` selects list items inside sections whose heading contains "usage." It emits JSON.

- That is the heading tree as a query, over unchanged bytes.

- **Datasette facets** ([docs](https://docs.datasette.io/en/stable/facets.html)) count the values of a column across the current result set, including JSON arrays and dates. Selecting a facet value adds a filter.

- Datasette suggests a facet only for a column with 30 or fewer unique values, and gives each suggestion query 50 ms.

- **IWE** ([Markdown as a database](https://iwe.md/blog/markdown-as-a-database/), 2 May 2026) derives a graph from how links are written. A link alone on its own line is an inclusion link and makes a parent-child edge; an inline link is a reference, "see also." The key is the path without `.md`.

- `iwe find` walks the graph with `--includes`, `--included-by`, `--references`, and `--referenced-by`, with `KEY:DEPTH` and a default of one hop, and filters on frontmatter. `iwe count` counts, and `iwe delete --filter ... --dry-run` previews a delete.

- The writer's ordinary Markdown is the structure: "Build for what humans already do; the agent inherits it." It also describes a model promoting prose details into frontmatter, which this store leaves out.

- **an indexing tool** ([README](https://github.com/jackjin1997/an indexing tool), undated) parses each Markdown file into a section tree with mdast and "never call[s] a model; the same input always produces the same tree." A subtree merges into its parent when reading it whole costs no more than routing through it (150 tokens of routing cost), and a node over 8,000 tokens splits at block boundaries. Node ids hash the ancestor title chain and sibling position.

- BM25 indexes only a node's own text, with titles weighted 10, merged headings 5, summaries 2, and body 1. The CLI is `index`, `tree`, `search`, `mcp`, and `eval`.

- A published test of 30 questions with Claude Code found the gold passage in 30 of 30, first in 93.3%, with 88.6% of cited lines inside the gold range, against 73.3% and 12.8% line precision for BM25 top-5.

- **A keyword endpoint inside an agent loop.** [Aaron Tay](https://aarontay.substack.com/p/what-changes-when-an-llm-agent-searches) (24 June 2026, just before the four-month window) found a single-pass natural-language search returned zero results where an agent calling the plain keyword search broadened, corrected spelling, and switched scope over several calls. A weaker model in the same loop did about as well.

- The loop does the reformulation, which only works if an empty result is honest.

### Recent results on depth and shape

- [Progressive disclosure](https://arxiv.org/abs/2607.17598) (20 July 2026, InfiniteBench, three harnesses, three model families): one level of routing from a short description down to passages beat raw navigation when there were many books. A second routing level did not help and sometimes lowered accuracy sharply.

- Their summary: "Progressive disclosure buys context, not intelligence." The abstract gives no accuracy figures.

- [RIT-RAG](https://arxiv.org/abs/2610.11370) (8 October 2026) builds one tree per document offline from its table of contents or sitemap. At query time a broad retrieval places chunks in those trees, and the positions form small sub-trees, possibly across documents, that an agent reads selectively. "Retrieval proposes where to look, while the agent decides what to read." On EntQABench, 2.84 million documentation pages, it beat the strongest baseline by 6.6 to 11.4 points across three models.

- a retrieval tool, above, filters with a Boolean query over titles, headings, sections, and metadata before ranking, and Boolean filtering improved every ranker it tested.

- [A study of agents' web search](https://arxiv.org/abs/2609.19244) (16 September 2026) found that "more frequent Web-search invocation does not necessarily yield better response quality."

### What one call could return (proposal)

- Structure derived on ingest, with no agent maintaining it:

- **Tree** (superseded in part by "Storing and retrieving core": the heading path is not a lexical field, and merging affects display only): the heading path of each section, from the bytes, with an indexing tool's rule that a small subtree stays merged and only an oversized node splits; the location of each record, from its scope and evidence paths.

- **Graph:** links written in the bytes, typed by position as IWE does (a link alone on a line includes, an inline link refers), parent links, closes and supersessions, evidence links, co-change from git, and a parent proposed when the same local path is added again with new bytes. The last one keeps editing natural: add the file again, and the new body names the old one as its parent.

- One command, one query string, mixing exact prefixes with ranked words. `mem` is a placeholder name:

```text
mem search 'under:"Git send" path:home/development/** "local-only" forwarding'
```

- The prefixes are exact filters: `scope:`, `path:` (glob, as in GitHub), `under:` (a heading path, as in a document query tool), `kind:note|doc`, `state:open|closed`, `id:`. The remaining terms go through the phrase pass and the all-terms pass already specified.

- The result has three parts:

- Hits grouped by document, each with its heading path, snippet, hash, and labels. RIT-RAG's sub-trees group hits by document, and the INEX run scored best when section and document evidence were mixed.

- One level of facet counts over the matched rows (corrected 2026-11-05; the accepted rule in "Location narrows" governs): by location, by top-level heading, by kind, and by state, such as the count of matches hidden because they are closed. Only rows the caller may see are counted.

- Progressive disclosure found that one level helps and two can hurt.

- A short related list from typed edges, never reordering the hits.

- The agent narrows by copying a facet into the next query, as a Datasette facet adds a filter. `mem show <hash> [--section <path>]` returns bytes.

- `mem count` answers a query with numbers only, as `an email indexer count` does. An empty result says nothing qualified, and gives the in-scope counts that explain why.

- Adding is `mem add <file|->`. Deleting is `mem forget <hash>` or a close.

- Neither asks the agent to place anything, because the structure comes from the bytes, the paths, and the links.

### Refining the priority: what an indexing tool, a document query tool, and a retrieval tool each settle

- **an indexing tool in detail** ([README](https://raw.githubusercontent.com/jackjin1997/an indexing tool/main/README.md), [eval](https://raw.githubusercontent.com/jackjin1997/an indexing tool/main/eval/README.md)).

- Five read-only tools carry the progressive disclosure. `browse_documents` lists a folder with path, title, description, and token count, paginated.

- `get_document` returns metadata and a `read_structure_first` flag above 15,000 tokens. `get_document_structure` returns the outline (node ids, titles, line ranges, token counts, summaries, no body) to a `depth`, default 2.

- `get_node_content` returns the original Markdown of 1 to 20 nodes with line numbers. `search_documents` is BM25 scoped by folder or document, "the escalation path when browsing stalls." The agent cites `[doc_path, L<start>-<end>]`.

- Indexing makes no network calls. Summaries are optional, from a model, behind `--summary-model`, and cached by node content hash.

- Files are compared by content hash, only changed documents rebuild, each in one transaction, and deleted files leave the index. Ids "stay stable while the outline does," and a stale id returns a hint to re-fetch the structure.

- The eval: 329 documents and about 38.1 million tokens; 151 questions written by the coding agent from sampled leaf nodes, paraphrasing the node, one gold line range each. BM25 top 5 found the gold for 80.1% (Hit@1 63.6%, English 64.8%) with 12.1% line precision.

- Claude Code limited to the five tools, on a 30-question sample, found 100%, Hit@1 93.3%, 88.6% line precision, a median of 6.5 evidence lines, and 3.1 turns per question. Caveats from the author: one model and one run, questions written from the gold text (which favors keyword search), and one gold per question.

- The gain over BM25 is mostly precision: the agent reads the right node instead of 200 lines around five hits. The test compares an agent using five tools with BM25 output, so it does not separate the structure from the agent's choices and repeated calls (corrected 2026-11-05).

- **a document query tool's matching rules** ([README](https://raw.githubusercontent.com/yshavit/a document query tool/main/README.md)). An unquoted string "is case-insensitive"; a quoted string "is case-sensitive"; `^` and `$` anchor; `/regex/` matches a pattern; `*` or nothing means any.

- `|` chains selectors so each narrows the last. `-q` prints nothing and exits 0 when something matched.

- These are a ready grammar for heading selectors.

- **a retrieval tool in detail** ([HTML](https://arxiv.org/html/2608.02751)).

- The query language restricts terms to fields (`term[ti]`, `term[sec]`, `term[tiab]`), with AND, OR, NOT, parentheses, `word*`, quoted phrases, and `date[RANGE]`.

- A result card holds the title, the section headings as fetch targets, the fields that matched, the infobox keys, and a 25-token query-focused snippet. A search returns ≤ five cards.

- Fetch returns one named section, or the infobox, never the whole page, within a 12,000-token ceiling per action.

- On HotpotQA, MuSiQue, and BrowseComp-Plus, a retrieval tool beat the best search-and-visit setup: 45.3 against 43.7 with 14k against 20k tokens, 29.2 against 26.1 with 21k against 43k, and 37.2 against 36.5 with 46k against 58k (Table 1). Against the matched control with the same rankers and section fetch but no Boolean filter, the filter improved all nine ranker and collection pairs; for BM25, 41.0 to 44.5, 24.5 to 27.9, and 30.6 to 33.6.

- Removing query-focused snippets cost 2.9 to 6.8 points.

- When the Boolean filter matched nothing, a retrieval tool returned a ranked fallback list. The fallback served 36.8% to 53.7% of search calls, and removing it cost 6.4 points on BrowseComp-Plus.

- **The tension with a settled rule.** a retrieval tool's fallback is a weak hit served when the exact query fails, and on those benchmarks it helped. This store has settled that empty is an honest miss and a weak hit is worse than silence.

- The two can meet without serving weak content: when a query is empty, return the counts of what each relaxation would match (drop `under:`, drop `path:`, widen to closed records, all within the caller's scope), and no snippets. The agent then widens on purpose.

- Aaron Tay's agent loop above recovered from zero results that way.

- **The shape these imply (proposal).**

- **Index** (superseded by "Storing and retrieving core" on three points: merging affects display only, the heading path is not an FTS column, and citations name an immutable body version)**.** One deterministic section tree per body, by an indexing tool's rules: a subtree stays merged when reading it whole costs no more than routing through it; a node over a token ceiling splits at block boundaries. Each section keeps its body hash and ordinal, and also a structural id from the ancestor heading chain and sibling ordinal, so the same section keeps its address when its document is edited and re-added.

- FTS indexes a section's own text, with its own heading weighted well above the body and the heading path as a low-weight column.

- **Query.** One string. Exact prefixes as above, plus a retrieval tool-style field restriction where it helps (`heading:`, `body:`).

- `under:` uses a document query tool's matching rules, and repeating it narrows, as `|` does in a document query tool: `under:usage under:flags` is a section under a heading matching "flags" inside one matching "usage".

- **Result.** At most five cards, grouped by document. Each card has the label, the heading path, the matched fields, a query-focused snippet of about 25 to 32 tokens, the child headings available to fetch, and the labels (`evidence-changed`, `parent-closed`, replaced record).

- Below the cards, one level of facet counts over the matched rows, and the related list.

- **Show.** `show <hash>` returns the outline to depth 2 when the body is long, as `read_structure_first` does, and the whole body when it is short. `show <hash> --section <id>` returns one section with line numbers.

- Citations take an indexing tool's form, a hash or path with a line range. A token ceiling applies per call.

- **Empty.** "Nothing qualified," plus relaxation counts, and nothing else.

- **Notes and documents differ.** Notes are short and many, so search is the entry. A long document is where the outline and section fetch pay off, and where an indexing tool's browse-first order fits.

- One CLI covers both. Progressive disclosure is several cheap calls to the same tool, not a second system.

- Open values, to set from the real cases rather than copy: the token ceiling at which a section splits, the routing cost that decides a merge when most notes are short, and the body length above which `show` returns the outline first. The empty-result reconciliation (relaxation counts, no fallback content) is recorded as a proposal against the settled rule, not yet accepted.

- Superseded on 2026-11-05: an empty result may name near-miss sections as hints (see Decisions).

- **Evaluation.** Adopt an indexing tool's method on real cases: gold as a line range, then hit rate, Hit@1, evidence lines, line precision, turns, and cost. Add the retrievability checks below.

- Questions written from the gold text overstate keyword search, so real cases come first.

### More input to the priority

- **Counts before fetching, and relaxations for an empty answer.**

- [Query previews](https://www.cs.umd.edu/~ben/papers/Donn1996Query.pdf) (Doan, Plaisant, and Shneiderman, 1996) split a query into a preview and a refinement. In the preview, rough attribute values show "the estimated number of matching data sets" so users "rapidly focus on a manageable number," and previews "prevent wasted steps by eliminating zero-hit queries." Only when the count is low enough is the query submitted for details.

- That is `mem count` and the facet counts, thirty years earlier.

- [The empty-answer problem](https://www.vldb.org/pvldb/vol6/p1762-mottin.pdf) (Mottin et al., PVLDB 6(14), 2013) names both failures of a conjunctive query: over-specified, "the empty-answer problem," and under-specified, "the many-answers problem." For an empty answer it suggests "alternative queries with less conditions," interactively, until the query returns something "or at a query for which no matter how many additional conditions are ignored, the answer will still be empty." This is the formal prior art for relaxation counts, including the honest case where no relaxation helps. Facets are the answer to the many-answers side.

- **A budgeted outline is navigation, not an answer.** a code-navigation tool ([design](https://a code-navigation tool.chat/2023/10/22/repomap.html)) uses tree-sitter for definitions and references, ranks files with "a graph ranking algorithm" over their dependencies, and fits the map into `--map-tokens`, 1k by default, adjusting the size to the state of the chat. It is the tree, the graph, and a token budget together.

- Its ranking is by importance across the repository, not by a search query, which is the failure "Location narrows, it does not answer" rules out for search. An outline of a scope can be an entry point that the agent asks for.

- It is never the result of a search.

- **Keeping a section's address across edits.** [GumTree](https://github.com/gumtreediff/gumtree) ("Fine-grained and accurate source code differencing," ASE 2014) is "a syntax-aware diff tool" that "can detect moved or renamed elements," not only insertions and deletions. Proposal: when a body is re-added with a parent, map the old section tree to the new one the same way, so a moved or renamed section keeps its structural id, its evidence anchors, and its edges.

- A section with no match is new. An unmatched old section is gone from the child, and still present in the parent's history.

- **The tool surface (July and August 2026).**

- [Restricting coding agents to code or bash](https://arxiv.org/abs/2607.10569) (12 July 2026) compared tool-rich, bash-only, and code-only surfaces for Claude Code and Codex CLI on synthetic tasks and SWE-bench Mini. Pass rates were statistically tied in every cell.

- Code-only was cheaper or tied in three of four cells, and directionally 14.4% costlier, not significant, in the fourth. A CLI surface costs nothing in pass rate.

- [The Bitter Lesson of Tool Calling](https://arxiv.org/abs/2608.06370) (6 August 2026) found programmatic tool calling, tools as typed stubs called from code, matched or beat JSON tool calls in 11 of 14 models on BFCL v4, and stayed stable where the JSON baseline lost 2.3% under context rot. Output that composes in a shell pipeline is not a handicap.

- [Output format and model identity](https://arxiv.org/abs/2607.21674) (23 July 2026) found no single output format best for every model across 4,013 single-round runs, though only one of its four projects produced any successes. Keep the default output plain and short, offer `--json`, and test the format with the models that will call it.

### Location narrows, it does not answer

- The risk: if location decides eligibility and the corpus in one folder is small, different queries from that folder return the same few notes. The folder then answers in place of the query.

- That is the same failure as filling a page with one-term overlaps, and it is worse, because it looks scoped.

- Prior art names the measurement. Retrievability ([Azzopardi and Vinay, CIKM 2008](https://dblp.org/rec/conf/cikm/AzzopardiV08.html)) is "the ease with which a document can be retrieved using a retrieval system.

- The more queries which retrieve the document, and the higher up the document is returned, the more retrievable it is" ([Wilkie, SIGIR 2016](https://static.aminer.org/pdf/20170130/pdfs/sigir/ek2n5nqwtwbhqs3rhxu8xaptispvdoja.pdf)). It needs no relevance judgments, so it runs on the live store, and the inequality across documents is summarized with a Gini coefficient.

- Vector search has the extreme case, hubness: a few items appear for many unrelated queries. FTS has a milder one, where a short note full of common project words matches many queries.

- Rules (accepted 2026-11-04):

- **Text must match.** Scope, `path:`, and `under:` only remove rows. They never add a hit, add a query term, or add score.

- A query whose words match nothing returns nothing, in any folder. This extends "Scope is not a weight" above.

- **No context expansion.** The store does not add terms from the current directory, branch, or open files. Only the agent's query string is searched.

- **The default scope is the checkout, not the working directory.** Narrowing to a subtree happens when the agent writes `path:` or `under:`. A note attached to a subdirectory still needs the text to match.

- **Counts come from matched rows.** Facet counts describe the hits for this query, not the eligible set, so two queries in one folder do not show the same facet list.

- **Related edges start from the hits.** The related list follows typed edges from the returned records only. A note that appears in related lists for many distinct queries is a hub, and lint reports it.

- **Mandatory terms stay mandatory.** The all-terms pass and required identifiers already keep one generic note from matching every query.

- Test (accepted 2026-11-04), without relevance judgments:

- **Overlap across queries.** For each scope, run a set of distinct queries: real ones from the collected cases, plus one known-item query per note built from its own heading and rarest terms. Measure the overlap of the result sets for unrelated pairs.

- It should be near zero.

- **Self-retrieval.** Each note should be the first hit for its own known-item query. A note that is not is under-retrievable.

- **Retrievability spread.** Count how often each note appears in the top results across all queries, and report the Gini coefficient per scope. Flag any note that appears for more than a set share of distinct queries.

- **Location invariance.** Run the same query from the checkout root and from a subdirectory. The results may differ only by rows a written `path:` or `under:` would remove.

- With no such prefix, they should be identical.

- These become lint checks and regression tests, so a change to tokenization, weights, or the tree does not quietly make one folder's notes answer everything.

### Storing and retrieving core, reviewed 2026-11-05

- Two independent reviews (Codex `gpt-6-astra` and Claude Fable) read this file, limited to storing and retrieving, and both rated the core "ready with named gaps." Each ran a throwaway SQLite 3.54.0 probe. Where they agreed, the point is recorded as agreed.

- Where they disagreed, the resolution below was chosen in the recorded study on 2026-11-05.

- **Agreed.**

- Bodies (original bytes by hash) and immutable envelopes are the durable units. Sections, offsets, trees, the FTS index, and validity are rebuilt locally.

- A section is never its own record.

- The heading path is not a lexical field. It is a plain column, used by `under:` and for display.

- FTS5's `|D|` is the whole row, so a weight of 0 does not remove a column's length from the score. Reproduced 2026-11-05: two rows with identical bodies, path column at weight 0, scored −2.6367 (5-character path) and −1.5336 (81-character path) in a 35-row table.

- Fable also found that a word appearing only in a title then matches every section of the document. Astra's reason: inherited text would make unrelated passages retrievable.

- [Does Document Structure Help Dense Retrieval?](https://arxiv.org/abs/2610.10170) (7 October 2026, dense only) found structure-aligned chunks beat contextualized fixed windows (+0.022 and +0.012 cov-nDCG@10) and real heading paths beat shuffled ones by only +0.010 and +0.016, effects the authors call small; naive two-stage hierarchical retrieval lost (−0.033 and −0.015).

- The matching unit is one section: the phrase pass and the all-terms pass match within one row, across that section's own heading and body. Document-wide conjunction is not the default.

- Filter, then rank by tier (phrase, then all terms), then heading-weighted `bm25()`, then fixed tie-breaks (body hash, section ordinal, envelope hash), then group by document, with a document card ranked by its best section. No location boost, no document-score blending, no any-term fill.

- At most five cards, each with record and body ids, label, matching section ids, heading paths, original-byte snippets, matched fields, and status labels. The related list starts only from returned hits, under the same eligibility.

- A card's section id fetches that section's original bytes directly. The outline is optional navigation for a long body.

- Oversized sections and outlines paginate explicitly.

- Empty is "Nothing qualified," with no fallback list. Relaxation counts remain a proposal.

- One query string with prefixes. Raw FTS syntax is not exposed.

- `OR` and `NOT` are left out at first. Unknown prefixes and malformed syntax are errors, not misses.

- **resolved ( 2026-11-05).**

- **Merging affects display, not the index.** Search rows follow the heading tree, with an oversized section split at block boundaries. an indexing tool's small-subtree merge applies only to how outlines and fetches are presented. A routing-cost setting must not change which terms count as matching together. (Fable proposed merged nodes as index rows.)

- **Labels do not satisfy query terms.** Only a section's own heading and own text match. The label is for display and explicit lookup.

- This follows "Location narrows": inherited text must not answer. (Fable proposed a label field in the FTS row.)

- **Citations name an immutable body version.** A section is addressed by body hash, derivation version, and section ordinal or byte span. An edited body gets new addresses.

- Correspondence across versions, for example by GumTree-style matching, is a separate optional relation, adopted only if something needs "this section's latest version." (Fable proposed a heading-path slug with a duplicate counter.)

- **The parts column satisfies pieces, never an exact identifier.** A query identifier such as `E_INDEX-42` must match the body token. Parts serve component searches such as `INDEX` or `force`.

- Astra's probe confirmed parts collide (`E_INDEX-42` and `E INDEX 42` look the same there) and that a parts-only hit has no highlight in the original body. (Fable proposed expanding every identifier to `body:"T" OR parts:"T split"`.)

- **Open.** `path:` semantics and relative links; global BM25 statistics across scopes; how section text is stored next to the body; counting records and sections before the display limit; small-corpus IDF; snippets for parts-only hits. See "Open storing and retrieving questions" below.

### Open storing and retrieving questions, researched 2026-11-05

- Each answer below comes from the source named or from a throwaway probe on SQLite 3.54.0 (`sqlite3`), run in a temporary directory that was then removed. Timings are from this machine on a synthetic corpus: 50,000 sections of 20 to 120 words drawn from a 5,000-word Zipf-like vocabulary, in 20 scopes.

- **Do other scopes change the ranking? Yes, and scope-local BM25 is cheap.**

- FTS5's statistics are table-wide. `xRowCount` "provides the total number of rows in the table, not the number that match the current query," and `xColumnTotalSize` counts tokens "considering all rows in the FTS5 table" ([FTS5 auxiliary API](https://www.sqlite.org/fts5.html)).

- A `WHERE scope = ?` filter does not change them.

- Probe: two scope-A rows that tied at −5.5597 for `"deploy" "cache"` separated to −5.0727 and −3.1111 after 41 scope-B rows containing "deploy" were added. Another checkout's notes decided the order inside this one.

- Elasticsearch has the same problem across shards: "Each shard is basically a Lucene index, which maintains its own TF and DF statistics," and DFS mode exists to compute global statistics ([Elastic, 10 February 2013](https://www.elastic.co/blog/understanding-query-then-fetch-vs-dfs-query-then-fetch)). Their answer, that "enough" data smooths it out, does not hold for a small store.

- Probe: BM25 recomputed in plain SQL from an `fts5vocab` `instance` table and cached per-section token counts reproduced FTS5's own `bm25()` exactly over the whole table (−3.1111 and −5.0727). Restricted to scope A, it returned the scores from before scope B existed (−5.5597 for both).

- On the 50,000-section corpus, with candidates from `MATCH` plus the scope filter, per-term document frequency from `MATCH` per term, and term frequency from the vocab table filtered by term, the scope-local top five took about 18 ms. It needs SQLite's math functions (`ln`), which this build has; the linked library must be probed for them.

- Per-section token counts are computed once at ingest. Counting one section's tokens from the main vocab table took about 105 ms, because `instance` is keyed by term.

- Inserting the section into a one-row temporary FTS5 table with the same tokenizer and counting its vocab rows took under 1 ms.

- Proposal: rank with BM25 computed over the eligible set only (this scope, unscoped records, accepted, open). That also keeps unaccepted, closed, and out-of-scope rows from shifting scores, and removes a ranking side channel across scopes.

- **Small-corpus IDF.** FTS5 floors IDF at 1e−6. Probe: one matching row out of two gives log((2−1+0.5)/(1+0.5)) = 0, and the score was −1.000e−06; out of four it was −0.7456.

- So any term in half the rows or more contributes nothing. Lucene's BM25 uses log(1 + (docCount − docFreq + 0.5)/(docFreq + 0.5)) ([BM25Similarity](https://lucene.apache.org/core/9_11_0/core/org/apache/lucene/search/similarities/BM25Similarity.html)), which stays positive.

- Proposal: with BM25 computed in SQL, use Lucene's IDF. Tiers still come first.

- **Where section text lives. Once, as a view over byte spans of the body.**

- "An external content FTS5 table is created by setting the content option to the name of a table, virtual table or view," and FTS5 reads values with `SELECT ... FROM <content> WHERE <content_rowid> = ?` ([FTS5 external content](https://www.sqlite.org/fts5.html)).

- Probe: `bodies(hash, bytes BLOB)`, `sections(id, body_hash, ordinal, heading, start, len)`, and a view that returns `CAST(substr(bytes, start+1, len) AS TEXT)` as the body and the same text with `-` and `_` replaced as parts. The FTS table used the view as content.

- `integrity-check` with rank 1 passed after insert and again after a forget. Snippets and exact identifier matches worked.

- A forget deleted the index rows by reading the old values from the view, then deleted the section and body rows, with FTS secure-delete on.

- A view cannot carry the triggers, so index maintenance is explicit SQL in the same transaction: insert sections, then insert into the index from the view; read from the view into a `delete`, then delete rows. Offsets are byte offsets into the original bytes, produced by the parser.

- The probe's hand-counted offsets cut a word in half, which is the reason they must not be computed any other way.

- One conflict to decide: the settled rule names a generated parts column. In a view, parts is a computed view column with the same expression.

- If a generated table column is required, section text must be materialized as a disposable cache instead, and the text is stored twice.

- **Snippets for a parts-only hit.** Probe: for `parts:"lease"`, `snippet()` on the body column showed no highlight, and `snippet()` with column −1 chose the parts column and showed `force with [lease]`, which is not the original bytes. Because the parts expression replaces one character with one character, `highlight()` on parts gives marker positions that map onto the body text one to one.

- Probe: `parts:"force with lease"` mapped back to the body span `force-with-lease`. Proposal: when a hit matched only in parts, take the marker offsets from parts and render the snippet from the body text.

- **Counting before the display limit.** Probe: `count(*)` and `count(DISTINCT record)` over all matched rows in one scope took about 0.2 ms on 50,000 sections. Counts over the full matched set cost nothing worth avoiding.

- **What `path:` matches, and relative links.**

- Prior art: GitHub's `path:` matches the result file's own path; Claude Code path rules match a rule's globs against the files a tool touches; an email indexer `path:` is the file's directory; IWE keys a document by its path relative to the workspace root. All sections above.

- Proposal: a record's location set is its repository-relative source path (a document added from inside the checkout), its evidence paths, and its declared globs. A query `path:` glob matches literal paths in that set.

- A declared glob on a record matches when the query names a literal path, the way a Claude Code path rule matches a touched file. `path:` filters records, not sections, because evidence belongs to the envelope.

- [CommonMark 0.31.2](https://spec.commonmark.org/0.31.2/) defines only the syntax of a link destination, keeps `./target.md` unchanged in its example, and says it "does not enforce any particular policy for rendering URLs." Resolution is the store's choice. Proposal: keep each destination exactly as written.

- Resolve relative destinations against the source path as a local projection, as validity is, because the source path is a local observation. Input from stdin keeps its links unresolved.

- A missing target stays a named link, as a linked-memory system does.

- **Still decisions or measurements.** Whether the parts column may be a view column (above). Whether terms may span sibling sections, which both reviews deferred to the real cases.

- Whether to accept relaxation counts on an empty result.

### Section cuts and lexical side columns (2026-11-07)

- Checked against plan 79 on the dev half of `corpus-q4`, and against the retrieval literature on where a section may be cut and what a side column may add. Sentence splitting was set aside.

- The exact `unicode61` index, with `_` and `-` kept inside a token, stays the identifier index.

- **The 1,400-byte pack is not where the doc-passage misses are.** A hit counts only when the card's byte span contains the gold span and stays within `max(3 × gold length, 1,500)` bytes, so a card of ≤ 1,400 bytes always passes the size check. On the 73 dev doc-passage cases, the first query's pages already contain the gold span in 52.

- Of the other 21, 11 never show the record. The remaining 10 show the record under a different heading.

- No gold span is cut by a block boundary, and none is longer than 1,400 bytes. Eight of those ten right headings are themselves small enough to be a legal card.

- The byte range that would cover both the shown heading and the gold heading fits in 1,400 bytes in one case, `s4-c-0018`. The other pairs run from 2,860 to 11,192 bytes.

- A different cut inside the shown heading cannot put that gold on the card. The miss is which heading ranked.

- **Keep the heading cut. Do not prepend the heading path.** On size-matched chunks, moving the boundary onto the document's own sections accounts for most of the dense-retrieval gain.

- Fixed windows with no prefix to structure-aligned chunks with no prefix: +0.041 cov-nDCG@10 on Wikipedia Featured Articles (0.362 to 0.403) and +0.019 on QASPER (0.098 to 0.117). Adding the real heading path on top of that cut adds +0.011 and +0.009.

- An LLM-written prefix on fixed windows scores 0.391 and 0.114, below the bare structural cut. Real paths beat shuffled paths of the same shape by +0.0099 and +0.0161 ([Kuehlkamp, Moreira, and Rund, 7 October 2026](https://arxiv.org/abs/2610.10170), authors' Table 2; the shorter citation in "Storing and retrieving core" is the same paper).

- This store already cuts on headings. The path is a plain column for `under:` and display, not an FTS field, so a path prefix has nothing to score, and its bytes would only push a span toward the size cap.

- [Anthropic's contextual retrieval](https://www.anthropic.com/research/contextual-retrieval) (19 September 2024) measured a model-written prefix of usually 50 to 100 tokens, not a heading breadcrumb. Their failure-rate drops (5.7% to 2.9% with contextual BM25, top 20) require that generated text inside the index.

- The write path here does not do that. They say generic document summaries gave "very limited gains," with no figure.

- **Do not return the parent section, and do not rank sections before cards.** The same paper's route-to-section stage, then chunks only inside the top sections, loses cov-nDCG@10 by 0.0326 on Wikipedia-FA (95% CI [−0.0442, −0.0217]) and 0.0147 on QASPER ([−0.0192, −0.0104]). The first stage kept a gold-bearing section 0.60 to 0.68 of the time on the 200-document corpus and 0.17 to 0.20 on QASPER, where it saw about 22,000 sections.

- A reranker shrank the QASPER gap to 0.0038 and did not reverse it. A parent card also fails the size rule whenever the heading is larger than the cap.

- That is already true for 2 of the 10 wrong-heading misses above. Small-to-big retrieval, as in a child index that shows the parent, both exceeds the cap and uses the embedding to choose the card.

- **Do not add an overlap window.** At a fixed chunk size, overlap did not raise token recall, and it lowered the score that charges for repeated tokens. `RecursiveCharacterTextSplitter` at 400 tokens with no overlap: recall 89.5 ± 29.7, IoU 3.6 ± 3.2.

- The same splitter with 200 tokens of overlap: recall 88.1 ± 31.6, IoU 3.3 ± 2.7. At 800 tokens with 400 of overlap, recall is 85.4 and IoU is 1.5 ([Smith and Troynikov, Chroma, 3 July 2024](https://www.trychroma.com/research/evaluating-chunking)).

- These are their reported means, not a paired test of overlap alone. A second copy of a boundary also spends one of five cards.

- [NVIDIA, 18 June 2025](https://developer.nvidia.com/blog/finding-the-best-chunking-strategy-for-accurate-ai-responses/) found 15% overlap better than 10% and 20% on FinanceBench at 1,024-token chunks, and never compared 15% with none. Their page-level end-to-end accuracy was 0.648 ± 0.107 against token chunking between 0.603 and 0.645.

- A page or a 1,024-token card is larger than 1,500 bytes for ordinary text, so it fails this hit rule for a short gold span.

- **The one structural hole still open is a fence larger than the cap, and it is not this miss.** cAST keeps a syntax node whole when it fits, splits an oversized node into children, then merges adjacent siblings back up to a character budget. Split-without-merge drops nDCG from 71.1 / 75.9 / 85.1 to 53.5 / 59.1 / 66.1 (BGE / GIST / CodeSage).

- Their abstract's Recall@5 gain is 4.3 points on RepoEval; Table 5 is the cell (GIST-base 70.7 to 75.0). Their budget peaks at 2,000 to 2,500 characters, which would fail this size rule for a short gold span ([Zhang et al., Findings of EMNLP 2025](https://arxiv.org/abs/2506.15655)).

- This corpus has no doc-passage gold longer than 1,400 bytes and no gold bisected by a block, so the change does not move this set. List items are already atomic.

- [MDKeyChunker](https://arxiv.org/abs/2603.23533) (Mangla, March 2026) states the same atom for lists, fences, and tables, with a soft maximum of 1,500 characters, on the author's own 30-query set. That sample is too small to justify coarser grouping.

- **A merge that stays inside 1,400 bytes cannot erase a current hit, and it does not reach these misses.** an indexing tool merges a small heading subtree into its parent when reading it whole costs no more than routing through it (150 tokens) and splits only above 8,000 tokens (an indexing tool). Copying that threshold makes a card fail the size rule for any short gold span.

- The README's own 30-question figure, 30 of 30 found and 88.6% of cited lines inside the gold range, is an agent reading line ranges, and the hit rule is overlap, not containment. Weighting merged headings in the FTS body would also put the heading path back into the index.

- Merging the shown heading with the gold heading stays inside 1,400 bytes in 1 of the 10 cases (`s4-c-0018`).

- **Lexical side columns, beside the exact index.** A dictionary lemma column was measured and does not get a missed section over the coverage line: on the 41 gold records no prepared query places in the top 5, an agreed open-class lemma adds one content term on two blocks (`typed`/`type`, and an inflection of `build`), and both stay short of the terms a candidate needs. The ideas below are different columns or a query-time disjunction.

- Identifiers still have to hit the exact index. Fenced code, backtick spans, and URLs are masked before any of them.

- A URL is one token ([Manning, Raghavan, and Schütze, 2008, §2.2](https://nlp.stanford.edu/IR-book/html/htmledition/tokenization-1.html)). Deleting a language's reserved words inside code did not help BM25 once camelCase and snake_case were already split: five of six CodeSearchNet scores fell ([Zhang, Xin, Yates, and Lin, NLP4Prog 2021](https://aclanthology.org/2021.nlp4prog-1.10.pdf)).

- That split is already the parts column.

- **An open-class side column.** Index a token in a second column only when its local part-of-speech block has ≥ as many nouns, verbs, and adjectives as closed-class tags. On WT2G, BM25 mean average precision moved from 0.280 to 0.297.

- On WT10G it moved from 0.234 to 0.233 ([Lioma and Ounis, COLING/ACL 2006](https://doi.org/10.3115/1273073.1273142)). `if`, `for`, and `not` are closed class and also real flags, so this column cannot be the one an identifier has to hit.

- **Head-modifier pairs, not whole noun phrases.** From a noun chunk, index the head with each modifier as an extra phrase beside the single words. On TREC-5 topics 251–300, head-modifier pairs raised initial precision from 0.4546 to 0.5162 and recall from 0.56 to 0.60.

- Stacking full phrases on top of that raised recall to 0.63 and gave most of the precision back ([Zhai, ANLP 1997](https://aclanthology.org/A97-1046.pdf)). The chunker ignores any token with `_`, `-`, or a digit, or it glues `E_INDEX-42` to the noun beside it.

- A chunk head is one of the lemmas already measured, so it cannot match a query word the section does not contain.

- **Discounted inflectional variants at query time.** Leave the index alone. If a dictionary gives one lemma for an ordinary word, search that word's plural, past, and `-ing` forms at a lower weight, and still require the exact token as a conjunct.

- Against an unstemmed AQUAINT index, this raised recall at 100 documents from 0.2720 to 0.2944, while Porter stemming on both sides lowered it to 0.2589. The same weighted expansion worsened total document reciprocal rank from 0.6403 to 0.6094 ([Bilotti, Katz, and Lin, 2004](http://www.cs.cmu.edu/~mbilotti/pubs/Bilotti04a.pdf)).

- Their expansion was every non-stopword, not the unique-lemma subset already measured here.

- **An additive spelling column, whole words only.** When a lexicon lists a British/American pair, add the other spelling beside the original. Do not apply `-ise`/`-ize` as a suffix rule, and do not replace the stored form.

- On the 173 MS MARCO dev queries that use only British spellings, BM25 recall at 1,000 moved from 0.8728 to 0.9306 when the document spelling matched the query, and fell to 0.5809 when the original form was replaced by the opposite convention ([Chari, MacAvaney, and Ounis, SIGIR 2023](https://doi.org/10.1145/3539618.3592030)). `program`/`programme` is a meaning split in software, so a variant hit cannot satisfy an identifier.

### FTS5, BM25, and the vector index (2026-11-07)

- The vector row is the block's own bytes, not a heading and not a synthetic heading. On the dev corpus that is 6,096 block vectors.

- A section starts at its heading line, so the first block of a headed section includes that heading: 4,101 sections have a heading. A later block of the same section does not repeat it: 790 vectors are those continuations.

- Ancestor headings are not prepended. A section that the pack splits is several vectors, not one vector of the whole section.

- The static model in the eval is the same family production would run through model2vec-rs; the eval calls the Python package.

- FTS5 and BM25 already share one lexical search. FTS5 decides which sections contain a term or a phrase, and its vocabulary supplies the counts.

- BM25 orders sections inside a tier (phrase, then every content term, then a partial). The two matchers, exact and Porter, are then interleaved.

- The vector index is not in that order. It runs only when a call has no cards, and only as a hint that must share a rare word.

- It does not add a card.

- Prior art combines the three by rank, on one row id, and it does not agree about letting the vector add a row the lexical search missed.

- sqlite-vec's hybrid note ([Garcia, 2 October 2024](https://alexgarcia.xyz/blog/2024/sqlite-vec-hybrid-search/index.html)) embeds the same text FTS5 indexes and gives three joins. Keyword-first returns the FTS5 hits, then fills with nearest neighbors.

- Reciprocal rank fusion, `1/(60+rank)` from each list with the constant from Cormack, Clarke, and Büttcher (SIGIR 2009), full-outer-joins the two lists, so a vector-only row still ranks. Re-rank by semantics never opens the vector table as a second candidate source: the FTS5 hits are sorted by cosine.

- libSQL's own vector type is a different API from sqlite-vec (`F32_BLOB`, `vector_distance_cos`, `vector_top_k`, a DiskANN index). an external SQLite adapter runs FTS5 BM25 and that libSQL vector search as one statement per mode, and its hybrid score is weighted reciprocal rank fusion, `weight/(60+rank)`, summed.

- A row that matches only one side still ranks. Filters are applied after the index returns candidates, so a selective filter can return fewer than the limit.

- The same fusion, FTS5 `bm25()` plus sqlite-vec cosine plus `1/(60+rank)`, is the local-first pattern in an external retrieval design and in the cartog and mika notes. an external retrieval design's reason for ignoring the raw scores: BM25 is unbounded and cosine sits in a band that moves with the model, so a weight tuned on one corpus does not travel.

- Checked on plan 81, dev half of `corpus-q4`. The adjustment that stays inside "the vector does not add a card" is Garcia's re-rank: the top 20 records of each matcher, after the other-repository penalty, sorted by the cosine of the block vector, five kept.

- Gold falls from 399/439 and 316/439 to 380/439 and 287/439. Cross-scope rises from 30/42 and 23/42 to 33/42 and 29/42.

- No-gold rises from 35/51 to 36/51. Reciprocal rank fusion of those same two ranks, with no vector-only row, falls to 394/439 and 307/439 and rises to 32/42 and 26/42.

- Applying plan 81's short-page fill after the cosine order reaches 382/439 and 291/439. The vector is preferring a semantically close document from another repository and pushing the caller's gold out of the five.

- That is the cross-scope gain and the gold loss together. Plan 81 is unchanged.

### Sorting the two lists, and what a card shows (2026-11-07)

- Two questions sit on that measurement. One is how a lexical list and a vector list should be sorted together when the vector is the weaker list.

- The other is whether those same vectors can write a better card. The card writer has not been part of plans 79 to 81.

- Those plans choose which block is shown. The text on the card is older, and the fill path does not use the same writer as the ranked path.

- **A convex combination beats reciprocal rank fusion when the dense model is the stronger list, and it drops below the lexical list when it is not.** Bruch, Gai, and Ingber ([TOIS 2023](https://arxiv.org/abs/2210.11934)) fuse BM25 with a trained dense retriever. Each list is normalized, then mixed as α times the semantic score plus (1 − α) times the lexical score.

- With α fixed at 0.8, nDCG@1000 on MS MARCO is 0.454, against 0.441 for the dense list alone, 0.425 for reciprocal rank fusion with offset 60, and 0.309 for BM25. On HotpotQA the lexical list is the stronger one, 0.682 against 0.520, and fusion with offset 60 scores 0.675, under BM25.

- Which bounded normalization is used matters less than that both scores are bounded, because a change of scale can be absorbed into α. Their dense models are trained retrievers.

- The vector here is a static mean of token vectors ([Minish, 14 October 2024](https://minish.ai/blog/2024-10-14-hf-blogpost/)): one vector per block, no word order, and a token has the same vector in every block. Scoring the unpooled static token vectors with late interaction was worse than the mean, 0.418 against 0.504 mean nDCG@10 on NanoBEIR (an external static-retrieval study).

- The dev reorder already put a semantically close other repository ahead of the caller's gold. An α fit on that dev set would be fit to the same displacement.

- The lexical tier stays the sort.

- **Ranked cards are still plan 64's line. Fill cards are a different window, and neither window asks the vector.** Inside the chosen block, the line with the most distinct query words is kept.

- If that line is longer than 160 characters, the window is centred on the first of those words. Cards added by the short-page fill skip that writer.

- They are 180 characters of the block, starting 50 characters before the earliest query term (`section_card` in the stemmed matcher). The stored vector cannot choose between two lines of one block, because it is one mean of the whole block.

- A line is not a row. Encoding each line at read time builds a new set of vectors; it does not look up a row the index already holds.

- The rows that do exist are consulted only on a silent call, as a hint that must share a rare word. Using them to sort a nonempty page was measured above and lost gold.

- **A query-biased extract beats the opening of the document. A semantic sentence pick is a different job from the line this agent uses.** Tombros and Sanderson ([SIGIR 1998](https://doi.org/10.1145/290941.290947)) score a sentence by the square of its query-term count, divided by the count of query terms, added to ordinary extractive features, and keep up to five sentences or 15% of the document.

- Subjects identified 15.84% more of the relevant documents they examined, looked at 22.62 documents against 20, and opened the full text for 1.32% of examined documents against 23.7%. FTS5 `snippet()` is the same idea in a window of ≤ 64 tokens: it maximizes the count of distinct query terms, with extra weight at the start of the column and after `.` or `:`.

- Plan 64 is that idea cut to one line and 160 characters. Plans 65 to 68 already replaced the line with a sentence, a BM25 sentence, key terms, and a window across lines.

- None beat the centred line. The sentence that contained the gold quote more often (58% against 45% on dev, 36% against 27% on h2) made the agent choose the section less often.

- **Cosine over sentences helps when it may change which document the sentence comes from, and it does not separate from word overlap when the document is already chosen.** On BioASQ9 synergy snippet F1 ([Mollá, CLEF 2022](https://arxiv.org/abs/2209.01815)), sentence-BERT with a global sort scored 0.1923 in round 1 against 0.1100 for tf–idf with the same global sort. Restricted to the top three sentences of each document, the same cosine was 0.1071 against 0.1031.

- By round 4 the four runs sit between 0.0619 and 0.0764. A cross-encoder that scores the query jointly with each sentence is another model: on an English snippet set, precision at 1 is 33.91 for BM25, 40.24 for a shallow semantic matcher, and 77.05 for DeepQSE ([Yi et al., EMNLP 2022](https://aclanthology.org/2022.emnlp-main.197.pdf)).

- That model is not this index. Encoding each line of an already chosen block with the static model can prefer a line that shares no query word.

- That hides the match the centred line was kept to show. The vector stays off the card.

- The open difference in the writer is the fill path's 180-character window, which is lexical, and it has not been put back through plan 64's line.

- **Untested at the time of writing: a line or sentence vector, written once at ingest, used only to choose the excerpt.** Measured below. After the block split, each non-empty line of a block, or each sentence from the same cut plan 65 used, can be embedded with the static model and stored under that block.

- Card creation then compares the query vector with those rows and keeps the nearest one. The query is still encoded when it is asked.

- The lines are not encoded again. This does not change the retrieval unit.

- The block stays the ranked row, the card span stays the block, and a line vector does not add a card, reorder the five, or break a silence. Sentence leaves as ranked sections stay set aside.

- The selection rule that matches the agent runs is narrower than a free nearest sentence: the line has to contain a query word, and the cosine only breaks ties among those lines, or picks the nearest line that does. A nearest line with no query word repeats the plan 65 result, where the excerpt held the gold quote more often and the agent chose the section less often.

- Ranked cards and fill cards would use the same rows, so the fill path's 180-character window would go through this writer too.

- **Checked on the dev half of `corpus-q4`, then the one rule that cleared those floors on all of `corpus-h2`.** Plan 81 stays 399/439 and 316/439, cross-scope 30/42 and 23/42, no-gold 35/51, section 55/73. The dev corpus has 44,118 non-heading lines and 55,137 sentences under the 6,096 block vectors.

| Rule | Gold any | Gold first | Cross-scope | No-gold | Section |
|---|---|---|---|---|---|
| Plan 81 | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Cosine only to break an equal BM25 score inside a tier | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Reciprocal rank fusion inside a tier, offset 60 | 400/439 | 313/439 | 34/42, 27/42 | 38/51 | 56/73 |
| That fusion on the phrase tier only | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Convex mix inside a tier, semantic weight 0.2 | 401/439 | 316/439 | 31/42, 24/42 | 36/51 | 55/73 |
| Fill block chosen by its nearest query-word line | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |

- The small floors are not the mixes leaving the page alone. On 1,470 dev queries the fused five changes its record set 674 times under the 0.2 mix and 923 times under within-tier fusion.

- The first card changes 113 and 192 times. Almost every record that leaves is a tier-2 partial: 338 of 367 under the mix, 860 of 927 under fusion.

- Gold enters or leaves that five on 21 and 66 of the 1,317 queries that have a gold. The other substitutions replace one non-gold partial with another, and the floor only counts whether the gold record is somewhere in the five.

- A tier of two records cannot be reordered by a semantic weight of 0.2. Min-max maps those two scores to 1 and 0 whenever they differ, so the lexical term contributes 0.8 and the cosine can contribute ≤ 0.2.

- Of 494 such tiers, 2 change order, which is the equal-score case the tie-break was aimed at. Tiers 0 and 1 have median size 2, and 0.42 and 0.44 of them have a single record.

- Tier 2, the partials, has median size 15 and is where the page actually moves: 8,432 of the matcher slots in a top five are tier 2, against 320 in the phrase tier. The mix is free to reshuffle those partials and almost never reaches the gold.

- The tie-break never moves the top five: after the penalty, BM25 scores inside a tier do not tie in a way that changes it. Fusion inside a tier raises gold any and drops gold first, with three gold-any losses (`s2b-c-0031`, `s4-c-0005`, `s5-c-0112`).

- The phrase-only fusion matches plan 81. Choosing the fill block by line cosine matches plan 81 on every floor.

- The convex mix is the one dev gain. The semantic weight 0.2 is the complement of the 0.8 Bruch, Gai, and Ingber used when the dense model was the stronger list.

- It was fixed before the run. Gold any rises by `s3a-c-0034` and `s3a-c-0054`, and no gold-any case is lost.

- Gold first stays 316 because three first-query golds move in and three move out (`s3a-c-0015`, `s4-c-0048`, `s4-c-0105` move out). On h2 the same weight, not retuned, raises gold from 335/354 and 264/354 to 336/354 and 267/354 and section from 230/271 to 232/271, and drops cross-scope from 56/62 and 46/62 to 53/62 and 45/62 and no-gold from 54/106 to 53/106.

- That is the same displacement as the unconstrained cosine reorder: more of the caller's gold, fewer of the other repository's targets. Plan 81 stays the plan.

- **The line index does not raise the excerpt proxy either.** On the 121 first-page gold cards that carry a quote, plan 81's current text contains the quote in 38. Passing every card through the centred line contains it in 37.

- The nearest line that contains a query word contains it in 36, the nearest such sentence in 26, and the nearest line with no such requirement in 33. The span is unchanged, so the retrieval floors do not move.

- A combination of the mix with this excerpt was not run on h2: the excerpt does not change which records are shown, and the mix already failed the holdout on its own.

### Where plan 81 misses (2026-11-07)

- These runs have no agent in the loop. A miss is the deterministic check: the gold record is not in the five, the card's byte span does not contain the gold span, a silent case came back with cards, or a named other-repository record was not shown.

- No checked case has a quote or a heading that is missing from its record, so the check is not rejecting a card that already contains the gold.

- The misses are one lexical fact, in four places.

- **The gold record never shares enough of the query's words.** On dev, 40 of 439 gold cases miss every prepared query. 14 share some words but not enough to clear coverage, 8 share none, 4 are missing a hard identifier (`RACE_CONDITION`, `allow-fs-read`), and 1 shares enough words and still is not ranked. 13 more are ranked and sit outside the five, almost all as tier-2 partials (ranks 6, 7, 9, 10, and 13 and beyond). On h2 the same split is 19 misses: 12 never ranked, 7 ranked outside the five.

- **The first wording misses and a prepared alternate hits.** 83 dev cases and 71 h2 cases are in the five for a later prepared query and not for the first. That is the same gap, closed by a second phrasing.

- The "first" floor counts only the first wording.

- **The record is shown under a different heading.** On the first page, 19 dev doc-passage cases and 53 h2 cases show the gold record and the card span does not contain the gold span. The shown heading is a sibling (`Environment variables in Expo Application` for a gold heading `How to read from environment variables`).

- A later call repairs some of these; the scored section floors are 55/73 on dev and 230/271 on h2. Six cases on each corpus never show the record at all.

- **Silence fails because some other record does share the words.** Dev no-gold misses are 7 true-empty cases that returned cards, 3 that returned a same-scope forbidden record, and 8 that should have shown a named other-repository record and did not. Those 8 are the cross-scope misses, not a second bug: the other-repository record is a tier-2 partial at rank 6 to 85, or it shares too few words.

- H2 is the same shape and larger on the true-empty side: 33 cases return a real overlap, 12 return a forbidden same-scope record, 7 miss the other-repository target.

- The mix and the within-tier fusion do not create a new kind of miss. They swap tier-2 partials near the edge of the five.

- On dev the 0.2 mix drops three first-query golds and one cross-scope target, and picks up the same number. On h2 it raises gold and drops the other-repository targets the empty and cross-scope checks were waiting for.

- **Ten further rules, aimed at those misses.** Of the 40 dev golds that never enter the five, 24 already have a full page, 9 have an empty page, and 7 have room. Thirty of the 40 do not contain the rarest query word.

- Rules that promote the rarest word, or that replace the last card, lose a gold or a cross-scope target. Rules that only add a card, and only on a short nonempty page, can raise a floor without removing one.

| Rule | Gold any | Gold first | Cross-scope | No-gold | Section |
|---|---|---|---|---|---|
| Plan 81 | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |
| One record per page, backfill the next hit | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Last card yields to the next hit with the rarest word | 398/439 | 314/439 | 29/42, 23/42 | 34/51 | 55/73 |
| Short page gains that rare-word hit | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Shown record switches to the section with the rarest word | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Rare-word replace only when the last card is a partial | 399/439 | 315/439 | 29/42, 23/42 | 34/51 | 55/73 |
| Last card yields to a record that shares more query words | 397/439 | 311/439 | 27/42, 22/42 | 34/51 | 55/73 |
| Short page gains the record one word under the line | 400/439 | 318/439 | 30/42, 24/42 | 35/51 | 55/73 |
| Short page gains the record holding every word but the most common | 400/439 | 317/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Shown record switches to the section with the most query words | 399/439 | 316/439 | 30/42, 23/42 | 35/51 | 53/73 |
| Drop the most common word, then replace the last card | 397/439 | 311/439 | 27/42, 22/42 | 34/51 | 55/73 |

- The one-word-under rule is plan 83. On dev it adds `s5-c-0102` and no gold-any case is lost.

- Gold first rises by two, cross-scope first by one. On h2 the same rule matches plan 81 on every floor: 335/354 and 264/354, cross-scope 56/62 and 46/62, no-gold 54/106, section 230/271.

- The dev gain does not appear on the holdout, and the holdout does not lose a floor. Switching the section to the one with the most query words drops section from 55/73 to 53/73.

- The ranked section was the right span more often than the word-count section.

- **The same sorts and the three Garcia joins, each crossed with the fill, then plan 83's append.** Eight orders: the lexical order, cosine only on an equal BM25 score, reciprocal rank fusion inside a tier, that fusion on the phrase tier only, the 0.2 convex mix inside a tier, Garcia's cosine reorder of each matcher's top 20, reciprocal rank fusion of that same pool, and a full-outer fusion that can rank a block the lexical list missed. Four fills: the heading-tree fill, that fill's block chosen by its nearest query-word line, nearest-neighbor blocks in the leftover slots, and no fill.

- The one-word append runs last, and an empty page stays empty. The lexical order with the heading-tree fill is plan 83.

- It reproduced on both corpora, and the reconstructed lexical page agreed with the search on every query (1,470 dev, 1,380 h2). Tie-break and phrase-only fusion match that page on every floor.

- Choosing the fill block by line cosine matches it on dev and drops one h2 section hit, 230/271 to 229/271.

- Dev, against plan 83's 400/439 and 318/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73:

| Order, then fill | Gold any | Gold first | Cross-scope | No-gold | Section |
|---|---|---|---|---|---|
| Lexical, heading-tree | 400/439 | 318/439 | 30/42, 24/42 | 35/51 | 55/73 |
| Lexical, nearest blocks in the leftover slots | 401/439 | 324/439 | 32/42, 25/42 | 36/51 | 55/73 |
| Lexical, no fill | 399/439 | 315/439 | 30/42, 24/42 | 35/51 | 55/73 |
| Within-tier fusion, heading-tree | 401/439 | 315/439 | 34/42, 28/42 | 38/51 | 56/73 |
| Within-tier fusion, nearest blocks | 401/439 | 321/439 | 36/42, 29/42 | 39/51 | 56/73 |
| Convex mix, heading-tree | 402/439 | 318/439 | 32/42, 25/42 | 37/51 | 55/73 |
| Convex mix, nearest blocks | 402/439 | 324/439 | 34/42, 26/42 | 38/51 | 55/73 |
| Cosine reorder of the top 20, heading-tree | 383/439 | 293/439 | 34/42, 30/42 | 37/51 | 53/73 |
| Inner fusion of that top 20, heading-tree | 396/439 | 313/439 | 32/42, 27/42 | 36/51 | 54/73 |
| Full-outer fusion, any fill | 392/439 | 329/439 | 32/42, 28/42 | 34/51 | 52/73 |

- The nearest-block fill is Garcia's keyword-first join: the lexical page keeps its hits, and a short page takes the nearest indexed blocks until it is full. The heading-tree cards are not on that page.

- On dev this raises every floor that plan 83 had room to raise, and it drops `s5-c-0074`, which the heading-tree fill had been carrying. The convex mix with the heading-tree fill and the append is plan 82's reorder plus plan 83.

- Gold any rises by two and gold first stays 318. Both of those clear the dev floors.

- The cosine reorder, the inner fusion, and the full-outer fusion still lose gold any. Full-outer raises gold first to 329 and drops no-gold to 34, because a vector-only block is a card.

- h2, the same frozen rules, against plan 83's 335/354 and 264/354, cross-scope 56/62 and 46/62, no-gold 54/106, section 230/271. The nearest-block fill reaches 335/354 and 270/354, cross-scope 57/62 and 47/62, no-gold 55/106, and section 229/271.

- Section falls by one. The convex mix with the heading-tree fill reaches 336/354 and 267/354 and section 232/271, and cross-scope falls to 53/62 and 45/62 and no-gold to 53/106.

- That is plan 82's holdout failure with the append left on. The mix plus nearest blocks reaches 336/354 and 273/354 with section 231/271, and cross-scope any falls to 54/62.

- Within-tier fusion drops gold any to 333/354. Cosine reorder drops gold any to 318/354.

- Full-outer fusion drops gold any to 327/354 and no-gold to 52/106, and raises gold first to 293/354. No permutation holds every holdout floor.

- The excerpt rewrite still leaves the block span alone, so it does not move these floors. On the plan 83 page the gold quote is present in 38 of 121 dev cards and 57 of 232 h2 cards.

- The centred line is 37 and 57. The nearest query-word line is 36 and 59.

- The nearest query-word sentence is 26 and 51. The nearest line with no query-word requirement is 33 and 55.

- Plan 83 stays the plan for every reorder and every card rewrite.

### The admission gate, applied to the search (2026-11-07)

- The reorder experiments shuffle records the word search has already kept. Two of the rules that helped on a short page are admission rules, and a full page never consults them.

- A section one word under the coverage line is not ranked. A record whose heading and children cover the query is not ranked either.

- Both can enter the ranking as partials: behind a phrase hit and behind an all-terms hit, ordered by BM25, with the other-repository penalty. Hard terms stay required.

- This run did not add a neighbor. The line that a vector hit never becomes a card is about the text on the card: the excerpt stays a span of the record.

- The widened gate runs only when that matcher already returned a section, so a silent query stays silent.

- Dev, against plan 83. The confirmed scorer matches these rows.

| What the search newly admits | Gold any | Gold first | Cross-scope | No-gold | Section |
|---|---|---|---|---|---|
| Plan 83 | 400/439 | 318/439 | 30/42, 24/42 | 35/51 | 55/73 |
| Every section one word under | 408/439 | 334/439 | 29/42, 23/42 | 35/51 | 55/73 |
| The one best section one word under | 405/439 | 330/439 | 30/42, 24/42 | 35/51 | 55/73 |
| Every section one word under, including a silent search | 412/439 | 337/439 | 29/42, 23/42 | 35/51 | 55/73 |
| Sections that meet the line once the most common word is ignored | 404/439 | 324/439 | 30/42, 24/42 | 35/51 | 55/73 |
| Heading trees that cover the query | 400/439 | 319/439 | 30/42, 24/42 | 35/51 | 55/73 |
| Heading trees one word under | 408/439 | 336/439 | 30/42, 23/42 | 35/51 | 55/73 |
| Every near section, plus covering headings | 408/439 | 334/439 | 29/42, 23/42 | 35/51 | 55/73 |
| The one best near section, plus covering headings | 406/439 | 331/439 | 30/42, 24/42 | 35/51 | 55/73 |

- Admitting every near section raises gold and drops cross-scope. Doing it on an empty search does the same.

- A heading tree one word under drops cross-scope first. The one best near section, the common-word skip, covering headings, and the one best near section together with covering headings hold every floor.

- The last of those is the strongest on dev.

- h2, those four rules unchanged. Plan 83 is 335/354 and 264/354, cross-scope 56/62 and 46/62, no-gold 54/106, section 230/271.

| Rule | Gold any | Gold first | Cross-scope | No-gold | Section |
|---|---|---|---|---|---|
| The one best section one word under | 338/354 | 270/354 | 56/62, 48/62 | 55/106 | 231/271 |
| The most common word ignored | 336/354 | 273/354 | 56/62, 46/62 | 54/106 | 230/271 |
| Heading trees that cover the query | 338/354 | 275/354 | 56/62, 46/62 | 54/106 | 232/271 |
| The one best near section, plus covering headings | 340/354 | 278/354 | 56/62, 48/62 | 55/106 | 233/271 |

- All four hold the holdout. The combined rule is plan 84.

- It is the strongest on both corpora. The other three also beat plan 83 and are smaller changes.

- Admitting every near section, or admitting one on a silent query, drops cross-scope on the holdout (50/62 and 42/62, no-gold 52/106).

- **Vector search and keywords made at ingest, on plan 84.** The static model's nearest vocabulary words are a poor extra index. For a block they mix words already in the block with unrelated neighbors (`debian`, `ansible` beside a note about an env file).

- Keeping the five nearest corpus words that are absent from the block, and searching those, finds the missed gold on 2 of 108 dev first pages and on 0 of 76 holdout first pages. Swapping one of those records onto a full page drops gold and cross-scope on both corpora.

- The five words of the block closest to its own vector are real keywords, and they are already in the text. Appending the record whose keywords cover the most query words raises gold first by one on both corpora and leaves the other floors where plan 84 had them.

- Using those keywords to choose the block inside a record drops section from 55/73 to 44/73 on dev and from 233/271 to 199/271 on the holdout. Using the keyword to choose the line holds the gold quote in 18 of 124 dev cards and 16 of 245 holdout cards, against 32 and 27 for the text plan 84 already shows.

- The neighbor rule that the hints already use does move the floors when it adds one card. The page must already have a card and still have room.

- The added card is the nearest block that shares a query word of IDF ≥ 2.5, which is the hint threshold, and the text is that block. Dev: gold 406/439 and 332/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.

- Plan 84 was 406/439 and 331/439. h2: gold 340/354 and 282/354, cross-scope 57/62 and 49/62, no-gold 56/106, section 233/271. Plan 84 was 340/354 and 278/354, cross-scope 56/62 and 48/62, no-gold 55/106.

- That is plan 85. Showing the same neighbor on an empty page raises gold first to 341/439 and 295/354 and drops holdout no-gold from 55/106 to 53/106.

- Replacing the last card of a full page with that neighbor drops gold first on both corpora.

- With shared search on, the neighbor is added to 68 dev first pages and 83 holdout first pages, always in an empty slot (page length 1–4). On dev the added record is the gold once (`s5-c-0027`, the word `revocation`) and a distractor the other 67 times.

- That one card is the whole first-hit rise. On the holdout the added record is the gold four times (`h2a2-c-0024`, `h2a3-c-0040`, `s7-c-0052`, `s7-c-0124`) and a cross-scope target twice (`h2a1-c-0102`, `h2a2-c-0090`).

- The other 77 are distractors. The shared word is one specific token, median IDF 4.58 on dev and 4.56 on the holdout.

- The full neutral eval, fixed agent, same thresholds, against plan 84 on the same corpora. Reports: `runs/tmp/eval85-dev`, `runs/tmp/eval85-h2`, and the plan 84 pair beside them.

- Q2 through Q6, Q4, R1, and the token and latency gates pass for both plans and do not move by a rate the table records, except hit@5, which gains 5 dev cases and 2 holdout cases with no loss. Q1 stays 35/51 on dev and rises from 55/106 to 56/106 on the holdout.

- G1 eligibility rises from 24 to 30 cases on dev and from 23 to 32 on the holdout. G2 must-not rises from 11 to 12 and from 24 to 26.

- The new violations are the neighbor: the vector index still holds closed, superseded, and candidate blocks, and this walk does not apply the filter the word search applies. Both plans already fail G1, G2, Q1, and R2.

- No passing gate starts failing. Median tokens per call rise from 169 to 178 on dev and from 177 to 184 on the holdout.

- Latency p95 rises from 13.0 ms to 14.6 ms on dev and from 12.2 ms to 13.0 ms on the holdout.

- **External hybrid system against plan 85.** The archived external baseline version, the `hybridQuery` path in `src/store.ts`, on the eligible dev notes. The index is every accepted, non-closed, non-superseded, non-stale record, one file per record, body only.

- The case list is the 53 dev cases whose gold and must-not records all sit in shards `s1` and `s1b`: 48 gold, 4 cross-scope, 5 empty. The cases were scored twice, once with only those 90 notes indexed and once with all 1,243 eligible notes indexed.

- Plan 85 was built on the same record set each time. Lists: `runs/tmp/external-baseline-cmp/`.

- The path, from the current tree. A 1.7B expansion model (`query-expansion model (1.7B parameters)`, Q4) is prompted `/no_think Expand this search query:` and constrained to lines `lex:`, `vec:`, or `hyde:`.

- A line is kept only when it still contains an original query term. Temperature is 0.7, not greedy.

- If the top keyword score is ≥ 0.85 and leads the second by ≥ 0.15, expansion is skipped. The original query is always run as keyword and, after embedding, as a vector.

- `lex` lines search FTS only. `vec` and `hyde` lines search vectors only.

- HyDE is a hypothetical passage, not a question. Reciprocal rank fusion uses k = 60.

- The original keyword list and the original vector list are weighted 2. A list that came from an expansion is weighted 1.

- A document that is rank 1 on any list gets +0.05, and rank 2 or 3 gets +0.02. The fused top 40 are the rerank candidates.

- Each document is cut into chunks and the chunk with the most query-term overlap is the one scored. The reranker is Qwen3-Reranker-0.6B through node-llama-cpp `rankAll`, not a yes/no prompt in this repo.

- The blend protects the retrieval order: ranks 1–3 stay 75% retrieval and 25% reranker, ranks 4–10 are 60/40, and rank 11 on is 40/60. The index embedding is EmbeddingGemma 300M, a different model from the static block vectors plan 85 uses.

- On this machine a hybrid query took 2.5 seconds median and 5.0 seconds at the 95th percentile once all 1,243 notes were embedded. Embedding those notes took 36 seconds.

- Overall, below, is the unweighted mean of six rates: gold on some wording, gold on the first wording, first-query MRR, cross-scope on some wording, cross-scope on the first wording, and empty-correct. Cross-scope has 4 cases and empty-correct has 5, so one miss moves that rate by 25 or 20 points.

| | Plan 85, 90 notes | Plan 85, 1,243 | Hybrid, 90 | Hybrid, 1,243 | Keyword, 1,243 |
|---|---|---|---|---|---|
| Gold, some wording | 95.8% (46/48) | 93.8% (45/48) | 97.9% (47/48) | 95.8% (46/48) | 66.7% (32/48) |
| Gold, first wording | 79.2% (38/48) | 72.9% (35/48) | 97.9% (47/48) | 83.3% (40/48) | 43.8% (21/48) |
| First-query MRR | 59.3% | 50.2% | 64.5% | 63.1% | 35.2% |
| Cross-scope | 4/4 | 4/4 | 4/4 | 4/4 | 2/4 |
| Empty-correct | 4/5 | 4/5 | 4/5 | 4/5 | 2/5 |
| Overall | 85.7% | 82.8% | 90.1% | 87.0% | 47.6% |

- On the full index, hybrid and plan 85 each lose one gold-any case when the other 1,153 notes are added. Hybrid loses 7 first-page hits (47 to 40) and plan 85 loses 3 (38 to 35).

- Hybrid's MRR barely moves (64.5 to 63.1). Plan 85's falls (59.3 to 50.2).

- The keyword column is the same 1,243 files with no expansion and no reranker. This is not the 439-case floor, and the holdout was not run through hybrid.

- The direction is the same on both index sizes: hybrid leads, keyword trails plan 85.

- **The same slice against plan 90 does not move.** Same 53 cases, same saved hybrid lists, same scorer. Plan 90 was built twice: on the 90 notes, and on the 1,243 eligible notes.

- Every case matches plan 85. The 90-note page is 46/48 and 38/48, MRR 59.3, cross-scope 4/4 and 4/4, empty 4/5, overall 85.7.

- The 1,243-note page is 45/48 and 35/48, MRR 50.2, the same cross-scope and empty counts, overall 82.8. Hybrid stays 47/48 and 47/48 (overall 90.1) on the small index and 46/48 and 40/48 (overall 87.0) on the large one.

- The plans between 85 and 90 gained their first pages on other shards. None of those queries is in this slice.

- On the 1,243-note index the first page still splits 30 both, 10 hybrid only, 5 plan 90 only, 3 neither. Plan 90's five are `s1-c-0002`, `s1-c-0006`, `s1b-c-0006`, `s1b-c-0010`, and `s1b-c-0038`.

- Gold on any wording is 44 both, hybrid only on `s1-c-0003` and `s1b-c-0059`, plan 90 only on `s1b-c-0006`, and neither on `s1b-c-0037`. The first-wording gap is still about 10 points (40/48 against 35/48).

- It is still the expansion, EmbeddingGemma, and the reranker together. Not adopted.

- **A second slice, shard `s5`, on the same 1,243-note index.** The 91 dev cases whose gold and must-not records all sit in `s5`: 81 gold, 10 cross-scope, 10 empty. Disjoint from the first slice.

- Hybrid was run again through the same `hybridQuery` path (273 queries, median 2.7 s, p95 5.4 s). Plan 90 was the build already scored on those 1,243 notes.

- Lists: `runs/tmp/external-baseline-cmp/hits-q4-hybrid-s5.jsonl`, `plan90-dev-elig.jsonl`.

| | Plan 85 | Plan 90 | Hybrid |
|---|---|---|---|
| Gold, some wording | 75/81 | 76/81 | 78/81 |
| Gold, first wording | 63/81 | 66/81 | 64/81 |
| First-query MRR | 53.4% | 55.6% | 56.2% |
| Cross-scope | 7/10 and 5/10 | 7/10 and 5/10 | 9/10 and 7/10 |
| Empty-correct | 9/10 | 9/10 | 9/10 |
| Overall | 72.3% | 73.5% | 80.3% |

- Plan 90's four gains over plan 85 on this shard are `s5-c-0044`, `s5-c-0051`, `s5-c-0085`, and `s5-c-0098`. Hybrid hits all four.

- On the first wording it also places `s5-c-0051`, where plan 90's first page is empty and a later wording is what scores. First pages split 55 both, 11 plan 90 only, 9 hybrid only, 6 neither, so plan 90 leads that one rate by two cases.

- Gold on any wording is 74 both, hybrid only on `s5-c-0004`, `s5-c-0041`, `s5-c-0090`, and `s5-c-0134`, plan 90 only on `s5-c-0074` and `s5-c-0142`, and neither on `s5-c-0141`. The overall gap is the cross-scope pair: hybrid adds `s5-c-0081` and `s5-c-0118`, and it is first on two more.

- Empty-correct is the same case for both. The stack stays out.

- What of that stack is already a rejected plan. Fusing a vector list into the page, including a vector-only row, lost gold-any and no-gold.

- Reordering the lexical top 20 by cosine lost gold. A within-tier mix with cosine weight 0.2 cleared dev and then lost holdout cross-scope and no-gold.

- Reciprocal rank fusion of the two lexical matchers on the same query did not beat the lexical order. Choosing the shown block by in-text keyword overlap cut section from 55/73 to 44/73 on dev.

- Replacing the last card of a full page, or filling an empty page, dropped a floor. Those runs used the static block vectors.

- They did not use EmbeddingGemma, a generative expansion, or a cross-encoder.

- Three ideas in the hybrid path are not those experiments.

- Extra lexical queries, fused under the original. The expansion's `lex` lines are further keyword queries, and the original list counts twice.

- Plan 85 already issues the corpus's prepared alternate wordings as later calls. It does not rewrite the first wording and fuse the rewrites into that same page.

- A rewrite that is not a model, hyphen splits and stems and dropping one term, is the only piece that stays inside the lexical store and aims at the rate we lost, first-wording gold. The measurement is the plan 86 section below.

- It moves plan 85's own first-page count by one on dev and two on holdout. It does not close the hybrid gap.

- The measured 10-point first-page gap appeared only when the generative expansion, the other embedding, and the reranker ran together.

- Protect the top of the lexical page and let a second score move only the tail. Ranks 1–3 keep three quarters of their retrieval rank.

- On a page of five, that leaves ranks 4 and 5. Freezing the top three and letting the static cosine reorder only those two slots is narrower than the cosine reorder of the top 20, which lost gold.

- It is also close to the last-card swap, which lost gold first on both corpora. It is not the source of hybrid's lead.

- The reranker sees 40 candidates and can promote a document the keyword list had ranked past five. Without that model, tail-only cosine is a small reorder of cards the lexical list already almost showed.

- The models themselves. EmbeddingGemma at index time, the 1.7B expansion, and the 0.6B reranker are why hybrid scores 87.0% against plan 85's 82.8% on this slice.

- HyDE vectors live in EmbeddingGemma's space. They cannot be dotted with the static block vectors.

- Taking this stack means a second embedding of every block, two more models on the query, and seconds instead of the 15 ms plan 85 spends. The earlier decision to leave those paths stands.

- The slice is the cost of that decision: about 4 overall points, and about 10 points on the first wording, on 48 gold cases.

- **The ten first-page losses are not a missing stemmer, and the vector is not allowed to repair them.** Checked against Porter, against the nlprule English lemma table (`runs/tmp/lemmatization-en.txt`, lemma then form), and against the rules plan 85 already runs. Matcher B already stems an ordinary word with Porter.

- A token with `_`, `-`, or adjacent camelCase is an identifier and does not go through that table or the parts column. The lemma column measured earlier, on the 41 gold records no prepared query places in the top 5, adds one content term on two blocks and leaves both short of coverage.

- Plan 43's hypothesis was the same inflection gap, and the stem column is what plan 85 inherited. Guarded stems that restore silence also give the recall back (plan 45).

- Porter and that lemma table agree on the inflections these notes actually contain. `commands` and `command` share a stem.

- `memories` and `memory` share a stem and a lemma. `authored` and `author` share both.

- `ordering` stems to `order`, and the lemma table also sends `ordering` to `order`. None of that is missing from plan 85 when the query word is ordinary.

- The pairs a lemma is often asked to join, and does not: `expire` and `expiry` (`expir` against `expiri`; the dictionary keeps them as two lemmas), `unread` and `read`, `trailers` and `attribution`, `pinned` and the Nix flake, `size` and `characters`.

- What the ten losses actually are:

- Five first pages are empty, so the one-word-short fill and the rare-word neighbor never run. Both require a page that already has a card.

- `s1-c-0001`, `s1b-c-0007`, `s1b-c-0018`, `s1b-c-0025`, `s1b-c-0050`.

- Eight of the ten are recovered by a later prepared wording that drops the extra words. The first wording is the loss.

- Two are never recovered. `s1-c-0003` asks for pinned tools; the note says the Rust commands run through the Nix flake.

- `s1b-c-0059` asks to expire unread memories; the note says expiry by last read is rejected. Those are the only gold-any losses.

- One is a hyphen, and hybrid won it with expansion skipped. `s1b-c-0018` queries `prior-art`.

- The note says "prior art". Plan 85 keeps `prior-art` as one exact token, nothing matches, and the page stays empty. the external system's keyword query splits one separator into the phrase of its parts.

- The vector does not recover any of the ten, and the reason is the rule rather than the embedding. An empty page does not consult it.

- A full page does not let it add or replace a card. The neighbor that does run demands a shared query word at IDF ≥ 2.5, and that word is an exact token, not a stem and not a nearest vector.

- `memories` does not count as `memory` there. Letting the static vector add a row the lexical list missed, or reorder the lexical top 20, already lost gold and no-gold.

- The gate is what keeps that from happening. It is also why an embedding, which does not need a shared stem, never gets to answer these paraphrases.

- **The eight later wordings are not a bug.** Gold-any already counts a hit on any of the three prepared queries. Gold-first is the first wording only.

- The store never sees `alt_queries`. Reading them inside the plan would be scoring the answer key.

- The first wording is the thing to change, by how its tokens are cut, and only when that change holds both floors.

- **Opening the empty page does not hold.** Same scorer as confirm80, plan 85 as the baseline (dev 406/439 and 332/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73; holdout 340/354 and 282/354, 57/62 and 49/62, 56/106, 233/271).

- The rare-word neighbor on an empty page: dev gold 408/439 and 342/439, cross-scope and no-gold and section unchanged. Holdout gold-any stays 340/354, first rises to 299/354, cross-scope first to 52/62, section 234/271, and no-gold falls 56/106 to 54/106 (`h2a2-c-0032`, `s7-c-0060`). Not adopted.

- The one-word-short fill on an empty page: dev gold 407/439 and 335/439, section 56/73, the other floors unchanged. Holdout first 289/354, cross-scope first 50/62, section 234/271, no-gold 55/106. Not adopted.

- A stem instead of the exact token, on a short page only, in place of plan 85's neighbor: dev gold 407/439 and 333/439, no-gold 36/51, section unchanged. It is the one change that places `s1b-c-0059` (`expire unread memories`).

- The note says "Anthropic's memory tool" and "expiry by last read". Porter already joins `memories` and `memory`.

- It does not join `expire` and `expiry`, or `unread` and `read`. The exact-token neighbor never sees `memory`.

- Holdout first falls 282/354 to 281/354 (`h2a3-c-0040`). Not adopted.

- **`expire` / `expiry` and `unread` / `read` are derivational, and the rewrite that joins them does not hold.** The nlprule lemma table keeps `expire` and `expiry` as two lemmas. Porter stems them to `expir` and `expiri`.

- `unread` is not in the lemma table; `read` is. Replacing the query with `expiry read memories` (drop final `e` and add `y` only when both forms are lemmas and the Porter stems share a prefix of five, and strip `un-` only when the whole word is not a lemma and the base is) puts `s1b-r-0072` at rank 2.

- Plan 85's own page for that query is already five other notes, so an empty-page rule never reaches it. Applying the replacement on every query, together with the hyphen split, raises dev gold to 411/439 and 351/439 and section to 57/73, and drops cross-scope any from 30/42 to 29/42.

- The lost case is `s4-c-0132`, whose later wording is `react-navigation theme import`. The exact token is the name of the package in the other repository.

- Splitting it loses the hit. Not adopted.

- **nlprule does not split English hyphens.** `configs/en/tokenizer.json` sets no `extra_split_chars`. The Spanish config does, and the characters are `-` and the dashes.

- The tokenizer comment names `-` as the example of a language-specific extra split. A blanket split is the rule that lost `react-navigation`.

- The English file's other sanitiser is a join regex that keeps a URL as one token. That does not touch these queries.

- Apostrophes are not in the ten losses. There is still no separate stemmer in the crate, and the lemma column measured earlier does not pay for itself.

- **Plan 86, hyphen fallback, beats plan 85.** `plans/86-hyphen-fallback/`. When plan 85 returns no card, a token that is exactly two alphabetic pieces and one hyphen, with no digit, no underscore and no colon, is split, and the query is run again.

- A quoted phrase is left alone. Two or more hyphens stay one token, so `git-ssh-access` does not split.

- A card from the second search is kept only when that section contains every content word of the rewritten query. The pieces of a hyphenated word in the section count.

- If none of the new cards cover every word, the page stays empty. The second search does not run when the first search already has a card.

- confirm80, `MEMEVAL_SHARED=1`, variant d: dev gold 406/439 and 333/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. Holdout gold 340/354 and 284/354, cross-scope 57/62 and 49/62, no-gold 56/106, section 233/271.

- The one dev first-page gain is `s1b-c-0035`, `folder-attached note query`. The two holdout first-page gains are `h2a1-c-0058`, `span-based`, and `h2a2-c-0037`, `context-events`.

- `s1b-c-0018` (`prior-art`) is not among them: the second search finds sections, and none of them contains every rewritten content word, so the page stays empty. That case remains a miss.

- The same split without the coverage check does not hold. On an empty page it reaches dev 409/439 and 345/439, section 57/73, cross-scope and no-gold unchanged, and holdout 342/354 and 305/354, cross-scope first 54/62, section 236/271, no-gold 55/106.

- The lost no-gold case is `s7-c-0060`, `blob-util remove Windows next major`. Keeping the token when the indexed text already contains it does not save that case: `blob-util` is not stored as its own token, the indexed name is longer, the gate still splits it, and holdout no-gold is still 55/106 (gold-any 342/354, first 299/354).

- The coverage check is what keeps the silence. Always splitting, including queries that already hit, is the dev cross-scope drop above. Not adopted.

- **The static embedding does not choose the split, and it is a bad lemmatizer.** potion-retrieval-32M stores one vector per vocabulary token. `encode` averages those token vectors and normalizes once.

- A hyphenated string is not a vocabulary entry. `blob-util` is the tokens `blob`, `-`, `util`, so its encoding and the encoding of `blob util` are the same vector, cosine 0.999.

- `prior-art` against `prior art` is 0.996. `react-navigation` against `react navigation` is 0.998.

- `git-ssh-access` against `git ssh access` is 0.998. The earlier figures of 0.80, 0.82, 0.83, and 0.72 do not reproduce on this model.

- Averaging two token vectors does not recover a compound that the tokenizer cut into different pieces. `reactnativeblobutil` is `react`, `##nat`, `##ive`, `##bl`, `##ob`, `##uti`, `##l`.

- Cosine to the average of `blob` and `util` is 0.153, and to the average of `react`, `native`, `blob`, and `util` is 0.275. The nearest vocabulary rows to the average of `blob` and `util` are `blob` at 0.706 and `util` at 0.699, then `blobs` and `blot`.

- No compound token sits at that average. `unread` against `read` is 0.34, so the same space does not join the pair the lemma table also refuses.

- `author` against `write` is 0.69, above `author` against `authored` at 0.62. The nearest neighbor of `expire` at cosine 0.71 is `expiring`, then `expiration` at 0.70.

- `expiry` is fourth, at 0.65. The note says expiry.

- Taking that nearest neighbor as the query word, when it shares a five-character prefix, cosine is ≥ 0.60, and the neighbor occurs in the index, rewrites 494 query types. `expire` becomes `expiring`.

- `expiry` becomes `expire`. `health` becomes `healthcare`.

- `cleanup` becomes `cleaned`. Applied to every query, dev falls to gold 380/439 and 307/439, cross-scope 29/42 and 23/42, no-gold 33/51, section 53/73.

- Applied only when plan 86's page is empty, every floor stays on plan 86's number. Not adopted.

- **Plan 87, the model's pieces, beats plan 86.** `plans/87-model-pieces/`. The tokenizer supplies this rewrite; vectors do not.

- Plan 86 opens only a single hyphen of two alphabetic pieces. `expo-modules-jsi` and `watch-kill-signal` have two hyphens, and `whatToTest` has none, so they stay identifiers and the page stays empty.

- The wordpiece tokenizer cuts them: `expo`, `modules`, `js`; `watch`, `kill`, `signal`; `what`, `to`, `test`. When plan 86 returns no card, each unquoted token is replaced by those alphabetic pieces longer than two characters, and the query is run again.

- A card is kept only when the section contains every content word of that query. A quoted phrase is left alone.

- A token that contains `:` is left alone.

- confirm80, `MEMEVAL_SHARED=1`, variant d: dev gold 406/439 and 335/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. Holdout gold 341/354 and 285/354, cross-scope 57/62 and 49/62, no-gold 56/106, section 234/271.

- Dev first-page gains: `s3b-c-0016` (`expo-modules-jsi`) and `s5-c-0044` (`watch-kill-signal`). The holdout gains are `h2a3-c-0014` (`max-old-space-size`, first and section) and `s7-c-0121` (gold-any, from the later wording `eas submit whatToTest`).

- Splitting every hyphen and underscore into alphabetic pieces longer than two characters, with the same empty-page and coverage gates and without the model, matches plan 87 on dev and on the holdout first-page and section gains. It misses `whatToTest`, so holdout gold-any stays 340/354.

- The tokenizer is the rule that catches that camelCase token. `Grok` becomes `gro` under the same tokenizer.

- No dev or holdout floor moved on account of that, because a section that does not contain every remaining content word is dropped.

- **WordNet and a second stemmer do not beat plan 87.** The index stemmer is FTS5 Porter. Matcher B already applies it to an ordinary word.

- NLTK's WordNet lemmatizer, restricted to a lemma whose Porter stem is the word's own stem and that occurs in the index, rewrites 323 dev query types (`memories` to `memory`, `commands` to `command`). Dev falls to gold 398/439 and 330/439, cross-scope 26/42 and 22/42, no-gold 31/51.

- The surface change moves matcher A off the body word that Porter would still have joined. Not adopted.

- WordNet `derivationally_related_forms`, kept only when the two words share a five-character prefix, the edit distance is ≤ 2, the Porter stems differ, and the relative occurs in the index, rewrites 74 types. The list includes `project` to `projector`, `server` to `serve`, and `expiry` to `expire`.

- It does not include `expire` to `expiry`: `expiry` is not among the related forms of `expire`. `unread` has no related forms.

- Dev falls to gold 397/439 and 325/439, cross-scope 27/42 and 23/42, no-gold 32/51. Not adopted.

- A Snowball or Lancaster stem is not a surface form the body contains, and the index will not stem the query with it, so it was not substituted in as a query word.

- **The embedding's remaining preprocessing use is a spelling check, and it ties.** For a query word absent from the index, the nearest corpus word at edit distance 1 is kept when the cosine is ≥ 0.65 and the words share a four-character prefix. On dev that rewrites `bands` to `band` and `glyph` to `glyphs`, and every floor stays on plan 87.

- The pairs a bare edit distance would also rewrite are far apart: `accented` against `accepted` is 0.03, `restating` against `restarting` is 0.17, `retriever` against `retrieve` is 0.18. The threshold refuses them, and the inflections it would accept are the ones Porter already joins.

- Nothing is gained. The same static vector still cannot choose a hyphen split, for the reason in the plan 87 paragraph.

- **Plan 88, unattested camelCase, beats plan 87.** `plans/88-unattested-camel/`. The body index already splits a camelCase word where a lowercase letter meets an uppercase letter, and stores the pieces.

- The query keeps that word as one identifier, so those pieces are never looked up. Plan 88 splits the query token the same way, before plan 87 runs, and only when the whole token does not occur in the index.

- `GitHub`, `TestFlight`, and `macOS` occur, so they stay whole. `iOS` does not match the split (there is no lowercase letter before the capital).

- A hyphen is left to plan 87. A quoted phrase and a `:` filter are left alone.

- confirm80, `MEMEVAL_SHARED=1`, variant d: dev gold 406/439 and 337/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. Holdout gold 341/354 and 287/354, cross-scope 57/62 and 49/62, no-gold 56/106, section 234/271.

- Dev first-page gains: `prebuiltArtifactPaths` and `StrictHostKeyChecking`. Holdout first-page gains: `fsyncSync` and `useNavigation`.

- The rule is the index's own `pieces()` split, applied to a query token the index has never stored whole. It does not need WordNet or a second stemmer.

- **The fallback checklist is stricter than FTS5, and dropping it is also worse.** Plan 86 keeps a card from the rewritten query only when every content word is a Porter stem of the raw heading or body. The checklist splits a hyphen.

- It does not split `_` or a camelCase boundary. The parts column does both, and matcher B stems them.

- `RACE_CONDITION` is one token to the checklist and two tokens to the index.

- Keeping every card FTS5 returned, with no checklist, is the loose end of that comparison. Dev gold rises to 411/439 and 356/439, section to 58/73, and no-gold stays 35/51.

- Holdout gold rises to 343/354 and 313/354, cross-scope first to 56/62, section to 237/271, and no-gold falls 56/106 to 55/106. The partials the checklist was refusing are real silence failures. Not adopted.

- Rebuilding the checklist so that it splits camelCase and underscores, the same way `pieces()` does, recovers `RACE_CONDITION` on dev (gold-any 407/439, nothing else moves) and then loses a holdout first-page hit, a source-origin identity query, 287/354 to 286/354. The extra card sits ahead of the gold. Not adopted. Using only the FTS5 tier, and ignoring the checklist, also recovers `RACE_CONDITION` and drops dev first 337/439 to 336/439 (`expo-modules-jsi`).

- The checklist was right about that card. Not adopted.

- Splitting every camelCase token, including ones the index has already stored, drops dev gold-any to 405/439 and cross-scope first to 23/42. `resetCache` is a token the index contains.

- The unattested gate stays.

- **Plan 89, FTS5 all-terms, beats plan 88.** `plans/89-fts-allterms/`. A fallback card the checklist accepts is kept.

- A card the checklist rejects is kept when either matcher ranked that section as a phrase or as all terms, tier 0 or tier 1. A partial, tier 2, is still dropped.

- The decision is the tier FTS5 already stored on the ranking row. No threshold change.

- confirm80: dev gold 407/439 and 337/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. Holdout gold 341/354 and 287/354, cross-scope 57/62 and 49/62, no-gold 56/106, section 235/271.

- The dev gain is `RACE_CONDITION`, gold-any, on a later wording. The holdout gain is section on `trusty self-signed certificate`, and no retrieval floor moves.

- **Wrapping a row does not change its vector.** Each block is one encoded row: the static model is called once per block, on that block's own text. A wrap that only inserts newlines at spaces does not change the word list.

- Of 9,945 lines longer than 80 characters on the dev corpus, none change their words under that wrap, so the mean vector is the same row. The block cutter already breaks at paragraphs, list items, and fences.

- It does not break at a column width. Seventy-six paragraphs are longer than the 1,400-byte limit and stay one row, because a paragraph with no earlier boundary is not sliced.

- Cutting those on whitespace, and encoding each piece as its own row, drops dev gold-any from 407/439 to 406/439. First, cross-scope, no-gold, and section stay put. Not adopted. The card's 180-character excerpt can end mid-word.

- The graded span is the block, not that excerpt, so wrapping the excerpt does not move a floor. Hyphenating inside a word to make the line fit would add hyphen tokens, which the identifier rule then treats as exact.

- That is not the wrap.

- **Adversarial decoding, measured on plan 89, does not become the plan.** Zhang, Zhang, and Shmatikov, [Adversarial Decoding](https://arxiv.org/abs/2410.02163), and [ZSInvert](https://arxiv.org/abs/2504.00147). Their retrieval scorer is the mean cosine of a candidate to a set of reference embeddings.

- Beam search grows the candidate. Their retrieval code also keeps a tight set at cosine 0.85.

- There is no Llama on this path. The proposal step is the static token table, restricted to the 11,275 attested words whose IDF is ≥ 2.5.

- Four uses, each on top of plan 89, one dev build:

- Reordering a page by lexical tier and then by mean cosine to each query-word embedding ties plan 89 on every floor: 407/439 and 337/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. The per-word cosine does not break the ties that matter.

- On a short page, appending one other-record block whose cosine to the first card is ≥ 0.85 and that shares a content word also ties. The ball is empty.

- The 99th percentile cosine from one block to the rest of the index is 0.34, and the only cosine of 1.0 is the block itself. The threshold in their code admits nothing here.

- On a short page, dropping the most common query word and adding the one attested word a one-step beam picks toward the mean of the query-word embeddings, then keeping a card the plan-89 gate accepts for that rewritten query, also ties.

- The inversion does not tie. On an empty page only, a two-step beam grows two attested words toward the query embedding, those two words are searched, and a card is kept only when the section contains an original query word of IDF ≥ 2.5.

- Dev gold rises to 410/439 and 342/439. Cross-scope, no-gold, and section stay.

- First-page gold gains: `s3b-c-0015`, `s4-c-0010`, `s4-c-0020`, `s4-c-0074`, and `s5-c-0098`. The same rule also fills empty pages that are not gold: on the holdout, gold rises to 342/354 and 291/354, cross-scope stays 57/62 and 49/62, section stays 235/271, and no-gold falls from 56/106 to 54/106.

- Distinct empty queries land on the same few notes (`s7-r-0033`, `s7-r-0034`, `s7-r-0036` repeat). A silence the lexical search was right to keep gets a card. Not adopted.

- **The index's own term count keeps the beam's first-page gains.** Plan 11 already requires a match to contain `min(n, max(2, ceil(n / 2)))` of the content words. Applied to the beam's cards, that count replaces the single rare-word overlap.

- Dev gold-first rises from 337/439 to 339/439. Gold-any stays 407/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.

- First-page gold gains: `s4-c-0020` (`simulator:start free account max duration`) and `s5-c-0098` (`npx react-native start reset-cache flag form`). confirm80 on the holdout: gold 341/354 and 288/354, plan 89 was 341/354 and 287/354. Cross-scope 57/62 and 49/62, no-gold 56/106, section 235/271. No other floor fell. This is plan 90.

- Three other gates were scored on dev and not selected. Requiring the card to be the nearest block that shares a rare original word reaches first 338/439.

- Requiring the plan-89 checklist on the original query also reaches 338/439. Requiring two shared words of IDF ≥ 2.5 ties plan 90 at 339/439.

- The tie went to the count the index already uses. That one was confirmed on the holdout.

- The beam has no separate corpus. Its candidates are the index's own words: alphabetic, longer than two characters, IDF ≥ 2.5, taken from the document frequencies built with the blocks.

- A new record adds its words. A word that becomes common enough to fall under 2.5 drops out.

- `forget` recomputes those frequencies, and the cached word vectors are dropped so the next search rebuilds them. The static model's vocabulary stays the one it was loaded with.

- The proposal list does not.

- **A weaker stemmer does not move plan 90, and a stronger one does not either.** The term count compares surface tokens. Replacing that equality with a less capable rule, and leaving the beam and the count `min(n, max(2, ceil(n / 2)))` alone, was scored on dev.

- A plural suffix (`ies` to `y`, then `es`, then `s`), the same plus `ed` and `ing`, WordNet `morphy` (inflection only), NLTK Porter, and Lancaster all keep every floor on plan 90: gold 407/439 and 339/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. Each one adds the same two records to `s2b-c-0020` (`metro:bundling spans`) and nowhere else.

- Porter does not retrieve a gold page the plural suffix misses. Lancaster does not either.

- The count does not need a capable stemmer, and giving it one does not raise a floor. Plan 90 stays on exact tokens.

- NLTK bigram PMI, scored with `BigramAssocMeasures.pmi` over adjacent content words inside each indexed section and a joint count of ≥ 3, is the neglected proposal source. The two strongest partners of the query words replace the beam.

- The term count stays exact. Dev gold-first falls from 339/439 to 337/439.

- First-page gold losses: `s4-c-0020` and `s5-c-0098`, which is plan 90's whole gain. Cross-scope, no-gold, and section stay.

- A collocation list assembled from the index does not stand in for the beam.

- WordNet as a query rewrite, and a second stemmer as a query rewrite, remain the measurements under plan 87. POS taggers, Punkt, and the parsers are not in the local NLTK data, and none of them is the equality this count uses.

- **Neither a Morton cell nor UMAP's fuzzy neighbor becomes the plan.** [barnes-hut-tree](https://crates.io/crates/barnes-hut-tree) is a Morton tree for force approximation in 2 to 7 dimensions. A distant cell is one center of mass.

- The block vectors are 512 wide, so the tree cannot be built on them. Projecting onto the first three principal components and interleaving 4 bits per axis, which is the crate's quantization, yields 1,335 cells on dev.

- On a short page the rare-word neighbor is taken from the query's cell when that cell has one, and the cosine neighbor remains otherwise. Dev gold-first falls from 339/439 to 338/439.

- Other floors unchanged. Not adopted.

- [rag-umap](https://crates.io/crates/rag-umap) is UMAP. The 2-dimensional layout is a picture of a graph, and Barnes-Hut only approximates the repulsive forces inside that picture.

- The graph is the part that can change a neighbor. Each block gets the paper's smooth 15-neighbor kernel.

- A query joins a block by the fuzzy union of both directions, so a block in a tight region scores below a block at the same distance in a sparse region. That order replaces the cosine order in the short-page neighbor.

- The empty-page beam stays. Dev gold rises from 407/439 and 339/439 to 408/439 and 341/439.

- First-page gold gains: `s1b-c-0049` and `s5-c-0090`. Cross-scope, no-gold, and section stay.

- Sixteen first pages change. On the holdout, gold-any stays 341/354 and gold-first falls from 288/354 to 287/354.

- Cross-scope, no-gold, and section stay. The local scale does not travel. Not adopted. Plan 90 stays the plan.

- **The two narrow gates in plan 90 are not hiding a floor.** The beam runs only on an empty page, and it grows exactly two words (`width` 4, `topk` 20, IDF ≥ 2.5). Letting that same beam append on a page that already has one to four cards leaves every dev floor on plan 90 and changes five first pages.

- Growing one word instead of two does the same, and changes seven first pages. Neither raises gold, cross-scope, no-gold, or section.

- The empty-page limit and the second word are narrower than the floors require, and widening them does not retrieve anything the floors count.

- The stack under that beam is four empty-page rewrites, and they are not one rule wearing four names. An unattested camelCase split, a single alphabetic hyphen, the model's wordpieces, and the beam each run only after the previous one has returned no card.

- The first three are admitted by the plan 89 gate. The beam is admitted by the term count.

- Asking the plan 89 gate to admit the beam, on the original query, reached dev gold-first 338/439, one below the term count. The term count is plan 11's `min(n, max(2, ceil(n / 2)))` applied to surface tokens.

- A plural suffix, Porter, and Lancaster, used as that equality, do not raise a floor either. IDF 2.5 is the same cut the neighbor and the vector hints already use. Plan 90 stays as it is.

- **The hybrid misses are absent from both rankings, and the static mean already places many of them first.** On the full dev build, the cases hybrid wins and plan 90 misses are not sitting past rank 5. They are missing from both matcher rankings: `s1-c-0001`, `s1-c-0003`, `s1-c-0011`, `s1b-c-0007`, `s1b-c-0008`, `s1b-c-0025`, `s1b-c-0040`, `s1b-c-0059`, `s5-c-0004`, `s5-c-0041`, `s5-c-0090`, `s5-c-0134`.

- Reranking the lexical top 40 cannot recover them. The overlap with the gold body is one ordinary word, or none (`s1b-c-0059` expiry against last-read; `s5-c-0041` against the lint that rejects low-value comments).

- Deleting one low-IDF query term, the automatic form of Fang's query-term deletion (SIGIR 2011), cannot bridge a one-word overlap to a unique note, and RM3 has no feedback documents on an empty page. The same queries, scored by the mean-pooled block vector, put the gold record at rank 1 or 2 for `s1-c-0001` (0.409), `s1-c-0003` (0.563), `s1b-c-0007` (0.642), `s1b-c-0040` (0.734), `s1b-c-0059` (0.409), and the cross-scope target of `s5-c-0118` (0.617).

- Several more sit inside the top 12. The vector is not the thing the card shows.

- It only chooses which block's own bytes to show.

- **An empty page's nearest eligible block, with no shared word, beats dev and loses holdout silence.** Of the 490 dev cases, 24 gold queries and 2 no-gold queries have an empty first page. The gold queries' nearest eligible cosine runs from 0.329 to 0.823 (median 0.533).

- The gold record is that neighbor in 6 of the 24, and inside the top 5 in 12. One true silence sits at 0.267 and one at 0.860.

- A floor of 0.40, chosen because it sits above that low silence and keeps the gold neighbors, was scored on the cached pages: gold 408/439 and 345/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. A gap of 0.04 on top of the same floor reaches 408/439 and 344/439 and holds the other floors.

- The 0.40 rule was the one sent to the holdout. confirm80's floors there: gold-any stays 341/354, gold-first rises from 288/354 to 299/354, cross-scope first rises from 49/62 to 51/62, section stays 235/271, and no-gold falls from 56/106 to 54/106. Two holdout silences have a nearest block at or above 0.40, and the page shows it.

- A weak hit is worse than silence. Not adopted.

- The floor was not refit on the holdout.

- **A token-level floor does not rescue that neighbor.** a late-interaction retrieval model's late interaction scores a query by the best document token for each query token (Khattab and Zaharia, SIGIR 2020). The usable gate is the worst query token, not the sum: the sum grows with the query.

- On the same empty first pages, that minimum was computed from the static model's own token vectors, each normalized. The six gold neighbors that are the gold record sit between 0.045 and 0.231.

- The no-gold query at cosine 0.860 sits at 1.0, which is a token the block already contains. Requiring the minimum to be 0.30, 0.40, 0.50, or 0.55, on top of the 0.40 block cosine, returns every dev floor to plan 90.

- The gate keeps the note that shares a token and drops the paraphrase. Not adopted.

- **Dropping the lowest-IDF query word does nothing.** Fang's query-term deletion (SIGIR 2011) helps when the oracle removes the term that over-specified the query. The automatic stand-in drops the one content word with the lowest IDF, and only when that IDF is under 2.5 and the query has ≥ three content words.

- The shortened query is searched with the same page as plan 90. Dev stays 407/439 and 339/439.

- Cross-scope, no-gold, and section stay. A common word was not what kept these gold notes out of the rankings.

- **A beam card the index ranked as a full match beats dev and loses one holdout silence.** Plan 90 drops a beam card that misses the original term count, even when the two proposed words are themselves a phrase or an all-terms match. Keeping those cards, and only those, on an empty page: dev gold 409/439 and 342/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.

- The holdout rises on gold, from 341/354 and 288/354 to 344/354 and 297/354. Cross-scope stays 57/62 and 49/62.

- Section stays 235/271. No-gold falls from 56/106 to 55/106.

- The lost silence is `s7-c-0060` (`blob-util remove Windows next major`): the two attested words co-occur in a note the case requires to stay off the page. Not adopted.

- The tier cut was not relaxed or tightened after that result.

- **The rarest query word, as its own vector, is the same silence failure.** TILDE ranks by the probability of each query term given the passage, and those probabilities can be stored at index time (Zhuang and Zuccon, SIGIR 2021). Without that model's language-model head, the static embedding of the single rarest content word (IDF ≥ 2.5) is the stand-in, and the nearest eligible block is the passage.

- On dev the gold record is that block for one empty first page (`s4-c-0074`, the word `multi-step`, cosine 0.473). The higher cosines, 0.53 to 0.84, are the wrong record: `nocase` at 0.84, `expo-file-system` at 0.76, `expo-image` at 0.74.

- A floor of 0.45 is the only cut in {0.45, 0.50, 0.55, 0.60, 0.70} that moves a floor: gold 408/439 and 340/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. A gap of 0.05 at 0.55 does not move.

- The 0.45 rule on the holdout leaves gold at 341/354 and 288/354 and cross-scope at 57/62 and 49/62, and no-gold falls from 56/106 to 54/106. The two lost silences are `h2a2-c-0032` and `s7-c-0060`, the same pair the rare-word neighbor already lost. Not adopted.

- **Porter on the camelCase and hyphen pieces does not move a floor.** The exact index already splits a lower-to-upper boundary, a hyphen, and an underscore into the parts column. The Porter index is the raw heading and body, so `Authored` inside `includeCoAuthoredBy` is not a token the stemmer sees, and the model's own wordpieces are no substitute: that identifier encodes as `include` and `thor`.

- Rebuilding the Porter index with the same pieces appended, then rescoring dev, leaves every floor on plan 90: gold 407/439 and 339/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. A query word that is itself an identifier never consults that index, and the ordinary words were not waiting on a piece stem.

- **Author term lists would clear the term count for 38 gold records, and they stay out.** Plan 71 strips `terms` and `section_terms` before build. Putting those lists back into the text, and ignoring rank, would push 38 of 439 dev gold records over `min(n, max(2, ceil(n / 2)))` when the body alone does not.

- Two of the hybrid misses are in that set in spirit: `s1b-c-0007` carries `cap` and `size` in the list, and the body has `grok` and `rules`. The list does not help the misses with no shared word (`s1b-c-0059`, `s5-c-0041`).

- This was not scored as a page. The writer does not supply those fields. Plan 90 stays the plan. The hybrid gap on these misses is a paraphrase with no shared stem.

- Each empty-page card that does not require that stem raises dev gold and then fails a holdout silence. The static mean can see the gold note.

- It cannot tell that note from a silence.

- **Gating the empty-page card by the search's own near misses does not protect the holdout.** The three backups above were rerun so a card is shown only when that same record is already one of the ≤ three hints on the empty call. Of 26 empty dev first pages, 25 already carry hints.

- The nearest block is one of those hints on 20, and it is the gold record on 5. Requiring both the 0.40 cosine and a hint: dev gold-first rises from 339/439 to 344/439.

- Gold-any stays 407/439. Cross-scope, no-gold, and section stay.

- The sixth ungated gain, `s6-c-0074`, is the nearest block and is not a hint, so the gate drops it. The same gate on the beam keeps one first page, `s1b-c-0018`, and dev first is 340/439.

- On the rare-word probe it keeps nothing, and dev stays 407/439 and 339/439. Widening the gate from a shown hint to a section one word short of the hint count does not change the cosine rule.

- It raises the beam to 408/439 and 341/439, still under the hint-gated neighbor. The neighbor was the one sent to the holdout.

- Gold-first there rises from 288/354 to 299/354 and cross-scope first from 49/62 to 51/62. No-gold falls from 56/106 to 54/106.

- The lost silences are again `h2a2-c-0032` and `s7-c-0060`. Both notes were already near misses of the empty search.

- The hint list is not a silence filter. A near miss the search refused to show as a card is still a card once the backup agrees, and on those two queries the backup agrees. Not adopted.

- **A full page of partials is the larger miss, and swapping one card for the nearest other block does not travel.** Of the dev gold misses on the first wording, 43 are five cards whose best tier is a partial, 23 are empty, 16 already contain an all-terms card, and 5 contain a phrase. An empty-page rule cannot reach the 43.

- Replacing the last of those five with the nearest eligible block that is not already shown, at a cosine floor of 0.45, 0.50, 0.55, or 0.60, drops dev gold-any from 407/439 to 406/439. Twelve of the cards in that last position are gold. Not adopted.

- Choosing the victim by cosine instead of by rank is the adjustment. On a page of five partials, replace the card whose own block is least like the query, and only when the incoming block's cosine is ≥ 0.50 and ≥ 0.05 higher.

- Dev gold-first rises from 339/439 to 340/439. Gold-any, cross-scope, no-gold, and section stay.

- A floor of 0.45 with no margin leaves gold-first at 339/439 and raises cross-scope from 30/42 and 24/42 to 31/42 and 25/42. Gold-first was the selection, so 0.50 with the 0.05 margin was the rule sent on.

- The holdout falls: gold 340/354 and 287/354, section 234/271. Cross-scope stays 57/62 and 49/62.

- No-gold stays 56/106. Two first pages are lost (`s7-c-0004`, `s7-c-0146`) and one is gained (`s7-c-0012`).

- Seven of the 130 dev pages that already contain gold would have removed that gold; the holdout is where the removal wins. Not adopted.

- The floor was not refit.

- **A character-trigram neighbor of a missing token is a punctuation variant or a hub word.** Fifty-five dev query tokens of length ≥ 5 are absent from the exact vocabulary. The nearest vocabulary term by character-trigram Jaccard is ≥ 0.5 and occurs in the gold text for 14 of them.

- The list is `--watch-kill-signal` for `watch-kill-signal`, `bundleidentifier` for `ios.bundleIdentifier`, `condition` for `RACE_CONDITION`, `phrase` for a quoted exact phrase, `access` for `co-access`. The first kind is the same token with a flag prefix or a dot.

- The second kind is a short word the pieces already produce. Neither is a new rewrite.

- The edit-distance spelling check remains the measurement of a nearest vocabulary word.

- **An identifier the index has never seen is a hard requirement, and releasing it fills a holdout silence.** A hyphen, an underscore, or a camelCase token is matched only against the exact body column, and every such token has to hit or the section is dropped. `all-word` blocks a note that says `all-terms`.

- `blob-util` is not stored as that token, so `blob-util remove Windows next major` returns nothing. Three repairs were scored.

- None is the plan.

- On an empty page, replace that token with the pieces that are already in the vocabulary and have IDF ≥ 2.5, then search again. The Porter index is left as it is.

- Dev gold rises from 407/439 and 339/439 to 409/439 and 348/439. Cross-scope stays 30/42 and 24/42.

- No-gold stays 35/51. Section rises from 55/73 to 56/73.

- Dropping the token and keeping none of its pieces is the weaker sibling on the same empty pages: 409/439 and 345/439, with the same other floors. Adding the repaired search's full matches in front of a page that already has no phrase and no all-terms card does not move a floor past the empty-page numbers.

- Keeping only a card the repaired query ranked as a phrase or as all terms returns every dev floor to plan 90. The piece rule was the one sent on.

- The holdout rises on gold, from 341/354 and 288/354 to 343/354 and 302/354, and on cross-scope first, from 49/62 to 52/62. Cross-scope any stays 57/62.

- Section rises from 235/271 to 237/271. No-gold falls from 56/106 to 55/106.

- The lost silence is `s7-c-0060`. The repaired query is `blob util remove Windows next major`, and the five cards are partial matches of `ReactNativeBlobUtil`.

- The partials are the gains and the silence. Not adopted.

- Appending the same camelCase, hyphen, and underscore pieces to every Porter row, and changing no query, leaves dev on plan 90. On the holdout, gold-first rises from 288/354 to 290/354, cross-scope falls from 57/62 and 49/62 to 56/62 and 48/62, and no-gold falls from 56/106 to 55/106.

- Section stays 235/271. An ordinary query word can now match a piece it never matched. Not adopted.

- The third repair leaves the token in the term count and stops requiring it to hit, on the first search, and only when the exact vocabulary does not contain it. A quoted phrase stays required.

- Dev gold rises to 408/439 and 344/439. Cross-scope stays 30/42 and 24/42.

- No-gold stays 35/51. Section rises to 56/73.

- The holdout rises further than the empty-page rewrite: gold 343/354 and 306/354, cross-scope first 52/62, section 237/271. No-gold falls from 56/106 to 55/106.

- The lost silence is again `s7-c-0060`. The page shows `s7-r-0034` and notes from other shards.

- The hard requirement is what keeps that query empty. Releasing it, whether by splitting the token or by no longer requiring it, shows a card.

- A weak hit is worse than silence. Not adopted. Plan 90 stays the plan.

- **Widening the field with the static model, then grading the candidates, does not retrieve the missed note.** Hybrid's measured path does not raise silence by cutting on a score. `hybridQuery` defaults `minScore` to 0.

- The reranker is blended in, and a document already in the top three keeps three quarters of its retrieval rank. On both slices empty-correct ties plan 90 (4/5 and 9/10).

- The lead is recall. An expansion line is kept only when it still contains an original query term, and a strong keyword hit skips expansion.

- The generative model and the cross-encoder are what turn that into a different page. The static model was asked to do the same job without them.

- The field is the vocabulary the beam already uses: alphabetic, length above 2, IDF ≥ 2.5, 11,275 words. A query word proposes the nearest of those words whose Porter stem differs, and a block is a candidate when it contains that word.

- On the 24 empty gold first pages the nearest such word is a string neighbor in a different note: `nix`/`nixpkgs`, `react`/`reactnative`, `nocase`/`nock`, `note`/`notes` at 0.781. Grading those candidates by the block cosine of the original query puts the gold record first once (`s6-c-0035`, `graph`/`a graph retrieval system`, cosine 0.347) and inside the top 20 once more (`s1b-c-0018` at rank 6).

- The other winners are the wrong record. Requiring two query words to land in the same block, each through a neighbor at cosine ≥ 0.60, recovers 0 of those empty gold pages.

- The paraphrase the model can see is real and too weak to win: `expire`/`expiry` is 0.648, `author`/`authored` is 0.618, `comment`/`comments` is 0.553, under `note`/`notes`. A floor that keeps the paraphrase keeps the hub.

- The one empty silence with a rare query word, `s2b-c-0050`, sits at block cosine 0.237, below the wrong notes a grader would have to accept to keep the single gold hit. Not adopted.

- **The adjacent cut, a neighbor that is rare and not a longer form of the query word, beats dev and loses one holdout cross-scope page.** Dropping a neighbor that contains the query word, or the query word contains it, leaves 0 of 23 empty gold pages and does not touch the empty silence. Keeping only a neighbor in ≤ five notes, or only inside the cosine band 0.58 to 0.72, also leaves those pages empty.

- Of 100 dev first-page misses, five gold notes contain any such neighbor. Four of those neighbors are common (`failed` in 67 notes, `remove` in 70, `incompatible` in 18).

- The selective one is `expire` to `expiry`, cosine 0.648, in 10 notes, on a page that already has five partials (`s1b-c-0059`). The scored rule keeps a neighbor at cosine ≥ 0.60 whose Porter stem differs, that neither word contains the other, and that occurs in ≤ 10 notes.

- On an empty page, or a page of five partials, it shows the eligible block that contains that neighbor and is closest to the query, at block cosine ≥ 0.40. Dev gold rises from 407/439 and 339/439 to 408/439 and 340/439.

- The new page is `s1b-c-0059`. Cross-scope stays 30/42 and 24/42.

- No-gold stays 35/51. Section stays 55/73.

- The holdout keeps gold-any at 341/354 and raises gold-first from 288/354 to 289/354 (`s7-c-0098`). No-gold stays 56/106.

- Section stays 235/271. Cross-scope first falls from 49/62 to 48/62. Not adopted.

- **Tricky terms collected at ingest, and matched as their own vectors, do not clear the bar.** A term is tricky when the body writes it with a hyphen, an underscore, an internal digit, or a camelCase boundary. Dev has 9,053 of them.

- A separate vector of those surface forms, compared with the whole query, has the gold note's own term inside the top five for 21 of 100 first-page misses. The nearest term is often a different note (`bare-expo` ahead of `expo-file-system`, `in-tree` ahead of `working-tree`).

- Ordinary words that occur in only one note almost never land there: 1 of the 100.

- The expansion that was scored keeps a tricky term when a query word is one of its pieces, the term's cosine to that word is ≥ 0.70, and the term occurs in ≤ eight notes. On an empty page it shows the block that contains the term, and only after two rechecks: the block's cosine to the query is ≥ 0.35, and the block contains a second query word.

- Dev gold-first rises from 339/439 to 340/439. The page is `s1-c-0001`, through `phrase` to `phrase-only`.

- Gold-any stays 407/439. Cross-scope, no-gold, and section stay.

- The frozen rule on a page of five partials, replacing the last card, drops dev gold to 406/439 and 338/439 and drops section. The empty-page rule on the holdout raises gold-first from 288/354 to 290/354.

- Cross-scope stays 57/62 and 49/62. Section stays 235/271.

- No-gold falls from 56/106 to 55/106. The lost silence is again `s7-c-0060`.

- Requiring the added block to come back as a phrase or an all-terms match of the query plus the tricky term returns every dev floor to plan 90: `phrase-only` is not in that ranking, and the one block that is a full match is the wrong note. Not adopted.

- **The silence was two common words, and a specific shared word keeps the page.** The first wording of `s7-c-0060` never matched. A later wording did.

- `blob-util windows support decision` matched `windows-1`, `windows-2`, `windows-3`, and `windows-4` in `s5-r-0023`. Those forms add no alphabetic piece: the digit is dropped, and the form is the query word with a number.

- The block shares `support` (IDF 3.31) and `windows` (3.89). `blob-util` (8.78) is not in it.

- The other wording, `blob-util major version windows`, matched `windows-native`, which does add `native`. That block shares `version` (2.43) and `windows` (3.89).

- `major` (5.06) is not in it. The dev gain is `phrase` to `phrase-only`, and that block contains `phrase` at IDF 5.58.

- The holdout gains share `pandoc` (7.39) or `length` (5.04).

- Two rechecks use that difference. The form must have an alphabetic piece other than the query word, so a numbered copy of the word is not a candidate.

- The block must contain a query word whose IDF is ≥ 5, so a pair of common words is not enough. The cosine floor of 0.70, the eight-note cap, and the block cosine of 0.35 stay where the failed rule had them.

- Replacing the last card of a full page stays out.

- confirm80: dev gold 407/439 and 340/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. First-page gain: `s1-c-0001`.

- Holdout gold 341/354 and 290/354, cross-scope 57/62 and 49/62, no-gold 56/106, section 235/271. First-page gains: `h2a2-c-0064` (`accept_changes`) and `s7-c-0089` (`single-file`).

- `s7-c-0060` stays empty. Four other holdout queries with no gold also gain a card (`h2a1-c-0107`, `s7-c-0041`, `s7-c-0107`, `s7-c-0109`).

- The no-gold count stays 56/106. This is plan 91, `plans/91-tricky-term/`.

- On the saved hybrid slices, one page changes. On the 1,243-note index, `s1-c-0001` goes from an empty first page to `s1-r-0001` at rank 1.

- Gold on some wording stays 45/48. Gold on the first wording rises from 35/48 to 36/48.

- First-query MRR rises from 50.2% to 52.3%. Cross-scope stays 4/4 and 4/4.

- Empty-correct stays 4/5. Overall rises from 82.8% to 83.5%.

- Hybrid on that index stays 46/48 and 40/48, overall 87.0%. The 90-note index does not move: `phrase` has IDF 3.10 across 312 blocks, under 5, so the same block is not shown.

- That page stays 46/48 and 38/48, overall 85.7%. The `s5` slice changes no page.

- Plan 91 is ahead of plan 90. It does not pass hybrid.

- **The same forms, averaged per section, retrieve four empty pages the single form missed.** Plan 91 indexes each tricky surface form and asks whether one query word is a piece of it. A section is a different object.

- Its vector is the mean of the tricky forms inside it that occur in ≤ eight notes. On an empty page, the closest eligible section is shown when that cosine is ≥ 0.35 and the section contains a query word of IDF ≥ 5.

- The one dev silence that shares such a word sits at 0.156, under the floor the single-form rule already uses.

- confirm80: dev gold 410/439 and 344/439, plan 91 was 407/439 and 340/439. Cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.

- First-page gains: `s3b-c-0015`, `s4-c-0002`, `s4-c-0010`, and `s5-c-0067`. The first three were gold-any misses as well.

- Holdout gold 341/354 and 294/354, plan 91 was 341/354 and 290/354. Cross-scope 57/62 and 50/62, plan 91 was 57/62 and 49/62.

- No-gold stays 56/106. Section rises from 235/271 to 236/271. Nothing falls. This is plan 92, `plans/92-section-term/`.

- Using that mean to replace the shown section of a record already on the page does not travel. The section with the most query words of IDF ≥ 5 would repair 2 dev section misses and break 13 that are already correct. Not adopted. The page keeps the section the word search returned.

- On the saved slices, against plan 90. The 1,243-note `s1` slice stays where plan 91 left it: 45/48 and 36/48, MRR 52.3%, overall 83.5%.

- One further page changes, `s1b-c-0050`, and the added record is not the gold. The `s5` slice gains `s5-c-0067` on the first wording: gold first 66/81 to 67/81, MRR 55.6% to 56.8%, overall 73.5% to 73.9%.

- Cross-scope stays 7/10 and 5/10. Empty-correct stays 9/10.

- Hybrid on that slice stays 78/81 and 64/81, overall 80.3%. The 90-note index stays 46/48 and 38/48, overall 85.7%.

- Plan 92 is ahead of plan 91. It does not pass hybrid.

- **The constants around that section, and the old identifier release, do not go further.** Four changes, each on top of plan 92. Widening the section mean from forms in ≤ eight notes to every tricky form adds no remaining empty gold page on dev, and no short page either.

- Lowering the shared-word cut from IDF 5 to 4 adds no gold page and does add the dev silence `s2b-c-0050`, at cosine 0.476. Appending the same section mean onto a short page that does not already show that record adds no first-page gold.

- The eight-note cap and the IDF 5 cut are not what the remaining misses are waiting on.

- The decision that does get reopened is the absent identifier. Dropping a hyphen, underscore, or camelCase token the exact vocabulary has never stored, and only on an empty page, used to fill `s7-c-0060`.

- The new gate keeps a card from that second search only when the section contains an original query word of IDF ≥ 5. On dev that raises gold first from 344/439 to 346/439.

- First-page gains: `s1b-c-0018` and `s6-c-0035`. Gold any stays 410/439.

- Cross-scope, no-gold, and section stay. On the holdout, gold first rises from 294/354 to 295/354 (`s7-c-0150`) and cross-scope first from 50/62 to 51/62.

- Section stays 236/271. No-gold falls from 56/106 to 55/106.

- The lost silence is again `s7-c-0060`: with `blob-util` removed, the second search shows `s4-r-0040` and `s3b-r-0032`, and one of those sections does contain an original word of IDF ≥ 5. `s7-c-0100` is shown the same way.

- The specific-word gate stops a tricky-form card. It does not stop an ordinary search of the words that remain. Not adopted. The hard requirement stays. Plan 92 stays the plan.

- **Ordinary words pointed at sections raise dev, and the holdout still loses two silences.** The new index is every word of length above 2 that contains a letter and has IDF ≥ 2.5. Dev has 18,186 of them, across 6,096 sections.

- A section's vector in that index is the mean of the words it contains. On a page plan 92 left empty, that mean is shown when its cosine to the query is ≥ 0.35 and the section contains a query word of IDF ≥ 5.

- Dev gold rises from 410/439 and 344/439 to 411/439 and 345/439. The new pages are `s1b-c-0007` on the first wording and `s1b-c-0059` on a later wording.

- Cross-scope, no-gold, and section stay.

- The block vector already stored, under that same gate, is the stronger reading. The candidates are only the sections that contain the specific word, and the closest of those texts is the card.

- Dev gold first rises further, to 346/439. Gold any is 411/439.

- The first-page gains are `s1b-c-0007` (`grok` and `per-file`, cosine 0.642, `s1b-r-0008`) and `s6-c-0035` (`co-access` and `edges`, 0.482, `s1b-r-0031`). `s1b-c-0059` is the later wording `memory expiry unread` (`expiry` and `unread`, 0.462, `s1b-r-0072`); the first wording stays a miss.

- No-gold stays 35/51. Section stays 55/73.

- Scoring those same sections by how each query word sits against the section's words, instead of by the block cosine, lands on the same three pages. Following a query word to a different indexed word, and from there to that word's section, moves no floor: a neighbor at cosine 0.70, in ≤ eight notes, with a different stem and no shared spelling, still has to clear the same two gates, and the page stays 410/439 and 344/439.

- Eleven dev gold queries still have an empty first page. Four of those gold notes contain a query word of IDF ≥ 5.

- The block rule takes two of the four. `s1b-c-0018` is at 0.326, under the floor.

- `s1b-c-0025` is at 0.337, and a different section is the one above 0.35. The other seven notes share no such word, so this index has nothing to rank.

- Appending the word mean onto a short page, rather than an empty one, is the smaller gain: gold first 345/439 on `s2b-c-0014`, gold any stays 410/439.

- The empty-page block rule is the reading that was checked on the holdout. Gold stays 341/354 and 294/354.

- Section stays 236/271. Cross-scope first rises from 50/62 to 51/62.

- No-gold falls from 56/106 to 54/106. `h2a2-c-0032` asks `visual baseline threshold` and is shown `s5-r-0025` at 0.409.

- `s7-c-0060` asks `blob-util major version windows` and is shown `s4-r-0024` at 0.355. Both pages are required to stay empty.

- A section that already contains a specific query word is still the wrong note when that word is all the two share. The floor was not moved. Not adopted. Plan 92 stays the plan.

- **A section is not the mean of its words. Two separate hits in one block are.** The mean of a block's ordinary words, and the mean of its rare surface forms, both collapse a combination into one vector.

- Two readings keep the hits apart and run only on a page plan 92 left empty. Neither one asks the block to contain a query word.

- The first takes two of the fifteen rare surface forms nearest the query text, each at cosine ≥ 0.35, and keeps a block only when both forms sit in it. Dev stays 410/439 and 344/439.

- Cross-scope, no-gold, and section stay. Not the plan.

- The second splits the content words into two phrases of ≥ two words. Three words become the first two and the last two.

- Four or more split in half. A sentence is a stretch of ≥ four words.

- The fifteen sentences nearest each phrase have to land on two different sentences of one eligible block, both at cosine ≥ 0.35. The lower cosine decides. confirm80: dev gold 410/439 and 345/439, plan 92 was 410/439 and 344/439.

- Cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. First-page gain: `s6-c-0035` (`co-access edges` at 0.456 on "It hurts when edges come from co-viewing during exploration.", `graph design` at 0.505 on "The graph belongs to a task, not to global popularity.", `s1b-r-0031`).

- Holdout gold 341/354 and 294/354, cross-scope 57/62 and 50/62, no-gold 56/106, section 237/271. The new holdout section is `h2a2-c-0074`. Nothing falls. This is plan 93, `plans/93-half-sentences/`.

- On the saved hybrid slices the rates stay where plan 92 left them. The new dev page, `s6-c-0035`, is not in either slice.

- On the 1,243-note index, `s1` stays 45/48 and 36/48, MRR 52.3%, overall 83.5%, and `s5` stays 76/81 and 67/81, MRR 56.8%, cross-scope 7/10 and 5/10, overall 73.9%. The 90-note index stays 46/48 and 38/48, overall 85.7%.

- Two first pages do change, and neither is gold: `s5-c-0051` gains `s5-r-0026` on the large index, and `s1b-c-0025` gains `s1b-r-0033` on the 90-note index. Hybrid stays 46/48 and 40/48, overall 87.0%, on the large `s1` index; 47/48 and 47/48, overall 90.1%, on the 90-note index; and 78/81 and 64/81, overall 80.3%, on `s5`.

- Plan 93 leads hybrid on the `s5` first wording, 67 against 64, and trails on gold any and on cross-scope. It does not pass hybrid.

- **The fifteen-sentence window is not what the remaining misses are waiting on.** Plan 93 only keeps a pair when both sentences are among the fifteen nearest in the corpus. Dropping that window, and taking the best two sentences inside each block, still at cosine 0.35, adds nothing to gold, cross-scope, or no-gold on dev.

- They stay 410/439 and 345/439, 30/42 and 24/42, 35/51. Section rises from 55/73 to 56/73.

- A section-only rise does not beat the plan. Of the gold queries whose first page is still empty, three can be split into two phrases and have two sentences in the gold note: `s1b-c-0007` at 0.222, `s1b-c-0018` at 0.252, and `s2b-c-0055` at 0.124.

- The others are too short to split. Not adopted. Plan 93 stays the plan.

- **An identifier the index has never stored can be opened when every piece has to be in the section.** Thirty dev gold first-page misses still contain a hyphen, an underscore, a camelCase boundary, or a digit. Most of those tokens are already in the exact vocabulary, and opening a stored token is the change that dropped `react-navigation`.

- Of the ones the vocabulary has never stored, one empty page has both pieces in the gold note. `s6-c-0102` (`native-build`) also has the pieces, by stem, and its page already has cards, so this step does not run there.

- `s1b-c-0018` asks `word tree prior-art spec section`. `prior-art` is absent.

- The note contains `prior` and `art`. The hyphen step already rewrites one alphabetic hyphen, then keeps a card only when the section contains every content word of the rewritten query.

- This note does not, so that step leaves the page empty. Dropping the token and keeping any section that contains one remaining word of IDF ≥ 5 recovers the page and then fills `s7-c-0060`.

- The gate that was not tried asks for the pieces of the absent identifier itself.

- On a page plan 93 left empty, a token is split only when it has one of those shapes, the exact vocabulary has no row for it, and it yields ≥ two alphabetic pieces longer than two characters. A quoted span stays whole.

- A colon leaves the run unchanged. An identifier the index has already stored stays whole.

- The rewritten query is searched through plan 93, and that search does not split again. A card is kept only when the Porter stem of every piece is among the stems of the section's own words.

- The other words of the query are not required. The first five such cards stay, in the order that search returned them.

- confirm80: dev gold 410/439 and 346/439, plan 93 was 410/439 and 345/439. First-page gain: `s1b-c-0018` (`prior-art` opened to `prior` and `art`, cards `s1b-r-0031` and `s1b-r-0066`).

- Cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. Holdout gold 341/354 and 294/354, cross-scope 57/62 and 50/62, no-gold 56/106, section 237/271. Nothing falls. Ordinary inflection is already joined by matcher B.

- `expire`/`expiry` and `unread`/`read` are not joined by it, and opening them was already measured and not adopted. This is plan 94, `plans/94-absent-pieces/`.

- **The rare-form index never sees the dropped identifier, and the forms that contain its pieces are the silence.** `blob-util` is not in the exact vocabulary. The lexical matcher requires that token in the body, so `blob-util remove Windows next major` stays empty.

- That empty page is required: `s7-c-0060` has no gold, and `must_not` is `s7-r-0069`. The rare-form index does hold the stored name.

- `reactnativeblobutil` occurs in six notes, under the eight-note cap, and its pieces are `blob`, `react`, `native`, and `util`. Thirty other forms in that index also contain both `blob` and `util`.

- They are the same name with another word attached, in `s7-r-0033` through `s7-r-0036` and the notes beside them. None of them is the token `blob-util`.

- The index selects a form only when a query word is one of that form's pieces. `blob-util` is one query word.

- It is not a piece of `reactnativeblobutil`, and it is not a piece of any other rare form. The pieces `blob` and `util` would be, but the query does not search them as words.

- The beam's rare-word list is the same shape: a card is kept from the two proposed words only when the section still contains enough of the original words, and the hyphenated token is one of those words. Plan 94's later split does search `blob` and `util`, then keeps a card only when both are words in the section.

- The camelCase name is one word, so the card is refused. All three wordings of `s7-c-0060` stay empty.

- The two `blob-util` wordings of `s7-c-0100` stay empty.

- Feeding the pieces into the rare-form index would point at `reactnativeblobutil` and show those notes. That is the silence.

- A form whose pieces are exactly `blob` and `util` is not in the index, so a same-piece gate would have nothing to show either. On dev, nine gold first-page misses still carry an identifier the vocabulary has never stored.

- For eight of them the gold note contains no rare form that includes those pieces. The ninth is `s5-c-0051`: `RACE_CONDITION` sits inside `err_require_esm_race_condition`, on a nonempty page. Not adopted. Plan 94 stays the plan.

- **Matching each piece's own vector, instead of the averaged surface string, does not raise a floor.** Plan 91 encodes the whole surface form and keeps a form only when that average is among the fifteen nearest to the query word. A long name never enters those fifteen, because its vector is the mean of the tokenizer's pieces.

- The correction encodes each camel, hyphen, or underscore piece alone. A form matches when every piece of one query probe has cosine ≥ 0.70 with a distinct piece of the form, the form has another piece, and it occurs in ≤ eight notes.

- On a page plan 94 left empty, that match fires 952 times on dev. The block average, at cosine 0.35, rejects 910 of them.

- The specific-word gate rejects the other 42. No card is added.

- Dropping the block average and keeping the specific-word gate lets nine forms through. Five queries gain a card, none of them gold: `s1b-c-0007` is shown `provider-grok` in `s5-r-0010`, `s2b-c-0050` is shown `force-with-lease` in `s1b-r-0031`, and the other three are later wordings.

- Gold stays 410/439 and 346/439. Cross-scope, no-gold, and section stay.

- The piece vectors find the forms the average had hidden, and those forms are the wrong notes. Not adopted. Plan 94 stays the plan.

- **One shared piece is the wrong note. A form the query explains, including a piece it did not spell, is the command in the gold note.** The 952 matches were single probes.

- `grok` selects `provider-grok`. Requiring every piece of the form to share a Porter stem with a query probe removes that card.

- It does not by itself choose the gold note. On `CI package check expo-audio` both `package-check` and `check-packages` are fully explained.

- `package-check` occurs in one note, `s2b-r-0098`, and rarity selects it. That note is not gold.

- `check-packages` occurs in seven notes, including the gold note `s3b-r-0111`, whose text is "Run et check-packages separately for each changed package." The two forms differ in one way: `packages` is not a word the query spelled, and `package` is. A form that only rejoins the query's own words is the accidental hyphen.

- The form that adds a stemmed piece is the compound the averaged string could not name.

- On a page plan 94 left empty, keep the rare form with the most such new pieces, then the longer form, then the rarer one. The card is the shortest section that contains it. confirm80: dev gold 410/439 and 347/439, plan 94 was 410/439 and 346/439.

- First-page gain: `s3b-c-0058` (`check-packages`, `s3b-r-0111`). Cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.

- Holdout gold 341/354 and 294/354, cross-scope 57/62 and 50/62, no-gold 56/106, section 237/271. Two holdout queries gain a card and stay misses: `s7-c-0030` is shown `github-actions` in `s3a-r-0064`, and `s7-c-0140` is shown `testutils` in `s3b-r-0002`. Nothing falls. This is plan 95, `plans/95-explained-form/`.

- Before this step, six dev gold queries had an empty first page. This step takes one of them.

- The other five have no rare form that two query probes can explain. Plan 95 is the conversion of the piece bug. It does not pass hybrid.

- **A quoted filename is not in the body, and free text never reads `source_path`.** Three dev gold first-page misses name a file that the gold note's body does not contain. `s1b-c-0025` quotes `tailscale.nix`.

- The note `s1b-r-0011` has that basename as `source_path` (`modules/server/tailscale.nix`) and the first page is empty. `s1b-c-0049` quotes `sshd.nix`, which is the source path of `s1b-r-0048`, and the page already shows other cards.

- `s1b-c-0048` quotes `instructions.nix`. That basename is the source path of 17 records, including the gold note `s1b-r-0004`.

- `path:` would match these paths. The queries quote the filename as text, and the text index is heading, body, parts, and terms.

- On every page, a quoted phrase that equals the basename of exactly one record's `source_path` is placed first. The card is that record's earliest section.

- A basename shared by two records is left alone, so `instructions.nix` stays out. The body is not changed. confirm80: dev gold 410/439 and 349/439, plan 95 was 410/439 and 347/439.

- First-page gains: `s1b-c-0025` and `s1b-c-0049`. Cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.

- Holdout gold 341/354 and 294/354, cross-scope 57/62 and 50/62, no-gold 56/106, section 237/271. No quoted basename on the holdout was unique. Nothing falls. This is plan 96, `plans/96-quoted-filename/`.

- On the saved slices, against plan 95. The 1,243-note index changes those two first pages and no others.

- Gold first rises from 37/48 to 39/48. First-query MRR rises from 54.3% to 58.5%.

- Overall rises from 84.2% to 85.6%. Gold any stays 45/48.

- Cross-scope stays 4/4 and 4/4. Empty-correct stays 4/5.

- Hybrid on that index stays 46/48 and 40/48, MRR 63.1%, overall 87.0%. The 90-note index changes the same two pages.

- Gold first rises from 39/48 to 41/48. MRR rises from 61.4% to 65.5%, above hybrid's 64.5%.

- Overall rises from 86.4% to 87.8%. Hybrid stays 47/48 and 47/48, overall 90.1%.

- The `s5` slice changes no page: 76/81 and 67/81, MRR 56.8%, overall 73.9%, against hybrid 78/81 and 64/81, overall 80.3%. Plan 96 leads hybrid on the 90-note first-query MRR and on the `s5` first wording. It does not pass hybrid.

- **A shared filename can be resolved when the other words of the query all sit in one of those notes.** `instructions.nix` is the source path of 17 records, so plan 96 leaves `s1b-c-0048` on the wrong page: `s6-r-0082`, `s1b-r-0031`, and `s6-r-0079`. The gold note is `s1b-r-0004`.

- The quote is removed, and the remaining content words are stemmed. There have to be ≥ two distinct stems.

- A block qualifies only when its own text, not its heading, contains every one of those stems. If every qualifying block belongs to one eligible record, that record's earliest qualifying block is placed first.

- Two records, or fewer than two stems, and the page stays as it was.

- confirm80: dev gold 410/439 and 350/439, plan 96 was 410/439 and 349/439. The only changed first page is `s1b-c-0048`, and the new first card is `s1b-r-0004`.

- Cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73. Holdout gold 341/354 and 294/354, cross-scope 57/62 and 50/62, no-gold 56/106, section 237/271. Nothing falls. This is plan 97, `plans/97-filename-coverage/`.

- The new case is not in the saved hybrid slices, so those rates stay where plan 96 left them. It does not pass hybrid.

### Closed wide rewrites (2026-11-08)

- **Baseline.** Both wide rewrites of nonempty pages were scored on dev with
  confirm80 after reproducing the current baseline: gold 410/439 and 350/439,
  cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.
- **Not adopted — replace a duplicate with a later-call distinct record.**
  Changed 14 first pages, gained no gold page, lost none. Floors stayed gold
  410/439 and 350/439, cross-scope 30/42 and 24/42, no-gold 35/51, section 55/73.
  No retrieval rise; no holdout run.
- **Not adopted — replace a card with an unshown strict-superset lexical candidate.**
  Required equal or better tier and a strict superset of query terms. Changed
  117 first pages, gained no gold page, lost 7. Floors fell to gold 407/439 and
  343/439, cross-scope 29/42 and 23/42, section 54/73; no-gold stayed 35/51.
  No holdout run. Neither rewrite became a new implementation; no plan 98.

### Ideas to explore: lateral movement beside progressive disclosure (2026-11-05)

- Notes, not proposals yet. They recombine parts already in this file.

- Nothing here has been tested.

- **Two axes from one position.** A position is a card: a record and one of its sections. The vertical axis is an indexing tool's disclosure: search, card, outline, section, body, each step deeper into one record.

- The lateral axis moves to a neighbor. Every lateral edge below is derived from bytes, git, or explicit writes, so no agent maintains it.

| Move | Edge comes from | Where it appears above |
|---|---|---|
| Previous, next, or sibling section | Heading tree | Trees; RIT-RAG sub-trees; a retrieval tool cards list child headings |
| Up to the enclosing section | Heading tree | INEX: mixed section and document evidence scored best |
| Links and backlinks | Links as written, typed by position (alone on a line includes, inline refers) | IWE; a linked-memory system keeps links to missing targets |
| Older or newer version | Parent link when the same path is added again; optional GumTree mapping | Storage tree; GumTree; immutable citations |
| Replaced by, or replaces | Close envelopes (supersede, correct) | Validity projection; StateAuditor |
| Out to the cited code | Evidence path, blob, and quote anchor | Evidence; W3C quote selectors; EA-Graph |
| From a file to the notes that cite it | Reverse of the `path:` location set | `path:` proposal |
| Files changed together | Git history | ROSE co-change |
| Same body under another envelope | Body hash | Body and envelope hashes |
| Read, then edited | Recorded typed events | Graphs from access |

- **Recombinations to explore.**

- **Each card shows its surroundings as counts**, for example "2 siblings · 3 backlinks · 1 newer version · replaced by 1 · cites 2 files." This joins query previews (counts before fetching), facets (counts over what matched), and the graph. The agent sees how many neighbors exist before it spends a call.

- A zero is a known dead end.

- **Edges are prefixes in the same query grammar**: `linked-from:<id>`, `cites:src/foo.rs`, `version-of:<id>`, `replaces:<id>`, `sibling-of:<id>`. The same CLI and card shape serve every move.

- An edge prefix with no words is navigation, like `show`. An edge prefix with words searches among neighbors (`linked-from:<id> "deploy"`), which keeps the accepted rule that search needs a text match.

- Filters compose with edges (`cites:src/** under:"Release"`).

- **One hop by default.** Progressive disclosure found one level helps and a second can hurt; IWE walks one hop by default. Further is an explicit repeated move.

- **Lateral moves obey search's rules.** They start only from returned hits (accepted rule 5), apply the same scope, accepted, and validity filters, never reorder hits, and never fill an empty result. The hub lint covers edges, so one hub note cannot become everyone's neighbor.

- **Time is a lateral axis, never the default.** Older versions and replaced records are reached only by an explicit `version-of:` or `replaces:` move. That gives "history is a separate query" its shape.

- **Code and notes in one loop.** `cites:<path>` moves from a file being edited to the notes about it. A note's evidence edge moves back to the cited lines, with the `evidence-changed` check.

- Co-change reaches notes about files that usually change with it. Only typed edges, and a signal git already records.

- **Session dedup on every move.** probe's rule of not returning what this session already saw applies to lateral moves too, so moving back and forth costs nothing extra.

- **What must hold before any of this is adopted.**

- A fixed list of edge types, each with one job. Overlapping mechanisms are the weak point for elegance.

- Card counts include only neighbors that pass the caller's filters, under the same visibility rule as facets.

- A test on the real cases: when a text query finds a near miss, does one lateral move reach the right note?

- Prior art on lateral navigation in search has not been searched yet.

### Answer or silence: query performance prediction (researched 2026-11-05)

- Eighteen plans in the test harness (`LEDGER.md`) sit on one curve between staying silent when nothing answers and finding the answer when it exists. Strict matching keeps silence near 0.96 with recall near 0.37; relaxed matching reaches recall near 0.88 with silence near 0.23.

- Information retrieval has a field for exactly this decision, query performance prediction (QPP), which estimates from retrieval statistics alone whether a query was answered well. None of it needs a model.

- **Score-based predictors.** [Meng et al.](https://arxiv.org/pdf/2305.10923) (2023) give the standard definitions. Weighted information gain (WIG) is the mean, over the top k results, of Score(q, d) minus the corpus score, divided by the square root of the query length, with k = 5.

- Normalized query commitment (NQC) is the standard deviation of the top-k scores normalized by the corpus score, with k = 100. The corpus score is the average score of the top 1,000 results.

- SMV combines magnitude and variance.

- **Calibration against a random null.** [Dias et al.](https://zenodo.org/records/22558176) (Zenodo, 6 September 2026, full manuscript restricted) compare the top retrieval score against a per-query random null and the shape of the score list, frozen and unsupervised. Oriented AUROC for detecting retrieval failure was 0.70 to 0.81 across three BEIR benchmarks, and abstaining on the signal was non-inferior to always answering.

- The formulas are not public, so this is a direction, not a recipe: score the query's top result against scores the same index gives to random queries of the same length drawn from the scope's vocabulary.

- **Choosing among query variants.** [Can QPP choose the right query variant?](https://arxiv.org/abs/2604.22661) (24 April 2026) reports that QPP "can reliably identify variants that improve end-to-end quality," and that simple pre-retrieval predictors often match post-retrieval ones. The abstract gives no numbers.

- **Proximity by pairwise distance, not span.** Tao and Zhai, "An exploration of proximity measures in information retrieval" (SIGIR 2007), compared span-based measures (Span, MinCover) with distance aggregates (MinDist, AveDist, MaxDist) on BM25 and KL-divergence models. Reported through secondary summaries only: distance aggregates, MinDist especially, improved retrieval; span measures generally did not, even normalized.

- Plans 12 and 13 used window coverage, the span-like form.

- **Pre-retrieval predictors.** Zhao, Scholer, and Tsegay (2008) proposed SCQ, a per-term collection-query similarity, aggregated as MaxSCQ, AvgSCQ, or SumSCQ; MaxIDF is the largest IDF of any query term. [Thomas et al.](https://microsoft.com/en-us/research/wp-content/uploads/2017/12/adcs17-thomas.pdf) (ADCS 2017) tested six pre-retrieval predictors and found MaxSCQ and MaxIDF best.

- They need only index statistics, before any search runs.

- **The calibration gap for coding agents.** [Agent Retrieval Bench](https://arxiv.org/abs/2607.24882) (27 July 2026) evaluates file-level retrieval for coding agents on 427 samples from 25 repositories: 345 positive, 50 natural no-gold, and 32 counterfactual controls. Selective thresholds calibrated with the counterfactual controls "do not improve selective success on natural no-gold cases", which the authors call a "calibration gap".

- This is the same wall the plan loops hit: silence calibrated on constructed near misses does not transfer for free.

- **Learned confidence from retrieval structure.** [RegimeAbstain](https://arxiv.org/abs/2609.22056) (18 September 2026, revised 18 October) fits a logistic Retrieval Confidence Score over up to nine structural features with no extra model call. On MuSiQue with an LLM judge, confidently wrong answers fell from 39.5% to 20.6% at 50% coverage, and the score trained on MuSiQue lost 0.5 points of AUC on 2WikiMultiHopQA.

- Dense retrieval, not BM25.

- Proposal for the next plans: decide answer-or-silence per query from WIG, NQC, MaxIDF, the top score against a same-length random-query null computed within the scope, MinDist of the matched query terms in the returned section, and the stability of the top result when each query term is dropped in turn. All are computable in SQLite from the FTS5 index and the vocab table.

- Fit any weights on dev only, as plans 14 to 17 did.

### Test-harness results so far (2026-11-05)

- Measured in `local workspace` on 1,376 records and 690 audited, pooled cases (492 dev, 198 holdout), with blind queries and the agent's own follow-ups. Gates in `harness/criteria.json` are unchanged.

- Details: `runs/LEDGER.md`.

- 35 plans sit on one curve between silence on empty cases (Q1) and recall (Q3). Strict section-local matching keeps silence near 0.96 with recall near 0.37; relaxed matching reaches recall near 0.88 with silence near 0.23.

- Author terms moved the curve, not the point on it. With terms on notes and on document sections, the best silence-passing plan (31) reaches Q1 0.904, Q3 0.708, Q2 0.779, Q4 0.595, Q5 0.718, R1 0.927, R2 0.103 to 0.128, against 0.372 recall for strict matching without terms.

- The strict ceiling rose from 0.474 to 0.609 with note terms; on document-gold cases from 0.510 to 0.635 with section terms.

- Structure presentation alone did not help: re-ranking a matched document's sections (plans 27, 32) and showing adjacent siblings (plan 34) left section hits unchanged. Section misses are mostly records never returned.

- The answer-or-silence decision is the bottleneck and is fragile: with 52 dev empty-gold cases one case moves Q1 by 0.019, and plan 31's recall moved 0.06 between refits on nearly identical labels. Query-performance features (WIG, NQC, MaxIDF, random-query null) did not beat a calibrated lexical scorer (plan 25), matching the calibration gap Agent Retrieval Bench reports.

- No plan has passed every gate yet; no holdout run has been made.

- Rounds 4 and 5 (plans 36 to 47) stay on the same curve. Porter stems in a derived column lift recall (plan 43: Q3 0.802, Q2 0.849) but answer empty cases (Q1 0.843); every evidence rule that restores silence also removes the gain (plan 47: Q1 0.941, Q3 0.642).

- Gating on scale-normalized features holds silence and loses recall (plan 42: Q3 0.441).

- Section choice is the binding gate. Even the union of all 22 runs with Q1 >= 0.90 shows the right section of a document passage in only 0.730 of cases (gate 0.75), although it finds the right document in 0.851.

- Judging the unjudged other sections of gold documents (667 cards, 71 answers) raised plan 43's Q4 from 0.608 to 0.649 only. Recall at Q1 >= 0.90 reaches Q3 0.756 and Q2 0.829 only with heading-path context, which the reviewed core excludes.

- Analysis: `runs/ANALYSIS-joint-ceiling.md`.

- Fusing three silence-passing matchers (plans 49, 53) is the first approach to pass silence, recall and exposure together within the settled rules: Q1 0.902, Q2 0.844, Q3 0.756, R2 0.091, with the first call capped per matcher; Q4 0.662. Adding a heading-context matcher (a labeled deviation, plans 54, 55) lifts Q2 to 0.869 and Q3 to 0.799 but Q4 only to 0.703 to 0.716.

- Every dev gate except Q4 passes; Q4 needs 56 of 74 document-passage cases and the best plan shows 53. Q1 has no margin (46 of 51) in every fused plan.

- At scale (16,906 records with node_modules and llms.txt distractors) the fused plans fall to Q1 0.569 with latency p95 121 to 125 ms (gate 50 ms); the single-matcher plans fall to Q1 0.63 to 0.69. Silence that holds on the curated corpus does not hold at scale for any plan measured.

- Ten document-passage cases are found by no plan: each query misses the gold section by a word found only in the label, scope or heading path, or by an inflection plus a paraphrase.

- Round 6 (plans 50 to 52) keyed the threshold on follow-up position and record kind. Cross- validation chose equal thresholds (0.77) for both, and recall fell (plan 51: Q1 0.922, Q3 0.713, Q4 0.622).

- Four false answers at confidence 0.75 to 0.95 on empty cases fix Q1 near 0.92; they are full matches of visible records, so they are a labeling question, not a matching one.

- With hints on silence and a judging agent (Sonnet reads every result and accepts or rejects it), every store keeps all 51 empty cases silent, and recall rises with how loose the store is: plan 11 with hints and doc-local disclosure reaches Q1 1.0, Q2 0.869, Q3 0.828, Q4 0.716. Haiku rejects many shown answers (Q3 0.762 on the same store as Sonnet's 0.819).

- Two gates still fail. Exposure (R2) is measured on what the store shows first, and a loose store floods it (0.318).

- Q4 stops at 0.73: the agent accepts a wrong section of the right document, or the store never shows the right section or document. A strict first call with loose later pages keeps R2 (0.091) but loses recall (Q2 0.794, Q4 0.622), because the agent accepts the strict call first.

- Progressive disclosure keeps a hub's weighted exposure at the gate (Metro options page: 0.318 by appearances, 0.077 to 0.102 by characters seen) but costs recall, because the agent judges from excerpts: Q3 0.828 with full cards, 0.742 with excerpts under three calls, 0.788 under a 1,800-token budget with outlines. Q4 stays between 0.676 and 0.716 under every reading rule.

- The instructions the tool gives the agent are part of the design, like a CLI's help or a skill file, and are tuned on dev only, with the holdout as the check against overfitting. Version 1 told the agent that an honest "nothing" beats a wrong answer; under the token budget it went back from 16 opened sections that held the answer.

- Version 2 says to accept once the section states the answer, and lets the agent ask an opened document's outline when the section is the wrong part.

- A judging agent makes the gates noisy. Sonnet repeats 90% of its decisions on a rerun; over a case's several decisions, two identical runs of the same plan differed in 37 found and 33 section results (Q2 0.859 against 0.804, Q4 0.703 against 0.662).

- Gate values from one run cannot rank variants that differ by less than about 0.05, and a single passing run is not a pass. **decided in the recorded study (2026-11-05):** each agent decision is the majority of three samples; if all three differ, the most cautious action is used (go on, then stop, open, outline, accept), so the agent never accepts on a split.

- Majority decisions are cached apart from single samples.

- With every agent decision resolved (an earlier grader bug scored unresolved steps as "stop"), the best plan fails only one dev gate. Plan 58b, plan 11 fused with plan 43's first call, with hints on silence, progressive cards with best-line excerpts, outlines on request, a 1,800-token budget and majority-of-3 Sonnet decisions: Q1 0.980, Q2 0.859, Q3 0.828, Q4 0.716, Q5 0.872, R1 0.957, weighted exposure 0.090, C1 median 92 tokens.

- Q4 is three cases short; automatic disclosure no longer adds Q4 and only adds exposure.

- After a gold-label audit of all 122 document-passage cases (5 golds moved, 2 cases dropped) and with heading context tried both ways, every leading plan passes every dev gate except Q4. Heading context used only to choose related pointers beside the cards (plan 58e) gives the best recall: Q1 0.961, Q2 0.884, Q3 0.851, Q5 0.923, exposure 0.087.

- Used as matched text (a deviation) it does no better. Q4 stays at 0.671 to 0.716 against 0.75, and varies by about 0.04 between draws of the agent's decisions even with majority voting.

- Tree moves from an indexing tool (an outline that merges small subtrees and counts hidden ones, expand one level, and neighbors limited to the heading tree and explicit anchor links) make deep sections reachable: the store shows the right section in 0.753 of document cases. The agent then spends its budget walking documents (700 outline requests, 206 neighbor moves), loses records elsewhere (Q3 0.851 to 0.819), and accepts a wrong nearby section in 8 of 40 cases reached through these moves.

- Q4 moves from 0.712 to 0.726, within its noise. Structural edges avoid lexical false positives; the remaining false positives are the agent's.

- **recorded scoring correction (2026-11-05): an answer is the one card the agent relies on.** The judged grading credited every card of an accepted result list ("top 5 per call"), and, after an addendum, the list a section was opened from as well. An agent that sees more within its budget then scores better without answering better, which contradicts "a weak hit is worse than silence".

- In plans 58b, 58e and 59, 13 to 18 found cases came only from the list behind a wrong opened card, and 32 to 35 more from lists whose gold card the agent may not have chosen. From now on the agent names the card it accepts, only that card counts toward Q2 to Q6 and must-not, and excessive movement is a cost: moves, tokens read and wrong acceptances per case are reported.

- Measured with that rule, the leading plans fail Q2 and Q4: plan 58e Q1 0.961, Q2 0.793, Q3 0.773, Q4 0.658; the tree-move plan 59 Q4 0.685 at a p95 of 1,809 tokens read per case. About one accepted answer in eight is wrong.

- The earlier judged gains were partly the inflated credit.

- **Precision: which card the agent relies on (2026-11-05).** In plan 58e's 43 accepted answers that miss the gold, the card the agent relied on was a record already judged a partial answer in 37, a judged non-answer in 5, and a must-not record in 1. The agent mostly takes a partial source, often a short note that restates part of a document, over the complete gold, and often takes the first card.

- Outside evidence: a 2026 study of shopping agents found position effects strongest at low reasoning effort, and suggests displayed attributes matter more than placement ([arXiv 2608.22697](https://ideas.repec.org/p/arx/papers/2608.22697.html)); a study of RAG systems found position bias marginal in realistic settings ([arXiv 2505.15561](https://arxiv.org/html/2505.15561v2)); work on redundant retrieval names incomplete and redundant context as causes of partial answers and ranks by completeness, not only relevance ([arXiv 2608.13956](https://arxiv.org/pdf/2608.13956), [arXiv 2604.24334](https://arxiv.org/pdf/2604.24334)). Measured next: each card shows how many query words its section covers, and cards are ordered by that coverage (plan 60).

- Result: neither coverage on cards (plan 60) nor instructions that demand a complete answer (instructions v3) moved it. Wrong records are 0.3% of acceptances; partial answers stay near 13%.

- Where Sonnet sees an answer, the Opus label judges saw a partial one. A blind relabel settled it (2026-11-05): two independent Opus judges, seeing only the task and the text, called 50 of the 62 "partial" cards the agent relied on full answers, at the rate of a credited control (51 of 62), and agreed on 119 of 124 items.

- The earlier "partial" verdicts came from judging each card against the gold rather than against the task. 49 records became equivalent answers, 3 equivalences were removed, and 7 credited gold cards are flagged for review.

- **holdout procedure ( 2026-11-05).** Plans 58e and 59 both pass every dev gate on these labels. Because the relabel covered only dev cases (52 of 52 changes), holdout is pooled the same way before it is graded: each plan's agent decisions on holdout are resolved and cached without computing any gate; the cards it relied on are judged blind by two Opus judges and a tie-break, by the dev rule; then each plan is graded once on holdout from those cached decisions, and checked at scale.

- Both plans go forward, which uses both holdout runs.

- **Holdout result (2026-11-05): no plan passes.** Plan 58e passes every dev gate (Q1 0.961, Q2 0.879, Q3 0.833, Q4 0.767, R2 0.097) and fails holdout Q2 (0.804) and Q4 (0.681, 47 cases, interval 0.54 to 0.80); every other holdout gate passes. Plan 59 fails holdout Q2 (0.775) and Q4 (0.681).

- At scale (16,906 records) plan 58e keeps silence for the first time (Q1 0.941, Q2 0.864) and fails Q4 (0.740) and latency (p95 186 ms against 50 ms). Q2 and Q4 fell 7 to 9 points from dev to holdout, although the holdout labels were pooled from these plans' own answers, which favors them.

- Both holdout runs are used; a new holdout claim needs a fresh case set.

### Agent-written queries and right-document misses (2026-11-05)

- Why plan 58e misses, read from its dev and scale traces (holdout outputs hide the gold):

- **Q4, 17 dev misses.** In 12 the right document was in the agent's view: in 5 it accepted another section of it, in 7 it never opened or accepted the gold section. In 5 the store never showed the document.

- Q2, 24 misses: 9 are a harness loop (below); in most others the agent saw the right record and passed it over, or the budget ended first.

- **A harness loop.** In 32 of 490 dev cases the agent repeated one move on one view (an outline already shown, or an accept whose pick named no card) until the 40-decision cap ended the case. All 9 such Q2 cases were misses.

- Fixed as measurement change 12 (a repeated move goes on).

- **The tested agent never searches.** It accepts, opens, asks for an outline, goes on, or stops; going on runs the next query prepared before the search. In the traces a follow-up could have used what the agent saw: hints naming the right document but not the "Trees" section, or the right heading ("Installation in bare React Native projects") under two other packages but not under `expo-file-system`.

- The ten cases no plan finds each miss "by a word found only in the label, scope or heading path" (above). Every source here that works puts query writing in the agent's loop: a retrieval tool's agent writes field-restricted queries, an indexing tool's agent drives search and outline, and Aaron Tay's agent "broadened, corrected spelling, and switched scope."

- **Some wrong sections are answers.** The dev label check judged cards of other records blind, but not other sections of a gold record; those had been judged against the gold. A blind check of the 12 such cards the agents relied on (two Opus judges, agreed on 24 of 24 with 12 controls) found 5 that answer the task.

- Added as judged sections (dev only).

- **Two-part tasks.** In some misses the agent opened the gold section and went on, because the task asks two things and the section answers one.

- Outside evidence, recent:

- [MADQA](https://arxiv.org/html/2603.12180v1) (12 March 2026, 2,250 questions over 800 PDFs): "right doc, wrong page" is 23.0% of 3,273 agent errors. Accuracy rises with how far an agent changes its query between steps (Claude Sonnet 4.5 drifts most, 0.38 per step; GPT-4.1 Nano least, 0.10), and agents loop with small query changes on problems they do not solve.

- Older than four months; cited for its error split.

- [RepoNav](https://arxiv.org/abs/2609.08355) (8 September 2026, EMNLP 2026): code agents reach the right file and choose the wrong function. A file-centred view that lets the agent compare sibling candidates narrows that gap; ablations credit the organization of evidence, not more structure.

- The abstract gives no numbers.

- [Long-horizon search agents](https://arxiv.org/abs/2608.01913) (3 August 2026): misses split into a retrieval gap (never found) and a use gap (found, not used). Useful evidence often comes early and agents keep searching; the best agents repeat fewer queries.

- [A note on agent queries](https://github.com/rmax-ai/rmax-ai.github.io/issues/80) (19 September 2026): agent queries are explicit and iterative and reuse "terms taken from earlier tool results." An argument, not a measurement.

- Stopping by evidence coverage ([2608.13237](https://arxiv.org/abs/2608.13237), [2609.26086](https://arxiv.org/abs/2609.26086), [2608.01285](https://arxiv.org/abs/2608.01285), August and September 2026) treats "enough evidence" as a learned or embedding-based judgment; none tests answers that need two sections.

- Loop guards in agent SDKs (OpenRouter's agent SDK, revka, pi loop-guard; 2026) end repeated identical tool calls; none is a measured evaluation.

- **decided in the recorded study (2026-11-05):** fix the loop (change 12), check the wrong-section labels blind (done), and let the agent write its own follow-up queries (change 13), then build plans on 58e with them. The side-by-side view of one document's candidates (RepoNav) and two-card answers for two-part tasks follow, in that order.

- **Measured on dev (2026-11-05, Sonnet majority of 3, 1,800-token budget, single-card credit).**

| Step on plan 58e | Q1 | Q2 | Q3 | Q4 | R2 |
|---|---|---|---|---|---|
| Before (old labels) | 0.961 | 0.879 | 0.833 | 0.767 | 0.097 |
| Wrong-section relabel only | 0.961 | 0.879 | 0.833 | 0.808 | – |
| + loop guard (plan 61) | 0.961 | 0.909 | 0.874 | 0.836 | **0.123** |
| + up to 2 agent-written queries (plan 62) | 0.941 | 0.899 | 0.885 | 0.808 | 0.090 |

- The loop guard gave the gain (Q2 +6 cases). It also lets cases that used to end in a loop go on reading, and three Metro cases then read more of the Metro options page, so exposure rose past 0.10.

- Agent-written queries added no net Q2 or Q4 gain over plan 61 (Q2: 5 cases gained, 7 lost; Q4: 3 gained, 5 lost). The agent wrote 165 queries in 115 of 490 cases, and they were sensible ("word tree meanings", "git-ssh-access repoRoot").

- But it wrote them at once, in place of the prepared follow-up, which in plan 61 often found the gold. The prepared follow-ups are themselves queries a model wrote from the task, so the hypothesis that the tested agent "never searches" was half right: it never reformulated from results, and doing so did not help on these cases.

- What the agent's queries did change is reading: exposure fell to 0.090, Q3 and Q6 rose slightly, and wrong accepted answers stayed at 1.2% of answers.

- Plan 62 passes every dev gate. This is not evidence that it passes on new cases: 58e also passed dev and fell 7 to 9 points on holdout, and on holdout 11 of 193 cases hit the decision cap, so the loop guard would recover only about 2 holdout Q2 cases at the dev rate.

- Scale latency (C2) came from the prototype, not the design: three full scans per call. Each term's FTS match was joined to records with `length(r.body)>0`; neighbor lookup walked every record and every section; and 58e's related pointers walked every section.

- Eligible sections and records computed once per scope, record-to-section and path indexes, and an index of heading-path words give identical output on every call (1,470 dev and 450 scale queries, cards, hints and related pointers) and cut the per-call p95 at scale from 241 ms to 39 ms (gate 50 ms), 9 ms on the curated corpus. Plan 62 uses them.

- **Plan 62 passes every dev gate and every scale gate (2026-11-05).** Dev: Q1 0.941, Q2 0.899, Q3 0.885, Q4 0.808, Q5 0.923, Q6 0.891, R1 0.958, R2 0.090, C2 p95 2 ms. Scale (16,906 records, dev cases, quick: no R1, R2 or determinism): Q1 0.922, Q2 0.889, Q3 0.868, Q4 0.795, Q5 0.897, Q6 0.909, C2 p95 22 ms.

- It is the first plan that holds silence and recall at scale. It has not been tested on unseen cases: both holdout runs of this case set are used, and plan 58e fell 7 to 9 points from dev to holdout.

- Margins on dev are 2 cases for Q1 (48 of 51, gate 46), 4 for Q4 (59 of 73, gate 55) and 9 for Q2. A pass claim needs a fresh holdout set.

- **Two robustness checks pass (2026-11-05, no tuning).** A second independent draw of every agent decision on dev (separate cache) passes every gate: Q1 0.961, Q2 0.884, Q3 0.862, Q4 0.808, R2 0.092; the two draws differ on 5 of 198 Q2 cases and 4 of 73 Q4 cases. A full scale grade with every gate passes: R1 0.942 over 2,061 records, R2 0.087, determinism 1.0, C2 p95 30 ms, Q1 0.922, Q2 0.889, Q4 0.795.

- So the dev pass is stable against the agent's noise and against 12 times more records. It still says nothing about unseen cases.

- A draft procedure for a fresh holdout set (about 450 cases, part of them from sources no shard used, labels pooled from six plans, one grade per plan) is in `HOLDOUT2-PROCEDURE.md`, pending approval.

- **Fresh holdout h2 (2026-11-06): plan 62 fails on exposure only.** Procedure approved before execution and pre-registered before any case existed (runs/H2-PREREGISTRATION.md; file hashes checked unchanged at grading). 460 sealed cases: 310 new cases over the existing records (part A) and 150 on a new shard of 97 records from two additional external repository populations (part B), written by new Opus sub-agents, with blind queries by Claude Haiku and Codex gpt-6-luna. Labels were pooled from six plans and judged blind before any grade (2,270 items; two Opus judges agreed on 2,172, a third decided 98; runs/H2-LABELS.md).

- One grade per plan:

  | Gate | Need | 62 | 62 at scale | 58e (reference) |
  |---|---|---|---|---|
  | Q1 empty-correct (106) | >= 0.90 | 0.991 | 0.981 | 0.991 |
  | Q2 identifier, phrase, doc (307) | >= 0.85 | 0.925 | 0.935 | 0.902 |
  | Q3 core | >= 0.75 | 0.936 | 0.944 | 0.903 |
  | Q4 section hit (271) | >= 0.75 | 0.908 | 0.919 | 0.871 |
  | Q5 lateral (40) / Q6 paraphrase (48) | >= 0.60 / 0.35 | 0.850 / 0.854 | 0.925 / 0.854 | 0.875 / 0.833 |
  | R1 self-retrieval | >= 0.90 | 0.953 | 0.942 | 0.953 |
  | **R2 exposure, weighted** | <= 0.10 | **0.225** | **0.184** | **0.185** |
  | G1 to G5, C1, C2 | | pass | pass (C2 17 ms) | pass |

- Part B, the new content, is lower and still above every recall gate (plan 62: Q1 0.972, Q2 0.868, Q4 0.868; part A: 1.000, 0.949, 0.923). Answer-or-silence and section choice generalize.

- Exposure does not. The failure is one record: a status document in the new
  external shard, included as a hub as the brief asked.

- In its scope it is shown in 33 of 45 cases where it is not the answer, usually as one of the first three cards, ≤ one card per list. The agent opened it in 6 of those cases (about 0.11 of the share on its own) and saw it only as a pointer in 27 more (about the same again).

- On dev, exposure passed only on hubs the plans had met (the Metro options page). Hub ranking at the store, not the agent's reading, is the open problem.

- Plan 58e at scale also fails exposure (0.184) and latency (176 ms; it has no indexed matching). Both runs on h2 are now used for these plans; any change tested next needs a new set (h3) before a claim, per the procedure.

- **Option 1, hub demotion in the store, tested (2026-11-06; h2 now counts as dev).** Three content-only hub signals, none from access counts:

- Share of the scope's shared vocabulary (words in two or more records). On the curated corpora it marks both known hubs: the new status document holds 0.71 against 0.45 for the next record, the Metro page 0.75 against 0.47.

- It also marks the FINDINGS copy and a migration plan, which are often answers. At scale, about 800 distractor READMEs per scope dilute it to 0.14 against 0.11, and it marks neither hub.

- Links: only 39 of the hub's 98 shown cards link to another stored record, and only 13 to a record in the same list.

- Heading match breadth (other records' headings fully matched in one section): 9 for the hub against 6, with 8 to 12 for ordinary documents elsewhere.

- Plan 63 used signal 1 and moved a hub's cards after every other card of a call, last among hints and never as a related pointer. On h2 the hub moved from cards 1 to 3 to cards 4 and 5, yet stayed in view, because most lists hold five or fewer matches.

- Exposure fell only from 0.225 to 0.211, and the agent still opened it in 6 non-answer cases. Recall was unchanged on h2 (Q2 0.928, Q4 0.911) and on dev (Q2 0.899, Q4 0.808, R2 0.090).

- So reordering cannot meet R2. The hub has to leave the list or lose its excerpt, and a store can do that only if it can tell a hub from a broad answer.

- Content alone did not do that at scale.

- **Option 2, cards easier to reject, tested (2026-11-06).** Limit first, from plan 62's h2 traces: the hub's 0.225 is about 0.113 from the 6 non-answer cases where the agent opened it, and about 0.112 from pointer appearances. So with no opens at all, R2 would still be about 0.11.

- Four of the 6 opens were on empty cases: the agent opened the near miss, rejected it and stayed silent, which is correct. Plan 64 keeps the accepted 160-character excerpt and centres it on the first matched query word.

- The old excerpt began at the start of the best line and was often cut before the match. On the 53 cases of the hub's scope (h2):

  | Plan | Q1 (14) | Q2 (27) | Q4 (22) | R2 | Hub opened (non-answer) |
  |---|---|---|---|---|---|
  | 62 | 0.929 | 0.741 | 0.727 | 0.225 | 6 |
  | 64 | 0.929 | 0.889 | 0.909 | 0.173 | 4 |

- Exposure falls but stays above 0.10, as the limit predicts. Recall in this scope rose by 4 Q2 and 4 Q4 cases: the agent picks the right card more often when it sees the words around the match.

- One scope and small counts. Whether this holds over all cases is the next measurement (full dev and h2; every card's text changes, so every decision is new).

- **Excerpt quality, iterated on plan 64 (2026-11-06, h2 as dev).** Research: agents open fewer pages when snippets state the answer fact ([arXiv 2607.10198](https://arxiv.org/html/2607.10198v1), July 2026, observational); models that answer from snippets alone give more outdated answers and unfaithful citations ([arXiv 2610.21731](https://arxiv.org/abs/2610.21731), October 2026); a retrieval tool scores passages by lexical overlap and cuts the best one to 25 words. A proxy without the agent: how often the excerpt of the gold section contains the gold quote.

- Plan 64's centred best line: 45% on dev, 27% on h2. Best sentence by distinct query words: 58% and 36%.

- BM25 sentence ranking: 48% and 35%. BM25 as tie-break: 50% and 36% (position, earliest first, breaks ties better).

- The ceiling is low because blind queries name the topic, not the fact. With the agent, on all 460 h2 cases:

  | Plan | Q2 | Q4 | Q6 | R2 | Opens of non-answers |
  |---|---|---|---|---|---|
  | 62 | 0.925 | 0.908 | 0.854 | 0.225 | 136 |
  | 64, centred line | 0.945 | 0.937 | 0.896 | 0.188 | 155 |
  | 65, best sentence | 0.925 | 0.897 | 0.812 | 0.189 | 136 |
  | 66, 65 plus cards narrowed to the best 600-byte block | 0.928 | 0.860 | 0.833 | 0.207 | 166 |

- Plan 64 beats 65 on Q4 by 14 cases to 3 (sign test p = 0.013), and 62 by 11 to 3 (p = 0.057). An excerpt that contains the answer quote more often did not help the agent choose the section; the centred line, which shows more of the context around the match, did.

- Narrow block cards made opens cheaper and more frequent and lost Q4, because the best block holds the quote in only 77% of gold sections. No excerpt variant lowered opens of non-answers.

- Q1 stayed at 0.991 and the wrong-answer share under 1% in every run. Plan 64 has not been run on the full dev set yet.

- **Key terms and fuller windows (2026-11-06, h2 as dev).** Older prior art: keyphrases shown beside the title and snippet helped users judge relevance, and keyphrases are query-independent where snippets are not ([Gutwin et al., Information Retrieval 2008](https://link.springer.com/article/10.1007/s10791-008-9044-1)); explanations of why a result matched raised agreement on the relevant document by 23% ([arXiv 2111.01314](https://arxiv.org/pdf/2111.01314), 2021). No study of agents was found.

- Plan 64 also had two defects: its best line could be a heading line, which the card already shows, and lines average 99 characters, so most of the 160 went unused. Tested on all 460 h2 cases, each against plan 64 case by case:

  | Plan | Q2 | Q4 | R2 | Opens of non-answers |
  |---|---|---|---|---|
  | 64, centred best line | 0.945 | 0.937 | 0.188 | 155 |
  | 67, three BM25-weighted section terms (no query or heading words) + centred line, 160 in total | 0.935 (+6/-9) | 0.923 (+3/-7) | 0.195 | 149 |
  | 67w, the same terms on top of the full 160 (labeled deviation) | 0.935 (+6/-9) | 0.923 (+5/-9) | 0.214 | 154 |
  | 68, a 160-character window across lines, heading lines skipped | 0.941 (+5/-6) | 0.919 (+4/-9) | 0.177 | 152 |

- None differs from plan 64 by more than noise (sign tests p >= 0.27). Plan 64's own lead over plan 62 is borderline (Q4 11 against 3, p = 0.057), and each plan is one draw of the agent's decisions.

- So once an excerpt shows the context around the match, its format is at a plateau. Opens of non-answers stay between 136 and 166 in every variant: the agent opens a plausible near miss to verify it before it answers or stays silent, whatever the card shows.

- Exposure cannot be fixed at the card; it depends on that verification policy or on how R2 counts a rejected open.

- **The same excerpt plans with other agents (2026-11-06).** The harness now also runs Codex and Cursor models in the agent's seat (codex exec, cursor-agent; one call per decision, from an empty folder, same prompts and parser). A quick check on a fixed sample of 80 h2 cases (40 doc-passage, 20 empty, 20 other; seed 20261111), one sample per decision, against Sonnet's majority-of-3 results on the same cases:

  | Agent | Plan | Q1 (20) | Q2 (48) | Q4 (40) | Opens of non-answers |
  |---|---|---|---|---|---|
  | Sonnet | 64 / 67 / 67w / 68 | 20 / 20 / 20 / 20 | 45 / 43 / 43 / 44 | 38 / 35 / 35 / 37 | 30 / 27 / 26 / 21 |
  | GPT-6.1 Sol | 64 / 67 / 67w / 68 | 20 / 20 / 20 / 20 | 46 / 44 / 45 / 45 | 39 / 37 / 37 / 38 | 25 / 18 / 17 / 23 |
  | Grok 4.7 | 64 / 67 / 67w / 68 | 20 / 20 / 20 / 20 | 42 / 41 / 43 / 42 | 36 / 34 / 36 / 36 | 13 / 8 / 9 / 10 |

- The direction is the same for all three agents. Plan 64 is best or tied on recall.

- Key terms (67, 67w) and the full window (68) cut opens of non-answers: by 10% to 43% for 67, and by 8% to 30% for

- Key terms cost 1 to 3 recall cases. Every agent stays silent on all 20 empty cases.

- Grok opens far less than the others and finds a little less. The counts are small and each run is one sample, so no single difference is significant.

- What is consistent is that plan 64 has the most opens with every agent: the terms help an agent reject a card without opening it, and they cost a little recall. A paired look at context shows the cost.

- Every card in plans 67, 67w and 68 is longer than plan 64's one-line excerpt (99 characters on average), and every list holds several cards. So plan 64 shows the fewest tokens per case and per answer found with all three agents.

- Sonnet over 460 h2 cases: 568 tokens per case and 791 per answer for plan 64, against 585 to 619 and 827 to 871. More cases use more tokens than fewer in every comparison (p < 0.001).

- The opens saved (6 of 460 for plan 67) do not pay for the longer cards. plan 64 was retained.

- **Plan 64, second draw, dev, and own-query timing (2026-11-06).** A second independent draw of plan 64 on all h2 cases repeats the first: Q2 0.945 both times, Q4 0.937 and 0.926, R2 0.188 and 0.180; the draws differ on 12 Q2 and 11 Q4 cases. Plan 64's gain over plan 62 on h2 is real.

- On dev, plan 64 passes every gate (Q1 0.922, Q2 0.874, Q3 0.865, Q4 0.781, R2 0.098, C2 5 ms) but is no better than plan 62's two draws (Q1 0.941 and 0.961, Q2 0.899 and 0.884, Q4 0.808 twice). Case by case, plan 64 never wins on dev (Q2 0 against 5, p = 0.06, and 1 against 3), so the centred excerpt helps on h2's long status and plan lines and not on dev.

- Plan 69 offered the agent its own query only once the prepared queries ran out, to stop own queries from replacing a prepared query that would have found the gold (all 7 of plan 62's dev Q2 losses). On h2 it lost against both plan-64 draws: Q2 0.919, Q4 0.897 (4 against 15 cases, p = 0.02), R2 0.206.

- On new content an early own query helps; the dev pattern was noise.

- **Where this leaves the frontier (2026-11-06).** On h2 (as dev), plan 64 is ahead of 62, 65, 66, 67, 67w and 69 on recall and on tokens per case. Plan 68 has a lower R2 (0.177) at more tokens and no significant recall difference, so 64 and 68 trade exposure against context.

- On dev, 62 and 64 are level. Tried and flat or worse: excerpt format (5 variants), hub demotion, the timing of own queries, BM25 in excerpts.

- Untested: telling the agent about the `under:` filter, two-card answers for two-part tasks, and instructions about when to open a near miss. Every plan fails R2 on h2 on one status document.

- The limit measured on plan 62's traces is about 0.11 from pointer cards alone, before any open, so no card or agent change measured so far can pass it. Passing needs the writer to mark status documents (option A) or a policy decision on how R2 counts a rejected open.

- A new sealed set (h3) is worth building only after that. Plan 64 and every excerpt variant were tuned on h2, so h2 no longer tests them.

- **The under: hint, tested (2026-11-06).** Plan 70 is plan 64 with one more sentence in the agent's query note: to look inside one part of a document, add under:"<words from a heading>" from a heading seen in the results. The store applies the filter (checked first: under:"Trees" returns only that section).

- On all 460 h2 cases: Q2 0.935, Q4 0.919, R2 0.191. Against both plan-64 draws it is within noise and slightly lower (Q2 5 against 8 and 4 against 7; Q4 4 against 9 and 3 against 5; p >= 0.27).

- The agent used under: in 33 of its 248 own queries (30 cases). 17 of those cases were empty, and in the 13 with a gold it found 7. The agent rarely needs to narrow inside a document.

- When it does, under: does not find more than its other moves. The hint stays out of plan 64.

- **Correction after review (2026-11-06).** For plan 64, the record that fails
  R2 on h2 is not the status page but the same external shard's build plan
  (`docs/PLAN.md`, 27,130 characters): 0.188 and 0.180 in the two draws. The
  status page is second, at 0.183.

- With plan 64, opens give about one fifth of the exposure; without opened views the shares are still 0.145 and 0.127. The statements above about one status document, half from opens, and about 0.11 from pointer cards hold for plan 62's traces, not for plan 64.

- Also corrected: author terms on a section reach matcher B's stemmed table, so they count as full matches for coverage and the strict rule, not at half weight (the half-weight terms-only path was empty in all 5,678 dev term evaluations). Details: `PLAN_97_ARCH.md`.

- **The plan verified as a CLI (2026-11-06).** The harness was changed to work like `linn` (`harness/linn.py`, `--cli`, measurement change 14). The agent had only a short protocol plus the plan's own help text, typed one command per turn, and ended with `ANSWER <fetch>` or `NOTHING`.

- Same store, same limits, same gates, Sonnet majority of 3. Dev: every gate passes (Q1 0.922, Q2 0.939, Q3 0.914, Q4 0.836, R2 0.067).

- That is better than the per-view harness on Q2 (15 cases against 2, p = 0.002). h2: every gate but R2 (Q1 0.991, Q2 0.938, Q4 0.915, R2 0.159 on the status page), within noise of the tested runs. The CLI agent opened fewer non-answers (95 against 143 and 155 on h2), read a little more (625 against 568 tokens per case), and accepted slightly more wrong answers (1.2% to 1.8% of answers, against under 1%).

- So the help text carries the tested instructions; per-view instructions are not needed.

- The review and the run also found defects, now fixed:

- `outline` needed a record id that no output printed.

- Unparsed replies counted as NOTHING votes.

- A timed-out model call lost a round of decisions.

- **Scope from the git remote (2026-11-06).** The earlier rule took a checkout's scope from its path under `local workspace` (or its git common directory). The plan now names a repository by its git remote: the URL path of `upstream`, else `origin`, else the only remote, without host or `.git`.

- Checked on every checkout in `local workspace`, it gives the corpus scope for all 21 scopes. It is also right where the path rule is not:

- extra clones and worktrees of one repository (an extra clone gives the same repository identity as its primary checkout)

- forks, whose `origin` is the fork

- a worktree whose git directory is outside `local workspace` (this checkout)

- It gives the same scope on every machine and account. A repository with no usable remote gets `local:<path>`.

- Outside a repository, the caller is unscoped. Details: `PLAN_97_ARCH.md` 2.1; implementation: `harness/linn.py` `scope_of`.

- **Writer fields removed, a small embedding model in their place (2026-11-07, sample results).** the recorded change removed `add` take no `evidence`, `terms` or `section_terms`, and that the model itself run at ingest and at query time (no derived tables or terms). Ablation (plan 71, the plan without the three fields), CLI mode, all 490 dev cases: no gate changed significantly, but Q6 lost 5 paraphrase cases (53/55 to 49/55, p = 0.22).

- Each lost case had been reached only through author terms. Recovery from the text is not possible for evidence: 279 of 731 evidence paths are not in the body, and links or backticks recover none.

- Plan 74 adds model2vec potion-base-8M (256 dimensions, 30.7 MB in f32): every index block is embedded at build time and every query at search time. Exact cosine search; in production a libsql vector index and model2vec-rs in the binary.

- Vector similarity does not separate gold from non-gold (gold median 0.56, best non-gold median 0.58), so vector hits never become cards and never decide answer or silence. They are near misses only.

- Results on a 140-case dev sample (55 paraphrase, 51 empty, 34 other; Sonnet majority of 3):

  | Run | Q1 | Q2 | Q4 | Q6 | Median tokens per case |
  |---|---|---|---|---|---|
  | Plan 64 (with author terms) | 46/51 | 29/29 | 6/7 | 53/55 | 447 |
  | Plan 71 (no author fields) | 45/51 | 28/29 | 6/7 | 50/55 | 447 |
  | 74a: vector near misses on silence | 45/51 | 29/29 | 6/7 | 53/55 | 602 |
  | 74b: 74a, and vector related pointers | 46/51 | 27/29 | 5/7 | 50/55 | 682 |
  | 74c: 74a, only neighbours that share a query word | 46/51 | 29/29 | 6/7 | 53/55 | 488 |

- 74b replaced the heading-match related pointers with vector neighbours. In s3b-c-0066 the heading pointer was the answer and the neighbours were worse.

- Rejected.

- 74a's extra tokens are almost all on empty cases (median 516 to 832): a vector index always returns neighbours. 74c keeps a neighbour only if its block shares a content word with the query. Offline on the sample, this halves vector hints on empty searches (193 to 98) and keeps 12 of the 13 gold pointers.

- Build: 5.6 s for 6,096 blocks. Search latency p95 2.2 ms.

- Two defects found and fixed: `forget` left the vectors of a forgotten record (G4); and a fetch id whose body is shared by records in two checkouts resolved to the wrong checkout's record (one G1 violation, s5-c-0148). The remaining G1 violations and the one G2 hit in these runs all come from the agent's `scope:` searches into another checkout.

- Full runs of 74c (dev 490 cases, h2 460), against the ablation and plan 64 on the same cases:

- every gate passes except G1 on dev (the 2 `scope:` cases, as in the other two designs) and R2 on h2 (0.162, the status page, as before)

- against the ablation, Q6 gains 7 cases and loses 1 over both sets (p = 0.070); no other gate moves by more than 2 cases

- against plan 64 with author terms, equal on every gate (Q6: dev 53/55 each; h2 46/48 against 44/48)

- 4% (dev) to 7% (h2) more tokens per case

- Model size, offline on all dev queries with a silent search: the gold is reached by a vector hint in 36 cases with potion-base-2M (8 MB), 48 with potion-base-8M (30.7 MB) and 50 with potion-retrieval-32M (130 MB; 6 cases gained against 4 lost, p = 0.75, and 17% more hints on empty searches). 8M stays.

- So the writer fields can go: the model at run time does what the author terms did, without derived data. Details: `PLAN_97_ARCH.md` 1.5.6, 1.5.9 and 3.1.9.

## Recent work, July to November 2026

- Every study, thread, and project here was dated on its own page between 4 July and 4 November 2026. Product docs cited in passing are not dated.

- Older finds from the same searches (Letta Context Repositories, February; Claude Code Auto Dream, March; AdaMem and the memory-poisoning study, June) were left out.

### Filesystem-based memory, read as a constraint

- [Filesystem-Based Memory for LLM Agents: Organization, Evolution, and Sustainability](https://arxiv.org/abs/2607.26637) (29 July 2026) is the closest study of a store like this one. Long-term memory is a tree of markdown files.

- A management agent writes and reorganizes it with six generic file operations and regex search. A search agent answers with view, regex search, a table of contents, and section reads.

- The models are gpt-5.4-mini for management and search, with nano and gpt-5.4 for strength studies. The stores are small, the same scale as this one: 88 KB to 151 KB on the conversational sets (Table 3).

- **Raw beats rewritten, unless the rewrite preserves.** In Table 1, a verbatim dump scored 84.2, 78.1, 69.0, and 77.6 on LoCoMo, PersonaMem 32k, PersonaMem 128k, and REALTALK. A reorganizer told to condense fell to 41.2 on REALTALK.

- The agent-curated store fell to 37.5 on PersonaMem 32k. Foldered sessions led most consistently.

- No shape won everywhere.

- **Organization lowers cost, not error.** On PersonaMem 32k a reorganized store cost 1.4 cents a query against 4.0 for the dump, and 1.6 against 3.9 at 128k. LoCoMo and REALTALK were near parity.

- The conclusion says the results "caution against treating organization as an end in itself."

- **Agent-made structure is unstable.** Two builds that differed only in a corrected tool description produced 2 files and 29 files. Taxonomy adherence eroded as most stores grew; only the strongest management agent held it.

- Skill stores shrank from 114 files (nano) to 45 (mini) to 16 (gpt-5.4) for the same input.

- **Agents do not clean up.** Files created early were "only edited, never moved or deleted," and early memories "overwhelmingly survive." The LoCoMo store ended 35% larger than its input. Changed preferences were often recorded as dated updates, which left the old fact standing as a live trait.

- **A ranked-search tool did not add accuracy.** Adding BM25 beside regex search and section reads scored 86.7 against 86.1 on LoCoMo, 64.3 against 66.7 on PersonaMem, and a tie on skills (net −2, p≈0.85). The search agent called it 54 times on LoCoMo and twice on PersonaMem.

- The paper's summary is that adding a tool "changes agent behavior but not outcomes."

- **A shell beat bespoke tools.** Bash over the same store reached 82.9% on the 140-task skill chain, the best mini-tier result under any tool set, against 76.4% with the default tools.

- **The consumer matters.** Raw episodes suited a strong execution agent; distilled guidance suited a weak one.

- **Limits, as the authors state them.** Fixed-size benchmarks, 140 tasks and one conversation length, single runs where several orderings sit within run-to-run variation, and quality metrics that are "largely blind" to changes in organization. They read the deltas as "suggestive rather than settled."

- Read as constraints on this store:

- Keep the original bytes as the record. A rewrite, summary, or reorganization is never the only copy.

- This is already the rule; the paper measures what breaking it costs.

- Structure must be derived deterministically, from headings, evidence paths, and explicit links. A layout an agent maintains will drift with the model and with a tool description.

- Cleanup is an explicit operation with its own actor: close, forget, and lint. The writer will not do it.

- A change of fact is a close of the old record, not a dated line appended to it.

- At this size, ranked search over notes is not where accuracy comes from. The case for the store has to rest on what grep and a file tree lack: the accept bar, scope and validity filters, evidence checks, and a cheaper answer when material is large.

- The tool is a shell command. The agent already drives a shell well.

### One call, structured results, sections on demand

- a retrieval tool (3 August 2026) filters candidates with a Boolean query over titles, headings, sections, and metadata, ranks them with an interchangeable ranker, shows structure-rich result cards, and fetches only the sections the agent picks. It used 20.7% to 50.6% fewer tokens and was more accurate than the strongest search-then-visit setup on each of three collections.

- Boolean filtering improved every ranker tested.

- [A deep agentic search study](https://arxiv.org/abs/2608.01507) (2 August 2026) on SWE-QA: a single index search answered 65.2%, a planner handing grep work to a subagent 46.2%, and the single search cost less than half as much per correct answer. 41.8% of the agentic failures were at the planner-to-subagent hand-off, usually silent and confident. The index used embeddings, so this compares one call against a search agent, not BM25 against vectors.

- [AgentMemRec](https://arxiv.org/abs/2610.34673) (26 October 2026) treats selection as budgeted slate recommendation: candidates, ranking, then a slate that fits the context budget. No model call at write time or at selection.

- It matched a model reranker at 3.2 times lower serving cost and beat A-Mem, a graph retrieval system 2, and others by 10 to 31 points on LongMemEval multi-session questions.

- [MemoType](https://arxiv.org/abs/2610.11573) (8 October 2026, NeurIPS 2026) routes each query to a retrieval strategy by memory type, with up to 16.18% higher Recall@1. An earlier proposal here, that a preference in scope be eligible without a term match, is withdrawn (accepted 2026-11-04): it makes every query in a scope return the same notes (see "Location narrows, it does not answer").

- A rule that must always apply belongs in `AGENTS.md`.

### Recording

- [AutoIndex](https://arxiv.org/abs/2607.18603) (21 July 2026) held BM25 fixed and searched over programs that slice, enrich, normalize, reweight, or reorganize documents before indexing, keeping a change only if retrieval improved on validation. Over full-document BM25 on 8 CRUMB tasks: +8.4% Recall@100 and +8.3% nDCG@10 on average, ≤ +30.5% and +43.6%.

- The representation is the lever. Sections, the parts column, and heading paths are what to tune, against a set of real cases.

- [Agentic context management](https://arxiv.org/abs/2607.21503) (23 July 2026) treats memory as a lifecycle and says consolidation must keep each item's source record. Crude summarization causes an "accuracy cliff." The 92% LongMemEval score is the authors' own system.

### Stale claims and evidence

- [EA-Graph](https://arxiv.org/abs/2608.04278) (4 August 2026) anchors each verification claim to the artifact content that established it, tracked below file level. With Haiku, anchored claims beat prose notes and no memory in all seven worlds (p = 0.0156 for each paired comparison).

- With Sonnet the anchored condition was perfect but the controls often hit the ceiling, so the contrasts were not significant. No session fabricated withheld content.

- The study is small: 42 sessions on generated repositories. It supports the evidence pointer, and span-level anchoring supports a quote anchor beside the blob id.

- [StateAuditor](https://arxiv.org/abs/2608.01619) (3 August 2026) found that agents which knew a stored value was old still planned with it. A model proposes old-to-new transitions; deterministic code checks the quotes and the chronology.

- Accuracy rose from 0.686 to 0.736, and a matched control with the same evidence and call budget gained only 0.6. Proposal: a hit that replaced another record names the record it replaced, so the agent drops the old premise.

- [Agent Zero Memory](https://arxiv.org/abs/2608.29606) (30 August 2026) gives every item its origin, timestamp, and evidence pointer, lets the reader cite only evidence it opened, and abstains when evidence is missing. It reports 95.60% on LongMemEval, with a 3.4-point spread and about 30 times cost spread across eight backbones.

- Retrieval is embeddings plus lexical.

- [ChurnBench](https://arxiv.org/abs/2609.11515) (10 September 2026) separates freshness errors, right when retrieved and wrong when graded, from reasoning errors. Disabling tiered refresh raised freshness errors from 4 to 45 at 28 days.

- It does not test memory stores. The part that carries is counting the two error kinds separately in an evaluation.

### Practitioners

- [Cal Paterson's memoryfields](https://calpaterson.com/memoryfields.html) (August 2026) proposes a zip of markdown pages under about 8 KB, with an optional SQLite vector index that is "a deletable cache, not the system," and recommends citations. In [the HN thread](https://news.ycombinator.com/item?id=49508317) (about 1 September 2026, 191 points), `nzach` wrote that "a really good AGENTS.md file almost always produce better results than enabling memory." `dataviz1000` said one poisoned line becomes "noise, will drift." `docheinestages`: "Nothing beats curated data." `sangwook` asked when a new fact updates a page and when it makes a new one, and nobody settled it.

- The author said semantic search is "substantially better than keyword search even for small corpuses," which is one person's claim against the settled rule.

### Instruction files, skills, and what on-demand loading keeps

- **On-demand loading keeps about half the gain.** [A study of skill revisions](https://arxiv.org/abs/2610.04832) (4 October 2026) over 2,608 revision pairs from 3,159 skills found that adding a checkable rule raised single-answer compliance by 0.41 across 16 open models, the rate of taking the required action by 0.23 across four sandboxed agents, and blind-judged correctness by 0.10 across three. Most of the gain came from rules that named a command or path the old skill did not.

- When the body loaded only once the agent decided it needed it, the four agents kept about 51% of the action gain and the open models about 38%. Loading a skill body raised episode tokens by 50%.

- A rule that must always apply belongs in the always-on file, as the [Codex memories docs](https://learn.chatgpt.com/docs/customization/memories?surface=app) also say ("Keep required team guidance in `AGENTS.md`"). The store holds what can be fetched when asked.

- **Precision falls as the pool grows.** [Demystifying Agent Skills](https://arxiv.org/abs/2608.14036) (14 August 2026) found skills beat workflow memory by 6.06 points in matched comparisons, mostly as procedural anchors (65.7% of cases) rather than knowledge injection (4.5%). Actual-use retrieval precision fell from 29.6% to 3.3% as the pool grew from 5 skills to 100.

- The eligible set for one query has to stay small. Hard filters and the write bar are how.

- **Instructions outrank code.** [Stale instruction files](https://arxiv.org/abs/2610.30571) (22 October 2026) built 40 Python projects whose instruction file asked for one convention while the code used another, and ran Claude Code and Codex CLI for 320 trials. 90.1% of choices followed the stale file. Claude Code noticed the conflict in 79 of 80 runs and still followed the file.

- Codex rewrote existing code to match in 20 of 80 runs, and tests then failed in 17. One sentence in the user-level instruction file giving the existing code precedence moved code-following from 4.2% to 91.4% and the test pass rate from 89.4% to 100%.

- The authors' explanation: "the instruction file reaches the model as an instruction and the code as tool output." A note from this store arrives as tool output, which is the lower-ranked channel; that was not measured for notes. Proposal: the same precedence sentence belongs in the shared instructions generated by `home/development/instructions.nix`.

- **Instructions accumulate.** [Catastrophic remembering](https://arxiv.org/abs/2608.11095) (11 August 2026) followed 247,694 instruction lifetimes across 1,867 repositories. Files more than tripled over their lifetime (+226%), gained 4.9 net instructions per commit, and older instructions became less likely to be deleted.

- Adding is cheap and removing safely is expensive. Comments that record the reasoning behind an instruction removed 99.3% of excess instructions in their inverted IFEval tasks, cutting growth from +211.3% to +1.4%.

- Proposal: a note carries its reason as a field, so a later reader can close it without guessing.

- **Copies diverge.** [An instruction-file study](https://arxiv.org/abs/2611.02383) (2 November 2026) covered 3,387 files in 1,000 repositories. 42.2% of the higher-popularity repositories used more than one format, 16.1% of 386 corresponding section pairs gave divergent guidance, and 40.4% of files were modified only once. One record with one source of truth avoids the divergence.

- **Task-specific beats personal.** [Personalized skills](https://arxiv.org/abs/2608.10319) (10 August 2026, revised 28 October) found task-specific skills most effective. Developer-preference skills helped only when the preference came up often and the tasks were closely related.

- **Verified traces, cited fields.** [Learning by Doing](https://arxiv.org/abs/2611.01386) (1 November 2026) compiles skills only from verified attempts, cites the supporting events in every field, gates admission per version, and serves skills through lexical search with progressive disclosure over an API or MCP. Skills from a teacher model raised three weaker executors by 4.5 to 18.8 points, mostly on tasks the teacher had solved.

- Verified use outcomes change a skill's rank, which is a different signal from the access counts this store excludes.

### Projects

- an external memory tool ([Show HN](https://news.ycombinator.com/item?id=49594008), about 7 September 2026, 93 points) keeps one SQLite file at `memory.db` with mode 0600, merges FTS5 BM25 and model2vec static embeddings with reciprocal rank fusion, gates retrieval per prompt behind a relevance floor, and returns a pack of about 4,000 characters. Records carry `origin_agent`, decisions form supersede chains, and `an external memory tool merge <OTHER.db>` imports another store.

- The author's claim of 153,000 tokens reduced to under 1,000 over 105 sessions is unverified. `esafak` noted "no lifecycle management or conflict resolution yet." `cedws` had seen models "save poor memories and recall them in the wrong context." `verdverm` said agents should draft candidates and a person should edit them.

- an external memory tool (v0.7.1, September 2026) is a Rust CLI over one SQLite file with WAL, FTS5, and 384-dimension vectors, fused with RRF at k=60. Its own benchmarks on an M4 Pro report FTS5 search at 171 µs for 10,000 memories and 1.49 ms for 100,000, against 172 ms for brute-force vector search at 100,000.

- It merges a new memory into an existing one above 0.92 cosine similarity and decays scores by access with a half-life of about 69 days. Both conflict with rules above.

- The latency numbers say FTS5 is not where the time goes.

### Seen, and not adopted

- [REALM](https://arxiv.org/abs/2609.16053) (13 September 2026) reconsolidates memories from retrieval feedback (+7.17 on LoCoMo, +1.31 on LongMemEval). Changing stored memories conflicts with immutable records.

- [RARG](https://arxiv.org/abs/2607.24223) (27 July 2026) guides a ripgrep agent with relevance ordering and reranked matches. The abstract gives no numbers.

## Decisions

- **One local database per account.** Each user account keeps its own store in its XDG data folder (`$XDG_DATA_HOME`, else the standard XDG data default). No store is shared between accounts.

- **Moving bytes between accounts is transport.** The store reads a file or stdin. How an envelope or a body got there is not the store's concern.

- Two transports are wanted: an explicit import over SSH, or paired CLIs on a blob-transfer library that sync across accounts and machines. Sync is a double import: each side imports the records and bodies it lacks from the other.

- Either way the store sees the same thing, a stream of envelopes and bodies named by hash, with tombstones applied before bodies.

- **The central question is search and narrowing by structure.** Trees, graphs, and search together give documents and notes structure that agents do not manage. The store derives it on ingest: the heading tree from the bytes, the location from evidence paths and scope, and the edges from explicit links, closes, and co-change.

- Because no agent maintains a layout, the structure cannot drift the way agent-curated stores did. If adding and deleting are natural operations, search can narrow through that structure instead of ranking a flat pile.

- **Highest priority to explore:** narrowing by derived structure, combining a document query tool-style selectors, an indexing tool's deterministic tree and progressive disclosure, the draft CLI in "Narrowing by derived structure," and a retrieval tool.

- **Location narrows; it never answers.** Different queries from the same folder must not yield the same results because of the folder. See "Location narrows, it does not answer."

- **Retrieval is one simple CLI or tool call.** The agent asks; the store answers. Structure such as trees or graphs can shape what one call returns.

- It does not add a second retrieval system or a push path.

- **No signing.** An accept or a close names its actor as a recorded field. The store does not apply an accept whose actor is the writer of the target.

- That stops a self-accept through the tool. It does not stop a forged actor.

- The OS user cannot be the actor, because shared agent instructions are installed for both CLI and agent processes, which may share one uid.

- **Storing and retrieving core (2026-11-05).** Bodies and immutable envelopes are the durable units; sections, the tree, the index, and validity are rebuilt. The heading path is a plain column for `under:` and display, not a lexical field.

- All terms match within one section. Filter, then rank by tier and BM25, then group by document; ≤ five cards; a section id fetches original bytes; empty is "Nothing qualified." The recorded policy is: merging affects display only; labels do not satisfy query terms; citations name an immutable body version (body hash, derivation version, ordinal or byte span); the parts column matches identifier pieces, never an exact identifier.

- Details in "Storing and retrieving core, reviewed 2026-11-05."

- **Superseded open decision: author terms on imported document sections (2026-11-05; now closed).** The settled rule is "no embedding model on the write or query path", motivated by a local model's memory, startup, download, and reindex cost. Terms the importing agent writes for each section of a document it adds cost the store nothing, like note terms, so they are read here as inside the rule.

- This is being tested (per-section terms written blind to tasks and queries) and is not adopted pending a separate decision. Measured motivation: under the best silence-passing plans, note gold is found 0.553 of the time and document gold 0.474; author terms on notes raised strict coverage of note-gold cases from 0.232 to 0.447.

- **Silence carries hints; the agent decides (accepted 2026-11-05).** An empty result shows no cards and may name up to three near-miss sections: heading path, record label, and which query words matched and which did not. Hints come only from rows the caller may already see (eligible in the scope, not forgotten), carry no body text, and do not count as an answer.

- The agent reads them against its task and chooses: stop with "nothing", open a hint (its section becomes a card), or query again. This moves the answer-or-silence choice from a store threshold to the agent, which knows the task; the store stays model-free.

- It replaces "relaxation counts and nothing else" on an empty result. Measured motivation (dev, before any agent judgment): for silent cases with a gold answer, a top-3 hint names the gold record in 73 of 95 (plan 51) and 51 of 70 (plan 53 P2), and the gold section in 43 and 29; but 85% of silent empty cases also get hints, so the agent's rejection rate decides whether the frontier moves.

- Evaluation: the harness puts a model in the agent's seat (Claude Haiku as the main agent, Claude Sonnet as a check), sees only the task, its query and the hints, runs at temperature 0, and caches each decision; Q1 counts a case right only if the agent ends with no opened card. Recorded as measurement change 6 in the test harness.

- **Revised the same day, after the first measurement:** with headings-only hints, both agents opened a hint on most silent calls (Haiku Q1 0.922 to 0.804, Sonnet to 0.725; Q3 0.713 to 0.83), because an opened hint counted as an answer before the agent had read it. Two changes, accepted in the recorded plan: (1) each hint also carries a short excerpt, the first line of its span that holds a matched word, ≤ 160 characters, from eligible rows only; (2) the agent reads every result it gets, store cards and opened hints alike, and either accepts it as the answer or rejects it and goes on (the store's next call for the same query, its next prepared query, or stop).

- Silence and recall are then graded on what the agent accepts: an empty case is right if the agent accepts nothing; a gold is found only in an accepted call. Every plan is graded the same way, so this changes what the answer gates measure: from what the store shows to what an agent concludes.

- Store-level checks (eligibility of anything shown, exposure, self-retrieval, location invariance, determinism) stay on what the store shows.

- **Progressive disclosure carries exposure (2026-11-05).** A search returns cards as pointers: label, heading path, and an excerpt of ≤ 160 characters. The agent opens a card to fetch its section (≤ 1,500 characters); an open is a call.

- One level of disclosure, as the cited progressive-disclosure study found best. A hub document the agent sees briefly and rejects is acceptable as long as its context only ever held part of it; it is still penalised.

- Exposure is therefore weighted by what the agent saw: each non-answer appearance counts the characters of that record shown to or read by the agent in the case, divided by 1,500 (one full section, the section-size bound) and capped at 1. A rejected excerpt counts about 0.1; an opened section counts 1.

- The 1,500-character unit is the evaluation agent configuration and can be changed. The earlier appearance count stays in reports beside it.

- **The agent's budget is tokens, not calls (2026-11-05).** With progressive disclosure, a first movement down an indexing tool's tree (search, outline, section) may take three or four cheap calls, and lateral movement then needs more. A fixed limit of three calls stopped that: with opens counted as calls, 202 of 491 dev cases ran out (plan 11 with hints and disclosure, Sonnet: Q3 0.742); with three extra opens, Q3 0.777, exposure 0.087.

- The limit is therefore a reading budget per case: 1,800 tokens of what the agent is shown (three calls at the 600-token median ceiling, the earlier implicit budget), with a hard cap of 12 calls so a loop ends. The agent may search, go on, open a card's section, or ask for a document's outline (its heading tree to depth 2, as pointers it can open).

- The numbers are the evaluation agent configuration and can be changed.

- **A static embedding model on the write and query path (2026-11-06).** The earlier rule kept every model off the write and query path. The recorded rule then required text-only writer input (no evidence, terms or section terms) and a small model in the binary at ingest and query time, with no derived tables or terms.

- The design embeds every block and every query with model2vec potion-base-8M and uses the nearest blocks only as near misses on silence, each sharing a query word. Results: "Writer fields removed, a small embedding model in their place" above, and `PLAN_97_ARCH.md` 3.1.9.

- **Historical shared-search decision (2026-11-07; scope-filter wording superseded above).** Search covered all repositories unless a scope filter was passed; the current scope rule above supersedes it. A search covers the records of every repository; `scope:<owner>/<repo>` narrows it to that repository and unscoped records. A record still carries the scope of the repository it came from (from the git remote), and a card from another repository shows it.

- This replaces the earlier rule that a search sees only the caller's scope and unscoped records. The runs before plan 79 used that rule, and G1 counted another repository's record as a violation.

- The shared search, with a penalty of 7 inside the tier, is what plan 85 measures (`PLAN_97_ARCH.md` 1.5.2 and 3.1.10).

- **No review step on add (2026-11-07).** `add:accept` and the `candidate` state are removed: a record is searchable as soon as it is added, and a wrong one is closed or replaced. This replaces the earlier write-bar rule that a person accepts a note and that the writer cannot accept its own (Link practice and "No signing", above).

- Close envelopes still record their actor.

- **Open decisions (2026-11-05).** Whether the parts column may be a computed column of the content view, which keeps section text stored once, or must be a generated table column, which stores it twice. Whether terms may span sibling sections, to settle from the real cases. (Whether an empty result may carry relaxation counts is settled above: it carries hints.) Proposals awaiting acceptance: scope-local BM25 with Lucene's IDF, the `path:` location set, and keeping link destinations as written with local resolution.

- Evidence in "Open storing and retrieving questions, researched 2026-11-05."

## References only — 2026-11-08

The primary sources below were checked during a research-only retrieval ideas
pass. These entries add references, not measurements, implementation decisions,
or permission to reopen closed experiments. All proposed Linn applications
remain unmeasured/untested; no adoption is claimed. Existing findings and their
recorded conclusions are unchanged.

- **[Web Annotation Data Model](https://www.w3.org/TR/annotation-model/)**
  (W3C Recommendation). Relevant conclusion: Text Quote Selectors distinguish
  repeated text with exact text and surrounding prefix/suffix; position selectors
  depend on a specific representation and are brittle when it changes. This
  supports investigating explicit ambiguity and version handling for navigation.
  Its normalized, character-based selections are not Linn's original-byte fetch
  ids. Proposed Linn application unmeasured/untested; no adoption claim.
- **[Unicode Standard Annex #15: Unicode Normalization Forms](https://www.unicode.org/reports/tr15/)**
  (Unicode Consortium). Relevant conclusion: canonical and compatibility
  equivalence are different normalization contracts, with defined conformance
  requirements. This supports auditing accidental normalization drift, not
  rewriting immutable bodies or changing identifier matching. Proposed Linn
  conformance audit unmeasured/untested; no adoption claim.
- **[BEIR: A Heterogenous Benchmark for Zero-shot Evaluation of Information Retrieval Models](https://arxiv.org/abs/2104.08663)**
  (Thakur et al., 2021; abstract verified). Relevant conclusion: heterogeneous
  tasks expose differences in retrieval generalization; BM25 is a robust baseline
  in the authors' comparison, while stronger average reranking/late-interaction
  results have higher computational costs. This motivates explicit reporting of
  benchmark coverage, not importing those algorithms. No Linn benchmark run or
  conversion performed; application unmeasured/untested; no adoption claim.
- **[BRIGHT: A Realistic and Challenging Benchmark for Reasoning-Intensive Retrieval](https://arxiv.org/abs/2407.12883)**
  (Su et al., 2024, revised 2025; abstract verified). Relevant conclusion:
  reasoning-intensive queries can require evidence selection beyond surface
  keyword or semantic similarity. This motivates distinguishing reasoning gaps
  from parser/tokenization defects; it does not authorize a query-expansion model.
  No Linn benchmark run or conversion performed; application unmeasured/untested;
  no adoption claim.
- **[Lost in the Middle: How Language Models Use Long Contexts](https://arxiv.org/abs/2307.03172)**
  (Liu et al., 2023; abstract verified). Relevant conclusion: the position of
  relevant information affected model performance in the authors' long-context
  tasks. Fetching a whole document therefore should not be equated with useful
  evidence access without a reader measurement. This does not authorize changing
  Linn's card order. Applicability to current Linn readers unmeasured/untested;
  no adoption claim.

## POS relation ranking and explicitly authorized recovery

- The completed query-and-source nominal-pair candidate at weight 1 measured
  q4-dev any 411/439, first 351/439, xs 30/42 and 24/42, empty 35/51,
  section 55/73. Its first frozen h2-all run measured any 341/354, first 295/354,
  xs 56/62 and 49/62, empty 56/106, section 238/271. The two cross-scope losses
  rejected that frozen configuration under the original rule.
- the candidate was then applied and adjusted
  it to recover those losses. This supersedes the no-retuning restriction for
  this POS study only. h2 now participates in selection; subsequent results are
  development measurements, not independent holdout confirmation.
- Recovery weight 0.5: q4-dev 411/439, 351/439, xs 30/42 and 24/42,
  empty 35/51, section 55/73; h2-all 341/354, 295/354, xs 57/62 and 50/62,
  empty 56/106, section 237/271. Original cross-scope losses recovered.
- Recovery weight 0.75: same q4-dev numerators; h2-all 341/354, 295/354,
  xs 57/62 and 50/62, empty 56/106, section 239/271. Selected for the larger
  section result with all retrieval and empty numerators preserved.
- Recovery weight 0.25: same q4-dev numerators; h2-all 341/354, 295/354,
  xs 57/62 and 50/62, empty 56/106, section 237/271. Not selected over 0.75.
- Page diagnostics confirm recovery of the original-wording cross-scope target
  in h2a2-c-0094 and the any-wording target in s7-c-0080. These case identities
  are diagnostic only; no runtime exceptions or corpus edits were added.
- The applied selector defaults to weight 0.75. Its source publication, embedded
  linguistic asset and original-byte guards remain intact. Existing stores need
  an enabled index rebuild; normal queries never retag the corpus. Preparation
  costs roughly 85–90 seconds for the curated indexes; measured fresh-process
  tagger deserialization was about 117 ms. The modest quality gains do not
  establish broad generalization or improved reader answer quality.

<a id="consolidated-evals-results-confirmation-2026-11-09-md"></a>

## 2026-11-09: Combined retrieval study — not adopted

**Closure:** None of this round's 112 configurations was adopted. The
unsuccessful runtime prototypes and their experimental LLP sections were
removed after measurement; accepted POS retrieval remains unchanged. The individual configuration ledgers and complementary notes are consolidated
below. Ignored evaluation artifacts are optional audit material, not dependencies
of the specification or supported runtime options. Descriptions of default-off mechanisms record
their state during the experiment, not code retained in the product. Encoder
replacement research declined the candidate and was not implemented or downloaded.

- **Frozen combination.** Full stem-aware lexical scoring, union of admitted
  matcher rows before record cuts, later-call preservation of displaced spans,
  adjacent-block support at weight0.5, anchored POS/embedding pairs at
  weight0.75/minimum0.5 with ordinary coverage credit, two-anchor whole-unit
  partial word assignment at minimum0.4 for admission only, and full
  record-frequency lexical IDF. Original source bytes, author-field exclusion,
  model, tiers, scope penalty7 and scorer are unchanged. Experimental/default-off.
- **Dev selection.** Applied default: gold411/439 any,351/439 first,
  xs30/42 and24/42, empty35/51, section55/73. Frozen candidate:414/439,
  362/439, xs32/42 and26/42, empty36/51, section55/73. Exact replay matches
  all1,470 candidate pages. Against default: gold-any4 gains/1 loss,
  gold-first12 gains/1 loss, xs-any2 gains/0 losses, xs-first3 gains/1 loss.
- **Score-scale ablation, not selected.** Uniformly scaling lexical IDFs by the
  query's summed record-IDF/block-IDF ratio preserves relative word weights.
  Without word assignment:413/439,358/439, xs32/42 and25/42, empty36/51,
  section55/73. With assignment:414/439,359/439, same other numerators.
  Neither reproduces the record-specific gains; both lose one xs-first against
  their corresponding preceding combination. Rarity's effect is not explained
  entirely by uniform lexical score scaling.
- **One frozen h2 confirmation — rejected.** Replayed applied default:
  gold341/354,295/354, xs57/62 and50/62, empty56/106, section239/271.
  Candidate:345/354,301/354, xs56/62 and48/62, empty55/106,
  section246/271. Cross-scope-any falls1, cross-scope-first falls2 and empty
  falls1. Gold and section rises do not override those losses. No retuning to
  h2 and no adoption. h2 was used in earlier POS development, so this is a
  reused-benchmark confirmation, not independent holdout evidence.
- **h2 paired findings.** 975 of1,380 wording pages change. Gold-any gains:
  h2a1-c-0006, h2a1-c-0093, h2a3-c-0037, h2a3-c-0041; no any losses.
  Gold-first gains: h2a1-c-0048, h2a1-c-0093, h2a2-c-0011,
  h2a2-c-0036, h2a2-c-0070, h2a3-c-0074, s7-c-0053. Gold-first loses
  h2a2-c-0034. Cross-scope-first loses h2a1-c-0034; cross-scope-any and
  first lose h2a2-c-0102. The all-call empty loss is not attributed from
  first-page exports alone. No case-specific runtime rule was introduced.
- **Fixed synthetic check.** Applied default Hit@1/3/5:31/49,35/49,35/49,
  MRR@5 0.6735. Candidate:33/49,35/49,35/49, MRR@5 0.6939.
  An earlier pre-adjacency candidate reached34/49 Hit@1 and0.7041 MRR@5.
  Better than default, below that older candidate; no broad-transfer or hybrid
  victory claim. No tuning followed this check.
- **Runtime limitation.** Frozen h2 candidate resident invocation median142.098ms,
  p95 568.698ms over1,380 wordings; store open43.411ms. This is not the neutral
  per-call gate. Query-time source-group annotation/assignment remains costly.
  The default's exact dev pages and all code/package checks still pass. The
  combined candidate is closed and must not be rerun as a new experiment or
  tuned to repair this h2 result; default retrieval remains unchanged.

<a id="consolidated-evals-results-fields-2026-11-09-md"></a>

## 2026-11-09: Field-normalization reference

- [Optimisation Methods for Ranking Functions with Multiple Parameters](https://www.microsoft.com/en-us/research/wp-content/uploads/2016/02/optimisation_multiple_parameters.pdf)
  (Taylor, Zaragoza, Craswell, Robertson and Burges, CIKM 2006), Appendix A,
  defines field-normalized frequencies combined before BM25F saturation.
  This motivates a distinct heading/body normalization control. It supplies
  no evidence of a Linn gain; its parameter-fitting method is not adopted.

- **Field normalization — three controls not adopted.** All start from the
  accepted default: gold 411/439 any, 351/439 first; xs 30/42 and 24/42;
  empty 35/51; section 55/73. No rejected-combination features were enabled.
  Heading-only normalization with h=0 reaches 411/439 and 350/439, losing
  first in s6-c-0044; 162 wording pages change.
- Separate body normalization with heading h=0 reaches 409/439 and 349/439:
  s3b-c-0038 loses any, s6-c-0044 loses first, and s6-c-0105 loses any/first;
  204 pages change. Separate fields with h=0.75 reaches 410/439 and351/439,
  losing any in s3b-c-0038; 193 pages change. All three keep xs 30/42 and24/42,
  empty35/51 and section55/73. No gold or cross-scope case gains occur.
- These specific settings are closed, with no h2 or synthetic selection run.
  Default output retains all 1,470 ordered baseline pages. Existing FTS5 column
  counts, original text, model, gates, scope penalty and scorer remain intact;
  no author fields or ancestor headings were added to evidence. These negative
  results do not establish that every possible field-scoring method is ineffective.

<a id="consolidated-evals-results-links-2026-11-09-md"></a>

## 2026-11-09: Explicit-link joint-coverage screen

- **Not adopted; population audit only.** Existing inline-link destinations,
  eligible source/target blocks and full joint content coverage produce 1,935
  block-target edges and witnesses on 488 of 1,470 dev wordings, in 312 cases.
  Each block contributes an exclusive term; all hard terms remain target-local.
  Twenty-four wordings are outside the screen. No original-wording gold miss
  has a witness; three never-found cases have alternate witnesses:
  s5-c-0004, s5-c-0041 and s6-c-0074. Thirty-four no-gold cases also activate.
- Whole-block evidence, ignored destination fragments and shared-path ambiguity
  make this a broad conditional ceiling, not semantic admission. Inspection
  finds a cross-repository AGENTS.md collision and a UI-style/comment-style
  mismatch in s5-c-0041; s5-c-0004 lacks fragment proof. The lifecycle-to-merge
  link in s6-c-0074 remains a possible navigation witness. No page changes,
  scored candidate, h2 run or retrieval improvement is claimed. Two native
  fixture tests pass. This screen does not close all source-link mechanisms.

<a id="consolidated-evals-results-support-2026-11-09-md"></a>

## 2026-11-09: Secondary qualifying passage support

- **Three settings not adopted.** A second same-record, same-tier, disjoint,
  differently-texted admitted block contributes a bounded score bonus before
  the existing scope penalty. Admission, primary spans, tiers and interleave
  stay. Applied alone to the accepted default, weight0.1 scores gold410/439
  any and350/439 first, xs30/42 and24/42, empty35/51, section55/73;
  323 ordered pages change. It loses gold-any in s3b-c-0038 and first in
  s6-c-0044, with no gains.
- Weight0.25 scores409/439 and346/439, with other metrics unchanged;
  541 pages change. Gold-any losses: s3b-c-0038, s4-c-0005. First losses:
  s3a-c-0020, s3b-c-0061, s4-c-0005, s5-c-0148, s6-c-0044. No gains.
- Weight0.5 scores406/439 and342/439, xs28/42 and24/42, empty35/51,
  section55/73; 740 pages change. Gold-any losses: s3b-c-0038, s3b-c-0042,
  s4-c-0005, s4-c-0133, s6-c-0102. First losses: s3a-c-0020, s3b-c-0042,
  s3b-c-0061, s3b-c-0063, s4-c-0003, s4-c-0005, s4-c-0103, s5-c-0148,
  s6-c-0044. Cross-scope-any losses: s4-c-0031, s5-c-0083. No gains.
- At weight0.1, s3b-c-0038's alternate “changeset not needed docs” loses
  short gold s3b-r-0068 to a second disjoint card of CONTRIBUTING.md
  (s3b-r-0002). In s6-c-0044's original “preview counts zero hits”, short
  gold s6-r-0243 is displaced by s3a-r-0106. More admitted passages are not
  reliable corroboration of the requested answer under this mechanism.
- Zero-weight replay matches all1,470 accepted pages byte-for-byte and keeps
  gold411/439 and351/439, xs30/42 and24/42, empty35/51, section55/73.
  These settings are closed; no h2 or synthetic-fixture tuning follows.

<a id="consolidated-evals-results-verb-audit-2026-11-09-md"></a>

## 2026-11-09: Verb–noun chunk population, not a retrieval result

- The existing native linguistic audit over1,470 dev wordings emits392 VP
  chunks and314 adjacent same-sentence VP/NP pairs on307 wordings. Requiring
  the final VP token's substantive dictionary analyses to be exclusively
  verbal retains122 pairs on120 wordings;29 pairs occur on missed-gold
  wording pages, eight on original-wording misses. No scoring trial follows
  from these counts alone.
- Those eight cases are s1b-c-0037, s1b-c-0059, s2a-c-0004, s2b-c-0004,
  s3b-c-0013, s3b-c-0014, s4-c-0013 and s4-c-0092. Several gold bodies lack
  the proposed query relation: “publish workflow” in s2a-c-0004 and “nested
  stack” in s3b-c-0013. s3b-c-0014's gold lacks protected “TypeScript”.
  Query activation is not source support. No protected term is dropped.
- Chunking calls “git” a VP in “git send <host>” and “file” verbal
  in “cursor settings file keys”; exclusive dictionary verb analyses do not
  guarantee the intended query role. Source occurrences, modifier/negation
  retention and protected boundaries remain necessary before any experiment.
  This audit is not a rejection of all verb relations and claims no gain.
- A separate read-only duplication screen finds290 nonempty exact block-body
  duplicate groups (770 rows;286 groups span records) across all q4 indexed
  rows. None of430 saved gold-missing wording pages contains a complete gold
  record's stripped body of at least40 characters in its first five records.
  This excludes only that exact whole-body explanation, not shared spans or
  duplication-induced statistics effects. No deduplication, record-IDF rerun,
  page substitution or retrieval change was made.

### Verb–noun source-side follow-up

- Native tagging of27,488 eligible original structural units yields11,683
  strict verb/NP pairs and9,914 distinct literal keys. Exact final-verb plus
  whole-NP matching supports eight dev wordings but no missed-gold source
  witness. Original byte spans and structural-unit fixture checks pass.
- A permissive screen retaining either exact verb or whole NP, with no
  semantic threshold, finds11 gold witnesses on five missed alternate
  wordings and none on original-wording misses. All retain the verb:
  s3a-c-0012 wording2, s3a-c-0015 wording2, s3a-c-0062 wording1,
  s3b-c-0008 wording2 and s6-c-0074 wording1. The latter's “merge closes”
  source chunk is an incorrect noun-phrase reading. A submit witness retains
  “will not automatically”; a command witness includes --platform ios.
- These are conditional population bounds without full-query, protected-atom,
  filter or semantic checks, not valid new cards. No vector tuning or retrieval
  experiment is justified by this narrow population as a large-gain mechanism.
  No product changes or h2 runs. Broader relations remain unproven; this does
  not close all grammatical matching or claim a retrieval improvement.

<a id="consolidated-evals-results-tier-weave-2026-11-09-md"></a>

## 2026-11-09: Cross-matcher tier priority

- **Not adopted.** Stable-sort the existing A1/B1/A2/B2 candidate sequence
  by each candidate's matcher-local tier before overlap deduplication and
  the five-card cap. Preserve within-tier alternating order, each matcher's
  existing record cut and scope penalty, spans, admission and later-call
  seeds. Unlike the earlier union, this compares no cross-matcher scores.
- Dev reaches gold410/439 any,351/439 first, xs30/42 and24/42, empty35/51,
  section55/73. There are250 changed ordered wording pages, one gold-any
  loss and no case-level retrieval gains. s5-c-0058 loses its successful
  alternate “remove failing test rules”: gold s5-r-0157 leaves the page;
  s5-r-0044 and s5-r-0007 enter as lower-tier evidence is displaced.
- The unchanged-default replay retains all1,470 accepted ordered pages and
  gold411/439 and351/439. The tier-weave prototype and switch were removed
  from product source after the failed measurement, with a snapshot and
  exact results retained locally. Ordinary interleave remains. No h2 run,
  scorer change or retuning of the rejected combined candidate.

<a id="consolidated-evals-results-bm25-curve-2026-11-09-md"></a>

## 2026-11-09: BM25 curve controls

Standard k1+1 scaling, unchanged heading/body/parts weights and original
block statistics; no other experimental controls. Dev only, no h2.
The accepted baseline is411/439 any,351/439 first, xs30/42 and24/42,
empty35/51, section55/73. Global settings, no case-specific selection.

Parameters (1.2, 1).
### length

any 411/439 first 351/439 xs 32/42 24/42; empty 37/51 section 55/73
Changed ordered pages: 627. dev qualifies; unconfirmed.
- gold_any gain: s3a-c-0014.
- gold_any loss: s5-c-0112.
- gold_first gain: s3a-c-0063, s4-c-0076.
- gold_first loss: s3a-c-0020, s5-c-0112.
- xs_any gain: s2b-c-0076, s3a-c-0028.

Parameters (0.6, 0.75).
### saturation

any 412/439 first 348/439 xs 30/42 24/42; empty 35/51 section 55/73
Changed ordered pages: 617. fails dev.
- gold_any gain: s3b-c-0013.
- gold_first gain: s3b-c-0013.
- gold_first loss: s3a-c-0015, s3a-c-0020, s4-c-0003, s6-c-0015.

Parameters (0.6, 1).
### both

any 413/439 first 351/439 xs 31/42 24/42; empty 37/51 section 55/73
Changed ordered pages: 666. dev qualifies; unconfirmed.
- gold_any gain: s3a-c-0014, s3b-c-0013.
- gold_first gain: s3b-c-0013, s4-c-0076, s4-c-0086.
- gold_first loss: s3a-c-0020, s4-c-0003, s6-c-0015.
- xs_any gain: s3a-c-0028.

Parameters (1.2, 0.9).
### length-mid

any 411/439 first 351/439 xs 32/42 24/42; empty 37/51 section 55/73
Changed ordered pages: 436. dev qualifies; unconfirmed.
- xs_any gain: s2b-c-0076, s3a-c-0028.

Parameters (0.9, 1).
### joint-mid

any 413/439 first 354/439 xs 32/42 24/42; empty 38/51 section 55/73
Changed ordered pages: 576. dev qualifies; unconfirmed.
- gold_any gain: s3a-c-0014, s3b-c-0013.
- gold_first gain: s3b-c-0013, s4-c-0076, s4-c-0086, s5-c-0105.
- gold_first loss: s3a-c-0020.
- xs_any gain: s2b-c-0076, s3a-c-0028.

Parameters (2.4, 1).
### repeat

any 411/439 first 349/439 xs 32/42 26/42; empty 37/51 section 55/73
Changed ordered pages: 937. fails dev.
- gold_any gain: s3a-c-0014.
- gold_any loss: s5-c-0112.
- gold_first gain: s3a-c-0063, s4-c-0076, s5-c-0013.
- gold_first loss: s4-c-0006, s4-c-0027, s4-c-0048, s5-c-0112, s6-c-0044.
- xs_any gain: s2b-c-0076, s3a-c-0028.
- xs_first gain: s4-c-0125, s5-c-0032.

Saturation and repeat fail dev selection and are closed. The other controls
are dev-qualified but unconfirmed and not adopted. Joint-mid is the strongest
current gold-first control; it is still a modest gain, not goal completion.
Its three empty-correct gains are native all-call results, not attributed
from first-page ids. Default replay has exact1,470-page parity.

<a id="consolidated-evals-results-curve-interactions-2026-11-09-md"></a>

## 2026-11-09: Curve/stem/semantic interactions

Existing components, seven declared dev-only combinations. Added stem and
subtracted exact scores share the selected BM25 curve; unset preserves old
arithmetic. The test expectation was corrected to retain the0.3 parts term.
No record-IDF, union assembly, adjacency or sentence assignment enabled.

### missing

any 415/439 first 354/439 xs 32/42 25/42; empty 37/51 section 55/73
Changed ordered pages: 850.
- gold_any gain: s3a-c-0014, s3a-c-0034, s3b-c-0013, s6-c-0074.
- gold_first gain: s2b-c-0068, s3b-c-0013, s4-c-0013, s4-c-0076, s4-c-0086, s5-c-0055, s5-c-0105.
- gold_first loss: s3a-c-0015, s3a-c-0020, s4-c-0027, s4-c-0105.
- xs_any gain: s2b-c-0076, s5-c-0081.
- xs_first gain: s5-c-0081.

### full

any 414/439 first 356/439 xs 32/42 25/42; empty 37/51 section 56/73
Changed ordered pages: 923.
- gold_any gain: s3a-c-0014, s3a-c-0034, s3b-c-0013, s6-c-0074.
- gold_any loss: s3a-c-0002.
- gold_first gain: s2b-c-0068, s3a-c-0022, s3b-c-0013, s3b-c-0038, s4-c-0013, s4-c-0076, s4-c-0086, s5-c-0055, s5-c-0105.
- gold_first loss: s3a-c-0002, s3a-c-0020, s4-c-0027, s4-c-0105.
- xs_any gain: s2b-c-0076, s5-c-0081.
- xs_first gain: s5-c-0081.

### half

any 414/439 first 355/439 xs 31/42 23/42; empty 37/51 section 56/73
Changed ordered pages: 739.
- gold_any gain: s3a-c-0014, s3b-c-0013, s6-c-0074.
- gold_first gain: s3b-c-0013, s4-c-0013, s4-c-0076, s4-c-0086, s5-c-0105.
- gold_first loss: s3a-c-0020.
- xs_any gain: s2b-c-0076.
- xs_first loss: s5-c-0082.

### length

any 413/439 first 355/439 xs 33/42 25/42; empty 37/51 section 56/73
Changed ordered pages: 938.
- gold_any gain: s3a-c-0014, s3a-c-0034, s6-c-0074.
- gold_any loss: s5-c-0112.
- gold_first gain: s2b-c-0068, s3a-c-0022, s3a-c-0027, s3a-c-0063, s3b-c-0038, s4-c-0013, s4-c-0076, s5-c-0055.
- gold_first loss: s3a-c-0020, s4-c-0027, s4-c-0105, s5-c-0112.
- xs_any gain: s2b-c-0076, s3a-c-0028, s5-c-0081.
- xs_first gain: s5-c-0081.

### length-mid

any 414/439 first 355/439 xs 33/42 25/42; empty 37/51 section 56/73
Changed ordered pages: 846.
- gold_any gain: s2b-c-0044, s3a-c-0034, s6-c-0074.
- gold_first gain: s2b-c-0068, s3a-c-0022, s3a-c-0027, s4-c-0013, s5-c-0055.
- gold_first loss: s4-c-0027.
- xs_any gain: s2b-c-0076, s3a-c-0028, s5-c-0081.
- xs_first gain: s5-c-0081.

### semantic-0

any 414/439 first 358/439 xs 32/42 25/42; empty 37/51 section 56/73
Changed ordered pages: 930.
- gold_any gain: s3a-c-0014, s3a-c-0034, s3b-c-0013, s6-c-0074.
- gold_any loss: s3a-c-0002.
- gold_first gain: s2b-c-0068, s3a-c-0022, s3a-c-0063, s3b-c-0013, s3b-c-0038, s4-c-0013, s4-c-0076, s4-c-0086, s5-c-0055, s5-c-0071, s5-c-0105.
- gold_first loss: s3a-c-0002, s3a-c-0020, s4-c-0027, s4-c-0105.
- xs_any gain: s2b-c-0076, s5-c-0081.
- xs_first gain: s5-c-0081.

### semantic-1

any 414/439 first 358/439 xs 32/42 25/42; empty 37/51 section 56/73
Changed ordered pages: 931.
- gold_any gain: s3a-c-0014, s3a-c-0034, s3b-c-0013, s6-c-0074.
- gold_any loss: s3a-c-0002.
- gold_first gain: s2b-c-0068, s3a-c-0022, s3a-c-0063, s3b-c-0013, s3b-c-0038, s4-c-0013, s4-c-0076, s4-c-0086, s5-c-0055, s5-c-0071, s5-c-0105.
- gold_first loss: s3a-c-0002, s3a-c-0020, s4-c-0027, s4-c-0105.
- xs_any gain: s2b-c-0076, s5-c-0081.
- xs_first gain: s5-c-0081.

Half fails dev and is closed. Other controls are dev-qualified, unconfirmed
and not adopted. Semantic-0 is selected by highest aggregate first-query gold,
with score-only preferred over identical aggregate admission results. It has
930 changed pages; semantic-1 has931. No exact-parity claim between them.
Reused h2 confirmation must be one frozen run, with no repair after a fall.

### Frozen curve/stem/semantic confirmation — not adopted

- The frozen joint-mid/full-stem/score-only semantic combination scores h2
  gold343/354 any and293/354 first, xs55/62 and48/62, empty54/106,
  section244/271. The unchanged default reproduces341/354 and295/354,
  xs57/62 and50/62, empty56/106, section239/271, with all1,380 saved
  default pages byte-identical. Both runs complete;889 candidate pages differ.
- Gold-any gains: h2a1-c-0006, h2a1-c-0093, h2a3-c-0037; loss:s7-c-0005.
  Gold-first gains:h2a1-c-0093, h2a2-c-0070; losses:h2a1-c-0067,
  h2a2-c-0034, s7-c-0005, s7-c-0136.
- Cross-scope-any gain:h2a3-c-0085; losses:h2a2-c-0102, h2a3-c-0090,
  s7-c-0080. Cross-scope-first loses h2a2-c-0102 and h2a3-c-0090,
  with no gains. Empty and section changes are native all-call results,
  not attributed to individual cases from first-page ids.
- Gold-first, cross-scope and empty regressions reject the combination despite
  any/section gains. It is closed, not adopted, and not retuned. No other dev
  alternative is selected from this h2 result. Release package checks pass;
  the accepted default remains. h2 was previously used in POS development,
  so this is reused-benchmark confirmation, not fresh independent evidence.
- Native resident query timing on 1,380 h2 wordings: default median 6.768 ms,
  p95 22.460 ms; candidate 16.971 ms and 47.782 ms. Sequential store-open samples
  are 92.361 ms and 37.856 ms, not proof of a causal opening improvement. These
  are neither fresh CLI timings nor a neutral C2 result. The rejected candidate
  has higher measured query cost and is not optimized or retuned to rescue it.

<a id="consolidated-evals-results-parts-2026-11-09-md"></a>

## 2026-11-09 — distinct parts evidence, not adopted

One isolated control scored only additional split-component occurrences as
parts TF: `max(parts_tf - body_tf, 0)`. Admission and the default BM25 curve,
POS, heading/body weights, statistics and page construction were unchanged.
This was not the earlier separate-field normalization or minimum-span gate.
Dev: any411/439 first350/439 xs31/42 24/42 empty36/51 section55/73,
against current default411/351,30/24,35,55. All1,470 wording pages were compared;
371 changed. Gold-any and first gain: s2b-c-0006. Gold-first losses:
s4-c-0006 and s6-c-0105; the latter also loses gold-any. Cross-scope-any gain:
s2b-c-0076. Aggregate empty rose one; first-page exports do not establish its
all-call attribution. A corpus-free frequency/clamping fixture passed.
Rejected because gold-first falls; experimental switch removed. No h2 run,
no threshold tuning, no adoption. Full artifacts remain local and uncommitted.

<a id="consolidated-evals-results-coordination-2026-11-09-md"></a>

## 2026-11-09 — query scaffolding and coordination score

The explicit modal/request/contraction screen in the coordination report
matched zero dev wordings. No query rewrite was implemented.

- **Not adopted, coordination power1.** Multiply original matcher score
  by its matched-content fraction to power1; preserve admission and tiers.
  any 411/439 first 345/439 xs 30/42 22/42; empty 35/51 section 55/73. 618 ordered pages changed.
  s3a-c-0020: gold_first loss.
  s3b-c-0063: gold_first loss.
  s4-c-0003: gold_first loss.
  s4-c-0064: xs_first loss.
  s4-c-0067: gold_first loss.
  s5-c-0082: xs_first loss.
  s6-c-0015: gold_first loss.
  s6-c-0074: gold_any gain.
  s6-c-0105: gold_any loss, gold_first loss.
- **Not adopted, coordination power2.** Multiply original matcher score
  by its matched-content fraction to power2; preserve admission and tiers.
  any 410/439 first 346/439 xs 28/42 21/42; empty 34/51 section 55/73. 661 ordered pages changed.
  s3a-c-0020: gold_first loss.
  s3b-c-0057: gold_any loss.
  s3b-c-0063: gold_first loss.
  s4-c-0003: gold_first loss.
  s4-c-0013: gold_first gain.
  s4-c-0031: xs_any loss.
  s4-c-0064: xs_first loss.
  s4-c-0067: gold_first loss.
  s5-c-0082: xs_first loss.
  s5-c-0116: xs_any loss, xs_first loss.
  s6-c-0015: gold_first loss.
  s6-c-0074: gold_any gain.
  s6-c-0105: gold_any loss, gold_first loss.
  Both controls lose gold-first and cross-scope-first; power2 also loses
  gold-any, cross-scope-any and empty. No h2 run or tuning. Switch removed.
  The default and all previous closed controls remain unchanged.

<a id="consolidated-evals-results-associations-2026-11-09-md"></a>

## 2026-11-09 — original-passage associations, population only

A label-blind native dev audit links a query-matching original unit through a
rare original word or four-word passage to a different eligible record that
retains an original query word. It reads no author fields and changes no index.
Across1,183 analyzed wordings (287 skipped), the word/phrase/structural-phrase
controls yield39,596/11,585/10,219 record-wording candidates, reach53/29/27
missed-gold wordings and9/6/5 original misses. They also yield candidates on
20/14/14 true-silence wordings. These are populations, not scored improvements.
The structural original cases are s1-c-0003, s1-c-0011, s3a-c-0008, s4-c-0013,
and s6-c-0083. All five pages are full. The unstructured phrase control also
reaches s4-c-0004 through a whole-table witness; separating cells removes it.
The single-word control additionally reaches s2b-c-0030, s3a-c-0016,
s5-c-0004 and s6-c-0121, but not s4-c-0004. Generic rare words and four-word
phrases remain false-association risks. The report records every reached
wording and the exact protocol. No retrieval adoption, rejection of an
unrun ranker, or floor change is claimed. Append-only feedback cannot recover
these original misses; any next experiment must measure candidate displacement
and section losses. No holdout run. Span checks and the structure fixture pass.

<a id="consolidated-evals-results-bridges-2026-11-09-md"></a>

## 2026-11-09 — passage-bridge selection, not adopted

Implemented the source-bound four-word bridge using existing body FTS lookups
and original unit cards. Plain-word queries only; nonempty page; unshown
eligible target retains a query word in its original unit. No author fields
or content enrichment. The first two controls replace only card5.
Copy ranks by shared target-phrase fraction; vector uses the existing model
query/unit cosine before that fraction. A declared follow-up, dominance,
promotes only above every incumbent unit's cosine, with no fitted cutoff.

**Not adopted — copy.**
any 409/439 first 346/439 xs 30/42 24/42
empty 35/51 section 55/73

552 of1,470 ordered pages changed.

- s1-c-0003: gold_any gain, gold_first gain.
- s1-c-0011: gold_first gain.
- s2b-c-0013: gold_any loss, gold_first loss.
- s3a-c-0002: gold_any loss, gold_first loss.
- s3a-c-0015: gold_first loss.
- s3a-c-0020: gold_first loss.
- s3b-c-0038: gold_any loss.
- s4-c-0003: gold_first loss.
- s4-c-0013: gold_first gain.
- s4-c-0027: gold_first loss.
- s4-c-0031: xs_first gain.
- s5-c-0026: gold_first loss.
- s5-c-0082: xs_first loss.
- s5-c-0112: gold_any loss, gold_first loss.
- s5-c-0141: gold_any gain.
- s5-c-0148: gold_first loss.
- s6-c-0083: gold_first gain.

agent invocation timings: n=1470 median_ms=9.748 p95_ms=31.788 max_ms=554.161; store_ms=44.976; excludes corpus loading, grading and output; not the neutral per-call gate


**Not adopted — vector.**
any 409/439 first 347/439 xs 30/42 24/42
empty 35/51 section 55/73

552 of1,470 ordered pages changed.

- s1-c-0003: gold_any gain, gold_first gain.
- s1-c-0011: gold_first gain.
- s2b-c-0013: gold_any loss, gold_first loss.
- s3a-c-0002: gold_any loss, gold_first loss.
- s3a-c-0008: gold_any gain, gold_first gain.
- s3a-c-0015: gold_first loss.
- s3a-c-0020: gold_first loss.
- s3b-c-0038: gold_any loss.
- s4-c-0003: gold_first loss.
- s4-c-0013: gold_first gain.
- s4-c-0027: gold_first loss.
- s4-c-0031: xs_first gain.
- s5-c-0026: gold_first loss.
- s5-c-0082: xs_first loss.
- s5-c-0112: gold_any loss, gold_first loss.
- s5-c-0148: gold_first loss.
- s6-c-0083: gold_first gain.

agent invocation timings: n=1470 median_ms=9.453 p95_ms=29.782 max_ms=284.843; store_ms=35.582; excludes corpus loading, grading and output; not the neutral per-call gate


**Not adopted — dominance.**
any 411/439 first 351/439 xs 30/42 24/42
empty 35/51 section 55/73

206 of1,470 ordered pages changed.

- s4-c-0013: gold_first gain.
- s4-c-0027: gold_first loss.
- s5-c-0026: gold_first loss.
- s6-c-0083: gold_first gain.

agent invocation timings: n=1470 median_ms=9.900 p95_ms=31.101 max_ms=557.326; store_ms=47.970; excludes corpus loading, grading and output; not the neutral per-call gate

All original lexical rules and other experimental settings stayed fixed.
Copy and vector fail dev selection. Dominance has two gold-first gains and
two losses, no aggregate rise; it does not qualify. These controls are closed.
No h2 run, retuning, push or adoption. Three fixtures and strict clippy passed;
prototype removed. The population audit was evidence to test, not permission
to promote a target. Generic phrase sharing and higher cosine do not guarantee
that replacing an existing card improves the page.

<a id="consolidated-evals-results-table-cuts-2026-11-09-md"></a>

## 2026-11-09 — bounded table-row cuts, not adopted

Native audit:105 literal-pipe tables in41 eligible records;17 oversized tables
in9 records,192 data rows. No original-query gold miss targets those9 records.
The two targeted gold cases, s4-c-0020 and s5-c-0112, already succeed.
An18-page distractor upper bound becomes8 actual oversized-table cards after
exact native replay: s1b-c-0013, s2a-c-0004, s3a-c-0063, s3b-c-0002,
s4-c-0004, s4-c-0033, s4-c-0075, s6-c-0077.

One ingest-only control adds row boundaries inside recognized oversized
tables, retains1,400-byte packing and keeps header/delimiter/first row together.
No overlap, header copying, original-body edit, new model or author fields.
A separate experimental q4 store scores any410/439 first349/439 xs30/42 24/42
empty35/51 section55/73, against accepted411/351,30/24,35,55.75 ordered pages
change. No gold/cross-scope gains; s4-c-0020 loses first and s5-c-0112 loses
any/first. Rejected; no h2 run or tuning. Prototype removed.

All original records, hashes and envelopes compare identical.17 old spans
become46 exact partitions, increasing blocks6,096→6,125; all indexed bodies
verify against original bytes. CR/UTF-8/fence/oversized-row and grammar
fixtures pass, as do strict clippy and LLP checks. The report records the
protocol and limitations. Finer table cuts alone discard useful context.

<a id="consolidated-evals-results-likelihood-2026-11-09-md"></a>

## 2026-11-09 — Dirichlet lexical scoring, not adopted

[Zhai and Lafferty, *A Study of Smoothing Methods for Language Models Applied to Ad Hoc Information Retrieval* (2001)](https://timan.cs.illinois.edu/czhai/pub/sigir2001-smooth.pdf) motivates a collection-frequency scoring alternative. The exact Dirichlet equation was checked against [Apache's IndriDirichletSimilarity source](https://apache.googlesource.com/lucene-solr/+/7ada4032180b516548fc0263f42da6a7a917f92b/lucene/core/src/java/org/apache/lucene/search/similarities/IndriDirichletSimilarity.java). Subtracting the query-only log-background constant gives the relative form below. Field weights and corpus-scaled mu choices are Linn-specific experimental adaptations, not claimed paper optima or Lucene parity. Negative contributions are retained. No Linn benefit is inferred from either reference.

An isolated primary score uses weighted heading/body/parts token frequency
and weighted length, rather than BM25 DF weighting. Its contribution is
log1p(tf/(mu*p))-log1p(length/mu), retaining missing-word penalties. Background
uses eligible collection token frequency plus an unknown bucket; no new model
or persistent index. Mean/4-times-mean smoothing controls are followed by
mean without POS and mean with a label-free one-occurrence reference scale.
The report gives the complete field and scale rules. Other experiments stay
off; original admissions, tiers and scope penalty7 remain.

**Not adopted — mean.**
any 411/439 first 348/439 xs 30/42 23/42
empty 35/51 section 56/73

841 of1,470 ordered wording pages changed.

- s3a-c-0015: gold_first loss.
- s3a-c-0020: gold_first loss.
- s3a-c-0027: gold_first gain.
- s4-c-0003: gold_first loss.
- s4-c-0006: gold_first loss.
- s4-c-0064: xs_first loss.
- s4-c-0076: gold_first gain.
- s4-c-0105: gold_first loss.
- s6-c-0015: gold_first loss.
- s6-c-0083: gold_first gain.

agent invocation timings: n=1470 median_ms=7.215 p95_ms=17.797 max_ms=669.304; store_ms=85.929; excludes corpus loading, grading and output; not the neutral per-call gate

**Not adopted — strong.**
any 408/439 first 344/439 xs 28/42 20/42
empty 34/51 section 56/73

981 of1,470 ordered wording pages changed.

- s2b-c-0006: gold_any gain, gold_first gain.
- s3a-c-0015: gold_first loss.
- s3a-c-0020: gold_first loss.
- s3a-c-0022: gold_first gain.
- s3a-c-0039: gold_any loss, gold_first loss.
- s3b-c-0015: gold_first loss.
- s3b-c-0038: gold_any loss.
- s3b-c-0060: xs_first loss.
- s4-c-0003: gold_first loss.
- s4-c-0005: gold_any loss, gold_first loss.
- s4-c-0006: gold_first loss.
- s5-c-0012: gold_first loss.
- s5-c-0033: xs_any loss, xs_first loss.
- s5-c-0082: xs_first loss.
- s5-c-0116: xs_any loss, xs_first loss.
- s6-c-0015: gold_first loss.
- s6-c-0102: gold_any loss.

agent invocation timings: n=1470 median_ms=6.548 p95_ms=16.363 max_ms=295.993; store_ms=35.303; excludes corpus loading, grading and output; not the neutral per-call gate

**Not adopted — no-pos.**
any 410/439 first 348/439 xs 29/42 22/42
empty 35/51 section 56/73

866 of1,470 ordered wording pages changed.

- s3a-c-0015: gold_first loss.
- s3a-c-0020: gold_first loss.
- s3a-c-0027: gold_first gain.
- s3a-c-0054: gold_any loss, gold_first loss.
- s4-c-0003: gold_first loss.
- s4-c-0006: gold_first loss.
- s4-c-0064: xs_first loss.
- s4-c-0076: gold_first gain.
- s4-c-0105: gold_first loss.
- s5-c-0033: xs_any loss, xs_first loss.
- s6-c-0015: gold_first loss.
- s6-c-0083: gold_first gain.
- s6-c-0086: gold_first gain.

agent invocation timings: n=1470 median_ms=5.664 p95_ms=14.807 max_ms=362.609; store_ms=42.644; excludes corpus loading, grading and output; not the neutral per-call gate

**Not adopted — anchor.**
any 409/439 first 346/439 xs 31/42 24/42
empty 36/51 section 56/73

833 of1,470 ordered wording pages changed.

- s3a-c-0015: gold_first loss.
- s3a-c-0020: gold_first loss.
- s3a-c-0027: gold_first gain.
- s3a-c-0028: xs_any gain.
- s4-c-0003: gold_first loss.
- s4-c-0006: gold_first loss.
- s4-c-0067: gold_first loss.
- s4-c-0076: gold_first gain.
- s4-c-0105: gold_first loss.
- s4-c-0133: gold_any loss.
- s5-c-0112: gold_any loss, gold_first loss.
- s6-c-0015: gold_first loss.
- s6-c-0083: gold_first gain.

agent invocation timings: n=1470 median_ms=6.953 p95_ms=17.598 max_ms=343.516; store_ms=40.238; excludes corpus loading, grading and output; not the neutral per-call gate

All four fail gold-first selection and are closed. The section rise does
not qualify. No holdout run or tuning, push, or adoption. SQL/math/lifecycle
and anchor fixtures and strict clippy pass. Prototype removed; pre-study
search source restored exactly. No conclusion about every possible lexical
probability model follows from these settings.

<a id="consolidated-evals-results-filenames-2026-11-09-md"></a>

### Unquoted dotted-basename population screen — 2026-11-09

- Native read-only screen across the accepted 1,470 dev wording pages: nine
  wordings name an eligible record's dotted basename outside double quotes;
  seven already hit gold, one is a gold miss, one has no gold. Zero unshown gold
  candidates remain even before disambiguating shared names. The miss is
  `s1b-c-0007`/2 (`AGENTS.md cap`); its gold has another source name. The no-gold
  wording is `s1b-c-0067`/1. Successful wordings: `s1b-c-0005`/1,
  `s1b-c-0006`/0, `s1b-c-0036`/0, `s4-c-0077`/1, `s5-c-0147`/1,
  `s5-c-0149`/2, `s6-c-0028`/0. `Co-Authored-By settings.json` names a
  nongold source, illustrating mention versus requested identity. Not adopted:
  this population cannot recover a gold miss. No retrieval code or floors changed,
  no h2 run. Extensionless names and approximate/full paths were not screened.
  Protocol, fixtures and reproduction: [consolidated finding](#consolidated-evals-results-filenames-2026-11-09-md).

<a id="consolidated-evals-results-record-coverage-2026-11-09-md"></a>

### Record-wide all-term admission — 2026-11-09

- Configuration108, `LINN_RECORD_COVERAGE=all`, unions each matcher's eligible,
  filter-matching postings by record. A row one term below ordinary coverage
  may enter primary ranking when all content terms occur somewhere in its
  record. Hard terms remain required in that row. No score or tier bonus, no
  other flags. This differs from adjacent-block support and short-page fill.
- Not adopted: dev any411/439, first352/439, xs30/42 and24/42, empty35/51,
  section54/73 against accepted section55/73. Sole case-level gold change:
  first gain `s6-c-0006`. That is the same case previously gained indirectly
  by adjacent-block admission, not a newly recovered distributed gold passage.
  This study did not establish the new control's exact stage-level causal path.
  No h2 run. Prototype removed; ignored snapshot and complete paired pages
  retained. Corpus-free fixture checks both matchers, off/on behavior, local
  hard terms, candidate exclusion and heading-filter restrictions; strict
  workspace clippy passed. No adoption or significant-improvement claim.

<a id="consolidated-evals-results-headroom-2026-11-09-md"></a>

### Saved-experiment headroom audit — 2026-11-09

- 139 complete dev exports in the system study form104 byte-distinct groups.
  Four aborted exports and four synthetic exports are excluded by exact-key
  validation. Across all exports, the label-informed union covers373/439
  original-query gold cases and421/439 any-wording cases, versus accepted351
  and411. This is a diagnostic upper bound across saved outcomes, not a
  deployable ranker, adoption result or independent generalization estimate.
- 66 original-query cases remain untouched: actual native FTS admission checks
  classify30 as missing a hard literal in every gold record/matcher,26 as
  insufficient coverage/local hard co-occurrence,1 as zero content overlap,
  7 as post-admission, and2 as filtered/invalid and unclassified. Post-admission
  IDs: `s1b-c-0026`, `s3a-c-0014`, `s3a-c-0016`, `s3a-c-0017`,
  `s3a-c-0035`, `s3b-c-0002`, `s4-c-0133`. Zero overlap: `s4-c-0053`.
  Unclassified: `s2b-c-0020`, `s5-c-0005`.18 cases remain missed on every
  wording in every complete export. No new retrieval change or floor run.
- Research decision: prioritize new source-witnessed admission/query
  interpretation over further score-only tuning. This does not reopen rejected
  identifier dropping/piece expansion or failed h2 combinations, and does not
  authorize a label-driven page selector. Native diagnostic, counterexample
  fixture and strict clippy pass. Detailed scope and reproducible artifacts:
  [consolidated finding](#consolidated-evals-results-headroom-2026-11-09-md).

<a id="consolidated-evals-results-boundaries-2026-11-09-md"></a>

### Exact hyphen/whitespace witness screen — 2026-11-09

- Native body-phrase postings plus original-byte whitespace adjacency expose
  extra witnesses on53 dev wordings. Only two contain an unshown gold record:
  `s4-c-0057` wording2 (`expo-router` → source `Expo Router`, `s4-r-0052`,
  bytes348..359), and `s5-c-0074` original (`sign-off` → source `sign off`,
  `s5-r-0027`, bytes13135..13143 and11609..11617). No stemming, term dropping,
  semantic similarity, label-driven construction or body rewriting.
- These are potential witnesses, not scored recoveries; only one is an original
  query. No ranker prototype was justified for the substantial-gain goal.
  No retrieval rule, floors or holdout changed. This does not reopen broader
  rejected identifier expansion. Fixture and strict clippy pass; exact protocol
  and artifacts: [consolidated finding](#consolidated-evals-results-boundaries-2026-11-09-md).

<a id="consolidated-evals-results-density-2026-11-09-md"></a>

### Source-only local vector density — 2026-11-09

- Reference: Conneau et al., **Word Translation Without Parallel Data**
  ([paper](https://arxiv.org/html/1710.04087v2),
  [authors' code](https://raw.githubusercontent.com/facebookresearch/MUSE/main/src/evaluation/word_translation.py)).
  CSLS subtracts local neighbour means from twice cosine; the paper uses K=10
  and reports better translation retrieval. Applicability to Linn is unproven.
- Native diagnostic over5,688 eligible unchanged block vectors: source-only
  density is mean top10 cosine to other records, excluding identical block
  text. Rank by twice query cosine minus that density, without query-derived
  reference distributions. This is an adaptation, not full cross-domain CSLS.
- Original unfiltered/valid dev subset:436 gold and49 no-gold cases. Accepted
  page gold350/436; raw cosine293/436; density300/436. Density gains23 and
  loses16 versus cosine; recovers36 accepted misses but loses86 accepted hits.
  Cosine recovers32 accepted misses. Maximum record exposure across485 queries:
  raw54, density46. No silence/section/cross-scope gate claim; every diagnostic
  list shows five records. Not adopted as retrieval. No thresholds tuned or
  h2 run. Candidate density merits source-evidence investigation, not automatic
  semantic admission. Details: [consolidated finding](#consolidated-evals-results-density-2026-11-09-md).
- Diagnostic density-over-cosine gains: `s1b-c-0025`, `s2b-c-0003`, `s2b-c-0006`, `s2b-c-0010`, `s2b-c-0047`, `s3a-c-0017`, `s4-c-0015`, `s4-c-0023`, `s4-c-0024`, `s4-c-0053`, `s4-c-0059`, `s4-c-0060`, `s4-c-0062`, `s4-c-0094`, `s4-c-0105`, `s4-c-0131`, `s5-c-0087`, `s5-c-0109`, `s5-c-0143`, `s6-c-0009`, `s6-c-0036`, `s6-c-0085`, `s6-c-0096`.
- Diagnostic density-over-cosine losses: `s1-c-0016`, `s1b-c-0022`, `s1b-c-0023`, `s1b-c-0028`, `s2b-c-0027`, `s2b-c-0080`, `s3b-c-0046`, `s3b-c-0052`, `s4-c-0021`, `s4-c-0041`, `s4-c-0048`, `s4-c-0056`, `s5-c-0074`, `s5-c-0085`, `s5-c-0134`, `s6-c-0016`.

<a id="consolidated-evals-results-density-integration-2026-11-09-md"></a>

### Density evidence and rare-neighbour integration — 2026-11-09

- Native matcher-B evidence audit of36 density-selected original-query gold
  recoveries:23 selected gold blocks satisfy hard terms,20 also satisfy
  one-term-short coverage,8 satisfy primary coverage. Checking other blocks of
  those records still yields only8 primary-admissible recovery cases. Among49
  no-gold queries,48 have a hard-compatible candidate,47 a near candidate,40 a
  primary-compatible candidate. These are overlapping evidence counts, not
  empty-correct or gate results; no-gold includes other-repository targets.
- Configuration109, LINN_NEIGHBOR_DENSITY=1, uses source-only top10 density only
  for the existing neighbour on a nonempty short page. Existing candidates,
  IDF2.5, eligibility, prior cards and silence rules stay. Not adopted: dev
  any411/439 first350/439 xs30/42 and24/42 empty35/51 section55/73.94 ordered
  pages change; sole case-level gold/XS difference is first loss `s5-c-0027`.
  No h2 or threshold tuning. Fixture verifies arithmetic and clock/publication
  cache identity; prototype removed. Scope and reproduction:
  [consolidated finding](#consolidated-evals-results-density-integration-2026-11-09-md).

<a id="consolidated-evals-results-density-primary-2026-11-09-md"></a>

### Bounded matcher-B vector feature — 2026-11-09

- Configurations110–112 fixed before measurement: density weights.25/.5 and
  cosine control.25. Already-admitted B rows use (1-w)*old+w*L*minmax(vector),
  where L is the largest old score; constant feature leaves scores alone.
  Density uses source-only top10 correction; all vectors/encoder unchanged.
  A, admission, hard terms, tiers, penalty7, extra fills and assembly stay.
- All not adopted. Density.25: dev411/439 any347/439 first, xs31/42 and23/42,
  empty36/51 section56/73. Density.5:413/439 and350/439, xs30/42 and23/42,
  empty36/51 section57/73. Cosine.25:410/439 and347/439, xs31/42 and23/42,
  empty36/51 section56/73. Every control loses gold-first and XS-first.
- Density.25 gold-any gains `s3a-c-0034`, `s6-c-0074`; losses `s4-c-0092`,
  `s6-c-0105`. First losses `s3a-c-0015`, `s4-c-0027`, `s4-c-0105`,
  `s6-c-0105`. XS-any gain `s2b-c-0076`; XS-first loss `s5-c-0082`.
- Density.5 gold-any gains `s3a-c-0016`, `s3a-c-0034`, `s6-c-0074`; loss
  `s6-c-0105`. First gains `s3a-c-0016`, `s3a-c-0022`, `s3a-c-0063`; losses
  `s3a-c-0015`, `s4-c-0027`, `s4-c-0105`, `s6-c-0105`. XS-any gain
  `s2b-c-0076`, loss `s4-c-0031`; XS-first loss `s4-c-0064`.
- Cosine.25 gold-any gains `s3a-c-0034`, `s6-c-0074`; losses `s3a-c-0002`,
  `s4-c-0092`, `s6-c-0105`. First losses `s3a-c-0002`, `s3a-c-0015`,
  `s4-c-0027`, `s6-c-0105`. XS-any gain `s2b-c-0076`; XS-first loss
  `s5-c-0082`. No h2 or further tuning. Prototype removed. Arithmetic fixture
  and strict workspace clippy passed. Details:
  [consolidated finding](#consolidated-evals-results-density-primary-2026-11-09-md).

## Fast replacement encoders — 2026-11-09

- isolated replacement research was authorized that embed in the native binary
  and remain similarly fast. Accepted defaults, original text, unread author
  fields and scoring stay. No additional concurrent model or reranker.
- [MinishLab, potion-code-16M-v2 model card](https://huggingface.co/minishlab/potion-code-16M-v2):
  code-trained static embeddings,256 dimensions,MIT; code-retrieval evidence
  motivated a direct substitution. Revision e9d2a44ca6a05ac6685f3b23709ea57eb7352d5b.
  Existing Rust loader decodes published F16 weights, normalized and zero-padded
  to512. No extra quantization, tokenizer conversion or runtime download.
- Code is not adopted: dev any410/439 first346/439,xs30/42 and24/42,empty35/51,
  section55/73;88 ordered pages change. Gold-any loses s4-c-0010; first loses
  s1-c-0001,s4-c-0010,s4-c-0020,s5-c-0098,s6-c-0035, with no gold/cross-scope
  gains. Encoding median14.879 µs/query,129.079 µs/block versus8.893/74.277
  accepted fails the predeclared maximum25% slowdown. No threshold tuning.
- [Sentence Transformers, static-retrieval-mrl-en-v1 model card](https://huggingface.co/sentence-transformers/static-retrieval-mrl-en-v1):
  static retrieval model with Matryoshka training supporting dimensional
  truncation. Revision f60985c706f192d45d218078e49e5a8b6f15283a. Test retained
  first512 of1024 coordinates before normalization; existing loader, original
  tokenizer, unmodified weights, same limits and retrieval gates.
- Matryoshka is not adopted: dev411/439 any351/439 first,xs30/42 and24/42,
  empty35/51,section55/73.60 ordered pages change but no case-level gold or
  cross-scope outcomes change. Query/block9.597/80.503 µs passes the speed screen.
  Fresh CLI median246.234 ms versus248.210 accepted,24 processes each over12
  fixed queries: no meaningful speed gain established. No h2 for flat dev.
- Raw-cosine diagnostic on436 gold-bearing unfiltered original dev queries:
  accepted encoder293 hits,code309,Matryoshka293; full accepted page350.
  Recoveries/losses against that page:32/89,32/73,23/80 respectively. All49
  no-gold diagnostic queries also show results. Code's stronger raw signal is
  not full retrieval improvement or a silence rule. Neither replacement selected;
  both substitution prototypes removed. No h2,neutral,scale or hybrid runs.
- [Sentence Transformers, all-MiniLM-L6-v2 model card](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
  reviewed for contextual encoding followed by pooling. Not downloaded or
  implemented; comparable CLI/resident speed remains unmeasured, not disproved.
- Accepted pages replay byte-for-byte across1470 wordings. Local protocol,
  samples and limitations: [consolidated finding](#consolidated-evals-encoder-study-md); raw artifacts and weights
  remain ignored. No substantial gain claimed; the quality goal remains open.
- Wrong-model diagnostic probe: opening a store with a different embedded model
  refreshes fallback arrays but leaves main block vectors unchanged. A check
  after open is insufficient. The diagnostic now checks model identity through
  read-only SQL before open. All scored experiments used fresh matching-model
  stores; the later negative probe's experimental publication was restored and
  candidate output checked byte-for-byte. Future replacement adoption requires
  a complete rebuild/refusal contract, not swapping the executable alone.

## Fixed algorithm controls — 2026-11-09

- runtime algorithm configuration was removed. POS is unconditional at
  the accepted weight 0.75; heading trees always use shared membership. Earlier
  POS-disabled, weight-override and null-scope-only settings describe historical
  experiments, not supported product modes. Historical measurements remain
  recorded; reproducing an old mode requires its recorded binary or revision.
  Diagnostic output and operational store-location settings do not alter search
  or ingestion. No retrieval threshold or accepted-default behavior changes.

<a id="consolidated-evals-retrieval-interactions-2026-11-10-md"></a>

## 2026-11-10 — Exact-evidence IDF and explicit grouping controls

- **Not adopted — global exact-evidence IDF.** Count distinct byte-identical indexed `(heading, body, parts)` tuples for primary BM25/POS N and DF, retaining original average row length, every record, fallback rarity, admission, model, weight0.75 and assembly. Eligible rows5,688 become5,429 evidence groups. Dev any411/439 first351/439 xs30/42 and24/42 empty35/51 section55/73;48 of1,470 ordered pages change, but no case-level gold/XS outcomes change. This tests copied-passage statistics, not the earlier record-IDF or card-deduplication controls. No retrieval rise; closed, no h2.
- **Not adopted — per-repository exact-evidence IDF.** Same predeclared control with lowercased original scope included in the evidence key:5,463 groups. Identical dev totals;42 ordered pages change and no case-level gold/XS outcomes change. Indexed scope stays null. No tuning, combination or h2. Both prototypes return SQL scalar aggregates, not full-corpus Rust indexes; their aggregate scans were not adopted as startup/query implementation. Original source and binaries restored.
- **Inactive, not adopted — inline code query groups.** Preserve a supported single-backtick span as an existing quoted phrase rather than stripping backticks word by word. No source edits, inferred keywords or filter/filename rewrite. Two parser fixtures pass, but there are zero supported spans in the prepared dev queries. All1,470 ordered pages and all six totals match accepted. Prototype removed; no h2. This is not evidence against the feature outside this corpus.
- **Not adopted — bidirectional nominal evidence.** Keep existing query POS selection and source NP annotations; award each existing pair's bonus once for either attested source orientation. Extend indexed preflight/membership and decoded witnesses consistently; no new model, synthetic relation, admission relaxation or weight change. Publication/forget/stale-identity fixture plus reverse preflight/score checks pass. Dev any411/439 first351/439 xs30/42 and24/42 empty35/51 section55/73;17 ordered pages change, with no case-level gold/XS outcomes changing. No retrieval rise; no tuning, combinations or h2. Reversal is co-occurrence evidence, not a semantic-equivalence claim. Prototype removed.
- Complete changed-case/wording ledgers, protocols, commands and limitations: local [consolidated finding](#consolidated-evals-retrieval-interactions-2026-11-10-md); ignored artifacts under `evals/output/duplicate-statistics/`, `evals/output/inline-literals/` and `evals/output/nominal-orientation/`. All four controls are flat, so there is no new hybrid-win claim or reason to run h2/scale/neutral. No rejected earlier score/admission experiment was reopened.

<a id="consolidated-evals-external-baseline-gap-research-2026-11-10-md"></a>

## 2026-11-10 — Twenty fixed primary-score controls

the study requested at least20 scored iterations on a separate research branch; baseline411/439 any,351/439 first, xs30/42 and24/42, empty35/51, section55/73. All20 native full dev runs and all ordered-page comparisons completed. These are20 configurations across eight families, not independent replications. No earlier closed setting was reopened; no author fields, model, admission, tiers or scoring harness changed.

References retained: [Lv and Zhai, Lower-Bounding Term Frequency Normalization (CIKM2011)](https://timan.cs.illinois.edu/czhai/pub/cikm11-bm25.pdf) motivates a positive-TF floor; [Lv and Zhai, When Documents Are Very Long, BM25 Fails! (SIGIR2011)](https://lintool.github.io/robust04-analysis-papers/p1103-lv.pdf) motivates shifting positive normalized TF before saturation. Their findings do not establish Linn gains. Linn retains its own IDF, fields, POS and pipeline; both adapted families below fail dev.

- **Not adopted, iteration01 flag-identity.** Dev any411/439 first351/439, xs30/42 and24/42, empty35/51, section55/73; 4/1,470 ordered pages change. No case-level gold/XS change. Flat, no adoption; closed.
- **Not adopted, iteration02 query-word-once.** Dev any411/439 first351/439, xs30/42 and24/42, empty35/51, section55/73; 0/1,470 ordered pages change. No case-level gold/XS change. Flat, no adoption; closed.
- **Not adopted, iteration03 kind-score-zero.** Dev any410/439 first350/439, xs30/42 and24/42, empty35/51, section55/73; 195/1,470 ordered pages change. gold_any_loss: s5-c-0112. gold_first_loss: s5-c-0112. At least one required metric falls; closed.
- **Not adopted, iteration04 identifier-body-score.** Dev any411/439 first351/439, xs30/42 and24/42, empty35/51, section55/73; 4/1,470 ordered pages change. No case-level gold/XS change. Flat, no adoption; closed.
- **Not adopted, iteration05 bm25plus-0125.** Dev any411/439 first347/439, xs30/42 and24/42, empty35/51, section55/73; 227/1,470 ordered pages change. gold_first_loss: s4-c-0003, s5-c-0148, s6-c-0015, s6-c-0044. At least one required metric falls; closed.
- **Not adopted, iteration06 bm25plus-025.** Dev any410/439 first345/439, xs30/42 and24/42, empty35/51, section55/73; 369/1,470 ordered pages change. gold_any_loss: s5-c-0148. gold_first_loss: s3a-c-0020, s4-c-0003, s4-c-0067, s5-c-0148, s6-c-0015, s6-c-0044. At least one required metric falls; closed.
- **Not adopted, iteration07 bm25plus-05.** Dev any411/439 first345/439, xs31/42 and24/42, empty36/51, section55/73; 568/1,470 ordered pages change. gold_any_gain: s2b-c-0006. gold_any_loss: s5-c-0148. gold_first_gain: s2b-c-0006. gold_first_loss: s3a-c-0020, s3b-c-0015, s4-c-0003, s4-c-0067, s5-c-0148, s6-c-0015, s6-c-0044. xs_any_gain: s3a-c-0028. At least one required metric falls; closed.
- **Not adopted, iteration08 bm25plus-1.** Dev any411/439 first343/439, xs31/42 and24/42, empty36/51, section55/73; 777/1,470 ordered pages change. gold_any_gain: s2b-c-0006, s3b-c-0013. gold_any_loss: s4-c-0133, s5-c-0148. gold_first_gain: s2b-c-0006, s3b-c-0013, s6-c-0086. gold_first_loss: s3a-c-0020, s3b-c-0015, s3b-c-0061, s3b-c-0063, s4-c-0003, s4-c-0067, s4-c-0073, s5-c-0148, s6-c-0015, s6-c-0044, s6-c-0061. xs_any_gain: s3a-c-0028. At least one required metric falls; closed.
- **Not adopted, iteration09 bm25l-0125.** Dev any412/439 first350/439, xs30/42 and24/42, empty35/51, section55/73; 274/1,470 ordered pages change. gold_any_gain: s2b-c-0006. gold_first_gain: s2b-c-0006. gold_first_loss: s4-c-0003, s6-c-0015. At least one required metric falls; closed.
- **Not adopted, iteration10 bm25l-025.** Dev any411/439 first346/439, xs30/42 and24/42, empty35/51, section55/73; 436/1,470 ordered pages change. gold_any_gain: s2b-c-0006. gold_any_loss: s6-c-0105. gold_first_gain: s2b-c-0006. gold_first_loss: s3a-c-0020, s4-c-0003, s5-c-0148, s6-c-0015, s6-c-0044, s6-c-0105. At least one required metric falls; closed.
- **Not adopted, iteration11 bm25l-05.** Dev any410/439 first343/439, xs30/42 and24/42, empty35/51, section55/73; 634/1,470 ordered pages change. gold_any_gain: s2b-c-0006. gold_any_loss: s5-c-0148, s6-c-0105. gold_first_gain: s2b-c-0006. gold_first_loss: s3a-c-0020, s3b-c-0015, s3b-c-0063, s4-c-0003, s4-c-0006, s5-c-0148, s6-c-0015, s6-c-0044, s6-c-0105. At least one required metric falls; closed.
- **Not adopted, iteration12 bm25l-1.** Dev any409/439 first340/439, xs29/42 and24/42, empty35/51, section55/73; 811/1,470 ordered pages change. gold_any_gain: s2b-c-0006. gold_any_loss: s4-c-0005, s5-c-0148, s6-c-0105. gold_first_gain: s2b-c-0006. gold_first_loss: s3a-c-0020, s3b-c-0015, s3b-c-0061, s3b-c-0063, s4-c-0003, s4-c-0005, s4-c-0006, s4-c-0067, s5-c-0148, s6-c-0015, s6-c-0044, s6-c-0105. xs_any_loss: s4-c-0031. At least one required metric falls; closed.
- **Not adopted, iteration13 idf-power-05.** Dev any412/439 first349/439, xs31/42 and23/42, empty36/51, section55/73; 603/1,470 ordered pages change. gold_any_gain: s2b-c-0003, s2b-c-0006, s6-c-0074. gold_any_loss: s3a-c-0054, s6-c-0105. gold_first_gain: s2b-c-0003, s2b-c-0006, s3a-c-0027, s5-c-0105. gold_first_loss: s3a-c-0054, s3b-c-0063, s4-c-0003, s5-c-0012, s6-c-0015, s6-c-0105. xs_any_gain: s2b-c-0076. xs_first_loss: s5-c-0082. At least one required metric falls; closed.
- **Not adopted, iteration14 idf-power-075.** Dev any411/439 first351/439, xs31/42 and24/42, empty36/51, section55/73; 371/1,470 ordered pages change. gold_any_gain: s2b-c-0006. gold_any_loss: s6-c-0105. gold_first_gain: s2b-c-0006. gold_first_loss: s6-c-0105. xs_any_gain: s2b-c-0076. Dev-qualified, subsequently rejected by frozen h2 below.
- **Not adopted, iteration15 idf-power-125.** Dev any410/439 first349/439, xs30/42 and24/42, empty35/51, section55/73; 340/1,470 ordered pages change. gold_any_loss: s3b-c-0038. gold_first_loss: s3a-c-0020, s5-c-0148. At least one required metric falls; closed.
- **Not adopted, iteration16 idf-power-15.** Dev any410/439 first349/439, xs30/42 and24/42, empty35/51, section55/73; 544/1,470 ordered pages change. gold_any_loss: s3b-c-0038. gold_first_gain: s6-c-0086. gold_first_loss: s3a-c-0020, s4-c-0073, s5-c-0148. At least one required metric falls; closed.
- **Not adopted, iteration17 max-field.** Dev any412/439 first349/439, xs32/42 and24/42, empty36/51, section55/73; 467/1,470 ordered pages change. gold_any_gain: s2b-c-0006. gold_first_gain: s2b-c-0006. gold_first_loss: s1-c-0016, s3a-c-0015, s4-c-0006. xs_any_gain: s2b-c-0076, s5-c-0081. At least one required metric falls; closed.
- **Not adopted, iteration18 post-saturation-fields.** Dev any396/439 first328/439, xs31/42 and23/42, empty37/51, section55/73; 1119/1,470 ordered pages change. gold_any_loss: s1b-c-0036, s1b-c-0041, s2b-c-0031, s3b-c-0042, s3b-c-0057, s4-c-0001, s4-c-0005, s4-c-0062, s4-c-0077, s4-c-0092, s4-c-0133, s5-c-0017, s5-c-0112, s6-c-0102, s6-c-0105. gold_first_gain: s3a-c-0063. gold_first_loss: s1b-c-0036, s1b-c-0041, s3b-c-0042, s3b-c-0063, s4-c-0003, s4-c-0005, s4-c-0006, s4-c-0007, s4-c-0027, s4-c-0040, s4-c-0056, s4-c-0062, s4-c-0073, s4-c-0103, s4-c-0129, s4-c-0132, s5-c-0012, s5-c-0017, s5-c-0112, s5-c-0135, s5-c-0148, s6-c-0020, s6-c-0044, s6-c-0105. xs_any_gain: s2b-c-0076, s3a-c-0028. xs_any_loss: s4-c-0031. xs_first_loss: s5-c-0082. At least one required metric falls; closed.
- **Not adopted, iteration19 binary-occurrence.** Dev any405/439 first336/439, xs26/42 and20/42, empty35/51, section59/73; 1247/1,470 ordered pages change. gold_any_gain: s2b-c-0006, s3a-c-0014, s3b-c-0002, s3b-c-0013, s6-c-0074. gold_any_loss: s2b-c-0032, s3a-c-0039, s3a-c-0050, s3b-c-0038, s3b-c-0042, s4-c-0005, s4-c-0133, s6-c-0015, s6-c-0020, s6-c-0060, s6-c-0105. gold_first_gain: s2b-c-0006, s3a-c-0014, s3b-c-0013, s6-c-0083. gold_first_loss: s3a-c-0020, s3a-c-0039, s3a-c-0050, s3b-c-0015, s3b-c-0042, s3b-c-0061, s4-c-0003, s4-c-0005, s4-c-0073, s4-c-0103, s5-c-0012, s5-c-0026, s5-c-0098, s6-c-0015, s6-c-0020, s6-c-0044, s6-c-0060, s6-c-0096, s6-c-0105. xs_any_loss: s4-c-0031, s4-c-0064, s5-c-0033, s5-c-0116. xs_first_loss: s4-c-0064, s5-c-0033, s5-c-0082, s5-c-0116. At least one required metric falls; closed.
- **Not adopted, iteration20 logarithmic-tf.** Dev any408/439 first341/439, xs33/42 and26/42, empty37/51, section53/73; 1155/1,470 ordered pages change. gold_any_gain: s2b-c-0003, s3a-c-0008, s3a-c-0014, s5-c-0004. gold_any_loss: s1b-c-0036, s1b-c-0041, s2b-c-0031, s3a-c-0054, s4-c-0092, s4-c-0133, s5-c-0112. gold_first_gain: s2b-c-0003, s3a-c-0008, s3a-c-0027, s4-c-0076, s4-c-0086, s5-c-0013. gold_first_loss: s1-c-0016, s1b-c-0036, s1b-c-0041, s2b-c-0067, s3a-c-0054, s4-c-0003, s4-c-0006, s4-c-0007, s4-c-0027, s4-c-0048, s5-c-0012, s5-c-0026, s5-c-0112, s5-c-0135, s6-c-0017, s6-c-0044. xs_any_gain: s2b-c-0076, s3a-c-0028, s5-c-0081. xs_first_gain: s5-c-0081, s5-c-0083. At least one required metric falls; closed.

### Frozen relative-IDF confirmation — rejected

Iteration14 reproduces all1,470 selected dev pages exactly. Its single completed h2-all run scores any342/354 first296/354, xs55/62 and49/62, empty55/106, section238/271, versus accepted341/354,295/354,57/62,50/62,56/106,239/271. Gold gains do not override four falling metrics. 361 of1,380 ordered pages change. No retuning or substitution from the failed batch; no candidate is adopted or committed.

- gold_any_gain: `h2a1-c-0093`.
- gold_first_gain: `h2a1-c-0093`, `h2a2-c-0028`.
- gold_first_loss: `h2a1-c-0042`.
- xs_any_loss: `h2a2-c-0102`, `s7-c-0080`.
- xs_first_loss: `h2a2-c-0102`.

The first launch of the replay script failed before scoring because the copied executable lacked its execute bit; permissions were corrected without changing its bytes. Exactly one native h2 confirmation completed. The mass-preservation/rarity-ratio fixture also passes. h2 has previously participated in development, so this is reused-corpus confirmation, not fresh independent validation. No hybrid, scale or neutral run follows a failed confirmation. Research continues with a separate mechanism.

Complete formulas, protocols, commands, source snapshots and every changed page are retained in local [consolidated finding](#consolidated-evals-external-baseline-gap-research-2026-11-10-md) and ignored `evals/output/external-baseline-gap-20261110/`. Accepted source restored. The goal remains open; no hybrid-win or improvement-commit claim.


## Anchored distributive noun phrases — not adopted

A source-witnessed grammatical experiment licenses an ordinary query term
`per-X` only through an original-text `each/every ... X` noun phrase
whose head is X and whose source unit contains another content-query anchor.
Exact identifiers, quoted phrases, inline code and fenced code remain protected.
Candidate discovery uses indexed lexical predicates; no author fields, generated
text, model changes or scoring-label access are involved.

One frozen configuration scores q4 dev any 411/439 first 351/439,
cross-scope 30/42 and 24/42, empty 35/51, section 55/73: all flat.
Two of 1,470 ordered pages change, `s1b-c-0007` wording 0 and
`s5-c-0027` wording 0, with no case-level metric gains or losses.
The former changes from silence to a nongold page headed by `s1b-r-0007`;
that case's acceptable field is not gold and is not counted as success.
The intended gold `s1b-r-0008` contains “each rules file”, but a diagnostic
shows the chunker ends the noun phrase at “rules” and tags “file” as a verb.
The actual observed witness is “each its own global file” in another record.
Thus this strict grammar is active but does not recover the intended answer.
Positive and adversarial source-span fixtures pass. No h2 run, tuning, adoption
or commit follows this flat result. The prototype is removed. This does not
establish that all grammatical matching fails; it exposes a specific ambiguity
in the annotation used by this experiment.


## Porter equality of selected nominal relations — not adopted

The existing query and source nominal-pair selectors were retained. Their words
were compared by existing per-word Porter equality, with each query pair counted
at most once at its original minimum exact IDF and weight 0.75. Exact and
inflected source duplicates received no double credit. Admission, chunks,
protected atoms, lexical scoring, tiers, scope penalty and page assembly stayed.
This is relation equality, not full stem TF/DF scoring or semantic alignment.

A read-only screen found 359 potentially supported pairs on 326 of 1,470 dev
wordings; this was not an eligibility or gold-success measurement.
The one native full dev configuration scores any 411/439 first 351/439,
xs 30/42 and 24/42, empty 35/51, section 55/73. All are flat.
45 ordered pages change, with no case-level metric gains or losses.
Ordered-single-witness, reversal, separate-witness and duplicate-credit fixtures
pass. The research reader uses bounded, publication-checked candidate JSON;
it temporarily bypasses exact-only preflight and makes no ingestion change.
No h2 confirmation or tuning follows. Prototype removed, not adopted.

Changed-page case ids: s1-c-0016, s1b-c-0013, s1b-c-0036, s2a-c-0004, s2a-c-0007, s2b-c-0031, s2b-c-0048, s2b-c-0069, s2b-c-0076, s3a-c-0010, s3a-c-0054, s3a-c-0058, s3a-c-0063, s3b-c-0002, s3b-c-0007, s3b-c-0037, s3b-c-0049, s3b-c-0063, s4-c-0001, s4-c-0021, s4-c-0035, s4-c-0045, s4-c-0046, s4-c-0053, s4-c-0058, s4-c-0064, s4-c-0067, s4-c-0069, s4-c-0075, s4-c-0123, s4-c-0126, s5-c-0002, s5-c-0058, s5-c-0079, s5-c-0095, s5-c-0109, s5-c-0129, s5-c-0151, s6-c-0006, s6-c-0016, s6-c-0053, s6-c-0096.


## Connected nominal source chains — not adopted

An independent exact-token control composes two already-selected nominal
relations A-B and B-C only when they share the same original B occurrence:
strictly overlapping/increasing byte spans, same sentence and original unit.
Selected adjacent query A-C receives source support 1/2; an exact source pair
retains support 1. Use the maximum witness once per query pair, existing minimum
IDF and weight 0.75. No stemming, synonym, admission or other scoring change.
The source population is 5,175 chains in 2,507 blocks across 648 records;
this does not establish correct interpretation or gold support.

One native full dev configuration scores any 411/439 first 351/439,
xs 30/42 and 24/42, empty 35/51, section 55/73, all flat.
Three of 1,470 ordered pages change, with no case-level metric gains or losses.
Shared-occurrence, direction, sentence/unit separation and duplicate/maximum
support fixtures pass. No h2, tuning or adoption; prototype removed.

Changed-page case ids: s3b-c-0039, s4-c-0021, s6-c-0053.


## Source-selected nominal-pair frequency — not adopted

Two configurations were declared before either measurement. For each existing
exact selected query/source POS pair, count distinct supporting blocks using
pos_pair_lookup joined by indexed keys to original lifecycle/expiry eligibility.
Use the existing eligible block total N and pair IDF
ln(1 + (N - pair_df + 0.5)/(pair_df + 0.5)).
This differs from token/record IDF and source-bridge trials: the event is an
already-selected original nominal relation. Query/source selection, the 0.75
multiplier, admission, tiers, penalty, spans and lexical scores stay unchanged.

- Iteration 24 replaces the smaller individual-word IDF with pair IDF.
  Dev any 411/439 first 350/439, xs 30/42 and 25/42, empty 35/51,
  section 55/73. 159 ordered pages change. Gold-first falls in s3b-c-0061;
  cross-scope-first rises in s4-c-0125. Rejected for the gold-first loss.
- Iteration 25 rescales pair IDFs to preserve their original total query weight;
  a single-pair query retains the original bits. Dev any 411/439 first 351/439,
  xs 30/42 and 24/42, empty 35/51, section 55/73. 27 ordered pages change,
  with no case-level metric gains or losses. Flat, not adopted.

Frequency direction, finite-positive weights, mass preservation and single-pair
invariance fixtures pass. EXPLAIN confirms query-key scans and primary-key
lookups for pair postings, blocks and record eligibility, with no full-record
scan. Neither configuration qualifies; no h2, tuning, adoption or commit.
Both prototypes are removed.

Iteration 24 changed-page case ids: s1-c-0008, s1b-c-0007, s1b-c-0013, s1b-c-0036, s1b-c-0055, s1b-c-0067, s2a-c-0008, s2b-c-0001, s2b-c-0004, s2b-c-0011, s2b-c-0031, s2b-c-0034, s2b-c-0035, s2b-c-0045, s2b-c-0046, s2b-c-0056, s2b-c-0059, s2b-c-0067, s2b-c-0068, s2b-c-0073, s2b-c-0074, s2b-c-0076, s3a-c-0003, s3a-c-0010, s3a-c-0013, s3a-c-0014, s3a-c-0016, s3a-c-0019, s3a-c-0020, s3a-c-0021, s3a-c-0023, s3a-c-0026, s3a-c-0033, s3a-c-0040, s3a-c-0041, s3a-c-0048, s3a-c-0050, s3a-c-0052, s3a-c-0057, s3a-c-0058, s3a-c-0063, s3b-c-0002, s3b-c-0009, s3b-c-0013, s3b-c-0019, s3b-c-0026, s3b-c-0037, s3b-c-0038, s3b-c-0039, s3b-c-0050, s3b-c-0051, s3b-c-0053, s3b-c-0060, s3b-c-0061, s3b-c-0063, s3b-c-0065, s4-c-0006, s4-c-0008, s4-c-0015, s4-c-0019, s4-c-0020, s4-c-0024, s4-c-0025, s4-c-0026, s4-c-0034, s4-c-0037, s4-c-0040, s4-c-0047, s4-c-0048, s4-c-0059, s4-c-0060, s4-c-0062, s4-c-0063, s4-c-0066, s4-c-0071, s4-c-0072, s4-c-0073, s4-c-0075, s4-c-0078, s4-c-0083, s4-c-0091, s4-c-0093, s4-c-0099, s4-c-0103, s4-c-0104, s4-c-0105, s4-c-0109, s4-c-0110, s4-c-0113, s4-c-0116, s4-c-0123, s4-c-0125, s4-c-0133, s5-c-0005, s5-c-0012, s5-c-0032, s5-c-0033, s5-c-0050, s5-c-0058, s5-c-0071, s5-c-0073, s5-c-0083, s5-c-0104, s5-c-0105, s5-c-0109, s5-c-0117, s5-c-0127, s5-c-0132, s5-c-0148, s5-c-0150, s5-c-0151, s5-c-0153, s6-c-0001, s6-c-0010, s6-c-0016, s6-c-0034, s6-c-0036, s6-c-0043, s6-c-0051, s6-c-0060, s6-c-0063, s6-c-0064, s6-c-0073, s6-c-0075, s6-c-0086, s6-c-0100, s6-c-0102, s6-c-0115, s6-c-0124.

Iteration 25 changed-page case ids: s1b-c-0067, s2b-c-0035, s2b-c-0056, s2b-c-0074, s3a-c-0016, s3a-c-0026, s3a-c-0041, s3a-c-0048, s3b-c-0002, s3b-c-0009, s3b-c-0053, s3b-c-0065, s4-c-0008, s4-c-0023, s4-c-0024, s4-c-0037, s4-c-0048, s4-c-0055, s4-c-0063, s4-c-0075, s4-c-0105, s4-c-0125, s5-c-0083, s5-c-0104, s5-c-0127, s6-c-0084, s6-c-0100.


## Explicit numeric-placeholder population — no recovery demonstrated

A native source-only screen recognizes an inline-code hyphen/underscore template
only when the same original structural unit explicitly says its uppercase slot
is a version/major number. It matches only a complete identifier with a decimal
value in that slot, not mere prefix resemblance. Negated declarations, quoted
query literals, nonnumeric substitutions and declarations in another unit do not
grant permission. The diagnostic reads original JSONL bodies and the native
lifecycle projection; scoring metadata is attached after candidates exist.

Across 1,470 dev wordings it finds 1 binding in
1 eligible record, supporting 3 wordings:
s3b-c-0018, s3b-c-0035, s3b-c-0048. None reaches a missed gold record. The declared source is not permission
to reinterpret identifiers in unrelated records. No retrieval prototype,
threshold tuning or h2 run follows. Corpus-free boundaries, negation and exact
slot-replacement fixtures and strict example clippy pass. Final outputs equal
the first population result after strengthening those diagnostic fixtures.
This is an audit, not an additional completed full retrieval configuration.


## Source-witnessed nominal compound admission — held, not adopted

One independent configuration licenses an unquoted two-part lowercase alphabetic
hyphen compound only if the existing POS selector recognizes its spaced query
as an adjacent nominal/modifier pair, and an eligible original source unit has
that same ordered pair under per-word Porter equality. Candidate discovery uses
the existing stem phrase index; publication-checked original source POS
relations verify the complete body witness. No independent-piece matching,
source metadata, quoted-literal relaxation, inferred synonym or generated text.
Other hard requirements, coverage need, tiers, lexical TF/IDF, POS weight0.75,
penalty and card assembly remain unchanged; the license adds no invented score.

Dev: any 411/439 first 352/439, xs 30/42 and 24/42, empty 35/51,
section 55/73. First gains s6-c-0102, whose original native-build query is
supported by original source words native builds. There are no other
case-level gold/cross-scope gains or losses, and 15 ordered pages change.
The frozen executable replays all 1,470 selected dev pages exactly.

Single frozen h2-all confirmation: any 341/354 first 295/354,
xs 57/62 and 50/62, empty 56/106, section 239/271. All six accepted totals
remain flat; six of 1,380 pages change without case-level gold/cross-scope changes.
Under the adoption rule this is a hold, not an adoption. No retuning, alternate
configuration, matched-hybrid victory claim, product commit or push follows.
The prototype is archived and removed from product source.

Actual-resource noun-pair, inflection, reversal, separated-source and protected
query-form fixtures pass; strict library clippy passes. Source reads are
candidate-local and guard publication identity. The prototype performs extra
query tagging; invocation costs were not optimized or established as acceptable.
Resident dev/h2 invocation medians are 8.113/9.416 ms and p95s 24.001/28.359 ms;
these sequential evaluator samples exclude startup/grading and are not a
matched CLI benchmark. Reused h2 is not fresh independent validation.

Dev changed-page case ids: s1b-c-0010, s1b-c-0044, s2a-c-0005, s2b-c-0073, s3b-c-0041, s3b-c-0042, s3b-c-0047, s4-c-0013, s5-c-0076, s5-c-0109, s5-c-0138, s5-c-0153, s6-c-0039, s6-c-0102.

h2 changed-page case ids: h2a1-c-0100, h2a1-c-0104, h2a2-c-0018, h2a2-c-0064, h2a3-c-0012.


## Kind words retained in nominal query pairs — not adopted

One independent configuration removes only the KIND/content filter before
forming query POS pairs. The raw-query selector and stored source witness
still require an actual adjacent nominal relation; a kind word does not become
mandatory for lexical admission, and an intervening word cannot be skipped.
Ordinary exact scoring, pair minimum IDF, weight 0.75, source preflight,
eligibility, tiers, scope penalty and assembly remain unchanged. This differs
from the earlier rejected removal of KIND words from BM25 scoring.

Dev any 411/439 first 351/439, xs 30/42 and 24/42, empty 35/51,
section 55/73, all flat. 18 of 1,470 ordered pages change without case-level
gold/cross-scope changes. The actual-resource fixture recognizes local file as
a pair while file remains optional for coverage, and rejects joining across
an intervening directory term. No h2, tuning or adoption; prototype removed.

Changed-page case ids: s2b-c-0046, s2b-c-0055, s3b-c-0038, s4-c-0008, s4-c-0065, s4-c-0077, s5-c-0068, s5-c-0083, s5-c-0099, s5-c-0109, s5-c-0138, s5-c-0139, s5-c-0142, s6-c-0054, s6-c-0066, s6-c-0077, s6-c-0085.


## Camel boundaries at pooled encoder input — not adopted

One independent encoder-input configuration applies the existing lexical index's
ASCII lower/digit-to-uppercase boundary split before static-model tokenization.
It cuts the original first 3,584 Unicode scalars before inserting spaces, keeps
the 512 retained-token budget, averaging, normalization and numerical kernels,
and leaves pieces() for lexical query rewrites unchanged. Model weights, raw
bodies, author-field policy, FTS, POS, gates and ranking constants stay.
A distinct numerical-publication identity and fresh private store prevent reuse
of old vectors. All original record/body hashes match; 2497
of 6096 block vectors change. No new model or download.

Full native dev is flat: any 411/439 first 351/439, xs 30/42 and 24/42,
empty 35/51, section 55/73. 27 of 1,470 ordered pages change without case-level
gold/cross-scope changes. No h2 or fresh-CLI claim follows; prototype removed.
Boundary, unchanged-input, original-window, spaced-form equality and zero-vector
fixtures pass.

Three alternating fresh processes per encoder, seven timing rounds per process,
one Rayon thread, same captured text: median query encoding 8.913 to 9.003
microseconds and block encoding 73.770 to 74.591 microseconds (about 1% slower).
This passes the declared 1.25x encoding screen, not a full CLI latency validation.
Model-load medians are 43.182 and 40.094 ms; no controlled cold-cache or speedup
claim. The rebuilt store takes 18.349 seconds including preparation; native dev
resident median/p95 are 5.948/15.461 ms, excluding startup and grading.

A separate fresh raw-cosine diagnostic over the same 485 original wordings
(436 gold, 49 no-gold; invalid/filtered queries excluded) finds
293/436 baseline versus 291/436
candidate gold hits among five distinct records. It has no lexical admission,
scope penalty or silence test and is not the scored product. This does not
demonstrate a useful candidate gain hidden by the existing gates.
Raw-vector gains: s2a-c-0007, s3a-c-0005, s3b-c-0049, s4-c-0023, s4-c-0062, s5-c-0080, s5-c-0114, s6-c-0096.
Raw-vector losses: s1b-c-0050, s2b-c-0027, s2b-c-0038, s3a-c-0026, s3b-c-0021, s3b-c-0046, s4-c-0021, s4-c-0049, s5-c-0004, s6-c-0110.
Full-page changed case ids: s1b-c-0004, s1b-c-0008, s1b-c-0027, s1b-c-0031, s1b-c-0059, s2a-c-0005, s2b-c-0027, s2b-c-0032, s2b-c-0041, s2b-c-0078, s3b-c-0010, s3b-c-0025, s3b-c-0046, s3b-c-0065, s4-c-0002, s4-c-0023, s4-c-0084, s4-c-0088, s4-c-0089, s5-c-0005, s5-c-0076, s5-c-0091, s5-c-0095, s5-c-0130, s5-c-0151, s6-c-0055, s6-c-0091.

## Heading frequency as presence — not adopted

One independent configuration caps only own-heading term frequency at one before
the existing weighted BM25 saturation. Body and parts frequencies, all constants,
POS, admission and page assembly stay unchanged. The captured index contains 141
heading-term/block pairs with repeated terms. Full native dev stays any 411/439
first 351/439, xs 30/42 and 24/42, empty 35/51, section 55/73. Only one of 1,470
ordered pages changes: s2b-c-0015 original wording swaps its fourth and fifth
records, s4-r-0026 and s4-r-0013. No case-level metric changes. Heading-presence,
body-frequency, unchanged single-heading arithmetic and zero-frequency fixtures
pass. Not adopted; prototype removed; no h2 run. Resident invocation median/p95
are 7.975/24.371 ms, a diagnostic rather than a matched CLI timing comparison.

## Term-distribution saturation — not adopted

One fixed configuration estimates a per-word BM25 saturation scale from positive
weighted TF in eligible blocks. Divide TF by .25+.75*length/avg, average ln(1+c),
then solve k*ln(k)/(k-1)=mean with its continuous value at one and 80 bisections.
Substitute k for 1.2 in numerator and denominator; missing evidence retains 1.2.
Hard and ordinary words use their existing column policies. IDF, field weights,
length slope, POS and all admission/assembly rules remain. Term-constrained FTS
grouping and primary-key joins supply data, without a whole-store resident map.
This adapts the estimator to Linn's weighted blocks rather than reproducing an
unweighted document benchmark.

[Lv and Zhai, A Log-Logistic Model-Based Interpretation of TF Normalization of
BM25 (ECIR2012)](https://lintool.github.io/robust04-analysis-papers/Lv-Zhai2012_Chapter_ALog-LogisticModel-BasedInterp.pdf)
derives index-based log-moment estimation and warns about sparse evidence.
[Trotman, Puurula and Burgess, Improvements to BM25 and Language Models Examined
(ADCS2014)](https://cs.otago.ac.nz/homepages/andrew/papers/2014-2.pdf) compares
BM25T and other variants with mixed benchmark differences. Neither establishes
a Linn gain.

Full native dev: any 409/439 first 345/439, xs 31/42 and 24/42, empty 36/51,
section 56/73. 580 of 1,470 ordered pages change. Gold-any losses: s3a-c-0054,
s6-c-0105. Gold-first losses: s3a-c-0015, s3a-c-0054, s4-c-0003, s6-c-0015,
s6-c-0017, s6-c-0105. Cross-scope-any gain: s3a-c-0028. Gold losses reject this
configuration; gains elsewhere do not override them. Numerical inverse/monotonic
fixtures and a native eligible weighted-posting fixture (candidate exclusion,
expiry, hard/ordinary fields, absent word) pass. Resident median/p95
9.169/27.912 ms is diagnostic, not a matched CLI benchmark. No h2, no tuning,
prototype removed. Detailed page changes remain in the local experiment export.
Changed page case ids: s1-c-0001, s1-c-0003, s1-c-0006, s1-c-0008, s1-c-0009, s1-c-0010, s1-c-0011, s1-c-0012, s1-c-0016, s1b-c-0002, s1b-c-0006, s1b-c-0007, s1b-c-0009, s1b-c-0010, s1b-c-0011, s1b-c-0012, s1b-c-0013, s1b-c-0015, s1b-c-0017, s1b-c-0018, s1b-c-0020, s1b-c-0022, s1b-c-0023, s1b-c-0024, s1b-c-0025, s1b-c-0026, s1b-c-0028, s1b-c-0029, s1b-c-0033, s1b-c-0036, s1b-c-0040, s1b-c-0043, s1b-c-0044, s1b-c-0046, s1b-c-0047, s1b-c-0048, s1b-c-0049, s1b-c-0052, s1b-c-0055, s1b-c-0059, s1b-c-0060, s1b-c-0061, s1b-c-0062, s1b-c-0066, s1b-c-0067, s2a-c-0004, s2a-c-0007, s2a-c-0008, s2a-c-0009, s2b-c-0001, s2b-c-0002, s2b-c-0004, s2b-c-0006, s2b-c-0009, s2b-c-0010, s2b-c-0011, s2b-c-0013, s2b-c-0015, s2b-c-0016, s2b-c-0019, s2b-c-0020, s2b-c-0023, s2b-c-0025, s2b-c-0026, s2b-c-0027, s2b-c-0031, s2b-c-0032, s2b-c-0033, s2b-c-0034, s2b-c-0035, s2b-c-0038, s2b-c-0041, s2b-c-0043, s2b-c-0044, s2b-c-0045, s2b-c-0046, s2b-c-0048, s2b-c-0049, s2b-c-0050, s2b-c-0051, s2b-c-0052, s2b-c-0055, s2b-c-0056, s2b-c-0058, s2b-c-0059, s2b-c-0060, s2b-c-0061, s2b-c-0067, s2b-c-0068, s2b-c-0069, s2b-c-0074, s2b-c-0076, s2b-c-0078, s2b-c-0079, s3a-c-0002, s3a-c-0003, s3a-c-0004, s3a-c-0005, s3a-c-0006, s3a-c-0008, s3a-c-0010, s3a-c-0012, s3a-c-0014, s3a-c-0015, s3a-c-0016, s3a-c-0018, s3a-c-0019, s3a-c-0021, s3a-c-0022, s3a-c-0023, s3a-c-0028, s3a-c-0031, s3a-c-0033, s3a-c-0034, s3a-c-0035, s3a-c-0039, s3a-c-0040, s3a-c-0041, s3a-c-0042, s3a-c-0043, s3a-c-0047, s3a-c-0048, s3a-c-0049, s3a-c-0050, s3a-c-0052, s3a-c-0053, s3a-c-0054, s3a-c-0056, s3a-c-0058, s3a-c-0062, s3b-c-0002, s3b-c-0007, s3b-c-0008, s3b-c-0009, s3b-c-0013, s3b-c-0014, s3b-c-0016, s3b-c-0018, s3b-c-0019, s3b-c-0020, s3b-c-0021, s3b-c-0026, s3b-c-0027, s3b-c-0028, s3b-c-0031, s3b-c-0033, s3b-c-0035, s3b-c-0037, s3b-c-0038, s3b-c-0039, s3b-c-0040, s3b-c-0041, s3b-c-0042, s3b-c-0045, s3b-c-0046, s3b-c-0048, s3b-c-0049, s3b-c-0050, s3b-c-0051, s3b-c-0052, s3b-c-0055, s3b-c-0057, s3b-c-0060, s3b-c-0061, s3b-c-0063, s3b-c-0065, s3b-c-0066, s3b-c-0067, s4-c-0001, s4-c-0003, s4-c-0004, s4-c-0005, s4-c-0006, s4-c-0007, s4-c-0008, s4-c-0011, s4-c-0012, s4-c-0015, s4-c-0018, s4-c-0019, s4-c-0020, s4-c-0021, s4-c-0023, s4-c-0024, s4-c-0025, s4-c-0026, s4-c-0027, s4-c-0028, s4-c-0029, s4-c-0031, s4-c-0033, s4-c-0034, s4-c-0036, s4-c-0037, s4-c-0040, s4-c-0041, s4-c-0043, s4-c-0045, s4-c-0046, s4-c-0047, s4-c-0048, s4-c-0049, s4-c-0051, s4-c-0052, s4-c-0053, s4-c-0054, s4-c-0055, s4-c-0057, s4-c-0058, s4-c-0059, s4-c-0060, s4-c-0062, s4-c-0063, s4-c-0064, s4-c-0065, s4-c-0066, s4-c-0069, s4-c-0071, s4-c-0072, s4-c-0073, s4-c-0074, s4-c-0075, s4-c-0076, s4-c-0077, s4-c-0078, s4-c-0079, s4-c-0080, s4-c-0083, s4-c-0086, s4-c-0091, s4-c-0093, s4-c-0094, s4-c-0097, s4-c-0098, s4-c-0099, s4-c-0100, s4-c-0102, s4-c-0103, s4-c-0104, s4-c-0105, s4-c-0107, s4-c-0109, s4-c-0112, s4-c-0113, s4-c-0115, s4-c-0116, s4-c-0117, s4-c-0118, s4-c-0119, s4-c-0120, s4-c-0123, s4-c-0124, s4-c-0125, s4-c-0126, s4-c-0129, s4-c-0131, s4-c-0133, s5-c-0001, s5-c-0002, s5-c-0003, s5-c-0004, s5-c-0005, s5-c-0011, s5-c-0013, s5-c-0018, s5-c-0019, s5-c-0026, s5-c-0028, s5-c-0031, s5-c-0032, s5-c-0033, s5-c-0036, s5-c-0040, s5-c-0041, s5-c-0043, s5-c-0044, s5-c-0046, s5-c-0048, s5-c-0050, s5-c-0051, s5-c-0055, s5-c-0058, s5-c-0062, s5-c-0067, s5-c-0068, s5-c-0073, s5-c-0078, s5-c-0079, s5-c-0080, s5-c-0081, s5-c-0082, s5-c-0083, s5-c-0087, s5-c-0095, s5-c-0097, s5-c-0098, s5-c-0099, s5-c-0102, s5-c-0104, s5-c-0105, s5-c-0107, s5-c-0109, s5-c-0110, s5-c-0112, s5-c-0114, s5-c-0116, s5-c-0117, s5-c-0122, s5-c-0124, s5-c-0127, s5-c-0132, s5-c-0134, s5-c-0135, s5-c-0138, s5-c-0139, s5-c-0141, s5-c-0143, s5-c-0147, s5-c-0148, s5-c-0151, s5-c-0154, s6-c-0001, s6-c-0009, s6-c-0010, s6-c-0015, s6-c-0016, s6-c-0017, s6-c-0018, s6-c-0020, s6-c-0021, s6-c-0024, s6-c-0026, s6-c-0031, s6-c-0034, s6-c-0035, s6-c-0036, s6-c-0037, s6-c-0040, s6-c-0042, s6-c-0049, s6-c-0051, s6-c-0053, s6-c-0054, s6-c-0055, s6-c-0058, s6-c-0060, s6-c-0061, s6-c-0064, s6-c-0066, s6-c-0071, s6-c-0075, s6-c-0077, s6-c-0083, s6-c-0084, s6-c-0085, s6-c-0086, s6-c-0088, s6-c-0096, s6-c-0097, s6-c-0099, s6-c-0100, s6-c-0102, s6-c-0105, s6-c-0107, s6-c-0109, s6-c-0110, s6-c-0112, s6-c-0115, s6-c-0117, s6-c-0118, s6-c-0119, s6-c-0121, s6-c-0124, s6-c-0125, s6-c-0128.

## Quoted-phrase and source-written acronym screens — no candidate selected

The saved query-word-once control already deduplicated lexical words across
phrases and bare terms. It was not rerun. A native Query parser audit finds only
five dev wordings with multi-token quoted terms: s1-c-0001 wording0,
s1b-c-0025 wording0, s1b-c-0048 wording0, s1b-c-0049 wording0, and
s1b-c-0054 wording2. Three are quoted filenames. All five already retrieve a
gold record. Atomic phrase-frequency scoring was therefore not implemented:
this population cannot repair a missed gold record in those wordings. This is
not a section or ranking-equivalence proof, nor an all-phrase-method rejection.

A second native read-only audit recognizes ASCII parenthetical initialisms of
two to six uppercase letters, requiring exact initials of the immediately
preceding full-name suffix or a complete following parenthesized full name.
Only original JSONL bodies are read. It finds 67 occurrence witnesses across
42 records. Query support is an ordinary parsed short form, or all ordinary
full-name words, but not both. Across 604 query/binding pairs, zero reach a gold
record absent from that wording's first five cards. Corpus eligibility and
semantic certification are not enforced in this screen, so the counts are an
overinclusive opportunity inventory, not accepted aliases. No cross-record
propagation, partial-name expansion, enrichment, new index or retrieval trial.
This narrow grammar has no measured missed-gold opportunity; broader grammars
and section effects remain unproven.

Both native diagnostics compile and pass targeted strict clippy. Source, witness
spans, populations and commands are retained in ignored phrase-audit and
acronym-audit directories under the current research output. Temporary examples
were removed. These screens do not count as additional full retrieval iterations;
30 configurations remain completed. No h2 run or product change follows.

## Content-token static pooling — not adopted

One independent configuration removes existing query stop-list WORD tokens from
the pooled encoder input, preserving not/no, compound tokens, punctuation and
remaining order. The original 3,584-scalar cut occurs first. Model weights,
tokenizer, 512 retained-token budget, mean/normalization kernels and pieces()
remain unchanged. All vector paths use the same preprocessing; lexical indexes,
POS, gates and page construction are untouched. A distinct publication identity
and fresh private store prevent reuse of baseline vectors. Original bodies and
lexical section rows match the accepted store; 5,726 of 6,096 block vectors differ.

Full native dev remains any 411/439 first 351/439, xs 30/42 and 24/42, empty
35/51, section 55/73. Five of 1,470 ordered pages change, with no case-level
gold or cross-scope changes. Input-window, compound, negation, unchanged-text,
explicit-content vector identity and zero-vector fixtures pass. Store preparation
takes 18.710 seconds; resident median/p95 6.036/15.971 ms is diagnostic, not
a matched CLI speed result. No stop-list sweep, h2 or adoption; prototype removed.
Changed pages: s2b-c-0031 wording1, s2b-c-0041 wording0, s3a-c-0022 wording1, s5-c-0043 wording0, s5-c-0095 wording0.

## Distinct source-vocabulary length — not adopted

One independent configuration substitutes distinct normalized WORD tokens in
original own heading/body for raw token count in lexical BM25's row/average
length ratio. TF, IDF, field weights, k1/b, POS, admission, gates and assembly
stay. Raw token counts remain for other operations. One scalar column in the
existing block index and cumulative eligibility/expiry totals is maintained on
writes; runtime loads by block id and clock, without a full-store resident map.
The research schema uses a separate structure identity and fresh private store;
no accepted store is migrated. Original bodies, lexical rows and block vectors
are byte-identical to accepted data.

Full native dev: any 411/439 first 349/439, xs 30/42 and 24/42, empty 35/51,
section 55/73. 610 of 1,470 ordered pages change. Gold-any and first gain:
s2b-c-0006. Gold-any loss: s3b-c-0038. Gold-first losses: s3b-c-0015,
s6-c-0015, s6-c-0044. Net gold-first loss rejects the configuration.
The corpus-free fixture verifies distinct versus raw counts, accepted/expiry
statistics, lazy reopening and recalculation after forget. Release build passes
with an unused raw-average field warning in this prototype; strict full checks
are not claimed. Initial compilation caught a field inserted into the wrong
struct; corrected before any evaluation. Fresh preparation takes 18.459 seconds;
resident median/p95 5.953/15.859 ms is diagnostic, not a matched latency result.
No sweep or h2 run; prototype removed.
Changed page case ids: s1-c-0001, s1-c-0002, s1-c-0003, s1-c-0004, s1-c-0006, s1-c-0008, s1-c-0009, s1-c-0010, s1-c-0011, s1-c-0016, s1b-c-0001, s1b-c-0002, s1b-c-0004, s1b-c-0006, s1b-c-0007, s1b-c-0011, s1b-c-0012, s1b-c-0013, s1b-c-0017, s1b-c-0022, s1b-c-0023, s1b-c-0024, s1b-c-0028, s1b-c-0029, s1b-c-0031, s1b-c-0033, s1b-c-0034, s1b-c-0036, s1b-c-0037, s1b-c-0038, s1b-c-0040, s1b-c-0041, s1b-c-0043, s1b-c-0044, s1b-c-0046, s1b-c-0047, s1b-c-0048, s1b-c-0052, s1b-c-0055, s1b-c-0060, s1b-c-0061, s1b-c-0062, s1b-c-0065, s1b-c-0066, s1b-c-0067, s2a-c-0002, s2a-c-0004, s2a-c-0007, s2a-c-0008, s2a-c-0009, s2b-c-0001, s2b-c-0002, s2b-c-0004, s2b-c-0006, s2b-c-0008, s2b-c-0009, s2b-c-0010, s2b-c-0011, s2b-c-0014, s2b-c-0015, s2b-c-0019, s2b-c-0023, s2b-c-0025, s2b-c-0026, s2b-c-0030, s2b-c-0031, s2b-c-0032, s2b-c-0033, s2b-c-0034, s2b-c-0035, s2b-c-0041, s2b-c-0043, s2b-c-0046, s2b-c-0048, s2b-c-0049, s2b-c-0052, s2b-c-0055, s2b-c-0056, s2b-c-0059, s2b-c-0068, s2b-c-0069, s2b-c-0073, s2b-c-0074, s2b-c-0076, s2b-c-0080, s3a-c-0002, s3a-c-0003, s3a-c-0004, s3a-c-0005, s3a-c-0006, s3a-c-0008, s3a-c-0009, s3a-c-0010, s3a-c-0012, s3a-c-0013, s3a-c-0014, s3a-c-0015, s3a-c-0016, s3a-c-0017, s3a-c-0018, s3a-c-0019, s3a-c-0020, s3a-c-0021, s3a-c-0022, s3a-c-0023, s3a-c-0026, s3a-c-0027, s3a-c-0028, s3a-c-0031, s3a-c-0034, s3a-c-0035, s3a-c-0039, s3a-c-0040, s3a-c-0041, s3a-c-0042, s3a-c-0043, s3a-c-0044, s3a-c-0047, s3a-c-0048, s3a-c-0049, s3a-c-0050, s3a-c-0052, s3a-c-0053, s3a-c-0054, s3a-c-0056, s3a-c-0057, s3a-c-0058, s3a-c-0062, s3a-c-0063, s3b-c-0002, s3b-c-0008, s3b-c-0009, s3b-c-0013, s3b-c-0014, s3b-c-0015, s3b-c-0016, s3b-c-0018, s3b-c-0019, s3b-c-0020, s3b-c-0025, s3b-c-0026, s3b-c-0027, s3b-c-0031, s3b-c-0033, s3b-c-0034, s3b-c-0035, s3b-c-0037, s3b-c-0038, s3b-c-0039, s3b-c-0040, s3b-c-0041, s3b-c-0042, s3b-c-0045, s3b-c-0046, s3b-c-0047, s3b-c-0048, s3b-c-0049, s3b-c-0050, s3b-c-0051, s3b-c-0052, s3b-c-0053, s3b-c-0054, s3b-c-0055, s3b-c-0057, s3b-c-0060, s3b-c-0061, s3b-c-0063, s3b-c-0065, s3b-c-0066, s3b-c-0067, s4-c-0001, s4-c-0002, s4-c-0003, s4-c-0005, s4-c-0006, s4-c-0008, s4-c-0009, s4-c-0010, s4-c-0011, s4-c-0012, s4-c-0014, s4-c-0015, s4-c-0018, s4-c-0019, s4-c-0020, s4-c-0021, s4-c-0023, s4-c-0024, s4-c-0025, s4-c-0026, s4-c-0028, s4-c-0033, s4-c-0034, s4-c-0035, s4-c-0036, s4-c-0037, s4-c-0040, s4-c-0041, s4-c-0043, s4-c-0046, s4-c-0047, s4-c-0048, s4-c-0049, s4-c-0051, s4-c-0052, s4-c-0053, s4-c-0054, s4-c-0056, s4-c-0057, s4-c-0058, s4-c-0059, s4-c-0060, s4-c-0062, s4-c-0063, s4-c-0064, s4-c-0065, s4-c-0066, s4-c-0067, s4-c-0069, s4-c-0071, s4-c-0072, s4-c-0073, s4-c-0074, s4-c-0075, s4-c-0076, s4-c-0077, s4-c-0079, s4-c-0080, s4-c-0082, s4-c-0083, s4-c-0086, s4-c-0088, s4-c-0089, s4-c-0091, s4-c-0092, s4-c-0093, s4-c-0094, s4-c-0095, s4-c-0097, s4-c-0100, s4-c-0102, s4-c-0103, s4-c-0105, s4-c-0109, s4-c-0110, s4-c-0112, s4-c-0113, s4-c-0116, s4-c-0118, s4-c-0119, s4-c-0120, s4-c-0123, s4-c-0124, s4-c-0125, s4-c-0126, s4-c-0129, s4-c-0131, s4-c-0132, s4-c-0133, s5-c-0002, s5-c-0003, s5-c-0005, s5-c-0011, s5-c-0013, s5-c-0018, s5-c-0019, s5-c-0028, s5-c-0031, s5-c-0032, s5-c-0033, s5-c-0036, s5-c-0043, s5-c-0046, s5-c-0048, s5-c-0050, s5-c-0051, s5-c-0055, s5-c-0058, s5-c-0059, s5-c-0062, s5-c-0068, s5-c-0073, s5-c-0074, s5-c-0079, s5-c-0080, s5-c-0081, s5-c-0083, s5-c-0085, s5-c-0091, s5-c-0097, s5-c-0098, s5-c-0099, s5-c-0102, s5-c-0103, s5-c-0104, s5-c-0105, s5-c-0107, s5-c-0109, s5-c-0110, s5-c-0112, s5-c-0114, s5-c-0116, s5-c-0117, s5-c-0118, s5-c-0122, s5-c-0124, s5-c-0126, s5-c-0130, s5-c-0132, s5-c-0134, s5-c-0135, s5-c-0137, s5-c-0138, s5-c-0139, s5-c-0141, s5-c-0142, s5-c-0143, s5-c-0144, s5-c-0147, s5-c-0148, s5-c-0149, s5-c-0150, s5-c-0151, s5-c-0152, s5-c-0153, s6-c-0001, s6-c-0003, s6-c-0009, s6-c-0010, s6-c-0015, s6-c-0016, s6-c-0017, s6-c-0020, s6-c-0021, s6-c-0024, s6-c-0028, s6-c-0030, s6-c-0031, s6-c-0034, s6-c-0035, s6-c-0036, s6-c-0037, s6-c-0043, s6-c-0044, s6-c-0049, s6-c-0051, s6-c-0054, s6-c-0058, s6-c-0060, s6-c-0061, s6-c-0063, s6-c-0064, s6-c-0065, s6-c-0066, s6-c-0071, s6-c-0074, s6-c-0075, s6-c-0077, s6-c-0079, s6-c-0083, s6-c-0084, s6-c-0085, s6-c-0088, s6-c-0089, s6-c-0094, s6-c-0096, s6-c-0097, s6-c-0100, s6-c-0102, s6-c-0105, s6-c-0107, s6-c-0109, s6-c-0115, s6-c-0119, s6-c-0121, s6-c-0124, s6-c-0125, s6-c-0128.

## Recovery inventory after 32 configurations — diagnostic only

A native saved-page audit checks all 32 complete exports against the same 1,470
keys and independently reproduces each ledger's gold-any/first counts using
object-level equivalents. Their label-informed success union is 419/439 any
and 365/439 first. Combining with the earlier 139-export inventory yields
423/439 any and 376/439 first, leaving 16 any-wording and 63 original-query
cases unreached by these exports. This is an unattainable per-case selection
upper bound over measured outputs, not a ranker, combined configuration or
performance claim; losses, no-gold, section and cross-scope tradeoffs are omitted.

New any recoveries beyond the earlier inventory: s2b-c-0003 (IDF power0.5 or
logarithmic TF), s3b-c-0002 (binary occurrence). New original-query recoveries:
s2b-c-0003 (same controls), s3a-c-0014 (binary occurrence), s6-c-0102
(source-witnessed nominal compounds). The relevant configurations remain rejected
or held; no case selector or combination is licensed. Later flat controls do
not add recovery headroom. Shift attention away from further generic weighting
changes, without claiming all such algorithms are disproved.

Native audit and targeted strict clippy pass. Source and full inventory remain
in ignored recovery-audit output under the current research directory. No search,
store mutation, holdout run or product change was needed for this audit.

## Nominal-token static pooling — not adopted

One independent configuration uses pinned NLP spans after the original encoder
input cut to retain any nominal/modifier-tagged token, tokens with no POS tags,
not/no, numeric tokens and hyphen/underscore/camel technical forms. Only original
token bytes are joined, in order. The same selector feeds all vector paths.
Weights, tokenizer, pooling/normalization, pieces(), lexical evidence, POS rank
relations and all gates remain. A distinct numerical publication identity and
fresh private store prevent old-vector reuse. Original bodies and lexical rows
match; 6,078 of 6,096 block vectors change. This is pooled grammatical selection,
not a new semantic admission or the prior pair/assignment mechanism.

Full native dev: any 411/439 first 350/439, xs 30/42 and 24/42, empty 35/51,
section 55/73. 72 of 1,470 ordered pages change. The only case-level gold/XS
change is gold-first loss s1b-c-0018. Reject; no tag-list tuning or h2 run.
Actual NLP fixtures preserve technical atoms and negation, select local cache
from the local cache, and handle empty input. Targeted tests and strict library
clippy pass. Preparation takes 45.010 seconds; resident median/p95
6.021/15.912 ms is diagnostic, not matched fresh-CLI validation. Prototype removed.
Changed page case ids: s1b-c-0002, s1b-c-0005, s1b-c-0007, s1b-c-0008, s1b-c-0015, s1b-c-0018, s1b-c-0031, s1b-c-0034, s1b-c-0041, s1b-c-0049, s1b-c-0059, s1b-c-0060, s1b-c-0062, s2a-c-0005, s2b-c-0020, s2b-c-0027, s2b-c-0028, s2b-c-0031, s2b-c-0041, s2b-c-0051, s2b-c-0078, s3a-c-0009, s3a-c-0028, s3b-c-0010, s3b-c-0028, s3b-c-0033, s3b-c-0064, s4-c-0005, s4-c-0015, s4-c-0023, s4-c-0067, s4-c-0080, s4-c-0088, s4-c-0089, s4-c-0092, s4-c-0115, s4-c-0123, s5-c-0001, s5-c-0003, s5-c-0005, s5-c-0043, s5-c-0076, s5-c-0090, s5-c-0091, s5-c-0095, s5-c-0097, s5-c-0124, s5-c-0130, s5-c-0132, s5-c-0151, s6-c-0006, s6-c-0036, s6-c-0039, s6-c-0055, s6-c-0074, s6-c-0083, s6-c-0088, s6-c-0091, s6-c-0094, s6-c-0112, s6-c-0118, s6-c-0128.

## Tokenizer-piece mass audit — format opportunity, not retrieval evidence

A native audit of the existing tokenizer uses the original 3,584-scalar window,
unknown-token removal and first 512 retained tokens. Across 1,470 dev wordings
it counts 6,864 retained tokens in 5,685 whitespace units; 566 units split into
multiple tokens, maximum 13. Across all 6,096 captured blocks it counts 772,724
retained tokens in 408,691 units; 114,341 units split, maximum 444. The largest
units are long dash runs in original Markdown table delimiter rows. These are
token-count observations, not measured vector-magnitude contributions, semantic
errors or retrieval gains. Whitespace units are not the lexical WORD grammar.

The pinned model2vec-rs loader keeps raw embedding storage private but exposes
unnormalized pooled outputs when normalization is false. Broad per-word pooling
was not implemented; the observed formatting population motivates a narrower
table-delimiter encoder-input control first. No new model or download. Native
audit and targeted strict clippy pass; source/population are retained under
ignored word-pooling-audit output, temporary example removed.

## Table-delimiter-free vector input — not adopted

One fixed configuration omits only local table delimiter rows from encoder
input, after the original scalar-window cut. Require at least two pipe-separated
cells of three or more dashes with optional alignment colons, and an immediately
preceding original line with the same cell count. Preserve a newline; inputs
containing triple-backtick or triple-tilde fences stay untouched. A block may
start inside a fence whose marker is outside the input; this local recognizer
does not certify whole-document Markdown structure. No arbitrary punctuation
removal, source/body mutation or lexical rewrite. Same model, tokenizer, token
budget, pooling and normalization, POS and all retrieval gates. A fresh private
store and separate vector-publication identity avoid stale-vector reuse.

Full native dev stays any 411/439 first 351/439, xs 30/42 and 24/42, empty
35/51, section 55/73. 101 of 6,096 block vectors and one of 1,470 ordered
pages change; no case-level gold or cross-scope changes. Original bodies and
lexical rows are identical. Formatting, fence, mismatch, window, model-equality
and empty-vector fixtures pass. Preparation takes 18.393 seconds; resident
median/p95 6.019/15.759 ms is diagnostic, not matched CLI validation. No grammar
tuning or h2 run. Flat result closes this configuration; prototype removed.
Changed page: s1b-c-0002 wording2.

## Static replacement screen: SSE v2 and Bekko references

[Stable Static Embedding for Retrieval MRL v2](https://huggingface.co/RikkaBotan/stable-static-embedding-fast-retrieval-mrl-en-v2)
publishes a 512-dimensional, 30,522-row static table with a learned elementwise
post-pooling transform. Its reported retrieval scores are model-card evidence,
not a Linn result. The [pinned inference source](https://huggingface.co/RikkaBotan/stable-static-embedding-fast-retrieval-mrl-en-v2/blob/928b418b2741b7f7b1b557a53845ac88ad5054c4/SSE.py)
includes unknown-token vectors in the mean, then applies
`beta * tanh(alpha * mean + bias)`. The native experiment normalizes that output,
retains Linn's 3,584-scalar/512-token bounds and zero empty-input contract, and
embeds the weights. Model card: Apache-2; inference source header: MIT.
Remote Python is inspected, never executed. No runtime download or second model.

The first scalar-tanh implementation took 12.487 versus 9.494 microseconds per
query and 76.188 versus 75.012 per block, failing the declared 1.25x query-speed
screen. [Apple vForce](https://developer.apple.com/documentation/accelerate/vforce)
and [vvtanhf](https://developer.apple.com/documentation/accelerate/vvtanhf(_:_:_:))
provide the same pointwise transform through native vector math. With this kernel,
three fresh processes of seven alternating rounds over 687 query strings and
6,096 original blocks measured 9.590 versus 9.604 microseconds per query and
73.621 versus 74.720 per block. Warm-cache load medians were 9.035 versus 43.127
milliseconds; these are not cold CLI measurements. An independent f64 equation
fixture passes at 2e-6 tolerance; no bit-identical cross-kernel claim. Targeted
strict clippy passes. The fixed native kernel passes the screen for a retrieval
trial; this alone is not a search-success improvement.

[bekko-static-embedding-v1: State-of-the-Art Retrieval for Multilingual Static Embeddings](https://huggingface.co/blog/hotchpotch/bekko-static-embedding)
and the [Bekko model card](https://huggingface.co/hotchpotch/bekko-static-embedding-v1)
describe a larger 200k-vocabulary, 1,024-dimensional multilingual static model
with 512-dimensional truncation support, mean pooling and no query prefix. Its
published retrieval gains do not establish Linn gains. Not downloaded or tested;
the larger storage footprint made the native-512 SSE candidate the first screen.
The [POTION collection](https://huggingface.co/collections/minishlab/potion)
remains the source family for the accepted retrieval model. Earlier closed
replacement trials remain closed.

## SSE v2 sole replacement — not adopted

One fixed native replacement, revision 928b418b2741b7f7b1b557a53845ac88ad5054c4,
using the speed-screened vector-tanh kernel and a fresh private store. All
lexical matching, POS, gates, thresholds and scoring stay unchanged. Full dev:
any 411/439 first 350/439, xs 31/42 and 25/42, empty 35/51, section 55/73.
81 of 1,470 ordered wording pages change. Gold-first gains s6-c-0073 but loses
s4-c-0020 and s6-c-0035; cross-scope-any and first gain s5-c-0081. Other case-level
gold and cross-scope metrics do not move. Net gold-first loss rejects adoption,
despite cross-scope gains. No threshold tuning or h2 run. This is iteration 35;
the isolated inference fixture and strict clippy passed, not a complete adopted
product validation. Experimental encoder removed and accepted source restored.

The gap to the matched external baseline slices remains unclosed. None of the 35
completed configurations has passed both selection and confirmation. No product
improvement is committed; research evidence is retained separately.

## Uppercase query atoms as hard obligations — not adopted

A mechanical query-only rule protects an unquoted ASCII alphanumeric atom of
at least two bytes with at least one letter and no lowercase letters, using the
existing camelCase hard-term contract. This is not acronym/full-name expansion.
The population contains 212 of 1,470 dev wordings. Heading/body matching replaces
ordinary parts evidence, and parts do not contribute TF for that atom; matcher B
retains the existing non-identifier Porter path. Subsequent rewrites retain their
existing behavior, including lowercase text where they already produce it. No
case-specific list, score weight, gate, model or input-body change.

Full dev: any 407/439 first 342/439, xs 30/42 and 24/42, empty 35/51, section
53/73. 160 ordered wording pages change. The mechanical parser fixture passes.
Gold and section regressions reject the rule; no acronym exceptions, retuning
or h2 run. Experimental source removed.
- gold_any_loss: `s3a-c-0054`, `s3b-c-0051`, `s3b-c-0064`, `s5-c-0074`.
- gold_first_gain: `s6-c-0083`.
- gold_first_loss: `s1b-c-0012`, `s3a-c-0054`, `s3b-c-0051`, `s3b-c-0064`, `s4-c-0003`, `s4-c-0018`, `s4-c-0062`, `s5-c-0050`, `s5-c-0143`, `s6-c-0100`.

Section attribution is aggregate: the first-page export does not describe every later-call span. Invocation median/p95 was 7.915/24.253 ms; diagnostic, not matched CLI validation.

## Bekko native replacement screen — eligible for a retrieval trial, not adopted

The [Bekko model card](https://huggingface.co/hotchpotch/bekko-static-embedding-v1)
and [pinned files](https://huggingface.co/hotchpotch/bekko-static-embedding-v1/tree/c18167b8f244c1bc6de21ce9b710bac8a29c4615)
specify a 200,000-row static vocabulary and Matryoshka truncation. Metadata shows
F16 [200000,1024] weights. A native build-only tool retains the first 512
coordinates, converts them losslessly to F32, and verifies every retained
coordinate's F16 round trip. The output table has 409,600,000 data bytes;
no additional quantization. This is materially larger than the accepted model.

The [Sentence Transformers StaticEmbedding implementation](https://github.com/huggingface/sentence-transformers/blob/main/sentence_transformers/sentence_transformer/modules/static_embedding.py)
uses all token IDs, including unknowns, without special tokens or padding, and
mean pooling. Native inference uses F32 mean and L2 normalization, Linn's
3,584-scalar/512-token ceilings and zero empty vectors. It is not a bit-parity
claim against PyTorch F16 pooling. Downloaded implementation is inspected only,
not executed; asset/source hashes and preparation code are retained locally.

The tokenizer-only lower bound was 7.637 versus 9.515 microseconds per query,
66.144 versus 74.200 per block, and 76.013 versus 40.676 milliseconds to load.
That did not exclude a full screen. Complete native inference then measured
8.814 versus 9.667 microseconds per query and 73.005 versus 74.445 per block;
warm-cache loads 67.658 versus 38.863 milliseconds. Both use three fresh
processes, seven alternating rounds, 687 query strings, 6,096 blocks, Rayon1.
The declared 1.25x encoding screen passes; slower loading and larger binary
remain constraints for actual CLI validation if retrieval qualifies. These are
not end-to-end CLI measurements. Independent f64 mean/normalization, Unicode,
unknown, repeated token, empty and token-limit fixtures pass; targeted strict
clippy passes. No score threshold or lexical mechanism is changed by the screen.

## Bekko 512-dimensional sole replacement — not adopted

One frozen native configuration, revision c18167b8f244c1bc6de21ce9b710bac8a29c4615.
The first 512 coordinates are converted losslessly from published F16 to F32;
mean pooling includes all token IDs, then F32 normalization. Candidate tokenizer
also supplies model pieces. Original source bodies, lexical/POS mechanisms,
thresholds, card assembly and scorer stay unchanged. A fresh private store and
model-publication identity prevent reusing accepted vectors.

Full dev: any 411/439 first 349/439, xs 30/42 and 24/42, empty 35/51, section
56/73. Gold-first falls by two; section gain alone cannot adopt a change.
No threshold tuning or h2 run. Experimental encoder and diagnostic examples
removed from build paths. Native model fixture and targeted strict clippy passed;
no adopted-product validation or CLI equivalence claim. Store preparation was
18.700 seconds; resident median/p95 5.986/15.639 ms is diagnostic only.

80 of 1,470 ordered wording pages change.
- gold_first_gain: `s2b-c-0014`.
- gold_first_loss: `s3b-c-0015`, `s5-c-0098`, `s6-c-0035`.

Section attribution is aggregate; first-page record exports do not establish later-call span changes. This completes configuration 37. No adopted gain or matched-external baseline gap closure is established.

## Exact source colon-form recovery — dev gain, held after flat h2

A native audit finds five original dev queries with Unknown-prefix parsing and
colon forms appearing in original body text: s2b-c-0020, s3a-c-0018, s3a-c-0028,
s4-c-0020 and s5-c-0005. The audit checks substrings, not full eligibility or
admission. One fixed candidate after an empty ordinary page quotes an unknown
bare namespace:value atom, preserving known filters and all other query text.
Prefixes use lowercase letters/hyphens; values begin with an ASCII letter and
continue with letters/digits/underscore/hyphen. Quoted spans, malformed quotes,
empty values and trailing punctuation are not reinterpreted. Refuse a proposal
that still does not parse. Original lexical search and fallback gating run on
the proposed query. Every retained card on every call must have every proposed
atom verbatim, case-sensitive, in its own original block, with alphanumeric,
underscore, hyphen and colon boundaries. No alias, model, score, threshold, body
rewrite or term dropping. A preexisting nonempty ordinary page is untouched.

Dev rises to any 411/439 first 352/439, xs 30/42 and 24/42, empty 35/51,
section 55/73. The sole case-level gain is gold-first s2b-c-0020: its original
page changes from s3a-r-0038 to gold s2b-r-0026. Other changed wording-0 pages:
s3a-c-0018 empty to s3a-r-0038/s3a-r-0043; s4-c-0020 appends s4-r-0115 after
s4-r-0029; s5-c-0005 changes from four unrelated records to s5-r-0001 but still
does not score a gold gain. Four of 1,470 ordered pages change; no case-level
metric losses. Frozen dev replay is byte-identical to all candidate pages.

Parser/boundary tests and a native store fixture pass: `build refresh` cannot
witness `build:refresh`, and every returned call is filtered. Strict library
clippy passes after a semantics-preserving parity-expression fix. No claim of
full adopted-product validation.

One frozen h2-all confirmation stays any 341/354 first 295/354, xs 57/62 and
50/62, empty 56/106, section 239/271. Flat h2 makes this a hold, not adoption.
No retuning, commit or matched-external baseline closure claim. Source restored; frozen
implementation, commands, outputs and page deltas remain local. Prior h2
exposure limits any claim of independent generalization from this confirmation.

H2 changes 1 of 1,380 ordered pages; case-level changes: {}.
- h2a1-c-0039 wording 1: [] → ['s3a-r-0038', 's3a-r-0065'].

## Information-weighted primary admission — not adopted

One fixed configuration replaces the primary matchers' equal-term count cutoff
with the same required fraction of existing BM25 IDF mass. Sum scoring-word IDF
within each content term, retaining existing absent-word and multiword behavior.
Require at least min(2,c) matching content terms and matched mass >= total mass
* need(c)/c. Hard terms remain mandatory. Required candidate feature reads use
the same admission test. Original count maps, tiers, BM25/POS scores, one-under
reads and fills, heading/record fills, penalties, fallbacks, model, source bodies
and scorer remain unchanged. This differs from closed score coordination and
IDF-power experiments: it changes primary admission, not score weighting.

Full dev: any 398/439 first 328/439, xs 30/42 and 23/42, empty 36/51, section
56/73. Gold and cross-scope-first regressions reject the rule despite the two
other gains. No h2 run or weight/threshold tuning. Equal-unit-mass cutoff,
rare-pair, frequent-triple, single-term minimum and empty fixtures pass; strict
library clippy passes. Experimental implementation removed. Invocation median/
p95 8.391/26.315 ms is diagnostic, not matched CLI validation.

862 of 1,470 ordered pages change. Case-level changes:
- gold_any_gain: `s3b-c-0066`.
- gold_any_loss: `s1b-c-0040`, `s2a-c-0001`, `s2b-c-0001`, `s2b-c-0013`, `s2b-c-0031`, `s4-c-0001`, `s4-c-0005`, `s5-c-0058`, `s5-c-0074`, `s5-c-0102`, `s5-c-0112`, `s5-c-0133`, `s6-c-0003`, `s6-c-0004`.
- gold_first_gain: `s3b-c-0018`, `s3b-c-0066`, `s4-c-0013`.
- gold_first_loss: `s2a-c-0001`, `s2b-c-0001`, `s2b-c-0013`, `s2b-c-0051`, `s3a-c-0020`, `s3b-c-0027`, `s4-c-0003`, `s4-c-0005`, `s4-c-0012`, `s4-c-0021`, `s4-c-0062`, `s4-c-0129`, `s4-c-0132`, `s5-c-0026`, `s5-c-0027`, `s5-c-0087`, `s5-c-0102`, `s5-c-0112`, `s5-c-0133`, `s5-c-0148`, `s5-c-0150`, `s6-c-0003`, `s6-c-0004`, `s6-c-0017`, `s6-c-0036`, `s6-c-0040`.
- xs_any_gain: `s2b-c-0074`.
- xs_any_loss: `s4-c-0132`.
- xs_first_loss: `s4-c-0064`.

Empty/section attribution is aggregate; first-page exports do not establish all-call span changes. No qualifying or committed retrieval gain is claimed.

## Extensionless technical file addressing — dev gain, held after unchanged h2

The earlier dotted-basename screen did not test a name with its final extension
omitted. A native audit over original JSONL records, projected lifecycle at the
fixed clock and native parsed queries finds 25 wordings naming an exact source
basename stem containing underscore or hyphen. Only s1b-c-0050 wording0 exposes
an unshown gold candidate, s1b-r-0044. Requiring all remaining content stems in
one original block leaves that record as the unique witness. The audit opens
no source_path and reads no author fields. Its population logic is permissive
about query filters; the executable candidate enforces them.

One fixed candidate runs after existing fallbacks, before quoted-name priority.
A bare underscore/hyphen query term can address a source basename with only its
last extension omitted. Require at least one other content stem, and every
remaining content stem in one original block, excluding separate heading and
path evidence. Respect the current eligibility clock and query filters. Only
when qualifying blocks belong to exactly one record, promote its earliest
block through existing filename-card mechanics; keep later calls. No score,
threshold, model, body enrichment or author hints. A bounded prefix lookup uses
the existing basename primary key, followed by exact final-extension validation;
only candidate records/blocks are loaded. No new persistent index or table.

Dev: any 411/439 first 352/439, xs 30/42 and 24/42, empty 35/51, section
55/73. Only s1b-c-0050 wording0 changes: [s6-r-0079] becomes
[s1b-r-0044, s6-r-0079]. One gold-first gain, no metric losses. Frozen replay
is byte-identical across all 1,470 candidate pages. A reopened-store fixture
checks original-body evidence, longer multi-extension refusal, missing-context
refusal, query filters and forget. Native EXPLAIN confirms a primary-key range
search. Fixture and strict library clippy pass; no full adopted-product checks
are claimed.

One frozen h2-all confirmation: any 341/354 first 295/354, xs 57/62 and
50/62, empty 56/106, section 239/271. All 1,380 ordered pages are identical
to accepted baseline. This is a hold, not adoption; no h2 retuning or commit.
Historical reuse of h2 limits independent-generalization claims.

The frozen candidate was also measured on matched original-document indexes
against the same archived external baseline predictions; the baseline was not rerun. Large-s1
gold-first improves from 39/48 to 40/48, tying the recorded baseline at 40/48. Linn MRR rises to
60.764%, below the recorded baseline at 63.090%; gold-any stays45/48 versus the recorded baseline at 46/48. Small-s1
gold-first rises from41/48 to42/48, still below the recorded baseline at 47/48; MRR rises to67.604%
versus the recorded baseline at 64.479%; gold-any stays46/48 versus the recorded baseline at 47/48. Cross-scope and empty
are unchanged. Large-s5 is unchanged: gold76/81 any67/81 first, xs7/10 and
5/10, empty9/10, MRR55.782%; recorded hybrid78/81,64/81,9/10,7/10,9/10,
MRR56.214%. These measurements show a narrow gap reduction, not a hybrid win
or permission to override the adoption rule. Section spans are unavailable
from the archived hybrid pages. Concurrent research runs make timing samples
unsuitable for matched latency claims.

Experimental source restored and audit helper removed from build targets.
Frozen implementation, population/witness ledgers, SQL fixture, full native
outputs and matched comparison artifacts remain ignored locally.


## 2026-11-10 — Fixed SSE v2 and technical file addressing together

A declared joint control combines the unchanged SSE v2 encoder and the unchanged
technical file-stem rule. No threshold sweep, runtime switch, per-case routing,
author fields, second model or reranker. The standalone SSE control lost one
dev first result; file addressing gained one but was flat on h2. Their combination
is selected jointly, not a reversal of either standalone verdict.

Dev: any 411/439 first 351/439, xs 31/42 and 25/42, empty 35/51, section 55/73.
82/1,470 ordered wording pages change. Gold-first gains s1b-c-0050 and s6-c-0073,
loses s4-c-0020 and s6-c-0035; both cross-scope counts gain s5-c-0081. No other
case-level gold/cross-scope outcomes change; empty and section totals are flat,
with no per-case attribution from the first-page-only export. Against SSE alone, precisely one page changes:
s1b-c-0050 wording 0 prepends source s1b-r-0044 to s6-r-0079. This establishes
additive first-page effects, not a claim of full-response identity.
The frozen dev replay is byte-identical. One frozen h2-all run: any 341/354
first 297/354, xs 57/62 and 50/62, empty 56/106, section 239/271. 96/1,380
pages change. Gold-first gains h2a1-c-0065, h2a3-c-0046 and s7-c-0029, loses
s7-c-0089. No other case-level gold/cross-scope outcomes change; the empty/section
comparison establishes aggregate parity only. H2 reuse limits independence;
this configuration was not tuned after confirmation.

Matched archived hybrid comparison, newly rebuilt Linn populations: large-s1
any 45/48, first 40/48, MRR@5 60.764% (external baseline 46/48, 40/48, 63.090%);
large-s5 any 76/81, first 66/81, MRR@5 56.091% (external baseline 78/81, 64/81, 56.214%),
xs 8/10 and 6/10 (external baseline 9/10 and 7/10); small-s1 any 46/48, first 42/48,
MRR@5 67.604% (external baseline 47/48, 47/48, 64.479%). The large-s5 first count loses one
versus the accepted 67/81 even though it remains ahead of external baseline. Other matched
empty/cross-scope counts stay unchanged. The gap narrows but is not closed.
This uses archived external baseline predictions, not a new execution of the external system.

Synthetic six-document benchmark remains Hit@1 31/49, Hit@3 and Hit@5 35/49,
MRR@5 0.6735. Fresh CLI sample: first 12 dev original queries, two alternating
rounds, 24 invocations per binary, warm filesystem cache, invocation through exit.
Median accepted 58.0 ms versus candidate 46.5 ms (rounded); individual queries
vary and the small sample is not a cold-start guarantee. The earlier isolated
encoding screen measured query 9.604 versus 9.590 microseconds and blocks 74.720
versus 73.621 microseconds. One embedded 62.5 MB weight table, no runtime download.
The native checked loader, numerical fixture and explicit incompatible-store
rebuild requirement accompany selection. Production checks are recorded separately.


Selected joint change: production dev export is byte-identical to the frozen
1,470-page export. Formatting, strict workspace/all-targets/all-features Clippy,
59 workspace tests (one external-data test ignored), LLP references and Nix
release build/checks pass. Workspace tests used one test thread; Nix's release
checks also passed. The packaged binary passed its no-Nix-path checks and two
response comparisons with runtime asset paths absent. Production changes after
confirmation only validate assets, report loading errors and reject incompatible
vector publications; inference and selected pages remain unchanged. Kernel
identity distinguishes macOS vector tanh from scalar tanh on other platforms.
The release binary is about 90 MiB versus 155 MiB before. This round has not
rerun full scale or neutral tables and makes no new claim about those gates.
The local CLI helper's historical `mrl` label denotes this SSE candidate.


## Attested closed-compound components — not adopted

One fixed control on the selected SSE/file-addressing baseline lets Matcher B
match an ordinary content term to the original body token of a closed compound.
The compound's first tokenizer piece has at least four ASCII lowercase letters
and the same Porter stem as the query; its original remainder has at least three.
Both parts must already be indexed vocabulary terms. A bounded FTS vocabulary
prefix query supplies candidates. Hard terms, phrase tiers, all scores and
thresholds, ingestion bytes, author fields and model vectors stay unchanged.
This tests additional literal component coverage, not semantic equivalence.

The preliminary exact-two-piece screen was not a retrieval trial. It showed why
a first-boundary screen is distinct: toolchain gives retained pieces tool/chai,
while its original tail chain is a separate attested word. The scored rule uses
that original tail; it does not fabricate a tokenizer token or a source word.
Attestation is the existing heading/body/parts vocabulary, not an author dictionary.

Dev any411/439 first351/439 xs31/42 and25/42 empty35/51 section55/73: all totals
flat. Fourteen of1,470 ordered wording pages change, with zero case-level
gold/cross-scope changes. Empty/section comparison is aggregate only. The original
Rust/tools query changes its page but still does not retrieve its gold record;
additional component admission alone does not establish a useful recovery.

Changed wording pages: s1-c-0003 wording 0, s2b-c-0049 wording 2, s3b-c-0010 wording 1, s4-c-0011 wording 0, s4-c-0015 wording 1, s4-c-0035 wording 2, s4-c-0041 wording 2, s4-c-0071 wording 1, s5-c-0011 wording 0, s5-c-0048 wording 2, s5-c-0062 wording 1, s5-c-0142 wording 1, s6-c-0010 wording 1, s6-c-0021 wording 2.

Corpus-free match, hard-term, and forget-attestation fixture passes, as does
strict library Clippy. Observed resident median8.163/p9525.203ms, not an isolated
causal latency comparison or a fresh CLI measurement. No qualifying retrieval
rise; prototype removed, no h2, scale, neutral or hybrid run. Native source,
protocol, executable and exact changed pages remain in ignored local output.

Restored selected-source executable reproduced all six totals and all1,470
ordered pages byte-for-byte. Product source has no diff from66b55f9.


## Scored original compound components — not adopted

A separate fixed control adds lexical TF to the preceding admission-only trial.
For an ordinary content word whose existing weighted TF is zero, use0.3 times
the original body frequency of its qualifying compounds. This is the existing
parts weight, with original query IDF and row length unchanged. Do not supplement
an already matched query word or score a hard term through components. Fetch
compound frequencies by candidate IDs; no corpus-wide startup map, ingestion
change, synthetic words, author evidence or reranker. All other algorithms stay
on66b55f9. This tests original-component score contribution, not a new generic
BM25 parameter sweep.

Dev any411/439 first351/439 xs31/42 and25/42 empty35/51 section55/73, all flat.
40 of1,470 ordered wording pages change, with no case-level gold/cross-scope
changes. Empty and section parity is aggregate only. In the generic rank trace
for s1-c-0003's original wording, s1-r-0008 enters matcher B at raw rank12,
tier1, score9.516686695572027; it remains below the selected page. This explains
that admission alone is not the remaining barrier for this wording; it does not
establish that increasing component weight would improve full retrieval.

Changed wording pages: s1-c-0003 wording 0, s1b-c-0017 wording 1, s1b-c-0036 wording 1, s1b-c-0036 wording 2, s2a-c-0004 wording 2, s2b-c-0048 wording 1, s2b-c-0049 wording 0, s2b-c-0049 wording 2, s2b-c-0051 wording 2, s2b-c-0069 wording 1, s3a-c-0058 wording 1, s3b-c-0010 wording 1, s3b-c-0010 wording 2, s3b-c-0037 wording 2, s3b-c-0051 wording 2, s4-c-0001 wording 2, s4-c-0007 wording 0, s4-c-0011 wording 0, s4-c-0015 wording 1, s4-c-0035 wording 2, s4-c-0037 wording 0, s4-c-0041 wording 2, s4-c-0067 wording 0, s4-c-0071 wording 1, s4-c-0073 wording 2, s5-c-0011 wording 0, s5-c-0012 wording 2, s5-c-0048 wording 2, s5-c-0062 wording 1, s5-c-0081 wording 0, s5-c-0098 wording 0, s5-c-0102 wording 1, s5-c-0132 wording 1, s5-c-0142 wording 1, s5-c-0144 wording 0, s5-c-0152 wording 0, s6-c-0010 wording 1, s6-c-0021 wording 2, s6-c-0053 wording 0, s6-c-0109 wording 2.

Corpus-free fixtures verify the frequency equation, body-only contribution,
no double credit for an existing match, protected hard terms, and forget. They
and strict library Clippy pass. No qualifying rise: source restored, no h2,
scale, neutral or hybrid run. The executable, protocol, trace and exact page
diffs remain in ignored local output. The selected model and addressing change
remain the only product change adopted in this research round.


## Static-vector geometry references — research lead, unmeasured

[All-but-the-Top: Simple and Effective Postprocessing for Word Representations](https://arxiv.org/abs/1702.01417), Jiaqi Mu, Suma Bhat and Pramod Viswanath, studies removing the common mean and dominant directions from word representations. Its reported word/sentence-task gains motivate a corpus-independent geometry screen, but do not establish gains for retrieval-trained SSE vectors, fixed Linn gates, or this corpus.

[Whitening Sentence Representations for Better Semantics and Faster Retrieval](https://arxiv.org/abs/2103.15316), Jianlin Su, Jiarun Cao, Weijie Liu and Yangyiwen Ou, studies whitening sentence representations and dimensionality reduction for semantics/retrieval. This supports investigating anisotropy rather than assuming raw cosine is optimally calibrated. No whitening transform, dimensionality reduction, threshold change or retrieval benefit is adopted here. Any candidate must remain one embedded native encoder, fit without case/gold data, and pass the normal timing and complete retrieval measurements.


## Model-only mean and principal-direction removal — flat and held

Native fitting used only21,745 ASCII lowercase whole entries from the pinned
SSE tokenizer, in token-id order, encoded by the selected unmodified encoder.
No corpus body, case, scope, query, gold or author evidence fits the transform.
The mean norm is0.0289027582; centered covariance trace0.9991646479 and its
leading variance fraction0.0080048464. This is much weaker common-direction
concentration than the motivating word-vector examples; geometry alone does not
establish a retrieval defect. Fitting took0.539s using nalgebra0.35's symmetric
eigendecomposition in an isolated native research tool. Parameters total4KB,
compiled into the prototypes with no startup fitting or runtime file access.

Control44 subtracts the float32 fitted mean from each nonzero normalized SSE
output and renormalizes in coordinate order. Control45 additionally subtracts
the leading-direction projection of that centered vector, then normalizes.
Empty remains zero, all dimensions stay512, and thresholds, lexical retrieval,
POS, source addressing and scoring stay fixed. These are output-space
adaptations, not full whitening or a claim of verbatim paper implementation.
Fit mean/covariance are float64; cast parameters to float32, normalize the
direction and fix its largest absolute coordinate positive. The fitter was
built against66b55f9, not recursively against a postprocessed encoder.

44: any411/439 first351/439 xs31/42 and25/42 empty35/51 section55/73.
Two ordered dev pages change, no case-level gold/cross-scope outcome changes.
No qualifying rise, not adopted; no h2 for this control.
45: any411/439 first352/439 xs31/42 and25/42 empty35/51 section55/73.
Four ordered dev pages change. The sole case-level retrieval gain is original
wording s2b-c-0020, gold-first. No other gold/cross-scope changes; empty and
section totals are flat. Frozen replay matches all1,470 pages byte-for-byte.
The one frozen h2-all confirmation stays any341/354 first297/354 xs57/62 and
50/62 empty56/106 section239/271. This is a hold, not adoption, and is not
retuned. H2 is reused benchmark data, not a fresh independent sample.

Control44 changed dev wording pages: s4-c-0015 wording 0, s5-c-0005 wording 0.

Control45 changed dev wording pages: s1b-c-0031 wording 0, s1b-c-0041 wording 1, s2b-c-0020 wording 0, s4-c-0015 wording 0.

Control45 changed h2 wording pages: h2a2-c-0059 wording 1, h2a2-c-0103 wording 0, h2a3-c-0003 wording 0, h2a3-c-0081 wording 0, s7-c-0036 wording 1, s7-c-0088 wording 0, s7-c-0095 wording 1, s7-c-0095 wording 2, s7-c-0129 wording 0.

Independent float64-equation fixtures and strict library Clippy pass for both
controls; the selected control also passes workspace tests. No fresh CLI speed,
scale, neutral or matched-hybrid improvement is claimed. Both prototypes are
removed, parameters and native reproduction artifacts remain ignored. The
selected66b55f9 model and addressing behavior remain the product.


### Selected encoder and file addressing: scale-q4 validation

The accepted change at 66b55f9 was measured on the unchanged 16,906-record
scale-q4 corpus, mode dev. Gold-any/first are 386/439 and 312/439, versus
387/439 and 313/439 before this research. Cross-scope-any/first rise from
27/42 and 21/42 to 28/42 and 22/42. Empty stays 31/51; section stays 51/73.
This is a scale tradeoff, not a universal retrieval improvement. It does not
close the matched hybrid gap. No scale-driven retuning was performed.

Of 1,470 ordered pages, 68 change. Case-level gold-any loses s4-c-0002;
gold-first gains s1b-c-0050 and loses s4-c-0002 and s5-c-0067. Both cross-scope
metrics gain s5-c-0081. These attributions concern gold/cross-scope only;
empty and section were compared as aggregate measurements.

New-store construction takes 495.805 seconds. Resident agent invocation median
is 90.475 ms, p95 318.136 ms across 1,470 wordings; the earlier run measured
501.746 seconds, 86.238 ms and 315.223 ms respectively. Different run times
and machine load prevent a causal speed claim. These are not fresh-process
CLI timings or neutral per-call gates. New scale-h2 and full neutral tables
have not been run.

### Model-only token norm diagnostic

Across 21,745 ASCII lowercase whole-word vocabulary entries, raw embedding-row
L2 norms have mean 50.7886, standard deviation 12.2504, median 51.7297, and
range 8.3366–95.4684. Several function words have the smallest norms, including
that, with, which, by, an, for, from, as, also and the. Equalizing token norms
would therefore amplify these words relative to many content words. This
weakens the proposed normalization rationale; it does not establish training
intent or retrieval causality. No normalization candidate was implemented or
scored, and no retrieval benefit is claimed.


### Negation-preserving morphology: dictionary feasibility screen

A proposed query rewrite would preserve negation when interpreting an unknown
`un-` whole word whose base has a past-participle dictionary tag, unlike the
closed experiment that discarded the prefix. The current pinned dictionary
invalidates the motivating premise: `unread` already has an adjective entry.
Across dev prepared queries, all 17 distinct alphabetic `un-` forms fail the
proposed unknown-whole/participle-base condition. The forms are unavailable,
uncommitted, undefined, unicode, universe, unknown, unlinked, unnecessary,
unneeded, unpublished, unread, unrelated, unsigned, unsupported, untrusted,
unused and unverified. Additional ambiguity controls (unpacked, unwrapped,
unlocked, unrolled, undone, uninstalled, unmounted) also have whole-word entries.
No retrieval implementation, scoring configuration, or benefit is claimed. Do
not generalize this screen into evidence that all polarity-aware matching is
ineffective. The frozen narrow proposal is inactive and was not broadened to
force a benchmark activation. Original diagnostic source and output are kept
locally; product source is unchanged.


### Fixed trained-prefix encoder control — not adopted

The [SSE Retrieval MRL v2 technical report](https://huggingface.co/blog/RikkaBotan/stable-static-embedding-v2-technical-report)
reports Matryoshka training and NanoBEIR mean nDCG@10 of 0.5036 at 256 dimensions
versus 0.5159 at 512. This supports a defined shorter-prefix control, not a claim
that shortening improves Linn. One fixed configuration keeps the first 256
post-transform coordinates, zeros the rest of the existing 512-wide vector,
and normalizes once. Empty stays zero. Tokenization, thresholds, lexical rules,
POS, addressing and scoring are unchanged. No fitted geometry or corpus training
is used. Distinct model publication identity and a fresh store prevent mixing
vectors. The prototype retains full transformation arithmetic before truncation;
zero padding itself provides no storage or measured speed improvement.

Dev any411/439 first349/439 xs31/42 and25/42 empty35/51 section55/73.
Gold-first falls by two versus the selected baseline. No h2 or prefix-length
sweep follows. The independent float64 equation fixture passes. The source
change is removed; accepted 512-dimensional behavior remains.

Ordered pages changed: 37/1470. Case-level gold/cross-scope changes: {"gold_first_gain": ["s6-c-0035"], "gold_first_loss": ["s1-c-0001", "s5-c-0067", "s5-c-0098"]}.

Control46 changed wording pages: s1-c-0001 wording0, s1b-c-0005 wording1, s1b-c-0018 wording2, s1b-c-0034 wording2, s1b-c-0044 wording0, s1b-c-0049 wording0, s1b-c-0060 wording0, s1b-c-0060 wording1, s2b-c-0032 wording2, s2b-c-0038 wording1, s2b-c-0051 wording0, s2b-c-0060 wording0, s2b-c-0080 wording0, s3a-c-0022 wording1, s3b-c-0021 wording0, s3b-c-0033 wording1, s3b-c-0034 wording0, s3b-c-0052 wording0, s3b-c-0065 wording2, s4-c-0013 wording1, s4-c-0044 wording2, s4-c-0089 wording2, s5-c-0003 wording0, s5-c-0005 wording0, s5-c-0043 wording0, s5-c-0067 wording0, s5-c-0076 wording0, s5-c-0091 wording0, s5-c-0095 wording0, s5-c-0098 wording0, s5-c-0102 wording0, s5-c-0102 wording1, s6-c-0035 wording0, s6-c-0055 wording2, s6-c-0083 wording2, s6-c-0112 wording2, s6-c-0128 wording2.


### Selected encoder and file addressing: scale-h2 validation

The unchanged accepted 66b55f9 binary on 17,003 scale-h2 records, mode all,
measures any327/354 first276/354 xs48/62 and40/62 empty47/106 section226/271.
The previous baseline measured any326/354 first273/354 with the other four
totals unchanged. This is a broader-scale measurement, not fresh independent
holdout validation or an external baseline-gap closure; scale-q4 separately lost one gold-any
and one gold-first. Neither scale result was used to retune the implementation.

Store construction takes 534.052 seconds; 1,380 resident agent invocations have
median98.913 ms, p95 366.904 ms, maximum872.480 ms. These exclude corpus loading,
grading/output and are not end-to-end CLI or neutral-gate timings. Other research
builds and a curated run overlapped, so do not infer a causal latency change.
No new full neutral tables have been run.

Ordered page changes: 100/1380. Case-level gold/cross-scope changes: {"gold_any_gain": ["h2a3-c-0015", "s7-c-0124"], "gold_any_loss": ["s7-c-0062"], "gold_first_gain": ["h2a2-c-0003", "h2a2-c-0033", "h2a3-c-0015", "s7-c-0029", "s7-c-0124"], "gold_first_loss": ["h2a1-c-0097", "s7-c-0062"]}.

Selected scale-q4 changed wording pages: s1-c-0012 wording2, s1b-c-0004 wording0, s1b-c-0004 wording1, s1b-c-0007 wording0, s1b-c-0008 wording0, s1b-c-0008 wording2, s1b-c-0010 wording0, s1b-c-0015 wording2, s1b-c-0018 wording2, s1b-c-0025 wording0, s1b-c-0027 wording1, s1b-c-0028 wording2, s1b-c-0031 wording0, s1b-c-0034 wording2, s1b-c-0041 wording1, s1b-c-0041 wording2, s1b-c-0044 wording0, s1b-c-0049 wording0, s1b-c-0050 wording0, s1b-c-0060 wording0, s1b-c-0065 wording1, s2a-c-0004 wording1, s2a-c-0005 wording1, s2b-c-0014 wording0, s2b-c-0027 wording0, s2b-c-0027 wording2, s2b-c-0038 wording0, s2b-c-0038 wording1, s2b-c-0041 wording0, s2b-c-0050 wording0, s2b-c-0060 wording0, s2b-c-0080 wording0, s3a-c-0002 wording1, s3a-c-0028 wording0, s3b-c-0010 wording0, s3b-c-0028 wording0, s3b-c-0048 wording0, s3b-c-0050 wording0, s3b-c-0065 wording2, s4-c-0002 wording0, s4-c-0020 wording0, s4-c-0044 wording0, s4-c-0074 wording0, s4-c-0088 wording0, s4-c-0089 wording0, s4-c-0091 wording0, s4-c-0115 wording0, s4-c-0115 wording1, s5-c-0001 wording0, s5-c-0005 wording0, s5-c-0043 wording0, s5-c-0059 wording1, s5-c-0067 wording0, s5-c-0076 wording0, s5-c-0076 wording1, s5-c-0081 wording0, s5-c-0090 wording0, s5-c-0090 wording2, s5-c-0097 wording0, s5-c-0109 wording1, s5-c-0130 wording1, s5-c-0139 wording0, s6-c-0036 wording2, s6-c-0044 wording1, s6-c-0055 wording0, s6-c-0085 wording2, s6-c-0093 wording0, s6-c-0112 wording2.

Selected scale-h2 changed wording pages: h2a1-c-0014 wording0, h2a1-c-0014 wording2, h2a1-c-0029 wording2, h2a1-c-0030 wording2, h2a1-c-0031 wording1, h2a1-c-0042 wording2, h2a1-c-0064 wording0, h2a1-c-0081 wording0, h2a1-c-0092 wording0, h2a1-c-0097 wording0, h2a1-c-0101 wording0, h2a1-c-0104 wording2, h2a1-c-0108 wording0, h2a2-c-0001 wording2, h2a2-c-0003 wording0, h2a2-c-0006 wording0, h2a2-c-0016 wording2, h2a2-c-0018 wording0, h2a2-c-0031 wording2, h2a2-c-0033 wording0, h2a2-c-0039 wording1, h2a2-c-0040 wording0, h2a2-c-0043 wording0, h2a2-c-0048 wording0, h2a2-c-0057 wording2, h2a2-c-0059 wording0, h2a2-c-0065 wording1, h2a2-c-0066 wording1, h2a2-c-0073 wording2, h2a2-c-0091 wording0, h2a2-c-0098 wording0, h2a2-c-0099 wording0, h2a2-c-0099 wording2, h2a2-c-0105 wording0, h2a2-c-0107 wording0, h2a2-c-0108 wording0, h2a3-c-0003 wording2, h2a3-c-0005 wording0, h2a3-c-0006 wording0, h2a3-c-0007 wording0, h2a3-c-0009 wording0, h2a3-c-0011 wording0, h2a3-c-0015 wording0, h2a3-c-0016 wording0, h2a3-c-0024 wording1, h2a3-c-0029 wording0, h2a3-c-0038 wording0, h2a3-c-0052 wording0, h2a3-c-0060 wording0, h2a3-c-0064 wording0, h2a3-c-0065 wording0, h2a3-c-0073 wording0, h2a3-c-0074 wording1, h2a3-c-0075 wording2, h2a3-c-0081 wording1, h2a3-c-0082 wording0, h2a3-c-0082 wording1, h2a3-c-0085 wording1, h2a3-c-0088 wording0, s7-c-0011 wording0, s7-c-0011 wording1, s7-c-0019 wording0, s7-c-0019 wording1, s7-c-0020 wording1, s7-c-0021 wording0, s7-c-0029 wording0, s7-c-0030 wording0, s7-c-0031 wording1, s7-c-0032 wording2, s7-c-0036 wording0, s7-c-0036 wording1, s7-c-0037 wording0, s7-c-0038 wording0, s7-c-0039 wording1, s7-c-0041 wording0, s7-c-0062 wording0, s7-c-0062 wording1, s7-c-0064 wording0, s7-c-0065 wording0, s7-c-0088 wording1, s7-c-0090 wording1, s7-c-0097 wording0, s7-c-0100 wording1, s7-c-0100 wording2, s7-c-0101 wording0, s7-c-0105 wording0, s7-c-0107 wording0, s7-c-0121 wording1, s7-c-0123 wording0, s7-c-0124 wording0, s7-c-0124 wording1, s7-c-0125 wording0, s7-c-0126 wording0, s7-c-0126 wording1, s7-c-0132 wording0, s7-c-0140 wording0, s7-c-0144 wording0, s7-c-0145 wording0, s7-c-0145 wording1, s7-c-0150 wording0.


### Original-line hierarchical pooling — not adopted

One fixed control retains the same original 3,584-scalar input window and first
512 global token ids, grouping ids by the physical LF line containing each
token's start byte. Each nonempty group uses the unchanged SSE mean, learned
transform and normalization. Average those vectors in source order and normalize
once. A single group returns unchanged; empty stays zero. This tests equal line
contribution instead of token-count contribution, without dropping text or
adding tokens. Physical lines are not claimed to be semantic sentences. There
is no model fitting, author evidence, lexical/POS/gate change, or new model.
A distinct publication identity and fresh store isolate changed vectors.

Dev stays any411/439 first351/439 xs31/42 and25/42 empty35/51 section55/73.
The numerical equation, one-line parity, empty-line and global token-budget
fixtures pass. Store construction takes18.008 seconds; resident invocation
median6.050 ms and p95 15.934 ms are diagnostic only, not matched CLI validation.
No qualifying retrieval rise; no h2 or line-boundary tuning. Prototype removed.

Changed ordered pages: 29/1470. Case-level gold/cross-scope changes: {}.
Changed wording pages: s1b-c-0015 wording2, s1b-c-0018 wording1, s1b-c-0034 wording2, s1b-c-0044 wording0, s1b-c-0048 wording0, s1b-c-0060 wording1, s2b-c-0027 wording0, s2b-c-0032 wording2, s2b-c-0041 wording0, s2b-c-0080 wording0, s3a-c-0009 wording0, s3a-c-0022 wording1, s3b-c-0013 wording0, s3b-c-0021 wording0, s3b-c-0025 wording0, s3b-c-0028 wording0, s3b-c-0033 wording1, s3b-c-0035 wording2, s3b-c-0048 wording0, s4-c-0088 wording0, s4-c-0089 wording0, s4-c-0092 wording2, s4-c-0123 wording1, s5-c-0001 wording0, s5-c-0091 wording0, s5-c-0095 wording0, s6-c-0022 wording0, s6-c-0091 wording2, s6-c-0117 wording2.


### Token-magnitude weighted pooling — not adopted

One fixed control replaces the selected encoder's token mean with
sum(norm(row)*row)/sum(norm(row)). Row norms use the same pretrained raw table,
float32 serial squared-coordinate reduction and square root; sum weights in token
order and clamp denominator at1e-9. Retain all original token ids, scalar/token
limits, learned SSE transform and final normalization. Empty stays zero. Compute
norms only for used tokens; no vocabulary-wide startup scan, fitted parameters,
new model, source enrichment, lexical/POS change or threshold change.

The preceding model-only norm audit motivated a test of learned magnitude as
an importance proxy; it did not prove that interpretation. Dev any411/439
first350/439 xs31/42 and25/42 empty35/51 section55/73. Gold-first falls by one.
The independent float64 equation fixture passes, but retrieval fails selection.
No exponent sweep or h2 follows. Prototype removed. Store18.584 seconds,
resident median6.002 ms and p95 15.577 ms are diagnostic, not matched CLI cost.

Changed pages: 38/1470. Case-level gold/cross-scope changes: {"gold_first_gain": ["s6-c-0035"], "gold_first_loss": ["s1-c-0001", "s5-c-0098"]}.
Changed wording pages: s1-c-0001 wording0, s1-c-0012 wording2, s1b-c-0005 wording2, s1b-c-0008 wording2, s1b-c-0015 wording2, s1b-c-0025 wording0, s1b-c-0041 wording0, s1b-c-0044 wording0, s1b-c-0049 wording0, s1b-c-0059 wording0, s1b-c-0060 wording0, s2b-c-0041 wording0, s2b-c-0051 wording0, s2b-c-0060 wording0, s3b-c-0010 wording0, s3b-c-0035 wording2, s3b-c-0048 wording0, s3b-c-0054 wording0, s4-c-0015 wording0, s4-c-0025 wording1, s4-c-0044 wording2, s4-c-0080 wording1, s4-c-0089 wording0, s4-c-0089 wording2, s5-c-0003 wording0, s5-c-0005 wording0, s5-c-0076 wording0, s5-c-0091 wording0, s5-c-0097 wording0, s5-c-0098 wording0, s5-c-0102 wording0, s5-c-0102 wording1, s6-c-0006 wording0, s6-c-0035 wording0, s6-c-0039 wording0, s6-c-0043 wording1, s6-c-0044 wording1, s6-c-0053 wording2.


### Small contextual encoder feasibility references

The [bekko-embedding-v1-a8m model card](https://huggingface.co/hotchpotch/bekko-embedding-v1-a8m)
distinguishes7.7M active from106M total parameters, with four ModernBERT-style
layers and384 dimensions. Its native CPU recommendation is OpenVINO; published
throughput uses batching and does not establish fresh Linn CLI cost. No weights
were downloaded and no native performance conclusion follows from the card.

The [paraphrase-MiniLM-L3-v2 model card](https://huggingface.co/sentence-transformers/paraphrase-MiniLM-L3-v2)
specifies17.4M parameters,384-dimensional mean pooling and a128-token sentence
limit. The code uses mean pooling despite an adjacent prose comment saying max.
The [Candle BERT implementation](https://github.com/huggingface/candle/blob/main/candle-transformers/src/models/bert.rs)
provides an existing native implementation to investigate, rather than a new
transformer port. This is the candidate for an isolated embedded-asset speed
screen; no download, implementation, speed result or retrieval gain is claimed
at this reference-review stage. Any eventual replacement must pass native
packaging, complete-invocation latency and ordinary retrieval adoption checks.


### MiniLM-L3 native feasibility measurement — not advanced to retrieval

Pinned `sentence-transformers/paraphrase-MiniLM-L3-v2` revision
`4ca70771034acceecb2e72475f72050fcdde4ddc`. An isolated Rust executable embeds
its published69,569,488-byte safetensors plus tokenizer/config and uses Candle
0.11.1 BERT with system Accelerate on CPU. Published128-token limit including
special tokens; original3584-scalar bound; attention-mask mean pooling,
normalized384 coordinates padded to512. Empty/whitespace stays zero. No Python
inference, ONNX service or runtime asset reads. The harness runs with model
location overrides pointing at nonexistent paths. It links both encoders only
for comparison, not as a product design or final candidate binary-size claim.

The same first96 original q4 queries and first96 original section/block spans
are measured in three fresh processes per encoder, three rounds per process,
alternating order. RAYON_NUM_THREADS and VECLIB_MAXIMUM_THREADS are both1.
No gold, author fields or source-path file reads participate. Finite/unit output,
determinism, shape, zero-input and128-token-bound assertions pass. Strict Clippy
passes after a type-alias-only cleanup. This is not full cross-runtime embedding
parity or packaged CLI verification.

Across all864 samples per kind, query median is0.011458 ms accepted versus
1.0045 ms candidate; block median0.1559585 versus5.7913125 ms. Across the later
two processes, query median/p95 is0.011292/0.018958 versus1.0029165/1.2285 ms;
block0.1470005/0.232291 versus5.7942295/6.419542 ms. Median load in those two
processes is8.8959165 versus20.5648955 ms. First-process loads are12.273292 and
140.931917 ms; filesystem caches were not flushed. Raw samples are retained.

The candidate's substantially larger encoding cost does not warrant product
integration under the similarly-fast constraint. Not advanced; no Linn retrieval,
h2, scale, hybrid, or full CLI latency measurement is claimed. This is not a
measured failure of the proposed complete-invocation25% screen, and does not
establish that every contextual architecture/backend is too slow. Root product
source/dependencies remain unchanged; the reproducible benchmark and assets stay
ignored. This screen is not counted as another scored retrieval configuration.


### Explicit shell-fence command witness screen — not advanced

A source-only recognizer requires a complete matching backtick/tilde fence,
at most three leading spaces, an explicit sh/bash/zsh/shell info string and a
nonempty noncomment body. Unknown-language outer fences suppress nested apparent
shell fences. Corpus-free completeness/language/indentation/nesting fixtures pass.
The permissive source inventory finds274 blocks in87 q4 records. Dev has44
wordings with an ordinary unquoted command/commands term. Only four have a gold
record with such a block; three miss that record: s4-c-0027 wording1 and
s5-c-0112 wordings1/2. The fourth, s4-c-0047 wording1, already succeeds. No
original-wording gold miss has this witness. Eligibility and lexical admission
were not imposed in this upper-bound screen; doing so cannot create additional
source witnesses. This does not rule out indirect ordering effects or broader
command syntax. No rank bonus, retrieval configuration or h2 run was justified
by this narrow population; no score or gain is claimed.


### Explicit file cue for an ordinary basename stem — not adopted

One fixed extension to source-backed file addressing allows a plain ASCII
alphanumeric stem of at least3 bytes, beginning alphabetically, when an ordinary
unquoted file/files term occurs in the query. The name must be a whole raw
whitespace atom after normal edge punctuation removal, not file/files itself;
internal dots/slashes are not split into addresses. Require at least two distinct
remaining content stems together in one original block. Reuse exact final-stem
matching with a nonempty extension, indexed basename range, eligibility/filters,
unique qualifying record, earliest block, ordinary excerpt and page promotion.
Existing underscore/hyphen addressing remains unchanged. No filename enters
body/lexical/vector evidence, no author hints, new model or threshold changes.

Dev stays any411/439 first351/439 xs31/42 and25/42 empty35/51 section55/73.
No qualifying rise, no h2 or cue/coverage tuning; prototype removed. Corpus-free
positive, missing-cue, one-stem, dotted/path-boundary, ambiguity, filter and forget
fixtures pass along with the existing indexed-lookup/addressing fixture. Reused
store open8.421 ms, resident median8.041 ms/p95 24.161 ms are diagnostic only,
not controlled CLI speed comparisons.

Changed pages: 1/1470. Case-level gold/cross-scope changes: {}.
Changed wording pages: s3a-c-0035 wording1.


### PL2 primary lexical scoring — not adopted

One fixed PL2 Normalisation2 control, c=1, replaces only the BM25 word
contribution. Weighted TF and eligible collection frequency use body +3*heading
+0.3*parts, excluding parts for hard terms. With lambda=CF/N and
tfn=tf*log2(1+avg_length/max(length,1)), contribution is
[tfn*log2(tfn/lambda)+(lambda-tfn)*log2(e)+0.5*log2(2*pi*tfn)]/(tfn+1).
Zero TF or CF contributes zero. Signed finite values remain. Query multiplicity,
POS weight0.75, original-scope penalty7, admission, tiers, ties, interleave,
fallbacks and model remain fixed. This is a field adaptation, not whole-system
Terrier parity. Collection counts use existing term-constrained libSQL postings
and the clock-specific query cache; no persistent table or corpus-wide Rust map.

Dev: any412/439 first348/439 xs32/42 and24/42 empty36/51 section55/73,
against accepted411/351,31/25,35,55. Gold-first and cross-scope-first fall;
rejected despite any/empty gains. No h2, parameter sweep or score-scale repair.
Prototype removed. Eligible collection-count and independent score-equation
fixture passes, including expiry and parts-only matches; library clippy passes.

Primary references: [Terrier PL2 implementation](https://raw.githubusercontent.com/terrier-org/terrier-core/5.x/modules/core/src/main/java/org/terrier/matching/models/PL2.java)
provides the default c and equation; [Terrier retrieval configuration](https://github.com/terrier-org/terrier-core/blob/5.x/doc/configure_retrieval.md)
describes the weighting-model interface. The fixed family does not qualify here.

Changed pages: 723/1470. Case-level gold/cross-scope changes: {"gold_any_gain": ["s6-c-0074"], "gold_first_gain": ["s3a-c-0063"], "gold_first_loss": ["s4-c-0003", "s4-c-0006", "s5-c-0012", "s6-c-0073"], "xs_any_gain": ["s2b-c-0076"], "xs_first_loss": ["s5-c-0082"]}.
Changed wording pages: s1-c-0001 wording2, s1-c-0002 wording2, s1-c-0003 wording2, s1-c-0004 wording0, s1-c-0004 wording1, s1-c-0004 wording2, s1-c-0006 wording0, s1-c-0008 wording0, s1-c-0008 wording2, s1-c-0009 wording2, s1-c-0011 wording2, s1-c-0016 wording0, s1-c-0016 wording1, s1b-c-0001 wording0, s1b-c-0001 wording1, s1b-c-0001 wording2, s1b-c-0002 wording0, s1b-c-0002 wording1, s1b-c-0004 wording2, s1b-c-0006 wording1, s1b-c-0007 wording1, s1b-c-0008 wording1, s1b-c-0009 wording2, s1b-c-0011 wording1, s1b-c-0012 wording1, s1b-c-0013 wording0, s1b-c-0013 wording2, s1b-c-0015 wording1, s1b-c-0017 wording0, s1b-c-0017 wording1, s1b-c-0017 wording2, s1b-c-0018 wording0, s1b-c-0022 wording0, s1b-c-0024 wording0, s1b-c-0024 wording1, s1b-c-0024 wording2, s1b-c-0025 wording2, s1b-c-0026 wording0, s1b-c-0026 wording2, s1b-c-0028 wording0, s1b-c-0028 wording1, s1b-c-0029 wording0, s1b-c-0029 wording1, s1b-c-0031 wording2, s1b-c-0033 wording0, s1b-c-0033 wording1, s1b-c-0033 wording2, s1b-c-0034 wording2, s1b-c-0035 wording1, s1b-c-0035 wording2, s1b-c-0036 wording0, s1b-c-0037 wording0, s1b-c-0037 wording1, s1b-c-0038 wording2, s1b-c-0043 wording2, s1b-c-0044 wording1, s1b-c-0044 wording2, s1b-c-0045 wording0, s1b-c-0046 wording0, s1b-c-0047 wording0, s1b-c-0047 wording2, s1b-c-0048 wording0, s1b-c-0048 wording2, s1b-c-0052 wording0, s1b-c-0052 wording1, s1b-c-0054 wording0, s1b-c-0059 wording0, s1b-c-0060 wording2, s1b-c-0061 wording0, s1b-c-0061 wording1, s1b-c-0061 wording2, s1b-c-0062 wording1, s1b-c-0065 wording2, s1b-c-0066 wording0, s1b-c-0066 wording2, s1b-c-0067 wording0, s2a-c-0001 wording0, s2a-c-0002 wording0, s2a-c-0004 wording0, s2a-c-0004 wording2, s2a-c-0007 wording2, s2a-c-0008 wording1, s2a-c-0008 wording2, s2a-c-0009 wording1, s2a-c-0009 wording2, s2b-c-0001 wording1, s2b-c-0001 wording2, s2b-c-0002 wording0, s2b-c-0002 wording1, s2b-c-0002 wording2, s2b-c-0003 wording1, s2b-c-0004 wording1, s2b-c-0004 wording2, s2b-c-0006 wording1, s2b-c-0008 wording0, s2b-c-0008 wording1, s2b-c-0010 wording0, s2b-c-0010 wording1, s2b-c-0011 wording0, s2b-c-0011 wording1, s2b-c-0013 wording1, s2b-c-0014 wording2, s2b-c-0015 wording0, s2b-c-0015 wording1, s2b-c-0015 wording2, s2b-c-0016 wording0, s2b-c-0017 wording0, s2b-c-0019 wording1, s2b-c-0020 wording1, s2b-c-0020 wording2, s2b-c-0023 wording1, s2b-c-0023 wording2, s2b-c-0026 wording2, s2b-c-0030 wording2, s2b-c-0031 wording0, s2b-c-0031 wording2, s2b-c-0032 wording0, s2b-c-0033 wording0, s2b-c-0033 wording1, s2b-c-0033 wording2, s2b-c-0034 wording0, s2b-c-0034 wording1, s2b-c-0035 wording1, s2b-c-0035 wording2, s2b-c-0039 wording0, s2b-c-0039 wording1, s2b-c-0039 wording2, s2b-c-0041 wording1, s2b-c-0041 wording2, s2b-c-0043 wording1, s2b-c-0044 wording0, s2b-c-0045 wording1, s2b-c-0046 wording0, s2b-c-0046 wording1, s2b-c-0046 wording2, s2b-c-0048 wording1, s2b-c-0049 wording1, s2b-c-0049 wording2, s2b-c-0050 wording1, s2b-c-0052 wording2, s2b-c-0055 wording2, s2b-c-0056 wording0, s2b-c-0056 wording2, s2b-c-0058 wording1, s2b-c-0059 wording1, s2b-c-0059 wording2, s2b-c-0060 wording1, s2b-c-0061 wording2, s2b-c-0067 wording0, s2b-c-0067 wording1, s2b-c-0067 wording2, s2b-c-0068 wording0, s2b-c-0069 wording0, s2b-c-0073 wording0, s2b-c-0073 wording1, s2b-c-0073 wording2, s2b-c-0074 wording0, s2b-c-0074 wording1, s2b-c-0074 wording2, s2b-c-0076 wording0, s2b-c-0076 wording1, s2b-c-0076 wording2, s2b-c-0078 wording2, s2b-c-0079 wording1, s2b-c-0080 wording1, s2b-c-0080 wording2, s3a-c-0001 wording0, s3a-c-0001 wording2, s3a-c-0002 wording0, s3a-c-0003 wording1, s3a-c-0003 wording2, s3a-c-0004 wording1, s3a-c-0004 wording2, s3a-c-0005 wording0, s3a-c-0005 wording1, s3a-c-0005 wording2, s3a-c-0006 wording1, s3a-c-0006 wording2, s3a-c-0008 wording0, s3a-c-0008 wording1, s3a-c-0010 wording0, s3a-c-0010 wording1, s3a-c-0010 wording2, s3a-c-0012 wording0, s3a-c-0012 wording1, s3a-c-0013 wording0, s3a-c-0013 wording1, s3a-c-0014 wording0, s3a-c-0014 wording1, s3a-c-0015 wording1, s3a-c-0015 wording2, s3a-c-0016 wording0, s3a-c-0017 wording1, s3a-c-0017 wording2, s3a-c-0018 wording1, s3a-c-0018 wording2, s3a-c-0019 wording1, s3a-c-0019 wording2, s3a-c-0020 wording0, s3a-c-0021 wording0, s3a-c-0021 wording1, s3a-c-0022 wording0, s3a-c-0022 wording2, s3a-c-0023 wording0, s3a-c-0023 wording1, s3a-c-0023 wording2, s3a-c-0026 wording0, s3a-c-0027 wording0, s3a-c-0028 wording1, s3a-c-0028 wording2, s3a-c-0031 wording1, s3a-c-0031 wording2, s3a-c-0033 wording1, s3a-c-0033 wording2, s3a-c-0034 wording0, s3a-c-0034 wording1, s3a-c-0034 wording2, s3a-c-0035 wording1, s3a-c-0039 wording0, s3a-c-0039 wording2, s3a-c-0040 wording0, s3a-c-0040 wording1, s3a-c-0040 wording2, s3a-c-0041 wording1, s3a-c-0041 wording2, s3a-c-0042 wording0, s3a-c-0042 wording1, s3a-c-0042 wording2, s3a-c-0043 wording0, s3a-c-0043 wording1, s3a-c-0043 wording2, s3a-c-0044 wording1, s3a-c-0047 wording0, s3a-c-0048 wording1, s3a-c-0048 wording2, s3a-c-0049 wording0, s3a-c-0049 wording2, s3a-c-0050 wording0, s3a-c-0050 wording2, s3a-c-0052 wording0, s3a-c-0052 wording2, s3a-c-0053 wording1, s3a-c-0054 wording0, s3a-c-0054 wording1, s3a-c-0054 wording2, s3a-c-0056 wording1, s3a-c-0057 wording0, s3a-c-0057 wording1, s3a-c-0058 wording0, s3a-c-0058 wording1, s3a-c-0058 wording2, s3a-c-0062 wording0, s3a-c-0062 wording1, s3a-c-0063 wording0, s3a-c-0063 wording2, s3b-c-0002 wording0, s3b-c-0002 wording1, s3b-c-0002 wording2, s3b-c-0007 wording0, s3b-c-0007 wording2, s3b-c-0008 wording1, s3b-c-0008 wording2, s3b-c-0009 wording0, s3b-c-0009 wording2, s3b-c-0010 wording1, s3b-c-0013 wording1, s3b-c-0013 wording2, s3b-c-0014 wording0, s3b-c-0014 wording1, s3b-c-0016 wording1, s3b-c-0018 wording1, s3b-c-0019 wording0, s3b-c-0019 wording1, s3b-c-0020 wording1, s3b-c-0026 wording1, s3b-c-0028 wording2, s3b-c-0033 wording0, s3b-c-0033 wording2, s3b-c-0034 wording1, s3b-c-0035 wording1, s3b-c-0037 wording0, s3b-c-0037 wording1, s3b-c-0037 wording2, s3b-c-0038 wording1, s3b-c-0039 wording0, s3b-c-0039 wording1, s3b-c-0040 wording1, s3b-c-0041 wording0, s3b-c-0041 wording2, s3b-c-0042 wording0, s3b-c-0042 wording1, s3b-c-0045 wording1, s3b-c-0045 wording2, s3b-c-0046 wording1, s3b-c-0046 wording2, s3b-c-0047 wording0, s3b-c-0047 wording1, s3b-c-0047 wording2, s3b-c-0048 wording2, s3b-c-0049 wording0, s3b-c-0049 wording1, s3b-c-0049 wording2, s3b-c-0050 wording2, s3b-c-0051 wording1, s3b-c-0052 wording2, s3b-c-0053 wording0, s3b-c-0053 wording1, s3b-c-0053 wording2, s3b-c-0055 wording0, s3b-c-0055 wording1, s3b-c-0057 wording0, s3b-c-0057 wording1, s3b-c-0057 wording2, s3b-c-0058 wording2, s3b-c-0060 wording0, s3b-c-0060 wording1, s3b-c-0060 wording2, s3b-c-0061 wording0, s3b-c-0061 wording1, s3b-c-0063 wording0, s3b-c-0063 wording1, s3b-c-0065 wording0, s3b-c-0066 wording0, s3b-c-0066 wording1, s3b-c-0066 wording2, s3b-c-0067 wording0, s3b-c-0067 wording2, s4-c-0001 wording0, s4-c-0001 wording1, s4-c-0002 wording1, s4-c-0003 wording0, s4-c-0003 wording2, s4-c-0004 wording2, s4-c-0005 wording0, s4-c-0005 wording1, s4-c-0006 wording0, s4-c-0006 wording1, s4-c-0006 wording2, s4-c-0007 wording0, s4-c-0007 wording1, s4-c-0007 wording2, s4-c-0008 wording0, s4-c-0008 wording2, s4-c-0010 wording1, s4-c-0012 wording1, s4-c-0014 wording0, s4-c-0015 wording1, s4-c-0015 wording2, s4-c-0018 wording1, s4-c-0019 wording0, s4-c-0019 wording1, s4-c-0019 wording2, s4-c-0020 wording1, s4-c-0021 wording0, s4-c-0021 wording1, s4-c-0021 wording2, s4-c-0023 wording1, s4-c-0023 wording2, s4-c-0024 wording1, s4-c-0026 wording0, s4-c-0026 wording1, s4-c-0029 wording1, s4-c-0031 wording2, s4-c-0033 wording0, s4-c-0033 wording1, s4-c-0033 wording2, s4-c-0034 wording1, s4-c-0034 wording2, s4-c-0035 wording0, s4-c-0035 wording2, s4-c-0036 wording0, s4-c-0036 wording2, s4-c-0037 wording0, s4-c-0037 wording1, s4-c-0037 wording2, s4-c-0040 wording0, s4-c-0040 wording1, s4-c-0041 wording1, s4-c-0043 wording0, s4-c-0044 wording1, s4-c-0045 wording0, s4-c-0046 wording0, s4-c-0046 wording1, s4-c-0047 wording0, s4-c-0047 wording1, s4-c-0048 wording1, s4-c-0048 wording2, s4-c-0049 wording0, s4-c-0051 wording0, s4-c-0052 wording1, s4-c-0052 wording2, s4-c-0053 wording0, s4-c-0053 wording1, s4-c-0053 wording2, s4-c-0054 wording0, s4-c-0054 wording1, s4-c-0054 wording2, s4-c-0055 wording0, s4-c-0057 wording0, s4-c-0058 wording2, s4-c-0059 wording0, s4-c-0059 wording1, s4-c-0059 wording2, s4-c-0060 wording0, s4-c-0060 wording1, s4-c-0060 wording2, s4-c-0062 wording0, s4-c-0062 wording1, s4-c-0062 wording2, s4-c-0063 wording0, s4-c-0063 wording1, s4-c-0063 wording2, s4-c-0064 wording0, s4-c-0064 wording2, s4-c-0065 wording2, s4-c-0066 wording0, s4-c-0066 wording1, s4-c-0066 wording2, s4-c-0067 wording0, s4-c-0067 wording1, s4-c-0069 wording0, s4-c-0069 wording1, s4-c-0071 wording0, s4-c-0072 wording0, s4-c-0072 wording1, s4-c-0072 wording2, s4-c-0073 wording0, s4-c-0074 wording2, s4-c-0075 wording0, s4-c-0075 wording2, s4-c-0076 wording1, s4-c-0077 wording0, s4-c-0077 wording1, s4-c-0077 wording2, s4-c-0078 wording0, s4-c-0078 wording1, s4-c-0078 wording2, s4-c-0079 wording0, s4-c-0079 wording2, s4-c-0080 wording0, s4-c-0082 wording1, s4-c-0083 wording0, s4-c-0083 wording1, s4-c-0084 wording2, s4-c-0086 wording0, s4-c-0086 wording2, s4-c-0089 wording1, s4-c-0091 wording0, s4-c-0091 wording1, s4-c-0091 wording2, s4-c-0093 wording0, s4-c-0094 wording1, s4-c-0094 wording2, s4-c-0097 wording0, s4-c-0098 wording2, s4-c-0099 wording0, s4-c-0099 wording1, s4-c-0099 wording2, s4-c-0100 wording2, s4-c-0102 wording1, s4-c-0103 wording0, s4-c-0104 wording2, s4-c-0105 wording0, s4-c-0105 wording1, s4-c-0105 wording2, s4-c-0107 wording1, s4-c-0109 wording0, s4-c-0109 wording2, s4-c-0110 wording0, s4-c-0110 wording1, s4-c-0112 wording0, s4-c-0112 wording1, s4-c-0112 wording2, s4-c-0113 wording0, s4-c-0113 wording1, s4-c-0113 wording2, s4-c-0115 wording0, s4-c-0115 wording2, s4-c-0116 wording0, s4-c-0116 wording2, s4-c-0117 wording2, s4-c-0118 wording0, s4-c-0119 wording0, s4-c-0119 wording1, s4-c-0119 wording2, s4-c-0120 wording0, s4-c-0120 wording2, s4-c-0123 wording0, s4-c-0123 wording2, s4-c-0124 wording0, s4-c-0124 wording1, s4-c-0124 wording2, s4-c-0125 wording1, s4-c-0125 wording2, s4-c-0126 wording0, s4-c-0126 wording1, s4-c-0131 wording0, s4-c-0131 wording1, s4-c-0131 wording2, s4-c-0132 wording1, s4-c-0133 wording0, s5-c-0001 wording1, s5-c-0002 wording0, s5-c-0005 wording1, s5-c-0011 wording1, s5-c-0011 wording2, s5-c-0012 wording0, s5-c-0012 wording1, s5-c-0012 wording2, s5-c-0013 wording0, s5-c-0013 wording1, s5-c-0013 wording2, s5-c-0017 wording0, s5-c-0017 wording1, s5-c-0017 wording2, s5-c-0018 wording0, s5-c-0018 wording2, s5-c-0019 wording1, s5-c-0026 wording1, s5-c-0027 wording1, s5-c-0028 wording0, s5-c-0028 wording1, s5-c-0031 wording2, s5-c-0032 wording1, s5-c-0032 wording2, s5-c-0033 wording0, s5-c-0033 wording1, s5-c-0036 wording1, s5-c-0036 wording2, s5-c-0040 wording0, s5-c-0040 wording2, s5-c-0041 wording0, s5-c-0043 wording1, s5-c-0043 wording2, s5-c-0046 wording2, s5-c-0048 wording2, s5-c-0050 wording0, s5-c-0050 wording1, s5-c-0050 wording2, s5-c-0051 wording2, s5-c-0055 wording0, s5-c-0055 wording1, s5-c-0055 wording2, s5-c-0058 wording0, s5-c-0058 wording1, s5-c-0058 wording2, s5-c-0062 wording1, s5-c-0062 wording2, s5-c-0068 wording0, s5-c-0071 wording2, s5-c-0073 wording2, s5-c-0076 wording2, s5-c-0078 wording0, s5-c-0078 wording1, s5-c-0079 wording1, s5-c-0080 wording2, s5-c-0081 wording2, s5-c-0082 wording0, s5-c-0082 wording1, s5-c-0083 wording0, s5-c-0083 wording1, s5-c-0083 wording2, s5-c-0085 wording1, s5-c-0087 wording0, s5-c-0090 wording1, s5-c-0091 wording1, s5-c-0091 wording2, s5-c-0095 wording2, s5-c-0097 wording1, s5-c-0097 wording2, s5-c-0098 wording0, s5-c-0098 wording1, s5-c-0099 wording2, s5-c-0102 wording1, s5-c-0104 wording0, s5-c-0104 wording1, s5-c-0105 wording0, s5-c-0105 wording1, s5-c-0107 wording0, s5-c-0107 wording2, s5-c-0109 wording2, s5-c-0110 wording0, s5-c-0112 wording2, s5-c-0114 wording0, s5-c-0114 wording2, s5-c-0116 wording0, s5-c-0116 wording2, s5-c-0117 wording1, s5-c-0118 wording0, s5-c-0118 wording1, s5-c-0122 wording2, s5-c-0126 wording1, s5-c-0127 wording2, s5-c-0128 wording2, s5-c-0129 wording1, s5-c-0132 wording1, s5-c-0132 wording2, s5-c-0134 wording0, s5-c-0134 wording1, s5-c-0135 wording0, s5-c-0135 wording1, s5-c-0137 wording1, s5-c-0137 wording2, s5-c-0138 wording0, s5-c-0138 wording1, s5-c-0139 wording1, s5-c-0142 wording1, s5-c-0142 wording2, s5-c-0143 wording0, s5-c-0143 wording1, s5-c-0144 wording2, s5-c-0147 wording0, s5-c-0147 wording1, s5-c-0147 wording2, s5-c-0148 wording0, s5-c-0148 wording2, s5-c-0149 wording0, s5-c-0149 wording1, s5-c-0150 wording0, s5-c-0150 wording1, s5-c-0150 wording2, s5-c-0151 wording1, s5-c-0151 wording2, s5-c-0152 wording0, s5-c-0152 wording1, s5-c-0153 wording2, s5-c-0154 wording0, s5-c-0154 wording2, s6-c-0001 wording0, s6-c-0001 wording2, s6-c-0003 wording2, s6-c-0009 wording0, s6-c-0009 wording2, s6-c-0010 wording1, s6-c-0015 wording2, s6-c-0016 wording0, s6-c-0016 wording1, s6-c-0017 wording1, s6-c-0017 wording2, s6-c-0018 wording0, s6-c-0020 wording0, s6-c-0020 wording2, s6-c-0021 wording0, s6-c-0021 wording1, s6-c-0021 wording2, s6-c-0022 wording1, s6-c-0022 wording2, s6-c-0024 wording2, s6-c-0026 wording1, s6-c-0030 wording2, s6-c-0031 wording1, s6-c-0031 wording2, s6-c-0034 wording1, s6-c-0034 wording2, s6-c-0035 wording2, s6-c-0036 wording0, s6-c-0042 wording0, s6-c-0043 wording0, s6-c-0043 wording2, s6-c-0044 wording0, s6-c-0044 wording2, s6-c-0049 wording1, s6-c-0049 wording2, s6-c-0051 wording0, s6-c-0051 wording2, s6-c-0053 wording0, s6-c-0054 wording1, s6-c-0054 wording2, s6-c-0060 wording0, s6-c-0061 wording0, s6-c-0061 wording1, s6-c-0061 wording2, s6-c-0063 wording1, s6-c-0066 wording0, s6-c-0066 wording1, s6-c-0066 wording2, s6-c-0073 wording0, s6-c-0073 wording1, s6-c-0074 wording1, s6-c-0075 wording0, s6-c-0075 wording1, s6-c-0077 wording0, s6-c-0077 wording1, s6-c-0079 wording0, s6-c-0079 wording1, s6-c-0084 wording0, s6-c-0084 wording1, s6-c-0085 wording1, s6-c-0086 wording0, s6-c-0086 wording1, s6-c-0089 wording0, s6-c-0089 wording1, s6-c-0089 wording2, s6-c-0094 wording2, s6-c-0096 wording0, s6-c-0097 wording1, s6-c-0097 wording2, s6-c-0099 wording1, s6-c-0100 wording1, s6-c-0100 wording2, s6-c-0102 wording1, s6-c-0102 wording2, s6-c-0105 wording0, s6-c-0105 wording1, s6-c-0105 wording2, s6-c-0107 wording0, s6-c-0107 wording1, s6-c-0107 wording2, s6-c-0109 wording1, s6-c-0109 wording2, s6-c-0110 wording0, s6-c-0110 wording1, s6-c-0112 wording0, s6-c-0115 wording1, s6-c-0115 wording2, s6-c-0117 wording1, s6-c-0118 wording2, s6-c-0119 wording1, s6-c-0121 wording2, s6-c-0124 wording0, s6-c-0125 wording0, s6-c-0125 wording1, s6-c-0125 wording2.


### Possessive query population — no candidate

A q4 dev inventory of all1,470 prepared wordings found no word containing an
internal ASCII or typographic apostrophe. No possessive/contraction rewrite
was implemented or evaluated. This is a population screen, not proof such a
rewrite cannot help other workloads. Ignored inventory: possessive-screen.

### Token-local nonlinear transform before pooling — not adopted

One fixed control applies the same learned SSE beta*tanh(alpha*row+bias) to
each retained token row, averages those transformed rows in input order, then
L2-normalizes once. No individual-token normalization. Same original3584-scalar
window, first512 ids including unknowns, model weights and tokenizer; empty
remains zero. This is not the trained inference contract. It tests cancellation
before nonlinear transformation, distinct from line pooling (normalized groups)
and magnitude weighting. No fit, new parameters, additional model, lexical/POS
change, threshold change or scorer change. Fresh private store and distinct
publication identity prevent mixing vector arithmetic.

Dev any411/439 first352/439 xs30/42 and24/42 empty35/51 section56/73.
Accepted baseline is411/351,31/25,35,55. Cross-scope-any and first fall; rejected
despite first/section gains. No h2, tuning or combination; prototype removed.
Independent float64 equation/empty/window fixture and strict library clippy
pass. Fresh store18,991.006ms; resident median6.330ms/p9516.450ms/max47.614ms.
These are diagnostic timings, not a controlled performance comparison.

Changed pages: 37/1470. Case-level gold/cross-scope changes: {"gold_first_gain": ["s4-c-0020", "s6-c-0035"], "gold_first_loss": ["s6-c-0073"], "xs_any_loss": ["s5-c-0081"], "xs_first_loss": ["s5-c-0081"]}.
Changed wording pages: s1b-c-0007 wording0, s1b-c-0018 wording2, s1b-c-0025 wording0, s1b-c-0027 wording1, s1b-c-0045 wording2, s1b-c-0048 wording0, s1b-c-0060 wording0, s2b-c-0028 wording2, s2b-c-0032 wording2, s2b-c-0060 wording0, s3a-c-0022 wording1, s3b-c-0018 wording2, s3b-c-0020 wording0, s3b-c-0025 wording0, s3b-c-0028 wording0, s3b-c-0035 wording0, s3b-c-0046 wording0, s4-c-0020 wording0, s4-c-0044 wording2, s4-c-0089 wording0, s4-c-0092 wording2, s4-c-0123 wording1, s5-c-0003 wording0, s5-c-0043 wording0, s5-c-0081 wording0, s5-c-0091 wording0, s5-c-0098 wording0, s5-c-0102 wording1, s5-c-0151 wording0, s6-c-0035 wording0, s6-c-0044 wording1, s6-c-0055 wording0, s6-c-0055 wording2, s6-c-0073 wording0, s6-c-0083 wording2, s6-c-0112 wording2, s6-c-0128 wording2.


### Matched hybrid admission audit — diagnosis, no retrieval change

Rebuild private copies of the matched large/small stores from their immutable
records with the accepted encoder publication. Verify every returned card
(record, span, text and order) against all591 saved slice/wording responses (432 distinct wordings). Native
release fixture passes. The original comparison stores predate the model marker;
ordinary open correctly refuses them. No marker bypass or in-place vector update.

Select the75 slice/wording pairs where archived hybrid shows object-level gold
and Linn's first five cards do not. Run both native matchers on the original
query and inspect eligible, filter-allowed gold rows: coverage, missing hard
terms and native ranking. These are overlapping slices/wordings, not75
independent cases or a new score. Native library/test clippy passes.

large-s1: coverage=9, admitted=5, coverage+hard=3, zero-content-overlap=3.

large-s5: admitted=12, coverage=16, zero-content-overlap=3, hard-only=3, coverage+hard=2.

small-s1: zero-content-overlap=4, coverage=10, coverage+hard=2, admitted=2, no-content-query=1.

Of21 original-query slice misses, three have an admitted gold row, seven fail
coverage alone, five coverage plus hard terms, four have zero content overlap
(kind-word matches may exist), and two meet coverage but fail a hard term.
This rules out primary score-only changes as a direct fix for most of these
original-query misses. It does not trace all later fallback attempts or prove
indirect page interactions impossible. Admission relaxation is not adopted;
prior failed relaxations stay closed. Labels are used only for diagnostics.

Diagnostic source and JSON evidence are retained in ignored hybrid-gap-audit
output; temporary product instrumentation is removed. No h2 or ranker change.

Affected slice/case/wording ledger: large-s1:s1-c-0003/0=coverage, large-s1:s1-c-0010/2=admitted, large-s1:s1-c-0011/0=coverage, large-s1:s1-c-0012/2=coverage, large-s1:s1b-c-0005/2=coverage, large-s1:s1b-c-0007/0=coverage+hard, large-s1:s1b-c-0008/0=coverage+hard, large-s1:s1b-c-0008/1=coverage, large-s1:s1b-c-0009/2=admitted, large-s1:s1b-c-0010/2=admitted, large-s1:s1b-c-0011/1=admitted, large-s1:s1b-c-0040/0=zero-content-overlap, large-s1:s1b-c-0041/1=coverage, large-s1:s1b-c-0041/2=zero-content-overlap, large-s1:s1b-c-0045/2=coverage, large-s1:s1b-c-0047/1=coverage+hard, large-s1:s1b-c-0059/0=coverage, large-s1:s1b-c-0059/1=coverage, large-s1:s1b-c-0062/1=admitted, large-s1:s1b-c-0062/2=zero-content-overlap, large-s5:s5-c-0001/2=admitted, large-s5:s5-c-0004/1=coverage, large-s5:s5-c-0011/1=admitted, large-s5:s5-c-0012/2=coverage, large-s5:s5-c-0013/0=admitted, large-s5:s5-c-0041/0=coverage, large-s5:s5-c-0041/1=coverage, large-s5:s5-c-0046/2=admitted, large-s5:s5-c-0048/1=admitted, large-s5:s5-c-0048/2=admitted, large-s5:s5-c-0051/0=zero-content-overlap, large-s5:s5-c-0058/0=coverage, large-s5:s5-c-0059/1=zero-content-overlap, large-s5:s5-c-0059/2=hard-only, large-s5:s5-c-0062/1=coverage, large-s5:s5-c-0071/0=admitted, large-s5:s5-c-0080/2=coverage, large-s5:s5-c-0085/0=hard-only, large-s5:s5-c-0090/0=coverage+hard, large-s5:s5-c-0090/1=coverage, large-s5:s5-c-0090/2=coverage, large-s5:s5-c-0095/2=admitted, large-s5:s5-c-0097/1=coverage, large-s5:s5-c-0102/1=zero-content-overlap, large-s5:s5-c-0105/0=admitted, large-s5:s5-c-0106/0=hard-only, large-s5:s5-c-0109/1=coverage+hard, large-s5:s5-c-0112/2=coverage, large-s5:s5-c-0121/1=coverage, large-s5:s5-c-0121/2=coverage, large-s5:s5-c-0122/1=admitted, large-s5:s5-c-0124/2=coverage, large-s5:s5-c-0129/2=coverage, large-s5:s5-c-0134/0=coverage, large-s5:s5-c-0135/1=admitted, large-s5:s5-c-0150/1=admitted, small-s1:s1-c-0001/0=zero-content-overlap, small-s1:s1-c-0011/2=coverage, small-s1:s1-c-0012/2=coverage, small-s1:s1b-c-0005/2=coverage, small-s1:s1b-c-0006/1=coverage, small-s1:s1b-c-0007/0=coverage+hard, small-s1:s1b-c-0008/0=coverage+hard, small-s1:s1b-c-0008/1=coverage, small-s1:s1b-c-0009/2=admitted, small-s1:s1b-c-0023/2=admitted, small-s1:s1b-c-0027/2=coverage, small-s1:s1b-c-0040/0=zero-content-overlap, small-s1:s1b-c-0041/1=coverage, small-s1:s1b-c-0041/2=zero-content-overlap, small-s1:s1b-c-0049/2=coverage, small-s1:s1b-c-0050/2=no-content-query, small-s1:s1b-c-0059/0=coverage, small-s1:s1b-c-0059/1=coverage, small-s1:s1b-c-0062/2=zero-content-overlap.


### Adjacent source camel-component groups — population only

Native q4 dev screen: retain original alphabetic camelCase atoms, split at
lowercase-to-uppercase boundaries without discarding short pieces, concatenate
each contiguous group of at least two components, and compare per-word Porter
stems with ordinary non-hard content query terms. Source witnesses preserve
original atom byte spans; no author fields, source-file reads or generated
index text. This differs from single-piece stemming and the earlier attested
closed-compound first-word rule. It remains a diagnostic, not an index.

Across1,470 wordings, five parser errors are reported/skipped and28 query-term
matches have source witnesses. Four case/wording pairs have gold witnesses:
s1b-c-0008/1 coauthor in includeCoAuthoredBy (coauthored), s2a-c-0007/2
cocoapods in CocoaPods, s3a-c-0040/0 ios in iOS, s3a-c-0042/2 github in GitHub.
The last three already match whole original tokens; only the first supplies
a new within-atom group. That alternate belongs to a case already found on
another wording. No direct gold-first or gold-any gain is established; indirect
ordering effects remain unmeasured. No retrieval implementation, scoring trial,
tuning or h2 run. Targeted strict clippy passes; diagnostic source/output kept
in ignored hybrid-gap-audit artifacts and active example removed.


### Broader explicit-equivalence source screen — no candidate

The earlier E1 implementation required a whole-line, backtick-delimited
declaration and a controlled request grammar; it had no source declarations.
That experiment remains closed. This different source-only upper-bound screen
recognizes also called/known as, formerly called/known as, alias for/of,
means the same as, and synonym for/of anywhere in original body lines.
No endpoints are inferred, no equivalence is certified, and no index or
retrieval rule is created. It is broader than the closed parenthetical
initialism screen and does not rerun either retrieval experiment.

q4 contains21 marker occurrences in14 records. Eight dev cases have a gold
record in that set. Only three prepared wordings miss all object-level gold:
s4-c-0020/0 (simulator duration; source marker concerns tap/press),
s6-c-0003/1 (Volta versus current manager) and s6-c-0003/2 (build runtime
setup; source markers concern yarn watch/start and shell development aliases).
These marker occurrences do not assert the relationships requested by the
missed wordings. No direct missed-gold opportunity is established by this
finite marker inventory. This does not rule out other declaration syntax,
indirect ranking effects or equivalences outside this corpus. No scoring trial,
weight tuning, h2 run, model download or product change. Original UTF-8 line
spans, record ids and missed-wording inventory remain in ignored
explicit-equivalence-screen output.


### Coordinate-wise maximum static pooling — not adopted

One fixed control replaces raw token mean with the maximum of each coordinate
over retained token rows, initialized to negative infinity. Apply the existing
learned beta*tanh(alpha*maximum+bias) and final L2 normalization once. Empty
returns zero before reduction. Same original3584-scalar window, first512 ids
including unknowns,512 dimensions, weights and tokenizer. This is not the
trained mean-pooling contract; it tests whether a strong coordinate diluted by
mean pooling improves retrieval, with possible noise/length amplification.
No top-k, temperature, mixing, fitted parameter, new model, lexical/POS change,
admission relaxation, threshold or scorer change. Fresh private store and
distinct vector publication identity.

Dev any409/439 first344/439 xs30/42 and24/42 empty35/51 section55/73,
against accepted411/351,31/25,35,55. All four adoption metrics fall. Rejected
and removed; no h2, pooling sweep or combination. Independent float64 maximum/
transform equation, empty and token-window fixture passes; strict library
clippy passes. Fresh store17,950.206ms; resident median5.973ms/p9515.500ms/
max59.048ms are diagnostic, not a controlled CLI speed comparison.

Changed pages: 150/1470. Case-level gold/cross-scope changes: {"gold_any_loss": ["s4-c-0002", "s4-c-0010"], "gold_first_gain": ["s4-c-0023"], "gold_first_loss": ["s1-c-0001", "s1b-c-0018", "s4-c-0002", "s4-c-0010", "s5-c-0027", "s5-c-0067", "s5-c-0098", "s6-c-0073"], "xs_any_loss": ["s5-c-0081"], "xs_first_loss": ["s5-c-0081"]}.
Changed wording pages: s1-c-0001 wording0, s1-c-0012 wording2, s1b-c-0002 wording2, s1b-c-0004 wording1, s1b-c-0005 wording0, s1b-c-0007 wording0, s1b-c-0008 wording0, s1b-c-0008 wording2, s1b-c-0010 wording0, s1b-c-0015 wording2, s1b-c-0018 wording0, s1b-c-0018 wording1, s1b-c-0018 wording2, s1b-c-0025 wording0, s1b-c-0027 wording1, s1b-c-0031 wording0, s1b-c-0034 wording2, s1b-c-0041 wording0, s1b-c-0041 wording1, s1b-c-0041 wording2, s1b-c-0044 wording0, s1b-c-0045 wording2, s1b-c-0048 wording0, s1b-c-0049 wording0, s1b-c-0054 wording2, s1b-c-0059 wording0, s1b-c-0059 wording1, s1b-c-0059 wording2, s1b-c-0060 wording0, s1b-c-0062 wording2, s1b-c-0065 wording1, s2a-c-0005 wording1, s2a-c-0005 wording2, s2a-c-0008 wording0, s2b-c-0003 wording2, s2b-c-0014 wording0, s2b-c-0020 wording0, s2b-c-0026 wording0, s2b-c-0027 wording0, s2b-c-0027 wording1, s2b-c-0027 wording2, s2b-c-0028 wording0, s2b-c-0028 wording2, s2b-c-0032 wording2, s2b-c-0038 wording1, s2b-c-0041 wording0, s2b-c-0045 wording2, s2b-c-0050 wording0, s2b-c-0051 wording0, s2b-c-0055 wording0, s2b-c-0058 wording0, s2b-c-0060 wording0, s2b-c-0061 wording1, s2b-c-0078 wording0, s2b-c-0080 wording0, s3a-c-0009 wording0, s3a-c-0009 wording1, s3a-c-0009 wording2, s3a-c-0022 wording1, s3a-c-0027 wording1, s3b-c-0010 wording0, s3b-c-0013 wording0, s3b-c-0018 wording0, s3b-c-0020 wording0, s3b-c-0021 wording0, s3b-c-0025 wording0, s3b-c-0028 wording0, s3b-c-0031 wording0, s3b-c-0035 wording0, s3b-c-0035 wording2, s3b-c-0046 wording0, s3b-c-0048 wording0, s3b-c-0050 wording0, s3b-c-0052 wording0, s3b-c-0054 wording0, s3b-c-0064 wording1, s3b-c-0065 wording1, s3b-c-0065 wording2, s4-c-0002 wording0, s4-c-0005 wording2, s4-c-0010 wording0, s4-c-0013 wording1, s4-c-0013 wording2, s4-c-0015 wording0, s4-c-0023 wording0, s4-c-0025 wording0, s4-c-0025 wording1, s4-c-0029 wording2, s4-c-0044 wording0, s4-c-0044 wording2, s4-c-0067 wording2, s4-c-0074 wording0, s4-c-0084 wording0, s4-c-0088 wording0, s4-c-0089 wording0, s4-c-0089 wording2, s4-c-0091 wording0, s4-c-0092 wording2, s4-c-0115 wording0, s4-c-0115 wording1, s5-c-0001 wording0, s5-c-0003 wording0, s5-c-0027 wording0, s5-c-0043 wording0, s5-c-0046 wording0, s5-c-0059 wording1, s5-c-0067 wording0, s5-c-0076 wording0, s5-c-0076 wording1, s5-c-0081 wording0, s5-c-0087 wording1, s5-c-0090 wording0, s5-c-0090 wording2, s5-c-0091 wording0, s5-c-0095 wording0, s5-c-0097 wording0, s5-c-0098 wording0, s5-c-0102 wording0, s5-c-0102 wording1, s5-c-0109 wording1, s5-c-0124 wording0, s5-c-0130 wording1, s5-c-0132 wording0, s5-c-0139 wording0, s5-c-0151 wording0, s6-c-0006 wording0, s6-c-0022 wording0, s6-c-0035 wording0, s6-c-0037 wording1, s6-c-0039 wording0, s6-c-0039 wording1, s6-c-0043 wording1, s6-c-0044 wording1, s6-c-0054 wording0, s6-c-0055 wording0, s6-c-0055 wording2, s6-c-0068 wording1, s6-c-0073 wording0, s6-c-0074 wording0, s6-c-0074 wording2, s6-c-0083 wording2, s6-c-0085 wording2, s6-c-0088 wording0, s6-c-0091 wording2, s6-c-0094 wording0, s6-c-0094 wording1, s6-c-0112 wording2, s6-c-0117 wording2, s6-c-0118 wording2, s6-c-0128 wording2.


### Full model-only unregularized whitening — feasibility abort

A predeclared control would fit float64 covariance of the unchanged accepted
SSE outputs for21,745 ASCII lowercase whole tokenizer entries in token-id
order, then use ZCA W=U diag(lambda^-1/2) U^T on centered outputs and normalize.
All512 directions retained; no regularization, shrinkage, dimension selection,
corpus/case/gold fitting or scorer changes. This differs from the closed
mean/one-direction removal controls. The declared fitter must abort if an
eigenvalue is not positive finite, before parameters or retrieval are deployed.

It aborts:29 computed eigenvalues are nonpositive, minimum
-1.6991626583212781e-18, maximum0.007998159516177269. No output coordinate is
individually constant over the fitted vocabulary. These tiny negative values
are consistent with numerical error around near-null directions, not proof
that the mathematical covariance is indefinite or the accepted encoder wrong.
The full spectrum is retained. At the diagnostic machine-precision criterion
512*epsilon_f64*lambda_max=9.092854629872441e-16,445 eigenvalues exceed the
criterion; this is numerical rank under that criterion, not a selected
retrieval dimension. Even tiny positive directions make an unregularized
inverse square root unstable. No regularizer was added to rescue this control.

Source, protocol and spectrum remain in ignored53-whitening output. The fit
uses an isolated nalgebra0.35 native tool; root dependencies and product code
are untouched. No index, full dev/h2 run or performance gain/loss is claimed.
This abort is not counted among the52 completed scored configurations.
A regularized or pseudoinverse transform would be a different experiment, not
a result established here. The whitening paper reference and its limits remain
in Static-vector geometry references.


### Model-only OAS stabilized whitening — dev gain, held after flat h2

[Chen, Wiesel, Eldar and Hero, Shrinkage Algorithms for MMSE Covariance Estimation](https://arxiv.org/abs/0907.4698) motivates covariance shrinkage under distributional assumptions; it does not establish retrieval gains for static word vectors. [scikit-learn OAS documentation](https://scikit-learn.org/stable/modules/generated/sklearn.covariance.OAS.html) and its [implementation](https://github.com/scikit-learn/scikit-learn/blob/main/sklearn/covariance/_shrunk_covariance.py) document a large-dimension convention omitting the paper's2/p factors. This experiment uses that convention, not exact paper-formula parity or a claim that vocabulary vectors are Gaussian.

For n centered model-word outputs and p=512, covariance C uses denominator n.
Let s=trace(C*C), t=trace(C). Shrinkage rho=min(1,(s+t*t)/((n+1)*(s-t*t/p))),
using1 when the denominator is zero. Csh=(1-rho)C+rho*t/p*I;
W=Csh^-1/2 by float64 symmetric eigendecomposition. All512 directions remain.
Fit only21,745 ASCII lowercase whole vocabulary entries in token-id order,
encoded by unchanged accepted SSE. No corpus, queries, gold or author fields.
The unregularized control stays aborted; no epsilon was tuned against retrieval.

Computed rho0.046732728419519094, smallest Csh eigenvalue
0.00009119861354400612, max absolute W*Csh*W-I residual
2.9531932455029164e-14. Native fit took0.610s. Cast mean and row-major W
to float32 and compile them into the experimental binary. For each nonempty
accepted unit vector, serial-coordinate float32 W*(v-mean), then normalize once.
Empty remains zero. Same model, tokenization, lexical/POS rules, thresholds,
source addressing, card assembly and scorer. Fresh model-identified stores.
No runtime fitting/files or second model. Independent float64 full-equation
fixture and strict library clippy pass; fitted artifacts/source hashes retained.

Dev any411/439 first352/439 xs31/42 and25/42 empty35/51 section55/73.
Only case-level gold/cross-scope gain: s6-c-0035 original wording, gold-first.
No case-level losses in those four metrics; empty/section flat. All1,470 frozen
ordered pages replay identically. One frozen h2-all confirmation returns
any341/354 first297/354 xs57/62 and50/62 empty56/106 section239/271: all
six accepted totals flat. Hold, not adoption. No retuning, alternate shrinkage,
runner-up, product commit or hybrid victory claim. Prototype removed. Reused
h2 is benchmark confirmation, not a fresh independent holdout.

Diagnostic timings: dev fresh store26,232.687ms, resident median6.036ms/
p9516.185ms; h2 store28,538.097ms, median6.893ms/p9519.669ms. Not controlled
fresh-CLI timing; no packaging/speed adoption claim or scale/neutral run.

Dev changed pages: 17/1470. Case-level gold/cross-scope changes: {"gold_first_gain": ["s6-c-0035"]}.
Dev changed wording pages: s1b-c-0005 wording2, s1b-c-0018 wording1, s1b-c-0025 wording0, s1b-c-0041 wording1, s1b-c-0060 wording1, s2b-c-0060 wording0, s3b-c-0020 wording0, s3b-c-0048 wording0, s3b-c-0065 wording2, s4-c-0044 wording2, s4-c-0080 wording1, s4-c-0115 wording1, s5-c-0097 wording0, s6-c-0035 wording0, s6-c-0053 wording2, s6-c-0055 wording2, s6-c-0128 wording2.

H2 changed pages: 25/1380. Case-level gold/cross-scope changes: {"gold_first_gain": ["h2a3-c-0015"], "gold_first_loss": ["s7-c-0029"]}.
H2 changed wording pages: h2a1-c-0031 wording1, h2a1-c-0049 wording0, h2a1-c-0064 wording1, h2a1-c-0101 wording0, h2a1-c-0108 wording0, h2a1-c-0108 wording1, h2a2-c-0001 wording0, h2a2-c-0006 wording0, h2a2-c-0007 wording1, h2a2-c-0057 wording1, h2a3-c-0003 wording0, h2a3-c-0015 wording0, h2a3-c-0016 wording0, h2a3-c-0024 wording1, h2a3-c-0055 wording0, h2a3-c-0078 wording1, s7-c-0029 wording0, s7-c-0031 wording1, s7-c-0038 wording0, s7-c-0088 wording0, s7-c-0095 wording1, s7-c-0095 wording2, s7-c-0129 wording0, s7-c-0132 wording0, s7-c-0148 wording2.


### Matched hybrid distinct-record cut audit — no retention defect

Extend the native matched-page audit with distinct-record ranks after the
existing scope penalty. Reconstruct the full unique-record order independently
and assert its first five section ids equal the product's penalized cut.
All591 saved slice/wording card responses still match exactly. This uses the
previously rebuilt private matched stores, unchanged accepted retrieval.

Of19 external baseline-only wording/slice pairs having an admitted gold row,15 place every
gold record beyond five in both matcher record orders. Four have a gold record
in one matcher's top five but not on the final page: large-s1 s1b-c-0011/1
(rank4), large-s5 s5-c-0122/1 (rank3), small-s1 s1b-c-0009/2 (rank4), and
small-s1 s1b-c-0023/2 (rank3). These are overlapping benchmark slices, not
independent case-level score gains. The three original-query ordering misses
are large-s5 s5-c-0013/0 (raw12, distinct6), s5-c-0071/0 (raw20, distinct9),
and s5-c-0105/0 (raw46, distinct8). Large raw section ranks therefore overstate
distance from the record cut, but duplicates do not consume its five slots.

The code already deduplicates records before penalty sorting and truncation.
No duplicate-section retention defect or reason to repeat closed diversity/
interleave controls was found. This does not prove all selection mechanisms
optimal. No retrieval configuration, threshold change, h2 or score gain.
Native release assertions pass; diagnostic source and row/record evidence are
retained under ignored record-cut-audit output. Product instrumentation removed.

Complete admitted-gold ranks (slice:case/wording raw→distinct): large-s1:s1-c-0010/2 45→7, large-s1:s1b-c-0009/2 33→9, large-s1:s1b-c-0010/2 9→6, large-s1:s1b-c-0011/1 7→4, large-s1:s1b-c-0062/1 7→6, large-s5:s5-c-0001/2 34→12, large-s5:s5-c-0011/1 7→7, large-s5:s5-c-0013/0 12→6, large-s5:s5-c-0046/2 30→6, large-s5:s5-c-0048/1 57→21, large-s5:s5-c-0048/2 27→6, large-s5:s5-c-0071/0 20→9, large-s5:s5-c-0095/2 9→8, large-s5:s5-c-0105/0 46→8, large-s5:s5-c-0122/1 10→3, large-s5:s5-c-0135/1 26→10, large-s5:s5-c-0150/1 11→7, small-s1:s1b-c-0009/2 17→4, small-s1:s1b-c-0023/2 10→3.


### Unique original qualified-identifier suffix — dev gain, held after flat h2

Native original-body screen recognizes uppercase underscore symbols with at
least two alphabetic components. Existing exact vocabulary tokens take
precedence; require a unique longer name ending in _QUERY. Across1,470 dev
wordings, five parser errors are reported/skipped. Only two wordings have
source witnesses: s5-c-0051/0 and/1, RACE_CONDITION names
ERR_REQUIRE_ESM_RACE_CONDITION. Wording1 already finds gold. Native probes
replace only that atom and retain all companion query words; the original
error-code wording then retrieves gold s5-r-0179. No dropping error/code or
using gold in search. The screen's component minimum is2; the fixed retrieval
prototype uses3 to ensure discovery through the existing parts index. Both
observed requests satisfy that requirement; no threshold sweep.

Before ordinary agent processing, resolve only a bare whole uppercase
underscore atom with at least two alphabetic components of at least3 bytes.
Quotes, filters, punctuation-attached atoms and other words remain unchanged.
If absent from the exact vocabulary, intersect existing FTS parts postings for
its components, apply eligibility and query filters, then inspect original
block body tokens. Require exactly one longer full symbol under the same
grammar with complete _QUERY suffix; repeated occurrences/records of one name
are permitted, multiple names abstain. Substitute only the attested full name
and run the unchanged retrieval pipeline. This is symbolic reference
resolution, not independent-piece matching or semantic synonym expansion.
No schema/index, source-body, author-field, model, score or threshold changes;
no full-store name map. Candidate reads use existing indexed postings.

Dev any411/439 first352/439 xs31/42 and25/42 empty35/51 section55/73.
Sole case-level gold/cross-scope change: gold-first gain s5-c-0051. Two pages
change, wordings0/1 of that case. All1,470 frozen pages replay exactly.
One frozen h2-all confirmation returns any341/354 first297/354 xs57/62 and
50/62 empty56/106 section239/271, all accepted totals flat. Hold, not adoption.
No boundary tuning, runner-up, hybrid victory claim or product commit.
Prototype removed. Reused h2 is benchmark evidence, not an independent sample.

Corpus-free exact-token precedence, quoted literal, filter/punctuation boundary,
missing suffix and ambiguous full-name fixtures pass; strict library clippy
passes. Native screen/probe source and complete outputs remain ignored.
Diagnostic dev resident median8.095ms/p9524.395ms; h2 median6.940ms/p9519.361ms
and fresh store19,570.427ms. These are not controlled fresh-CLI comparisons.

Dev changed pages: 2/1470. Case-level gold/cross-scope changes: {"gold_first_gain": ["s5-c-0051"]}.
Dev changed wording pages: s5-c-0051 wording0, s5-c-0051 wording1.

H2 changed pages: 0/1380. Case-level gold/cross-scope changes: {}.
H2 changed wording pages: none.


### Distinct token-ID static pooling — not adopted

One fixed control keeps only the first occurrence of each tokenizer id in the
original first512 ids after the3584-scalar limit. Unknown ids remain, once;
deduplication never extends the input window. Existing serial float32 mean,
learned SSE transform and final normalization remain unchanged. Empty zero.
Apply symmetrically to every encoded text; lexical TF and Porter columns stay
untouched. This tests repetition in vectors, distinct from lexical query-word
deduplication, distinct lexical length, POS/stopword removal and magnitude
weighting. No repeat-cap sweep, fitted weights, second model, generated text,
threshold or scorer changes. Fresh private model-identified store.

Dev any411/439 first353/439 xs30/42 and24/42 empty35/51 section55/73.
Accepted411/351,31/25,35,55. First gains s1b-c-0008 and s6-c-0035;
cross-scope-any and first lose s5-c-0081. Rejected despite the gold-first
gains. No h2, repeat-weight tuning or combination. Prototype removed.
Independent float64 mean/transform fixture, repetition invariance and
truncation-boundary tests pass; strict library clippy passes. Diagnostic
fresh-store21,269.337ms; resident median6.888ms/p9517.894ms/max58.033ms.
No controlled CLI-speed or hybrid-win claim.

Changed pages: 77/1470. Case-level gold/cross-scope changes: {"gold_first_gain": ["s1b-c-0008", "s6-c-0035"], "xs_any_loss": ["s5-c-0081"], "xs_first_loss": ["s5-c-0081"]}.
Changed wording pages: s1b-c-0008 wording0, s1b-c-0008 wording2, s1b-c-0015 wording2, s1b-c-0018 wording1, s1b-c-0027 wording1, s1b-c-0047 wording1, s1b-c-0048 wording0, s1b-c-0049 wording0, s1b-c-0060 wording0, s1b-c-0062 wording2, s2b-c-0026 wording0, s2b-c-0027 wording0, s2b-c-0027 wording1, s2b-c-0027 wording2, s2b-c-0028 wording2, s2b-c-0041 wording0, s2b-c-0051 wording0, s2b-c-0060 wording0, s3a-c-0009 wording1, s3a-c-0009 wording2, s3a-c-0022 wording1, s3b-c-0010 wording0, s3b-c-0020 wording0, s3b-c-0021 wording0, s3b-c-0025 wording0, s3b-c-0028 wording0, s3b-c-0031 wording0, s3b-c-0035 wording0, s3b-c-0035 wording2, s3b-c-0046 wording0, s3b-c-0048 wording0, s3b-c-0050 wording0, s3b-c-0054 wording0, s3b-c-0065 wording2, s4-c-0013 wording2, s4-c-0015 wording0, s4-c-0025 wording0, s4-c-0025 wording1, s4-c-0044 wording0, s4-c-0044 wording2, s4-c-0067 wording2, s4-c-0084 wording0, s4-c-0089 wording0, s4-c-0089 wording2, s4-c-0091 wording0, s4-c-0092 wording2, s4-c-0115 wording0, s4-c-0123 wording1, s5-c-0001 wording0, s5-c-0003 wording0, s5-c-0005 wording0, s5-c-0040 wording1, s5-c-0043 wording0, s5-c-0046 wording0, s5-c-0051 wording0, s5-c-0076 wording0, s5-c-0081 wording0, s5-c-0087 wording1, s5-c-0091 wording0, s5-c-0095 wording0, s5-c-0097 wording0, s5-c-0102 wording0, s5-c-0124 wording0, s5-c-0139 wording0, s6-c-0022 wording0, s6-c-0030 wording1, s6-c-0035 wording0, s6-c-0044 wording1, s6-c-0053 wording2, s6-c-0054 wording0, s6-c-0055 wording2, s6-c-0085 wording2, s6-c-0091 wording2, s6-c-0112 wording2, s6-c-0117 wording2, s6-c-0118 wording2, s6-c-0128 wording2.


### One structural unit per index row — not adopted

One fixed ingestion control calls the existing original-byte block cutter with
packing limit0, retaining each existing blank/list/fence-delimited unit as its
own index row. Section parsing, byte spans, complete fences and oversized
units remain unchanged; no overlapping windows or text synthesis. The separate
heading column still belongs to the first block only. Regenerate all derived
rows, FTS statistics, POS and vectors in a fresh private store; collection
statistics necessarily change. This is an evidence-granularity control, not
an isolated scoring ablation. Distinct from closed cross-record bridges,
adjacent-block support and table-row cuts; none of those was rerun. Same
model, query grammar, thresholds, tiers, page procedure and scorer.

q4 index rows rise from6,096 to24,109 for the same1,376 records. Dev
any407/439 first331/439 xs32/42 and24/42 empty37/51 section44/73, against
accepted411/351,31/25,35,55. Gold-any/first, cross-scope-first and section
fall. Cross-scope-any and no-gold gains do not qualify the control. Rejected
and removed; no packing-limit sweep, h2 or adoption. Original-byte section/
ancestry/fence/oversized-unit/UTF-8 cutter tests and strict library clippy pass.
Diagnostic fresh store18,996.627ms, resident median8.677ms/p9524.602ms/
max66.177ms; not a controlled CLI comparison. First-page exports attribute
gold/cross-scope changes only; empty and section changes are aggregate native
all-call scores, not per-case claims inferred from those exports.

Changed pages: 1320/1470. Case-level gold/cross-scope changes: {"gold_any_gain": ["s2b-c-0003", "s3a-c-0034", "s3b-c-0002", "s3b-c-0066", "s6-c-0074"], "gold_any_loss": ["s1-c-0016", "s1b-c-0041", "s3b-c-0008", "s3b-c-0057", "s4-c-0002", "s4-c-0005", "s5-c-0112", "s6-c-0102", "s6-c-0105"], "gold_first_gain": ["s2b-c-0003", "s3a-c-0027", "s3b-c-0002", "s3b-c-0066", "s4-c-0076"], "gold_first_loss": ["s1-c-0016", "s1b-c-0041", "s2a-c-0007", "s3a-c-0005", "s3a-c-0006", "s3a-c-0013", "s3a-c-0020", "s3b-c-0008", "s4-c-0002", "s4-c-0003", "s4-c-0005", "s4-c-0007", "s4-c-0067", "s5-c-0012", "s5-c-0017", "s5-c-0026", "s5-c-0027", "s5-c-0028", "s5-c-0067", "s5-c-0112", "s6-c-0015", "s6-c-0020", "s6-c-0044", "s6-c-0073", "s6-c-0105"], "xs_any_gain": ["s2b-c-0076", "s3a-c-0028"], "xs_any_loss": ["s4-c-0031"], "xs_first_loss": ["s5-c-0082"]}.
Changed wording pages (case: wording indices): s1-c-0001:1/2, s1-c-0002:0/1/2, s1-c-0003:1/2, s1-c-0004:0/1/2, s1-c-0006:0/1/2, s1-c-0008:0/1/2, s1-c-0009:0/1/2, s1-c-0010:0/1/2, s1-c-0011:0/2, s1-c-0012:0/1/2, s1-c-0016:0/1/2, s1b-c-0001:0/1/2, s1b-c-0002:0/1/2, s1b-c-0004:0/2, s1b-c-0005:0/1/2, s1b-c-0006:0/1/2, s1b-c-0007:0/1/2, s1b-c-0008:1/2, s1b-c-0009:1/2, s1b-c-0010:2, s1b-c-0011:0/1, s1b-c-0012:0/1/2, s1b-c-0013:0/1/2, s1b-c-0015:1/2, s1b-c-0017:0/1/2, s1b-c-0018:1/2, s1b-c-0020:0/1/2, s1b-c-0022:1/2, s1b-c-0023:0/1/2, s1b-c-0024:0/1/2, s1b-c-0025:0/1/2, s1b-c-0026:0/2, s1b-c-0027:0/1, s1b-c-0028:0/1/2, s1b-c-0029:0/1, s1b-c-0031:0/1/2, s1b-c-0033:0/1/2, s1b-c-0034:0/1/2, s1b-c-0035:2, s1b-c-0036:0/1, s1b-c-0037:0/1/2, s1b-c-0038:0/1/2, s1b-c-0040:1, s1b-c-0041:0/1/2, s1b-c-0043:0/1/2, s1b-c-0044:0/1/2, s1b-c-0045:0/1/2, s1b-c-0046:0/1/2, s1b-c-0047:0/2, s1b-c-0048:0/1/2, s1b-c-0049:0/1/2, s1b-c-0050:1, s1b-c-0052:0/1/2, s1b-c-0054:0/1, s1b-c-0055:0/1/2, s1b-c-0059:0/2, s1b-c-0060:0/2, s1b-c-0061:0/1/2, s1b-c-0062:0/1/2, s1b-c-0065:0/1/2, s1b-c-0066:0/1/2, s1b-c-0067:0/1/2, s2a-c-0001:0/1, s2a-c-0002:0/1/2, s2a-c-0004:0/2, s2a-c-0005:0, s2a-c-0007:0/1/2, s2a-c-0008:0/1/2, s2a-c-0009:0/1/2, s2b-c-0001:0/1/2, s2b-c-0002:0/1/2, s2b-c-0003:0/1, s2b-c-0004:0/1/2, s2b-c-0006:0/1/2, s2b-c-0008:0/1/2, s2b-c-0009:0/1/2, s2b-c-0010:0/1/2, s2b-c-0011:0/1/2, s2b-c-0013:0/1/2, s2b-c-0014:2, s2b-c-0015:0/1/2, s2b-c-0016:0/1/2, s2b-c-0017:0/1, s2b-c-0019:0/1/2, s2b-c-0020:1/2, s2b-c-0023:0/1/2, s2b-c-0025:0/2, s2b-c-0026:0/2, s2b-c-0027:0/2, s2b-c-0028:1/2, s2b-c-0030:0/1/2, s2b-c-0031:0/2, s2b-c-0032:0/1, s2b-c-0033:0/1/2, s2b-c-0034:0/1/2, s2b-c-0035:0/1/2, s2b-c-0038:0/1/2, s2b-c-0039:0/1/2, s2b-c-0041:0/1/2, s2b-c-0042:0/1/2, s2b-c-0043:0/1/2, s2b-c-0044:0/1/2, s2b-c-0045:0/1, s2b-c-0046:0/1/2, s2b-c-0047:0/2, s2b-c-0048:0/1/2, s2b-c-0049:0/1/2, s2b-c-0050:0/1/2, s2b-c-0051:0/1/2, s2b-c-0052:0/1/2, s2b-c-0053:0/1/2, s2b-c-0055:2, s2b-c-0056:0/1/2, s2b-c-0058:0/1/2, s2b-c-0059:0/1/2, s2b-c-0060:0/1/2, s2b-c-0061:0/1/2, s2b-c-0067:0/1/2, s2b-c-0068:0/1/2, s2b-c-0069:0/1/2, s2b-c-0073:0/1/2, s2b-c-0074:0/1/2, s2b-c-0076:0/1/2, s2b-c-0078:1/2, s2b-c-0079:0/2, s2b-c-0080:0/1/2, s3a-c-0001:0/1/2, s3a-c-0002:0/2, s3a-c-0003:0/1/2, s3a-c-0004:0/1/2, s3a-c-0005:0/1/2, s3a-c-0006:0/1/2, s3a-c-0008:0/1/2, s3a-c-0009:1, s3a-c-0010:0/1/2, s3a-c-0012:0/1/2, s3a-c-0013:0/1/2, s3a-c-0014:0/1/2, s3a-c-0015:0/1/2, s3a-c-0016:0/1/2, s3a-c-0017:0/1/2, s3a-c-0018:1/2, s3a-c-0019:0/1/2, s3a-c-0020:0/1/2, s3a-c-0021:0/1/2, s3a-c-0022:0/1/2, s3a-c-0023:0/1/2, s3a-c-0026:0/1/2, s3a-c-0027:0/1/2, s3a-c-0028:1/2, s3a-c-0031:0/1/2, s3a-c-0033:0/1/2, s3a-c-0034:1/2, s3a-c-0035:0/1/2, s3a-c-0039:0/1/2, s3a-c-0040:0/1/2, s3a-c-0041:0/1/2, s3a-c-0042:0/1/2, s3a-c-0043:0/1/2, s3a-c-0044:0/1/2, s3a-c-0047:0/1/2, s3a-c-0048:0/1/2, s3a-c-0049:0/1/2, s3a-c-0050:0/1/2, s3a-c-0052:0/1/2, s3a-c-0053:0/1/2, s3a-c-0054:0/1/2, s3a-c-0056:0/1/2, s3a-c-0057:0/1/2, s3a-c-0058:0/1/2, s3a-c-0062:0/1/2, s3a-c-0063:1/2, s3b-c-0002:0/1/2, s3b-c-0007:0/1/2, s3b-c-0008:0/1/2, s3b-c-0009:0/1/2, s3b-c-0010:0/1/2, s3b-c-0013:0/1/2, s3b-c-0014:0/1/2, s3b-c-0015:2, s3b-c-0016:1/2, s3b-c-0018:0/1/2, s3b-c-0019:0/1/2, s3b-c-0020:0/1/2, s3b-c-0021:0/1/2, s3b-c-0025:0/1/2, s3b-c-0026:1/2, s3b-c-0027:1/2, s3b-c-0028:1/2, s3b-c-0031:0/1/2, s3b-c-0033:0/1/2, s3b-c-0034:1/2, s3b-c-0035:1/2, s3b-c-0037:0/1/2, s3b-c-0038:0/1/2, s3b-c-0039:0/1/2, s3b-c-0040:0/1/2, s3b-c-0041:0/2, s3b-c-0042:0/1/2, s3b-c-0045:0/1/2, s3b-c-0046:0/1/2, s3b-c-0047:0/1/2, s3b-c-0048:1/2, s3b-c-0049:0/1/2, s3b-c-0050:0/1/2, s3b-c-0051:0/1/2, s3b-c-0052:1/2, s3b-c-0053:0/1/2, s3b-c-0054:1/2, s3b-c-0055:0/1/2, s3b-c-0057:0/1/2, s3b-c-0058:1/2, s3b-c-0060:0/1/2, s3b-c-0061:0/1/2, s3b-c-0063:0/1/2, s3b-c-0064:0/2, s3b-c-0065:0/1/2, s3b-c-0066:0/1/2, s3b-c-0067:0/2, s4-c-0001:0/1/2, s4-c-0002:0/1/2, s4-c-0003:0/1, s4-c-0004:0/1/2, s4-c-0005:0/1, s4-c-0006:1/2, s4-c-0007:0/1/2, s4-c-0008:0/1/2, s4-c-0009:0/1/2, s4-c-0010:1, s4-c-0011:0/1/2, s4-c-0012:0/1/2, s4-c-0013:0/1/2, s4-c-0014:0/1/2, s4-c-0015:0/1/2, s4-c-0018:0/1/2, s4-c-0019:0/1/2, s4-c-0020:1/2, s4-c-0021:0/1, s4-c-0023:0/1/2, s4-c-0024:0/1/2, s4-c-0025:2, s4-c-0026:0/1, s4-c-0027:0/1/2, s4-c-0028:0/1/2, s4-c-0029:0/1/2, s4-c-0031:0/1/2, s4-c-0033:0/1/2, s4-c-0034:0/1/2, s4-c-0035:0/1/2, s4-c-0036:0/1/2, s4-c-0037:0/1/2, s4-c-0040:0/1/2, s4-c-0041:0/1/2, s4-c-0043:0/2, s4-c-0044:2, s4-c-0045:0/1/2, s4-c-0046:0/1/2, s4-c-0047:0/1/2, s4-c-0048:0/1/2, s4-c-0049:0/1/2, s4-c-0051:0/1/2, s4-c-0052:0/2, s4-c-0053:0/1/2, s4-c-0054:0/1/2, s4-c-0055:0/1/2, s4-c-0056:0/1/2, s4-c-0057:0/1/2, s4-c-0058:0/1/2, s4-c-0059:0/1/2, s4-c-0060:0/1/2, s4-c-0062:0/1/2, s4-c-0063:0/1/2, s4-c-0064:0/1/2, s4-c-0065:1/2, s4-c-0066:0/1/2, s4-c-0067:0/1/2, s4-c-0069:0/1/2, s4-c-0071:0/1/2, s4-c-0072:0/1/2, s4-c-0073:0/1/2, s4-c-0074:1/2, s4-c-0075:0/1/2, s4-c-0076:0/1/2, s4-c-0077:0/1/2, s4-c-0078:0/1/2, s4-c-0079:0/1/2, s4-c-0080:0/1/2, s4-c-0082:0/1/2, s4-c-0083:0/1/2, s4-c-0084:0/2, s4-c-0086:0/1/2, s4-c-0088:0/1/2, s4-c-0089:1/2, s4-c-0091:0/1/2, s4-c-0092:0/1/2, s4-c-0093:0/1/2, s4-c-0094:0/1/2, s4-c-0095:0/1/2, s4-c-0097:1/2, s4-c-0098:0/1/2, s4-c-0099:0/1/2, s4-c-0100:1/2, s4-c-0102:0/1/2, s4-c-0103:0/1, s4-c-0104:0/1/2, s4-c-0105:0/1/2, s4-c-0107:0/1/2, s4-c-0109:0/1/2, s4-c-0110:0/1/2, s4-c-0112:0/1/2, s4-c-0113:1/2, s4-c-0115:0/1/2, s4-c-0116:0/1/2, s4-c-0117:0/1/2, s4-c-0118:0/1/2, s4-c-0119:0/1/2, s4-c-0120:0/1/2, s4-c-0123:0/2, s4-c-0124:0/1/2, s4-c-0125:0/1/2, s4-c-0126:0/1/2, s4-c-0129:0/1/2, s4-c-0131:0/1/2, s4-c-0132:0/1, s4-c-0133:0/1/2, s5-c-0001:0/1/2, s5-c-0002:0/1/2, s5-c-0003:0/1/2, s5-c-0004:0/1/2, s5-c-0005:0, s5-c-0011:0/1/2, s5-c-0012:0/1/2, s5-c-0013:0/1, s5-c-0017:0/1/2, s5-c-0018:0/1/2, s5-c-0019:0/1/2, s5-c-0026:0/1/2, s5-c-0027:0/1/2, s5-c-0028:0/1/2, s5-c-0031:0/1/2, s5-c-0032:0/1/2, s5-c-0033:0/1/2, s5-c-0036:0/1/2, s5-c-0040:0/1/2, s5-c-0041:0/1, s5-c-0043:0/1/2, s5-c-0046:0/1/2, s5-c-0048:0/1/2, s5-c-0050:0/1/2, s5-c-0051:1/2, s5-c-0055:0/1/2, s5-c-0058:0/1/2, s5-c-0059:0/1, s5-c-0062:0/1/2, s5-c-0067:0/1/2, s5-c-0068:1/2, s5-c-0071:0/1/2, s5-c-0073:0/1/2, s5-c-0074:2, s5-c-0076:0/2, s5-c-0078:0/1/2, s5-c-0079:0/1/2, s5-c-0080:0/2, s5-c-0081:0/1/2, s5-c-0082:0/1/2, s5-c-0083:0/1/2, s5-c-0084:1/2, s5-c-0085:0/1/2, s5-c-0087:0/1/2, s5-c-0090:1/2, s5-c-0091:0/1/2, s5-c-0095:0/1/2, s5-c-0097:0/1/2, s5-c-0098:0/1/2, s5-c-0099:0/1/2, s5-c-0102:0/1/2, s5-c-0103:1/2, s5-c-0104:0/1, s5-c-0105:0/1/2, s5-c-0106:0/1/2, s5-c-0107:0/2, s5-c-0109:0/1/2, s5-c-0110:0/1/2, s5-c-0112:0/1/2, s5-c-0114:0/1/2, s5-c-0116:0/1/2, s5-c-0117:1/2, s5-c-0118:0/1/2, s5-c-0121:0/1, s5-c-0122:0/1/2, s5-c-0124:0/1/2, s5-c-0126:0/1, s5-c-0127:0/1/2, s5-c-0128:0, s5-c-0129:1/2, s5-c-0130:0/1/2, s5-c-0132:1/2, s5-c-0133:0/1/2, s5-c-0134:0/1/2, s5-c-0135:1/2, s5-c-0137:0/1/2, s5-c-0138:0/1/2, s5-c-0139:1/2, s5-c-0141:0/1/2, s5-c-0142:0/1/2, s5-c-0143:0/1/2, s5-c-0144:0/1, s5-c-0147:0/1/2, s5-c-0148:0/1/2, s5-c-0149:0/1/2, s5-c-0150:0/1/2, s5-c-0151:0/1/2, s5-c-0152:0/1/2, s5-c-0153:1/2, s5-c-0154:0/1/2, s6-c-0001:0/1/2, s6-c-0003:0/1/2, s6-c-0004:0/1/2, s6-c-0006:2, s6-c-0009:0/1/2, s6-c-0010:0/1/2, s6-c-0015:0/1/2, s6-c-0016:0/1/2, s6-c-0017:0/1/2, s6-c-0018:0/2, s6-c-0020:0/1/2, s6-c-0021:0/1/2, s6-c-0022:1/2, s6-c-0024:0/1/2, s6-c-0026:0/1/2, s6-c-0028:0/1/2, s6-c-0030:0/1, s6-c-0031:0/1/2, s6-c-0034:0/1/2, s6-c-0035:0/1/2, s6-c-0036:0/2, s6-c-0037:0/2, s6-c-0039:2, s6-c-0040:1/2, s6-c-0041:1/2, s6-c-0042:0/1/2, s6-c-0043:0, s6-c-0044:0/1/2, s6-c-0049:0/1/2, s6-c-0051:0/1/2, s6-c-0053:0/1/2, s6-c-0054:1/2, s6-c-0055:1/2, s6-c-0058:0/1/2, s6-c-0060:0/1/2, s6-c-0061:0/1/2, s6-c-0063:0/1/2, s6-c-0064:0/2, s6-c-0065:0/1/2, s6-c-0066:0/1/2, s6-c-0068:0/1/2, s6-c-0071:0/1/2, s6-c-0073:0/2, s6-c-0074:1, s6-c-0075:0/1/2, s6-c-0077:1/2, s6-c-0079:0/1/2, s6-c-0083:0/2, s6-c-0084:0/1/2, s6-c-0085:0/1/2, s6-c-0086:0/1/2, s6-c-0088:0/1/2, s6-c-0089:0/1/2, s6-c-0091:0/1/2, s6-c-0093:1/2, s6-c-0094:0/1/2, s6-c-0096:0/1/2, s6-c-0097:0/1/2, s6-c-0099:0/1/2, s6-c-0100:0/1/2, s6-c-0102:0/1/2, s6-c-0105:0/1/2, s6-c-0107:0/1/2, s6-c-0109:0/1/2, s6-c-0110:0/1, s6-c-0112:0/1, s6-c-0115:0/1/2, s6-c-0117:0/1/2, s6-c-0118:0/1/2, s6-c-0119:2, s6-c-0121:0/1/2, s6-c-0124:0/1/2, s6-c-0125:0/1/2, s6-c-0128:0/1.


### Punctuation-only tokenizer-piece omission — not adopted

One fixed vector-only control applies the original3584-scalar and first512-id
limits, then omits ids whose tokenizer piece has no Unicode letter or number.
Unknown [UNK] and missing token mappings remain. Preserve stopwords, repeated
ids, wordpiece fragments and input order; no retokenization or extension after
filtering. Existing SSE mean/learned transform/normalization and512 dimensions;
no retained ids returns zero. Lexical bytes/TF, Porter, POS, query grammar,
gates, source addressing, page assembly and scorer stay unchanged. Distinct
from closed content-word removal (which retained punctuation), table-delimiter
preprocessing and repeated-token deduplication. Fresh private model-identified
store; one token-class rule, no class/threshold sweep.

Dev stays any411/439 first351/439 xs31/42 and25/42 empty35/51 section55/73.
No qualifying rise; no h2, adoption or hybrid claim. Prototype removed.
Independent float64 mean/transform equation, punctuation-equivalence, all-
punctuation zero, unknown preservation and original-window boundary fixtures
pass; strict library clippy passes. Diagnostic fresh store17,996.018ms; resident
median5.969ms/p9515.854ms/max50.486ms. Not a controlled CLI speed comparison.

Changed pages: 18/1470. Case-level gold/cross-scope changes: {}.
Changed wording pages: s1b-c-0025 wording0, s1b-c-0041 wording1, s1b-c-0060 wording1, s2b-c-0032 wording2, s2b-c-0038 wording1, s2b-c-0051 wording0, s3b-c-0020 wording0, s3b-c-0021 wording0, s3b-c-0035 wording2, s3b-c-0046 wording0, s3b-c-0054 wording0, s4-c-0015 wording0, s4-c-0115 wording0, s5-c-0003 wording0, s5-c-0005 wording0, s5-c-0076 wording0, s6-c-0044 wording1, s6-c-0112 wording2.

### MiniLM-L3 actual CLI feasibility follow-up — not advanced

This fills the explicitly missing end-to-end measurement in the earlier native
MiniLM screen; it is not a new retrieval configuration or a reopening of a
rejected quality result. Same pinned MiniLM-L3 revision, Candle 0.11.1 CPU
Accelerate implementation, original-text input, mask-mean384 padded to512, and
128-token limit. An isolated product prototype embeds only the replacement
model and uses a fresh model-identified q4 store. Search rules, thresholds,
POS and card assembly are unchanged. Build assets are already local; no new
download. The native candidate executable is102,047,632 bytes versus93,979,760
for accepted. This is a local Cargo binary test, not a Nix package release.

The predeclared screen requires both fresh CLI and resident search median/p95
within25% of accepted. Twelve dev original queries, selected by corpus order
without gold inspection, run twice. Fresh CLI binaries alternate order between
rounds; RAYON_NUM_THREADS and VECLIB_MAXIMUM_THREADS are1 for both. CLI median/
p95 is49.795271/107.064166ms accepted versus56.9696455/112.388667ms candidate:
this sample passes. Filesystem caches are not flushed, so it is not a cold-disk
claim. Separate resident processes use the same wordings and their own correct
stores. Initial resident median/p95 is7.844229/63.830958ms accepted versus
7.557458/84.874917ms candidate, failing the p95 limit.

Because this is a small timing sample, three additional paired resident
processes were measured, alternating order, with no model/code/configuration
change. These follow-up repetitions were added after the first measurement;
they are not a predeclared replication count. All data, including the first
sample, is retained. Repetition p95 pairs are52.358/50.334958,
49.182417/48.6315 and48.901667/48.587291ms. Across all96 observations per model,
resident median/p95 is7.7328335/52.358ms accepted versus7.748979/65.662625ms
candidate. P95 is25.411% higher, narrowly outside25%. This is a noisy borderline
non-pass, not proof of an architectural speed impossibility. Do not discard the
first samples, round it into a pass, or claim retrieval improvement. Different
model outputs can change fallback work; this measures the resulting invocation,
not a pure kernel attribution.

No full candidate dev score, h2, scale or hybrid run. The store-building command
selected no cases and printed zero denominators; those lines are not an eval.
The prototype and temporary root dependencies were removed. Accepted product
reproduces any411/439 first351/439 xs31/42 and25/42 empty35/51 section55/73,
with all1470 ordered pages byte-identical. LLP reference check passes. The
earlier model card and Candle references remain the provenance for this same
implementation. No quality conclusion or new scored-configuration count.


### Repeated morphological query obligations — population screen only

A native screen of all1470 dev wordings groups non-hard content terms by the
existing Porter function. Six wordings have more than one term per stem:
s1b-c-0036/0 AGENTS/agent; s3a-c-0054/0 Update/updates; s3b-c-0058/2
package/packages; s4-c-0052/1 generated/generator; s5-c-0141/2 window/Windows;
s6-c-0099/0 PDF/pdf. The first, second, third, fourth and sixth already show
gold in the affected wording's first five cards. The remaining case distinguishes
a startup window from Windows, so stem identity is not redundant meaning.
Generated/generator similarly distinguishes output from tool. No automatic
obligation collapse, scoring run or retrieval gain is justified by this screen.
This is distinct from the closed exact query-word score deduplication control.

### Tokenizer-word balanced static pooling — not adopted

One fixed configuration gives each consecutive tokenizer word_id group equal
mass. Retain the original3584-scalar/512-token bounds, all unknowns, punctuation
and repetitions. Average raw vectors within each retained group, average groups
equally, then apply the same SSE learned transform and normalization. A missing
word_id is its own group; a cut final group uses only retained pieces. Empty
stays zero. Same equation on every vector path, fresh model-identified store.
No changes to original bodies, lexical indexing, POS, gates, tiers or scoring.
This tests the earlier explicitly unimplemented broad per-word pooling question;
it differs from line pooling, token deduplication and punctuation omission.

Dev any412/439 first352/439 xs30/42 and24/42 empty35/51 section55/73.
Gold improves, but both cross-scope metrics fall against accepted31/42 and25/42.
Rejected without h2 or tuning; no hybrid or latency improvement claim.
Independent float64 grouped-mean/transform fixture, single-piece arithmetic,
repeated-word and512-token window checks pass; strict library clippy passes.
Prototype removed. Accepted-page restoration is checked separately.

- gold_any_gain: s5-c-0090.
- gold_first_gain: s5-c-0090, s6-c-0035.
- gold_first_loss: s6-c-0073.
- xs_any_loss: s5-c-0081.
- xs_first_loss: s5-c-0081.

Changed ordered pages: 62/1470. Wording ledger: s1-c-0012/2, s1b-c-0004/0, s1b-c-0005/0, s1b-c-0005/1, s1b-c-0007/0, s1b-c-0015/0, s1b-c-0018/1, s1b-c-0025/0, s1b-c-0031/0, s1b-c-0034/2, s1b-c-0041/0, s1b-c-0044/0, s1b-c-0045/2, s1b-c-0059/0, s1b-c-0060/0, s1b-c-0065/1, s2a-c-0008/0, s2b-c-0027/2, s2b-c-0028/0, s2b-c-0031/1, s2b-c-0032/2, s2b-c-0038/1, s2b-c-0041/0, s2b-c-0060/0, s3a-c-0009/0, s3a-c-0022/1, s3a-c-0027/1, s3b-c-0033/1, s3b-c-0034/0, s3b-c-0035/2, s3b-c-0048/0, s3b-c-0065/2, s4-c-0015/0, s4-c-0025/1, s4-c-0067/2, s4-c-0080/1, s4-c-0084/0, s4-c-0088/0, s4-c-0089/2, s5-c-0005/0, s5-c-0040/1, s5-c-0076/0, s5-c-0081/0, s5-c-0090/0, s5-c-0090/2, s5-c-0095/0, s5-c-0102/0, s5-c-0124/0, s5-c-0139/0, s5-c-0151/0, s6-c-0022/0, s6-c-0035/0, s6-c-0044/1, s6-c-0053/2, s6-c-0055/2, s6-c-0073/0, s6-c-0074/2, s6-c-0091/2, s6-c-0112/2, s6-c-0117/2, s6-c-0118/2, s6-c-0128/2.

Restoration reproduced all six accepted dev counts and all1470 ordered pages
byte-for-byte; LLP reference check passes.


### Fixed word-balanced pooling and OAS interaction — not adopted

One joint configuration composes the exact tokenizer-word mean from the previous
trial with the exact model-only OAS mean/matrix and serial-float32 transform
already measured separately. No refitting, coefficient sweep, dimension change,
corpus/query/gold calibration, additional model or selective per-query switch.
The prior components remain closed and unadopted; this measures their interaction,
not a new claim for either component. The OAS whole-vocabulary single-word fit
is compatible with word-balanced pooling because one tokenizer group has the
same raw mean. Fresh model-identified q4 store; original text, lexical/POS,
gates, tiers, eligibility and scorer remain unchanged.

Dev any412/439 first351/439 xs30/42 and24/42 empty35/51 section55/73.
Both cross-scope metrics fall versus accepted31/42 and25/42; reject without
h2, retuning or hybrid claim. The original-query gold gain from word balance is
also lost in aggregate: the joint result loses s5-c-0098 first versus the
word-balanced-only control. This does not demonstrate useful retrieval synergy.
The independent float64 composed equation and input-window/zero/repeated-word
fixtures pass; strict library clippy passes. Prototype removed. Store26,517.920ms;
resident median6.030ms/p9515.981ms is diagnostic, not a matched speed claim.

- gold_any_gain: s5-c-0090.
- gold_first_gain: s5-c-0090, s6-c-0035.
- gold_first_loss: s5-c-0098, s6-c-0073.
- xs_any_loss: s5-c-0081.
- xs_first_loss: s5-c-0081.

Changed ordered pages versus accepted: 66/1470; versus word-balanced-only: 20/1470.
Wording ledger versus accepted: s1-c-0012/2, s1b-c-0004/0, s1b-c-0005/0, s1b-c-0005/1, s1b-c-0007/0, s1b-c-0015/0, s1b-c-0018/1, s1b-c-0025/0, s1b-c-0031/0, s1b-c-0034/2, s1b-c-0044/0, s1b-c-0059/0, s1b-c-0060/0, s1b-c-0065/1, s2a-c-0008/0, s2b-c-0027/2, s2b-c-0028/0, s2b-c-0031/1, s2b-c-0032/2, s2b-c-0038/1, s2b-c-0041/0, s2b-c-0060/0, s3a-c-0009/0, s3a-c-0022/1, s3b-c-0020/0, s3b-c-0033/1, s3b-c-0034/0, s3b-c-0035/2, s3b-c-0048/0, s3b-c-0065/2, s4-c-0015/0, s4-c-0025/1, s4-c-0044/2, s4-c-0067/2, s4-c-0080/1, s4-c-0084/0, s4-c-0088/0, s4-c-0089/2, s4-c-0091/0, s4-c-0115/1, s5-c-0005/0, s5-c-0040/1, s5-c-0076/0, s5-c-0081/0, s5-c-0090/0, s5-c-0090/2, s5-c-0095/0, s5-c-0098/0, s5-c-0124/0, s5-c-0132/0, s5-c-0139/0, s5-c-0151/0, s6-c-0022/0, s6-c-0030/1, s6-c-0035/0, s6-c-0044/1, s6-c-0053/2, s6-c-0055/2, s6-c-0073/0, s6-c-0074/2, s6-c-0091/1, s6-c-0091/2, s6-c-0112/2, s6-c-0117/2, s6-c-0118/2, s6-c-0128/2.

Accepted restoration reproduces all six dev counts and all1470 ordered pages
byte-for-byte; LLP reference check passes.


### Full tokenizer sequence inside bounded original text — not adopted

One fixed configuration removes only the512-token cap after the original
3584-Unicode-scalar cut. All resulting ids, including unknowns, punctuation and
repetitions, enter the same serial-float32 mean, SSE learned transform and
normalization. The static lookup/mean has no positional attention matrix
requiring512 tokens. This is an alternative contract, not a claimed bug fix.
No larger character window, extra model, lexical/POS/section-cut/gate/card or
scorer change. Fresh model-identified store; no cap sweep.

Forty-four of6096 stored block vectors differ (read-only byte comparison).
Nevertheless all1470 first-page exports are byte-identical, with dev any411/439
first351/439 xs31/42 and25/42 empty35/51 section55/73. No qualifying rise;
no h2, adoption or hybrid claim. This tested token cap is not hiding a scored
dev recovery. It does not establish that arbitrary longer character windows or
all-call output would be equivalent.

Independent mean/transform equation and long-input tests pass: more than512
tokens retained, exact equivalence to the3584-scalar prefix, and1792 single-letter
tokens retained inside that bound. Strict library clippy passes after replacing
iter/copied/collect with equivalent to_vec; the scored executable predates that
allocation-only cleanup. Source versions retained. Store18,472.814ms; resident
median5.990ms/p9515.818ms is diagnostic, not a matched latency claim.
Prototype removed; no case-level gold/cross-scope changes or moved pages.

Accepted restoration reproduces all six dev counts and all1470 first-page
exports byte-for-byte; LLP reference check passes.


### Mean of unit transformed-token vectors — not adopted

One fixed configuration maps each retained original token row through the
existing beta*tanh(alpha*row+bias), normalizes that transformed token to unit
length, averages those directions in token order, then normalizes the mean.
Serial float32 reductions,1e-9 denominator clamps, original3584-scalar/512-token
window, unknowns, punctuation and repeats retained. Empty remains zero. This
differs from the closed unnormalized token-transform mean and raw-norm weighting:
it removes transformed radial magnitude before pooling. No coefficient sweep,
fitted parameters, added model, lexical/POS/gate/card or scorer changes.
Fresh model-identified private store.

Dev any411/439 first348/439 xs31/42 and25/42 empty35/51 section55/73.
Gold-first falls three overall; reject with no h2, tuning or hybrid claim.
Independent float64 composed equation covers empty, singleton, repeated and
fragmented text;512-token boundary assertion and strict library clippy pass.
Store19,025.003ms; resident median5.957ms/p9515.471ms is diagnostic, not a
matched speed claim. Prototype removed.

- gold_first_gain: s4-c-0020, s6-c-0035.
- gold_first_loss: s1b-c-0018, s3b-c-0015, s5-c-0027, s5-c-0067, s6-c-0073.

Changed ordered pages: 102/1470. Wording ledger: s1b-c-0004/0, s1b-c-0005/0, s1b-c-0005/1, s1b-c-0007/0, s1b-c-0008/0, s1b-c-0015/0, s1b-c-0015/2, s1b-c-0018/0, s1b-c-0018/1, s1b-c-0018/2, s1b-c-0025/0, s1b-c-0027/1, s1b-c-0031/0, s1b-c-0041/1, s1b-c-0041/2, s1b-c-0044/0, s1b-c-0045/2, s1b-c-0047/1, s1b-c-0048/0, s1b-c-0049/0, s1b-c-0054/2, s1b-c-0059/0, s1b-c-0059/2, s1b-c-0060/0, s1b-c-0060/1, s1b-c-0062/2, s2a-c-0008/0, s2b-c-0020/0, s2b-c-0026/0, s2b-c-0027/0, s2b-c-0028/0, s2b-c-0028/2, s2b-c-0032/2, s2b-c-0060/0, s3a-c-0009/0, s3a-c-0009/2, s3a-c-0022/1, s3a-c-0028/0, s3b-c-0010/0, s3b-c-0015/0, s3b-c-0020/0, s3b-c-0021/0, s3b-c-0025/0, s3b-c-0028/0, s3b-c-0033/1, s3b-c-0034/0, s3b-c-0035/0, s3b-c-0035/2, s3b-c-0046/0, s3b-c-0048/0, s3b-c-0050/0, s3b-c-0052/0, s3b-c-0064/1, s3b-c-0065/2, s4-c-0013/1, s4-c-0015/0, s4-c-0020/0, s4-c-0044/2, s4-c-0084/0, s4-c-0088/0, s4-c-0089/0, s4-c-0091/0, s4-c-0092/2, s4-c-0115/0, s4-c-0115/1, s4-c-0123/1, s5-c-0001/0, s5-c-0003/0, s5-c-0027/0, s5-c-0043/0, s5-c-0051/0, s5-c-0067/0, s5-c-0076/0, s5-c-0076/1, s5-c-0085/0, s5-c-0087/1, s5-c-0090/0, s5-c-0090/2, s5-c-0091/0, s5-c-0095/0, s5-c-0097/0, s5-c-0098/0, s5-c-0102/0, s5-c-0102/1, s5-c-0132/0, s5-c-0139/0, s5-c-0151/0, s6-c-0022/0, s6-c-0030/1, s6-c-0035/0, s6-c-0037/1, s6-c-0039/0, s6-c-0044/1, s6-c-0053/2, s6-c-0055/0, s6-c-0055/2, s6-c-0068/1, s6-c-0073/0, s6-c-0083/2, s6-c-0091/2, s6-c-0094/0, s6-c-0128/2.

Accepted restoration reproduces all six dev counts and all1470 ordered pages
byte-for-byte; LLP reference check passes.

### Further encoder source inspection — native feasibility pending

[TrynMini v2 model card](https://huggingface.co/LNTTushar/trynmini-v2-static-7m-v2),
revision1568199a8b45413369fb274c1490b111e8d5c528, describes an int8 static table,
SIF-weighted pooling and a learned post-pooling matrix. Its published evidence
is STSBenchmark development similarity, not Linn retrieval. Inspected loader
drops reserved ids including unknown, dequantizes rows, and applies the matrix.
No weights downloaded or inference run; not selected ahead of a contextual
feasibility screen. No gain or speed claim inferred from its card.

[Ogma Small model card](https://huggingface.co/axiotic/ogma-small) points to
[Open Ogma Small](https://huggingface.co/axiotic/open-ogma-small), the permissive
successor with a different training recipe and output heads. Inspected small
revision3f4cd68f43719ca494772fcc80005c115d8d91ff has a six-layer256-wide trunk.
Do not transfer its metrics, routing convention or license claim to the older
noncommercial model or assume either is similarly fast in Linn. Source/config
only; no small-model weights or runtime measurement.

[Open Ogma Micro model card](https://huggingface.co/axiotic/open-ogma-micro)
motivates a distinct native feasibility screen: two128-wide contextual layers,
a64-wide token table and a default384-dimensional projection head. Proposal
was written before asset download. Revisionacb318a6f4411a42085838e674ebc2aed1219310:
F32 safetensors10,024,312 bytes, SHA256
0adf9dff70a6bbb3e792aa6ecca308f565af901f0188e4e51d7ff1c48483972d;
SentencePiece771,009 bytes, SHA256
b98cfeacdc4019bf12674417507bfeafe93d1bed6711d82adbc1fbbca4d5f548.
Pinned source and tensor layout inspected; remote Python was not executed.
The source normalizes pooled trunk output before the biased head, then
normalizes again. Its symmetric default is the declared fixed route; no
head/routing sweep. Native implementation, numerical/tokenization validation,
fresh CLI/resident latency and retrieval are still pending. Published benchmark
numbers are not evidence of Linn gains.

[SentencePiece Rust runtime documentation](https://docs.rs/sentencepiece-rs/0.2.2/sentencepiece_rs/)
offers a pure-Rust processor for embedded model bytes, but compatibility must
be checked on this actual tokenizer. No root dependency, product code, accepted
algorithm or scored page changed. The proposed screen retains original-text
ingest and the accepted lexical pieces tokenizer; only embedding tokenization
changes. One embedded replacement model, no runtime fetch or second runtime.
The existing adoption and similarly-fast requirements remain. This inspection
does not add a scored configuration;61 remain completed.

## Native Open Ogma Micro feasibility — not selected, 2026-11-10

The preceding proposal was implemented in an isolated native Candle/C++
SentencePiece build, with embedded pinned assets, symmetric task, small384 head,
512 body-token cap and zero-padding to512. Accepted lexical pieces, search rules
and scoring remained fixed. Two pre-normalized RoPE/SwiGLU layers and pooled
trunk normalization before the biased head follow the inspected source.
An independent scalar float64 forward pass agreed on six fixtures with maximum
coordinate error3.7163425517516835e-7; deterministic, finite, unit/zero and token
bound checks passed. This is implementation agreement, not upstream Python
inference parity or evidence of retrieval quality.

Pure-Rust sentencepiece-rs0.2.2 differed from the upstream C++ implementation
bundled by sentencepiece0.14.0 on48 of3,437 q4 query/alternate/full-body strings;
13 first differences were before raw token510. The encoder therefore used the
statically linked upstream implementation. Full-body tokenizer comparisons did
not apply the scalar cap and do not establish changed scored pages. The Nix
spm_encode executable crashed before output for an undetermined reason; the
C++ binding completed the comparison. Small fixtures alone had missed this
compatibility issue.

Three encoder-only rounds covered687 queries and6,096 blocks. Candidate query
rounds were99.284/98.176/99.148ms; accepted138.616/7.397/6.730ms. Candidate block
rounds6168.997/6177.417/6169.858ms; accepted688.010/542.117/543.202ms. Preserve
first-use observations rather than excluding them. These are encoding costs,
not end-to-end search measurements.

The predeclared actual-search feasibility gate required both median and p95
within25% of accepted for fresh CLI and resident search. Twelve fixed dev
original queries, two rounds, paired CLI order:24 invocations/model. Accepted
CLI median47.798ms/p95115.537ms; candidate56.025/163.578ms. Candidate p95 rose
41.58%, so it fails. Initial24 resident samples/model passed: accepted7.766/
63.664ms, candidate7.499/23.144ms. Including all three predeclared additional
paired resident processes,96 samples/model: accepted7.972/53.875ms, candidate
7.660/23.985ms. Resident success does not override the failed fresh-CLI gate;
no extra repetitions, head/cap tuning or favorable-sample selection.

Not selected; no full retrieval score, h2 run, quality-gain claim or additional
scored configuration. The initial corpus build deliberately selected zero
cases and is not an evaluation. This closes this fixed native feasibility
configuration, not all contextual encoders. Product dependencies and source
were restored. All1,470 accepted dev pages replayed byte-identically, with
411/439 gold-any,351/439 first,31/42 and25/42 cross-scope,35/51 no-gold and
55/73 section. LLP references pass. No product change is retained.

### Harmonic aggregation of matched term support — not adopted

One fixed primary-scoring configuration replaces the sum of positive per-word
BM25 contributions with IDF-weighted harmonic aggregation of their saturation
factors: (sum matched IDF)^2 / sum(IDF/saturation). Zero and single-positive-term
scores preserve the old arithmetic exactly. Missing terms add no weight or
support; existing admission, phrase/partial tiers, field weights, length
normalization, POS bonus, scope penalty, tie rules and interleave remain.
The accepted static encoder and original-text ingestion are unchanged. This
is a cross-term interaction, distinct from earlier per-term curve/IDF controls;
no new model, reranker, extra evidence or coefficient sweep.

Dev any411/439 first351/439 xs31/42 and25/42 empty35/51 section57/73.
All four retrieval totals are flat; the section-only rise from55/73 cannot
qualify under the adoption rule. Not adopted, no h2 or hybrid claim. Individual
gold-any gain s3b-c-0066 is offset by loss s3a-c-0054. Gold-first gains
s3a-c-0063 and s3b-c-0066 are offset by losses s1-c-0016 and s3a-c-0054.
There are418 changed ordered pages among1,470. First-page exports establish
those gold/cross-scope changes, not which all-call section objects changed.

A corpus-free property test passes: balanced two-term support outranks a
strong/weak pair that the arithmetic sum prefers; empty remains zero, a single
positive term remains exact, equal factors retain the arithmetic score, and
increasing one factor increases the score. No full candidate-suite claim:
the existing independent arithmetic-score assertion describes the accepted
algorithm and was not rewritten merely to pass this experiment. Prototype
and property test are archived outside the product before restoration.
Accepted source restored and all1,470 dev pages replayed byte-identically, with
all six accepted totals unchanged. LLP reference checks pass. This is the62nd
scored configuration in this branch's recorded loop, not a qualifying gain.

further model swaps after their accumulated weak gains.
The newly proposed TrynMini screen was stopped after pinned asset/dependency
fetch, before implementation, inference or evaluation. No model trial is counted;
further swaps are deprioritized in favor of retrieval mechanics. Product source
and dependencies were never changed for that screen.

### Matched hybrid fallback-stop counterfactual — diagnosis only

The existing admission/cut audit did not trace the later cascade. New native
instrumentation records actual stage exits, then invokes the existing beam,
rare-form block, rare-form section and sentence-pair stages counterfactually on
the75 wording/slice pairs where archived hybrid finds gold and Linn does not.
All591 saved card responses match exactly in both runs: record, span, text and
order. No returned-page, model, threshold, query, author-field or scorer change.
These overlapping slices are not75 independent cases or a new test set.

First nonempty actual stage:61 base-after-camel,4 gated wordpiece rewrite,
1 sentence pair;9 stay empty. Original-query subset:16 base-after-camel,
3 gated wordpiece rewrite,2 empty. Merely observing an early nonempty page
therefore does not establish that a later stage could recover the gold.

Counterfactual nonempty/gold-return counts: beam11/1, form-block2/0,
form-section14/0, sentence-pair20/0. Sole recovery: large-s5 s5-c-0059 wording2,
“agent commit sign-off”; the beam returns s5-r-0021 and s5-r-0152. No original
query recovers gold. Gold includes each object's own equivalents, never the
case-level field. No automatic card placement or section/silence scoring of a
bypass is claimed. Simply continuing these existing four stages after nonempty
pages has little measured recovery capacity on these matched hybrid misses.
This does not establish a ceiling for different stage internals, new evidence
mechanisms, or cases outside the measured slices.

No bypass implementation, scored configuration, h2 run or retrieval gain.
Full stem-aware scoring was also checked against the closed log and was not
rerun. Diagnostic source/events are retained locally; temporary instrumentation
was removed, product files match accepted source and LLP references pass.
The scored-configuration count remains62.

### Original source path-fragment population — no candidate

The earlier dotted-basename screen explicitly left full paths unexamined. A new
native read-only screen extracts slash-containing original-query tokens after
removing explicit filters; component characters are alphanumeric, underscore,
hyphen, dot and @ (including scoped package paths). Match lowercased source_path
on complete component boundaries, allowing a directory fragment anywhere in the
path. No body enrichment, metadata-to-body copy, approximate matching or file
opening. This is a permissive source-metadata ceiling before lifecycle,
uniqueness and outside-word disambiguation, not a proposed admission rule.

Among1,470 accepted dev wordings, seven contain a path fragment; five match
source metadata and already show gold, with zero unshown gold candidates.
Those five are s3b-c-0065/2, s4-c-0104/0, s4-c-0105/0, s6-c-0054/0,
and s6-c-0107/0. The two missed wordings, s1b-c-0036/2 and s3b-c-0060/0,
have no matching source_path. Original project paths are omitted here for privacy;
case IDs and population counts are unchanged. Neither metadata promotion nor further restricting
this population supplies a missing gold record. This does not cover approximate
paths, paths mentioned in bodies, case-first improvements among already-shown
gold records, or arbitrary metadata scoring changes.

No retrieval implementation, scored configuration, h2 run or gain. The native
source and complete population are retained locally; the temporary example was
removed. Product source and dependencies are unchanged. Unquoted dotted basenames
were checked against their closed finding and not rerun. Scored count remains62.

### Conditional association of selected nominal pairs — not adopted

One fixed configuration multiplies the existing smaller-token-IDF POS weight by
positive normalized association of the existing source relation. With eligible
block total N, relation document frequency J and exact-word frequencies A/B,
p=(J+.5)/(N+1), a=(A+.5)/(N+1), b=(B+.5)/(N+1), factor is
clamp(ln(p/(a*b))/-ln(p),0,1). Zero joint count gives zero. Apply only to
single-token pairs; multi-token pair weights remain unchanged. Counts use the
existing indexed relation lookup joined to lifecycle/expiry eligibility;
inconsistent joint/marginal/total counts cause an error and none occurred.

Unlike the closed pair-IDF controls, this compares relation incidence against
its words' marginal incidence rather than rewarding relation rarity alone.
No query expansion, new relation, model, threshold, admission, lexical score,
gap support, global0.75 multiplier, scope penalty or card-assembly change.
No coefficient sweep or corpus-wide resident frequency map.

Dev any410/439 first350/439 xs31/42 and25/42 empty35/51 section55/73.
Gold-any and first each fall one from accepted; reject, no tuning or h2.
Gold-any loses s3a-c-0054. Gold-first gains s6-c-0086 but loses s3a-c-0054
and s4-c-0073. There are138 changed ordered pages among1,470. Corpus-free
zero/weak-association, near-independence, strong-binding and argument-symmetry
fixtures pass. This is a failed retrieval mechanism, not evidence that the
counts were invalid or a reason to adjust a coefficient after seeing losses.
Prototype and fixtures archived outside the product, then source restored.
All1,470 accepted dev pages replay byte-identically and all six totals remain
unchanged. LLP reference checks pass. Scored configuration count is now63.

### Exact nominal pairs inside fenced code — population only

A native screen retains lines from complete matching backtick/tilde fences,
including scoped languages without NLP inference on code. It tests adjacent
single-token query terms selected by the existing nominal relation tagger
against literal same-line FTS tokens. No cross-line pair or invented occurrence.
Of1,470 dev wordings, five have unsupported primary syntax and are explicitly
excluded: s2b-c-0020/0, s3a-c-0018/0, s3a-c-0028/0, s4-c-0020/0,
s5-c-0005/0. Across13,606 code lines,127 pair witnesses cover119 supported
wordings; zero reach an unshown gold record. Lifecycle, admission and uniqueness
were not imposed, so this is a permissive witness screen, not a ranker trial.
Complete-fence, incomplete-fence, marker-length, indentation and line-boundary
fixtures pass. No code-pair scoring candidate, h2, score or gain is claimed.
Temporary instrumentation removed; source and population retained locally.

<a id="consolidated-evals-external-baseline-gap-final-65-md"></a>

### Final two fixed interactions; study stopped at 65

the study had a hard limit of 65 scored configurations, with success in
the report defined by any improved aggregate metric, distinguished from adoption.
The final two were declared together before either ran. No model swap, coefficient
sweep, runner-up after h2 or configuration66 is permitted in this research run.

64 combines unchanged source-witnessed nominal compound admission from26 and
unique original qualified-identifier suffix resolution from54 on the accepted
SSE/file-addressing baseline. These are explicitly tested interactions of held
mechanisms, not newly discovered individual rules. Dev any411/439 first353/439,
xs31/42 and25/42, empty35/51 section55/73;17 ordered pages change. Gold-first
gains s5-c-0051 and s6-c-0102, with no other gold/cross-scope case changes.
Dev-qualified but not selected for confirmation; no adoption claim.

65 is exactly64 plus the unchanged fixed harmonic positive-term aggregation
from62. Dev any411/439 first353/439, xs31/42 and25/42, empty35/51 section57/73;
433 ordered pages change. Gold-any gains s3b-c-0066 and loses s3a-c-0054.
Gold-first gains s3a-c-0063, s3b-c-0066, s5-c-0051 and s6-c-0102; loses
s1-c-0016 and s3a-c-0054. These gains do not establish interaction synergy:
the source licenses add their previously observed recoveries to the fixed score.

The predeclared selection order was gold-first, gold-any, XS-first, XS-any,
no-gold, section, with ties preferring64. Both qualify on dev;65 wins on section
and its frozen executable replays all1,470 selected pages byte-identically.
Existing compound-source and suffix ambiguity/literal contract fixtures pass.

One frozen65 h2-all run: any341/354 first296/354, xs57/62 and49/62,
empty55/106 section238/271. Against accepted341/297/57/50/56/239, first,
XS-first, no-gold and section each fall one. Reject; no retuning and no
confirmation of64 as a replacement. h2 is reused benchmark evidence, not a
fresh independent holdout. Full scorers establish empty/section changes; record
exports alone cannot attribute their case changes.

H2 changed ordered pages: 468/1,380. Paired record metrics:
- gold_any_gain: h2a1-c-0093.
- gold_any_loss: h2a1-c-0088.
- gold_first_gain: h2a1-c-0093.
- gold_first_loss: s7-c-0010, s7-c-0078.
- xs_any_gain: h2a3-c-0085.
- xs_any_loss: h2a3-c-0090.
- xs_first_loss: h2a3-c-0090.

Final experimental source removed. Accepted product source/dependencies match
commit66b55f9's algorithm; all1,470 accepted dev pages replay byte-identically,
all six totals remain411/351/31/25/35/55, and LLP references pass. The ledger
contains exactly configurations1 through65. Thirty have at least one positive
aggregate dev delta against their contemporaneous baseline; this broad reporting
criterion includes tradeoffs, flat h2 holds and the rejected final confirmation.
Only41 was adopted in this65-configuration loop, in product commit66b55f9.
The matched external baseline gap remains open. Research stops at the study limit;
no more configuration, alternative confirmation or model trial is pending.

<a id="consolidated-evals-source-combination-findings-2026-11-10-md"></a>

### Source-evidence combination after the 65-run stop

A bounded combination study selected and applied multiple successful configurations after the earlier hard stop. Combine
unchanged source-witnessed nominal compound admission26, exact colon-form
recovery38 and unique qualified-identifier suffix54 on accepted41. The SSE v2
encoder and accepted pooling remain. Word-balanced pooling from670dd97 is not
included: its prior gains lost both cross-scope metrics. Harmonic scoring is
excluded after its rejected combined confirmation. No added model or author
fields, corpus rewrite, threshold or scoring change.

Suffix resolution precedes ordinary search; nominal compound witnesses augment
lexical hits; colon recovery runs after the initial empty rewrite search. A
new mixed colon/suffix fixture failed because unknown-prefix parsing prevented
early suffix resolution. Resolving the suffix after safely quoting the colon
atom fixes composition while retaining original literal-body checks. This
correction preceded freeze/h2 and leaves the measured dev pages unchanged.

Combined dev any411/439 first355/439, xs31/42 and25/42, empty35/51 section55/73.
Gold-first gains s2b-c-0020, s4-c-0020, s5-c-0051 and s6-c-0102; no other
case-level gold/cross-scope changes.21/1,470 ordered pages change. Frozen replay
is byte-identical. One h2-all confirmation stays341/354 and297/354, xs57/62 and
50/62, empty56/106 section239/271. Seven h2 pages change with no case-level
record-metric changes: h2a1-c-0039/1, h2a1-c-0100/0, h2a1-c-0104/0 and/2,
h2a2-c-0018/0, h2a2-c-0064/0, h2a3-c-0012/1. Flat h2 is a hold under LLP0005,
not adoption. h2 is reused benchmark evidence, not independent validation.

A fixed refinement tested whether relaxing the query-POS gate could produce a
dev gain. It removes only the isolated-query nominal-tagging requirement; exact original
source nominal-pair witnesses, literal protections, grammar, eligibility and
all scoring remain. Require an incremental dev retrieval rise over the held
combination before another h2 run. Dev remains411/355/31/25/35/55; two pages
change but no metric improves: s5-c-0074/1 appends s4-r-0006, and s5-c-0137/0
introduces s5-r-0045 while dropping s5-r-0033. Refinement not selected; no h2,
retuning or gain claim. Neither this query-gate change nor the held combination
is promoted over the accepted product.

Combined-candidate validation passed formatting, strict workspace clippy, all
64 workspace tests (one ignored), and Nix build including release checks.
After both trials, accepted source was restored; all1,470 accepted dev pages
replay byte-identically, with411/351/31/25/35/55 totals. LLP references pass.
The held patch and refinement remain local ignored research artifacts only.


<a id="consolidated-evals-recombined-findings-2026-11-10-md"></a>

### Source and lexical-score recombination

A bounded process selected complementary changes and evaluated one combined candidate rather than every historical trial. Three fixed new dev combinations share unchanged nominal-compound, colon-literal and qualified-suffix source rules on the accepted SSE encoder and pooling. No author fields, second model, corpus enrichment or scorer changes. R1 uses maximum weighted field TF; R2 uses BM25L delta0.125 with summed TF; R3 uses both. The predeclared selection orders qualifying candidates by gold-any, gold-first, XS-any, XS-first, no-gold and section. No coefficient sweep or h2-driven adjustment.

R1 dev: any 412/439 first 353/439 xs 32/42 25/42; empty 36/51 section 56/73. 484/1470 ordered pages change. Paired record-metric outcomes: {"gold_any_gain": ["s2b-c-0006"], "gold_first_gain": ["s2b-c-0006", "s2b-c-0020", "s4-c-0020", "s5-c-0051", "s6-c-0102"], "gold_first_loss": ["s1-c-0016", "s3a-c-0015", "s4-c-0006"], "xs_any_gain": ["s2b-c-0076"]}.

R2 dev: any 412/439 first 354/439 xs 31/42 25/42; empty 35/51 section 55/73. 291/1470 ordered pages change. Paired record-metric outcomes: {"gold_any_gain": ["s2b-c-0006"], "gold_first_gain": ["s2b-c-0006", "s2b-c-0020", "s4-c-0020", "s5-c-0051", "s6-c-0102"], "gold_first_loss": ["s4-c-0003", "s6-c-0015"]}.

R3 dev: any 412/439 first 353/439 xs 31/42 25/42; empty 35/51 section 56/73. 442/1470 ordered pages change. Paired record-metric outcomes: {"gold_any_gain": ["s2b-c-0006"], "gold_first_gain": ["s2b-c-0006", "s2b-c-0020", "s4-c-0020", "s5-c-0051", "s6-c-0102"], "gold_first_loss": ["s3a-c-0015", "s4-c-0006", "s6-c-0015"]}.

All three qualify against accepted aggregate dev totals; R2 wins the declared tie-break. Relative to the held source-only combination, R2 trades one original-query success for one additional any-wording gold case. It is not a strict improvement over that control on every metric. The initial sparse/full-scan fixture retained the old BM25 equation and failed; updating only its expected equation to BM25L resolves that mismatch. Formatting, strict clippy, all64 workspace tests (one ignored), and LLP checks pass. The frozen native executable replays all1,470 selected dev pages byte-identically.

h2: any 341/354 first 298/354 xs 56/62 49/62; empty 55/106 section 237/271, against accepted 341/297/57/50/56/239. 297/1380 ordered pages change. Paired record-metric outcomes: {"gold_first_gain": ["h2a3-c-0074"], "xs_any_loss": ["h2a3-c-0090"], "xs_first_loss": ["h2a3-c-0090"]}.

scale-q4: any 388/439 first 316/439 xs 28/42 22/42; empty 31/51 section 51/73, against accepted 386/312/28/22/31/51. 324/1470 ordered pages change. Paired record-metric outcomes: {"gold_any_gain": ["s3b-c-0013", "s6-c-0102"], "gold_first_gain": ["s2b-c-0020", "s3b-c-0013", "s4-c-0020", "s5-c-0051", "s6-c-0102"], "gold_first_loss": ["s5-c-0129"]}.

scale-h2: any 325/354 first 275/354 xs 48/62 41/62; empty 47/106 section 225/271, against accepted 327/276/48/40/47/226. 354/1380 ordered pages change. Paired record-metric outcomes: {"gold_any_loss": ["h2a2-c-0019", "s7-c-0004"], "gold_first_gain": ["h2a3-c-0078"], "gold_first_loss": ["h2a2-c-0019", "s7-c-0004"], "xs_first_gain": ["h2a2-c-0094"]}.

Frozen h2 rejects R2: first rises one but XS-any, XS-first and no-gold each fall one, and section falls two. No runner-up confirmation or retuning. H2 is reused benchmark evidence, not independent validation. R1/R3 remain dev-only, unadopted combinations. Accepted source is restored; experimental snapshots remain ignored.

Synthetic six-document results remain Hit@1 31/49, Hit@3 and Hit@5 35/49, MRR@5 0.6735. Matched historical-hybrid comparison, candidate any/first/MRR: large-s1 45/48,40/48,0.6094; large-s5 76/81,67/81,0.5677; small-s1 46/48,42/48,0.6830. Versus accepted, large-s5 first rises one and each MRR rises slightly; all any/XS/no-gold totals are flat. Recorded external baseline still leads gold-any in all three slices. No fresh external baseline execution or gap-closure claim.

q4 full neutral: 12 passing rows and 4 failing rows; C1 occupies two rows. Failures: G1 eligibility violations=26.000, G4 integrity and forget=0.000, Q1 empty-correct=0.686, R2 maximum exposure share=0.229.

h2 full neutral: 11 passing rows and 5 failing rows; C1 occupies two rows. Failures: G1 eligibility violations=25.000, G2 must-not rate=0.052, G4 integrity and forget=0.000, Q1 empty-correct=0.519, R2 maximum exposure share=0.429.

scale-q4 full neutral: 11 passing rows and 5 failing rows; C1 occupies two rows. Failures: G1 eligibility violations=6.000, G4 integrity and forget=0.000, Q1 empty-correct=0.608, C2 latency p95 ms=123.920, R2 maximum exposure share=0.103.

The remaining scale-h2 neutral run was canceled; it produced no final gate table and is not a pass. Eight of nine evaluation commands completed, plus the matched historical-hybrid comparison. Scale-q4 neutral took3877 seconds, including independent rebuild and five scratch forget probes; this is not CLI query latency. Prior complete neutral tables predate SSE, so differences from them cannot be attributed solely to this combination. Detailed protocols, logs, timings and case/page ledgers remain in local evals/RECOMBINED-FINDINGS-2026-11-10.md and ignored recombination output. No product improvement adopted, push or PR.

Final accepted release build and replay reproduce411/351/31/25/35/55 and all1,470
ordered dev pages byte-for-byte. LLP references pass; product source and
dependencies match the accepted algorithm. A redundant accepted-source Nix
packaging rebuild was stopped, not counted as passing. No candidate packaging
validation or full scale-h2 neutral table is claimed.

### Acceptance of the combined source-evidence page

The combined changes were accepted for the parent branch. This supersedes the
prior R2 rejection;
it does not erase the measurements or claim the ordinary no-fall rule passed.
Adopt exactly the frozen source-witnessed nominal-compound admission, literal
colon recovery, unique qualified-identifier suffix resolution and BM25L delta
0.125 combination on accepted SSE v2 and technical file addressing. No further
tuning or alternative selection. Word-balanced pooling, maximum-field scoring,
whitening, harmonic scoring and the query-POS-gate removal remain unadopted.

Current dev: any412/439 first354/439, xs31/42 and25/42, empty35/51 section55/73.
Current h2-all: any341/354 first298/354, xs56/62 and49/62, empty55/106 section237/271.
Scale dev: any388/439 first316/439, xs28/42 and22/42, empty31/51 section51/73.
Scale h2-all: any325/354 first275/354, xs48/62 and41/62, empty47/106 section225/271.
The documented h2 and scale-h2 losses are accepted tradeoffs. This is not an
external baseline-gap closure or a full neutral pass; scale-h2 neutral was canceled.
LLP0002 now specifies every adopted mechanism; LLP0005 records this narrow
acceptance exception and current page, and AGENTS enforces it.

The research log includes all65 earlier scored configurations and their
individual outcomes, population/feasibility screens, the source-only combination,
the unsuccessful query-POS refinement, R1/R2/R3 interactions, and the previously
uncommitted exact-evidence IDF, inline-code and bidirectional nominal controls.
Historical not-adopted findings stay closed unless explicitly superseded above.
No corpus, model weights, experimental switches, raw eval output or reports are
included in the product commits.

Follow-up latency measurement: four serialized q4-dev runs in accepted/candidate/
candidate/accepted order on the same existing store, one Rayon thread and1470
wordings each. Prior accepted median8.276/8.125ms and p95 25.391/24.587ms;
combined candidate median7.890/8.026ms and p95 24.159/24.158ms. All runs reproduce
their respective retrieval counts. Roughly neutral on this small controlled
sample, not proof of a speedup or fresh CLI latency. The full scale-q4 neutral
suite takes3877s versus prior3894s; six POS publications account for2643.535s.
Earlier saved scale-query timings are not a controlled comparison: the prior
runs followed fresh ingest, while the candidate opened existing stores. No
additional scale run was made after scale runs were excluded from the plan.

Acceptance validation: formatting, strict workspace clippy, all64 workspace
tests (one ignored), LLP references and the Nix packaged build pass. Rebuilding
the accepted source reproduces all1,470 frozen candidate dev pages exactly.
No additional h2 or scale evaluation is needed for this unchanged frozen code.


## Consolidated research and performance notes — 2026-11-10

This supplement absorbs the remaining local research reports before their deletion.
It is historical evidence, not another specification or a request to implement
old proposals. Dates, comparison binaries, incomplete runs and negative outcomes
belong to each recorded experiment. Commands and transient progress logs are
omitted; measurement methods, configuration ledgers, limitations and references
are retained. Raw artifacts remain ignored under `evals/output/`.

The accepted source-evidence/BM25L combination recorded above remains current.
Earlier “current”, “default off”, “not merged”, model restrictions, temporary
switches, source paths and code line references describe their historical revision.
In particular, the later POS recovery study and fixed weight 0.75,
the focused embedded NLP runtime, subsequent lazy SQL migrations, and accepted
SSE v2 combination supersede their preliminary states below. Old missing-table
claims are superseded by the completed historical neutral runs; the accepted
combination's canceled scale-h2 neutral remains incomplete. No historical result
here is a fresh rerun, a new adoption, or authorization to retune a closed trial.
Untested proposals remain proposals; an inactive narrow grammar does not establish
that its entire research family was tested and rejected.

<a id="consolidated-ideas-md"></a>

### IDEAS.md

Original 58-idea disposition ledger, at the November 8 review. Entries 31/51 record the rejected eligibility experiment; later POS is a distinct mechanism. Entry 45 predates accepted colon-literal recovery; entry 48 predates the matched hybrid comparison. Historical unresolved questions are not asserted as present-day defects.

#### Per-entry ledger

R means prepared-query retrieval; U means explicit evidence access/use; D means
diagnosis/reliability. A U/D result alone never qualifies an R change.

| Idea | Merit verdict and evidence | Stream |
|---|---|---|
| 01 Vector completeness | D: healthy 6,096 vectors have no measured discrepancy; corruption probes reveal integrity coverage gaps. | E |
| 02 Exact body slices | D: all healthy rows match original slices and hashes; some independent span/hash corruption passes integrity. | E |
| 03 Porter derivation | D: all healthy stem text agrees; wrong/missing source derivation can pass internal integrity. | E |
| 04 Failure atomicity | D defect: SQL insertion abort rolls back durable rows but leaves failed record in resident memory. Encoder/commit faults unresolved. | E |
| 05 Collision/row-gap lifecycle | Already implemented; additional joint same-body/shared-basename/forget/reopen fixture passes. No healthy defect. | E |
| 06 Canonical id references | D defect: fetch-ID filters fail bare and quoted; no dev id-filter population, so no R gain. | C |
| 07 Token contracts | D: intentional contracts remain separate; additional norm/surface/identifier fixtures pass. No unification justified. | C/E |
| 08 Model/build identity | Much already implemented for numerical artifacts. Primary-vector provenance and external runtime manifest are narrower residual questions. | E |
| 09 Truncation signals | Existing show truncation and full reads pass probes; U merit, no reader-success measurement. | F |
| 10 Direct immutable access | Already available. No explicit supplied-reference miss population established; workflow benefit unmeasured. | F |
| 11 Original ingest completeness | Captured 1,376 bodies agree byte-for-byte. External intended-input completeness needs an authorized source manifest, not source_path reads. | E |
| 12 Explicit lifecycle corrections | Existing lifecycle behavior/tests pass; operational usefulness conditional on an independently documented real change sequence. Benchmark not edited. | E |
| 13 Setext headings | No genuine missing-heading population established; 101 apparent candidates were front-matter closers. Conservative screen, not complete Markdown certification. | B |
| 14 Container headings | No established population in the original-body screen; renderer extensions not exhaustively certified. | B |
| 15 Reference links | U merit; 28 explicit-label uses in conservative corpus screen. Full collapsed/shortcut/source-map syntax remains unresolved. | B/F |
| 16 Fragments | U merit: four section misses have fragment-bearing routes; unique anchor mapping and reader benefit unverified. | F |
| 17 linn links | No corpus population in syntax screen; no feature experiment justified. | F |
| 18 Path ambiguity | U/D merit: 69 shared paths, 159 ambiguous link occurrences; two section-miss routes ambiguous. No automatic choice justified. | F |
| 19 Replacement access | Partly available; old-fetch refusal passes. Explicit close/branch/cycle traversal and caller benefit unresolved. | F |
| 20 Outline pagination | Fixture confirms 20-of-26 cap; zero cap-only missed found targets on dev. Completeness merit, no demonstrated success gain. | F |
| 21 Targeted depth expansion | U merit: two found-record section targets hidden by depth. Caller selection and equal-budget benefit unvalidated. | F |
| 22 Record-local locate | U hypothesis; no caller-literal miss population established. Gold-supplied literals are only an offline ceiling. | F |
| 23 Bounded continuation | U merit: late targets and long-card views exist; source-byte reconstruction probe passes. Product cursor and reader success untested. | F |
| 24 Batch selected spans | U convenience hypothesis; all 18 section misses have one gold target. Multi-open necessity and success benefit unresolved. | F |
| 25 Contextual quote selector | D fixture passes; no repeated-gold-quote population among found-record section misses. | F |
| 26 Explicit relations | Partly implemented in later calls; finite route population overlaps already-found records. Incremental U benefit unmeasured. | F |
| 27 Facets/counts | U merit: 1,021 wordings exceed five raw lexical records with multiple source paths. No caller refinement/success evidence. | G |
| 28 Filter contrast | No independently known mistaken-filter task population. Existing path-only query does not establish a mistake. | G |
| 29 Parse preview | D/U merit: eight actual parse probes expose supported/no-op/error behavior. No automatic query repair or fixed-query R gain. | G |
| 30 UTF-8 hint safety | Prescribed behavior, not an accidental defect. Zero damaged hint chunks among 2,010 examined gold-record chunks; no R mechanism. | F |
| 31 Eligibility everywhere | Tested narrow eligibility candidate rejected after h2 falls; not a reusable dev-only win. | A |
| 32 Coherent clocks | Broader clock policy remains a separate unresolved question; fixed-clock dev had no new population. Not permission to retune A. | A |
| 33 Context-keyed caches | Publication guards already implemented. Broader context question unproven; failed A cannot be rescued by rebundling. | A |
| 34 Explicit heading/body grammar | No caller-explicit prepared-query population established. New contract alone does not improve existing wordings. | C |
| 35 Earliest stage | D delivered for executed matcher/page/fallback stages; pre-count rejected postings and one no-matcher subcause remain unresolved. | D/G |
| 36 Lexical ceiling | D delivered and reconciled with actual rewrites/admissions. Ceiling alone is not a full route trace. | D/G |
| 37 Rank displacement | D delivered: recorded keys/cuts/interleave reconstruct with zero errors. Does not authorize closed reorders. | G |
| 38 First versus any | D reconciled: 350 first, 60 alternate-only, 29 never recovered. Arithmetic opportunities, not achievable-gain claims. | D/G |
| 39 Span reachability | D delivered across all 18 section misses; gold-assisted access ceiling, not reader success. | F |
| 40 Silence versus cross-scope | D delivered: 41 target-required no-gold and 10 true-silence cases; distinct 42-case first-page cross-scope measure preserved. | G |
| 41 Duplicate cost | D delivered: first pages have 79 extra disjoint same-record spans, zero identical/overlapping repeats. Causal section necessity unresolved. No replacement experiment. | G |
| 42 Corpus-growth causes | D merit: 1,287 changed saved pages, 1,104 with added records; 183 without final added records. Exact DF/length/route attribution unresolved. | H |
| 43 Unseen encoded bytes | 37 truncated blocks, zero truncated dev queries. One wholly excluded gold quote belongs to an always-retrieved record. No missed-direct-record opportunity established by this screen. | H |
| 44 Mean-vector order ambiguity | D experiment confirms three relation permutations effectively identical; negation-addition control differs. No confidence-gate inference. | H |
| 45 Query syntax inventory | D delivered earlier; syntax limitation is distinct from caller-explicit literal intent. No silent unknown-prefix reinterpretation. | C/G |
| 46 Fallback interception | D delivered for observed activations and producers; no counterfactual disabled-component recall or authorization to reorder rescues. | G |
| 47 Paired outcomes/uncertainty | D discipline applied; unchanged diagnostics have zero discordant pairs. No candidate effect estimate to report in round two. | A/G |
| 48 Comparison coverage | D verified historical body/query exports; matched current index/scorer comparison still required. Synthetic six-document fixture is not neutral or hybrid superiority evidence. | H |
| 49 Retrieved/opened/supported/used | D/U distinction retained. Frozen outputs establish retrieval, not supported answers or correctly used evidence. | F |
| 50 Mechanism novelty | Applied across streams. Cut losses alone suggest closed families, not a novel algorithm. No round-two R experiment qualified. | G/all |
| 51 Joint eligibility snapshot | Narrow candidate failed h2; broader snapshot/DF variant not tested and not authorized as retuning. | A |
| 52 Explicit link-to-span route | U merit, not validated: six candidate section-miss routes, all target records already found. Exact resolution/selection remains open. | F |
| 53 Explicit literal mode | No demonstrated explicit-intent population in prepared queries; no automatic failed-query reinterpretation. | C |
| 54 Markdown/source access bundle | Heading population absent in screen; native offsets already exist. Link/fragment mapping remains U work with an unresolved renderer contract. | B/F |
| 55 Conjunctive witnesses | Narrow explicit two-clause population empty among misses. Broader relation mechanisms not proven impossible, but no novel candidate established. | D |
| 56 Full path intent | Existing path-plus-text already expresses tested narrowing; redundant as proposed. No inferred metadata promotion. | C |
| 57 Typed canonical execution | Query types exist; fetch-ID defect remains D backlog. Preview and broad parse-once migration have no fixed-query R evidence. | C/G |
| 58 Versioned bounded access | Immutable pins/currentness already exist and probes pass. Multi-command generation invalidation/recovery is unresolved U/D research. | F |


<a id="consolidated-ideas-astra-md"></a>

### IDEAS-ASTRA.md

Research only, 2026-11-08. Baseline `129897f950208faeefaa43ddd10f55537e46c092`, branch `reimplement`. No product, specification, scorer, corpus, existing report, or FINDINGS edits; no new retrieval evaluation, holdout inspection/run, scale run, commit, push, or PR. This report and `evals/output/astra-reassessment/` are the only new files. The existing uncommitted work is preserved.

#### Conclusion and priorities

There is still a credible research space inside the hard boundaries. The best remaining direction is **lexical retrieval that represents relationships between occurrences and returns their original evidence**, rather than another adjustment to the current fallback thresholds. I recommend three investigations, in order of implementation confidence:

1. A primary positional lexical matcher that distinguishes coherent local occurrences from scattered word coverage, evaluated before any per-matcher top-five cut.
2. An entity-bound join over original spans, jointly designing admission and which spans become cards. It must work on ordinary keyword queries, not require the benchmark to contain an explicit conjunction operator.
3. A much more speculative source-local binding mechanism for names and definitions actually written in the body. Test the population before building it; never manufacture a synonym or keyword field.

The first is the cheapest plausible ranking improvement; the second is the larger architecture with potential to change candidate populations; the third could bridge some vocabulary mismatch but has no established Linn opportunity yet. None has measured gains. A substantial generic paraphrase improvement, or dominance over matched external baseline, is **not established and should not be promised**. Several actual misses require information absent from the query/body intersection, or require the engine to decide that a syntactically mandatory term was merely context. Structural engineering cannot manufacture that information.

The strongest practical alternative, if those designs fail, is an explicit query contract separating required literals, descriptive text, and requested source restrictions. That is a separate product decision and study, not permission to reinterpret the fixed evaluation questions or reopen identifier-release failures.

#### What the evidence establishes

Read: all 58 entries of [consolidated finding](#consolidated-ideas-md), the three parent IDEAS reports, AGENTS, LLP 0000–0005, the closed retrieval history and surrounding research in FINDINGS, G/H/F/K/L reports, and the relevant implementation and scorer paths. In particular: `search.rs` scoring, matcher admission, per-record selection, page assembly and later calls; `fallback.rs` rewrites and early termination; `query.rs` token contracts; `sections.rs` original-byte parsing; `embed.rs` retained-token/mean-pooling implementation; `store.rs` publication guards, indexed structural projections, and on-demand positions/statistics. I did not inspect h2 case files or outputs; historical aggregate/disposition evidence in the requested reports remains historical.

##### The remaining problem is information selection, not missing caches

Baseline dev is gold-any 410/439, gold-first 350/439, cross-scope-any 30/42, cross-scope-first 24/42, no-gold 35/51, section 55/73. “First” means the original query's top-five page, **not Hit@1**. There are 60 alternate-only successes and 29 never-recovered cases. Repairing only the former cannot increase gold-any. The absolute arithmetic headroom is 89 gold-first and 29 gold-any cases, not a forecast.

G's 430 missed wording pages have first-executed-query barriers of coverage 197, zero eligible lexical overlap 65, hard term 53, matcher cut 100, interleave cut 14, and no matcher 1. These are conditional execution observations. Removing the recorded first barrier neither proves a candidate passes the next one nor identifies the final cause of failure. A later rewrite can use a different query entirely.

G also attributes 385 of those missed pages to a terminal lexical/rewrite route. An empty-page-only feature cannot reach most of them. Conversely, blindly moving a closed rescue before the lexical page is not a new design: the historical full-page swaps already show why that can lose gold.

Current code already materializes structural and numerical projections in libSQL, loads queried FTS positions and frequencies on demand, and prepares query-independent fallback encodings on writes. Old suggestions to add these caches are stale. A new design must preserve that operational pattern; rebuilding global positions or embeddings at every CLI start would be a regression, not a retrieval contribution.

##### New bounded diagnosis in this pass

I aggregated G's existing labels over the **original wordings only**. The diagnostic asserts that all 1,470 source rows belong to the q4 dev split. It uses G's hit flags rather than implementing a scorer.

| First executed barrier on original miss | Alternate-only cases | Never-recovered cases | Total |
|---|---:|---:|---:|
| Coverage | 20 | 15 | 35 |
| Hard term | 23 | 5 | 28 |
| Matcher record cut | 12 | 7 | 19 |
| Interleave cut | 1 | 0 | 1 |
| Zero eligible lexical overlap | 4 | 2 | 6 |
| Total | 60 | 29 | 89 |

This changes emphasis: the 28 hard-term original misses are mostly a first-wording problem, whereas coverage accounts for 15 of the 29 never cases. Do not translate that into “release hard terms” or “lower coverage”: those interventions are closed. It identifies the information a different mechanism would have to supply.

I also checked raw whitespace-adjacent alphabetic query pairs against exact gold bodies and the original byte spans of the shown cards for the 20 cut cases. Three had a pair in the exact gold body. Only one had a pair absent from every shown card:

- `s3a-c-0054`, “EAS Update incompatible updates channel compatibility”: `incompatible updates` appears in `s3a-r-0036`, but not in the five shown spans.
- `s3a-c-0014`: `runtime version` occurs in gold and four shown cards.
- `s3a-c-0035`: `eas update` occurs in gold and three shown cards.

These are **literal occurrence proxies**, not FTS candidate eligibility, legal gold-span containment, relevance judgments, candidate ranks, or gains. Full gold bodies and exact primary objects were used; equivalents were not included. The two ambiguous examples are as important as the favorable one. An adjacent-phrase bonus alone has little demonstrated reach. They do not rule out gapped, stemmed or relation-bound occurrences, which were not measured.

Command, exit 0:

Artifacts: `population.py`, `population.json`, `pair-witnesses.json` in that ignored directory. The script reads G attribution and q4 JSONL only, opens no source_path destination, reads no author-evidence fields for its analysis, changes no page and computes no retrieval score. The existing Rust scorer remains the sole grading authority. A separate read-only body inspection, restricted by those dev IDs, informed the examples below. No baseline rerun or fresh parity claim is made.

##### Actual source/query mismatches constrain every proposal

Examples from original q4 body bytes, used only for offline diagnosis:

- “Rust pinned tools command” points to a note about running Rust commands through the Nix flake. A positional algorithm cannot make the absent concept `tools` appear. Some useful words are also deliberately KIND words, excluded from coverage.
- “Grok rules per-file size cap” points to “Grok truncates each rules file at 10,000 characters.” `per-file` is hard; size/cap are paraphrases. Joining adjacent spans does not solve that.
- “sdk-57 branch release bug” points to guidance about main and `sdk-XX`. A generic template interpretation might look attractive, but `XX` is not automatically an authorized wildcard. Do not treat prefix resemblance as evidence of exact identity.
- “mobile:install iOS dev client” has the literal `mobile:install` in the original note. Its zero-overlap barrier is not proof that the body lacks vocabulary: colon parsing and subsequent execution need separate diagnosis. The proposed fetch-ID fix does not establish a fix for this query contract.
- “expire unread memories” points to expiry by last read being rejected. The relevant answer can contradict the premise of a query. A polarity filter that rejects contrary evidence would erase useful retrieval.
- The cut case “changeset docs test fixtures” points to a compact rule about documentation/tests not needing changesets. Ordinary match counts alone do not represent the relation between the topic and the exception.

H's same-token relation permutations support a real limitation of the static mean encoder: more accurate pooling of the same tokens cannot recover word order. H's truncation screen, however, establishes no missed-direct-record population caused by whole-quote truncation. K establishes that extra disjoint spans are sometimes indispensable: 21 wording/object sole witnesses and two individually necessary across wordings. A one-card-per-record redesign is therefore not automatically conservative.

F finds 14 of 18 section misses already retrieve the exact record somewhere among first pages. This argues for better evidence localization, but offers at most a section/access opportunity until a proposed mechanism also raises a qualifying retrieval metric. An already-found record is not necessarily a correctly opened or used answer.

G's no-gold breakdown is crucial: 41 cases require a cross-scope target; 10 require true silence. The 35/51 numerator is not a homogeneous abstention rate. All ten true-silence cases already show cards in the captured run. That gives no license to add distractors: all-call target/forbidden behavior still matters, and the tiny dev silence population is weak evidence about generalization.

#### Why the previous research framing was too restrictive

The earlier reviews correctly prevented repeated threshold sweeps and preserved source integrity. Their weakness was the search strategy for **new mechanisms**, not the validity of their measured rejections.

1. **A disposition ledger became a stopping criterion.** Fifty-eight entries assessed is not fifty-eight algorithms tested. G explicitly implemented zero new retrieval mechanisms. Most of the catalogue concerns integrity, syntax, navigation or diagnosis. Completing it does not exhaust retrieval design.
2. **An explicit syntax screen narrowed an algorithmic question unnecessarily.** D's absent two-clause population says the declared grammar did not trigger. It does not test whether a keyword query's evidence is spread across spans with a shared subject. Such a mechanism need not look for `and`, a semicolon or two externally supplied clauses.
3. **“That would reorder” is too broad a novelty objection.** Every useful replacement primary matcher eventually changes ordering. The closed experiments establish adverse evidence for their particular candidate sources and signals: cosine, term supersets, rarity, arbitrary swaps. They do not establish that occurrence relations carry no information. Conversely, adding one new name to the same old swap is not novelty.
4. **Counting before admitting and selecting a span was treated too independently.** A union of words can nominate the right record and choose the wrong passage. A useful replacement must specify a source witness, candidate admission, record competition, card span and stopping rule together. Testing only union recall would be misleading.
5. **The fallback cascade conditions the experiment population.** A new late stage can have strong standalone candidates and zero deployed opportunity. An experiment should trace its intended activation before implementation, then compare the whole page. This does not authorize running existing rejected semantic rescues earlier.
6. **Conservative syntax proxies were sometimes promoted into broad architectural pessimism.** No Setext population, no wholly hidden quote, and no explicit literal intent are useful narrow results. They do not certify all structure, semantic visibility, or operational-literal queries. The correct response is to name what was tested and what remains unknown, not automatically invent another variant.
7. **Information absence was conflated with an indexing defect.** Some queries demand a contextual association the source never writes. A model-free lexical design cannot guarantee that association. Research should expose this limit rather than replace the absent words with researcher-supplied synonyms.

There is also extensive historical adaptation to known development and holdout failures. I will not use historical h2 examples to derive a new exception. Repeated exposure makes small dev wins less persuasive even when they pass the exact numerical rule; mechanism-level controls and a predeclared design matter.

#### Ranked research designs

These are research specifications, not adoption recommendations. All preserve immutable originals, one native Rust binary, one local libSQL file, the embedded static model, no runtime downloads, no learned reranker/RRF, no extra model, and no manufactured content. A replacement primary lexical matching function is distinct from applying a second scorer to a retrieved shortlist. If a proposal degenerates into that forbidden or closed operation, stop it.

##### 1. Primary positional lexical retrieval over original occurrences

**Hypothesis.** Two rows can satisfy the same coverage and tier while one expresses connected query concepts and the other merely scatters their words. The current BM25 equation sums token evidence; its phrase tier recognizes the whole joined content query, not all meaningful subrelations. Confidence computes positional features but its gate is open. Most genuine keyword queries are not one verbatim phrase. A primary matcher using local occurrence relations could distinguish some of the 114 all-wording cut misses without creating vector-only candidates.

**Candidate source and representation.** Keep the existing exact and Porter eligibility/hard-term/coverage definitions for the first prototype. Enumerate the complete admitted postings, not each matcher's top five or top twenty. Each hit carries `(record, block, original field, token occurrence, byte witness, match kind)`. Exact and Porter positions remain distinct; parts hits map to the original identifier occurrence and do not invent adjacency across identifier boundaries. Body and heading fields cannot form a phrase by concatenation. The same original token cannot be counted twice because it appears in heading and body indexes.

Use original paragraph/list-item/fence boundaries as local units for occurrence relations, without changing 1,400-byte index blocks or embeddings. A fence is not parsed as natural-language prose. Read queried positions from FTS's instance vocabulary; source-offset mapping for candidate text must be checked against the actual tokenizer, not approximate Python tokens. Start with body-only exact relations, extending to already-permitted Porter matches only in a separately declared ablation.

**Concrete first prototype.** For adjacent content-term pairs in the parsed query, find the closest ordered occurrence pair inside one original local unit. Let `gap` be intervening token count, and let pair support be `min(idf(a), idf(b))/(1+gap)`, at most once per pair. Phrases and identifiers remain indivisible required terms. Use the existing lexical score plus the sum of pair support as the primary row score; preserve tier priority, penalty 7 and stable ties. This fixed equation is a falsifiable first design, not an invitation to sweep weights or gap cutoffs. It is deliberately weaker evidence than a semantic relation parser. Reverse-order and unordered occurrence measurements are diagnostic controls, not extra rules silently added after failure.

Initially keep the existing per-matcher record grouping, interleave, card spans, later calls and fallback order. This isolates new information in the primary lexical score before attempting a larger executor replacement. A later unified matcher is justified only if the trace shows independent A/B cuts losing useful candidates that the new evidence can actually distinguish. Merging lists alone is not the proposed benefit and is not permission for RRF or duplicate replacement.

**Admission, span and stopping.** Admission unchanged in this first prototype; do not award pair evidence to an unadmitted row. Return the original winning block, with current excerpt and size behavior. Stop at five through the existing assembly. Empty lexical queries remain empty at this stage; existing later routes still execute. Changing selected records also changes later calls and potentially short-page/fallback behavior, so even admission parity does not guarantee no-gold parity.

**Affected population and ceiling.** The first-query cut group is 20 cases, seven never recovered by any wording. Twenty is the ceiling for fixing those particular original misses without changing admission, not a ceiling across all alternative wording pages; later wordings of the other never cases might have cut barriers too. All-wording 114 is a count of pages, not cases. The raw-pair screen gives only one especially distinct exact phrase witness, not twenty likely gains. Cross-scope and section effects are unquantified. It cannot rescue absent mandatory tokens or zero-overlap gold.

**Why silence might survive / why it might fail.** No new lexical admission is introduced, so the direct risk is displacement rather than automatic empty-page filling. Nevertheless a more coherent wrong passage can outrank gold, and changing the first five changes all-call navigation. Contradictory evidence can be relevant; do not use `not` as a negative-relevance label. Short notes and duplicated documentation may dominate proximity scores. Bag-like queries may have arbitrary term order.

**Closed comparison.** Unlike cosine/convex mixes, no vector similarity or second ranking list enters. Unlike strict-superset replacement, candidates need not contain more query words and there is no chosen victim card. Unlike historical window-coverage/QPP gates, coverage is not tightened or relaxed and no confidence cutoff is fit. The missing signal is ordered within-unit occurrences, applied before the candidate cut. Merely using the old minimum-span confidence feature to gate cards would not qualify as this experiment.

**Cost.** For query posting volume `P` and `m` terms, merge ordered positions in roughly `O(mP)` for adjacent-pair relations, plus candidate selection. No corpus-wide map on startup and no new vectors. A local-unit offset table is linear in original structural units; it can be materialized on writes in the same publication if needed. Initially parse only candidate spans to avoid a schema change. Frequent-term queries can still approach corpus size; record touched postings, candidate bytes and peak memory. Separate CLI opening from resident query timing. Pair-IDF tables and all-pairs corpus statistics are unnecessary for this prototype.

**Minimum proof and falsifier.** First reconstruct exact/Porter term occurrence provenance on the 20 original cut cases and matched currently successful controls, with all words/token boundaries preserved. Determine whether the gold's usable relation differs from its actual competitors within its tier. If no meaningful distinctions exist beyond the single raw phrase witness, stop the broad architecture investment. Then one fixed dev-only implementation and Rust scorer run, with paired pages and full spans. Controls: baseline; new executor with relation contribution disabled (must reproduce baseline); ordered relation contribution; corpus-free reversed-relation, scattered-words, separate-field, repeated-token and code-fence examples. If apparent gains come only from unintended admission, stem-score, scope, or span changes, the hypothesis is falsified. A falling dev floor ends that candidate; no weight sweep.

**Decision changes.** LLP 0002 scoring and exact occurrence/match provenance; LLP 0001 only if new original-offset/unit projections are persisted. Existing query semantics and encoder contract remain. Any unified matcher follow-up additionally changes two-matchers/call assembly and must be a separately specified architecture, not a refactor claim.

##### 2. Entity-bound evidence joins with joint record/span selection

**Hypothesis.** Some relevant records distribute the requested properties across nearby original units. Current heading-tree unions can admit broad co-occurrence, but choose the representative block by content-word count, and skip tree admission for records already represented in the ranking. A tightly bound witness could both admit a missed record and localize the evidence more accurately. This is not “more tree coverage”: it requires a shared original subject connecting the witnesses.

**Candidate source and representation.** Start from eligible original postings for every query term, before coverage rejection. Group candidate units by exact record and parsed section, not label, filename, ancestor path or scope. Join two units only if both contain the **same exact identifier that also occurs in the query**, with its full spelling verified in original bytes. Each unit must contribute at least one different non-anchor query content term. All query hard terms must be witnessed exactly somewhere in the pair; every content term must have a permitted exact/Porter witness across the pair. No similarity edge, inferred entity, generic shared word, shared source_path, or cross-document join.

This is an incidence graph over existing occurrences, represented by references to bytes. It creates no textual surrogate. Begin with two units in the same parsed section; do not expand to arbitrary siblings or documents if this declared population is absent. The important change from old idea 55 is implicit keyword-query execution plus an explicit repeated-subject binding, not an explicit two-clause query grammar. The high bar is intentional: lowering it to `need-1` would collapse into a closed family.

**Admission and ordering.** A valid pair is one record candidate with two distinct span witnesses. It enters a separately defined partial tier after single-block full matches; within partials compare the weakest witness's existing BM25 (each witness must contribute evidence), subtract the existing scope penalty once per record, then stable source ties. Do not sum every matching section: long documents should not win by volume. This is a proposed primary matching rule over the witness set, not a reranker of the old top five. Run it for all text queries with the required literal anchor, including nonempty and full pages. Query features alone trigger it, never case category, gold or outcome.

**Span choice and stopping are part of the mechanism.** A pair is useful only if the reader can reach its actual witnesses. Nominate the constituent block with the stronger non-anchor evidence for call 1, using existing lexical score then original offset to resolve ties. Schedule its other distinct block in the bounded additional-sections call; never claim that the first block alone covers the whole query. If both units fit in one existing block, keep that block. Do not create a huge parent span or score a union of two cards as one section hit. Five first-call cards and three total calls remain. Do not deduplicate other disjoint baseline spans merely to create room. If this packaging cannot fit without losing necessary evidence, that is a real failure of the architecture under this contract.

For a first diagnostic prototype, keep pairs in shadow output and test whether a legal full page could use them without oracle span selection. For an actual candidate, the above competition and later-call rule must be frozen; no fallback interception after a page is built. Old semantic fallbacks remain available only if the resulting primary search is empty. This execution order is a durable change.

**Population and ceilings.** Coverage 197 and hard-term 53 wording barriers are only an overinclusive search space. The original 35 coverage and 28 hard-term misses are not estimates of eligible joins. All-hard/full-content witnesses will exclude most short-note paraphrases shown above. Hard-term misses are affected only when the exact term exists in another bound span, never when absent from the record. The actual witness population has not been measured. Section upside is bounded by 18 current failures, with 14 already-found records; that alone cannot qualify adoption. A record/heading boundary screen with no bisected gold does not test this mechanism: it joins query evidence without requiring the gold answer itself to cross a boundary.

**Failure modes.** The same identifier can denote unrelated examples in a long reference section. Two facts about one subject do not necessarily answer their requested relationship. A query may name several subjects; requiring the same anchor can miss them. Full coverage can still match a negated or hypothetical claim. Min-of-two score is conservative but may discard the valuable weak witness. Selecting one block for call 1 can preserve record hit and lose section containment. Long fences should not be treated as several natural-language claims.

**Closed comparison.** Current trees join normalized word sets without subject-occurrence proof and select by count; old broader tree/one-under experiments relax coverage. This proposal instead requires full content evidence and a repeated exact query anchor. Old two-sentence fallback splits the query in half, uses cosine and corpus top-15 sentences, activates on emptiness and returns one block. Here candidates come from lexical postings, the binding is exact, matching may cross blocks, and it runs before page selection. Old “most query words” section substitution and strict-superset replacement change a winner after retrieval; this selects a new compound evidence object before record competition. D's zero explicit-clause population does not test this object. If removing the repeated-subject constraint makes the only gains appear, do not rename that as a success of bound joins.

**Cost.** Naive all-pairs joining is quadratic in units per common identifier. Use SQL intersections keyed by `(record,section,anchor)` and query coverage bitsets; retain only nondominated witness assignments for the fixed two-unit grammar, preserving distinct source spans. An exact diagnostic may enumerate all pairs on q4 to establish the ceiling. Production must not silently cap candidates and call that equivalent; a budget cap requires explicit incomplete-result semantics or a proven bound. Memory scales with touched anchor postings and candidate pairs, not every corpus word. Ingest adds unit membership/byte boundaries if absent, no re-embedding. Repeated broad identifiers can be costly even with a small query.

**Minimum proof and decisive controls.** Offline Rust diagnostic using public parsing and actual FTS occurrences: enumerate bound witnesses across *all dev queries*, then separately attach gold, baseline hit, no-gold category and section labels for analysis. Report candidate false associations as well as potential gold reach. This is an oracle upper-bound diagnostic until a fixed rank/span policy is executed. Controls: unchanged baseline; single-unit-only version; join evidence with original arbitrary representative-block selection; complete join plus witness-aware span scheduling. Compare the last two to test the essential admission/packaging interaction. Do not run rejected tree relaxations as controls. Corpora-free disjoint-subject, same-anchor/unrelated-topic, contradictory-clause and hidden-heading fixtures should reject spurious proofs. No nonempty miss population, or no gain beyond section, falsifies its priority for R. Any floor fall rejects the frozen candidate.

**Decision changes.** LLP 0001 derived unit memberships and publication/forget behavior; LLP 0002 matching unit, hard-term locality across a pair, compound-candidate ordering, call-1 and additional-section scheduling. LLP 0004 only if exposing match explanations in JSON. LLP 0005 scoring is unchanged: no union credit, no additional cards. No changes to author-field or ancestry prohibitions.

##### 3. Source-local name/definition bindings as an evidence operation

**Hypothesis.** Some apparent vocabulary mismatch is a name and its explanation already connected in the original document. An exact binding can be stronger than ordinary co-occurrence or vector proximity. This is potentially useful for technical documentation, but it is the least established proposal here.

**Scope and representation.** Recognize a small predeclared grammar of original definition structures: a Markdown table with a literal name cell and its description cell; or an explicit definition containing two quoted names and a fixed relation such as “also called”. Store only source byte ranges, containment and the relation kind. No synthesized alias strings, keyword lists, summaries, semantic synonym dictionary, copied headings, or rewritten FTS body. In particular, do not infer `expire = expiry`, a package from a filename, or `sdk-XX = sdk-57`. Parenthetical proximity alone is insufficient proof of an alias.

For tables, a row is an original contiguous source span; headers delimit structure but their words do not become row evidence. For explicit two-name definitions, both strings must be present verbatim in the same original unit. A name's repeated occurrence elsewhere must be in the same record and section for the first prototype. This is a relational lookup over original occurrences, not a global entity knowledge base.

**Candidate generation and admission.** Query text must literally match a name or descriptive terms in a definition unit. Use existing postings to reach that unit, then exact name occurrence postings to reach one associated unit. Require an additional original query content term in that target unit, and full hard/content coverage across the two units. A hard literal must occur as itself somewhere in the evidence: a purported alias cannot satisfy a missing hard token. Thus this does **not** reopen absent-identifier release, substring identifier matching, or parts-as-exact evidence. The mechanism can connect description vocabulary to a named technical passage only when the source supplies the bridge.

**Order/span/stopping.** Treat the definition and associated passage as a two-witness candidate using design 2's partial-tier, weakest-witness ordering and bounded span schedule. Prefer a direct single-unit witness over a joined witness at otherwise equal evidence; resolve remaining ties by existing source keys. One hop, no transitive aliases, no traversal through other records. Run as part of the primary evidence join, not after an arbitrary wrong result terminates the cascade. Return actual original blocks; if the required original table/definition unit is oversized and no existing legal card contains the target, refuse to claim a section repair rather than invent new chunking.

**Affected population.** Possible subset of coverage and hard-locality failures; none measured here. The reported six zero-overlap original misses are not an upside estimate: this operation itself needs a literal/description posting, and cannot repair genuine no-overlap with no source binding. A parser-caused zero-overlap has a different remedy. No defensible numeric gain ceiling below the overall 89/29 is available until the binding inventory is done. Most of the inspected short-note mismatches do not contain the needed binding; expect the population study to be capable of stopping this avenue quickly.

**Novelty and boundaries.** Beam and PMI replace a query using corpus-neighbor words; this follows an explicit original relation and returns its witness. Rare-form methods use shape/pieces and model cosines; this requires written definitions and exact name identity. Literal search idea 53 is one contiguous caller-requested substring; this is a bounded structural join with description evidence. It is not permission to create “aliases” from model reasoning or researcher expertise. If the source does not state the binding, the candidate is absent.

**Costs/failures.** Linear source parsing on ingest, a small indexed relation table in the existing DB, query-time work proportional to matched definition/name occurrences. Common names and reference tables can still create large fanout; no unbounded graph expansion. Ambiguous table structure, negated aliases, historical renames and examples can all create false bindings. Forget/invalidation must remove relation endpoints and every derived lookup. No second model, new vectors or runtime tokenizer service is needed.

**Proof and falsifier.** First inventory the frozen grammar in original q4 bytes and manually audit every purported binding used by a missed dev query, alongside matched successful and no-gold controls. Then a Rust shadow evaluator reports potential witness pairs without producing pages. If no missed gold has a verifiable bridge, close this *grammar/population*, not all structured retrieval. Do not add a succession of bespoke relation phrases to catch known misses. If viable, compare design 2 with and without source bindings using the same rank/span policy; remove relation edges as a counterfactual diagnostic while keeping original text intact. Reject if the apparent benefit depends on absent-hard-term substitution or manufactured content.

**Decision changes.** LLP 0001 original relation projections; LLP 0002 a narrowly specified source-binding query operator and evidence admission. No change to the “original documents ingest on their own” requirement: all derivation is deterministic from source bytes, and callers supply no extra fields. This interpretation must be expressly accepted before implementation; it must not be smuggled in as an enrichment step.

#### Interactions and architectures to avoid

Designs 1 and 2 can be synergistic: occurrence relations distinguish local evidence; an exact shared subject connects complementary units. Neither alone guarantees the right card span. Design 3 supplies a different, explicitly written connection for the same witness executor. That shared executor is the coherent large replacement: query requirements → indexed occurrence witnesses → bounded source relations → record competition → original-span cards. It is not a bundle of existing fallback scores.

Test 1 independently first. Test 2 independently if its population exists, then 1+2 with both ablations. Add 3 only if its inventory supports it. Require actual overlap/complementarity of changed cases; summing independent numerator gains is invalid. A local proximity preference may conflict with a necessary distributed witness. A single best record score may conflict with K's required disjoint spans. A definition bonus may favor encyclopedic hubs. Later-call changes can lose all-call empty or section even when the first-page record set improves.

Do not spend the next round on: numeric BM25/IDF/coverage sweeps; putting every old fallback in a parallel pool; cosine to replace the tail; a new confidence model; universal identifier softening; hub/recency/scope demotion; more arbitrary chunk overlap; or new embeddings. None supplies the new evidence relation proposed above. Eligibility candidate `a4c5b39` remains rejected and closed; its dev +1 is not a component to combine. Canonical-reference candidate `fef2faf` remains separate and its broader quote/filter grammar change must be reviewed on its own merits.

#### A different query contract may be the more honest product change

The hard-term rule infers “required” from punctuation/capitalization. Users also use those forms as background context. Meanwhile KIND removal means words like `pinned`, `version`, `command` and `file` score but do not establish coverage. Those are intentional semantics, not bugs. A query language can make these distinctions explicit: required literal constraints, ordinary descriptive terms, and explicit path/under selectors. For known operational commands, punctuation-preserving literals should be expressible without being mistaken for a filter.

This is related to old ideas 34/45/53/57, not claimed as a wholly new algorithm. The useful extension would be a clear required-versus-descriptive contract, with unchanged legacy mode, not another preview command. A caller would deliberately opt into it. Do not automatically downgrade `per-file`, reinterpret `sdk-57` as a template, or use a gold-informed rewritten query. Do not evaluate it by feeding prepared alternatives or source quotes into the caller. It could improve practical success on future caller-written queries, but cannot increase the fixed original-query R numerator merely by adding syntax that those queries never use.

A future separate study needs fresh task-driven queries from people or the existing caller, equal read/call budgets and verified original-source use. No model agent, caller study, benchmark rewrite or new corpus acquisition is authorized in this pass. It is reasonable for Linn to remain a precise lexical evidence tool rather than promise a natural-language answer engine with the same semantic reach as a larger hybrid stack.

#### Practical next experiment sequence and authorization

1. **Freeze the research definition before implementation.** Preserve current corpus/model/publication/build fingerprints, case split, scorer, baseline pages, query sequence and context. Name every changed trigger, candidate source, representation, admission rule, order, span rule and stopping rule. Record a fixed small experiment budget; do not search dozens of weights and select a lucky dev gain.
2. **Read-only native witness diagnosis.** Start with design 1's exact positions and competitor distinctions, plus design 2's two-span bound-witness count. Run over dev queries only and attach gold metadata afterward. Use baseline successful queries and every dev no-gold wording, not only the 89 misses. The small Python proxy here is insufficient for candidate admission or section conclusions. Keep trace/query generation separate from the Rust scorer. No scale or h2 needed.
3. **Implement at most one fixed primary lexical candidate first, after authorization.** Reproduce all six baseline numerators and all 1,470 ordered pages with the candidate disabled. Preserve repeated IDs, and compare source spans/calls as well as page IDs. Check both resident sequence behavior and fresh invocation where caches matter; page-neutral instrumentation must remain page-neutral. Run the fixed candidate on the full dev split through the unchanged Rust harness.
4. **Report paired transitions, not just totals.** For every changed wording, report executed parse/route, entering/leaving IDs, full original spans, first causal divergence, and lost or retained section witnesses. Aggregate by case, preserving correlations among wordings. Keep gold-first, gold-any, cross-scope pair, target-required no-gold, true silence and section distinct. Report gains and losses even when net flat. Do not relabel a plausible non-gold answer to rescue a candidate.
5. **Evaluate the essential interaction only when justified.** If bound witnesses exist, freeze the complete design 2 admission/order/span rule and its two meaningful ablations. They test a new interaction, not previously rejected heuristics. Combine with design 1 only after paired dev evidence identifies complementary behavior. Design 3 follows only a successful population audit.
6. **Retain unchanged adoption rules.** At least one of gold-any/first or cross-scope-any/first must rise on dev; none may fall; no-gold and section may not fall. Section-only gain is insufficient. Small dev gains can qualify numerically yet remain fragile; report case-level uncertainty, not a made-up success probability. Check the full implementation with repo fmt, clippy, tests, Nix build and LLP references before freezing.
7. **Parent alone controls confirmation.** Freeze the entire selected algorithm and decision changes. Exactly one parent-authorized h2-all confirmation, with the same no-fall and retrieval-rise rule: flat means hold; fall means reject without retuning. No per-case h2 exploration, no selecting among multiple h2 candidates, and no reopening the rejected eligibility variant. This pass has no such authorization and performs none.
8. **Measure external claims separately.** After a genuinely confirmed improvement, request a matched external baseline comparison and any scale/neutral work separately. Match eligible original bodies/IDs, index size, query keys and wordings, version/model/index identities, ranking budget and scorer. A full-index result restricted to old slice case IDs is not the same experiment as building the restricted index. Save all-call/spans if making empty or section claims; historical document hit lists cannot support those claims. Full neutral validation requires the real fixed-loop population, not an empty selection or synthetic fixture.

Authorization boundary is straightforward: this report and diagnostic analysis were authorized. Product prototypes, persisted projection changes, LLP decisions, new caller/query-contract studies, new scale/neutral/hybrid runs and any h2 confirmation are future work requiring the parent/study-owner authorization. No permission question is needed to finish this research deliverable. The constraints on models, RRF, reranking, authorship surrogates and protected trees remain even with permission for large changes.

#### Matched external baseline: plausible competition, insufficient evidence of a win

The historical comparison is mixed. H verified body/query correspondence for restricted indexes and slices; the historical hybrid can lead gold-any/cross-scope while Linn leads first-wording on another slice. The current 439-case full-index numbers cannot be compared directly to a 48- or 81-gold restricted comparison. The six-document synthetic benchmark is neither the neutral gate nor a hybrid baseline.

I consider improvement over Linn's current page plausible, and a win on some matched hybrid metrics plausible. Uniform superiority across matched recall, silence and section under these hard boundaries is substantially less certain. Positional and source-bound evidence could help where hybrid overgeneralizes and where lexical evidence is present but poorly selected. It cannot generally infer the missing association between pinned tools and Nix, or determine when an unmentioned technology is implied by a task. The static mean's inability to encode order is a reason to use lexical positions, but not evidence those positions solve every semantic miss.

No causal ablation of the historical hybrid stack establishes how much of its gain came from expansion versus the other embedding versus its reranker. The prior assertion that the joint stack proves each component is necessary is too strong. Equally, it gives no basis to claim that a deterministic replacement will reproduce the joint gain. The attainable claim is a falsifiable sequence of new evidence mechanisms, followed by matched measurement. If those mechanisms lack a useful population or fail the no-fall rule, retain the current page and consider the explicit query contract as a separate product improvement.

#### Primary references verified for this reassessment

These references justify mechanisms or implementation feasibility, not Linn gain estimates. Historical FINDINGS citations were not all revalidated and their numerical claims are not imported as new evidence.

- **Donald Metzler and W. Bruce Croft, “A Markov Random Field Model for Term Dependencies,” SIGIR 2005.** [Author-hosted paper](https://ciir.cs.umass.edu/pubfiles/ir-387.pdf). It models unigram, ordered-phrase and unordered-phrase evidence and reports benefits on its own collections. It also discusses mixed prior phrase results and learns parameters, unlike the fixed deterministic prototype proposed here. Conclusion for Linn is only that term dependence is a substantive retrieval signal beyond unigram scoring; it supplies neither a transferable gain nor permission for a new learned model.
- **SQLite, “FTS5 Extension.”** [Official documentation](https://www.sqlite.org/fts5.html), especially phrases, NEAR, and the fts5vocab instance table. It documents positional/phrase capabilities and term/document/column/offset information. This supports on-demand occurrence retrieval in the existing database. It does not mean FTS offsets are UTF-8 byte offsets, that tokenizer streams align, or that the pinned libSQL exposes every newer auxiliary API. The prototype must verify the pinned engine and retain original-byte maps.

The concrete designs and predictions in this report are my inferences from Linn's code, source bodies and diagnostics. They remain unimplemented and unmeasured.


<a id="consolidated-ideas-astra-expansion-md"></a>

### IDEAS-ASTRA-EXPANSION.md

Proposal and novelty record. The E1–E4 implementation outcomes below close their measured configurations, not these broader hypotheses. No new experiment was run during consolidation.

#### Recommendation

There is a credible research direction inside the stated model boundary, but
not evidence for another general semantic fallback. Use the static vectors to
propose **which original phrase might substitute for which query phrase**, and
require a separately inspectable account of the substitution's scope, arguments,
and remaining obligations. A candidate must not certify its own relevance merely
because a query rewritten from that candidate retrieves it.

The best first investment is a small population and precision screen for
**source-attested equivalence**, mechanism E1 below. E2 is a more ambitious,
candidate-specific phrase alignment experiment; E3 exploits explicit row/field
structure where it actually exists. E4 is a coordinated alternative to today's
first-card stopping procedure, needed if the aim includes fixing existing false
silences rather than merely avoiding additional ones. These are three candidate
mechanisms and one joint architecture, not four independently established gains.

I would not yet spend a full changed-page experiment on E2–E4. Their decisive
questions can first be answered with original-byte witnesses and corpus-free
counterexamples. In particular, reciprocal similarity is not entailment, a
unique candidate can be uniquely wrong, and a source's negative answer may be
exactly the relevant evidence. No proposal here promises to beat hybrid.

#### 7. Exact novelty ledger

These comparisons refer to the closest actual trials, not to an imaginary weak
baseline. “New” means not measured in the reviewed log; it is not a claim of
novelty in the IR literature or permission to implement.

| Dimension | Closed nearest-word / beam / late-interaction families | E1 | E2 | E3 | E4 |
|---|---|---|---|---|---|
| Trigger | Empty page, short page, or five partials, depending on trial | Supported explicit local equivalence; initial output only if final page has room | Supported ordered two-anchor frame; initial output only with room | Exact object plus explicit original row/property structure; initial output only with room | Supported obligation grammar, regardless of page fullness; replaces that route's stopping policy |
| Candidate source | Global attested words/forms, nearest blocks, or lexical top twenty | Explicit equivalence endpoints in query-anchored original blocks | Intersection of exact anchor postings, then source gaps in one region | Exact row-key postings plus actual original headers/cells | Pre-cut lexical candidates plus one independently screened new generator |
| Representation | Token/block means, form means, residual beam, token minima | Source relation offsets; vectors schedule attested edges only | Separate phrase means plus ordered, injective positional bindings | Header/property means plus typed row/column addresses | Obligation-to-original-span certificates with witnessed/contradicted/unverified states |
| Admission | Cosine/IDF/frequency gates, original word count, or rewritten tier | Explicit local equivalence AND full remaining original intent | Complete reciprocal gap alignment AND exact anchors/operators; still an empirical synonym hypothesis | Exact row binding AND explicit equivalence for nonliteral field AND real value witness | Certificate required for every emitted card on the supported route |
| Ordering | Cosine, score mix, rank fusion, beam score, nearest-victim replacement | Common original-query lexical keys, no semantic score fusion | Same | Same | Filter before original cut/interleave; new certified rows tier 2; no uncertified fills |
| Span | Block/section chosen by similarity, counts or rare form | Existing block containing declaration and use | Existing block containing full aligned frame | Existing block containing header, row and mapping proof | Existing blocks with full certificates, including later calls |
| Stop | First successful fallback / fixed top-k / confidence cutoff | Exhaust applicable local declarations; conflicting proof means no new card | Exhaust structural alignments; incomplete/ambiguous means no new card | Resolve all row/field bindings; unsupported means no new card | Finish bounded proof search; supported query may stay empty; no weaker fallback |

What would collapse these proposals back to closed work:

- E1 without an actual equivalence statement becomes ordinary semantic term
  expansion. If it relies on extra author terms, it is forbidden.
- E2 without ordered original arguments becomes token late interaction or
  nearest-word-plus-block scoring. Reverse-dotting the same pair adds no signal.
- E3 without row binding becomes tree/record word union; without source field
  evidence it becomes a nearest-header confidence heuristic.
- E4 without new certificates becomes another coverage/confidence gate. E4
  with the old empty neighbor as generator becomes a renamed closed rescue.

##### Attractive ideas I would not spend experiments on

**Residual subtraction as a new sense encoder.** Subtracting normalized query
terms from a normalized mean is not an exact leave-one-out pooled vector. More
fundamentally, even exact mean arithmetic leaves token context unchanged. The
current residual beam already explores that direction. It needs a new source
constraint, not another residual formula.

**Reciprocal nearest neighbors alone.** A two-node antonym pair can be mutual
nearest; duplication and corpus vocabulary change reverse ranks. UMAP's local
neighbor-density adjustment already gained dev and lost holdout first. Another
mutuality or local-scale cutoff without typed evidence is not a credible new
silence mechanism.

**Bootstrap paraphrases from shared argument contexts.** This is richer than
adjacent PMI in principle, but `enable/disable`, `increase/decrease`, and
`read/write` can share arguments. A small technical corpus also supplies few
independent contexts. Without an original equivalence or relation-preserving
witness, a DIRT-like relation inventory adds inferred synonyms as another
unverified representation. Do not build it as a permanent table or claim the
joint “learn then narrow” bundle is novel enough by itself.

**Require expansion stability under dropping words.** Closed QPP research already
considered variant stability. Stability can mean a hub ignores the important
constraint. Counterfactual *source-binding* tests in E2–E4 are useful falsifiers;
random-query nulls or a tuned robustness score are not a new admission rule.

**Antonym veto lists.** A list written from q4 failures is an answer-key probe.
A small generic polarity vocabulary also misses implicit opposites. Protect
explicit operators and unsupported syntax; do not claim a complete negative
semantic lexicon without one.

#### 9. Primary sources checked and what carries

These sources motivate distinctions, not Linn improvement estimates. None's
models, datasets, derived dictionaries or learning steps are imported.

- **Melamud, Levy and Dagan (2015), [A Simple Word Embedding Model for Lexical
  Substitution](https://aclanthology.org/W15-1501.pdf).** The task is substituting
  a word while preserving meaning in its sentence. Their method explicitly uses
  skip-gram context embeddings as well as target embeddings. Linn's static mean
  table does not expose that learned context channel. Inference here: separate
  substitution validity from topic similarity; do not claim E2 reproduces their
  contextual model merely by adding nearby words.
- **Pavlick et al. (2015), [PPDB 2.0: Better paraphrase ranking, fine-grained
  entailment relations, word embeddings, and style
  classification](https://aclanthology.org/P15-2070.pdf).** The paper distinguishes
  equivalence from directional entailment and other relations, and improves
  rankings with supervised judgments. Inference here: a proposed synonym needs
  a relation type and direction; a broader term can discard intent. Their
  learned ranking and external paraphrase database are not available components
  under this task's boundary.
- **Lin and Pantel (2001), [DIRT—Discovery of Inference Rules from
  Text](https://www.cs.columbia.edu/~kapil/documents/candidacy_papers/lin01.pdf).**
  The authors extend distributional similarity from words to dependency paths
  connecting arguments, and explicitly include relations that are not exact
  paraphrases. Inference here: keeping argument roles is a substantive change
  from adjacent collocations, but sharing arguments does not certify
  substitutability. E2 uses original explicit positions, not their parser or
  mined rule collection. No DIRT-derived synonym store is proposed.

Existing sources in FINDINGS on contextual retrieval, confidence calibration,
late interaction, sufficient context, structure, and hybrid are retained as
historical context. Their numerical results are not fresh validation and are
not combined with Linn scores. The six-document synthetic fixture lacks the
silence/scope/section populations needed here. Historical hybrid comparisons
use matched 90- or 1,243-record eligible indexes and particular case slices;
they cannot be compared directly to the full q4 floors. The hybrid lead came
from expansion, a different embedding model and a reranker together. This
report neither runs nor prescribes that stack.

#### 10. Candid feasibility conclusion

The promising distinction is **proposal versus license**. Static embeddings
can cheaply suggest a multiword correspondence; an original equivalence
statement, an ordered argument binding or a row/field relation can provide
information absent from the pooled vector. Prior failures do not logically
close that combination. But only E1 supplies a relatively strong substitution
license, and it may be too sparse to improve this corpus. E2's reciprocity is
still a hypothesis, and E3 needs explicit source structure. E4 can affect true
silence only by taking on the difficult job of validating existing lexical
cards as well as new semantic ones.

My order is: **E1 population/precision screen; E2 adversarial alignment screen;
E3 structure population screen; E4 only if an actual complementary population
survives.** Do not begin with a broad ranker rewrite or a full experiment matrix.
Do not relax hard identifiers, borrow h2 counterexamples for new gates, or
manufacture author evidence to make these populations exist.

If the useful paraphrases have neither literal anchors, explicit equivalences
nor source structure, the current boundary leaves no independently grounded
semantic validator in this report. The missing capability is general contextual
relation understanding, not another cosine threshold. A future discussion could
instead change the explicit user query contract or evaluate caller-driven
clarification as a separate access task. That would not improve the unchanged
prepared-query benchmark, and no unauthorized model or runtime is prescribed.
Retaining an honest miss is preferable to claiming that a reciprocal nearest
neighbor has proved an answer.


<a id="consolidated-ideas-astra-pos-md"></a>

### IDEAS-ASTRA-POS.md

Research only, 2026-11-08. Parent baseline `129897f950208faeefaa43ddd10f55537e46c092`, branch `reimplement`. This is the requested new Astra POS investigation. I worked alone. No product/specification/FINDINGS changes, dependency installation, asset download, corpus changes, new checkout, commit, push, PR, or retrieval/adoption run. Writes are limited to this report and ignored `evals/output/astra-pos/`.

#### Recommendation

**Investigate POS as occurrence metadata and as a way to propose phrase boundaries on actual keyword queries. Do not use it to keep only nouns or to certify relevance.** The new information is a possible distinction between different uses of the same spelling, and between connected and accidental co-occurrences. It is not another lemma, a synonym dictionary, a dependency parse, or an entailment model.

Both Astra reports already propose the important retrieval architecture: original occurrence witnesses, bound joins, phrase alignment, row/property bindings, and preserved query obligations. Neither specifies an nlprule POS implementation. This report adds a concrete linguistic front end, ambiguity policy, source-offset contract, and ablations that can show whether it improves those architectures. Much of the architecture is deliberately reused; calling all of it new would be misleading.

Best next steps, in priority order:

1. **Establish resource reproducibility and audit keyword-query tagging before implementing retrieval.** An English tokenizer binary exists locally, but the offline probe failed before compilation because `bimap` is not cached. No tags, chunk accuracy, query activation, RSS, or latency were measured here.
2. **POS-selected occurrence pairs for P1.** Compare against P1's untyped ordered pairs, with exactly the same candidates, score equation, and page executor. This is the smallest experiment that could demonstrate new evidence beyond activation.
3. **Whole noun-phrase anchors for P2 and literal E3 lookup.** Work from raw keyword sequences without requiring “what is … for …”. Keep the entire modified phrase as the identity; a head noun alone is not an entity key. First establish source-witness and legal-card populations.
4. **Typed phrase obligations for E2/E4, initially in shadow only.** POS can expose a dropped verb/modifier or an implausible substitution type. It cannot turn a reciprocal vector neighbor into a synonym or settle ambiguous argument attachment.
5. **Context-sensitive treatment of KIND words only after the above.** A word such as `command`, `memory`, or `version` can be the object being requested. POS alone cannot distinguish that from a weak request noun. Begin by preserving roles in diagnostics, not changing coverage.

The implementation priority is therefore diagnostic front end → P1 ablation → independently viable P2/E3 source binding → E2/E4 only with actual new evidence. No implementation or adoption is recommended now.

#### Evidence and measurement boundary

Reviewed AGENTS, LLP0000–0005, FINDINGS, both complete Astra reports, the parent results/review/follow-up reports, G/H reports, and relevant query, matcher, fallback, section, embedding and storage paths. Baseline files remain untouched. Historical holdout summaries in these requested documents were read as dispositions; no h2 case files, outputs, or searches were opened/run. No scale/full-neutral/hybrid evaluation occurred. Original body access for any future diagnostic must use q4 `records.jsonl`, never a `source_path` destination. No external corpus checkout was changed.

Current q4 DEV floors remain **any 410/439, first 350/439, xs 30/42 and 24/42, empty 35/51, section 55/73**. “First” is original-wording top-five success, not rank-one success. The original misses are 89: 35 coverage, 28 hard-term, 20 matcher/interleave cuts, six zero eligible overlap, classified under the first executed query. Sixty are alternate-only; 29 never recovered. These are saved diagnostic populations, not new POS results or gain forecasts. No syntax-only feature can claim the entire 89-case opportunity.

The supplied implementation-wave update says E1/E3/E4 have zero DEV activation under their controlled grammars; E1 also has zero frozen source declarations. P3 has four source bindings in one record but zero eligible cross-block pairs even with unbounded gap/fanout. These are parent-supplied observations, not reruns here. POS could change the query-parser population; it cannot create missing source declarations or cross-block witnesses. E1/P3 remain stopped under those grammars unless an independently specified source mechanism provides new evidence.

G distinguishes 41 no-gold cases needing another-repository targets from ten true-silence cases; the separate cross-scope first-page denominator is 42. Holding empty 35/51 is not proof of good abstention. H demonstrates that same-token role permutations are effectively indistinguishable to the existing mean encoder. POS/chunks may preserve additional order information outside that vector, but a chunk sequence still is not a semantic role assignment.

#### 1. Verified nlprule capability

##### What actually runs

Official documentation reports version **0.6.4**. The existing clone at `<historical-checkout>` is revision `ebb54617ea5f3378c0dc63486631724b7e97e13c`; its package also says 0.6.4. Source was inspected read-only. The public overview correctly describes lookup tagging, statistical chunking and rule-based disambiguation as different operations. [nlprule crate documentation](https://docs.rs/nlprule/0.6.4/nlprule/)

| Component | Verified implementation/API | Consequence for Linn |
|---|---|---|
| Dictionary tagger | `tokenizer.tagger().get_tags(word)` / `get_tags_with_options(word, add_lower, use_compound_split_heuristic)` return multiple `(lemma, POS)` analyses. | This alone has no sentence context. Do not select the first tag as the answer. |
| Main pipeline | `Tokenizer::new(path)` or `Tokenizer::from_reader(reader)`; `pipe(text)` iterates sentences; `sentence.tokens()`, `token.word().tags()`, `tag.pos().as_str()`, `tag.lemma()`, `token.chunks()`, `token.span().byte()/char()`. | Use actual original strings and preserve all analyses and ranges. Grammar correction `Rules` is unnecessary. |
| Pre/post comparison | `sentencize(text)` yields incomplete sentences after tagging/chunking; `disambiguate(sentence)` then `into_sentence()` gives the contextual rule pass. `pipe` does these together. | `sentencize` is **not** a cheap token-only operation; it already runs the chunker when present. |
| Disambiguation | Ordered contextual rules can remove, add, replace, filter or unify analyses. Public `rules()` exposes the rule set. | It is not merely dictionary equality and is not guaranteed to leave one correct POS. Empty/ambiguous output must stay explicit. |
| Chunking | Optional `Chunker`, accessible through `chunker()`, internally runs maximum-entropy tokenization, maximum-entropy POS tagging, then maximum-entropy chunking. | Three statistical assets, not “just rules.” This is the requested linguistic capability to assess, not an authorized second semantic embedding stack. |
| Internal versus public POS | The statistical POS sequence feeds chunk prediction; `Chunker::apply` populates `.chunks`. It does not replace public dictionary analyses with the internal best POS sequence. | A public `NN/VB` ambiguity can coexist with one predicted NP/VP chunk. Those outputs are not independent semantic votes. |
| Heads/relations | No exposed dependency tree, head index, subject/object relation, coreference, or entailment API in the inspected pipeline. | Noun-head selection and attachment are additional Linn hypotheses, not nlprule features. |

Source references: [tokenizer pipeline](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/src/tokenizer.rs), [lexical tagger](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/src/tokenizer/tag.rs), [disambiguation operations](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/src/rule/disambiguation.rs), [chunker](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/src/tokenizer/chunk.rs). The detailed findings above come from these local source files, not an executed binary.

English examples in the upstream documentation include `JJ`, `NN:UN`, `VBZ`, `VBN`, punctuation tags and `B-/I-/E-NP-singular`, as well as VP labels. A noun classifier that checks only `tag == "NN"` is wrong. Freeze an explicit mapping of the actual English resource's noun/proper-noun/verb/adjective tags after inventory; do not silently interpret every string starting with `N` as a noun. Distinguish “at least one noun analysis” from “all surviving analyses are nominal,” and retain the original full tag strings. A singleton NP can carry both begin and end labels. [Upstream example](https://github.com/bminixhofer/nlprule#readme)

**Important offset and chunk limitation.** The chunker retokenizes an internal copy, replaces curly apostrophe `’` with ASCII apostrophe, computes character ranges, then attaches chunk labels only where an internal token's character range exactly equals a main-pipeline token's range. Thus some tokens can lack chunk labels because tokenizations disagree; this is not proof of “outside noun phrase.” Number decoration is based on dictionary NNS evidence. Use `Span::byte()` for original slicing, not character offsets, and verify each slice. `pipe` shifts sentence spans to the original input; candidate-local calls need the candidate's original base offset added once. See [span types](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/src/types.rs) and the chunker's `apply` implementation.

##### Technical text and underspecified keyword queries

The English config enables lowercase analyses and retains the last analysis under its relevant disambiguation option; its compound-split heuristic is disabled. It supplies a URL-join regex and no language-specific extra hyphen splitting. Main tokenization and the statistical chunker's internal tokenization are separate. None of this changes Linn's FTS `_`/`-`, camelCase, phrase or filter contract. [English tagger configuration](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/configs/en/tagger.json), [English tokenizer configuration](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/configs/en/tokenizer.json)

Treat these as explicit audit strata, not assertions of measured accuracy:

- **OOV names and identifiers:** dictionary lookup can have no analysis; a statistical chunk prediction does not prove a technical name's identity. Exact quoted/underscore/hyphen/camelCase terms stay protected even if tagged as verbs, punctuation or unknown.
- **Keyword ambiguity:** `record changes`, `build cache`, `release notes`, `test fixtures`, and `runtime version` omit grammatical context. Do not insert “find the” or a synthetic sentence to induce the desired tags. Record query-only alternatives before source inspection. A source sentence cannot retroactively make one query reading uniquely intended.
- **Code, flags, URLs and inline literals:** classify region boundaries from original Markdown, keep literal offsets, and abstain from linguistic attachment across them. Code still participates in baseline word search. Excluding it from POS evidence must not exclude it from search.
- **Unicode and normalization:** preserve UTF-8 byte offsets, combining marks, CRLF, curly apostrophes, punctuation and original case. Do not tag `norm` text and reuse its offsets. FTS token positions, nlprule tokens, surface words and model wordpieces are four distinct coordinate systems.
- **Hyphens and compounds:** no new split or release rule. `react-navigation`, `read-only`, versions and `--no-cache` cannot become ordinary noun pieces merely because the tagger struggles.
- **Long prose and Markdown boundaries:** sentence segmentation is not the block cutter. A heading, table cell, code fence or blank-line join must not manufacture an NP or relation crossing independent original regions.

For a first head heuristic, accept only a contiguous two-token NP with an unambiguous nominal final token and nominal/adjectival first token, outside protected regions; call its output a **proposed modifier–head pair**. Do not assert that every preceding word modifies the last noun of a long NP. A three-noun compound such as `cache invalidation policy` has internal attachment choices; rightmost head does not settle them. Longer compounds should retain a span and an unresolved head set until separately audited. This conservative heuristic may have low support; zero support is an acceptable diagnostic outcome.

##### Resources, licensing, offline packaging and costs

The existing cache contains `<historical-checkout>`, **11,590,973 bytes**, SHA-256 `e2b51e506aede495268ee05dee0b97956e768107f5bb5891585144a5ff2fd773`, plus compressed copies. That is a file measurement, not loaded memory or verified model provenance. No compatible executable was found in the inspected clone/cache paths. The asset was not copied or changed.

Attempted only:

Exit **101**, before compilation: `error: no matching package named 'bimap' found`. The probe depends on the existing local nlprule clone with defaults disabled and `regex-fancy` requested; it does not depend on Linn or open a store. Its source and stdout/stderr remain in `evals/output/astra-pos/`. No retry without offline mode, installation, or fallback tokenizer occurred. **All runtime tagging, cached-bin compatibility, actual resource contents, accuracy and performance tests remain unexecuted.** This is missing runnable dependencies, not proof the tokenizer assets are absent or a POS hypothesis failed.

The upstream build recipe uses LanguageTool dictionary dumps, added/removed entries, disambiguation XML, SRX sentence rules, optional multiwords, and English OpenNLP `en-token.bin`, `en-pos-maxent.bin`, `en-chunker.bin`; conversion stores the statistical parameters in the tokenizer resource. Runtime does not require Java/Python if the compiled binary is already supplied. The crate itself is MIT OR Apache-2.0; upstream declares distributed language binaries derived from LanguageTool 5.2 under LGPLv2.1. This does not establish all component provenance for the cached file. A future packaging decision needs pinned resource origin/checksum, original notices and source/rebuild materials, and the applicable distribution/relinking mechanism for embedding the licensed data. Do not assume that the Rust crate's MIT license covers its dictionaries/models. [Build resources and recipe](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/build/README.md), [binary license statement](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/binaries/README.md)

`include_bytes!` plus `Tokenizer::from_reader` makes build-time embedding technically plausible. The convenience builder normally acquires binaries; do not add an implicit network build step. A future Nix fixed input must supply validated bytes offline. Check native dependencies: 0.6.4 defaults to `regex-onig`, with `regex-fancy` available instead. Backend substitution needs rule-equivalence verification; it is not a free portability claim. Confirm the final release is freestanding with the existing packaging checks. No runtime filesystem model lookup, service, Java, Python, or second product binary is proposed.

Cold loading is not free: deserialization reconstructs tag word maps and analysis vectors from serialized FST data, and chunk parameters are expanded from serialized bf16 to f32 in the inspected implementation. This is the linguistic asset's own format, not permission to quantize/change Linn's 512d model. The source has statistical beam decoding and contextual-rule work; OOV and long sentences can cost differently from dictionary hits. Measure, separately, executable size, cold tokenizer load, RSS increment/peak, query-only tagging, candidate prose tagging, ingest/projection preparation, store open, first-use query and resident query latency. No numeric latency estimate is justified here. Lazy loading avoids paying for unused routes but does not remove fresh-CLI cost when a route runs.

#### 2. Exact overlap and closed-work ledger

P1/P2/P3 below denote designs 1/2/3 in [consolidated finding](#consolidated-ideas-astra-md); E1–E4 denote [consolidated finding](#consolidated-ideas-astra-expansion-md). The latter's §2 explicitly excluded a hidden POS tagger. The user's new request authorizes **assessment** of that capability; future implementation needs explicit LLP decisions describing statistical linguistic resources. It does not authorize a dependency parser or learned relevance model.

| Prior idea or result | Already proposed/tested | New POS information/component | Synergy, redundancy and decisive falsifier |
|---|---|---|---|
| P1 primary ordered occurrences | Adjacent query pairs, within-unit positions, pre-cut primary scoring | Select pairs by compatible nominal/verb chunk use; distinguish same spelling used in different roles | Complements position evidence. Redundant if untyped pairs make the same distinctions; stop if gains require candidate/coverage changes rather than POS. |
| P2 entity-bound two-unit join | Repeated exact **query identifier**, all content/hard evidence, joint card scheduling | Propose a full modified ordinary NP as an additional anchor type on keyword queries | Real extension, but weaker identity than an identifier. Stop if only generic head matches or unrelated same-phrase occurrences create pairs. |
| P3 local names/definitions | Explicit source declaration connecting description and named passage | Boundary/role annotations on already-existing endpoints | Cannot create a declaration. Four bindings/zero pairs is not fixed by better query tagging. No independent population, no P3 work. |
| E1 source-attested equivalence | Written equivalence plus use, no semantic license from cosine | POS can flag verb/noun endpoint mismatches, or delimit an ordinary endpoint | Does not license equivalence. Zero declarations remains zero; do not change grammar to fit known misses. |
| E2 phrase-gap alignment | Ordered exact anchors, injective gap assignment, separate operators | NP/VP spans and ambiguity sets proposed from actual keywords, not question templates | Replaces span enumeration, not reciprocal/equivalence proof. One gap pair remains vacuous; accepting an antonym with matching POS falsifies POS-as-validator. |
| E3 row/object/property | Original row key/header/value addresses; strict field proof | Enumerate NP object/property partitions without “what is property for object” | Parser activation can improve while source rows/certificates remain zero. Full literal row/header evidence must distinguish a swapped-row control. |
| E4 obligations and stopping | Raw-query obligations; witnessed/contradicted/unverified; validation before cuts and all later cards | Preserve POS/phrase role alternatives and modifiers in that obligation representation | A frontend improvement only. If “has noun” becomes “witnessed,” or ambiguous keyword order becomes asserted subject/object, reject it. |
| FINDINGS lemma ceiling, around 1224 | One extra content term on two blocks among 41 never-hit gold records; neither clears coverage | Contextual occurrence roles/chunks were not that experiment | No permission to repeat lemma/stem equality. POS must distinguish identical lexical inventories or establish a new bound witness. |
| FINDINGS nlprule discussion, around 1699/1757/1767 | English lemma equality, derivational/non-equivalent forms, English hyphen configuration | Full pipeline contextual rules and statistical chunks | Not measured Linn POS failures. Still cannot turn `unread` into `read` or release identifiers. |
| FINDINGS POS/head-modifier literature proposals | Open-class side-column and head-modifier suggestions cite external results | Concrete nlprule feasibility, limits, metadata-only design and controls | General POS/NP ideas are not new. Newness lies in implementation contract and source-bound use, not renaming a side column. |
| H mean order ambiguity | Same-token permutations collapse in mean embedding | External occurrence order and chunk boundaries can survive pooling | POS sequence alone also matches many role reversals; source anchor binding is still necessary. |
| Closed cosine/beam/PMI/confidence/superset/A trials | Concrete rejected mechanisms/dispositions | None reopened | A noun-weighted nearest neighbor, noun-only beam, POS confidence score, or rebundled `a4c5b39` is not this research. |

**Literature correction, recorded here only:** Lioma/Ounis (2006) Table 2 does contain BM25 .280→.297 on WT2G and .234→.233 on WT10G for POSCGT. But it reduces POS blocks in the **query narrative**, using POS-pattern statistics induced from Europarl and TreeTagger; title/description/narrative contribute to retrieval. It does not test the proposed Linn document side column. It includes foreign words in its open-class mapping and uses probability filtering as well as content load. Those figures are not evidence for noun-only deletion on short keyword queries. [Paper and Table 2](https://aclanthology.org/P06-2069.pdf)

Zhai (1997) does report single words plus head-modifier pairs improving initial precision .4546→.5162 and recall .56→.60; combining both phrase types gives recall .63 with initial precision .4747. This is a learned NP-structure parser and CLARIT experiment with feedback on TREC-5 topics, not nlprule, not a rightmost-noun heuristic, and not Linn top-five scoring. Whole NPs alone also improve precision in the paper; “head-modifier pairs always beat full NPs” is unsupported. The useful conclusion is to test relationships **in addition to** words, with separate phrase ablations. [Paper and Table 1](https://aclanthology.org/A97-1046.pdf)

#### 3. Common contract for new mechanisms

All mechanisms below are unimplemented research candidates. A POS annotation references original bytes; it never becomes a searchable noun list, summary, label, synthetic sentence or copied ancestor heading. No writer-supplied terms. Baseline matching, 512d encodings and numeric kernels remain untouched during diagnostics.

Keep the raw query and typed filters. `query::content` removes `not/no` and short words; `Query::parse` retains `not/no` as lexical terms, but does not resolve negation scope. Neither is a semantic obligation tree. Preserve quotes, hard identifiers, flags, numbers, versions, quantities, comparisons, direction, modality, negation and all modifiers. Freeze a query-only set of possible analyses before seeing candidates. Unknown or incompatible analyses mean no new proof, never noun defaulting or word deletion. An answer can be negative: a note rejecting expiry by last read may be relevant to an expiry query. A polarity-equality filter is unsafe.

Minimal diagnostic representation:

Exact/Porter/parts provenance remains separate. A parts hit maps to its original identifier occurrence; it never becomes a hard-token hit or a fabricated adjacent phrase. Repeated tokens cannot pay twice for distinct arguments. Head selection is metadata and never drops its modifiers. No generated searchable text or new semantic representation is stored.

##### Rank 1 — POS-selected occurrence evidence for P1

**Trigger.** Any valid keyword/prose query with at least one unambiguous two-token NP modifier–head pair, or an unambiguous verb chunk adjacent to a nominal chunk. No `can`, `what`, or question-mark requirement. For a keyword noun run with ambiguous verb use, retain it as unresolved; no relation bonus. Support is determined from raw query only.

**Candidates and representation.** Use all already-admitted A/B rows before per-matcher record cuts, exactly as P1. Map query pairs to source occurrences in one original paragraph/list item, respecting field boundaries. First arm uses exact body occurrences only. A typed pair has the same words/order as the query plus compatible POS/chunk membership. NP–VP adjacency is a shallow occurrence relation, not subject/object truth. A pair spanning code, separate fields, clauses or ambiguous chunk alignment receives no extra evidence.

**Admission/order/span/stopping.** Admission, tiers, original scope penalty, card spans and fallback order remain baseline. For a future isolated P1 experiment use P1's existing proposed pair contribution `min(idf(a), idf(b))/(1+gap)`, at most once per selected pair, added before the record cut. Change **which pairs qualify**, not the equation or constants, in the POS arm. No second shortlist reranker. Return the existing winning block and stop through ordinary five-card assembly; no matching noun permits a new empty-page fill. This is an LLP0002 scoring departure if implemented, not a current change.

**Added information.** Porter joins word forms; the static mean loses local role/order; neither distinguishes an ordinary word used as a verb from the same spelling embedded in a noun compound. POS might discard accidental pairs and retain a genuine phrase. However P1 already captures adjacency, so the incremental signal could be zero.

**Population/ceiling.** The 20 original cut misses (seven never cases) are the appropriate first screen, not 89 potential gains. Some alternative wordings of other cases can also have cut barriers, so 20 is not a global any ceiling. The first Astra exact-pair proxy found only one especially discriminative witness among these 20; POS cannot assume more. No absent-hard-term or true zero-overlap repair follows.

**Failures/protection.** Technical noun stacks, imperatives tagged as nouns, participial modifiers, nominalized actions, and arbitrary keyword order can lose useful pairs. A negative statement can still receive topic evidence; do not suppress it. One tiny coherent wrong sentence may beat a broader correct passage. Stop if typed pairs do not distinguish gold from actual competitors beyond untyped positions, or if gains derive from admission drift.

**Cost/LLPs.** Candidate-local prose tagging initially; repeated CLI tagging cost may be unacceptable. A later occurrence projection uses LLP0001 publication/forget rules. LLP0002 must define tag ambiguity, exact pair grammar, field isolation and scoring. LLP0004 must define linguistic asset packaging. No scorer change.

##### Rank 2 — Full nominal anchors for P2 and keyword row/property requests for E3

**Trigger.** Query-only NP spans plus remaining ordinary/protected terms, without controlled question syntax. For example, `cache invalidation policy` may supply nominal spans, but the parser must not assume from the source alone which is the object or property. Enumerate a finite set of contiguous NP partitions from the actual tag/chunk output; retain all remaining words as obligations. Examples here illustrate the rule, not measured tag outputs.

**Two narrowly separated uses.**

1. **P2 ordinary-phrase binding:** extend identifier-only anchors to a multiword, exact, fully modified nominal phrase present in both original units. Keep the original same-record/same-section/two-unit bound, full content coverage, exact hard-term evidence, and distinct non-anchor contributions. Do not join units merely sharing `cache` when the query names `local build cache`. The same full NP is still only a referent hypothesis; unrelated examples of the same phrase are a falsifier.
2. **E3 literal lookup:** enumerate which query NP could bind a row key and which remaining span could bind an original header/property. Candidate rows come from exact object postings. Require a complete literal row/header/value witness within one existing block, including all qualifiers; multiple incompatible assignments mean unresolved. A compact keyword query can activate this without “what is … for …”. Nonliteral header mappings still require E1's independent source equivalence, so zero E1 declarations does not become semantic field recall.

**Candidates/representation.** Existing FTS posting intersections first, then exact original NP occurrences and structural row addresses. P2 reads indexed membership within the anchor record/section; E3 reads the actual table/key-value region. No all-corpus vector scan or general noun posting index. A head can help enumerate a candidate span, but only the whole phrase with its modifiers can complete the anchor. No pronoun resolution, aliases or cross-record joins.

**Admission/order/span/stopping.** P2 inherits the proposed partial-tier compound candidate behind single-block full matches, weakest-witness BM25 and one scope penalty, stable ties; choose/schedule the original constituent blocks using P2's fixed policy. Never score a union span as one card. E3 begins as shadow-only; a future append experiment inherits E3's at-most-one certified block after the baseline final page when room remains, original-query lexical ordering, unchanged baseline later calls. Exhaust structural bindings before selecting; no unique-nearest-header shortcut, no retry with a shorter noun phrase after rejection. These are separate experiments, not a blended ranking rule.

**Added information.** POS proposes phrase boundaries and potential nominal identity in ordinary queries, relaxing P2's identifier-only population and E3's template-only frontend. Row coordinates and repeated full anchors, not noun membership, supply new retrieval evidence. A no-POS contiguous-phrase enumerator is the essential control: if it finds the same valid bindings, POS is an optimization/hypothesis filter, not the cause of better recall.

**Population/ceiling.** Some of 35 coverage and 28 hard-locality original misses may have distributed exact witnesses; an absent hard literal cannot be repaired. The union of these buckets is only a screening domain, not a 63-case attainable ceiling. Original cut cases can benefit from binding competition too. Before a witness inventory, the only universal gold ceilings are 89 first and 29 any. E3's actual source-table intersection could remain zero despite more query parses. P3's four declarations/zero pairs does not establish a population for ordinary-NP joins. Section-only upside cannot qualify adoption.

**Failures/protection.** Same noun phrase can denote different versions/examples; `default cache` and `remote cache` must not unify. Generic nouns create high fanout. Header/value nouns do not prove a requested operation; verbs such as remove versus preserve, modifiers such as minimum versus maximum, units and direction stay required. An absent field and a field saying `no` are different. Stop if complete witnesses are absent, if anchors reduce to generic heads, or if row swaps cannot be distinguished.

**Cost/LLPs.** Two-unit joins may be quadratic for repeated phrases: use `(record, section, anchor occurrence)` intersections and coverage bitsets, not global all-pairs tables. E3 fetches rows only for anchor candidates. Declare any work cap and report cap abstentions; do not silently call truncated enumeration exact. LLP0001 needs source-span/structure tables; LLP0002 needs ordinary-phrase identity, compound admission and card scheduling; LLP0004 needs packaging. E3 can initially reuse existing row-parser metadata without new POS persistence.

##### Rank 3 — POS-preserving phrase obligations for E2/E4

**Trigger.** A query-only analysis contains exact protected anchors and identifiable ordinary NP/VP/modifier spans, even in a keyword query. This authorizes proposing a decomposition, not asserting a complete relation frame. If keyword order permits multiple attachments, retain the alternatives; semantic certification is unsupported unless the required binding is independently explicit.

**Candidates/representation.** E2's exact anchor intersections or E3's row bindings nominate original spans. Keep the existing static Encoder for any original-phrase comparison. Add a typed alignment ledger: noun phrase to nominal source phrase, operation to verb phrase, modifier to its original head occurrence, operator to its exact endpoints. Require injective obligation mapping. Compatible POS is a necessary screen in this research arm, never a synonym license. Cross-POS nominalization may be valid language; leave it unresolved rather than silently implementing `expire→expiry` again.

**Admission/order/span/stopping.** No new automatic cards in the first study. For a later E2 integration, preserve its exact anchor/operator and non-vacuous alignment/equivalence requirements, existing block witness and append-only ordering. For a later E4 integration, supported queries would validate before cuts and across later calls, keeping baseline lexical order among witnessed rows and forbidding uncertified fills; this is E4's already-proposed major stopping departure. POS alone does not authorize it. Unsupported queries retain baseline. No nearest-word retry after a failed obligation.

**Added information.** Can reveal that a candidate alignment consumes an object noun as though it were the requested operation, loses an adjectival qualifier, or reuses a span for two obligations. This is different from a stem match and different from comparing a pooled vector twice. `enable/disable`, `increase/decrease`, `read/write` can have identical POS and arguments: POS does not resolve them. Rule-based disambiguation is not factual validation.

**Population/ceiling.** Zero known complete new certificates here. Query support and original-source certificate counts must be separate. Cannot claim any of the six original zero-overlap cases, since candidates still require exact anchors. E2 append-only cannot fix full pages or existing all-call silence; E4 could affect those only after a sound support/certificate policy exists. Arithmetic headroom is not evidence.

**Failures/protection.** Wrong-sense same-POS synonyms, passive voice, coordination, negation attachment, exceptions, conditions, quoted examples and ambiguous keyword speech acts. Preserve explicit `no/not/without`, quantities and hard literals independently of linguistic tags. A negative answer about the requested topic remains relevant. Falsify with same-word role swaps and same-POS antonyms; if tags merely increase parser activation while every certificate still fails, report no retrieval evidence.

**Cost/LLPs.** Candidate tagging plus E2's bounded phrase matrix, not a new embedding model. No phrase vocabulary built at startup. LLP0002 must specify ambiguity and what can be certified; LLP0001 only if persisted; LLP0004 for resources. LLP0005 floors and scorer stay unchanged.

##### Rank 4 — Context-sensitive ordinary-term roles alongside fixed KIND

**Trigger.** A KIND-listed token participates in a supported NP/VP span. The baseline excludes these words from coverage regardless of context while retaining score contribution. POS may reveal that `memory` modifies `allocation`, `version` is a requested property, or `command` is the head of the sought object. Conversely a bare noun in “command for …” may still be a request cue. All are nouns; tag identity alone is insufficient.

**Candidates/representation.** Keep baseline candidate generation and fixed KIND untouched in the first diagnostic. Store a parallel query-only role annotation: requested object/property, modifier, operation, or unresolved/request cue. Use these tokens in the source-binding obligations of mechanisms 1–3, retaining every other query constraint. Candidate evidence comes from their original postings; no new lexicon.

**Admission/order/span/stopping.** Initial arm changes no page: report where the contextual role would differ from KIND. A future tested version may require a KIND noun when it belongs to a complete source-bound phrase, under the relevant P2/E3 admission; it must not weaken requirements for other terms. Rank/span/stop inherit that mechanism. A wholesale dynamic-KIND replacement or “all nonnouns optional” is not specified or recommended. Any candidate changing `content`/`need` is a separate LLP0002 retrieval decision, not a parser fix.

**Information, ceiling and falsifier.** Helps preserve intent that current weak-term coverage misses; can improve discrimination but can also reduce recall. It does not manufacture absent `tools` or infer Nix. No measured affected population or defensible POS-specific gain count. If annotations can be reconstructed from the existing KIND list and word order alone, or only succeed by deleting verbs/modifiers, POS adds no useful information. Measure successful and no-gold controls before considering coverage changes. Cost is query-local tagging plus normal candidate postings; reuse the same fingerprint/projection contract.

#### 4. Projection and execution design if research survives

Start transiently in a standalone ignored diagnostic. Do not open the parent product store through a path that might migrate/publish it. Do not alter the active agents' examples, Cargo files, reports or scratch databases. No NLP tables belong in the product until the mechanism shows a population and the LLP decision is made.

If source tagging becomes useful, tag query-independent original units on writes and persist numerical/structural metadata in **the same libSQL file**. An illustrative layout, not a schema change in this pass:

Indexes should lead with `(block, unit, occurrence)` for candidate reads and `(record, unit)` for bound joins/deletion. Existing libSQL term indexes nominate blocks and supply exact term frequencies/positions on demand; do not duplicate the corpus vocabulary into a POS search engine. Only if a validated mechanism requires direct anchor-to-unit traversal should an index map an existing lexeme ID to source occurrence IDs; that is source provenance, not a second lexical text column. Do not persist every possible word pair; store unit membership and selected chunk edges linearly, join queried occurrences at runtime.

Publish against the exact structural publication and body identity with the same writer-lock/external-version checks as LLP0001. Invalidate on original record/section writes, rebuild and tokenizer/grammar changes. Forget must remove occurrence/chunk/link rows and cached source diagnostics containing forgotten content in the same secure-delete lifecycle; stale publications cannot serve evidence. A tagger upgrade invalidates linguistic metadata, not immutable bodies or the existing static vectors. Normal open verifies manifests and loads no corpus-wide linguistic map. Query work is proportional to touched postings and candidate bytes/pairs; generic anchors remain a worst-case risk.

Future durable decisions: LLP0000 explicitly permits these pinned linguistic assets within one native binary; LLP0001 defines metadata/publication/forget; LLP0002 defines role semantics, matching and any changed ordering/stopping; LLP0004 defines offline embedding, notices and freestanding checks. Follow LLP0003's repository-only citation rule when those decisions are actually made. LLP0005's scorer, floors, split and confirmation policy do not change. Research-paper links belong in this report, not new external LLP citations.

#### 5. Minimal decisive experiment sequence

These are **future experiments**, not executed results. Do not spend a full candidate score on a zero-activation parser or a noun-count proxy.

1. **Resource/API audit.** Once dependencies are available through an authorized offline environment, run the retained corpus-free probe against the cached asset. Record resource hash/version, chunker presence, rule count and actual tag vocabulary. Compare dictionary lookup, post-chunk/pre-disambiguation, and full-pipeline analyses. Calling the dictionary API alone cannot evaluate contextual POS. A genuine lookup-plus-rules/no-chunker ablation needs a separately built resource/API arrangement and verification, because `sentencize` already chunks; do not label that call “rules-only.”
2. **Freeze query-only support before relevance labels.** Read only q4 DEV query strings and ordinary context, selecting DEV by the unchanged hash split. The diagnostic boundary excludes gold, must_not, category and alternate lists from the parser. If wordings are studied, feed each separately as an ordinary query, with evaluation keys held outside the parser. Save raw-offset spans, all tag alternatives, missing chunk labels, protected-token masks, partitions, unresolved operators and support/failure reasons. Do not normalize fragments into questions or inject a title/subject. Count originals and all wordings separately.
3. **Actual ambiguity/OOV audit.** Before attaching relevance labels, choose a deterministic sample covering short keywords, ambiguous noun/verb spellings, NP stacks, proper names/OOV, identifiers, Unicode/hyphens and full sentences. Have human research inspection label token roles and acceptable alternate analyses from the query alone. Acknowledge irreducibly ambiguous queries; do not force one gold POS. Report token coverage, multi-analysis rate, chunk-alignment gaps, protected-span boundary errors, head-selection errors and support precision by stratum. A correctly tagged noun is not a relevant-query anchor.
4. **Source evidence after parser freeze.** Use indexed candidates or saved G candidate provenance and q4 JSONL original bodies. Tag candidates without relevance labels; save every proposed binding and rejected obligation. Only then attach baseline outcome, gold, no-gold subtype and section targets for diagnosis. Distinguish six stages: parser activation → indexed candidate → complete source witness → lexical admission → page eligibility/order → legal shown span. Count zero at each stage honestly. Inventory both missed and already-successful queries; include all target-required and true-silence controls, not just favorable failures.
5. **Ablate the actual new information.** Baseline fixed KIND/A/B; P1 untyped ordered pairs; same P1 with POS-selected pairs; P2/E3 contiguous-phrase partitions without POS versus POS partitions with identical source constraints; E2/E4 obligations without versus with POS types. A relation-disabled executor must reproduce all baseline pages. Keep match provenance, equation, candidate budget, ordering and spans fixed between the causal pair. Do not compare a POS arm with relaxed hard terms against an unchanged baseline and attribute the result to POS. No lemma side column or closed nearest-neighbor rerun is needed.
6. **Run counterfixtures as falsifiers.** Test the unchanged literal `E_INDEX-42` versus `E_INDEX-43`; `--no-cache`; quoted phrases; `react-navigation`; camelCase; minimum/maximum and 10/100; from/to direction; remove/preserve; `local build cache` versus `remote build cache`; repeated identical words with distinct roles; NP attachment alternatives; composed/decomposed Unicode and curly apostrophes; code containing English keywords. Test `Alice follows Bob` versus its reversal, row/cell swaps, wrong-version rows, omitted properties, affirmative/negative answers, `not only`, unsupported coordination and source statements rejecting the query premise. Tags must neither erase constraints nor turn a supported negative answer into an irrelevant result.
7. **Only a surviving witness population earns changed-page work.** Reproduce baseline with the unchanged Rust scorer and all 1,470 ordered wording pages before changing retrieval. Execute one predeclared DEV candidate through the native harness, not Python scoring. Report all six numerators, complete paired pages, gains/losses, original versus alternate-only versus never-recovered cases, exact card spans, and all-call no-gold subgroups. Audit whether a claimed new hit came from new evidence rather than accidental dropped constraints or a changed fallback route.
8. **Cost and adoption remain separate gates.** Measure cold asset load, ingest/projection cost and disk growth, store opening, first-use and resident queries; retain unsupported/capped-query counts. Do not claim full neutral validation from fixtures. Future candidate constant selection may use DEV as authorized by the parent, but benchmark/adoption gates stay unchanged and holdout cannot guide tuning. A selected frozen candidate must raise at least one of the four retrieval numerators with none falling and neither empty nor section falling; the parent's later one-shot h2 confirmation has the same rule, flat means hold, fall means reject. **No h2 run or inspection is requested/performed here.**

#### 6. Synergy with the ongoing work, and stop conditions

For **P1**, the integration seam is an occurrence relation selector before the matcher cut. Require a competitor pair where POS changes the supported relation while lexical vocabulary/positions and the executor are held fixed. If plain P1 already captures the relation, ship no tagger on that rationale.

For **P2**, the seam is full ordinary-NP anchor identity alongside exact identifiers. Require at least one new, complete, non-spurious two-unit witness and a viable existing-block schedule. More noun anchors with no legal joined evidence is only activation. P1's preference for local coherence can conflict with genuinely distributed P2 evidence; evaluate their interaction after each survives, not by adding their separate score gains.

For **E2**, the seam is contiguous phrase segmentation and typed obligations. Require multiple meaningful alignments and independently grounded source constraints. A POS-constrained unique pair can still be uniquely wrong. It cannot solve genuine no-anchor paraphrase or supply missing equivalence declarations.

For **E3**, the first worthwhile seam is query parsing for literal row/header joins. Require that a compact keyword request reaches a real row, the correct field and a legal block, and that a no-POS partitioner cannot do equally well. Only later consider semantic headers with independent proof. Do not interpret a source value noun as the queried property by convenience.

For **E4**, retain three states and distinguish parser support from certificate support. POS may make obligations visible; it does not justify suppressing all current cards on ambiguous keyword queries. Before any all-route validation, show a nonempty interaction where a new generator supplies evidence the validator requires and the validator rejects a genuinely wrong binding while preserving correct negative evidence. Generator-only, validator-only and joint shadow outcomes must be retained. Zero E4 query activation under the old grammar does not validate a broader grammar's precision.

Do not attempt noun-only deletion, noun-only vectors, POS-based global rarity bonuses, a generated headword field, all-nouns-as-hard terms, longest-NP-as-answer, rightmost-head subject joins, a dependency parser concealed inside chunking, inferred synonym tables, or a new learned reranker. Do not reopen lemma, stems, identifier release/splitting, cosine/beam, tree-coverage relaxation, confidence, duplicate replacement, or rejected A eligibility work. Do not fit grammar productions to gold cases or promote parser coverage to search success.

Missing proof is specific: runnable pinned resources; actual technical-keyword tag reliability; unique role information beyond order and KIND; a nonzero source-witness population; correct treatment of ambiguity and negative answers; paired-page gains without floor losses; affordable fresh-CLI and write costs; verified redistributable offline packaging. Until those are established, preserve the measured product and an honest miss.

#### 7. Primary references and actual conclusions

- **nlprule 0.6.4 documentation and source**, [API](https://docs.rs/nlprule/0.6.4/nlprule/), [pinned source tree](https://github.com/bminixhofer/nlprule/tree/ebb54617ea5f3378c0dc63486631724b7e97e13c). Verified lookup analyses, contextual rule modification, public spans/chunks and the separate statistical chunk pipeline. No dependency-head or relevance guarantee follows. Detailed source/asset references appear above.
- **Christina Lioma and Iadh Ounis (2006), “Examining the Content Load of Part of Speech Blocks for Information Retrieval,”** [primary paper](https://aclanthology.org/P06-2069.pdf). Tests POS-block query reduction, including an open/closed-class content criterion, on narrative text; effects differ by collection/model. Its exact figures and transfer limits are stated in §2. This is outside evidence, not a Linn POS measurement.
- **Chengxiang Zhai (1997), “Fast Statistical Parsing of Noun Phrases for Document Indexing,”** [primary paper](https://aclanthology.org/A97-1046.pdf). Learns NP structure and supplements single-word indexing with parsed phrases; combinations trade precision and recall in its own retrieval setup. It does not validate nlprule chunks as heads or allow discarding nonnouns.
- **Rob Koeling (2000), “Chunking with Maximum Entropy Models,”** [primary paper](https://aclanthology.org/W00-0729.pdf). Uses local words, POS tags and surrounding phrase tags to predict chunk labels. It supports describing chunking as statistical sequence labeling; it is not dependency parsing, entailment, or a measured search gain. nlprule links this paper in its chunker documentation; that link does not establish identical training data/weights for the cached English asset.

No literature number is imported as a Linn gain or accuracy/latency estimate. No source says that the presence of a relevant-looking noun proves that a card answers the query.


<a id="consolidated-evals-pos-review-astra-md"></a>

### evals/POS-REVIEW-ASTRA.md

Review of the partial audit at fb7b2ae, before retrieval integration. Its finding that POS was absent from retrieval is historical and was superseded by the completed continuation, recovery and focused runtime.

Research review, 2026-11-08. Reviewed POS code commit `fb7b2ae543ccbb33780539a2ccf6c832a8cd75e5` in `<historical-checkout>`; parent is `129897f950208faeefaa43ddd10f55537e46c092`. Worked alone. This report and ignored `evals/output/astra-pos-review/` diagnostics are the only intentional writes. No product implementation, adoption, commit, push, holdout inspection/run, corpus modification, source_path access, neutral/hybrid/scale run, or messages to the POS thread.

#### Answer

**Was POS used effectively for retrieval? No: it was not used in retrieval at all.** It was used productively to establish runnable resources, expose actual token/chunk behavior, preserve original-byte slices, and construct counterexamples. Those are useful prerequisites. They do not complete any of the three requested retrieval mechanisms. The branch's full `evals/RESULTS-POS.md` accurately calls itself an incomplete handoff; its baseline parity is not a POS/no-POS experiment.

**Are there more useful ways to use it? Yes.** The best immediate experiment is a query-and-source relation selector inside the existing experimental P1 primary matcher, before candidate cuts. The next is a factored representation of original occurrences and groups, with explicit assignment constraints rather than another whole-phrase mean. Whole ordinary-NP identity and keyword-to-field partitions are separate, conditional extensions. POS is most plausibly useful for deciding which occurrences belong together and which matches are compatible, not for inventing synonyms or deciding that a proposition is true.

The user's additional suggestion—index individual keyword embeddings and detect overlapping subgroups—is mathematically well founded as a **representation and matching problem**. It deserves a constrained set-matching experiment. However, neither independent nearest-token matches nor injectivity alone is new here: both have close closed predecessors. The untested part must be original occurrence identity, source-unit membership, group boundaries and relation bindings, with embeddings evaluated separately from those structural contributions. Exact subgroup overlap can be certified; semantic relevance cannot be certified from cosine alone.

#### Evidence and prioritized findings

All `POS:` references below mean the **child checkout at the reviewed commit**, not similarly named parent files. Links point there. The uncommitted report and ignored diagnostics were inspected as retained evidence, not as committed guarantees. I inspected the actual diff including Cargo.lock, the complete POS report/library/example, preparation script, saved query and fixture analyses, coverage/cost logs, relevant P1/P2 reports and implementation seams, the parent proposal/results context, governing LLPs, and closest closed FINDINGS mechanisms. I did not rerun the full baseline or development scorer.

##### 1. High: the requested causal retrieval experiment is absent — unfinished scope

POS: evals/src/lib.rs:1 (historical child source) explicitly says the library is never consulted by product search. POS: pos_groups.rs:23 (historical child source) analyzes input and prints JSON. Its only consumers are diagnostics/tests; there is no source annotation read in the primary matcher, no POS-selected occurrence contribution, no new admitted row, no ordinary-NP join scheduler and no field binding frontend. POS: LLP0005:368 (historical child source) appropriately records only an experimental audit.

Consequently all 1,470 baseline pages remaining unchanged establishes that the baseline is reproducible, not that POS is ineffective. Zero changed-retrieval configurations were run. The most important missing deliverable is one complete query → candidate → source witness → admission → primary competition → original-card execution, with a matched control.

The concrete P1 seam already exists in the other experimental checkout: P1: src/search.rs:237 (historical child source) adds the occurrence contribution to the primary memoized score before grouping/cuts. Changing its query/source pair selector is a small, reviewable experiment. Appending the POS output after a page is built would test a different and heavily intercepted mechanism.

##### 2. High: exhaustive contiguous groups are not meaningful POS proposals — design gap

POS: evals/src/lib.rs:49 (historical child source) enumerates every token interval within each sentence, regardless of POS. The actual linguistic use begins at line 60, where public chunk labels create additional groups. Public lemma/POS analyses are stored but never interpreted by a nominal/verb/modifier compatibility function. Thus “POS groups” currently combines two distinct inventories: untyped O(n²) spans and a much smaller set of predicted chunks.

The query output contains 15,527 untyped contiguous spans plus NP/VP/etc. spans; the source summary totals 6,383,242 groups. This is broader than bigrams, but breadth is not a linguistic selection strategy. A chunk duplicates an already enumerated interval with a kind; the embedding matrix re-encodes both. There is no deduplication by original span, no subgroup selection inside long NPs, no protected-identifier mask, no attachment alternatives, and no source-unit relation graph.

The saved actual analyses make this consequential:

- `EAS Update incompatible updates channel compatibility` is one entire NP-plural. The useful P1 pair `incompatible updates` is merely an untyped subspan.
- `changeset docs test fixtures` is one entire NP-plural too. Whole-NP membership cannot distinguish why untyped P1 gained on the former and lost on the latter.
- `EAS workflow YAML validate` is one NP-singular, despite the operation-like last word.
- `Metro scheduled shared cache build options` contains a predicted VP `scheduled shared cache build`. That is not evidence of a correctly interpreted technical operation.

These are from saved raw dev query analyses, summarized in this review's `audit.json`. They were not new relevance runs. A useful selector needs inner nominal/modifier pairs and explicit ambiguity handling; requiring every keyword query to match a whole source NP is likely too restrictive. Conversely, accepting every pair inside a giant predicted NP barely changes P1.

##### 3. High if promoted to a verifier: witnesses are literal matching, not POS-aware role binding

POS: evals/src/lib.rs:101 (historical child source) greedily finds equal original group strings, optionally requiring equal chunk-kind strings. It advances a source byte cursor, which prevents source-span reuse in that returned sequence. It does not bind subject/object, modifier/head, negation scope, values or directions. It returns partial pairs without a complete-obligation result. Alternatives are not represented as alternative analyses; future overlapping groups could compete with one another as if all were required. Exact `NP-singular` versus `NP-plural` is not a semantic type system.

POS: literal_support:201 (historical child source) is an ordered token-subsequence check. It never reads POS or chunks, has no gap limit or sentence/unit identity, and permits arbitrary intervening source tokens. In a fresh execution of the already-built example:

- query `cache policy`, source `cache. Unrelated policy.` → `Literal`;
- query `not cache`, source `not unrelated text. cache works.` → `Literal`;
- query `build cache`, source `We build a cache.` → `Literal`.

This conforms to its narrow occurrence-coverage behavior. It becomes a defect only if used as a locality, relation or semantic certificate. Rejecting `Alice follows Bob` against its reversal is accomplished by string order without POS; it does not validate POS roles. The existing negative-answer fixture correctly remains a literal lead, not a claim that its premise is true.

POS: shared_np_anchors:132 (historical child source) preserves the full modified string and returns separate offsets, a good narrow property. But it is only an all-pairs equality helper, invoked by a synthetic test, not an executed ordinary-NP retrieval/join path. It lacks source record/section identity, exclusive non-anchor contributions, lifecycle checks, scheduling and the proof that both original witnesses can actually be shown. None should be inferred from its name.

##### 4. Medium: resource-faithful labels are not necessarily faithful linguistic groups

The corrected extractor handles the important mixed convention: NPs have explicit E labels, while VP and other chunks use BIO. A singleton NP can have both B and E. The earlier zero-VP output was an extractor error that was fixed, not an experimental negative. I found no basis to claim the final normal NP/VP label scan still has that same error.

The actual pipeline is richer, and less definitive, than dictionary lookup: public tags are lookup analyses subsequently changed by contextual rules; a separate statistical POS sequence feeds statistical chunking. Those internal best POS tags are not exposed as the public tag list. Chunk attachment requires matching internal/main token character spans. Missing attachments cannot be interpreted as “not a noun.” The local upstream implementation at `ebb54617...`, `tokenizer.rs:302–376` and `tokenizer/chunk.rs:704–802`, was inspected; the [official pinned chunker source](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/nlprule/src/tokenizer/chunk.rs) and [0.6.4 API overview](https://docs.rs/nlprule/0.6.4/nlprule/) were also checked. These are linguistic resources, not a prohibited extra semantic embedding model.

Concrete saved fixture limitations: bare `Bob` in `Alice follows Bob` receives `O`; `local to` becomes an NP inside the direction query; `enable cache` becomes an NP even though `enable` has only public verb analyses; the quoted `local build cache` loses labels on `local` and predicts `build` as VP. Adding a period in this review's probe changes the final Bob chunk. The Unicode possessive fixture has unmatched chunk positions and an orphan NP continuation. These results do not justify blindly aligning public nominal tags with a single internal chunk decision.

Offsets are checked correctly against original bytes in `analyze` (line 35), including Unicode. That proves slicing, not correct attachment. Record sentence/unit IDs explicitly; retain incomplete chunk status rather than silently treating every surviving BIO fragment as a complete relation. A future nominal mapping must include actual tags such as `NN:UN` and `NN:U`, not just `NN` or every string beginning with N. Number decoration is not a reliable identity constraint.

##### 5. Medium: the ambiguity metric overcounts linguistic ambiguity — measurement issue

POS: pos_groups.rs:27 (historical child source) counts more than one nonempty analysis, including `UNKNOWN` and `SENT_END`, and counts analyses rather than distinct POS values. Recomputing the saved query output gives:

- 5,825 token occurrences; 3,737 with multiple nonempty analyses, reproducing the report;
- 3,090 with multiple distinct tag strings after removing `UNKNOWN` and `SENT_END`;
- 612 occurrences carrying `UNKNOWN`; 157 with no attached chunk labels; another 184 with only `O`.

The adjusted number is still not a gold POS ambiguity score: tag subtypes and lexical alternatives require an explicit mapping, and a surviving ambiguity can be legitimate. The report's raw count is reproducible; interpreting it as the proportion of uncertain POS decisions would be wrong. The pipeline test's `analyses.len() > 1` assertion (line 257) is even weaker because empty analyses are included.

##### 6. High for downstream use: the source audit has no retrieval-addressable provenance — unfinished scope

POS: prepare.py:9 (historical child source) emits nonblank `body.splitlines(keepends=True)` units containing only text and `summary:true`. It removes record identity, body hash, original base offset, section/block identity and region type. POS: pos_groups.rs:24 (historical child source) then emits counts and skips all witness/matrix processing. The retained source output is not an indexed source-group inventory usable by a matcher.

Line units avoid cross-line groups but break wrapped prose and ignore fenced-code state, table cells, list structure and heading roles. Python `splitlines` also has a broader separator interpretation than the specified byte-line scanner. No byte slicing bug was demonstrated in the isolated line strings; the missing property is a mapping back to immutable original-body coordinates. Repeated identical lines cannot be resolved from summary counts. A source pipeline must carry `(body hash, publication, record, block, unit, base byte offset)` before tagging, without copying filenames or ancestor headings into content.

##### 7. Medium: test strength and work costs do not establish a product-ready feature

The two default library tests use hand-created groups. The optional resource test asserts slice validity, some ambiguity, one VP and literal reversal rejection; the code/fence input is not checked for a no-cross-fence invariant. The example pins the asset SHA, whereas the optional test only checks chunker presence and rule count. Saved logs establish successful ordinary checks and a separate three-test resource run; they do not establish real keyword accuracy, field binding, group assignment, publication/forget correctness or retrieval improvement. I did not rerun the workspace suite for this document-only review.

The asset was independently rehashed here: 11,590,973 bytes, SHA256 `e2b51e506aede495268ee05dee0b97956e768107f5bb5891585144a5ff2fd773`. The report's provenance matches the official release URL and retained release metadata; a locally computed digest is not an upstream signature. Dependency is eval-only `nlprule =0.6.4`, defaults disabled, `regex-fancy`; Cargo.lock adds that stack rather than replacing the existing embedding library. Root Cargo.toml is unchanged. No product resource packaging exists. Upstream distinguishes crate licensing from the distributed linguistic binaries; see its [binary provenance statement](https://github.com/bminixhofer/nlprule/blob/ebb54617ea5f3378c0dc63486631724b7e97e13c/binaries/README.md). Distribution details remain a packaging task, not proof of a runtime blocker.

Saved debug logs: query audit 3.52 seconds total, 1,809 ms load, about 159 MB peak RSS; source process 146.25 seconds total, 143.23 seconds summed per-unit analysis, about 169 MB RSS. They are not release fresh-CLI measurements. In POS: pos_groups.rs:35 (historical child source), Encoder loads inside each embedding-enabled input iteration. Matrix timing therefore includes repeated model loading as well as extraction/encoding; it cannot estimate a cached matcher cost. Every source group encoded at 512 float32 dimensions would cost about 13.1 GB for 6.38 million rows before metadata—an arithmetic warning, not measured storage. Do not persist exhaustive spans.

#### What the three requested ideas and embedding countertest actually establish

1. **POS-selected primary lexical occurrences:** not implemented or tested. Group generation is not admission or ranking. P1's untyped weights 1 and .25 lost gold; .05 was flat despite 39 changed first pages. A new POS selector must explain what it removes or adds relative to those actual pair contributions, with the same candidate set/equation. “More positions” has already failed its declared study.
2. **Whole ordinary-NP joins and compact field bindings:** helper scaffolding only. P2 had five exact-identifier proofs and no page changes at three weights. Every combined P1/P2 page equalled its P1 control. Ordinary modified-NP identity can broaden a genuinely different source population, but only if complete complementary witnesses and legal card capacity exist. P3's zero source pairs, E1's zero declarations and E3/E4's zero controlled-query activation are different failure stages; a broader POS parser fixes none automatically.
3. **Typed group obligations/embedding alignment:** matrices exist; a complete alignment/validation policy does not. The 14 fixtures include useful technical strings, modifiers, quantities and negative answers, not just SVO toys, but only one unchanged-phrase positive and no representative positive paraphrase/field-binding retrieval population. They are adequate falsifiers of a strong semantic claim, inadequate estimates of utility or precision.

The matrix implementation is legitimate as far as it goes: each original contiguous query/source group is encoded through the unchanged Encoder and float32 dot kernel. It does not secretly average normalized group vectors. But no matrix entry is used to match a candidate or choose a card. No partial-order assignment, overlapping-group conflict resolution, role-sensitive score or source-bound ledger consumes the matrix.

Whole `Alice follows Bob` / reversal and direction reversal have cosine 1.0. For the same untruncated token multiset, a static pooled representation necessarily loses permutation information, aside from insignificant arithmetic effects. This falsifies **using that pooled similarity as a role certificate**. It does not falsify separate role-indexed vectors plus explicit ordered source constraints. An unordered set of singleton vectors also has the same reversal problem: injective matching can perfectly swap Alice and Bob. Order/binding must live outside cosine.

The saved increase/decrease cosine (~.866), wrong-identifier cosine (~.869), and negation control (~.903) show that high similarity is not equivalence. They do not by themselves test all possible thresholds; the stronger E2 reciprocal-opposites result already closes that proposed automatic synonym certificate. No margin tuning should conceal it. Conversely, this review does not require a mathematical relevance proof before any primary lexical research: structural evidence may improve ranking empirically while remaining fallible. It must be described and evaluated as such.

#### Individual embeddings and mathematical subgroup overlap

**Use occurrences and groups, not a deduplicated bag of keyword strings.** One source occurrence must not satisfy two independent query obligations. Two occurrences of `cache` can belong to different clauses or arguments. A whole group and its constituent words are alternative resolutions of the same bytes, not independent pieces of corroboration. “Keywords” here means indexed original tokens/spans; it does not mean generated keyword enrichment or noun-only text.

Let query occurrences/groups be Q and candidate-source occurrences/groups be S. For each original slice compute `e(x) = Encoder.encode(original[x])`; cache exact resulting float32 rows, without a new pooling rule. Define `C[i,j] = dot(e(q_i), e(s_j))`. Add a separate admissibility relation A carrying literal/protected identity, possible POS compatibility, source region, and any explicitly supported role endpoints. C is similarity; A describes allowed bindings. Neither is an oracle relevance label.

Three useful mathematical levels should be kept distinct:

**Exact occurrence intersection.** Existing term postings enumerate matching occurrences, then grouping restricts them to a shared unit/phrase. Bitsets or sorted posting intersections can establish precisely which query obligations have exact witnesses. This can find a small relevant subgroup inside a large block without dilution by the block's other words. It is the strongest immediate certainty available and requires no embedding. Apply it before cutting candidates, preserving the existing hard/coverage rules in the first experiment.

**Capacity-constrained subgroup assignment.** Use binary variables `x[i,j]`, only for allowed edges. Each query obligation has at most one assigned source group; each source occurrence has capacity one. Overlapping source groups conflict by original occurrence membership. Record unmatched obligations explicitly. A full matching must cover every required obligation; partial subgroup overlap must never silently become full coverage. Maximize a declared sum of edge similarities only to propose an assignment, with byte-order ties, not to certify synonymy. A bipartite matching establishes a finite combinatorial property; simple independent MaxSim does not. Once interval overlap and inter-group role constraints are added, this is no longer plain bipartite matching in general. Restrict the first implementation to nonoverlapping partitions and monotone alignment, solvable by dynamic programming; do not promise cheap exact arbitrary hypergraph matching.

**Group/role consistency.** A query NP's members must bind inside one compatible original source group; several query groups must retain their explicitly known relation. For an ambiguous keyword bag, retain an unordered alternative rather than inventing subject/object order. For a supported `from X to Y` frame, preserve the two endpoint bindings and exact connectors. Permuting source arguments should then change the relation test while leaving the raw embedding matrix's best unordered score unchanged. This is where new information enters. The benefit comes from the conjunction of assignment and original structural constraints, not the theorem that a matching exists.

There are important mathematical limits:

- Averaging pairwise similarities over all query/source tokens collapses to a dot product of aggregate vectors; it reintroduces mean dilution and contributes no independent group reasoning.
- Independent row maxima can reuse one hub occurrence for many query terms. An injective assignment fixes reuse, but can still match antonyms and swap roles.
- Balanced optimal transport forces all source mass to participate and can penalize a useful small subgroup inside a long source. Partial/unbalanced transport permits unmatched mass, which needs explicit obligation accounting; otherwise it solves the problem by dropping the difficult query term. Fractional mass is also not an original occurrence witness.
- A perfect same-word assignment can be confidently about the wrong fact. Neither a cosine margin, uniqueness nor agreement with POS is a calibrated probability of relevance. Do not multiply correlated signals as independent evidence.
- General subgroup enumeration is exponential; all contiguous groups are only quadratic, yet their cross-matrix is already O(m²n²) before encoding cost. Bounded source units and sparse admissible edges matter more than a clever global index initially.

**Index design:** use libSQL lexical postings to nominate eligible blocks, then load occurrence/group metadata for those blocks. Cache singleton encodings by exact original surface bytes plus model/encoder identity; occurrences point to the shared row while retaining independent positions. Cache only selected original group vectors, not all subsets. If the source surface string is identical, static embedding reuse is valid; POS metadata stays occurrence-specific. Do not equate these encodings to raw wordpiece-table rows: one surface word may tokenize into several pieces. Do not reconstruct phrase encodings by averaging these normalized singletons.

This first design cannot recover a genuinely zero-anchor candidate outside the lexical pool. A later semantic candidate index would need a separate, explicit experiment: exact scans or a proven conservative pruning scheme over stored float32 group rows, followed by original-source obligations. Global nearest-word postings alone revive closed expansion. ANN, centroid quantization and top-k truncation would alter recall and are neither required nor justified for this first test. Upper bounds can prove that an unseen candidate cannot beat an assignment score, but not that the current winner answers the query.

Relevant primary research checked for this addition: [Word Mover's Distance, Kusner et al., 2015](https://proceedings.mlr.press/v37/kusnerb15.html) formulates word-embedding document distance as transport; it does not supply a relevance certainty guarantee. [a late-interaction retrieval model, Khattab and Zaharia, 2020](https://arxiv.org/abs/2004.12832) establishes fine-grained query/document interaction, but uses contextual BERT representations unavailable here. [a late-interaction retrieval method, Santhanam et al., 2022](https://arxiv.org/abs/2205.09707) shows how candidate pruning accelerates that architecture; its centroid mechanism and empirical performance are not transferable guarantees for Linn. These are design references, not proposed dependencies or evidence of Linn gains.

Closest CLOSED trials are decisive: FINDINGS lines 2035–2041 tested a static-token minimum gate that kept literal noise and removed paraphrase; lines 2427–2439 tested distinct-piece embedding matches that found the wrong forms. Thus “individual vectors + injectivity” alone is already insufficient. The new arm must demonstrate a decision changed by coherent source subgroup or relation membership, holding edge similarities and candidate population fixed. Otherwise stop rather than rename the closed experiment.

#### Ranked next mechanisms

##### 1. Query-and-source selected lexical relations before cuts

**Algorithm:** raw query analysis with protected literals → existing eligible A/B candidates → exact occurrence pairs retained only when query and source both support a declared local group relation → unchanged P1 bonus equation → existing tiers, penalty, stable ties, cuts/interleave → existing original block cards and stopping. First freeze a small selector: adjacent ordinary occurrences inside an NP with compatible nominal/adjectival analyses, and separately adjacent verb/NP-boundary pairs where public analyses and chunk transition support that proposal. Exclude unresolved operator crossings, opaque identifier interiors and code attachment. No inferred dependency head. Ambiguous analyses produce a possible relation, explicitly marked; they must not erase query obligations or certify facts.

Start with the NP selector alone for the smallest causal arm; verb relations are a separately declared extension, not an opportunistic fix after seeing losses. Preserve all query terms for baseline admission, including terms outside selected pairs. Retain short operators in the annotation even when baseline content normalization omits them.

**Why new:** P1 rewarded every eligible adjacent pair. This gates the same equation on query **and source** structure. Query-only selection cannot distinguish the same spelling used differently in candidates. Source-only selection can reward a relation the query never requested. Whole-query NP selection alone is too coarse, as the saved real analyses demonstrate.

**Population/ceiling:** the 20 original cut-barrier misses are a relevant screening set; 114 all-wording cut misses are pages, not cases. An admitted-candidate-only experiment cannot fix missing hard evidence or absent lexical candidates. P1's actual gain `incompatible updates` had raw rank 9→5 in both matchers, while changeset/workflow/tool-installation examples lost. Audit the pair contributions in both gold and their actual competitors; do not encode those case IDs or bespoke word rules. If the selector leaves the beneficial and harmful pair populations indistinguishable, the hypothesis has no established leverage.

**Cost/falsifier:** indexed candidate occurrence reads plus linear stored annotation. Reject attribution if gains come from different coverage, tokenization or scope. Stop if identical selected pairs reproduce P1, or if no new candidate ordering survives the no-fall rule. Compare untyped, query-only, source-only and both-side selection with identical equation/weight at the diagnostic level; score only the predeclared meaningful arms.

##### 2. Factored original groups with constrained assignment

**Algorithm:** same query/source annotations → anchor-indexed candidate units → singleton and selected NP/VP/subgroup vectors → sparse assignment respecting protected literals, occurrence capacities, unit boundaries and explicit relation endpoints → source-bound evidence ledger → unchanged legal block spans. For the first experiment, lexical admission and ranking remain mechanism 1's; vector edges are shadow proposals with complete/unmatched obligation accounting. Score no automatic semantic configuration until a nonliteral edge has an independently specified admission basis. Exact identities and actual source bindings are usable now; POS-compatible cosine alone is not.

Compare (a) no groups/independent matching, (b) groups without POS, (c) POS-selected groups, and (d) the same admissible assignments with vectors disabled and literal identity only. Use the actual Encoder, including positive multiword matches and wrong-role, repeated-token, same-POS opposite, wrong-version and disjoint-sentence controls. If vector scheduling exhaustively visits the same exact witnesses and produces the same page, report a possible cost benefit, not an embedding retrieval gain. If no independent source license exists, the semantic arm ends in a quantified unverified population, not another purported synonym certificate.

**Why new/ceiling:** explicit source subgroup coherence and external role constraints differ from token minimum/identifier-piece matching and E2 reciprocity. They can retain a small useful facet in a long block without whole-block mean dilution. There is no measured new certificate count or gain ceiling yet; exact-anchor candidates exclude genuine no-overlap misses. The model cannot resolve same-POS opposites, query intent, implicit negation or ambiguous attachment. Keep this bounded: a runnable assignment and causal diagnostics are valuable; a second all-corpus matrix dump is not.

##### 3. Ordinary full-NP joins with joint card scheduling

**Algorithm:** query full modified NP → existing term postings → same-record, same-section source units sharing that entire exact NP → two distinct non-anchor contributions and full original hard/content coverage → P2 partial-tier/weakest-witness competition with one penalty → actual constituent blocks scheduled within five cards/three calls → reject unschedulable pairs, no synthetic union credit. Keep single-block candidates ahead under the declared P2 policy.

**Why new:** expands the anchor population beyond identifier shape, unlike the already-flat P2 weight changes. Group identity is the whole phrase, not the last noun. Repeated ordinary phrases are weaker referent evidence than identifiers, so independently inspect same-phrase/different-example and version conflicts. A no-POS original contiguous-phrase partitioner with identical length/region/protection rules is essential. If it finds the same useful proofs, the retrieval benefit belongs to the join, not POS.

**Population/cost/falsifier:** inventory new complementary witnesses before scoring. Query NP prevalence is not a ceiling. Coverage/hard-locality misses are only a screening domain; an absent hard literal stays absent. Index by candidate block and record/section; avoid global all-pairs anchors. No new proof or no new reachable span stops this arm. Four existing same-block P2 proofs are not four new access gains.

##### 4. Keyword NP partitions for literal row/property lookup

**Algorithm:** enumerate query object/property partitions from original spans, preserving ambiguous alternatives → exact object postings → actual source row key/header/value coordinates → literal complete property/qualifier binding → existing original-query lexical ordering under E3's bounded append policy → one existing block containing all needed evidence → exhaust compatible rows/fields, abstain on unresolved competing bindings. No `what is … for …` template requirement; no generated headers or value labels.

**Why new:** changes E3's zero-activation frontend while retaining the stronger source structure. POS can reduce partition count; it does not make a header synonym true. All-contiguous no-POS partitions are the control. Source row/column identity must distinguish swapped values with identical block token multisets. Nonliteral field correspondence remains unverified without original source evidence; E1's zero declarations cannot be wished away.

**Population/cost/falsifier:** only actual same-block literal object/property witnesses count. Query activation without rows or legal spans stops implementation. Cost is candidate-local structural reads and a small partition join. Append-only integration cannot fix full pages or existing false silence; report that limit rather than attributing every recovered parse to retrieval opportunity.

##### 5. Contextual weak-term obligations and attested-form narrowing — conditional extensions

KIND words such as `version`, `memory` or `command` can be parts of the requested object/property. Carry them inside the above exact group bindings without globally replacing KIND or making verbs optional. Query/source compatible NP or operation groups can also narrow which **already attested original forms** are considered by an existing proposal path; retain that path's original gates, protected identifiers, rank, spans and stop behavior. This could reduce work or reject wrong-role proposals; it cannot certify a synonym or release an absent identifier.

These are lower priority because no affected valid-source population has been measured. Use identical candidate proposals with/without the additional role constraint; count lost positives as well as rejected noise. If the result is just nearest-word expansion with noun filtering, or a hidden coverage relaxation, stop. Source-local definitions/P3 and semantic E4 should remain inactive unless actual new source evidence—not merely a more permissive query parser—changes their previously empty certificate population.

#### A completion plan that should fit one end-to-end implementation task

Do not assign another general “implement all POS ideas” task. Give the implementer one mandatory changed-retrieval candidate and two conditional, bounded follow-ons with explicit completion criteria.

1. **Build one addressable annotation path, then immediately connect the lexical hook.** Reuse the working resource/loader. Add source unit/sentence identity and checked body-base offsets, protected original query spans, explicit tag mapping/unknown states and selected groups. Respect existing Markdown block/fence semantics; natural-language attachment stops at code, heading/body, list/table-cell boundaries. Baseline search eligibility remains unchanged. Do not build a full Markdown implementation, dependency parser, noun-head system, grammar corrector, exhaustive ngram index or dictionary-lemma column. Return query and source relation selections to the P1 pre-cut scorer; disabled must match the measured page.
2. **Complete the first causal lexical trial.** Predeclare one both-side selector and weight 1 for its initial P1-equation comparison; use archived untyped P1 as the closest control, not a reopened weight sweep. If bounded dev tuning is still desired, allow only one further selected-pair weight (.25), declared in advance and reported even if it fails. Require a source-witness ledger and at least one first causal divergence from untyped P1. If none exists, stop this arm rather than pretending baseline parity tests it. Run the full unchanged Rust DEV scorer for actual changed configurations, export all 1,470 ordered pages and complete cards/calls/spans, and attach labels only to diagnostic output after generation.
3. **Implement the factored assignment as a bounded executable, not a dump.** Use the same annotation objects and existing Encoder; deduplicate original intervals, distinguish alternate segmentations, retain capacities and unresolved obligations. Finish the no-group/group/POS and vector-off controls on fixtures and candidate-bound q4 DEV diagnostics. A small dynamic program over nonoverlapping ordered groups is enough; no arbitrary graph solver or global vector index. It must output chosen assignments and why alternatives fail. Automatic synonym cards remain disabled unless a genuinely new source-backed admission rule survives the known counterexamples. Lack of that rule is an explicit stop result, not permission to use a cosine cutoff.
4. **Inventory then finish only viable NP/field paths.** Run the full ordinary-NP join and literal field certificate inventory against the same captured bodies/candidate hooks, with provenance. If ordinary-NP pairs exist, integrate the existing P2 scheduling semantics and its no-POS control; if literal fields exist, integrate the compact frontend and its no-POS partition control. No weight variations for these population questions. Reserve at most four further changed DEV configurations for those two paired comparisons. Total changed configurations at most six across this plan; baseline parity and corpus-free assignment controls are not tuning configurations. Any missing population receives a concrete zero-at-stage result, not a parser-activation success claim.
5. **Finish lifecycle and operational proof for a surviving mechanism.** Persist selected query-independent occurrence/group metadata on writes in the existing libSQL file, keyed to body identity, structural publication, resource digest, annotation/tag-mapping version and region-parser version. Store only the exact original-span float32 rows actually required, additionally guarded by model and encoder identity. Index candidate block/unit access and record deletion. Normal open verifies manifests; it must not rebuild corpus-wide word/position/POS maps. Publish transactionally, invalidate on writes/resource changes, honor stale-publication checks and secure-forget. Build-time embed the pinned asset in the one native product and provide its provenance/notices; no runtime download, daemon, Java or second product runtime. The exploratory transient version can precede this, but must be labeled a prototype, not production completion.

For every changed configuration report all six exact numerators, original versus alternate outcomes, full paired gains/losses, and the first changed stage: selection → source relation → score/admission → cut → card/span → later calls. Inspect successful controls, the 42 cross-scope cases, 10 true-silence cases and 41 target-required no-gold cases separately. IDs/categories/gold never enter query analysis or matching. Record candidate counts, omitted/unverified obligations, cap activations and source witness-to-shown-card counts. A small annotation sample is an accuracy audit, not a substitute scorer.

Separate write-time parsing/encoding and database growth from normal open, tokenizer deserialization, static-model load, query annotation, candidate I/O, assignment, first use and resident query latency. Measure fresh release CLI processes in a no-daemon environment, including both POS-active and unsupported routes; lazy loading only avoids costs on routes that never activate. Reusing a resident debug audit's average is not a CLI cost estimate. Use the repo's Nix fmt/clippy/tests/build/reference checks for implementation, including actual pinned-resource tests and source-boundary/lifecycle fixtures.

Future implementation changes need explicit LLP0001 projection/publication decisions, LLP0002 selector/admission/rank/span/stop decisions, and LLP0000/0004 resource/runtime decisions. Large new designs are allowed by the user; current LLP algorithms are the baseline, not a blanket veto on research. LLP0005 scorer and adoption thresholds remain unchanged: at least one gold-any/first or xs-any/first rises, none falls, empty/section do not fall. Only the parent may select, freeze and make the reserved one-shot h2 confirmation. No h2 is justified by this review.

#### Reproduction and remaining unknowns

New bounded execution used the already-built child `target/debug/examples/pos_groups`, with its existing `evals/output/POS/en_tokenizer.bin`; no build, dependency fetch or asset download was needed. Files here:

- `evals/output/astra-pos-review/fixtures.jsonl`: six corpus-free inputs described above.
- `fixtures.out.jsonl`, `fixtures.log`: actual unchanged example output and load log.
- `audit.json`: counts recomputed from saved query output plus the five selected raw-query chunk examples. These are descriptive diagnostics, not relevance scoring.

Reproduce the fixture probe from the parent with:

The count audit parses each saved `queries.out.jsonl` query occurrence: raw ambiguity is more than one nonempty analysis; adjusted ambiguity is more than one distinct nonempty tag after removing `UNKNOWN` and `SENT_END`; missing chunks is an empty list, separately from `['O']`. It does not open case labels or compute scores. Existing primary evidence remains in the child's `prepare.py`, `queries.out.jsonl`, `source.out.jsonl`, `coverage.json`, `source.coverage.json`, `fixtures.out.jsonl`, `queries.cost.log`, `source.cost.log` and test/build logs. The complete POS report includes its baseline reproduction and explicit missing work.

Unknowns remain substantial: human accuracy on technical keyword queries; pre/post-disambiguation contribution; regex-backend equivalence; useful missed-candidate source-group population; whether POS beats a simpler structural/no-POS selector; safe nonliteral assignment admission; actual selected-group embedding cost; fresh release CLI cost; and distribution-ready resource integration. No gain is invented to fill these gaps.

**Recommendation:** complete the smallest pre-cut lexical experiment, and test the user's subgroup idea with source-constrained individual/group assignments. Continue ordinary-NP/field bindings only where actual source witnesses exist. Stop automatic semantics based on pooled cosine, reciprocal nearest neighbors or POS agreement alone. More informative grouping can improve discrimination without a new model; POS cannot manufacture missing source evidence, resolve arbitrary intent, or make static embeddings understand relations they do not encode. FINDINGS remains unchanged.


<a id="consolidated-evals-ideas-results-2026-11-08-md"></a>

### evals/IDEAS-RESULTS-2026-11-08.md

Uncommitted research report. Baseline: `129897f950208faeefaa43ddd10f55537e46c092`.
Four independent worktrees were used. No parent product/specification changes,
push, PR, author-field indexing, additional models, or threshold tuning occurred.

Every stream reproduced dev `any 410/439 first 350/439 xs 30/42 24/42;
empty 35/51 section 55/73`, with exact parity across 1,470 ordered pages.

| Stream | Objective result | Disposition |
|---|---|---|
| A: eligibility/context | Dev first +1, other measures flat; h2 cross-scope any −1, first −1, empty −1 | Reject |
| B: Markdown source structure | No genuine missing heading population established | No candidate |
| C: literal/reference contracts | Fetch-ID filter defect reproduced; zero dev id-filter wordings | Correctness backlog, no retrieval candidate |
| D: structural witnesses | Zero missed-gold wordings in declared explicit conjunction population | No candidate |

#### Eligibility candidate and reserved confirmation

Candidate commit: `a4c5b392ec51cc6945619e7857463f97958ecbf1`, retained only on
the isolated research branch. It filters lifecycle-ineligible records before
choosing record-fill and rare-neighbor winners. It changes no model, thresholds,
scorer, or corpus. Tests, clippy, formatting, Nix build and LLP references passed
in the child worktree. Its 28 changed dev wording pages are detailed in its report.
The sole case-level gain is `s5-c-0085` original wording: fourth card
`s5-r-0186` becomes gold `s5-r-0187`. No dev case-level losses were reported.

Parent reviewed the code and LLP departure, then ran the frozen candidate once:

Exit 0, 1,473 records, 6,477 blocks, all 460 cases:

Frozen baseline in `evals/output/cli-final/h2-frozen.stdout`:

**Objective rating: rejected under LLP 0005 because three holdout numerators
fell.** No retuning, cherry-pick, or replacement variant follows this result.
The parent implementation remains the baseline.

#### Reports and limitations

- A: `<historical-checkout>`.
- B: `<historical-checkout>`.
- C: `<historical-checkout>`.
- D: `<historical-checkout>`.

These reports and their diagnostic outputs remain uncommitted. The heading
audit is conservative, not a complete CommonMark certification. The witness
diagnostic is an original-query lexical ceiling, not a complete route trace.
The CLI fetch-ID defect was not fixed or counted as a retrieval gain.

No candidate survives to combine; no scale or new full neutral tables were run.
No new like-for-like external baseline comparison was run after rejection. Historical
hybrid measurements use restricted case slices and identical eligible-record
indexes; full dev floors and the synthetic six-document fixture cannot establish
a hybrid win. **Overall rating: no adopted retrieval improvement and no evidence
of beating external baseline from this research round.**


<a id="consolidated-evals-ideas-followups-2026-11-08-md"></a>

### evals/IDEAS-FOLLOWUPS-2026-11-08.md

Uncommitted research report. All 58 numbered ideas had already received merit
reviews; this batch examined four specific unresolved questions. Parent baseline
remains `129897f950208faeefaa43ddd10f55537e46c092`, with no tracked changes.
No retrieval, threshold, card order, scoring, corpus, author-field or model change
was integrated. No new h2 or scale search/evaluation, push or PR.

#### Objective ratings

| Follow-up | Rating | Result |
|---|---|---|
| I: canonical references | Tested correctness candidate; not merged | Bare/quoted references and final filters agree on the fixture matrix; all 1,470 q4-dev ordered pages unchanged. |
| J: failed-write recovery | Confirmed defect; no patch | Reusing a failed Store can make the failed body durable during a later successful add. Discard/reopen recovered four diagnostic tests across primary and publication failures. |
| K: repeated-card necessity | Diagnostic completed | Extra disjoint spans are sole section witnesses in 21 wording/object pairs, two individually necessary across all wordings. Identical later-call repeats add no containment. |
| L: growth statistics | Partial causal closure | Exact SQL statistics reverse one sampled original-row score order. Complete final-page causality still needs paired scale traces. |

Overall: **no retrieval-success improvement or hybrid win established**. These
are correctness and diagnostic outcomes, not four negative ranker trials. The
earlier rejected eligibility candidate remains closed and was not retuned.

#### I — reference correctness candidate

Isolated commit `fef2fafc65ee150b39bc2044e9e4a31e098a8ac3` changes CLI/query
code, CLI tests and the governing query/runtime LLPs. Resolution uses the existing
reference resolver; canonical values drive execution and final card/hint/related
filters. Cursors retain the original query and resolve it again on replay.

The patch also changes global query splitting to ignore prefix matches inside
double-quoted prose. That is explicitly documented in its LLP change and is
broader than reference-value substitution. Parent review retains this scope
distinction; the candidate remains isolated after this check rather than being
silently incorporated as a page-neutral refactor.

The child reproduced the original failure and tested bare/quoted record IDs,
hashes, fetch IDs, wrong paths, unknown/ambiguous references, reference-looking
prose, shared bodies, adjacent filters and deterministic cursors. Its fmt,
workspace clippy/tests, Nix build and LLP-reference checks passed. The unchanged
Rust dev scorer reports:

All 1,470 ordered pages match. This validates healthy measured pages and the
tested correctness inputs; it does not certify all possible query strings or
claim a retrieval gain. No holdout confirmation is needed to establish that the
candidate has no measured dev gain; no h2 run occurred.

#### J — failed-write safety

New probes extend the earlier insertion-abort finding:

- A failed accepted child remains resolvable in memory and incorrectly hides its
  parent. Removing the failure trigger and adding an unrelated record on the same
  Store can also persist the failed child.
- After a primary insertion abort or tested deferred commit failure, discard and
  reopen removes the failed child and restores the parent. A later add then does
  not reintroduce the failed body.
- After a successful primary commit followed by structural/numerical publication
  failure, reopen preserves the committed child and restores projections. Deleting
  the child to compensate would incorrectly undo a committed write.

Four corpus-free Rust tests passed. No patch was offered: a complete contract must
distinguish validation refusal, confirmed rollback, confirmed primary commit with
incomplete publication, and unknown outcomes. A guard on add alone would leave
public resident records accessible. Actual encoder failure, ambiguous I/O commit,
cancellation and crash/concurrency combinations remain untested. No new healthy
search run or fix-parity claim was made by J.

#### K — exact section witness necessity

Offline Rust replay uses the existing containment predicate verbatim and exact
gold-record object rule. It reproduces 55/73 section cases across all wordings
and verifies reused captured pages/cards; it does not execute a new search.

Across the 73 section cases and 74 gold objects, 21 additional disjoint-card
witnesses are individually sole witnesses within a wording/object. Two remain
individually necessary across all wordings in otherwise passing cases:

- `s4-c-0029`, `s4-r-0050`, original wording, call 1/position 0 (zero based),
  bytes `25074..25600`.
- `s6-c-0001`, `s6-r-0003`, original wording, call 2/position 2 (zero based),
  bytes `3523..4445`.

Identical later-call repeats have zero incremental containment; this says nothing
about whether a reader previously opened the reference. Redundancy of an individual
card is not proof that removing several together is safe. No cards were removed,
replaced or combined, and the closed replacement experiments were not rerun.
Hidden-token causality remains unproven: prefix visibility is not a controlled
upstream causal test or an annotation of decisive semantic content.

#### L — growth mechanisms from saved SQL

Read-only exact SQL/native FTS evidence on six preselected dev queries shows
original-row matcher admission unchanged across corpus sizes. New intermediate
lexical candidates exist in two of three sampled pages whose final cards all
come from original records. A final page alone therefore cannot identify its
candidate population.

For `s3a-c-0002`, original-row TF and lengths are unchanged. Specified scoring
arithmetic with actual corpus statistics changes `s3a-r-0066` from 15.957134 to
26.764515 and `s3a-r-0037`/`s3a-r-0052` from 15.788296 to 26.939422. That reverses
their raw score order, consistently with the saved page. The baseline equation
scores agree exactly with G's actual trace. This establishes a statistics-driven
raw-score mechanism, not exclusivity of that cause for the final page.

No new scale searches or DB mutations occurred. A paired scale execution trace
and publication manifest are still missing for complete final-page attribution.
The report specifies a bounded future measurement; it was not run or treated as
permission to change statistics, gates or ranking.

#### Detailed evidence

- I report (historical child source).
- J report (historical child source).
- K report (historical child source).
- L report (historical child source).

Exact commands, source evidence, fixtures and outputs are retained in those
reports and each worktree's ignored evals/output directory. Reports, notes,
corpora, model weights and output remain uncommitted.


<a id="consolidated-evals-astra-implementation-results-2026-11-08-md"></a>

### evals/ASTRA-IMPLEMENTATION-RESULTS-2026-11-08.md

Uncommitted research ledger. Parent remains at
`129897f950208faeefaa43ddd10f55537e46c092`; no experimental product code merged,
push or PR. All implementation work used isolated Sol 6.1 medium worktrees.
The user authorized bounded dev tuning of new candidate constants. Scorer,
corpus, query lists, splits, evaluation/adoption thresholds, model and author-field
boundaries remained unchanged. No new h2, scale, full neutral or hybrid run.

#### Objective verdict

**No standalone or combined configuration qualifies for adoption.** Experimental
code and fixtures exist for all eight proposals; this is no longer only a
catalogue/population review. Some paths remain inactive on the measured corpus,
and one semantic proposal was falsified before automatic-card integration.
Neither an inactive prototype nor a failed narrow grammar disproves a broader
research family. No holdout confirmation is justified by these results.

Frozen baseline:

Every lane's disabled/legacy path reproduced these numerators and all 1,470
ordered dev pages. Code, tests and LLP changes in the experimental branches were
checked with formatting, strict workspace clippy, workspace tests, Nix build and
LLP references. Detailed reports retain initial failures and final passing logs.
Corpus, models, reports and outputs were not committed.

#### Per-proposal outcome

| Proposal | Local experimental commit | Objective rating |
|---|---|---|
| P1: positional lexical matching | `40ec6f80819863a38a78221029453cd0ecf6fb0d` | Active: two weights lose gold; smallest weight is flat, not selected. |
| P2: subject-bound source joins | `884a2db8577835a6c118d6d0e33a28bfd8ef3d9a` | Five proofs; three weights leave all responses unchanged, not selected. |
| P3: source name/definition bindings | `1a848956dffe1e391315566ed7b7668e3ab41f23` | Inactive: no source pair under the frozen grammar, not selected. |
| Q: required/descriptive contract | `aff55ad45a6a6a4dd98e644ad6e173d76e9e5d54` | Constructed intent tests pass; opt-in path unused by legacy cases, no R gain. |
| E1: source-attested equivalence | `f58046eb383d0a1ab380122aeeb5ce210f85912b` | Zero source declarations and supported requests under the frozen grammar, not selected. |
| E2: anchored phrase alignment | `fbbf092ea0dbdcb4b63283c7e223b9e8bd01ffe1` | Actual Encoder accepts scalar opposites; rejected for automatic cards. |
| E3: row/field validation | `9558ad149615cfb29510526574014d459c69978b` | Source-span fixtures pass; controlled-question parser has zero dev activation, not selected. |
| E4: validate before stopping | `f40b4c43ec9250c30eab92e81fc59599c8282f97` | Role/negative-answer/leak fixtures pass; controlled grammar has zero dev activation, not selected. |

E1/E3/E4 each evaluated one frozen configuration. A numeric weight cannot activate
a raw-query grammar that matches no query; source absence likewise cannot be
fixed by tuning a score. Their implementations are narrow and do not test a
general keyword-query frontend. Q intentionally does not reinterpret legacy
questions; claiming fixed-floor gains by converting them would change the task.

P3 used gap 4096 and fanout 8. Parent independently ran its exact Rust inventory
and coverage helpers with gap/fanout removed: 1,376 records, four bindings in one
record, **zero same-section cross-block exact-name pairs even before query
coverage or eligibility**. Thus these two resource constants do not suppress an
otherwise available pair in q4. Reproducer and parent review are in that lane's
ignored `evals/output/P3/parent_inventory_review.rs` and `parent-review.md`.

E2 has four mechanically parseable dev wordings, but its actual Encoder matrix
accepts both positive substitutions and opposite meanings under strict nonvacuous
reciprocity. It retains a diagnostic alignment API, not a synonym certificate or
an enabled card hook. No cutoff was fitted to hide this falsifier.

#### P1 and P2 standalone configurations

| Mechanism/configuration | Gold any | Gold first | XS any/first | Empty | Section | Verdict |
|---|---:|---:|---|---:|---:|---|
| Baseline |410/439|350/439|30/42,24/42|35/51|55/73|Control|
| P1 weight 1 |408/439|342/439|31/42,24/42|36/51|55/73|Reject: gold falls|
| P1 weight .25 |409/439|346/439|30/42,24/42|35/51|55/73|Reject: gold falls|
| P1 weight .05 |410/439|350/439|30/42,24/42|35/51|55/73|Flat, not selected|
| P2 weights 1, .5, 2 (each) |410/439|350/439|30/42,24/42|35/51|55/73|Flat, not selected|

P1 at .05 changes 39 first pages and 49 complete responses, despite no case-level
metric gains or losses. Its larger weights recover some cases but lose more
gold performance; cross-scope/empty gains do not compensate under LLP 0005.
P2's five witnesses cover four same-block pairs and one cross-block pair whose
secondary is already reachable in baseline later calls. Its complete responses
are identical across the three tested weights and baseline.

P1's initial enabled timings included heavy diagnostic serialization and predate
its final body-posting work-avoidance guard. Do not use those old 120–143 ms
medians as final implementation latency. The combination study below reruns the
final sources with untraced invocation timings and standalone controls.

#### Combination experiment: actual arithmetic, not summed gains

Experimental semantic merge:
`f92acce07ee17883817e5ec110fd875a7fb790e2` in the combination worktree.
P1 contributes exactly once to the primary row memo. P2 consumes those combined
source scores, competes using its weakest witness, applies penalty seven once,
and retains original-span/later-card capacity behavior. Both are disabled by
default. Final-source controls reproduce the individual quality results above.

| Combined P1 / P2 weights | Gold any/first | XS any/first | Empty | Section | Verdict |
|---|---|---|---|---|---|
| .05 / 1 |410/439,350/439|30/42,24/42|35/51|55/73|Flat|
| .25 / 1 |409/439,346/439|30/42,24/42|35/51|55/73|Reject|
| 1 / 1 |408/439,342/439|31/42,24/42|36/51|55/73|Reject|
| .05 / .5 |410/439,350/439|30/42,24/42|35/51|55/73|Flat|
| .05 / 2 |410/439,350/439|30/42,24/42|35/51|55/73|Flat|

Each joint configuration's **1,470 complete responses exactly equals its
corresponding P1-only control**, including calls, cards, spans and metadata.
Parent independently parsed and compared these five pairs after handoff; no
retrieval searches or substitute scoring were involved in that verification.
There are zero combined-versus-P1 paired gains or losses.

An internal score interaction is real: P1 changes the router-tester primary but
not its weakest secondary; it also increases the integrity-check weakest score.
Neither creates a new proof or different shown evidence. Tier-2 copies of rows
already represented at stronger tiers do not improve their record selection.
This justified testing the two conditional P2 weight variants rather than
assuming independent flat results could not interact. The resulting negative
output evidence closes this declared five-configuration combination study.

The unchanged no-gold diagnostic split is 10 true-silence and 41 target-required
cases, separate from the 42-case first-page cross-scope denominator. True-silence
remains 0/10 in every configuration. Target-required is 35/41 except the weight-1
P1 configurations at 36/41. Holding the baseline's poor silence result is not
proof of general safety.

Why other unchanged prototypes were excluded from combination: P1/P2 score
changes cannot activate E1/E3/E4 raw-query grammar or invent P3 source endpoints;
E2's rejected semantic certificate cannot authorize cards; Q needs an explicit
caller opt-in absent from all legacy wordings. This rules out these particular
unchanged combinations, not redesigned POS-aware parsers or future source rules.

#### Cost and evidence limits

The combination study's final-source untraced invocation median/p95 milliseconds:

- Baseline 3.840/9.185; P2-only 4.330/20.151.
- P1-only .05: 7.529/35.125; .25: 7.366/35.802; 1: 7.572/35.855.
- Joint configurations: medians 7.714–8.433; p95 43.150–46.432.

These are descriptive run samples, not neutral C2 gates or perfectly controlled
microbenchmarks. Invocation excludes grading/output; initial baseline store time
includes ingestion and cannot be compared with later cached opens as an opening
speedup. Without a quality rise, the extra measured work offers no adoption case.

Source-span fixtures establish narrow behavior, not general semantic validity.
No full reader study was run, and no claim of beating matched external baseline follows.
The separately authorized POS/grouping continuation is recorded independently in
[consolidated finding](#consolidated-evals-pos-continuation-review-2026-11-08-md). Its corrected candidate qualifies on dev,
but parent h2 confirmation loses both cross-scope measures and is rejected
without retuning. It is not part of the P1/P2 configuration table above and was
not merged into the product.

#### Detailed reports and reproduction

- P1 (historical child source).
- P2 (historical child source).
- P3 (historical child source).
- Q (historical child source).
- E1 (historical child source).
- E2 (historical child source).
- E3 (historical child source).
- E4 (historical child source).
- P1/P2 combined (historical child source).

Reports contain exact Nix commands, native scores, complete configuration histories,
paired cases/wordings, original byte witnesses, checks and limitations. Each
worktree retains ignored output/corpus and reproducible diagnostics. No model,
corpus, raw/intermediate output or uncommitted report entered a product commit.


<a id="consolidated-evals-results-system-2026-11-09-md"></a>

### evals/RESULTS-SYSTEM-2026-11-09.md

Historical configuration ledger supplement. The combined-study closure above governs: none of this round’s 112 configurations was adopted; the runtime prototypes were removed. Intermediate dev qualification is not adoption.

Uncommitted research report. Fifty declared configurations have been measured.
The goal of a substantially larger search-success improvement remains open.
All new mechanisms are default-off; no candidate has been adopted. Product
scoring defaults, original bodies, the model and card budgets are unchanged.
No h2, scale, hybrid or neutral table was run in this study. No push or commit.

#### Protocol and strongest result

Starting commit: `8310cfad51bc342acb0bdf947a5b2e09efdeee3f`. The current
POS-enabled baseline is 411/439 any, 351/439 first, cross-scope 30/42 and 24/42,
empty 35/51, section 55/73. It is not the older POS-off baseline. The unchanged
Rust evaluator scores 490 q4 dev cases and 1,470 prepared wordings. No case IDs,
gold, alt_queries or author fields enter retrieval. Every configuration was
recorded before its run in `evals/output/system-experiment/configurations.md`.

The strongest aggregate first-query configuration is `adjacent-joint-half`:
**413/439 any, 358/439 first, cross-scope 32/42 and 26/42, empty 36/51,
section 55/73.** This is +2 any, +7 first, +2/+2 cross-scope, +1 empty,
flat section. Objective rating: passes the dev aggregate improvement condition;
not confirmed or adopted, and not yet the requested large gain.

It changes 981 ordered wording pages. Paired case changes are 3 gains/1 loss
for gold-any, 8 gains/1 loss for gold-first, 2 gains/0 losses for cross-scope-any,
and 3 gains/1 loss for cross-scope-first. Aggregate improvement does not mean
no individual regressions. The lost gold-any case is `s5-c-0058`; the lost
original-query gold is `s3a-c-0015`; cross-scope-first loses `s4-c-0064`.
Complete paired ledgers and record IDs are retained beside each result.

#### What is implemented and what the measurements say

- **POS plus embeddings:** align an original source nominal pair with a query
  pair through a same-position literal anchor. Compare the other words with
  the existing Encoder before lexical cuts. Optional ordinary-term coverage
  credit never satisfies a hard literal. Alone this gains one first-query hit.
- **Stem-aware scoring:** matcher B admits Porter matches but its original score
  uses surface TF. Missing-only and full stem-aware score controls address that
  mismatch. This has broader benefit than the pair feature by itself.
- **Joint matcher assembly:** merge complete admitted lists before the five-record
  cut. Keep literal tiers and the original scope penalty. The one-record union
  improves first-query hits; retaining distinct matcher spans favors sections.
- **Passage preservation:** retain displaced spans of still-selected records in
  spare later-call slots, without eviction or larger budgets. This repairs a
  section loss from the one-record union.
- **Adjacent original blocks:** touching spans in the same record and ATX section
  may jointly cover ordinary terms. Score only missing-term support, keep hard
  literals in the displayed block, and offer its witness in spare later-call
  slots. Weight 0.5 is flat alone but adds one original-query hit to the joint
  configuration. Weight 1 loses a section hit and is not qualifying.
- **NLP ablations:** ordinary query pairs and source-word windows test whether
  POS restrictions are the limiting factor. Broader windows alone lose gold;
  recombining them with the stronger assembly does not improve its metrics.
- **Two-member alignment and vocabulary proposals:** allow both pair words to
  differ, first with an exact context word, then using the existing persisted
  vocabulary matrix and intersected FTS postings. Actual ordered source pairs
  remain required. Neither beats anchored matching in this study.
- **Semantic soft TF:** use best relation similarity as fractional TF for missing
  ordinary query words, with existing BM25 normalization. Strict POS variants
  are flat against the stronger combination; broader windows lose two first
  hits. No demonstrated extra benefit from changing the semantic score scale.
- **Kind-word downweighting:** words excluded from coverage still contribute to
  ranking. Removing or halving those score contributions regresses results.

These are query-time experiments over the existing persisted source/POS/FTS
projections. No ingestion enrichment, phrase-vector index, second model, RRF,
reranker, manufactured keyword list or document rewrite was introduced.
Rules and controls are documented in LLP 0002. All experiments remain opt-in.

#### Concrete witnesses and gate interaction

`deployment identifier` aligns with original source `deployment ID` at cosine
0.5221374 in gold `s3a-r-0112`, gaining first-query retrieval for `s3a-c-0063`.
The source bytes are unchanged; `semantic-witness.stderr` preserves the trace.

Adjacency's additional original-query recovery is `s6-c-0006`, “minimatch nocase
option”. Its gold note `s6-r-0185` is a single short block. The direct supported
blocks instead belong to longer minimatch records: touching spans
[18288,19340) and [19340,20659), rows 5160/5161 and 5727/5728. Their admission
makes the primary ranking nonempty. Existing near-match admission can then
select gold row 6029. This is an indirect gate interaction, not a claim that the
gold note spans two blocks. The original-scope trace and returned source spans
are in `adjacent-witness.stderr` and `adjacent-witness.json`.

The alternate “cursor settings file keys” also gains a wording hit, but its case
already succeeded on another wording, so it contributes no gold-any rise.
Broader silent-search admission was already measured in FINDINGS; it was not
rerun as a new idea here.

#### Complete measured ledger

Numerators below use denominators 439,439,42,42,51,73. Replays are included
alongside the fifty distinct configurations. “Dev improves” means the aggregate
adoption precondition only; “flat” permits changed pages; neither means adoption.

| Configuration | Any | First | XS any | XS first | Empty | Section | Changed pages | Objective rating |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| adjacent-full | 411 | 351 | 30 | 24 | 35 | 54 | 50 | regresses / does not qualify |
| adjacent-half | 411 | 351 | 30 | 24 | 35 | 55 | 25 | flat |
| adjacent-joint-full | 413 | 358 | 32 | 26 | 36 | 54 | 988 | regresses / does not qualify |
| adjacent-joint-half | 413 | 358 | 32 | 26 | 36 | 55 | 981 | dev improves |
| admit | 411 | 351 | 30 | 24 | 35 | 55 | 31 | flat |
| aligned-admit | 413 | 355 | 32 | 26 | 36 | 55 | 988 | dev improves |
| aligned-broad | 413 | 356 | 32 | 26 | 36 | 55 | 992 | dev improves |
| aligned-score | 413 | 355 | 32 | 26 | 36 | 55 | 988 | dev improves |
| anchor-strict-admit | 413 | 355 | 32 | 26 | 36 | 55 | 988 | dev improves |
| anchor-strict-score | 413 | 355 | 32 | 26 | 36 | 55 | 988 | dev improves |
| baseline | 411 | 351 | 30 | 24 | 35 | 55 | 0 | flat |
| both-window-broad | 410 | 351 | 31 | 24 | 36 | 55 | 172 | regresses / does not qualify |
| both-window | 410 | 351 | 30 | 24 | 35 | 55 | 110 | regresses / does not qualify |
| broad | 411 | 352 | 30 | 24 | 35 | 55 | 47 | dev improves |
| current-default | 411 | 351 | 30 | 24 | 35 | 55 | 0 | flat |
| disabled | 411 | 351 | 30 | 24 | 35 | 55 | 0 | flat |
| final-disabled | 411 | 351 | 30 | 24 | 35 | 55 | 0 | flat |
| full-half | 411 | 350 | 31 | 22 | 36 | 56 | 356 | regresses / does not qualify |
| full | 413 | 352 | 32 | 24 | 36 | 56 | 708 | dev improves |
| joint-kind-half | 413 | 356 | 32 | 25 | 36 | 55 | 1000 | dev improves |
| joint-kind-zero | 413 | 356 | 32 | 25 | 36 | 55 | 1008 | dev improves |
| joint | 413 | 354 | 32 | 25 | 36 | 55 | 563 | dev improves |
| kind-half | 410 | 350 | 30 | 24 | 35 | 55 | 144 | regresses / does not qualify |
| kind-zero | 410 | 350 | 30 | 24 | 35 | 55 | 195 | regresses / does not qualify |
| last-candidate | 413 | 357 | 32 | 26 | 36 | 55 | 975 | dev improves |
| last-default | 411 | 351 | 30 | 24 | 35 | 55 | 0 | flat |
| ordinary-broad | 411 | 352 | 30 | 24 | 35 | 55 | 58 | dev improves |
| ordinary-strict | 411 | 351 | 30 | 24 | 35 | 55 | 35 | flat |
| preserve-spans-full | 413 | 356 | 32 | 26 | 36 | 56 | 992 | dev improves |
| preserve-union-full | 413 | 357 | 32 | 26 | 36 | 55 | 975 | dev improves |
| preserve-union-missing | 412 | 356 | 32 | 26 | 36 | 55 | 946 | dev improves |
| proposed-admit | 413 | 356 | 32 | 26 | 36 | 55 | 971 | dev improves |
| proposed-both | 413 | 357 | 32 | 26 | 36 | 55 | 1008 | dev improves |
| proposed-broad | 413 | 357 | 32 | 26 | 36 | 55 | 975 | dev improves |
| proposed-query | 413 | 357 | 32 | 26 | 36 | 55 | 975 | dev improves |
| proposed-score | 413 | 356 | 32 | 26 | 36 | 55 | 971 | dev improves |
| proposed-source | 413 | 357 | 32 | 26 | 36 | 55 | 993 | dev improves |
| recombine-both | 413 | 357 | 32 | 26 | 36 | 55 | 1007 | dev improves |
| recombine-query | 413 | 357 | 32 | 26 | 36 | 55 | 975 | dev improves |
| recombine-source | 413 | 357 | 32 | 26 | 36 | 55 | 993 | dev improves |
| score | 411 | 351 | 30 | 24 | 35 | 55 | 28 | flat |
| soft-anchor | 413 | 357 | 32 | 26 | 36 | 55 | 973 | dev improves |
| soft-proposed | 413 | 357 | 32 | 26 | 36 | 55 | 973 | dev improves |
| soft-window | 413 | 355 | 32 | 26 | 36 | 55 | 1002 | dev improves |
| source-window | 410 | 351 | 30 | 24 | 35 | 55 | 76 | regresses / does not qualify |
| spans-full-joint | 413 | 356 | 32 | 26 | 36 | 55 | 992 | dev improves |
| spans-full | 413 | 354 | 32 | 26 | 36 | 55 | 987 | dev improves |
| spans-joint | 412 | 355 | 32 | 26 | 36 | 54 | 948 | regresses / does not qualify |
| spans | 411 | 351 | 31 | 24 | 36 | 55 | 615 | dev improves |
| stem-0.25 | 412 | 351 | 31 | 24 | 36 | 55 | 149 | dev improves |
| stem-0.5 | 412 | 350 | 31 | 24 | 36 | 55 | 259 | regresses / does not qualify |
| stem-1.0 | 413 | 353 | 32 | 25 | 36 | 55 | 541 | dev improves |
| stronger | 411 | 352 | 30 | 24 | 35 | 55 | 82 | dev improves |
| union-full-joint | 413 | 357 | 32 | 26 | 36 | 54 | 975 | regresses / does not qualify |
| union-joint | 412 | 356 | 32 | 26 | 36 | 54 | 946 | regresses / does not qualify |
| union | 411 | 352 | 31 | 24 | 36 | 54 | 637 | regresses / does not qualify |
| verified-candidate | 413 | 356 | 32 | 26 | 36 | 55 | 992 | dev improves |
| verified-disabled | 411 | 351 | 30 | 24 | 35 | 55 | 0 | flat |

#### External fixture and invocation cost

The already-frozen `preserve-union-full` configuration (before adjacency) was
measured once on the six-document, 49-question synthetic document fixture.
Hit@1 rises 31/49 to 34/49; Hit@3 and Hit@5 stay 35/49; MRR@5 rises 0.6735 to
0.7041. The three top-rank gains are in the deep-research slice. Objective rating:
better top rank, unchanged document coverage. This fixture is known and synthetic,
not an unbiased independent holdout. No tuning followed from it. The newer
adjacency combination has not been measured there.

Fresh CLI timing for that same pre-adjacency combination used 12 queries and
two alternating rounds. Baseline medians were 196.057 and 187.776 ms; candidate
medians 257.543 and 255.973 ms. Baseline p95 was 250.691/261.738 ms; candidate
265.747/274.136 ms. Objective rating: slower in this diagnostic sample. Some
runs overlapped builds, so this is not a clean C2 measurement or a causal
attribution. New semantic vectors are query-local; full stem scoring also
scans resident parts metadata. Neither is production-optimized.

#### Validation and limits

Current default replay reproduces all 1,470 baseline ordered record-ID pages
and all six baseline metrics. Formatting, strict workspace clippy, workspace
all-feature tests and LLP references pass. The final `nix build --cores 1` also passed, including its release tests and
packaging checks. No corpus, model, output or
report has been staged. The scorer is unchanged.

The embedding work is still narrow: single-token endpoints in two-word source
relations, plus a bounded positional-window ablation. It does not exhaust full
noun phrases, relational composition or group-level semantic candidate discovery.
The existing static mean-pooled model cannot by itself distinguish role reversal;
word similarity remains relevance evidence, not a certificate. Proposed-pair
and soft-TF controls being flat do not prove embeddings are fully exploited.

The strongest candidate still has individual regressions and substantially
changes the page population. All selection used q4. Prior h2 tuning means that
corpus is not a fresh independent holdout, and it was not consulted this round.
Do not infer a hybrid win or full neutral validation. The strongest mechanism
combination is worth retaining for investigation, not silently enabling.

#### Reproduction

Unset all other experimental controls. The copied store is prepared for q4;
never reuse it with another corpus. With no controls set, reproduce the current
default export instead. Raw outputs, timing samples, witnesses, configuration
ledger, binaries and paired case ledgers remain under ignored
`evals/output/system-experiment/`.


<a id="consolidated-evals-results-sentence-2026-11-09-md"></a>

### evals/RESULTS-SENTENCE-2026-11-09.md

Historical configuration ledger supplement. The combined-study closure above governs: none of this round’s 112 configurations was adopted; the runtime prototypes were removed. Intermediate dev qualification is not adoption.

Uncommitted research, continuing the fifty configurations in
RESULTS-SYSTEM-2026-11-09.md. All mechanisms remain default-off. No adoption,
holdout confirmation, hybrid win or large-gain claim. Original bodies, scoring
harness, model, hard literals and card budgets are unchanged. No h2, scale,
neutral or hybrid run. No push or commit.

#### Result and interpretation

The strongest completed admission-only controls reach **414/439 any, 359/439
first, cross-scope 32/42 and 26/42, empty 36/51, section 55/73**. Against the
applied POS-enabled default this is +3 any, +8 first, +2/+2 cross-scope, +1 empty
and flat section. Objective rating: dev-qualified aggregate improvement;
experimental and unconfirmed, still short of the requested large gain.

Against the preceding best combination, `adjacent-joint-half`, admission-only
adds one any and one first hit, with no case-level gold or cross-scope losses.
The two-anchor control changes 10 of 1,470 ordered wording pages; the one-anchor
control changes 68. Both recover `s2b-c-0006`. Against the applied default, the
two-anchor combined candidate changes 982 pages: gold-any 4 gains/1 loss,
gold-first 9 gains/1 loss, cross-scope-any 2 gains/0 losses, cross-scope-first
3 gains/1 loss. Aggregate improvement does not remove the earlier individual
regressions in the preceding combination.

The useful distinction is **admission versus scoring**. The same partial source
assignment, given a scalar score bonus, displaces other gold results. Permitting
it to establish ordinary coverage while retaining lexical scoring recovers the
new case without those additional losses. This is a measured interaction, not
proof that vector similarity certifies meaning or that all semantic scoring is
harmful.

#### Mechanism and source witness

The previous combination still missed 26 gold cases across all prepared
wordings. Their original bodies and queries were inspected; no source or scorer
was changed. The inventory is `evals/output/system-experiment/remaining-misses.json`.

Align 2–8 ordinary query content words to distinct original word occurrences.
Candidate blocks require an exact rare query word. Source groups begin as
nlprule sentences; controlled variants also offer adjacent sentences or the
whole original structural unit. They never cross separate paragraphs, headings,
table cells, fences, blocks or records. Source groups over 128 words are skipped.
An occurrence cannot pay for two query words. Existing individual word vectors,
lexical IDFs and deterministic subset assignment select the match; an optional
possible-POS overlap gate restricts nonexact links. This uses the existing model,
not a second embedding system or manufactured author evidence.

Complete assignment requires every ordinary word. Partial assignment uses the
existing lexical coverage need, with at least one more assigned word than the
configured exact-anchor count. Unassigned words remain unmatched. Hard terms
still require exact evidence in the candidate block. Admission-only and
score-only controls separate coverage credit from the semantic fractional-TF
bonus. LLP 0002 states the detailed algorithm and controls.

For `s2b-c-0006`, original query `pairing auth failure stored state`, gold
`s2b-r-0007` has original span [0,294). It describes authenticated()/readRules()
deleting stored enrollment after authentication errors, and a late profile-A
response erasing a newer profile-B pairing. Whole-unit assignment links exact
`pairing` and `stored`, and `auth` with source `authentication`. `failure` and
`state` remain unmatched. The three supported words meet ordinary coverage;
the two-anchor admission-only page places the original gold record first.
No keyword is inserted into the body. Trace and full original cards are retained
in `sentence-witness.stderr` and `sentence-witness.json`.

The partial trace's query and source arrays are not positional pairs: unmatched
query entries are omitted only from the source array. Do not zip those arrays
when auditing. The code and explicit assignment fixture preserve unmatched
entries internally.

#### Complete native measurements

All rows use the previous strongest combination and weight 1. Denominators are
439 gold, 42 cross-scope, 51 empty and 73 section. Every completed configuration
has all 1,470 ordered wording pages and native all-call scoring. Configurations
were recorded before their runs in `configurations.md` under ignored output.

| # | Control | Any | First | XS any/first | Empty | Section | Objective rating vs preceding best |
|---|---|---:|---:|---|---:|---:|---|
| 49 | Preceding best, no assignment | 413 | 358 | 32/26 | 36 | 55 | Comparison |
| 51 | Complete sentence, untyped, min .5 | 413 | 358 | 32/26 | 36 | 55 | Flat |
| 52 | Complete sentence, POS, min .5 | 413 | 358 | 32/26 | 36 | 55 | Flat |
| 53 | Complete sentence, POS, min .4 | 413 | 358 | 32/26 | 36 | 55 | Flat |
| 54 | Complete sentence, POS, min .4, two anchors | 413 | 358 | 32/26 | 36 | 55 | Flat |
| 55 | Complete neighbors, POS, min .4 | 413 | 358 | 32/26 | 36 | 55 | Flat |
| 56 | Complete unit, POS, min .5 | 413 | 358 | 32/26 | 36 | 56 | Section-only rise |
| 57 | Complete unit, POS, min .4 | 413 | 358 | 32/26 | 36 | 56 | Section-only rise |
| 58 | Partial sentence, POS, min .5 | 411 | 356 | 32/26 | 36 | 55 | Gold regression |
| 59 | Partial neighbors, POS, min .5 | 411 | 355 | 33/27 | 36 | 55 | Gold regression, XS rise |
| 60 | Partial unit, POS, min .5 | 412 | 354 | 33/27 | 36 | 55 | Gold regression, XS rise |
| 61 | Partial unit, POS, min .4 | 412 | 354 | 33/27 | 36 | 55 | Gold regression, XS rise |
| 62 | Partial unit, POS, min .4, two anchors | 413 | 357 | 32/26 | 36 | 55 | First regression |
| 63 | Partial unit, POS, min .4, admission only | 414 | 359 | 32/26 | 36 | 55 | Retrieval rise, no aggregate loss |
| 64 | As 63, two anchors | 414 | 359 | 32/26 | 36 | 55 | Same rise, fewer changed pages |
| 65 | Partial neighbors, POS, min .5, admission only | 413 | 358 | 32/26 | 36 | 55 | Flat |
| 66 | Partial unit, POS, min .4, score only | 411 | 355 | 33/27 | 36 | 55 | Gold regression, XS rise |
| 67 | As 63, no POS compatibility | 414 | 360 | 32/26 | 35 | 55 | Extra first hit, one empty loss vs 63 |
| 68 | As 64, no POS compatibility | 414 | 359 | 32/26 | 36 | 55 | Flat vs 64 |

Unless stated, there is one exact anchor and both admission and score are active.
The above regression labels compare with configuration 49, not the applied
production baseline. Some regressing controls still improve that older baseline;
none is a better balanced result than 63/64.

Partial-unit both-effects gains s2b-c-0006 any/first, loses any/first s3a-c-0054
and s3b-c-0042, and loses first s2a-c-0007, s4-c-0021 and s4-c-0105. It gains
cross-scope any/first s4-c-0057. Two anchors remove some collateral changes but
still lose s3a-c-0054 any/first and s4-c-0021 first. Admission-only avoids all
these new metric losses; full paired changes are in `sentence-versus-joint.json`.
The native scorer remains authoritative for empty and section metrics.

#### Execution, cache correction and limits

Four incomplete untyped attempts were explicitly stopped to remove repeated
work. They are not scores. Their 17, 175, 336 and 515 completed pages all match
the corrected full execution's corresponding prefixes, as recorded in
`sentence-execution-parity.json`. No result is inferred from incomplete pages.

Source word/POS arrays and individual word vectors are connection-local libSQL
TEMP caches, bounded to 4,096 blocks and 16,384 vectors. Source-write triggers
clear both transactionally, including forget. Vectors preserve all 512 float32
values bit-for-bit. No persistent schema or source text changes. Query-local
cosine caching reuses the same scalar operation. Runs set RAYON_NUM_THREADS=1
after observed oversubscription; no arithmetic change was made.

The first cache implementation was undone by agent() rolling back its read
snapshot. A helper-only test missed that boundary. Successful experimental calls
now commit the read snapshot to preserve TEMP inserts; errors and default calls
still roll back. An actual-agent test proves survival, repeat-output equality,
unchanged immutable envelopes and forget invalidation. This corrects execution,
not retrieval thresholds.

A two-query resident probe measured 1.889/1.272 seconds cold and 10.884/12.210 ms
on repeat, with identical cards. Objective rating: useful resident reuse,
unacceptable cold CLI cost. These diagnostic samples are not a latency gate.
Source arrays would need guarded write-time publication before adoption.
A stack sample of the stripped binary did not resolve Rust function names and
cannot establish which component dominates latency.

Tests cover distinct occurrences, missing obligations, deterministic ties,
partial unmatched entries, source-unit boundaries, lifecycle filtering, float32
round trips, cache reuse/forget, and independent admission/score effects. The
latest executable's default run reproduces 411/351, 30/24, 35,55 and all 1,470
ordered baseline pages. Scorer, corpus and author fields remain untouched.

Two final controls (67–68) remove only possible-POS compatibility from 63/64.
Both completed. One anchor changes 38 pages against its typed control, gaining
first-query gold s6-c-0083 but losing one aggregate empty success. Two anchors
change four pages with no metric movement. A subsequent native per-case replay of the four changed no-gold cases
identifies s5-c-0118 as the empty loss (1/1 to 0/1); the other three are flat.
Those replays use the same frozen controls, not new configurations. Evidence is
in `untyped-empty-native.json`. The untyped one-anchor result is another dev-qualified trade-off
against the applied default, but does not dominate the balanced 414/359 result.
Removing POS compatibility is not a demonstrated general improvement.

The complete 1,470-query timing export for the two-anchor admission control has
median 99.68 ms and p95 448.50 ms (220.18 seconds summed). The prior joint run
recorded 14.52/35.49 ms. These runs had different concurrent workload, so their
ratio is not an isolated speed comparison. Objective rating: the prototype's
observed resident latency is not production-ready; its success-rate gain does
not justify silently shipping this query-time annotation/assignment cost.

Final validation: format check, strict workspace clippy, all workspace tests,
LLP reference check and git diff whitespace check passed. The v8 default native
run and an exact export comparison both confirm all 1,470 baseline pages.
Final v8 Nix build, release package tests and fixup also passed. The packaged
binary is `/nix/store/90l0msdd33cdx3hnqz6fkga8yzd46b7r-linn-0.1.0/bin/linn`.

#### Reproduce the balanced candidate

Build the workspace release evaluator through the flake, then use the prepared
local q4 store or omit `--store` to build one. The captured corpus stays ignored.
The feature settings below are the complete configuration; unset other
experimental flags. `RAYON_NUM_THREADS` controls execution only.

The broader goal remains open. This experiment has demonstrated one additional
case recovery, not general semantic understanding. Two concrete vector-assisted
witnesses so far are `identifier`/`ID` and `auth`/`authentication`; they do not
establish broad paraphrase gains. Future work should distinguish term variation
from truly missing source vocabulary, and candidate admission from final-page
competition, rather than assuming another similarity cutoff will solve both.
Previously closed prefix/lemma and vector-neighbor trials remain closed.


<a id="consolidated-evals-results-record-idf-2026-11-09-md"></a>

### evals/RESULTS-RECORD-IDF-2026-11-09.md

Historical configuration ledger supplement. The combined-study closure above governs: none of this round’s 112 configurations was adopted; the runtime prototypes were removed. Intermediate dev qualification is not adoption.

Uncommitted research continuing configurations 1–68. No adoption, push, commit,
holdout, scale, neutral or hybrid run. All new features remain default-off.
The broader substantial-improvement goal remains open.

#### Result

Combining full record-frequency lexical scoring with the preceding two-anchor
source-group admission candidate reaches **414/439 any, 362/439 first,
cross-scope 32/42 and 26/42, empty 36/51, section 55/73**. Against the applied
POS-enabled default this is +3 any, +11 first, +2/+2 cross-scope, +1 empty,
flat section. Objective rating: dev-qualified aggregate improvement, not an
independent confirmation or an adoption. The combined candidate retains earlier
individual regressions; no claim of universally better pages. It changes 1,050
ordered wording pages against the applied default. Paired totals are gold-any
4 gains/1 loss, gold-first 12 gains/1 loss, cross-scope-any 2 gains/0 losses,
and cross-scope-first 3 gains/1 loss.

The record-frequency control alone regresses. Its benefit is an interaction
with the stronger stem/assembly combination. Against that combination it gains
three first-query gold cases, with no gold-any or gold-first losses. The same
three gains survive the sentence-admission combination. This is more useful
than another isolated semantic cutoff adjustment, but still a modest overall
improvement rather than the requested broad search-success breakthrough.

#### Hypothesis, scope and implementation

The page selects records, while original BM25 rarity counts blocks. A long
record can therefore contribute repeatedly to a term's DF. An illustrative
read-only index audit found `query` in 216 blocks from 77 records, `deployment`
in 127/44, and `option` in 286/85. These illustrative totals include every
indexed row, not the lifecycle-filtered scoring population.

The experiment keeps block TF, heading/body/parts weights, block length and
average length unchanged. Only lexical IDF is blended toward distinct-record
IDF. Record N counts eligible records with indexed rows. Record DF counts each
eligible record once across heading/body/parts. Existing term-constrained SQL
joins supply those counts; no persistent schema, global frequency map or body
surrogate is added. Explicit filters do not narrow the population. Original
block IDFs remain in POS/semantic evidence, rarity gates, confidence and fallback
vocabulary. Coverage, hard literals, tiers and scope penalty 7 are unchanged.

Full stem scoring applies the same record-level DF to its stem/parts posting
union and subtracts the adjusted exact contribution. Adjacent-block lexical
support uses the adjusted exact IDF. Thus the experiment does not accidentally
leave the old exact score in the stem replacement. Weight 0 is the default.
The test covers repeated sections in one record, candidate-state exclusion,
query-filter independence, original semantic IDFs and interpolation arithmetic.
Detailed rules are in LLP 0002.

This is distinct from prior scope-local statistics proposals and closed
vector-fusion, alternate-stemmer and query-rewrite trials. No new model or
term enrichment is involved. The study does not prove that record DF itself
causes all gains: changing lexical score scale also changes its balance with
unchanged POS bonuses and the fixed scope penalty. That causal distinction
remains open and should be controlled before a structural claim is made.

#### Complete native ledger

Every row has all 1,470 q4 dev wording pages and native all-call scoring.
Configurations were written before execution in ignored
`evals/output/system-experiment/configurations.md`.

| # | Configuration | Any/439 | First/439 | XS/42 | Empty/51 | Section/73 | Objective rating |
|---|---|---:|---:|---|---:|---:|---|
| — | Applied default | 411 | 351 | 30/24 | 35 | 55 | Baseline |
| 49 | Previous joint, no sentence assignment | 413 | 358 | 32/26 | 36 | 55 | Prior comparison |
| 64 | Joint plus two-anchor sentence admission | 414 | 359 | 32/26 | 36 | 55 | Prior balanced candidate |
| 69 | Default plus record-IDF weight .5 | 410 | 350 | 30/24 | 35 | 55 | Gold regression |
| 70 | Default plus record-IDF weight 1 | 410 | 350 | 30/23 | 35 | 55 | Gold and XS-first regression |
| 71 | Joint plus record-IDF weight .5 | 413 | 360 | 32/25 | 36 | 55 | First rise, XS-first loss vs 49 |
| 72 | Joint plus record-IDF weight 1 | 413 | 361 | 32/26 | 36 | 55 | Three first gains, no aggregate loss vs 49 |
| 73 | Configuration 64 plus record-IDF weight 1 | 414 | 362 | 32/26 | 36 | 55 | Three first gains, no aggregate loss vs 64 |
| 74 | Joint plus record-IDF weight .75 | 413 | 360 | 32/26 | 36 | 55 | Two first gains, no aggregate loss vs 49 |

The half-weight joint control regresses against 49 in XS-first, although it
still improves the applied default. Do not confuse these comparison bases.
Weight 1 was tested before .75; the latter is a declared intermediate blend,
not a hidden sweep. No h2 data selected these configurations.

#### Paired gains and the fallback interaction

Configuration 72 changes 441 wording pages against 49. Configuration 73 also
changes 441 against 64. In both, the gold case-level movement is precisely three
first-query gains, no any gains/losses and no first losses:

- `s3a-c-0027`: `offline emergency rollback old guidance timing`; gold
  `s3a-r-0060` enters first position.
- `s4-c-0072`: `update native library version changed`; gold `s4-r-0077`
  enters the fifth slot.
- `s6-c-0073`: `embedding-free query path`; the empty page becomes gold
  `s6-r-0086`.

The last is not a newly discovered vector meaning. The unchanged fallback
expands/searches `embedding free`. Its raw candidate membership is unchanged (verified across the traced
queries and both matchers), but
near-match selection chooses row 5934 (the gold note) with record IDF instead
of row 2479 under block IDF. The original-query coverage check then accepts the
gold note, which contains `query` and `path`. The rewritten larger query's gold
row was already first in raw ranks in both runs, but failed the stricter rewrite
checklist; its presence there alone did not guarantee a final card.

Original bodies are untouched and legacy author fields remain unread. The gold
note's historical text says this store has no embedding model; that is corpus
content, not a description of the current implementation. Its labels/terms are
not retrieval evidence. Traces are `record-idf-witness-0.stderr` and `-1.stderr`,
and full cards are the corresponding JSON files. Paired records are retained in
`record-idf-versus-joint.json`, `record-idf-versus-sentence.json`, and each
configuration's baseline-relative `.paired.json`.

#### Cost and validation

The full native timing exports report median/p95 16.31/40.07 ms for configuration
72 and 89.71/388.75 ms for 73. Runs overlap other work; these are measured samples,
not isolated relative speed claims or neutral C2 gates. Objective rating: the
record-frequency control is inexpensive in this sample, while sentence matching
still dominates cost and is not production-ready.

The new executable's default run preserves 411/351, 30/24, 35,55 and exactly all
1,470 baseline page exports. Full workspace tests, strict clippy, formatting,
LLP reference checks and whitespace checks pass. Final Nix build, release package tests and fixup also pass. Scorer, corpus, model and thresholds were not modified.

#### Reproduction

Use the same environment as configuration 64 in
[consolidated finding](#consolidated-evals-results-sentence-2026-11-09-md), adding `LINN_RECORD_IDF_WEIGHT=1`, for the full
candidate. Omit all LINN_SENTENCE_* variables for configuration 72. Use
`target/release/linn-eval --corpus evals/corpus/q4 --mode dev` through the flake,
optionally with the prepared `evals/output/system-experiment/q4.db`, and export
pages/timings to a fresh ignored path. The measured executable is retained as
`evals/output/system-experiment/record-idf-v1-eval`. Unset unrelated experimental
variables. Do not treat reused h2 data as independent holdout confirmation.

#### Fixed synthetic-document transfer check

After freezing all six configurations, the current default and full configuration
73 were run once on the existing synthetic six-document, 49-question fixture.
No tuning followed. Default Hit@1/3/5 is 31/49,35/49,35/49 with MRR@5 0.6735;
the candidate is 33/49,35/49,35/49 with MRR@5 0.6939. The standard 24-question
slice stays 19/24; the deep-research slice rises from 12/25 to 14/25 Hit@1.
Objective rating: improves top rank over the applied default, unchanged coverage;
not independent confirmation and not a matched hybrid win.

The earlier pre-adjacency candidate scored 34/49 Hit@1 and MRR@5 0.7041 here.
The latest combination is therefore one Hit@1 lower than that older experimental
candidate, despite improving q4. Several mechanisms differ, so this check cannot
attribute the loss specifically to record IDF. Preserve that limitation rather
than tuning on this small fixture. Native tables and pages are retained as
`synthetic-record-idf-baseline.*` and `synthetic-record-idf-candidate.*`.


<a id="consolidated-evals-lookup-optimization-2026-11-08-md"></a>

### evals/LOOKUP-OPTIMIZATION-2026-11-08.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Implementation checks complete. The original scale queue was stopped at the
study-owner request and replaced with the optimized executable. Complete old
scale results do not exist, so a full scale before/after claim cannot be made.
This report and all measured outputs are uncommitted.

#### Change

Cache record-to-block ids in ascending persisted id order, their existing
normalized word unions, and the exact section surface-token sets previously
recomputed for hints and related pointers. Recreate them on build, open, add,
rebuild and forget. Reopen uses an exact `(record,start,end)` map instead of
repeatedly scanning all parsed blocks. Install an auxiliary SQL index on
`sections(record,id)` for record-qualified database operations.

The search uses the memory lookup because it is already loaded and was faster
than indexed SQL in the measured lookup test. These caches trade additional
derived memory for avoiding repeated scans and tokenization. They contain only
the existing content-derived values; author fields remain unread. Thresholds,
eligibility, query matching, score arithmetic, embeddings, ordering, excerpts,
and original bytes are unchanged.

#### Objective outcomes

- **PASS — dev counts:** any 410/439, first 350/439; xs 30/42 and 24/42; empty 35/51; section 55/73, exactly the frozen baseline.
- **PASS — dev order:** all 1,470 wording pages have identical ordered record ids, including repeated ids.
- **PASS — h2 counts:** any 341/354, first 294/354; xs 57/62 and 50/62; empty 56/106; section 237/271, exactly the frozen baseline.
- **PASS — h2 order:** all 1,380 wording pages have identical ordered record ids.
- **PASS — full calls:** 24 fixed, evenly spaced original dev queries return identical complete call JSON, including every card, span, excerpt, hint and related pointer. This sample is not a claim of complete full-call parity on all wordings.
- **PASS — synthetic six-document retrieval:** all 49 ordered pages and all group metrics are unchanged; overall Hit@1 31/49, Hit@3 35/49, Hit@5 35/49, MRR@5 0.6735.
- **PASS — scale-q4 prefix:** all 333 complete wording pages captured by the interrupted original run have identical ordered record ids. Complete old scale results do not exist, so full scale parity is not claimed.
- **MEASURED — record lookup:** 12,800 operations per method over the first 128 q4 records; memory averaged 63 ns/op and indexed SQL 5,842 ns/op. Both returned identical ordered row ids. This warm local lookup microbenchmark is not a search-latency gate or a whole-system speedup claim.
- **PASS — lifecycle:** cached values agree with full scans across build, forget, reopen with persisted row-id gaps, add and rebuild; original surface tokens remain distinct from normalized words and labels. Existing version-1 stores regain the auxiliary index on open.
- **PASS — automated checks:** all 25 workspace tests, Clippy with warnings denied, formatting, LLP references, and `nix build` passed. The index-upgrade extension of the lifecycle test also passed separately after being added.

#### Artifacts and limits

Raw outputs, ordered pages, complete-call samples and copied stores are under
`evals/output/lookup-optimization/`. The Rust `lookup_check` example reproduces
the SQL/memory comparison and full-call sample; evals/README.md documents it.

The original executable at `target/release/linn-eval` remains byte-identical to
`evals/output/lookup-optimization/frozen-linn-eval`. The optimization was built
separately at `target/lookup-opt/release/linn-eval`; the replacement queue freezes
a copy at `evals/output/baseline-fixed-20261108T160319Z/linn-eval`.

The optimized scale-q4 run continues in the replacement baseline queue. The
checker waiting for complete old results was canceled when the original run
was stopped. The replacement queue also schedules scale-h2 and all four neutral
tables on the fixed executable; their outcomes will appear in
evals/BASELINE-2026-11-08.md. Background baseline work was active during the
validation above, so local command wall times are not isolated latency
measurements. No result was used to retune retrieval.


<a id="consolidated-evals-query-performance-2026-11-08-md"></a>

### evals/QUERY-PERFORMANCE-2026-11-08.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Uncommitted measurement. Exact execution optimization; no changes to retrieval thresholds, ordering, embedding model, gold spans, or scorer rules.

#### Measured invocation latency

| Corpus | Records | Blocks | Queries | Median ms | p95 ms | Max ms | Store open s | Query total s |
|---|---:|---:|---:|---:|---:|---:|---:|---:|
| q4 | 1376 | 6096 | 1470 | 3.801 | 9.388 | 563.378 | 1.063 | 7.658 |
| h2 | 1473 | 6477 | 1380 | 5.308 | 19.552 | 910.182 | 1.297 | 10.098 |
| scale-q4 | 16906 | 189767 | 1470 | 60.681 | 186.264 | 14429.118 | 51.092 | 131.111 |

Timers cover Store::agent, including first-use caches, excluding corpus loading, grading and export. Store open is separate. This is not neutral C2, which divides returned-call latency differently. Shared machine: old baseline and some builds overlapped these runs. No discarded warmup.

**Objective rating:** q4 resident p95 below 10 ms; h2 below 20 ms; scale-q4 below 200 ms. No new pass thresholds are defined by these descriptive summaries. Scale latency is still corpus-dependent and the worst invocation exceeds 14 seconds.

#### Exactness

- q4: all 1,470 ordered first pages match the original export; any 410/439, first 350/439, xs 30/42 and 24/42, empty 35/51, section 55/73.
- h2, after dev passed on frozen product code: all 1,380 ordered first pages match; any 341/354, first 294/354, xs 57/62 and 50/62, empty 56/106, section 237/271.
- scale-q4: all 1,470 ordered pages match the lookup-only executable; any 389/439, first 315/439, xs 27/42 and 21/42, empty 31/51, section 51/73. Lookup-only had also matched all 333 complete pages available from the interrupted original run. Full original scale parity cannot be claimed.
- Synthetic six-document retrieval: all 49 ordered pages match; Hit@1 31/49, Hit@3 and Hit@5 35/49, MRR@5 0.6735.
- Complete JSON calls on 24 fixed dev originals match, including every call, span, excerpt, hint and related pointer. See `evals/output/query-optimization/complete-calls.log`.
- Indexed SQL and memory record lookups return identical ordered rows. 12,800 lookups each: memory 61 ns/op, SQL 6,406 ns/op. Native FTS term lookup supplies lazy eligible DF; in-memory postings avoid per-row SQL calls.
- 29 workspace tests pass, including numerical bit checks, scan-vs-index equivalence, cache lifecycle, scorer output and timing instrumentation. Formatting, clippy, LLP checks and nix build pass.

#### What changed

Record-to-block and structural inverted lookups; cached surface tokenization and query normalization; candidate-only shared BM25 and term-constrained native FTS vocabulary DF; reused packed float32 matrices; occurrence-preserving section means; equivalent partial selection instead of full sorting. Cache invalidation follows store lifecycle. No author fields are read.

#### Remaining costs

Store open still eagerly loads and reconstructs derived state. Scale open took 51 seconds and the process used substantial memory. First use of model/form/sentence caches can take seconds. Exact dense vector stages still depend on corpus size, and broad lexical posting lists grow with matching data. This is a large improvement in repeated search execution, not constant-time end-to-end CLI search.

The baseline queue now uses the verified frozen binary under `evals/output/baseline-query-20261108/`. Progress and one-line objective summaries remain in [consolidated finding](#consolidated-evals-baseline-2026-11-08-md). Original and lookup-only reports/output were preserved. Nothing was pushed or committed.


<a id="consolidated-evals-startup-performance-2026-11-08-md"></a>

### evals/STARTUP-PERFORMANCE-2026-11-08.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Historical stage report. The later numerical-cache migration and final fresh-process measurements are in [CLI-LATENCY-2026-11-08.md](#consolidated-evals-cli-latency-2026-11-08-md).

Uncommitted measurement of an existing local scale store: 16,906 records, 189,767 blocks. Warm OS pages; other baseline jobs/builds shared the machine. No daemon, second data file, new embedding model or ranking change. Before is commit 39707af plus phase instrumentation; after is the working tree.

#### Opening breakdown

| Phase | Before seconds | After seconds |
|---|---:|---:|
| FTS reads and feature maps | 19.278 | 0.380 |
| Block parsing and surface DF | 7.487 | 5.032 |
| Derived postings, links and trees | 14.015 | 6.709 |
| Other open work | 0.674 | 0.658 |
| Total | 41.454 | 12.778 |

**Objective rating:** startup reduced by 3.2x in the phase profile; it remains corpus-dependent and too expensive for a fast CLI.

These are mixed end-to-end phase times, not all pure SQL. Records include JSON decoding, vectors include decoding, and derived phases are Rust construction. Native database connection, PRAGMAs, schema and scratch setup took 5.6 ms before and 4.6 ms after.

#### libSQL-only diagnostics

The Rust example steps rows and extracts values, without parsing bodies or building search maps. It returned 25,083,572 exact-token instances and 12,170,078 stem instances. Separate timers measure preparation, next-row and extraction; total also includes loop/timer overhead. They are not additive to the full opening profile because this is a separate run.

| Configuration/trial | Total seconds | Row step seconds | Value extraction seconds |
|---|---:|---:|---:|
| sql-default | 10.642 | 5.445 | 3.318 |
| sql-default-repeat | 11.143 | 5.706 | 3.416 |
| sql-tuned | 10.041 | 5.027 | 3.191 |
| sql-tuned-repeat | 10.900 | 5.503 | 3.429 |

Default: cache_size=-2000 KiB, mmap_size=0, 4096-byte pages, delete journal. Tuned diagnostic connection: cache_size=-262144 KiB, requested 2048 MiB mapping, actual mmap_size=2147418112 bytes. Orders were default/tuned and then tuned/default. Secure deletion and durability were unchanged.

**Objective rating:** mean raw scan 10.892 s default versus 10.470 s tuned (3.9% reduction), a small and noisy effect compared with avoiding the scans. Product PRAGMA defaults were left unchanged.

#### Implemented changes

- Use the native FTS distinct-term vocabulary and stored FTS5 column sizes at open. Read exact term frequencies/body offsets and stem offsets with term-constrained SQL only when needed. Cache per Store, clear on lifecycle changes. No full token-position map at startup.
- Reuse the same block surface tokenization for normalized block words.
- Resolve links by record id and inverse source path instead of scanning every record per source. Preserve deliberately retained link state after forget.
- Group heading trees by path prefix, preserving tree order, row order and exact membership.

#### Fresh CLI invocations

`result/bin/linn --store <scale-store> --json search git`, separate processes with no startup instrumentation:
- Trial 1: 13.423 s, exit 0.
- Trial 2: 12.642 s, exit 0.

**Objective rating:** both CLI invocations complete below 14 seconds, still far above an interactive target. These wall times include process startup, first query, output capture and teardown, unlike resident query timers.

#### Exactness and checks

- Dev: all 1,470 ordered pages identical; any 410/439, first 350/439, xs 30/42 and 24/42, empty 35/51, section 55/73.
- Holdout, after dev passed on frozen product code: all 1,380 ordered pages identical; any 341/354, first 294/354, xs 57/62 and 50/62, empty 56/106, section 237/271.
- Scale-q4: all 1,470 ordered pages identical; any 389/439, first 315/439, xs 27/42 and 21/42, empty 31/51, section 51/73.
- Complete calls identical on 24 fixed dev samples, including all spans, excerpts, hints and related pointers.
- Native docsize lengths, lazy frequencies/positions, repeated loading, lifecycle invalidation and confidence float bits compared with full instance scans in committed-source tests. Link/tree construction compared with the previous scan rules, including duplicate source paths and deliberately retained links after forget.
- 31 workspace tests, clippy, formatting and LLP checks pass. Nix release build passes, including the final lifecycle/link fixture assertions.

#### Remaining architecture

The store still eagerly parses every body and constructs block-word sets, section/record postings and heading trees in Rust. Those account for most remaining startup. The next architectural step is durable, indexed libSQL structural projections maintained on writes and read as candidates, rather than rebuilding full-corpus maps or introducing a parallel snapshot file. Small candidate-local Rust maps and exact numerical kernels remain useful; SQL is not automatically faster for every operation.

At this stage the original full baseline queue was still running on its identified frozen executable; these timings were separate. Corpora, model weights, raw reports, stdout and timing output remain uncommitted.


<a id="consolidated-evals-structural-migration-2026-11-08-md"></a>

### evals/STRUCTURAL-MIGRATION-2026-11-08.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Historical stage report. The later numerical-cache migration and final fresh-process measurements are in [CLI-LATENCY-2026-11-08.md](#consolidated-evals-cli-latency-2026-11-08-md).

Uncommitted report. One local libSQL file, one product binary, no daemon and no retrieval retuning. Starting source: commit 39707af plus the prior startup work. Corpora and all output stay uncommitted.

#### Fresh process wall time

Three separate native CLI invocations per corpus, `linn --store <copied-store> --json search git`. No hidden warmup; OS pages were warm. Shared machine. Each result includes process startup, opening, first query, stdout capture and teardown. This query exercises the lexical path; it is not a worst-case fallback benchmark.

| Corpus | Documents | Median seconds | Range seconds |
|---|---:|---:|---:|
| q4 | 1376 | 0.072 | 0.072–0.094 |
| scale-q4 | 16906 | 0.988 | 0.974–0.988 |

**Objective rating:** the measured lexical CLI invocation is under 100 ms on q4 and under one second on scale-q4 in these trials. These are descriptions of this workload, not newly adopted latency thresholds. Prior scale trials took 12.642 and 13.423 seconds after the first startup optimization.

#### Open and first use

The scale phase profile is saved in `evals/output/structural-migration/scale-q4-final-open.stderr`. Opening took 1.010 seconds with instrumentation and concurrent work; the full scorer recorded 0.880 seconds. Curated scorer open times were 45 ms (q4) and 96 ms (h2). Timers exclude corpus JSONL loading. Vectors and the FTS vocabulary are deferred to first use, not omitted from query timings.

| Existing-store projection build | Seconds |
|---|---:|
| q4 | 1.163 |
| h2 | 1.282 |
| scale-q4 | 48.846 |
| scale-h2 | 44.629 |

**Objective rating:** migration cost is paid once per old/invalidated store. The scale projection publication itself took about 29–33 seconds in these samples. Subsequent opens do not repeat parsing/tokenization or structural map construction. The persisted SQL indexes add disk space; they do not store a second copy of record body text.

#### What is persisted and read

- A shared surface/normalized word dictionary and indexed block/section postings. Section flags distinguish own text from heading-path/label; author fields remain unread.
- Original parsed section/block metadata, exact tree membership/order, surface DF and first-encounter word order, and resolved links.
- Record/tree word unions are obtained from indexed block memberships, avoiding separate copies. Query-local Rust sets contain only loaded words.
- Core writes invalidate the publication marker. Complete projections publish atomically after the core write. Record deletion clears derived content in the same transaction, preserving secure forget even for older writers.
- Construction, normal open and agent calls verify version/publication consistency. Read snapshots prevent mixed generations. A stale Store must reopen before searching or writing.
- No daemon, sidecar data file, model swap, query rewriting change, threshold change or card-order change.

#### Exact result checks

All commands exited zero and compared complete selected first-page key sets, not only aggregate scores:
- **q4: PASS**, all 1470 ordered pages identical. any 410/439 first 350/439 xs 30/42 24/42; empty 35/51 section 55/73.
- **h2: PASS**, all 1380 ordered pages identical. any 341/354 first 294/354 xs 57/62 50/62; empty 56/106 section 237/271.
- **scale-q4: PASS**, all 1470 ordered pages identical. any 389/439 first 315/439 xs 27/42 21/42; empty 31/51 section 51/73.
- **scale-h2: PASS**, all 1380 ordered pages identical. any 326/354 first 273/354 xs 49/62 40/62; empty 48/106 section 225/271.
- **Synthetic six-document fixture: PASS**, all 49 ordered pages identical; original Hit@1/3/5 and MRR unchanged.
- **Full-call sample: PASS**, 24 fixed dev originals, including all calls, cards, spans, excerpts, hints and related pointers.
- **Neutral q4 dev: MEASURED**, all non-latency values match the earlier full table; latency p95 is 3.410 ms, below its existing 50 ms bar. Existing G1, G2, G4, Q1 and R2 failures are unchanged and do not authorize retuning.
- **Checks: PASS**, 33 workspace tests, formatting, clippy, LLP references and Nix release build. Tests compare lazy SQL reads to full scans, confidence float bits, row-id gaps, rollback, stale publications, external writes and secure projection deletion.

#### Dependency capability checks

The product remains on libsql 0.9.30 with local-only core features. Its in-memory Rust probe reports SQLite 3.45.1, ordinary views supported, and `CREATE MATERIALIZED VIEW` rejected with `near "MATERIALIZED": syntax error`.

The crates.io registry reports libsql 0.10.0 as the latest stable, published 2026-10-13. An isolated Rust project outside this repository compiled that version with core-only features. It reports SQLite 3.47.0, the same materialized-view syntax error, and working FTS5 unicode61/Porter table creation. This was not a product dependency upgrade.

An alternative SQLite-compatible engine documents automatic incremental materialized views; that is not a capability supplied by either libSQL version probed here. The checked projection tables are manually maintained materialized results within the existing database.

#### Limits

The store still reads all envelopes and thin block/section metadata at open; that cost remains corpus-dependent. Cold empty-page stages still scan and encode corpus-derived forms, proposals or sentences and can take seconds (the scale query maxima remain roughly 14–15 seconds). Exact dense vector kernels still depend on corpus size. The fast lexical CLI samples must not be generalized to those paths.

The original full baseline run completed on its own frozen executable; its version and measurements remain in [consolidated finding](#consolidated-evals-baseline-2026-11-08-md). This report and raw output record the subsequent migration separately.


<a id="consolidated-evals-cli-latency-2026-11-08-md"></a>

### evals/CLI-LATENCY-2026-11-08.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Uncommitted performance report. Source commit: `129897f950208faeefaa43ddd10f55537e46c092`. The product remains a freestanding native CLI using local-file libSQL 0.9.30 and the same packaged model. No daemon, alternate embedding model, reranker, author terms, retrieval tuning or threshold changes.

#### Fresh packaged CLI sample

Each wording runs in a new `result/bin/linn` process. Wall time includes process creation, store open, first query, output capture and teardown. These final samples ran serially after the full-corpus jobs/builds stopped. OS pages were warm; process caches were cold. The CLI uses its real eligibility clock and caller repository scope, not the scorer context.

Selection: two evenly spaced original dev queries per identifier, phrase, doc-passage, lateral, paraphrase and no-gold group, deduplicated by case id, plus focused dev case s1b-c-0018. Deduplication yields 12 queries per corpus. No alternate queries are supplied implicitly. This is a fixed sample, not a production percentile distribution or a neutral C2 measurement.

| Corpus | Documents | Invocations | Median ms | p95 ms | Maximum ms |
|---|---:|---:|---:|---:|---:|
| q4 | 1376 | 12 | 73.686 | 261.568 | 281.851 |
| scale-q4 | 16906 | 12 | 986.119 | 1927.623 | 3248.335 |

**Objective summary:** all 24 CLI processes completed with supported exit codes; q4 sample p95 is below 300 ms, scale sample p95 is below 2 seconds, and the scale maximum is below 3.3 seconds. These are descriptive outcomes, not new acceptance gates.

#### Individual requests

| Case | Group | q4 ms | Scale ms |
|---|---|---:|---:|
| s1b-c-0002 | identifier | 71.110 | 927.055 |
| s6-c-0121 | identifier | 71.489 | 982.769 |
| s1-c-0001 | phrase | 261.568 | 1927.623 |
| s6-c-0119 | phrase | 67.502 | 1008.003 |
| s1-c-0009 | doc-passage | 69.770 | 978.875 |
| s6-c-0128 | doc-passage | 73.637 | 989.468 |
| s1b-c-0043 | lateral | 85.530 | 1143.874 |
| s1-c-0002 | paraphrase | 75.313 | 932.809 |
| s6-c-0091 | paraphrase | 66.155 | 907.692 |
| s1b-c-0004 | no-gold | 127.848 | 1180.764 |
| s6-c-0125 | no-gold | 73.735 | 980.085 |
| s1b-c-0018 | focused | 281.851 | 3248.335 |

The initial structural-only scale sample reached 17,062 ms on s1b-c-0018. The same focused request now takes 3,248 ms in the isolated final sample. Separate before/after comparisons on seven fixed requests found identical semantic CLI JSON and exit codes; only the clock-bearing cursor was excluded. Earlier timing comparisons had shared-machine contention, so no precise universal speedup ratio is claimed.

#### Honest silence

Curated h2 case s7-c-0060, original query `blob-util remove Windows next major`: 526.679 ms, exit 1, no cards. Complete semantic output matches the saved pre-numerical-cache CLI. On scale-h2, the same query retains its existing distractor hit x-r-03059; that is unchanged baseline behavior, not a newly accepted answer or a reason to retune.

**Objective summary:** the protected curated silence stays empty; scale behavior stays unchanged.

#### Remaining migration completed

- Structural metadata, surface-word postings, exact tree membership, word order/DF and links are indexed projections in the same SQL database. Normal opens avoid corpus-wide tokenization and rebuilding Rust word maps.
- Query-independent form, beam-proposal and sentence encodings are now prepared during writes/rebuilds or a one-time upgrade of older stores. Queries load the original float32 bits from chunked SQL projections, without re-encoding or renormalizing these arrays.
- Model identity is a build-time fingerprint of the packaged config/tokenizer/weights plus an encoder-contract version. Numerical data is also tied to the current structural publication. Both identities must match.
- Record deletion securely clears numerical and structural derived data in the same transaction. Read snapshots, writer checks and publication identity prevent mixing generations.
- Block vectors load directly into the original ordered packed matrix, avoiding a separate full per-block vector-map allocation on the search path. Beam accumulation reads its packed rows rather than a duplicate vector array.

**Objective summary:** repeated CLI invocations reuse durable derived data; no parallel data file, corpus enrichment or new retrieval behavior was introduced.

#### Cost shifted to writes and disk

| Scale numerical projection | Rows | Chunks | Payload bytes |
|---|---:|---:|---:|
| beam | 64863 | 4 | 132839424 |
| forms | 79980 | 5 | 163799040 |
| sentences | 932056 | 57 | 1908850688 |

Numerical payload total: 2,205,489,152 bytes (2.205 GB), excluding metadata/index overhead. Matrices are chunked at 16,384 rows (32 MiB) to stay below per-value limits.

**Objective summary:** scale numerical preparation took about 19 seconds for scale-q4 and 29 seconds for scale-h2, once; payload storage increases by about 2.2 GB. Preparation is not hidden in the repeated-invocation timings.

#### Verification

- Dev and holdout floors remain unchanged; final frozen curated runs are saved separately under this report’s output directory.
- All 1,470 scale-q4 and 1,380 scale-h2 ordered first pages match their saved exports. Their respective measured scores remain 389/439 and 315/439, xs 27/42 and 21/42, empty 31/51, section 51/73; and 326/354 and 273/354, xs 49/62 and 40/62, empty 48/106, section 225/271.
- The synthetic six-document fixture matches all 49 ordered pages.
- All calls, cards, spans, excerpts, hints and related pointers match on 24 fixed dev originals.
- Full q4 neutral run: every non-latency value is unchanged; latency p95 is 3.110 ms and passes the existing 50 ms bar. Existing G1, G2, G4, Q1 and R2 failures are retained as measurements.
- 35 workspace tests pass, including computed-versus-persisted fallback arrays, float-bit and chunk-boundary round trips, model/generation guards, lazy reads, secure forget, rollback and stale-writer rejection. Formatting, clippy, LLP reference checks and Nix release build pass.

**Objective summary:** exactness and required checks pass; no retrieval threshold or card order was changed.

#### Limits

Store open still reads all envelopes and thin block metadata, so its cost grows with corpus size. Exact global vector comparisons still load sizeable arrays and perform corpus-sized work. Query text still needs the packaged static model. The final scale maximum remains seconds; this is not constant-time search or a claim that every invocation completes under one second.

The earlier baseline and structural-only reports are historical measurements of their named frozen executables. Raw JSONL, stdout/stderr, SQLite stores, reports and corpus files remain local and uncommitted. Nothing has been pushed.


<a id="consolidated-evals-latency-2026-11-09-md"></a>

### evals/LATENCY-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

The final subset-loading results below supersede the initial-stage timings. Both stages preserve retrieval.

Local, uncommitted measurement report. Accepted baseline: e308de7. Candidate: working-tree latency changes; no ranking constants or scoring rules changed.

#### Changes

- Maintain an indexed POS pair projection in the same libSQL database. Skip query tagging only when a current, complete publication proves no source relation could contribute. Invalid or missing publications conservatively keep the original path. Writes and forget invalidate the publication transactionally.
- Read surface document frequencies with requested postings. Load the full ordered dictionary only when rebuilding a fallback proposal cache.
- Borrow the aligned embedded F32 tensor through model2vec-rs rather than copying its 129,210,368 weight bytes; parse the tokenizer once. Other layouts retain the checked owned decoder. Vector bits and artifact identity remain unchanged.
- Optional stderr startup timings separate path resolution, store opening, scope/context and command/output.

#### Fresh CLI comparison

`evals/examples/cli_latency_compare.rs` compares 11 fixed queries spanning six categories, three repetitions each, alternating executable order. OS caches are warm; each query starts a fresh process. Cursors retain their presence but their live-clock payload is normalized for comparison. All other response fields and exit codes must match.

| Corpus / configuration | Before median | After median | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| q4, accepted POS default | 226.94 ms | 72.40 ms | 346.14 ms | 227.83 ms |
| scale-q4, POS disabled in both | 993.03 ms | 900.83 ms | 1915.63 ms | 1837.46 ms |

Objective rating: q4 fresh-process median improved 68%; all 33 paired responses matched.

Objective rating: scale fresh-process median improved 9%; all 33 paired responses matched. This legacy scale index lacks POS publication, so this is not a default-POS scale validation.

The descriptive p95 uses sorted index floor(0.95 * (n - 1)); this is not a neutral gate run.

#### Resident query comparison

Three complete q4 dev runs per binary (1,470 wordings each), excluding startup and grading:

| Run | Before median / p95 | After median / p95 |
|---|---:|---:|
| 1 | 4.941 / 12.314 ms | 5.137 / 12.972 ms |
| 2 | 4.984 / 12.316 ms | 4.686 / 11.905 ms |
| 3 | 5.204 / 13.006 ms | 4.731 / 12.158 ms |

Objective rating: resident performance remains approximately flat; no consistent large gain is claimed.

#### Startup attribution and remaining costs

Representative q4 final trace: store open 24.7 ms; scope/context 17.5 ms. Queries requiring the POS tagger still deserialize it in roughly 143–170 ms. Eight of the 11 sampled queries avoid it. Model initialization traces improve from roughly 56 ms to 39 ms by borrowing the same embedded tensor.

Scale trace: SQLite open/schema 0.77 ms; record read/decode 122.6 ms; validity 16.8 ms; structural metadata 504.1 ms; FTS lengths 40.1 ms; fallback artifacts 8.8 ms; total store open about 699 ms. The dominant remaining cost is decoding and allocating corpus-wide record/block metadata, not opening libSQL. Removing that requires further lazy-loading work.

Objective rating: startup remains corpus-size dependent; these changes do not establish constant-time CLI startup or a universal 50 ms query budget.

#### Retrieval and packaging checks

- q4: all 1,470 ordered pages identical; any 411/439, first 351/439, xs 30/42 and 24/42, empty 35/51, section 55/73.
- h2: all 1,380 exported pages byte-identical; any 341/354, first 295/354, xs 57/62 and 50/62, empty 56/106, section 239/271. Performance parity check, not new retrieval selection.
- Borrowed versus owned encoders: exact float32 bit equality for all q4 blocks and prepared queries, plus Unicode, empty and truncation fixtures.
- Complete-response parity on 24 fixed dev queries: all calls, cards, spans, excerpts, hints and related pointers identical.
- Workspace tests: 45 passed, one ignored; formatting, strict workspace Clippy, LLP references and diff whitespace checks passed.
- nlprule already uses compile-time include_bytes. A fresh CLI process successfully tagged a query outside the checkout with model and POS resource variables set to nonexistent paths. Runtime deserializes embedded bytes; it does not open a linguistic resource file.

Objective rating: measured retrieval pages unchanged; no quality gain or retuning claimed.

#### Reproduction artifacts

Ignored artifacts are under `evals/output/latency-round/`: saved before/final binaries, paired `q4-final-cli.json` and `scale-final-cli.json`, resident timing JSONL, page exports, build/check logs, and embedded-POS smoke diagnostics. Corpus and model files remain uncommitted.

Run the paired CLI example with `--help` for its required corpus, page-export, executable and store arguments. Full h2 parity command:

#### Final subset-loading migration

The second stage removes corpus-wide startup materialization. It reads compact
record headers and exact per-record block/FTS-length totals. Bodies, sections,
blocks, heading trees, links and quoted basenames are fetched by indexed keys.
Header body length and stripped-nonempty are distinct, preserving spaces-only
word-search eligibility. Unloaded blocks never change corpus counts or IDF.
Original hash references use the stored hash index; source-path neighbors use
an expression index; incoming links have a reverse index.

The new `record_headers`, `record_index_totals` and `record_basenames` tables
are derived ingestion indexes in the same libSQL file. Writes publish them;
legacy files perform a one-time guarded migration. Header/name invalidation and
secure deletion follow record writes, while totals share structural publication.
The auxiliary tables contain no authored evidence or search enrichment.

##### Final fresh CLI measurements

Same 11 queries, alternating order, three repetitions per binary, warm OS cache:

| Corpus / configuration | Before median | After median | Before p95 | After p95 |
|---|---:|---:|---:|---:|
| q4, accepted POS default | 215.93 ms | 40.48 ms | 345.06 ms | 200.55 ms |
| scale-q4, POS disabled in both | 991.69 ms | 331.81 ms | 1899.41 ms | 1533.38 ms |

Objective rating: q4 median improved 81%; all 33 paired responses and exits match.

Objective rating: scale median improved 67%; all 33 paired responses and exits match. This is the legacy POS-disabled scale index, not default-POS scale validation.

Representative final startup trace:

| Stage | q4 | scale-q4 |
|---|---:|---:|
| libSQL open/schema/scratch | 0.79 ms | 0.80 ms |
| Record headers | 1.85 ms | 20.45 ms |
| Stored validity | 0.02 ms | 0.04 ms |
| Structural totals and small lookups | 0.49 ms | 5.52 ms |
| Complete store open | 3.24 ms | 26.91 ms |
| CLI scope/context | 13.84 ms | 14.12 ms |

Objective rating: the formerly ~699 ms scale startup is ~27 ms in this trace. Startup still reads compact headers/totals proportional to record count; it is not constant-time.

##### Which operations actually need all data?

- Ordinary lexical/navigation lookup: subsets of original blocks, sections,
  trees, links and names only, through existing or new SQL indexes.
- Global scoring denominators and lifecycle predicates: compact headers plus
  stored integer totals, without original bodies or block-object allocation.
- Exact numerical fallback: complete ordered matrices and block-to-record
  mappings where the algorithm scores every row. Source bodies still load only
  when needed. Scale has 189,767 block vectors; one slow trace spends ~418 ms
  reading the block matrix, ~112 ms reading beam vectors, and ~164 ms reading
  forms. These broad numerical stages remain the long-tail cost.
- Ingest/rebuild/publication: full original-text derivation, as required by the
  existing ordering, encoding batch boundaries and corpus-derived arrays.
- Forget: retains the original full surviving-index regeneration semantics,
  transaction checks and secure-delete behavior. No retrieval change is hidden
  in incremental-write work.
- Explicit conformance diagnostics can request full materialization; normal
  opens do not. The lookup diagnostic explicitly hydrates the records it times.

Objective rating: no claim that every query meets 50 ms; the exact fallback tail remains substantial.

##### Final correctness evidence

- Reopened q4: 1,470 pages identical; all six accepted metrics unchanged.
- Reopened h2: 1,380 pages identical; all six accepted metrics unchanged.
- All complete calls match on 24 fixed dev queries, including excerpts, hints,
  related pointers and spans.
- Paired fresh CLI responses match for both q4 and scale samples.
- The governing rules and the global-operation exceptions are in LLP 0001,
  with the contributor rule in AGENTS.md. No scoring thresholds changed.

Raw artifacts for this stage are ignored under `evals/output/subset-loading/`.
No corpus, model or benchmark output is staged or committed.

##### Final resident timings and checks

Three complete 1,470-wording runs per binary, after builds completed:

| Run | Before median / p95 | After median / p95 |
|---|---:|---:|
| 1 | 5.048 / 12.134 ms | 4.789 / 12.792 ms |
| 2 | 5.045 / 12.156 ms | 4.726 / 12.670 ms |
| 3 | 5.047 / 12.345 ms | 4.712 / 12.436 ms |

Objective rating: resident latency remains approximately unchanged; the large gain is fresh CLI startup.

Complete response traces now match on all 1,470 dev and 1,380 h2 wordings—not merely page ids or a sample. All calls, cards, text, spans, hints and related pointers compare equal. All 45 workspace tests passed (one ignored), as did formatting, strict workspace Clippy, LLP reference checks, and the Nix package build including its no-Nix-path/dependency checks.


<a id="consolidated-evals-nlp-runtime-2026-11-09-md"></a>

### evals/NLP-RUNTIME-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Local, uncommitted measurement report. Raw artifacts are under
`evals/output/scale-index-round/`. No retrieval adoption or threshold change.

#### Changes

- Removed the temporary `vendor/nlprule` fork and the product/build dependency
  on nlprule. `src/nlp` contains the required annotation runtime; the unchanged
  crate remains in the eval package for differential checks.
- Build a reproducible, embedded, lookup-ready linguistic asset. Use the
  dictionary FST, flat spelling/tag arrays, and sorted word-membership arrays
  instead of rebuilding large bidirectional maps and hash sets at startup.
- Evaluate disambiguation rules in their original order without speculative
  parallel work. Bound group-token scans; cache exclusion ranges per immutable
  sentence/rule; avoid revisiting permanently rejected or masked positions.
  Preserve repeated zero-width matches and the original chunker arithmetic.
- Compute character offsets in a forward pass. Correct the existing protected
  atom boundary panic on multibyte whitespace, with a regression fixture.
- Read candidate metadata separately from original bytes. Fetch selected byte
  slices inside libSQL and aggregate ordered term offsets there. Search scoring,
  candidate cuts, numerical kernels and card assembly remain unchanged.
- Consolidate the three original code/data license texts and references into
  `src/nlp/LICENSE`; package it as `NLP-LICENSE`.

#### Fresh CLI latency

29 queries sampled across identifier, phrase, passage, lateral, paraphrase and
no-gold groups; three timed fresh processes per query and binary (87 samples
each). Before/after order alternates. Timing includes process spawn, captured
JSON output and exit; it excludes the development shell. Filesystem cache is
warm; no daemon. Default thread settings. Background validation jobs were active,
so these are paired local measurements, not isolated hardware benchmarks.

| Corpus | Median before | Median after | p95 before | p95 after |
|---|---:|---:|---:|---:|
| q4, 1,376 records | 166.54 ms | 91.43 ms | 198.49 ms | 125.79 ms |
| scale-q4, 16,906 records | 467.42 ms | 328.83 ms | 1,134.37 ms | 888.42 ms |

Objective rating: q4 median improves 45.1%, scale median improves 29.7%; both
p95 values improve, and every paired CLI response and exit status matches.
Only the clock-bearing cursor value is normalized for CLI comparison. A fourth,
traced round is excluded from latency statistics.

These workloads are representative samples, not the distribution of every eval
wording. Scale latency is still materially higher. The existing full-corpus
fallback work remains expensive; this change does not establish constant-time
search or a completed scale floor run.

#### Correctness

- Final q4: all 1,470 complete responses match the pre-change capture.
  `any 411/439 first 351/439 xs 30/42 24/42`,
  `empty 35/51 section 55/73`.
- Focused-runtime h2 replay: all 1,380 complete responses match.
  `any 341/354 first 295/354 xs 57/62 50/62`,
  `empty 56/106 section 239/271`.
- Final compact-membership implementation: complete token, tag, chunk and span
  output matches the unchanged engine on 24,736 q4 query/source/fixture inputs
  and 1,387 h2 query/fixture inputs. The build asset also passes independent
  decode/compile byte-reproducibility checks.
- Rebuilt scale source projection: all 189,767 `pos_relations` rows and all
  470,014 `pos_pair_lookup` rows match byte for byte. This rebuild used the
  focused runtime before the final exact-membership container compaction;
  the latter is additionally covered by the differential checks above.
- Full paired scale retrieval replays were stopped after 346 complete responses
  each; every completed response matches. These are partial exports, not scale
  scores. The uncompleted wordings have not been claimed as verified.

Objective rating: no observed linguistic, card, ordering or retrieval-score
regression; full scale retrieval verification remains incomplete.

#### Checks and reproduction

Format, strict workspace Clippy, workspace tests (49 passed, one ignored), LLP
references and Nix packaging passed. Product/build dependency output contains no
nlprule crate. The packaged binary passes the existing freestanding checks.
All original license texts remain verbatim in the combined license file.

The original binary snapshots are `before-cli` and `before-eval` under the
artifact directory. Useful artifacts:

- `summary.json`, `q4-final-cli.json`, `scale-final-cli.json`;
- `q4-final.calls.jsonl`, `h2-nlp.calls.jsonl`;
- `nlp-final-analysis.stdout`, `nlp-final-h2.stdout`;
- `scale-pos-parity.json`, `scale-partial-parity.json`;
- `nlp-tests.stdout`, `nlp-final-clippy.stderr`, `nlp-final-llp.stdout`,
  `nlp-final-nix.stderr`, `product-dependencies.txt`.

The differential analysis example accepts `--cases` and `--units` JSONL inputs.
`pos_units` exports original source units for that check. The paired CLI example
is `cli_latency_compare`, with `--per-group 5 --rounds 3`; use the same corpus,
page manifest, store and binary pair for both sides. Run development commands
through the flake. Corpus, generated assets and outputs stay uncommitted.


<a id="consolidated-evals-sql-performance-2026-11-09-md"></a>

### evals/SQL-PERFORMANCE-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Local, uncommitted measurement report. Artifacts are in
`evals/output/sql-performance-round/`. Retrieval algorithms, model, thresholds,
card ordering and scoring definitions are unchanged.

#### Implemented

- Full nearest-vector walks retain the original dense matrix and arithmetic,
  but carry compact record ordinals rather than constructing a Block, heading
  path and token containers for every matrix row. Selected hints read block
  metadata and bytes through existing indexes.
- Rare-form/block comparisons read only the claimed form's vector rows through
  the existing vector primary key. They retain the same float32 bytes and vector
  dot kernel. An already-loaded global matrix supplies those rows without another
  SQL read. No new persistent vector copy or candidate cap.
- Set a fixed connection-local `mmap_size=268435456` (256 MiB). Keep the default
  page-cache budget and existing secure deletion, foreign keys, busy timeout,
  temporary-memory storage and durability. This is a native pager mapping, not
  another file, daemon, preloaded document cache or runtime algorithm option.
- Add `sql_read_profile`, a read-only native libSQL diagnostic for effective
  settings, EXPLAIN QUERY PLAN and row extraction. It accepts a store path,
  `--cache-kib`, `--mmap-mib`, and `--rounds`; these are diagnostic options only.

#### Audit of the five questions

1. Excess work: the global block-metadata construction and unconditional global
   vector load for rare-form comparisons were unnecessary. They are removed.
   Ordinary startup schema/scratch setup is already about a millisecond in the
   traced scale invocations. Reading and decoding all record headers and totals
   remains a measurable startup cost.
2. Data access: bodies, section JSON, paths, links, POS relations and candidate
   vectors use existing libSQL indexes on demand. Compact record headers/totals
   remain resident for lifecycle, scope penalties and corpus arithmetic. Some
   fallbacks still read full ordered numerical arrays; their global selection,
   kernel shapes and float32 reductions are part of existing behavior. This
   change does not establish constant-time search as the corpus grows.
3. Derived tables: no table was added. FTS supplies token counts, postings and
   Porter tokenization. The source-path expression index and trigger-maintained
   POS-pair lookup already use native SQL features. Custom section cuts, Unicode
   normalization and NLP relations still require the specified derived indexing;
   replacing them with a view does not make that work disappear. The pinned
   engine has no automatically maintained materialized-view facility. No safe,
   measured table replacement was established in this pass.
4. Indexes: the audited surface-word, normalized-word, tree-member, section,
   basename, backlink, source-path, POS-block and selected-vector accesses use
   existing B-tree/primary-key indexes. Term positions use the term-constrained
   FTS vocabulary virtual-table index. DISTINCT, GROUP BY and explicit ordering
   still require temporary B-trees in some plans. Full matrix scans intentionally
   visit every row; another index does not eliminate that population. No missing
   read-path index was established by these probes, so none was added.
5. Settings: the linked issue concerned using a single-statement query API for
   multiple PRAGMAs. Linn already uses execute_batch. Cache and mapping trials
   are recorded below. WAL and synchronous=NORMAL were not copied from an insert
   microbenchmark: this pass does not weaken write durability or change the
   checkpoint/forget contract to improve a read measurement.

References:
- [libSQL issue and corrected PRAGMA execution](https://github.com/tursodatabase/libsql/issues/1458#issuecomment-2156742384).
- [SQLite mmap_size](https://www.sqlite.org/pragma.html#pragma_mmap_size).
- [SQLite cache_size](https://www.sqlite.org/pragma.html#pragma_cache_size).

#### Settings experiments

Read-only native libSQL profiles used fresh connections, three rounds per
configuration, and the same scale database. The first round is excluded from
these warm medians. This is sequential screening, not a statistically isolated
benchmark; paired full CLI comparisons follow it.

| Cache KiB | Mapping MiB requested | Headers ms | Blocks ms | Beam bytes ms |
|---:|---:|---:|---:|---:|
| 2,000 | 0 | 6.031 | 190.054 | 104.174 |
| 16,000 | 0 | 6.613 | 207.225 | 104.693 |
| 2,000 | 256 | 5.755 | 177.413 | 102.153 |
| 2,000 | 2,048 | 5.862 | 183.350 | 96.899 |

The requested 2,048 MiB mapping was capped by the engine at 2,147,418,112 bytes.
The 16 MiB cache did not improve these reads. The larger mapping did not improve
all stages enough to justify choosing it over 256 MiB.

The isolated 256 MiB mapping comparison used identical candidate binaries apart
from that setting, with 29 queries across six categories, three timed fresh
processes per query and side, alternating order. All responses and exit statuses
matched; only the clock-bearing cursor value was normalized.

| Corpus | Median without / with mapping | p95 without / with mapping |
|---|---:|---:|
| q4 | 83.72 / 81.63 ms | 113.24 / 110.56 ms |
| scale-q4 | 318.42 / 302.14 ms | 811.63 / 747.90 ms |

Objective rating: the paired mapping trial improves both reported latency
summaries on both samples, without response changes; keep the 256 MiB limit.
The cache and 2 GiB mapping trials are not adopted.

#### Final validation and latency

Final packaged binaries were measured on the same 29-query category sample,
three timed fresh invocations each (87 per binary and corpus), alternating order.
A separate traced round is excluded from these latency summaries. No other
validation or build job from this pass ran alongside the timed samples. Times
include process spawn, JSON output and exit, with warm filesystem caches and no
daemon. Percentiles use nearest rank; medians use the middle sample.

| Corpus | Median before | Median after | p95 before | p95 after |
|---|---:|---:|---:|---:|
| q4 | 81.33 ms | 79.20 ms | 113.00 ms | 116.81 ms |
| scale | 316.49 ms | 288.87 ms | 821.73 ms | 770.14 ms |

Objective rating, q4: median is 2.6% lower but p95 is 3.4% higher. Treat small-
corpus latency as roughly flat; do not claim a general q4 speedup.

Objective rating, scale-q4: median is 8.7% lower and p95 is 6.3% lower.
All paired responses and exit statuses match on both corpora, normalizing only
the clock-bearing cursor. Full scale retrieval scoring is not claimed from this
representative CLI sample. Expensive fallback invocations still exceed these
percentiles; these are not latency ceilings.

Correctness on the final implementation:

- q4 dev: `any 411/439 first 351/439 xs 30/42 24/42`,
  `empty 35/51 section 55/73`; all 1,470 complete responses match the prior
  captured baseline, including later calls, hints, excerpts and card order.
- h2 all: `any 341/354 first 295/354 xs 57/62 50/62`,
  `empty 56/106 section 239/271`; all 1,380 complete responses match.
- Workspace tests: 50 passed, one intentionally ignored. The new test verifies
  effective PRAGMAs, unloaded block metadata after a full numerical scan, exact
  vector bits, indexed selected-vector reads, cached reuse and diagnostic use.
- Strict workspace Clippy, formatting, LLP references and Nix packaging pass.
- SQL plans for the expanded read-path audit are retained in
  `final-sql-audit.jsonl`. No database schema or publication identity changed.

Objective rating: zero observed retrieval or output differences on either full
corpus replay; no algorithm, threshold, card-order or scoring change.

Reproduction entry points (run tools through the flake):

- `cargo run --release -p linn-eval --example sql_read_profile -- STORE --mmap-mib 256`
- `cargo run --release -p linn-eval --example cli_latency_compare -- --corpus CORPUS --pages PAGES --before BEFORE --after AFTER --store STORE --per-group 5 --rounds 3`
- `linn-eval --corpus evals/corpus/q4 --mode dev --store STORE --pages PAGES`
- `linn-eval --corpus evals/corpus/h2 --mode all --store STORE --pages PAGES`

Full-response captures use the existing `LINN_POS_TRACE=1` diagnostic, extracting
only JSON objects whose stage is `calls`. The exported page ids alone are not the
full-response check. Per-wording resident timing uses the existing `--timings`
option. None of these diagnostics changes the search algorithm.

#### Startup and resident query breakdown

The traced fresh-CLI round gives these medians. Model and matrix phases are
conditional on being reached, so these phase medians must not be summed to
reconstruct the median end-to-end time.

| Phase | q4 before / after ms | scale-q4 before / after ms |
|---|---:|---:|
| libSQL connect, schema, scratch | 0.820 / 0.819 | 0.849 / 0.840 |
| Read/decode record headers | 1.921 / 1.772 | 20.893 / 20.780 |
| Structural totals/setup | 0.515 / 0.499 | 5.596 / 5.481 |
| Complete Store::open | 3.370 / 3.218 | 27.458 / 27.285 |
| Git scope/context discovery | 17.006 / 16.813 | 17.704 / 17.589 |
| Static-model load, when used | 39.009 / 38.742 | 41.119 / 39.119 |
| Global block matrix, when used | 13.756 / 5.514 | 443.778 / 216.279 |

Objective rating: the matrix-read stage is materially cheaper; startup is
essentially unchanged. The database connection itself is not the scaling
bottleneck. Header decoding, model/tokenizer initialization, broad lexical
postings and global numerical work remain costs worth distinguishing.

The existing evaluator's --timings measured the same 29 selected cases with all
three prepared wordings, sequentially in one Store (87 agent invocations). The
first invocation includes any lazy initialization. These are absolute after-only
measurements, not a before/after gain claim, and this population differs from
the CLI's repeated original queries. Corpus reading, grading and output are
excluded from the agent timers.

| Corpus | Store open | Resident median | Resident p95 | Maximum |
|---|---:|---:|---:|---:|
| q4 | 3.170 ms | 7.050 ms | 24.574 ms | 91.270 ms |
| scale-q4 | 26.640 ms | 104.622 ms | 404.920 ms | 1,118.499 ms |

Objective rating: resident query cost still grows substantially at scale;
this pass is an improvement, not a solution to all scaling costs. Resident
percentiles use the evaluator's existing rounding convention.

The final latency captures are q4-final-cli.json and scale-final-cli.json;
final-cli-summary.json and phase-summary.json summarize them. Full replay
comparisons are q4-final-parity.json and h2-final-parity.json. The resident
samples are q4-resident.jsonl and scale-resident.jsonl. All are ignored local
artifacts; corpus, model files and eval output remain uncommitted.


<a id="consolidated-evals-lazy-catalog-2026-11-09-md"></a>

### evals/LAZY-CATALOG-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Local uncommitted report. Artifacts: `evals/output/lazy-catalog-round/`.

#### Removed from healthy startup and ordinary lexical reads

- Full record-header deserialization and full record-id/ordinal maps.
- Full validity/closed/superseded sets and per-clock eligible-record sets.
- Scanning all per-record block/length totals to compute the lexical denominator.
- Complete vocabulary materialization for empty-page membership checks.
- Scanning every cached block for third-call neighbors and quoted basenames.

Healthy opening reads scalar publication metadata. Candidate headers, lifecycle
references, body/hash references and blocks use indexed keys. Sparse resident
maps contain requested records only. Clock statistics use an ordered seek into
integer expiry suffix totals; term DF joins the requested native FTS posting to
persisted lifecycle eligibility. Arithmetic, expiry boundaries, whitespace-body
eligibility, ordinal ties and hidden-after-forget behavior are preserved.

Derived tables store record ordinals/lifecycle/expiry, integer expiry
suffix totals, candidate-addressable rare-form section means and beam term ordinals. They live in the same libSQL file, with a generation-bound atomic
publication and invalidation/secure-delete triggers. They are ingestion indexes,
not another serialized in-memory database. Existing core bodies remain authority.
Parent and close-target expression indexes and a partial nonempty-record index
support the remaining record-selection operations.

The vector comparison diagnostic now orders the loaded vector keys directly;
it no longer assumes all block headers have been loaded as a side effect.

#### Remaining full-population work

This change does **not** establish sublinear worst-case search or zero scans:

- Exact block-vector hints score the global matrix. SQL supplies compact ordinals
  and eligibility flags without loading all record headers, but the numerical
  comparison and best-per-record accumulation remain global.
- Beam, rare-form and sentence fallback stages read their specified global
  persisted numerical populations. Their exact kernels, batches and selection
  rules are retained. Section means now use candidate-only indexed reads.
- Writes, rebuild, forget, initial legacy-index migration, and explicit integrity
  or conformance diagnostics can materialize the corpus. This pass targets healthy
  opening and reads; it does not make ingestion incremental.
- A common lexical term can legitimately match most documents. Candidate and
  posting work scales with that match population, not with an unconditional
  preload of unrelated documents. Long-lived instances retain query-local caches
  across calls and can eventually accumulate many candidates.

Moving an exact vector walk into SQL alone would not make it sublinear and could
change float32 arithmetic. No approximate index, candidate cap, threshold change,
new model or retrieval retuning was introduced.

#### Validation

- New corpus-free test asserts zero resident headers/lookups/order maps after
  healthy opening and after unattested queries across six clocks.
- It compares corpus counts, average-length bits and DF against independently
  materialized fixture records, including whitespace bodies, expiry, candidate,
  closed and close-envelope records. Native EXPLAIN verifies an indexed expiry seek.
- Targeted vocabulary tests cover both present and absent terms without loading
  record headers. Existing lifecycle, secure-forget, stale-reader/publication,
  structural and numerical parity tests remain applicable.
- q4 DEV: 411/439 any, 351/439 first; xs 30/42 and 24/42; empty 35/51;
  section 55/73. All 1,470 complete call responses equal the previous binary's
  captured responses, including ordering, hints and related pointers.

Further final checks and measurements are appended below after completion.

- h2 all: 341/354 any, 295/354 first; xs 57/62 and 50/62; empty 56/106;
  section 239/271. All 1,380 complete call responses match, not just counts.
- Workspace tests: 51 passed, one ignored. Strict workspace/all-targets clippy,
  formatting and LLP reference checks passed.

The final pass also removes the whole-population section-mean map. Means are
computed in their original occurrence order during publication and stored by block
id with the original first-encounter ordinal. Query reads request only rare-word
candidate blocks, preserve that ordinal order and deserialize exact float32 bytes.
A fixture checks persisted mean order and every float32 bit. Missing mean indexes
are upgraded once; healthy startup only verifies their publication identity.

The beam proposal-to-position map also moves to `beam_terms`: query words perform
primary-key lookups. The global beam numerical matrix and proposal order stay
unchanged, but no full vocabulary HashMap is constructed at query time.

#### Audit classification

| Path | Healthy read behavior |
|---|---|
| Store open | Schema checks, scalar publication markers/counts; zero document headers |
| Lexical eligibility/statistics | Indexed expiry seek and term-constrained DF joins |
| Headers, bodies, sections, blocks | Candidate-key batches; original bytes read only when needed |
| Heading trees and surface terms | Queried postings and candidate tree memberships |
| Parent, close, source path, links, basename | Indexed exact references or named records |
| Vocabulary membership | Requested terms only, including cached misses |
| POS | Fixed embedded linguistic resource; indexed pair preflight and candidate relations |
| Rare-form section means | Candidate block keys and preserved encounter ordinal |
| Beam term positions | Requested query words only |
| Global vector/beam/form/sentence scoring | Full numerical population required by current exact algorithm |
| Writes, migrations, integrity diagnostics | Explicit maintenance; may scan and materialize the corpus |

Neither SQL lookup caching nor the fixed model/NLP assets are an approximate
retrieval index. The remaining global numerical stages are the boundary to a
claim of sublinear worst-case end-to-end search.

#### Latency investigation

The initial lazy-read implementation improved scale store opening from 27.30 ms
median to 1.02 ms, but did not improve end-to-end latency: scale median was
294.31 → 295.65 ms and p95 was 779.81 → 846.27 ms. This is **not** an overall
latency win. The full paired exports are `q4-cli.json` and `scale-cli.json`.

A subsequent pass removed redundant block/header joins from the global vector
read by storing the two existing eligibility bits together and joining the vector
row's original record key directly to its read-index entry. A second full paired
trial still showed a slow lexical tail: scale median 285.36 → 289.38 ms and p95
762.98 → 847.42 ms (`scale-cli-final.json`). The slowest traces did not enter a
numerical-array stage, so vector-read joins were not the explanation for that tail.

The final SQL rewrite deduplicates FTS block ids before joining lifecycle
eligibility, avoiding repeated joins for multiple occurrences in one block.
Distinct block count is algebraically identical because both joined keys are
unique. EXPLAIN shows a term-constrained vocabulary walk followed by primary-key
lookups, not an unconditional scan of records. A one-round screen against the
preceding candidate retained every CLI response and measured median 309.90 →
304.35 ms, p95 896.03 → 850.60 ms. This screen is not the final baseline comparison.

Derived SQL tables remain. This pass removes full-store in-memory catalogues and
query-time lookup-map construction; it does not remove the ingestion indexes or
claim that every retrieval stage is sublinear.

#### Final paired measurements

29 queries across six categories, three timed fresh processes per query and binary
(87 samples per side), alternating invocation order on the same store. Filesystem
cache is warm; no build or eval ran concurrently. Startup medians come from the
separate traced invocations, excluded from end-to-end timing samples. One-time
index upgrades are excluded. All sampled CLI responses and exit statuses match;
only the clock-bearing cursor is normalized.

| Corpus | Store open before / after | CLI median before / after | CLI p95 before / after |
|---|---:|---:|---:|
| q4 | 3.24 / 1.01 ms | 80.20 / 78.19 ms | 110.34 / 108.50 ms |
| scale | 27.41 / 1.02 ms | 284.53 / 282.96 ms | 749.19 / 827.49 ms |

Objective rating: PASS for removing unconditional startup materialization and for
response parity; NOT an established end-to-end scale latency improvement.
The measured tail remains slower, despite the startup reduction.

Final source checks: fmt, strict workspace/all-targets clippy, 51 tests passed
(one ignored), Nix packaged build, and LLP references passed. Final q4/h2 exports
(`q4-df` and `h2-df`) match all 1,470 and 1,380 full responses respectively.
No retrieval thresholds or card order changed. No push or commit.


<a id="consolidated-evals-candidate-reads-2026-11-09-md"></a>

### evals/CANDIDATE-READS-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Local, uncommitted performance report. Baseline: `3f37c7c`.
Artifacts: `evals/output/candidate-read-round/`.

#### Changes

- Read block body hashes from the existing compact `record_headers` table instead
  of the main record row containing the original document. The original hash and
  publication checks are unchanged; selected text still comes from original bytes.
- Reuse the record ids already returned with block metadata. Buffer each existing
  bounded batch, load missing parent headers, then publish its blocks to resident
  maps. Remove the preliminary query that rediscovered those same record ids.
- Let libSQL return integer term-frequency counts directly. Preserve ordered body
  and stem positions, but stop transferring and parsing heading/parts position
  arrays that Rust only used to obtain their lengths.
- Prepare compact headers before numerical projections when upgrading a legacy
  store. A corpus-free test removes both and verifies recovery, original hashes,
  original body bytes and numerical publication.

No schema, table, index, model, tokenizer, ranking constant or query rule is added.
No source text is altered. This is a change to data access and aggregation only.

#### Initial measurements

The native libSQL diagnostic reads all nine metadata columns for 1,785 blocks
matching `memory` in scale-q4. Its last two rounds measured 5.98/6.40 ms using
main-record hashes and 3.66/3.86 ms using compact-header hashes. EXPLAIN shows
primary-key lookups for both, with no unconditional record scan.

The compact-header change alone did not establish a CLI gain: a one-round paired
screen over 29 queries measured 286.09 → 287.69 ms median and 824.89 → 829.13 ms p95.

With native frequency counts added, the same screen measured 297.21 → 266.55 ms
median and 872.64 → 777.67 ms p95. All sampled CLI responses and exit statuses
matched. These preliminary samples are not the final packaged comparison.

Global numerical fallback walks and match-population-dependent lexical work
remain. This pass does not claim sublinear worst-case retrieval.

#### Final packaged comparison

29 queries across six categories, three timed fresh CLI invocations per query and
binary (87 samples per side). Invocation order alternates. Filesystem cache is
warm; no build or evaluation ran concurrently. Separate traced invocations are
excluded from end-to-end timing samples. Each complete CLI response and exit status
matched, with only the clock-bearing cursor normalized.

| Corpus | CLI median before / after | CLI p95 before / after | Store open median before / after |
|---|---:|---:|---:|
| q4 | 77.27 / 77.75 ms | 115.64 / 107.92 ms | 1.00 / 1.01 ms |
| scale | 303.36 / 271.76 ms | 828.55 / 770.90 ms | 1.02 / 1.02 ms |

#### Correctness and checks

- q4 dev: any 411/439, first 351/439; xs 30/42 and 24/42; empty 35/51;
  section 55/73. All 1,470 complete call responses match the committed baseline.
- h2 all: any 341/354, first 295/354; xs 57/62 and 50/62; empty 56/106;
  section 239/271. All 1,380 complete call responses match.
- Formatting, strict workspace/all-targets clippy, workspace tests (52 passed,
  one ignored), Nix build and LLP references passed.
- Existing token-feature conformance tests compare every frequency and body/stem
  position with a full native instance walk and check repeated loads do not double TF.

Objective rating: PASS for exact response parity and required checks. Latency
values above are a paired workload measurement, not a worst-case guarantee.
Changes and this report are uncommitted; nothing was pushed.

Latency rating: IMPROVED on this scale workload (median 10.4% lower, p95 7.0%
lower). The q4 median is essentially flat (+0.48 ms); its p95 is 7.72 ms lower.
This does not establish constant-time queries or eliminate global vector scans.


<a id="consolidated-evals-read-refactor-2026-11-09-md"></a>

### evals/READ-REFACTOR-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Local, uncommitted report. The baseline is the saved binary from the preceding
compact-header/native-frequency pass, including that pass's uncommitted changes.
Artifacts and the implementation checklist are in `evals/output/read-refactor/`.

#### Changes

1. Separate global corpus statistics from per-candidate term features. Read exact
   frequencies and positions for positive-coverage rows and candidate tree winners;
   confidence reads stem positions for its first five distinct-record ranks.
2. Share candidate completion across a normalized term set. Reuse one connection-local
   candidate-id table across its term queries. This is query scratch, not another
   persistent ingestion index or external data file.
3. Use the existing POS pair index for batched candidate membership. Preserve body,
   resource and generation guards, duplicate-pair weights and addition order. Traces
   validate the resulting score bit-for-bit against original stored witnesses.
4. Batch vocabulary existence probes and initialize occurrence-tokenizer scratch once.
   Batch original token surfaces and whole-sentence reconciliation as separate inputs.
5. Batch exact per-word Porter operations, keeping one input per row and the original
   tokenizer and first-term semantics. Index stemming is still a separate tokenizer.
6. Choose record-fill winners from block metadata before reading text. Repeated
   record-to-block loads reuse the complete candidate-local id list.
7. Hints and related pointers reject candidates and sort using metadata before
   reading selected record bodies. Related pointers exclude already-shown records
   before loading sections; hints reject ineligible records before section reads.
8. Unique basename promotion seeks the earliest block; shared names read eligible
   records only, then run the unchanged original-body stem test.
9. Sentence fallback applies numerical thresholds and same-block constraints before
   fetching headers and reads only the winning block's text.
10. Block presentation reads only id and original byte slice, reusing already-read
    heading, path and hash under the search snapshot.
11. Batch navigation words and tree-member requests; SQL returns ordered block and
    distinct record postings. Record ordinal lookups operate on distinct records.
12. Return each queried surface word's frequency once. Do not hydrate all its matching
    blocks: attach surface membership when a requested block header is loaded.
13. Batch outgoing links, backlinks and same-source-path neighbors with their original
    direction-specific exclusions.
14. Reject ineligible candidates before fetching individual vectors and section means.
    Keep the same surviving order and numerical operations.
15. Batch original-byte reads for the selected later-call cards.
16. Resolve show references from metadata. For normal bounded spans, transfer at most
    6,000 bytes for the first 1,500 Unicode characters and count the full span in SQL.
    Preserve Rust lossy decoding for spans cutting Unicode scalars and for embedded
    NULs, where SQLite text counting differs. Rejected records need no body read.

Startup guards and full numerical fallback populations remain intact. No model,
retrieval constant, author-field processing, scoring rule, or persistent schema
was added or changed. No claim of sublinear worst-case exact vector search is made.

#### Development checks and rejected implementations

The first candidate preserved all 1,470 full dev responses, but its scale CLI
screen regressed. It was not retained as the final implementation. Investigation
used the pinned libSQL's EXPLAIN QUERY PLAN and temporary per-function timing.
The plans showed indexed candidate lookups. Repeated per-term candidate bookkeeping
and broad aggregate work were significant. The final implementation shares candidate
completion and scratch ids across a term set, batches POS membership, and resolves
navigation record ordinals after distinct-record selection. Temporary timing code
and the standalone plan-audit example were removed from product/example sources;
the diagnostic source and measurements remain under ignored output.

Corpus-free tests cover partial feature loads followed by further subsets and a
full read, batch/scalar Porter equivalence, repeated POS weights with order-sensitive
floating-point sums, guarded POS publication, and bounded display over Unicode,
embedded NULs and arbitrary byte spans. Existing lifecycle, forget, CLI and scorer
fixtures also remain required.

#### Final validation

- q4 dev: any 411/439, first 351/439; cross-scope 30/42 and 24/42;
  empty 35/51; section 55/73. All 1,470 complete responses match.
- h2 all: any 341/354, first 295/354; cross-scope 57/62 and 50/62;
  empty 56/106; section 239/271. All 1,380 complete responses match.
- Formatting, strict workspace/all-targets clippy, workspace tests (54 passed,
  one ignored), Nix build and LLP references pass. No temporary instrumentation
  remains in product source. No push or commit was performed.

Correctness rating: PASS — all 2,850 full responses are identical, including cards,
hints and later calls; every score remains unchanged.

Checks rating: PASS — all required development and packaging checks passed.

#### Final packaged CLI measurement

29 representative queries across six categories, three timed fresh invocations
per query and binary: 87 samples per side. Invocation order alternates; filesystem
cache is warm. No builds or evals ran concurrently. Separate traced invocations
are excluded from end-to-end timing. All sampled complete CLI responses and exit
statuses match, with only the clock-bearing cursor normalized. This is a sampled
scale workload, not a rerun of every scale-corpus case or a worst-case guarantee.

| Corpus | Median before / after | p95 before / after | Store open median before / after |
|---|---:|---:|---:|

| q4 | 77.01 / 76.90 ms | 110.83 / 107.87 ms | 1.02 / 1.13 ms |
| scale-q4 | 283.93 / 277.02 ms | 769.06 / 750.84 ms | 1.03 / 1.15 ms |

Latency rating: MODEST IMPROVEMENT on the scale sample (median 2.4% lower,
p95 2.4% lower). q4 median is flat; its p95 is 2.7% lower. Opening increases by
about 0.1 ms because scratch structures are initialized once and reused. The main
benefit of this pass is reducing unnecessary hydration and repeated work while
preserving the exact page; it is not a substantial overall latency breakthrough.


<a id="consolidated-evals-cli-breakdown-2026-11-09-md"></a>

### evals/CLI-BREAKDOWN-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Commit measured: `0a3078b`. Local, uncommitted report. Artifacts and the removable instrumentation patch are under `evals/output/cli-breakdown/`.

#### Method

- Same 29-query stratified workload as the previous latency pass, covering identifiers, phrases, passages, lateral, paraphrase and no-gold queries. q4 has 1,376 records; scale-q4 has 16,906.
- Three timed fresh processes per query and binary: 87 samples per corpus. The committed packaged binary and a temporary instrumented binary alternate order. Filesystem cache is warm; no concurrent builds or evals.
- Nested wall-clock timers record inclusive and exclusive duration. Exclusive components add to the instrumented invocation time without counting nested work twice. The outer subprocess timer measures invocation through exit.
- Storage timers include libSQL execution, row transfer, Rust decoding, allocation and materialization. They do not isolate SQLite engine CPU. Outside-CLI time includes process loading, Tokio setup, profiler-summary emission, exit and parent collection.
- All 232 sampled output/exit-status pairs (including separate diagnostic rounds) match after normalizing only clock-bearing cursors.
- Timing instrumentation changes code layout and adds overhead. Its mean was 1.0% below the control on q4 and 2.6% below on scale; treat attribution as an approximation, not as a speedup. Use the uninstrumented figures for end-to-end latency.

#### Committed CLI latency

| Corpus | Median | p95 | Mean |
|---|---:|---:|---:|
| q4 | 77.43 ms | 114.65 ms | 68.41 ms |
| scale | 276.10 ms | 764.98 ms | 371.41 ms |

#### Additive breakdown

These are exclusive **means**, not components of the median. They sum to the instrumented mean; query paths vary substantially.

| Component | q4 | Scale |
|---|---:|---:|
| Process/runtime outside CLI timer | 4.99 ms | 6.20 ms |
| CLI parsing, output and cleanup | 1.46 ms | 10.89 ms |
| Git scope discovery | 17.25 ms | 17.78 ms |
| Store open and validation | 1.09 ms | 1.12 ms |
| Indexed reads and materialization | 12.13 ms | 218.21 ms |
| POS and exact tokenization | 22.21 ms | 30.15 ms |
| Static embedding model load | 2.73 ms | 2.73 ms |
| Numerical arrays: read and decode | 1.07 ms | 26.64 ms |
| Embedding encode and vector arithmetic | 0.14 ms | 1.26 ms |
| Matching, ranking and fallback logic | 4.67 ms | 46.65 ms |
| Total | 67.74 ms | 361.64 ms |

#### Where the time goes

- Opening the local libSQL database and connection takes about 0.12–0.13 ms. Schema, scratch and pragma setup takes about 0.88–0.89 ms. Total healthy store opening and validation is about 1.1 ms; it is not the scale bottleneck.
- Git scope discovery takes about 17–18 ms per invocation in this checkout. The current implementation launches three Git commands: repository root, remote list, then remote URL. This cost depends on caller environment.
- POS resource deserialization takes about 36–37 ms when tagging is needed. The indexed preflight required tagging for 15 of 29 q4 queries and 19 of 29 scale queries. POS remains enabled; this is conditional work within the fixed algorithm.
- The static embedding model loads in roughly 40 ms when needed. Only two sampled queries per corpus loaded it, so the average model-load cost is small. This sample does not establish that embeddings are cheap on every workload.
- At scale, indexed reads and materialization average 218.21 ms, roughly 60% of instrumented end-to-end time. The exclusive contributors below exclude nested header reads, so they can be compared without double counting.

| Scale lookup component | Mean exclusive time |
|---|---:|
| `storage.mod.load_navigation` | 67.55 ms |
| `storage.mod.load_block_headers` | 54.60 ms |
| `storage.mod.read_term_features` | 41.75 ms |
| `storage.catalog.load_record_headers` | 17.31 ms |
| `storage.mod.load_section_words` | 14.68 ms |
| `storage.mod.scope_stats` | 11.65 ms |

- Matching/ranking/fallback logic contributes another 46.65 ms on average at scale, mostly lexical matcher work. CLI search post-processing/output itself is under 1 ms on average; the broader CLI category includes cleanup of the resident Store.
- Long fallback paths differ from these averages. The slowest observed instrumented scale sample, `s1-c-0001`, took 1,242.76 ms: numerical-array read/decode 538.80 ms, indexed reads/materialization 369.07 ms, matching/fallback work 197.99 ms, static model load 38.96 ms, and vector encoding/arithmetic 32.23 ms. These arrays are loaded during the query, not healthy store opening.

#### Outcome

Objective rating: PASS for diagnostic output parity and restoration. The measurement identifies query-time metadata/postings work as the main scale cost, with Git discovery and conditional POS initialization as substantial fixed costs. It does not establish pure SQLite CPU time or cold-cache latency.

All temporary source instrumentation was removed byte-for-byte. The normal release CLI was rebuilt; tracked files are unchanged. No retrieval changes, commits or pushes.


<a id="consolidated-evals-candidate-selection-2026-11-09-md"></a>

### evals/CANDIDATE-SELECTION-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Local, uncommitted report. Baseline: `0a3078b`. Artifacts are under
`evals/output/candidate-selection/`. No retrieval constants, thresholds,
card ordering, encodings, tokenization or ingest behavior changed.

#### Changes

- Ordinary lexical queries apply lifecycle and clock eligibility inside the FTS
  query. Reuse qualified hit sets by expression, tokenizer and clock. Invalidation
  follows the existing hit cache. Explicit query filters retain their existing path.
- Preserve complete hit sets and posting arrival order, but admit raw, one-under
  and tree-winner candidates before loading frequencies or computing POS bonuses.
  Positive-coverage tree winners retain their existing POS score even when they
  fail raw admission; zero-coverage winners retain their lexical-only score.
- SQL selects qualifying trees and their winning original block using the existing
  normalized-word index. Only positive-overlap members can win a qualifying tree;
  zero-overlap members are not materialized. Missing hard navigation words allow
  an indexed early miss. Trees are query-local, not a substitute for full structure.
- Record fills select the winning record and block in SQL, preserving coverage,
  descending record ties, ascending block ties and zero-overlap last-record behavior.
- Scoring reads block identity, length, hash and spans before heading/path metadata.
  Presentation upgrades preserve features already loaded. Record headers remain
  candidate-local; this change does not add a parallel persistent index or table.
- Frequency reads no longer build position arrays. Body-presence POS checks use
  exact body counts. Confidence loads body/stem offsets only for its first five
  distinct-record ranks. Frequency and position completion are tracked separately.
- Linked calls hydrate only hits belonging to linked records, and skip term reads
  when there are no neighbors. CLI pointer filtering explicitly loads each pointer's
  containing block instead of relying on incidental prior hydration.

#### Fresh CLI latency

Milliseconds, invocation through exit. Same 29 stratified queries, covering six
query categories. Three alternating rounds: 87 timed samples per binary/corpus,
plus a separate traced round. Warm filesystem cache; no concurrent build or eval.
Only the cursor's clock is normalized for response comparison. All 232 output and
exit-status pairs match. Corpora: q4 1,376 records; scale-q4 16,906 records.

| Corpus | Binary | Median ms | p95 ms | Mean ms |
|---|---|---:|---:|---:|
| q4 | before | 77.04 | 108.09 | 67.83 |
| q4 | after | 74.98 | 105.60 | 64.58 |
| scale | before | 267.36 | 747.22 | 363.00 |
| scale | after | 221.26 | 526.61 | 281.30 |

Objective rating: PASS for sampled CLI parity. Scale median improves 17.2%, p95
29.5%, and mean 22.5%. q4 changes are small. These are warm-cache measurements,
not a cold-disk guarantee or a claim that every query improves.

#### Retrieval and work verification

- Dev: any **411/439**, first **351/439**, cross-scope **30/42**, **24/42**,
  empty **35/51**, section **55/73**. All **1,470 complete responses** match.
- H2 all: any **341/354**, first **295/354**, cross-scope **57/62**, **50/62**,
  empty **56/106**, section **239/271**. All **1,380 complete responses** match.
- Dev primary-score trace events fall from **1,264,234** to **879,564** (30.4%).
  These are actual score computations across original/alternate and internal
  fallback queries, not unique records. Feature selection retains 1,549,286 rows
  across matcher invocations versus 2,251,715 positive-coverage rows; retained
  features also include tree winners. Counts are not a latency benchmark.
- New corpus-free test compares SQL tree and record winners with full in-memory
  selection, including repeated queries, hard words, ties, exclusions and empty
  query coverage. It checks lazy eligibility, deferred positions, feature-preserving
  metadata upgrades and pointer block reads.

Objective rating: PASS. Counts and complete response content/order are unchanged.

#### Query plans and remaining limits

Native libSQL plans confirm indexed normalized-word, block-membership and
kind/tree lookups. No new persistent tables or indexes were introduced.

The pinned FTS vocabulary implementation only pushes term constraints into its
virtual-table seek. Candidate document predicates reduce grouping, offset
construction and row transfer, but still walk that term's postings. A diagnostic
`configuration` query returns 8,493 grouped rows without a candidate restriction
and ten groups for five matching blocks with the restriction. Both plans use the
same term-index virtual-table seek. Do not claim candidate-local posting traversal.

Explicit CTE materialization was also measured for the SQL tree query. The
unhinted query took about 25–26 ms in the probe; materializing words took about
45–48 ms, and materializing matches was flat. No materialization hint was adopted.
This is a query-planner experiment, not a retrieval experiment.

Temporary profiling located a 71 ms mean cost in the first implementation's
hit-id serialization and eligibility round-trip. The final implementation moved
that qualification into FTS and cached it between matchers. Temporary profiling
source was removed; its binary, inputs and output remain ignored.

Broad/common terms and the specified global vector fallbacks still scale with
matching postings or numerical populations. This pass does not make all queries
constant-time and does not narrow a vector population to manufacture a speedup.

#### Checks and reproduction

Passed: formatting, workspace all-target/all-feature strict clippy, workspace
all-feature tests (**55 passed, one ignored**), LLP references and `nix build`.
The packaged CLI reproduces the scale fallback response with `LINN_MODEL` unset.

Dev and h2 full-call traces were exported with `LINN_POS_TRACE=1` from the saved
`after-eval` binary, using `--corpus evals/corpus/q4 --mode dev` and
`--corpus evals/corpus/h2 --mode all`, respectively. Existing stores were reused;
ingestion and source bytes were untouched. Full response diffs are recorded in
`q4-parity.json` and `h2-parity.json`.

Latency helper: `target/release/examples/cli_latency_compare --corpus evals/corpus/q4
--pages evals/output/scale-index-round/q4-final.pages.jsonl --before
 evals/output/candidate-selection/before-cli --after
 evals/output/candidate-selection/after-cli --store <store> --per-group 5 --rounds 3`.
Use the q4 or scale store; run through `nix develop -c`.

No commit or push. Corpus, model files, diagnostics and this report remain uncommitted.


<a id="consolidated-evals-porter-audit-2026-11-09-md"></a>

### evals/PORTER-AUDIT-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Local investigation of `ae8f911`; product, LLPs and dependencies unchanged.
Artifacts and the disposable Rust prototype: `evals/output/porter-audit/`.

#### Findings

Linn has three distinct uses of Porter:

1. The persistent stem FTS5 index performs lexical matching. Keep its tokenizer
   and indexed contents unchanged.
2. `Store::porter` and `prepare_porter` lowercase each independent input, tokenize
   with `porter unicode61`, and return the lexicographically first stem, falling
   back to the lowercased input when there are no tokens. They cache completed inputs.
3. `index_stems` uses `porter unicode61 tokenchars '_-'` and returns sorted unique
   stems for confidence. It does not apply the helper's Rust lowercase pass.

These are intentional differences. `foo-bar` becomes `bar` in the per-word helper,
while it remains one token in the index helper. Replacing the first-vocabulary-row
rule with the first token, using modern Unicode segmentation, or changing stemmer
variants would change behavior. The pinned Porter implementation passes tokens
shorter than three bytes or longer than 64 bytes through without its stemming steps.

#### Direct tokenizer experiment

Used libSQL's existing exported FFI to retrieve its registered Porter tokenizer
through `fts5_api.xFindTokenizer`. Created the two configurations and called
`xTokenize` directly, using document-tokenization flags, with explicit sorting,
deduplication, lowercase and empty-input handling as appropriate.

The prototype owns an ephemeral in-memory tokenizer connection; it is not another
persistent store. It uses the same statically linked library already in the binary.
No dependency, model, runtime or download was added. The persistent index is untouched.

Compared 21,259 inputs: all distinct q4 body vocabulary, the case queries
and up to two alternates, plus punctuation, Unicode, NUL and token-length boundary
fixtures. Each input was checked against both existing SQL helper configurations.
All three runs have zero word-helper and zero index-helper differences. Corpus
`source_path`, author terms and section terms were not read for tokenization.

Median total time across three runs, milliseconds:

| Operation | Total ms |
|---|---:|
| SQL per input, uncached | 524.341 |
| SQL single bulk batch | 31.681 |
| Direct tokenizer, per-word helper | 3.193 |
| Setup of both direct tokenizer instances | 0.021 |

SQL measurements use TEMP FTS tables and memory temp storage, matching production's
scratch-table placement. An initial disk-table probe was not representative and is
explicitly excluded. Direct timing includes lowercase, token collection, sorting
and selection, but excludes setup; setup is reported separately. Both SQL timings
exclude schema creation. Corpus parsing is excluded from all timings.

This is a bulk helper microbenchmark, not a CLI benchmark. Current production
caching and per-checklist batching mean the uncached column does not describe an
ordinary query. A temporary instrumented binary timed scalar `porter`, batched
`prepare_porter`, and `index_stems` on 29 stratified fresh scale CLI invocations.
Combined helper time averaged **0.344 ms**, median **0.318 ms**,
with a maximum of **1.200 ms** (s1-c-0001).
All 58 paired CLI responses and exit statuses matched, including a second diagnostic
round. This covers helper calls, not Porter work internal to FTS5 indexing/MATCH.
Timers add overhead, especially for cached scalar calls; this is an attribution
measurement, not an integrated replacement benchmark.

Temporary source instrumentation was restored byte-for-byte. The normal release
CLI was rebuilt. No tracked files changed.

#### Recommendation

A focused adapter in `src/nlp/porter.rs` is feasible. Prefer the existing native
SQLite tokenizer over maintaining a second Porter/Unicode61 implementation in Rust.
Keep it independent of Store and ranking; expose separate per-word and index-token
operations. Production integration needs checked FFI errors and input lengths,
callback error propagation without unwinding, RAII destruction order and explicit
thread ownership. The assertion-based diagnostic prototype is not production code.

Replace scratch-table helper operations only; preserve the lexical stem index,
caches and token configurations. Verify full dev/h2 responses and fresh CLI latency
before adopting. No end-to-end speedup is established by this experiment. This is a
small query-path simplification opportunity, not evidence that Porter is the main
scale bottleneck or that retrieval needs a different stemmer.

Objective rating: PASS for helper equivalence on the tested inputs; promising
microbenchmark; NOT YET validated as an integrated CLI optimization.

Reference: [SQLite FTS5 tokenizer API](https://www.sqlite.org/fts5.html#extending_fts5)
and [Porter tokenizer semantics](https://www.sqlite.org/fts5.html#porter_tokenizer).


<a id="consolidated-evals-porter-direct-2026-11-09-md"></a>

### evals/PORTER-DIRECT-2026-11-09.md

Historical performance stage; later stages supersede its remaining-work list. These are workload measurements, not new latency gates or claims of constant-time search.

Baseline: `ae8f911`. Local uncommitted report; diagnostic artifacts are under
`evals/output/porter-direct/`.

#### Implementation

- `src/nlp/porter.rs` calls the pinned libSQL FTS5 tokenizer API directly. It
  keeps separate `porter unicode61` and `porter unicode61 tokenchars '_-'`
  instances. No new dependency or alternate stemming algorithm was added.
- Per-word results retain Rust lowercasing, lexicographic first-stem selection,
  and lowercased-input fallback. Index helper results retain original-input
  tokenization and sorted unique stems. Tokenization uses document flags.
- Native handles are initialized lazily on their owning thread. A private
  ephemeral memory connection supplies the API; it contains no documents or
  persistent state. Handles do not cross threads or await points. Exclusive
  borrowing rejects reentrancy. Destruction releases tokenizer instances before
  the connection; local statements are finalized on success and error paths.
- The adapter checks native status codes, nullable API functions and byte-length
  conversion. It copies token bytes, validates UTF-8 and contains callback
  unwinding. Errors propagate through Linn's typed error channel.
- Store helper caches and prewarming remain. The store reuses its already
  lowercased cache key. Four temporary FTS/vocabulary tables and their
  delete/insert/select operations were removed. Persistent stem index schema,
  contents, ingestion, ranking, thresholds and card ordering remain unchanged.
- Shared annotation declarations moved to `src/nlp/annotation.rs`, with explicit
  module paths. The asset compiler reuses those declarations without acquiring a
  build-time libSQL dependency. POS algorithms and data did not change.
- LLP 0001, LLP 0002 and the NLP README describe the implementation boundary.

#### Verification

Objective rating: **PASS for semantic equivalence.**

- Production adapter: **21,259** q4 vocabulary, wording and edge-case probes;
  zero differences against both SQL helper configurations.
- Dev: **411/439 any, 351/439 first, xs 30/42 and 24/42, empty 35/51,
  section 55/73**. All **1,470 full responses** match the baseline.
- H2 all: **341/354 any, 295/354 first, xs 57/62 and 50/62, empty 56/106,
  section 239/271**. All **1,380 full responses** match the baseline.
- New corpus-free tests cover SQL equivalence, punctuation, duplicate tokens,
  Unicode, embedded NUL, token-length boundaries, callback errors, reentrancy
  and independent thread lifetime/destruction.
- **58 tests passed, one ignored**; formatting, strict workspace/all-target
  clippy, LLP reference checks and `nix build` passed.
- The packaged CLI matches its predecessor with model and POS build-resource
  environment variables unset. No runtime files or downloads were introduced.

The temporary corpus conformance example was moved into ignored output after
execution. It is not part of the product or the committed test suite.

#### Fresh CLI latency

Same 29-query stratified workload, three alternating timed rounds per binary and
corpus: 87 samples each, plus a diagnostic round. Filesystem cache was warm; no
build or eval ran concurrently. Timing covers invocation through exit. Only
clock-bearing cursors are normalized for output comparison. All **232 paired
CLI output and exit-status checks** match.

| Corpus | Binary | Median ms | p95 ms | Mean ms |
|---|---|---:|---:|---:|
| q4 | before | 75.40 | 103.42 | 64.89 |
| q4 | after | 74.46 | 101.95 | 63.89 |
| scale | before | 233.92 | 544.15 | 295.07 |
| scale | after | 222.75 | 537.65 | 284.51 |

Objective rating: **PASS for sampled CLI parity; no aggregate latency regression
observed.** The improvements are modest. The prior audit measured about 0.344 ms
of Porter helper work per scale invocation, so these end-to-end differences must
not all be attributed to direct stemming; code layout and timing variation can
also contribute. This is not a claim that every query improves.

Reproduction inputs and binaries: `before-cli`, the packaged `result/bin/linn`,
`conformance-final.json`, `q4-parity.json`, `h2-parity.json`, `q4-cli.json`,
`scale-cli.json`, and `packaged-parity.json` under the artifact directory.
The paired benchmark uses `cli_latency_compare --per-group 5 --rounds 3`, the
existing q4 and scale stores, and the previously captured q4 page selection.

No commit or push. Corpus, model files, output and this report remain uncommitted.


<a id="consolidated-evals-baseline-2026-11-09-md"></a>

### evals/BASELINE-2026-11-09.md

Historical nine-suite run at c6d44f1, all nine commands exited zero. This is not the later accepted combination. The tables retain the November 8 prior values and November 9 values, including failures; no threshold changed.

#### Corpus identity

| Corpus | Records | Cases | Records SHA-256 | Cases SHA-256 |
|---|---:|---:|---|---|
| q4 | 1376 | 687 | `6b1dafa52bdaad4d979e1b84bd4500f77b2c00d56b7f79c4279e1ef56928bcd9` | `6d39712be2dd5c4d4c041e27fe989a349174dcd350ceed01a1b6594f4c2223d5` |
| h2 | 1473 | 460 | `217fa80ddd06b913f6161d0551fd3e151aef92ca9916a560e9304acdc7829151` | `ea0a783ccc22f7182880393d88409adf708f8b311e28bcb60a726fd3b8ac3a22` |
| scale-q4 | 16906 | 687 | `be42984d615165066a2d3cfc16d863aa010b0a9ec823cde395b7e1c813231a5f` | `6d39712be2dd5c4d4c041e27fe989a349174dcd350ceed01a1b6594f4c2223d5` |
| scale-h2 | 17003 | 460 | `13945f3462a22faeb459987974be4d1e6ee8b19cbcf8b33b032c765f03c970a6` | `ea0a783ccc22f7182880393d88409adf708f8b311e28bcb60a726fd3b8ac3a22` |
| synthetic-six-docs | 6 | 49 | `063bea2c462f91b78fa326280872a4461a8337c2cc63cfd0f1c1c87a90ef3723` | `bc058d9593532458035a7aee8d8fc9330ba1aefc3826f0e182cee7f42d39741e` |

#### Retrieval comparisons

| Corpus | Gold any | Gold first | XS any | XS first | Empty | Section |
|---|---|---|---|---|---|---|
| q4 | 410 → 411/439 (+1) | 350 → 351/439 (+1) | 30 → 30/42 (+0) | 24 → 24/42 (+0) | 35 → 35/51 (+0) | 55 → 55/73 (+0) |
| h2 | 341 → 341/354 (+0) | 294 → 295/354 (+1) | 57 → 57/62 (+0) | 50 → 50/62 (+0) | 56 → 56/106 (+0) | 237 → 239/271 (+2) |
| scale-q4 | 389 → 387/439 (-2) | 315 → 313/439 (-2) | 27 → 27/42 (+0) | 21 → 21/42 (+0) | 31 → 31/51 (+0) | 51 → 51/73 (+0) |
| scale-h2 | 326 → 326/354 (+0) | 273 → 273/354 (+0) | 49 → 48/62 (-1) | 40 → 40/62 (+0) | 48 → 47/106 (-1) | 225 → 226/271 (+1) |

##### q4 retrieval

Objective rating: NO METRIC REGRESSIONS versus prior full-suite baseline.

Ordered first-call pages changed: 146/1470.

agent invocation timings: n=1470 median_ms=5.966 p95_ms=15.666 max_ms=46.918; store_ms=17926.306; excludes corpus loading, grading and output; not the neutral per-call gate

##### h2 retrieval

Objective rating: NO METRIC REGRESSIONS versus prior full-suite baseline.

Ordered first-call pages changed: 141/1380.

agent invocation timings: n=1380 median_ms=6.794 p95_ms=19.098 max_ms=46.254; store_ms=19729.292; excludes corpus loading, grading and output; not the neutral per-call gate

##### scale-q4 retrieval

Objective rating: REGRESSION versus prior full-suite baseline; recorded without retuning.

Ordered first-call pages changed: 181/1470.

agent invocation timings: n=1470 median_ms=86.238 p95_ms=315.223 max_ms=1072.946; store_ms=501745.820; excludes corpus loading, grading and output; not the neutral per-call gate

##### scale-h2 retrieval

Objective rating: REGRESSION versus prior full-suite baseline; recorded without retuning.

Ordered first-call pages changed: 187/1380.

agent invocation timings: n=1380 median_ms=93.200 p95_ms=350.310 max_ms=872.564; store_ms=529720.331; excludes corpus loading, grading and output; not the neutral per-call gate

#### Synthetic six-document benchmark

Objective rating: UNCHANGED — Hit@1 31/49, Hit@3 and Hit@5 35/49, MRR@5 0.6735.

| group | cases | Hit@1 | Hit@3 | Hit@5 | MRR@5 |
|---|---|---|---|---|---|
| all | 49 | 31/49 | 35/49 | 35/49 | 0.6735 |
| deep-research | 25 | 12/25 | 16/25 | 16/25 | 0.5600 |
| deep-research/hard | 25 | 12/25 | 16/25 | 16/25 | 0.5600 |
| standard | 24 | 19/24 | 19/24 | 19/24 | 0.7917 |
| standard/easy | 6 | 6/6 | 6/6 | 6/6 | 1.0000 |
| standard/fusion | 6 | 4/6 | 4/6 | 4/6 | 0.6667 |
| standard/hard | 6 | 6/6 | 6/6 | 6/6 | 1.0000 |
| standard/medium | 6 | 3/6 | 3/6 | 3/6 | 0.5000 |

#### q4 neutral

Objective rating: 11 pass, 5 FAIL; failing gates: G1 eligibility violations, G2 must-not rate, G4 integrity and forget, Q1 empty-correct, R2 maximum exposure share.

| Gate | Prior value | Current value | Need | n | Current status |
|---|---:|---:|---|---:|---|
| G1 eligibility violations | 24.000 | 24.000 | <= 0 |  | FAIL |
| G2 must-not rate | 0.022 | 0.022 | <= 0.02 |  | FAIL |
| G3 location invariance | 1.000 | 1.000 | >= 1 | 490 | pass |
| G4 integrity and forget | 0.000 | 0.000 | >= 1 |  | FAIL |
| G5 determinism after rebuild | 1.000 | 1.000 | >= 1 |  | pass |
| Q1 empty-correct | 0.686 | 0.686 | >= 0.9 | 51 | FAIL |
| Q2 core found | 0.985 | 0.985 | >= 0.85 | 198 | pass |
| Q3 overall found | 0.980 | 0.980 | >= 0.75 | 348 | pass |
| Q4 section hit | 0.822 | 0.822 | >= 0.75 | 73 | pass |
| Q5 lateral found | 0.974 | 1.000 | >= 0.6 | 39 | pass |
| Q6 paraphrase found | 0.909 | 0.927 | >= 0.35 | 55 | pass |
| C1 tokens median | 176.000 | 177.000 | <= 600 |  | pass |
| C1 tokens p95 | 287.000 | 287.000 | <= 1500 |  | pass |
| C2 latency p95 ms | 2.950 | 7.810 | <= 50 |  | pass |
| R1 self-retrieval@5 | 0.988 | 0.988 | >= 0.9 | 1070 | pass |
| R2 maximum exposure share | 0.226 | 0.227 | <= 0.1 |  | FAIL |

#### h2 neutral

Objective rating: 11 pass, 5 FAIL; failing gates: G1 eligibility violations, G2 must-not rate, G4 integrity and forget, Q1 empty-correct, R2 maximum exposure share.

| Gate | Prior value | Current value | Need | n | Current status |
|---|---:|---:|---|---:|---|
| G1 eligibility violations | 24.000 | 24.000 | <= 0 |  | FAIL |
| G2 must-not rate | 0.052 | 0.052 | <= 0.02 |  | FAIL |
| G3 location invariance | 1.000 | 1.000 | >= 1 | 460 | pass |
| G4 integrity and forget | 0.000 | 0.000 | >= 1 |  | FAIL |
| G5 determinism after rebuild | 1.000 | 1.000 | >= 1 |  | pass |
| Q1 empty-correct | 0.528 | 0.528 | >= 0.9 | 106 | FAIL |
| Q2 core found | 0.993 | 0.993 | >= 0.85 | 307 | pass |
| Q3 overall found | 0.981 | 0.981 | >= 0.75 | 267 | pass |
| Q4 section hit | 0.900 | 0.904 | >= 0.75 | 271 | pass |
| Q5 lateral found | 1.000 | 1.000 | >= 0.6 | 40 | pass |
| Q6 paraphrase found | 0.958 | 0.958 | >= 0.35 | 48 | pass |
| C1 tokens median | 180.000 | 180.000 | <= 600 |  | pass |
| C1 tokens p95 | 299.000 | 299.000 | <= 1500 |  | pass |
| C2 latency p95 ms | 3.230 | 8.330 | <= 50 |  | pass |
| R1 self-retrieval@5 | 0.989 | 0.989 | >= 0.9 | 1138 | pass |
| R2 maximum exposure share | 0.435 | 0.427 | <= 0.1 |  | FAIL |

#### scale-q4 neutral

Objective rating: 11 pass, 5 FAIL; failing gates: G1 eligibility violations, G4 integrity and forget, Q1 empty-correct, C2 latency p95 ms, R2 maximum exposure share.

| Gate | Prior value | Current value | Need | n | Current status |
|---|---:|---:|---|---:|---|
| G1 eligibility violations | 9.000 | 9.000 | <= 0 |  | FAIL |
| G2 must-not rate | 0.018 | 0.018 | <= 0.02 |  | pass |
| G3 location invariance | 1.000 | 1.000 | >= 1 | 490 | pass |
| G4 integrity and forget | 0.000 | 0.000 | >= 1 |  | FAIL |
| G5 determinism after rebuild | 1.000 | 1.000 | >= 1 |  | pass |
| Q1 empty-correct | 0.608 | 0.608 | >= 0.9 | 51 | FAIL |
| Q2 core found | 0.970 | 0.970 | >= 0.85 | 198 | pass |
| Q3 overall found | 0.957 | 0.957 | >= 0.75 | 348 | pass |
| Q4 section hit | 0.795 | 0.795 | >= 0.75 | 73 | pass |
| Q5 lateral found | 0.949 | 0.949 | >= 0.6 | 39 | pass |
| Q6 paraphrase found | 0.909 | 0.891 | >= 0.35 | 55 | pass |
| C1 tokens median | 184.000 | 186.000 | <= 600 |  | pass |
| C1 tokens p95 | 309.000 | 310.000 | <= 1500 |  | pass |
| C2 latency p95 ms | 63.190 | 125.490 | <= 50 |  | FAIL |
| R1 self-retrieval@5 | 0.965 | 0.965 | >= 0.9 | 2061 | pass |
| R2 maximum exposure share | 0.103 | 0.103 | <= 0.1 |  | FAIL |

#### scale-h2 neutral

Objective rating: 10 pass, 6 FAIL; failing gates: G1 eligibility violations, G2 must-not rate, G4 integrity and forget, Q1 empty-correct, C2 latency p95 ms, R2 maximum exposure share.

| Gate | Prior value | Current value | Need | n | Current status |
|---|---:|---:|---|---:|---|
| G1 eligibility violations | 11.000 | 11.000 | <= 0 |  | FAIL |
| G2 must-not rate | 0.046 | 0.046 | <= 0.02 |  | FAIL |
| G3 location invariance | 1.000 | 1.000 | >= 1 | 460 | pass |
| G4 integrity and forget | 0.000 | 0.000 | >= 1 |  | FAIL |
| G5 determinism after rebuild | 1.000 | 1.000 | >= 1 |  | pass |
| Q1 empty-correct | 0.453 | 0.443 | >= 0.9 | 106 | FAIL |
| Q2 core found | 0.971 | 0.971 | >= 0.85 | 307 | pass |
| Q3 overall found | 0.959 | 0.959 | >= 0.75 | 267 | pass |
| Q4 section hit | 0.871 | 0.871 | >= 0.75 | 271 | pass |
| Q5 lateral found | 0.975 | 0.975 | >= 0.6 | 40 | pass |
| Q6 paraphrase found | 0.917 | 0.917 | >= 0.35 | 48 | pass |
| C1 tokens median | 188.000 | 187.000 | <= 600 |  | pass |
| C1 tokens p95 | 327.000 | 327.000 | <= 1500 |  | pass |
| C2 latency p95 ms | 68.020 | 144.870 | <= 50 |  | FAIL |
| R1 self-retrieval@5 | 0.963 | 0.963 | >= 0.9 | 2302 | pass |
| R2 maximum exposure share | 0.326 | 0.324 | <= 0.1 |  | FAIL |

#### Scope and artifacts

Raw output, pages, timings, fresh stores, binaries, source/corpus hashes and exact command script are under ignored `evals/output/baseline-20261109/`. This report is uncommitted. No hybrid comparison has been performed as part of this report yet.

Timing comparability: this run explicitly sets RAYON_NUM_THREADS=1. The historical controller inherited its environment and did not record that value. Its runs also shared the host with other work. Do not attribute timing differences to an isolated code change.

Accepted curated parity: q4 matches all 1,470 and h2 all 1,380 current accepted ordered pages. The newer scale-q4 capture has only 346 pages; all 346 overlap pages match, while 1,124 are absent from that partial capture. It is not a full-scale baseline.

Resource observation: scale-q4 neutral was sampled at about 21 GiB RSS while retaining the measured store and independent scratch rebuild. This is one grader-process sample, not peak memory or ordinary CLI search memory. G4 is a combined integrity/probe boolean and does not identify a specific failure component.

#### Write-side work inside full neutral runs

Logged POS projection phases include annotation and SQL publication. They are excluded from C2 search timing but included in whole-command runtime. These are elapsed phase sums, not CPU profiles.

| Corpus | Whole command seconds | POS projection seconds | Share of command |
|---|---:|---:|---:|
| q4 | 158 | 89.0 | 56.3% |
| h2 | 175 | 97.7 | 55.8% |
| scale-q4 | 3894 | 2662.8 | 68.4% |
| scale-h2 | 4029 | 2688.7 | 66.7% |

Paired case outcomes from the original exported summaries: q4 gains gold-any
s3a-c-0054; first gains s3a-c-0054 and s4-c-0073, loses s6-c-0086.
H2 first gains h2a1-c-0067. Scale-q4 any loses s3a-c-0050 and s4-c-0005;
first gains s4-c-0073 and loses s3a-c-0050, s4-c-0005 and s5-c-0013.
Scale-h2 XS-any loses h2a3-c-0072; XS-first gains h2a1-c-0103 and loses
h2a3-c-0072. First-page exports do not establish per-case empty/section parity.


<a id="consolidated-evals-baseline-2026-11-08-md"></a>

### evals/BASELINE-2026-11-08.md

The full historical gate values are retained in the prior-value columns of the November 9 comparison. Corpus identities are identical. The original interrupted queue is not a complete scale comparison; the later completed frozen queue is a separate run.

### Linn baseline — indexed query execution — 2026-11-08

Status: complete. Updated: 2026-11-08T18:19:28.982512+00:00.

**Current stage:** All suites attempted.

- Frozen executable: `evals/output/baseline-query-20261108/linn-eval`, SHA-256 `9e1cedcdb6dee8b02de1181c613aecf2a0189fcdf30d98bc43e2d94cbfc3bc25`.
- Source changes captured in `evals/output/baseline-query-20261108/implementation.patch`; base revision `3dd84dfaad038bf9e1107efa84a48bcc9ec8fdb4`. No retrieval, threshold, card-order or scoring-rule change.
- This replaces the slower lookup-only queue after exact page checks. Its partial report and outputs are preserved under `evals/output/baseline-fixed-20261108T160319Z/`.
- Curated dev and holdout ordered exports match all 1,470 and 1,380 original pages. Scale-q4 matches all 1,470 lookup-only pages, which matched the 333 available original pages. All 49 synthetic pages and 24 full-call samples match.
- Scale quality differs from curated quality because distractors are present; it is a separate measurement. Synthetic six-document retrieval is a small synthetic fixture, not an unbiased benchmark.
- Resident invocation timings include first-use cache work, exclude corpus loading, grading and output, and are distinct from neutral per-returned-call C2. Store opening is reported separately. Runs share the machine with other work.
- Corpora, model weights, stores, output, this report and IDEAS.md remain uncommitted. Hybrid comparison has no runnable harness flow here.

**Objective summary:** PASS: 29 tests, 0 failures.

**Objective summary:** MATCHES unchanged measured page: any 410/439 first 350/439 xs 30/42 24/42; empty 35/51 section 55/73.

**Objective summary:** MATCHES unchanged measured page: any 341/354 first 294/354 xs 57/62 50/62; empty 56/106 section 237/271.

**Objective summary:** MATCHES unchanged measured page: any 389/439 first 315/439 xs 27/42 21/42; empty 31/51 section 51/73.

**Objective summary:** MEASURED, unchanged: | all | 49 | 31/49 | 35/49 | 35/49 | 0.6735 | (49 ordered pages identical).

**Objective summary:** MEASURED: 11 pass, 5 FAIL, 0 insufficient; failing gates: G1 eligibility violations, G2 must-not rate, G4 integrity and forget, Q1 empty-correct, R2 maximum exposure share.

**Objective summary:** MEASURED: 11 pass, 5 FAIL, 0 insufficient; failing gates: G1 eligibility violations, G2 must-not rate, G4 integrity and forget, Q1 empty-correct, R2 maximum exposure share.

**Objective summary:** MEASURED: 11 pass, 5 FAIL, 0 insufficient; failing gates: G1 eligibility violations, G4 integrity and forget, Q1 empty-correct, C2 latency p95 ms, R2 maximum exposure share.

**Objective summary:** MEASURED; no scale quality threshold defined: any 326/354 first 273/354 xs 49/62 40/62; empty 48/106 section 225/271.

**Objective summary:** MEASURED: 10 pass, 6 FAIL, 0 insufficient; failing gates: G1 eligibility violations, G2 must-not rate, G4 integrity and forget, Q1 empty-correct, C2 latency p95 ms, R2 maximum exposure share.

Exit: 0. Elapsed: completed verification run.

Exit: 0. Elapsed: completed verification run.

Exit: 0. Elapsed: completed verification run.

Exit: 0. Elapsed: completed verification run.

Exit: 0. Elapsed: completed verification run.

Exit: 0. Elapsed: 39.335 s.

Exit: 0. Elapsed: 44.461 s.

Exit: 0. Elapsed: 804.306 s.

Exit: 0. Elapsed: 220.473 s.

Exit: 0. Elapsed: 854.030 s.


<a id="consolidated-evals-hybrid-comparison-2026-11-10-md"></a>

### evals/HYBRID-COMPARISON-2026-11-10.md

Historical c6d44f1 comparison against saved external baseline predictions, before the later accepted combination.

Objective verdict: **mixed tradeoffs; Linn does not beat the recorded hybrid across these matched slices.** Linn wins gold-first on s5 and first-query MRR on the small index, while hybrid wins gold-any in all three comparisons.

This comparison was performed only after all nine current Linn evals completed and their numbers were reported. See [the full baseline](#consolidated-evals-baseline-2026-11-09-md). No retrieval changes, tuning, model substitution, or author terms were introduced.

#### What was compared

Current Linn at `c6d44f16893637b228f3229ca4671e5d9c9b29e6`, freshly built on exactly the archived document-id populations, against saved external hybrid predictions. The external system was **not rerun**. This is a comparison with recorded predictions, not a fresh head-to-head benchmark or a claim about every current external baseline configuration.

The recorded stack uses EmbeddingGemma 300M, query expansion, RRF and a Qwen3 reranker. It is a comparator only; none of those components was added to Linn. Its provenance and historical protocol are documented in [FINDINGS.md](FINDINGS.md).

Verified all 1,243 large-index and 90 small-index document files against the corresponding captured q4 body bytes. Ingest uses the captured JSONL bodies, not source_path or those verification files. Original record metadata is retained in Linn, including original scope for its fixed penalty and original source_path for ordinary filename behavior. the external system indexed body-only files named by record id; this is a product-level comparison, not an isolated embedding comparison.

Verified every archived query string against its captured case wording: 159 s1/s1b wordings and 273 s5 wordings, with the same 159-wording s1/s1b slice repeated on the small index. All selected cases are dev. Duplicate, missing, extra or mismatched query/page keys, unknown returned ids, changed body bytes and case references outside the captured corpus are rejected.

The Rust comparator reproduces the recorded hybrid gold-any/first numerators before accepting its output. The final run after validation/refactoring reproduced all measured outcomes and full call exports. An intermediate over-strict validation run rejected four forbidden references omitted from the eligible index; those negatives correctly retain their original corpus scope for grading, even though they are not indexed. Two corpus-free scorer fixtures and strict example clippy passed.

#### Scores

| Index / slice | System | Gold any | Gold first | First MRR@5 | XS any | XS first | Empty | Historical overall |
|---|---|---:|---:|---:|---:|---:|---:|---:|
| 1243 / s1 | Current Linn | 45/48 | 39/48 | 58.7% | 4/4 | 4/4 | 4/5 | 85.6% |
| 1243 / s1 | Recorded hybrid | 46/48 | 40/48 | 63.1% | 4/4 | 4/4 | 4/5 | 87.0% |
| 1243 / s5 | Current Linn | 76/81 | 67/81 | 55.8% | 7/10 | 5/10 | 9/10 | 73.7% |
| 1243 / s5 | Recorded hybrid | 78/81 | 64/81 | 56.2% | 9/10 | 7/10 | 9/10 | 80.3% |
| 90 / s1 | Current Linn | 46/48 | 41/48 | 65.5% | 4/4 | 4/4 | 4/5 | 87.8% |
| 90 / s1 | Recorded hybrid | 47/48 | 47/48 | 64.5% | 4/4 | 4/4 | 4/5 | 90.1% |

Objective rating by slice:

- Large s1/s1b: hybrid leads gold-any, gold-first and MRR; cross-scope and empty tie.
- Large s5: Linn leads gold-first by three cases; hybrid leads gold-any, cross-scope and MRR; empty ties.
- Small s1/s1b: Linn leads MRR; hybrid leads gold-any and gold-first; cross-scope and empty tie.

Historical overall is an unweighted mean of six rates, including MRR and tiny cross-scope/empty populations. It is shown to reproduce the historical comparison, not used as an adoption criterion. The two large-index slices have 144 distinct cases and 129 gold cases; small-index s1 repeats the same 53 cases and is not additional independent evidence.

#### Scoring and limits

Gold is each gold object’s record plus its object-level equivalents. Gold and cross-scope inspect the first five cards of the first call, with any across the first three prepared wordings and first on the original wording. MRR@5 uses the first original-query gold rank and averages across gold cases. Cross-scope targets use original corpus scope.

Empty correctness uses the same other-repository target / same-repository forbidden rule as the full retrieval measurement. Linn contributes every returned card across its calls and wordings; the saved hybrid output is its single five-result page per wording. Hints are not answers. Archived hybrid results do not contain section spans or later navigation calls, so section or neutral-gate comparisons are not claimed.

These matched indexes differ from the full 1,376-record q4 corpus used for the 439-gold dev floor. Restricting the full-store pages to these case ids would not be a matched-index comparison; both smaller Linn stores were rebuilt from the required populations. There is no recorded matched hybrid result here for h2, either scale corpus, or the synthetic six-document benchmark. Do not extrapolate these slice results to those suites.

#### Reproduction

Uncommitted native helper: `evals/examples/hybrid_compare.rs`.

The archive supplies docs-q4, docs-sub, queries-sub.json, queries-s5.json, hits-q4-hybrid-sub.jsonl, hits-q4-hybrid-s5.jsonl and hits-sub.jsonl. Existing output stores are refused. No Python scorer or hybrid model runner is invoked. The helper is a local measurement tool, not a new product dependency.

Ignored `evals/output/baseline-20261109/hybrid-final/` retains the two new stores, every current full response, corresponding historical pages, paired case outcomes, corpus/hit-list hashes, model fingerprint and native timing samples. The timing samples are resident Linn calls, not CLI startup or a controlled contemporary comparison with historical hybrid latency. No corpus, model or output was committed.

##### Paired outcome differences: large-s1

- any: Linn-only s1b-c-0006; hybrid-only s1-c-0003, s1b-c-0059.
- first: Linn-only s1-c-0002, s1-c-0006, s1b-c-0006, s1b-c-0010, s1b-c-0038, s1b-c-0049; hybrid-only s1-c-0003, s1-c-0011, s1b-c-0007, s1b-c-0008, s1b-c-0040, s1b-c-0050, s1b-c-0059.
- xs_any: Linn-only none; hybrid-only none.
- xs_first: Linn-only none; hybrid-only none.

##### Paired outcome differences: large-s5

- any: Linn-only s5-c-0074, s5-c-0142; hybrid-only s5-c-0004, s5-c-0041, s5-c-0090, s5-c-0134.
- first: Linn-only s5-c-0001, s5-c-0012, s5-c-0017, s5-c-0028, s5-c-0067, s5-c-0104, s5-c-0112, s5-c-0114, s5-c-0121, s5-c-0142, s5-c-0143, s5-c-0148; hybrid-only s5-c-0013, s5-c-0041, s5-c-0051, s5-c-0058, s5-c-0071, s5-c-0090, s5-c-0105, s5-c-0106, s5-c-0134.
- xs_any: Linn-only none; hybrid-only s5-c-0081, s5-c-0118.
- xs_first: Linn-only s5-c-0031; hybrid-only s5-c-0032, s5-c-0081, s5-c-0118.

##### Paired outcome differences: small-s1

- any: Linn-only none; hybrid-only s1b-c-0059.
- first: Linn-only none; hybrid-only s1-c-0001, s1b-c-0007, s1b-c-0008, s1b-c-0040, s1b-c-0050, s1b-c-0059.
- xs_any: Linn-only none; hybrid-only none.
- xs_first: Linn-only none; hybrid-only none.


<a id="consolidated-evals-pos-continuation-review-2026-11-08-md"></a>

### evals/POS-CONTINUATION-REVIEW-2026-11-08.md

Historical methods and limitations supplement. Later acceptance, controls and model-authorization entries above supersede historical status statements; this does not reopen a closed configuration.

Uncommitted report. Parent product remains at `129897f`; candidate is isolated at
`559fe3ed403d4b48010f7c1314da4b9b77fe4140`, with the partial POS audit and P1
prerequisites in its branch history. No push or PR.

The continuation now connects actual query-and-source POS-selected relations
to lexical scoring before candidate cuts. It retains all original query terms,
hardness, filters, coverage, tiers, scope penalty and card spans. The selected
relation is an adjacent possible modifier/nominal pair inside a complete NP,
with independent SQLite/NLP coordinate reconciliation. The corrected candidate
excludes endpoints inside protected punctuation/path/identifier atoms.

It also supplies an executable constrained assignment with group/no-group,
POS/no-POS and vector-off controls. These remain shadow proposals: their
remaining false semantic completions do not authorize cards. Ordinary-NP joins
and literal field inventories found no complete new POS witnesses under the
declared contracts. Source annotations are guarded derived rows in the same
libSQL file, with invalidation/forget tests and a pinned build-embedded nlprule
resource. These are substantive additions beyond the previous audit-only work.

#### Frozen dev candidate

Baseline is 410/439 and 350/439, with the other four numerators identical.
Paired metric changes: `s3a-c-0054` gains any and first; `s4-c-0073` gains first;
`s6-c-0086` loses first. Thus dev qualifies by aggregate rules, but is not uniform
case improvement. The earlier prototype (411/350) is superseded by the protected
atom correction; only corrected configuration 2 is frozen.

Parent verified frozen ordered-page SHA-256:
`4dd768e5976a68332142d6db007ab8cae02b0941cf2e2428f028a9b24ca2b614`.
Runtime switches: `LINN_POS_PAIRS=1 LINN_ORDERED_PAIR_WEIGHT=1`.
The candidate remains default off; no parent code was merged.

#### Material operating costs

Child release measurements report approximately 90 seconds to prepare selected
source relations for 6,096 blocks, about 4.7 MB projection payload, and a roughly
117 ms fresh-process tagger load. Sample fresh CLI time is about 190 ms enabled
versus 70 ms disabled. Final resident medians are about 6.0 versus 4.6 ms.
These have different measurement scopes and are not neutral C2 gates or a clean
scale benchmark. They are material costs even if retrieval generalizes.

#### Parent-only one-shot confirmation

After reviewing the code, unchanged scoring path, configuration history and
paired changes, parent froze configuration 2 and launched exactly one h2-all run:

Run directory: `<historical-checkout>`.
Native stdout/stderr are captured as `parent-h2.stdout` and `parent-h2.stderr`
in that ignored output directory. The baseline h2 is any 341/354, first 294/354,
xs 57/62 and 50/62, empty 56/106, section 237/271.

Completed successfully (process exit 0), 1,473 records / 6,477 blocks, all 460
cases. POS projection preparation alone took 84,791.572 ms.

**Verdict: rejected under LLP 0005.** First and section each rise by one, but
cross-scope-any falls 57 to 56 and cross-scope-first falls 50 to 49. Those losses
cannot be compensated by improvements in other measures. No per-case holdout
investigation, retuning, second holdout configuration or product merge follows.
The unchanged production baseline remains the reference page. The result closes
this frozen corrected POS configuration, not every future linguistic mechanism.

Detailed implementation, fixtures, checks, source identities, assignments,
inventories and cost evidence: child report (historical child source).
No corpus, model, linguistic resource or raw output is committed.


<a id="consolidated-evals-pos-recovery-2026-11-08-md"></a>

### evals/POS-RECOVERY-2026-11-08.md

Historical methods and limitations supplement. Later acceptance, controls and model-authorization entries above supersede historical status statements; this does not reopen a closed configuration.

Uncommitted measurement report. The follow-up study measured further
adjustment after the first POS confirmation failed. q4-dev and h2-all therefore
both participated in selection. These are tuning results, not an independent
holdout confirmation or proof of generalization. Scorer, cases, source bytes,
model, scope penalty, hard terms and evaluation thresholds are unchanged.

#### Result

**Selected weight 0.75: recovers both original cross-scope losses and retains
the measured retrieval gains.** POS-selected nominal pairs are enabled by
default. Existing stores need `index:rebuild`; `LINN_POS_PAIRS=0` restores legacy
ranking. Explicit `LINN_ORDERED_PAIR_WEIGHT` remains an experimental override.

| Configuration | q4 any / first | q4 XS any / first | q4 empty / section | h2 any / first | h2 XS any / first | h2 empty / section |
|---|---|---|---|---|---|---|
| Legacy |410/439,350/439|30/42,24/42|35/51,55/73|341/354,294/354|57/62,50/62|56/106,237/271|
| Original POS weight 1 |411/439,351/439|30/42,24/42|35/51,55/73|341/354,295/354|56/62,49/62|56/106,238/271|
| Recovery 0.5 |411/439,351/439|30/42,24/42|35/51,55/73|341/354,295/354|57/62,50/62|56/106,237/271|
| Recovery 0.75 |411/439,351/439|30/42,24/42|35/51,55/73|341/354,295/354|57/62,50/62|56/106,239/271|
| Recovery 0.25 |411/439,351/439|30/42,24/42|35/51,55/73|341/354,295/354|57/62,50/62|56/106,237/271|

All three adjustment weights were declared before their measurements. Weight
0.75 has the larger section result with every other aggregate equal to the
other recovery configurations. No case-specific rule or new semantic gate was
added; the adjustment simply reduces how strongly linguistic pair evidence can
displace the original lexical ranking. No additional variants were tried.

#### Paired document-hit evidence

Diagnostics compare existing native first-page exports; the native Rust scorer
remains authoritative. Against legacy:

- q4 `s3a-c-0054`: gold-any and original-wording success gained.
- q4 `s4-c-0073`: original-wording success gained.
- q4 `s6-c-0086`: original-wording success lost. The aggregate improvement is
  not uniform case improvement.
- h2 `h2a1-c-0067`: original-wording success gained; other document-hit and
  cross-scope case outcomes are unchanged.

The specific losses from weight 1 are recovered: `h2a2-c-0094` again shows its
cross-scope target on the original wording; `s7-c-0080` again shows its required
target on some wording. These IDs are diagnostic output only. The source and
ranking code contain no case identities, expected answers or corpus exceptions.

Artifacts are under ignored `evals/output/POS-RECOVERY/`: predeclaration, every
stdout/stderr and ordered-page export, `recovered-original-xs-losses.json` and
`paired-document-hit-changes.json`. The adjusted section/empty aggregates come
from the unchanged native scorer over every card/call, not the page diagnostic.
No new per-case section classification is claimed by the first-page comparison.

#### Application and verification

The original partial audit, positional prerequisite and completed POS candidate
were cherry-picked into reimplement, retaining their history. The recovery
changes default POS activation and its default bonus weight, documents migration,
and records the the recorded exception to the prior one-shot rule in AGENTS and
LLP 0005. Other governing LLPs describe current defaults; FINDINGS preserves
both the failed frozen result and this later authorized decision.

Initial parameter measurements used the exact candidate executable and local
copies of its already-prepared q4/h2 stores; annotations and their identity are
independent of the bonus weight. Final default-source runs compare every ordered
page against the selected 0.75 exports, and an explicit legacy run compares
against all 1,470 original q4 pages. Check logs and replay commands are retained
in `verify.py` and `check-*.log` in the ignored output directory.

Final verification passed: formatting, strict workspace/all-target/all-feature
clippy, workspace/all-feature tests, all three real-resource eval library tests,
LLP references, whitespace checks and `nix build --cores 1`, including release
package tests and binary fixup. The default q4 run exactly matches all 1,470
selected 0.75 pages; h2 matches all 1,380 selected pages. The explicit legacy q4
run exactly reproduces all 1,470 old pages and the original six numerators.
No corpus, linguistic binary, embedding weights or eval output is staged.

#### Costs and practical limits

This carries the earlier POS costs: roughly 85–90 seconds to prepare curated
source annotations on a rebuild, and roughly 117 ms for a fresh-process tagger
load in the recorded release probe. The former applies to enabled writes through
the existing rebuild path; the latter affects CLI queries. No daemon, runtime
download, second runtime or second embedding model is introduced. The linguistic
resource is embedded at build time with its separate notices.

The measured gains are modest. Reusing h2 for selection prevents claiming that
these adjustments independently generalize; no matched hybrid, scale or new
neutral measurement was performed in this adjustment task.


<a id="consolidated-evals-encoder-study-md"></a>

### evals/ENCODER-STUDY.md

Historical methods and limitations supplement. Later acceptance, controls and model-authorization entries above supersede historical status statements; this does not reopen a closed configuration.

Status: closed direct substitutions, neither selected. Accepted retrieval remains
unchanged. Experimental encoder support was removed after measurement; the
configuration details describe the saved experiment, not current runtime options.

Replacement-encoder research measured candidates that embed in the binary and remain
similarly fast. The first candidate is minishlab/potion-code-16M-v2, revision
`e9d2a44ca6a05ac6685f3b23709ea57eb7352d5b`. Its MIT-licensed static architecture
uses the existing model2vec-rs loader. The published F16 weights are decoded by
that loader; no extra quantization or second inference stack is introduced.
Its code-oriented training is the hypothesis for better technical retrieval,
not evidence that prose notes will improve.

Sources: [model card](https://huggingface.co/minishlab/potion-code-16M-v2),
[published files](https://huggingface.co/minishlab/potion-code-16M-v2/tree/e9d2a44ca6a05ac6685f3b23709ea57eb7352d5b).

#### Declared configuration

One direct substitution, without threshold or ranker adjustments. Keep the
existing 512-token and 3,584-character bounds, pooling and normalization. The
candidate produces 256 dimensions; append zeros after normalization so existing
512-column SQL projections and kernels need no migration. The candidate's own
tokenizer also supplies the encoder's token pieces. Rebuild every numerical
projection in a separate store. Retain POS and lexical algorithms unchanged.

Build-time `LINN_RESEARCH_MODEL_DIM=256` permits the candidate; the normal build
requires 512 dimensions. `LINN_MODEL` supplies local embedded files. There is no
runtime model selection or download. Weights, executables and output stay under
ignored local evaluation storage.

Before dev selection, compare model loading, query encoding and block encoding
on identical captured q4 text. Use one Rayon thread, warm repetitions and fresh
processes. A slowdown above 25% in median encoding or fresh invocation is not
"similarly fast" for this screen. Compare full query latency as well; do not
hide a gate-induced cost behind cheap encoding. This margin is a research speed
screen, not a retrieval threshold.

Score q4 dev with the unchanged native evaluator and retain all ordered pages
and paired case outcomes. Select only under LLP 0005; no h2 unless dev qualifies.
No trained corpus mapping, author fields, generated content, hybrid fusion or
reranker is part of this experiment. Existing closed fixed-encoder trials stay
closed.

#### Second declared configuration

After the code candidate failed the speed screen and lost dev retrieval, screen
sentence-transformers/static-retrieval-mrl-en-v1. This is an independently trained
static retrieval model with Matryoshka training. Keep its first 512 of 1,024
coordinates before normalization, as supported by that training. Use the same
speed screen, unchanged wrapper limits and retrieval rules, with a separate
executable and fully rebuilt store. There is no threshold tuning and no combination
of the two replacements. Its missing model2vec config is supplied locally as
`{"normalize":true}`; the actual weights and tokenizer are unmodified.

Source: [model card](https://huggingface.co/sentence-transformers/static-retrieval-mrl-en-v1).

The pinned revision is `f60985c706f192d45d218078e49e5a8b6f15283a`.
Before considering a new retrieval integration, compare raw cosine top-five
distinct-record coverage for both replacements and the accepted encoder on
original dev queries without explicit filters. Eligibility, tie order and
original blocks stay fixed. This is a candidate-signal diagnostic, not a
replacement page or silence measurement; no thresholds are fit to its results.

#### Results

| Model | Gold any / first | Cross-scope any / first | Empty | Section | Changed pages |
|---|---|---|---|---|---|
| Accepted | 411/439, 351/439 | 30/42, 24/42 | 35/51 | 55/73 | 0 |
| Code | 410/439, 346/439 | 30/42, 24/42 | 35/51 | 55/73 | 88 |
| Matryoshka | 411/439, 351/439 | 30/42, 24/42 | 35/51 | 55/73 | 60 |

Objective rating: code fails retrieval and encoding speed; Matryoshka is flat,
with no case-level gold/cross-scope changes, so is not selected. Neither warrants
h2. No thresholds were adjusted; no scale, neutral or hybrid run was performed.

Code loses gold-any on s4-c-0010. Gold-first losses are s1-c-0001, s4-c-0010,
s4-c-0020, s5-c-0098 and s6-c-0035. There are no gold or cross-scope gains.
The unchanged native evaluator supplies empty and section measurements; saved
pages supply gold/cross-scope attribution only.

#### Timing

Three fresh encoder processes per model, seven rounds each over687 original q4
queries and6096 original blocks, one Rayon thread. Median over21 rounds divided
by text count; no labels enter timing:

| Model | Model load median ms | Query µs | Block µs |
|---|---:|---:|---:|
| Accepted | 54.207 | 8.893 | 74.277 |
| Code | 57.224 | 14.879 | 129.079 |
| Matryoshka | 32.796 | 9.597 | 80.503 |

First process loads were261.980/113.529/230.032 ms respectively. These are
observed cache effects, not controlled cold-cache measurements. Code's smaller
artifact did not produce a repeatable median load improvement.

Fresh actual CLI: first12 evaluated dev cases in corpus order, twice each,
alternating executables and reversing order in round two. Real clock and
cwd-derived scope, same Cargo release profile, captured output, exit0/1 allowed.
Warm OS caches; these are latency probes, not fixed-clock scoring:

| Model | Median ms | P95 ms | Maximum ms |
|---|---:|---:|---:|
| Accepted | 248.210 | 375.979 | 776.505 |
| Matryoshka | 246.234 | 348.516 | 744.578 |

P95 uses sorted index floor(0.95*(n-1)); this does not replace the neutral
grader's percentile rule. Matryoshka passes the speed screen;24 samples per
executable do not establish a meaningful CLI speed gain. Both native binaries
contain their model bytes and need no external model file or inference runtime.

Full dev resident agent invocation median/p95: accepted5.007/12.425 ms,
code4.427/11.061 ms, Matryoshka4.367/10.781 ms. Sequential single runs, excluding
opening/grading; not a controlled throughput comparison or neutral per-call
measurement. Changed fallback paths change work. New store construction was
25.526 s/code and29.295 s/Matryoshka; accepted reused a store, so its opening is
not a comparable construction measurement.

#### Raw candidate signal

485 original dev queries without explicit filters:436 with gold and49 without.
Rank eligible original blocks by cosine descending, row id ascending on ties;
keep five distinct records. No lexical gate, scope penalty, POS or silence gate.

| Encoder alone | Gold hits /436 | Accepted misses recovered | Accepted hits lost |
|---|---:|---:|---:|
| Accepted | 293 | 32 | 89 |
| Code | 309 | 32 | 73 |
| Matryoshka | 293 | 23 | 80 |

The full accepted page hits350/436 on this subset. All49 no-gold diagnostic
queries also show results; this is not an empty-correct score. Code has stronger
raw aggregate recall but does not recover more accepted misses and loses full
retrieval quality on direct substitution. These observations do not authorize
reopening rejected score blends or fitting silence thresholds. Static pooling
still lacks word order and relation direction.

#### Artifacts and limits

Ignored `evals/output/encoder-study/` contains pinned metadata, weights,
executables, complete pages, timings, paired changes, raw candidates, cost
samples and the experimental patch. Native examples `encoder_cost`,
`encoder_candidates`, `page_changes` and `cli_cost` perform diagnostics. Ordinary
`linn-eval` performs retrieval scoring. Accepted replay matches all1470 pages.

The [all-MiniLM-L6-v2 model card](https://huggingface.co/sentence-transformers/all-MiniLM-L6-v2)
was reviewed for contextual token encoding followed by pooling. No contextual
model was downloaded or implemented; its speed within Linn remains unmeasured,
not disproved. This round establishes no substantial gain and no hybrid win.

The wrong-model diagnostic probe found that `Store::open` refreshes fallback
numerical publications for the executable's model but does not rebuild the
main block-vector table. A model check after open is therefore too late. The
diagnostic now checks publication identity via a read-only connection first;
wrong-model refusal and matching-model output parity are checked. The initial
negative probe changed only the experimental code store's fallback publication,
after all scores and candidate exports were captured. Reopening with the saved
code executable restores its publication and reproduces the original candidate
export byte-for-byte. Any future adopted replacement needs an explicit whole-
store rebuild or refusal contract; swapping a binary over an old store is unsafe.


<a id="consolidated-evals-system-review-parent-md"></a>

### evals/SYSTEM-REVIEW-PARENT.md

Historical methods and limitations supplement. Later acceptance, controls and model-authorization entries above supersede historical status statements; this does not reopen a closed configuration.

Research notes, not a specification or adoption claim. Inspected source at
8310cfad51bc342acb0bdf947a5b2e09efdeee3f. No retrieval modification or new search
run was made for this review.

#### Remaining headroom

The saved current q4 dev export contains exactly the expected 1,470 distinct
case/wording keys. Recomputing document hits from each gold object's record and
equivalents gives 351 original hits, 60 alternate-only hits, and 28 cases missed
on every wording. This reproduces 411 gold-any and 351 gold-first. Diagnostic
details are in ignored `evals/output/system-review/baseline-headroom.json`.

This is evidence of wording sensitivity, not proof of its cause. Alternate
queries can change specificity as well as surface vocabulary. The 60 cases
need paired source/query and stage evidence before attributing them to synonym
failure or a gate. Categories overlap; their counts must not be summed.
First-page exports cannot verify no-gold or section, which require all calls
and source spans. Keep the native scorer authoritative for those measurements.

#### Embeddings are active, but mostly outside primary lexical ranking

- `src/search.rs:237`: POS-selected occurrence evidence changes lexical scores.
  This bonus is not an embedding comparison.
- `src/search.rs:808`: vector-based rare-neighbor selection can append a card
  only when the lexical/fill page already has between one and four cards.
  A full page cannot benefit through this route. This is a structural limit,
  not evidence that changing it will improve retrieval.
- `src/search.rs:1031`: embeddings choose vector hints, but hints are pointers,
  not scored cards. Better hint relevance alone does not raise document-hit
  counts in the fixed prepared-query measurement.
- `src/fallback.rs:80`: the first successful base page stops rewrite attempts.
  Model-tokenizer pieces are used in the later rewrite; tokenization is not
  semantic similarity or POS-plus-vector matching.
- `src/fallback.rs:587`: a retained rewrite page stops beam, form-block,
  form-section and sentence-pair attempts. Each successful later attempt also
  stops the remainder. These routes cannot repair an already populated wrong
  page under the current contract.
- `src/fallback.rs:643`: later identifier-piece and rare-form attempts also
  require an empty first page. Quoted-filename promotion is a separate final
  exception that can affect a populated page.
- `src/fallback.rs:851`: stopping tests the first call's cards, not a learned
  confidence estimate or semantic correctness.

Thus, adding another empty-page embedding fallback cannot address every
original-query miss. A promising experiment must identify its reachable
population before claiming that a better representation will improve the page.
Conversely, removing the stopping rule indiscriminately may displace correct
lexical results and harm silence. These are experiment hypotheses, not fixes.

#### What a substantial candidate must demonstrate

Compare against the applied POS baseline, not the older disabled baseline.
Record both gains and losses by case and wording, distinguish original-only
improvements from recovery of all-wording misses, and report every attempted
configuration. Relevance is assessed from the original body bytes; relevance
labels and alternative query lists must not enter the store.

For POS-plus-embedding proposals, distinguish an ordering feature from a claim
of semantic certification. Pooled-vector role reversals disprove certification,
but do not alone disprove a fallible ranking feature. Use no-POS and lexical-only
controls to determine whether linguistic filtering helps or simply prevents
the mechanism from running. Inspect candidate admission as well as scoring.

Keep separate accounting for source preparation, fresh CLI model/tagger load,
resident query cost and disk growth. A result confined to a few hand-picked
fixtures is not corpus evidence. Reused h2 measurements are not independent
holdout confirmation. No new adoption threshold is defined by these notes.


<a id="consolidated-evals-research-checkpoint-2026-11-09-md"></a>

### evals/RESEARCH-CHECKPOINT-2026-11-09.md

Historical methods and limitations supplement. Later acceptance, controls and model-authorization entries above supersede historical status statements; this does not reopen a closed configuration.

The significant-search-improvement goal is not achieved. The accepted store is
unchanged. No new candidate from this system study has passed the full adoption
process. This is a decision checkpoint, not proof that improvement is impossible.

#### Verified evidence

The experiment ledger reaches configuration112. Trials cover POS plus existing
embeddings, lexical/semantic alignment, source-unit constraints, stemming and
scoring consistency, matchers and cuts, passage preservation, record statistics,
field normalization, score curves, record support, table cuts, likelihood
scoring, record-wide coverage, and source-density vector selection. The report
series preserves individual definitions, rejected settings and paired outcomes.
Closed settings must not be retried under new names.

The two frozen combinations tested on h2 both failed: the record-IDF combination
returned345/354 any,301/354 first, XS56/62 and48/62, empty55/106, section246/271;
the curve combination343/354 and293/354, XS55/62 and48/62, empty54/106,
section244/271. Accepted h2 is341/354 and295/354, XS57/62 and50/62,
empty56/106, section239/271. Improvements in some metrics did not excuse losses.
h2 has been used before and is not independent new validation.

The saved-output headroom audit (before the latest density experiments) found
most never-recovered original queries blocked before primary lexical ranking:
30 missing hard literals,26 insufficient coverage/local hard co-occurrence,
1 zero-content-overlap,7 post-admission,2 unclassified. It is a diagnostic subset,
not a claim that no future candidate generator can retrieve the records.

Raw vector density correction reveals complementary records but is not semantic
proof. Its tested neighbour and primary-score integrations fail selection.
Repeatedly tuning their weights is not a justified next experiment. Similarly,
the proposed abbreviation avenue overlaps the closed five-character-prefix,
cosine-gated vocabulary rewrite; it is not being rerun as a new mechanism.

Last restored native dev run: any411/439 first351/439 XS30/42 and24/42,
empty35/51 section55/73, with all1,470 ordered pages byte-identical to accepted.

#### Boundary requiring study direction

AGENTS.md requires the specified local static model, prohibits substituting new
embeddings and forbids a reranker. Those constraints remain binding. A broad
request to improve search does not silently authorize a replacement model,
model downloads, generated document evidence or a second runtime.

A reasonable next research direction is an isolated benchmark of one small
contextual encoder as a possible replacement for the static encoder, rather
than adding a second model or a reranker. The hypothesis is that context could
help distinguish relations and disambiguate query/source terms where static
averages failed. This is not an improvement claim: negation and relation
correctness still require adversarial tests and complete retrieval evaluation.
Any approved experiment must preserve original text, unread author fields,
scoring rules and accepted defaults, and measure fresh CLI latency and packaging
cost. No download, implementation or change to the specification is authorized
by this document.

Alternatively, keep the fixed-model boundary and continue only when a distinct,
source-supported mechanism has a credible population; do not manufacture new
configurations merely to prolong tuning. Fresh independently prepared evaluation
data would improve the evidentiary basis of either path. It would supplement,
not replace or retune, the frozen measurements.

#### Source evidence

- `AGENTS.md`: model, original-content and closed-measurement constraints.
- [consolidated finding](#consolidated-evals-results-confirmation-2026-11-09-md): first frozen rejection.
- [consolidated finding](#consolidated-evals-results-curve-interactions-2026-11-09-md): second frozen rejection.
- [consolidated finding](#consolidated-evals-results-headroom-2026-11-09-md): scope and limits of the headroom audit.
- [consolidated finding](#consolidated-evals-results-density-primary-2026-11-09-md): latest complete trials and restoration.
- `evals/FINDINGS.md`: closed measurements and primary references.

This checkpoint changes no product rule, adoption criterion or goal status.


### Additional retained reference conclusions

1. **[SQLite FTS5 Extension](https://www.sqlite.org/fts5.html)** — official documentation for external-content indexes, tokenizers and integrity checks. Internal index consistency and agreement with external content are distinct checks. Supports integrity/conformance investigation; it does not validate Linn’s custom BM25 or suggest changing its thresholds. Sections: [external content pitfalls](https://www.sqlite.org/fts5.html#external_content_table_pitfalls), [integrity-check](https://www.sqlite.org/fts5.html#the_integrity_check_command). Existing reference, rechecked.
2. **[CommonMark Spec, version 0.31.2](https://spec.commonmark.org/0.31.2/)** — defines Setext headings, container blocks and reference links beyond Linn’s current regex subset. This establishes concrete syntax to investigate, not that adopting the entire standard improves retrieval. No body rewriting is implied. Existing reference, rechecked.
3. **[Web Annotation Data Model](https://www.w3.org/TR/annotation-model/)** — W3C Recommendation; Text Quote Selectors use exact text with prefix/suffix, and position selectors depend on the selected representation. This motivates explicit ambiguity/version handling. Its character-based normalized selector model is not interchangeable with Linn’s byte fetch IDs. Verified primary reference; Linn use unmeasured.
4. **[Unicode Standard Annex #15: Unicode Normalization Forms](https://www.unicode.org/reports/tr15/)** — distinguishes canonical and compatibility equivalence and supplies conformance guidance. Useful for detecting unintended normalization changes. It is not permission to normalize immutable bodies, erase meaningful identifier differences, or unify intentionally different token paths. Verified primary reference; proposed Linn audit unmeasured.
5. **[BEIR: A Heterogenous Benchmark for Zero-shot Evaluation of Information Retrieval Models](https://arxiv.org/abs/2104.08663)** — Thakur et al. compare retrieval systems across heterogeneous tasks and describe BM25 as a robust baseline. The lesson here is evaluation coverage and transfer risk, not importing their expensive ranking methods or numerical results as Linn predictions. Verified abstract; no Linn run.
6. **[BRIGHT: A Realistic and Challenging Benchmark for Reasoning-Intensive Retrieval](https://arxiv.org/abs/2407.12883)** — Su et al. study queries whose relevant documents require reasoning beyond surface similarity. This supports distinguishing reasoning-dependent misses from tokenizer defects. Its reasoning interventions are not authorized additions to Linn. Verified abstract; no benchmark preparation or Linn run.
7. **[Lost in the Middle: How Language Models Use Long Contexts](https://arxiv.org/abs/2307.03172)** — Liu et al. find that relevant-evidence position affects performance in their long-context tasks. This cautions against assuming that fetching a whole long document establishes useful access. It does not justify changing Linn card order. Verified abstract; current-reader applicability untested.



### Accepted combination: complete measurement supplement

These are the frozen R2 measurements subsequently accepted explicitly by the
operator. “REGRESSION” describes a metric comparison with its predecessor, not
its final adoption status. No new runs were performed; scale-h2 neutral was
canceled and remains unmeasured for this combination. Earlier complete neutral
tables belong to their earlier binaries.

#### Completed full-suite measurements

##### q4 retrieval

Objective rating: **IMPROVED** against then-accepted predecessor; numerator deltas (gold-any, gold-first, XS-any, XS-first, no-gold, section): [1, 3, 0, 0, 0, 0].

any 412/439 first 354/439 xs 31/42 25/42; empty 35/51 section 55/73


Ordered pages changed: 291/1470. Paired record-metric outcomes:

- gold_any_gain: s2b-c-0006.
- gold_first_gain: s2b-c-0006, s2b-c-0020, s4-c-0020, s5-c-0051, s6-c-0102.
- gold_first_loss: s4-c-0003, s6-c-0015.


agent invocation timings: n=1470 median_ms=7.899 p95_ms=23.747 max_ms=101.640; store_ms=1.805; excludes corpus loading, grading and output; not the neutral per-call gate

##### h2 retrieval

Objective rating: **REGRESSION** against then-accepted predecessor; numerator deltas (gold-any, gold-first, XS-any, XS-first, no-gold, section): [0, 1, -1, -1, -1, -2].

any 341/354 first 298/354 xs 56/62 49/62; empty 55/106 section 237/271


Ordered pages changed: 297/1380. Paired record-metric outcomes:

- gold_first_gain: h2a3-c-0074.
- xs_any_loss: h2a3-c-0090.
- xs_first_loss: h2a3-c-0090.


agent invocation timings: n=1380 median_ms=9.516 p95_ms=29.205 max_ms=129.841; store_ms=2.669; excludes corpus loading, grading and output; not the neutral per-call gate

##### scale-q4 retrieval

Objective rating: **IMPROVED** against then-accepted predecessor; numerator deltas (gold-any, gold-first, XS-any, XS-first, no-gold, section): [2, 4, 0, 0, 0, 0].

any 388/439 first 316/439 xs 28/42 22/42; empty 31/51 section 51/73


Ordered pages changed: 324/1470. Paired record-metric outcomes:

- gold_any_gain: s3b-c-0013, s6-c-0102.
- gold_first_gain: s2b-c-0020, s3b-c-0013, s4-c-0020, s5-c-0051, s6-c-0102.
- gold_first_loss: s5-c-0129.


agent invocation timings: n=1470 median_ms=108.837 p95_ms=412.006 max_ms=4643.826; store_ms=3.386; excludes corpus loading, grading and output; not the neutral per-call gate

##### scale-h2 retrieval

Objective rating: **REGRESSION** against then-accepted predecessor; numerator deltas (gold-any, gold-first, XS-any, XS-first, no-gold, section): [-2, -1, 0, 1, 0, -1].

any 325/354 first 275/354 xs 48/62 41/62; empty 47/106 section 225/271


Ordered pages changed: 354/1380. Paired record-metric outcomes:

- gold_any_loss: h2a2-c-0019, s7-c-0004.
- gold_first_gain: h2a3-c-0078.
- gold_first_loss: h2a2-c-0019, s7-c-0004.
- xs_first_gain: h2a2-c-0094.


agent invocation timings: n=1380 median_ms=125.543 p95_ms=484.056 max_ms=3458.919; store_ms=3.253; excludes corpus loading, grading and output; not the neutral per-call gate

##### q4 neutral

Objective rating: **12 passing rows, 4 failing rows, 0 insufficient-N rows**. C1 has two rows. A completed command is not a passing gate table.

| gate | value | need | n | status |
|---|---|---|---|---|
| G1 eligibility violations | 26.000 | <= 0 |  | FAIL |
| G2 must-not rate | 0.020 | <= 0.02 |  | pass |
| G3 location invariance | 1.000 | >= 1 | 490 | pass |
| G4 integrity and forget | 0.000 | >= 1 |  | FAIL |
| G5 determinism after rebuild | 1.000 | >= 1 |  | pass |
| Q1 empty-correct | 0.686 | >= 0.9 | 51 | FAIL |
| Q2 core found | 0.985 | >= 0.85 | 198 | pass |
| Q3 overall found | 0.980 | >= 0.75 | 348 | pass |
| Q4 section hit | 0.822 | >= 0.75 | 73 | pass |
| Q5 lateral found | 1.000 | >= 0.6 | 39 | pass |
| Q6 paraphrase found | 0.945 | >= 0.35 | 55 | pass |
| C1 tokens median | 176.000 | <= 600 |  | pass |
| C1 tokens p95 | 287.000 | <= 1500 |  | pass |
| C2 latency p95 ms | 7.920 | <= 50 |  | pass |
| R1 self-retrieval@5 | 0.987 | >= 0.9 | 1070 | pass |
| R2 maximum exposure share | 0.229 | <= 0.1 |  | FAIL |

##### h2 neutral

Objective rating: **11 passing rows, 5 failing rows, 0 insufficient-N rows**. C1 has two rows. A completed command is not a passing gate table.

| gate | value | need | n | status |
|---|---|---|---|---|
| G1 eligibility violations | 25.000 | <= 0 |  | FAIL |
| G2 must-not rate | 0.052 | <= 0.02 |  | FAIL |
| G3 location invariance | 1.000 | >= 1 | 460 | pass |
| G4 integrity and forget | 0.000 | >= 1 |  | FAIL |
| G5 determinism after rebuild | 1.000 | >= 1 |  | pass |
| Q1 empty-correct | 0.519 | >= 0.9 | 106 | FAIL |
| Q2 core found | 0.993 | >= 0.85 | 307 | pass |
| Q3 overall found | 0.981 | >= 0.75 | 267 | pass |
| Q4 section hit | 0.900 | >= 0.75 | 271 | pass |
| Q5 lateral found | 1.000 | >= 0.6 | 40 | pass |
| Q6 paraphrase found | 0.958 | >= 0.35 | 48 | pass |
| C1 tokens median | 180.000 | <= 600 |  | pass |
| C1 tokens p95 | 299.000 | <= 1500 |  | pass |
| C2 latency p95 ms | 8.500 | <= 50 |  | pass |
| R1 self-retrieval@5 | 0.989 | >= 0.9 | 1138 | pass |
| R2 maximum exposure share | 0.429 | <= 0.1 |  | FAIL |

##### scale-q4 neutral

Objective rating: **11 passing rows, 5 failing rows, 0 insufficient-N rows**. C1 has two rows. A completed command is not a passing gate table.

| gate | value | need | n | status |
|---|---|---|---|---|
| G1 eligibility violations | 6.000 | <= 0 |  | FAIL |
| G2 must-not rate | 0.016 | <= 0.02 |  | pass |
| G3 location invariance | 1.000 | >= 1 | 490 | pass |
| G4 integrity and forget | 0.000 | >= 1 |  | FAIL |
| G5 determinism after rebuild | 1.000 | >= 1 |  | pass |
| Q1 empty-correct | 0.608 | >= 0.9 | 51 | FAIL |
| Q2 core found | 0.970 | >= 0.85 | 198 | pass |
| Q3 overall found | 0.957 | >= 0.75 | 348 | pass |
| Q4 section hit | 0.795 | >= 0.75 | 73 | pass |
| Q5 lateral found | 0.949 | >= 0.6 | 39 | pass |
| Q6 paraphrase found | 0.891 | >= 0.35 | 55 | pass |
| C1 tokens median | 185.000 | <= 600 |  | pass |
| C1 tokens p95 | 310.000 | <= 1500 |  | pass |
| C2 latency p95 ms | 123.920 | <= 50 |  | FAIL |
| R1 self-retrieval@5 | 0.965 | >= 0.9 | 2061 | pass |
| R2 maximum exposure share | 0.103 | <= 0.1 |  | FAIL |

##### scale-h2 neutral

Objective rating: **NOT MEASURED — canceled at the study plan**. Partial execution supplies no final gate table and is not a pass.

##### Synthetic six-document retrieval

Objective rating: **FLAT** against then-accepted predecessor: Hit@1 31/49, Hit@3 and Hit@5 35/49, MRR@5 0.6735.

| group | cases | Hit@1 | Hit@3 | Hit@5 | MRR@5 |
|---|---|---|---|---|---|
| all | 49 | 31/49 | 35/49 | 35/49 | 0.6735 |
| deep-research | 25 | 12/25 | 16/25 | 16/25 | 0.5600 |
| deep-research/hard | 25 | 12/25 | 16/25 | 16/25 | 0.5600 |
| standard | 24 | 19/24 | 19/24 | 19/24 | 0.7917 |
| standard/easy | 6 | 6/6 | 6/6 | 6/6 | 1.0000 |
| standard/fusion | 6 | 4/6 | 4/6 | 4/6 | 0.6667 |
| standard/hard | 6 | 6/6 | 6/6 | 6/6 | 1.0000 |
| standard/medium | 6 | 3/6 | 3/6 | 3/6 | 0.5000 |

##### Matched historical hybrid comparison

Objective rating: **GAP REMAINS**: candidate gold-any trails recorded hybrid in all three slices. The synthetic benchmark is separate.

| Slice | System | Gold-any | Gold-first | MRR@5 | XS-any / first | No-gold |
|---|---|---|---|---|---|---|
| large-s1 | Accepted | 45/48 | 40/48 | 0.6076 | 4/4, 4/4 | 4/5 |
| large-s1 | Candidate | 45/48 | 40/48 | 0.6094 | 4/4, 4/4 | 4/5 |
| large-s1 | Recorded hybrid | 46/48 | 40/48 | 0.6309 | 4/4, 4/4 | 4/5 |
| large-s5 | Accepted | 76/81 | 66/81 | 0.5609 | 8/10, 6/10 | 9/10 |
| large-s5 | Candidate | 76/81 | 67/81 | 0.5677 | 8/10, 6/10 | 9/10 |
| large-s5 | Recorded hybrid | 78/81 | 64/81 | 0.5621 | 9/10, 7/10 | 9/10 |
| small-s1 | Accepted | 46/48 | 42/48 | 0.6760 | 4/4, 4/4 | 4/5 |
| small-s1 | Candidate | 46/48 | 42/48 | 0.6830 | 4/4, 4/4 | 4/5 |
| small-s1 | Recorded hybrid | 47/48 | 47/48 | 0.6448 | 4/4, 4/4 | 4/5 |


### Consolidation disposition

The dated notes above preserve historical measurements and untested proposals;
they do not override the accepted source-evidence/BM25L page or the governing LLPs.
Rejected/flat configurations remain closed. Corpus files, raw outputs, models,
per-case exports and temporary instrumentation were not transferred into this log.
No retrieval code, constants, card order, scorer or corpus changed in this cleanup.

### Commit-readiness check: intermittent SQLite initialization failure

On November 10, the documentation/helper cleanup passed formatting, strict
workspace clippy and LLP references. The first parallel workspace test run
failed 25 product-library tests after libSQL initialization poisoned its `Once`;
an unchanged retry passed the full suite, including the two hybrid scorer tests.
The Nix package compiled but failed its product tests with the same pattern:
`sqlite3_config(SQLITE_CONFIG_SERIALIZED)` returned 21 instead of 0, followed
by poisoned-initialization failures. Product source, product tests and dependency
lockfile were unchanged in this cleanup.

The direct Porter adapter opens SQLite through FFI, while libSQL configures its
threading mode on its own first database open. This is consistent with an
initialization-order conflict; it is not yet an isolated causal reproducer or a
retrieval regression. Keep it as an unresolved reliability finding, not a passing
package check. No product workaround or test suppression was applied. Native
build/test logs remain in ignored `evals/output/document-consolidation/`.

### November 10 latency pass: native exact tokenization and initialization

The tokenizer-first integration test isolates the earlier SQLite initialization
conflict. Enter libSQL's process-wide initialization guard before opening the
thread-owned raw tokenizer connection. Workspace tests and the Nix package checks
now pass; no retry or suppression is required. This resolves the preceding
commit-readiness finding.

Query/source POS token reconciliation now calls the pinned exact unicode61
adapter directly, retaining document flags, repetitions and occurrence order.
It removes temporary FTS insert/read/delete work without changing the tokenizer.
A freshly built q4 index has identical annotations for all 6,096 blocks and
31,596 pair-lookup entries. All 1,470 q4 and 1,380 h2 complete responses match the
frozen pre-pass executable. Accepted scores remain q4 412/439 any, 354/439 first,
XS 31/42 and 25/42, empty 35/51, section 55/73; h2 341/354, 298/354, XS 56/62 and
49/62, empty 55/106, section 237/271. This is a performance/reliability change,
not another retrieval configuration. Local artifacts: `evals/output/latency-20261110/`.

### November 10 latency pass: reuse unchanged POS publication rows

Record/section mutation invalidates the complete POS publication but now deletes
only affected annotations and pair postings. Unchanged rows are reusable only
with matching record, body hash, annotation identity and every source-unit span.
Republish the generation under the writer guard without rebuilding unchanged
pair postings. Rollback, changed identity/span, secure deletion and retained-row
bytes are tested. No new table, startup scan or search algorithm is introduced.

In full q4 neutral evaluation, the five scratch-forget POS phases fell from
14.7–20.1 seconds each to 66.9–73.3 milliseconds. The candidate whole run took
85.101 seconds; the baseline command was not timed end to end, so those phase
figures do not establish a whole-run speedup. All non-latency neutral values and
statuses match, including existing G1/G4/Q1/R2 failures. Concurrent validation
work limits timing precision. A full rebuild still annotates original units;
this change avoids retagging surviving documents after each forget.

### November 10 latency pass: count blocks without expanding occurrences

For a normalized term that the exact tokenizer leaves unchanged, document
frequency now counts FTS MATCH rows directly against the same lifecycle/expiry
projection. Other terms retain literal vocabulary equality. This removes an
occurrence scan and temporary DISTINCT tree, with no frequency definition change.
A Unicode/lifecycle fixture compares both paths, including repeated terms,
compound tokens, candidate/closed records and an expiry boundary. q4 and h2
complete responses match the frozen executable; scale-q4 retains all 1,470
ordered pages and 388/439 any, 316/439 first, XS 28/42 and 22/42, empty 31/51,
section 51/73.

On the 189,767-block scale store, six warm per-connection counts for
`configuration` took approximately 3.50–3.61 ms through vocabulary DISTINCT and
2.49–2.59 ms through MATCH. Exclude the first cold run from that comparison.
This is one SQL-phase measurement, not a whole-query or whole-eval speedup.
`sql_read_profile --only document-count` retains the plans and timing probe.

### November 10 latency pass: retain one packed rare-form matrix

Rare-form projections previously loaded one matrix, copied every row to a
separate vector, discarded the matrix, then packed those vectors again for
scoring. Keep metadata beside one packed matrix instead. Scoring and
occurrence-weighted section means read the original rows directly, preserving
float32 bits, batch shape, order and arithmetic. Persisted format is unchanged.
q4 and h2 complete responses and all scale-q4 ordered pages remain identical to
the pre-pass executable; array and section-mean bit-conformance tests pass.
Formatting, strict workspace clippy, workspace tests, LLP references and the Nix
package build pass.

On scale-q4 query s1-c-0001, peak resident memory measured by the system time
utility fell from 1,403,518,976 to 1,067,745,280 bytes versus the count-optimized
control (about 320 MiB saved). A provisional paired sample of 87 fresh CLI
invocations per side had median 248.760 versus 231.972 ms and p95 635.330 versus
539.067 ms. Builds/evaluation were concurrent: retain these as provisional,
not isolated final timings. Complete CLI JSON and exit codes matched after
normalizing the clock-bearing cursor. Artifacts remain under the same ignored
latency output directory.

### November 10 latency pass: isolated CLI and full scale validation

The full scale-q4 neutral run with native tokenization and POS row reuse took
1,704.660 seconds (28.411 minutes), versus the recorded accepted run's 3,877
seconds (64.617 minutes): about 2.27× faster, not the desired tenfold improvement.
The new run overlapped package builds and parity checks; it is not a controlled
back-to-back whole-suite benchmark. All non-latency gate values/statuses match.
C2 was 138.930 ms versus recorded 123.920 ms; both fail the unchanged 50 ms gate.
Five forget POS publications took 1.936–2.136 seconds, reusing all surviving
blocks; initial scratch-build POS took 351.872 seconds. This whole-suite run
predates the subsequent block-count and packed-form optimizations.

After all heavy validation jobs stopped, alternate the frozen pre-pass CLI and
all four committed optimizations over the same 29 stratified q4 queries, five
rounds each (145 fresh processes per side), with separate identical store copies.
Times include invocation through exit, with warm filesystem caches. Complete JSON
and exit codes match after normalizing only clock-bearing cursors.

| Store | Before median / p95 ms | After median / p95 ms |
|---|---:|---:|
| q4, 1,376 records | 75.596 / 102.358 | 75.097 / 101.155 |
| scale-q4, 16,906 records | 222.811 / 519.239 | 220.248 / 510.879 |

Objective rating: **whole-suite improvement; fresh CLI nearly neutral**. These
isolated results supersede the larger provisional latency gains above. Deep
fallbacks still read the same full numerical populations. Diagnostic medians for
the optimized scale sample: store open 0.787 ms, scope/context 17.251 ms, NLP
resource deserialization when needed 36.427 ms, global block matrix when needed
237.342 ms. Conditional phase medians are not additive. Numerical arrays and
remaining write/publication work are still scaling costs; no claim of constant
query time, a neutral gate pass, or a tenfold speedup is warranted.

Full q4/h2 responses, scale-q4 pages, neutral tables, elapsed records, memory
measurements and paired CLI exports remain in `evals/output/latency-20261110/`.
Retrieval rules, thresholds, card order, corpus bytes and scorer rules are unchanged.

The final combined executable also completed full q4-dev neutral in 81.695
seconds with every non-latency gate value/status identical to the frozen baseline.
No full scale neutral timing is claimed for this final combined executable.

### 2026-11-10: skip provably unnecessary startup and vector work

- Git scope discovery combines remote enumeration and the selected fetch URL in
  one `git remote -v` call after establishing the worktree root. Ordinary remote
  configurations need two subprocesses instead of three. Unknown output falls
  back to the former lookup. Temporary-repository tests preserve upstream/origin
  precedence, URL rewrites, push/fetch differences, partial-clone annotations,
  multiple URLs, absent URLs, bare repositories and non-repository directories.
- Rare-form metadata loads separately from its exact packed float32 matrix.
  The matrix remains lazy when no rare-word block or no structurally qualifying
  form exists. When a form can survive, the original full population, numerical
  kernels, top-15 ordering and claimed-before-block behavior remain unchanged.
- Short-page rare neighbors exclude already-shown records before requesting the
  block matrix. An empty remainder proves there is nothing to append; a fixture
  checks both the returned card and absence of a matrix load.
- All 1,470 q4-dev and 1,380 h2-all complete responses are byte-identical to the
  accepted implementation. All 1,470 scale-q4 first pages match. Full q4-dev
  neutral preserves every non-latency gate, including existing failures.
- A separate feasibility probe retained the full matrix dimensions and selected
  row positions while zeroing unused rows. Selected float32 scores matched for
  eight generated queries at nine matrix sizes through 189,767 rows. This is
  not a product optimization or a corpus-vector parity result; candidate-only
  payload reads still require implementation, exact verification and timing.

Artifacts are in `evals/output/latency-round2-20261110/`. No retrieval rule,
threshold, card order or scorer definition changes in this pass.

After all build and scoring jobs stopped, paired fresh CLI processes used warm
filesystem caches and identical separate stores. The q4 sample contains 29
stratified queries over five rounds per executable; the broader scale sample
contains 114 queries over three rounds. The latter excludes and reports
`s3a-c-0018` and `s3a-c-0028`, whose prefixes are outside the product CLI grammar.
The initial expanded sampler aborted on an unsupported prefix; it was corrected
to report those exclusions rather than changing the product grammar.
Every timed response and exit code matches, normalizing only clock-bearing
cursors. Percentiles use the nearest, ties-even index at `0.95 * (n - 1)`.

| Store | Fresh invocations per side | Before median / p95 ms | After median / p95 ms |
|---|---:|---:|---:|
| q4 | 145 | 75.314 / 105.186 | 69.438 / 96.733 |
| scale-q4 | 342 | 226.183 / 666.329 | 215.190 / 657.292 |

Objective rating: **modest fresh-CLI improvement, not an order-of-magnitude gain**.
Median improvements are 7.8% and 4.9%. The broader scale sample is not directly
comparable to the preceding 29-query sample. Diagnostic conditional medians on
scale: store open 0.802 ms; Git scope/context 11.885 ms versus 17.445 ms before;
NLP deserialization 36.545 ms; block-matrix loading 240.274 ms when requested.
These phase medians are not additive. Block matrices were loaded for 12 of 114
diagnostic queries on both sides, so this sample does not demonstrate fewer
block-matrix loads from the shown-record guard. Full scale-neutral has not been
rerun for this pass. Formatting, strict workspace clippy, workspace tests,
packaged Nix build and LLP references passed. Product commits: `b83afbf` and
`5931aee`; sampler correction: `b15a216`.

### 2026-11-10: candidate-only rare-neighbor vector reads

Keep the rare-neighbor candidate predicate and full matrix dimensions unchanged.
Read only candidate embedding blobs by primary key, place them at their original
row positions, zero unused rows, run the original float32 kernel, and discard
unused outputs. An already-loaded full matrix is reused. Native `vectors_order`
supports counts and surviving-row positions; contiguous ids use subtraction and
gapped ids use SQL window positions. No parallel id map or partial full-matrix
cache is introduced. Allocation and arithmetic still scale with the global shape.

Contiguous/gapped corpus-free fixtures match full-matrix score bits. On the scale
store's 189,767 actual vectors, two candidate subsets (strides 37 and 101) and
four query vectors also match every selected float32 bit. The ignored diagnostic
requires an explicitly supplied populated copied store; normal tests need no
corpus. All 1,470 q4-dev and 1,380 h2-all full responses match the accepted page;
all 1,470 scale-q4 first pages match. All retrieval counts are unchanged.

Paired warm-filesystem fresh-process measurements alternate the previous pass's
binary with this candidate, using separate identical stores and checking JSON
and exit codes after clock-cursor normalization. There are 145 invocations per
side for 29 q4 queries and 342 per side for the same 114-query scale sample.

| Store | Before median / p95 ms | After median / p95 ms |
|---|---:|---:|
| q4 | 81.239 / 111.904 | 78.858 / 111.219 |
| scale-q4 | 224.957 / 650.549 | 214.219 / 656.804 |

The candidate-read path activates for these scale sample cases. These are
latency changes, not moved retrieval pages:

| Case | Vector rows read of 189,767 | Candidate score phase ms | Before / after invocation median ms |
|---|---:|---:|---:|
| s1b-c-0004 | 3,831 | 27.626 | 325.589 / 120.270 |
| s2b-c-0041 | 3,332 | 27.137 | 388.700 / 163.743 |
| s3b-c-0025 | 2,882 | 26.770 | 498.924 / 277.499 |
| s4-c-0044 | 1,859 | 24.728 | 375.579 / 142.110 |
| s4-c-0088 | 843 | 22.775 | 293.725 / 72.246 |
| s5-c-0081 | 6,900 | 34.213 | 361.654 / 148.635 |
| s6-c-0054 | 15,465 | 46.074 | 465.039 / 265.580 |

On q4, s1b-c-0004 reads 131 of 6,096 vectors in 1.023 ms; its invocation median
falls from 47.624 to 40.078 ms. Objective rating: **keep the exact read reduction;
substantial gains on the activated scale queries, modest aggregate gain, no p95
gain**. The broader scale p95 is slightly higher in this sample, and no tenfold
whole-suite improvement is claimed. Absolute timings drift between passes;
use each paired comparison rather than attributing cross-run drift to code.

The worst previously traced queries still load the global sentence matrix:
s5-c-0098 and s4-c-0074 read 932,056 sentence vectors in 1,091 and 1,079 ms,
respectively. This optimization does not remove that global top-15 competition.
Artifacts, bitwise checks and paired outputs are retained under
`evals/output/candidate-vectors-20261110/`.

Full q4-dev neutral also preserves every non-latency gate value and status;
existing failures remain failures. Formatting, strict workspace clippy, workspace
tests, the populated scale bitwise diagnostic, Nix build and LLP checks passed.
The full scale neutral suite has not been rerun for this candidate.

### 2026-11-10: precompiled NLP feature lookups

The compiled NLP asset now carries immutable linear-probe feature slots and
expanded float32 parameters instead of rebuilding three hash maps and expanding
bf16 parameters on every invocation. Original hashed keys, duplicate-key
replacement, empty features, parameter offsets and evaluation order are retained.
Outcome ids stay as bytes and widen only at array indexing. The wire decoder
still accepts the pinned original resource; the embedded format marker is version
2. No model, dictionary, linguistic rule or source annotation changes.

The first prototype stored expanded `usize` outcome ids: its binary was
109,120,928 bytes versus 94,045,744 before. It passed annotation fixtures and the
q4/h2/scale retrieval checks, but was superseded before latency sampling by byte
outcome ids. The final candidate is 102,367,568 bytes, an increase of 8,321,824
bytes (about 7.94 MiB) over the prior binary.

Both prototypes match all 24,736 complete annotations from q4 prepared queries,
22,668 saved source units and fixtures against the unchanged comparison engine.
The final run reports 14,514 ms in the focused engine and 61,340 ms in the older
engine; these mixed validation timings do not isolate this change from earlier
NLP optimizations. Deterministic asset serialization and collision/replacement
lookup fixtures pass. All 1,470 q4-dev and 1,380 h2-all complete responses and
all 1,470 scale-q4 first pages match. Full q4-dev neutral preserves every
non-latency gate value/status, including existing failures. Formatting, strict
workspace clippy, tests, final Nix build and LLP checks pass. The intermediate
Nix build was canceled when the compact outcome representation replaced it.

Artifacts are retained under `evals/output/nlp-features-20261110/`.

After heavy jobs stopped, the same paired warm-filesystem CLI sampler used 29
q4 queries over five rounds and 114 scale queries over three rounds per side.
All normalized JSON and exit codes match. Timed invocations exclude the separate
diagnostic round; only clock-bearing cursors are normalized.

| Store | Invocations per side | Before median / p95 ms | After median / p95 ms |
|---|---:|---:|---:|
| q4 | 145 | 69.181 / 95.441 | 61.178 / 85.652 |
| scale-q4 | 342 | 210.436 / 640.182 | 203.224 / 633.683 |

Conditional NLP-load medians fall from 36.387 to 27.897 ms on q4 (17 diagnostic
queries) and 36.340 to 27.949 ms on scale (69 diagnostic queries). Objective
rating: **keep; lower fresh-CLI median and p95 on both samples, at an additional
7.94 MiB in the native binary**. This is an 11.6% median improvement on q4 and
3.4% on scale, not a tenfold whole-suite result. Full scale-neutral has not been
rerun for this pass.

### 2026-11-10: lossless vector-storage opportunity measurement

The read-only Rust `vector_duplicates` diagnostic groups complete 512-component
float32 payloads by SHA-256; it does not change a store or search. The following
dictionary estimates include one four-byte ordinal for every original row.

| Population | Rows | SHA-256-distinct payloads | Largest repeat group | Current bytes | Dictionary + ordinals bytes |
|---|---:|---:|---:|---:|---:|
| Sentences | 932,056 | 274,807 | 2,836 | 1,908,850,688 | 566,532,960 |
| Forms | 79,980 | 79,919 | 62 | 163,799,040 | 163,994,032 |
| Beam | 64,863 | 64,767 | 16 | 132,839,424 | 132,902,268 |

Read-plus-hash times were 5,426, 502 and 386 ms respectively, with other validation
work running; these are not pure SQL timings. Objective rating: **sentence
dictionary storage merits implementation; forms and beam dictionary storage are
not adopted because both estimates are larger than their current payloads**.
Sentence duplicates comprise 70.516% of rows. No compression speedup is measured
yet. A future implementation must retain every original row, order and float32
value in the numerical population; duplicate payloads are not permission to
deduplicate top-15 candidates or change kernel dimensions.

The same scale store has 16,906 record headers but only 8,357 distinct body hashes
and 89,914,648 total original body bytes. Encoding batches and initial source POS
annotation currently repeat computation for identical text. This identifies a
write-time reuse opportunity, not a measured improvement or a new author field.

### 2026-11-10: reuse identical ingest computations

Encoding batches now compute byte-identical input text once and copy its exact
float32 vector into every original row position. POS publication similarly
reuses relative annotations for identical complete source units within that
publication. Each use still validates its own original body interval and writes
its own absolute base/end. Both maps are temporary; neither adds an index table,
startup scan, normalization, author field or content surrogate.

An isolated sequential fresh q4 build/scoring pair measured store creation at
14,034.961 ms before and 13,437.516 ms after. POS publication took 11,076.602 and
10,257.061 ms. The new publisher annotated 19,117 distinct units and reused
10,127, while retaining all 6,096 block rows. This is one before/after pair, not
a broad latency distribution. Resident invocation median/p95 were 5.902/15.456 ms
before and 5.931/15.803 ms after; no query-speed improvement is claimed.

The new read-only `ingest_parity` diagnostic verifies exact equality of all
6,096 block vectors, three fallback metadata records, five packed fallback-vector
rows and 6,096 POS payloads across those fresh stores. Model identities and byte
spans are compared; publication generations are excluded. All 1,470 complete
q4 responses match after reopening the new store. Repeated-vector tests compare
float32 bits, and repeated-unit fixtures cover distinct headings, Unicode and
original offsets. Objective rating: **modest verified q4 ingest improvement;
scale benefit remains to be measured**. Artifacts are under
`evals/output/ingest-reuse-20261110/`.

### 2026-11-10: lossless sentence dictionary and fresh-CLI measurement

The sentence projection now stores bit-identical vectors once, in first-occurrence
order, with an ordinal for every original sentence. It restores the complete
matrix before the unchanged dense scoring kernel. No candidate, repeated
occurrence, float32 bit, matrix dimension or top-fifteen ordering is removed.
Forms and beam retain ordinary packed storage: their earlier duplication
measurement did not justify dictionaries. Dictionary lookup compares complete
bit rows, including signed zero and NaN payloads, rather than trusting hashes.

Scale sentence-vector payload falls from 1,908,850,688 to 562,804,736 bytes, plus
the ordinal metadata. The database file need not shrink immediately: SQLite can
retain freed pages for reuse. Expansion temporarily holds both the dictionary
and the full scoring matrix; this reduces reads, not the algorithm's global
matrix population or its asymptotic scoring cost. The legacy scale migration
took 27,001.262 ms with concurrent validation/build work. It uses stored vectors,
does not load source bodies or re-encode, removes legacy chunks atomically, and
is not included in the steady-state CLI timings below.

Isolated alternating fresh-process measurements, with no other heavy job running:

| Corpus/sample | Before median/p95 ms | After median/p95 ms |
|---|---:|---:|
| q4, 29 queries x 5 rounds, 145 invocations per binary | 60.562 / 85.157 | 61.304 / 85.242 |
| scale-q4, 114 queries x 3 rounds, 342 invocations per binary | 207.449 / 656.020 | 201.794 / 630.558 |

The two sentence-fallback cases show the direct effect: s5-c-0098 median
2,437.304 -> 1,755.946 ms, and s4-c-0074 1,885.616 -> 1,242.109 ms. Diagnostic
sentence-payload reads fell respectively from 1,079.729 to 336.475 ms and
1,067.062 to 350.844 ms. Those phase timers exclude subsequent expansion; the
complete invocation figures include it. Other sampled cases fluctuate in both
directions; the largest observed per-case median increase was 16.797 ms on
s5-c-0002. Do not interpret the two large wins as a universal query speedup.
The sampler retained its deterministic category selection and separately
reported unsupported CLI-prefix exclusions s3a-c-0018 and s3a-c-0028.

All paired CLI responses match. Exact expanded-index comparison passes for q4,
h2 and scale-q4, including all block vectors, sentence vectors/block mappings,
form/beam arrays and POS payloads. All 1,470 q4 and 1,380 h2 complete responses
match; all 1,470 scale first pages match. Scores stay q4 412/439 any, 354/439
first, cross-scope 31/42 and 25/42, empty 35/51, section 55/73; h2 341/354,
298/354, 56/62 and 49/62, 55/106, 237/271; scale-q4 388/439, 316/439, 28/42
and 22/42, 31/51, 51/73. Every q4 neutral non-latency gate is unchanged,
including existing failures. Formatting, strict workspace clippy, workspace
tests, LLP references and packaged Nix checks pass. Objective rating: **large
verified savings on sentence-heavy scale queries, modest aggregate scale gain,
and essentially flat q4 latency; no retrieval change**. Full combined scale
neutral timing is a separate measurement. Artifacts are under
`evals/output/sentence-dictionary-20261110/`.

### 2026-11-10: exact scale ingest reuse and consuming matrix packing

A fresh scale-q4 store with identical-text reuse and consuming matrix packing
prepared 16,906 records and 189,767 blocks in 189,338.807 ms. POS publication
took 108,556.534 ms: 215,944 units were annotated and 854,752 reused exact prior
annotations within that publication. An earlier scale rebuild measured
351,872 ms for POS publication; these are separate runs with different background
load, not an isolated paired speedup. Consuming matrix packing releases each
individual vector as its bits enter the packed allocation, instead of retaining
the entire source array until the copy finishes.

The fresh store matches every byte of all 189,767 block vectors and POS payloads,
both form/beam manifests and their nine packed chunks, and all 932,056 sentence
vectors and block ids in the accepted store. All 1,470 ordered scale pages match:
any 388/439, first 316/439, cross-scope 28/42 and 22/42, empty 31/51, section 51/73.
Query timing overlapped later compilation and is not an isolated latency result.
Objective rating: **scale ingest reuse validated exactly; substantial POS-work
reduction, with no changed scored page**. Artifacts remain under
`evals/output/ingest-reuse-20261110/`.

### 2026-11-10: combined full scale neutral timing

The frozen sentence-dictionary binary completed the entire scale-q4 dev neutral
protocol in 1,292.188 seconds (21.536 minutes), including independent rebuild,
2,061 self-retrieval probes and five scratch forget probes. It used an already
upgraded store; one-time migration is excluded. No other heavy job ran during
this measurement. The historical accepted run took 3,877 seconds; the first
performance pass took 1,704.660 seconds. Those earlier runs overlapped other work,
so the approximately 3.0x historical and 1.32x intermediate ratios are contextual
end-to-end comparisons, not isolated paired benchmarks. The roughly 10x objective
has not been achieved.

Every non-latency gate exactly matches the prior complete scale table. C2 p95
is 123.910 ms versus 138.930 ms in the intermediate run; both fail the unchanged
50 ms bar. Existing G1, G4, Q1 and R2 failures remain. No ranker adjustment follows
from those failures. Scratch POS publication took 108,372.824 ms initially and
2,230.684, 2,224.900, 2,220.162, 2,233.558 and 2,238.113 ms after the five forgets;
all latter annotations were reused. Other forget phases still consume substantial
time and require separate profiling rather than attribution to POS. Objective
rating: **complete unchanged gate outcomes with a faster full run; substantial
remaining scaling work, not a tenfold success**. Full stdout, stderr and elapsed
time are in `evals/output/sentence-dictionary-20261110/scale-neutral.*`.

### 2026-11-10: indexed hint and related-pointer preselection

Lazy stores now select word hints and related pointers through the existing
term, section-word, section and record-eligibility indexes. SQL applies the fixed
hint clock, exclusions, coverage and original tie breaks before returning only
three hint sections or two related records. This avoids loading every candidate
record's complete section list. Content words remain deduplicated by the existing
query function. Related selection takes the best section per record before its
global limit. No table, index, author field or retrieval rule was added. Native
query-plan tests show indexed term/posting/section/record seeks; grouping and
sorting still process the matching postings, so this is not constant-time search.

Isolated alternating fresh-process measurements:

| Corpus/sample | Before median/p95 ms | After median/p95 ms |
|---|---:|---:|
| q4, 29 queries x 5 rounds, 145 invocations per binary | 60.762 / 86.080 | 60.058 / 83.997 |
| scale-q4, 114 queries x 3 rounds, 342 invocations per binary | 211.682 / 655.130 | 193.183 / 590.816 |

The largest scale median savings include s6-c-0119 216.534 -> 151.842 ms,
s4-c-0113 624.133 -> 562.082 ms, s5-c-0149 580.122 -> 519.848 ms,
s3a-c-0056 539.491 -> 480.194 ms and s3b-c-0015 356.758 -> 297.478 ms.
The largest scale median increase was 5.464 ms on s3b-c-0033. These are paired
samples; do not compare their absolute baseline with another run's wall time.
The sampler still reports unsupported CLI-prefix exclusions separately.

All paired CLI outputs and all 1,470 q4, 1,380 h2 and 1,470 scale-q4 complete
responses are byte-identical. Scores remain the accepted values recorded above.
Every q4 neutral non-latency gate is unchanged. Workspace tests, strict clippy,
formatting, LLP references, native query-plan checks and packaged Nix checks pass.
The eval's new `--responses` export avoids verbose ranking traces and reproduces
the previously saved full calls exactly; it changes neither queries nor scoring.
Objective rating: **verified aggregate scale CLI improvement with unchanged
responses; small q4 gain**. Artifacts are under `evals/output/sql-pointers-20261110/`.

### 2026-11-10: scratch forget phase profile

A temporary timing build forgot s1-r-0001 (one block) from a copied scale-q4
store, in a fresh CLI process with other heavy jobs stopped. Command execution
and output took 82,399.415 ms. The source timing edits were removed; the copied
store reports integrity `ok`. This is one write-path profile, not a search
latency benchmark or an optimization result.

| Forget phase | Milliseconds |
|---|---:|
| Materialize structure | 2,028.313 |
| Delete record and derived rows | 13,477.114 |
| FTS optimize and commit | 1,732.817 |
| Materialize block words | 2,183.338 |
| Statistics | 1,702.100 |
| Navigation | 5,696.119 |
| Persist structural indexes | 30,382.190 |
| Prepare fallback artifacts | 18,648.979 |
| Publish POS | 6,528.554 |
| Publish catalog | 16.985 |

POS reused all 189,766 surviving block annotations; its phase differs from the
resident neutral probes and must not be compared as the same workload. Structural
rewriting and fallback publication dominate, followed by deletion cleanup.
The secure-delete optimize step is only 1.733 seconds here and remains required
by the pinned engine's integrity behavior. Objective rating: **write-path
bottlenecks isolated; no further write optimization claimed**. Logs, the temporary
instrumentation patch and executable remain under `evals/output/sql-pointers-20261110/`.

### 2026-11-10: matcher result reuse and shared coverage — not adopted

Tested sharing immutable matcher hit sets, counts, scores and encounter positions
with coverage, and retaining the latest matcher pair per query/caller scope for
reuse at the same clock. Writes invalidated reuse; first-clock coverage and
clock-keyed ranking history stayed unchanged. This added no persistent index.

The first implementation preserved all aggregate scores but changed the original
wording of s1b-c-0018 on scale-q4: cards x-r-12539 and x-r-03372 disappeared.
Matcher B also restores query-local heading trees, and a reuse hit had skipped
that side effect after another query. Restoring the same indexed tree lookup
fixed the difference. The corrected candidate matched all 1,470 q4, 1,380 h2 and
1,470 scale-q4 complete responses byte for byte. Scores remained q4 412/439,
354/439, 31/42, 25/42, 35/51, 55/73; h2 341/354, 298/354, 56/62, 49/62,
55/106, 237/271; scale-q4 388/439, 316/439, 28/42, 22/42, 31/51, 51/73.
Reopened-store tests covered alternating queries, clocks, scopes and write
invalidation. Workspace tests, strict clippy, Nix packaging and LLP checks passed.

Isolated full q4 neutral took 84.059 -> 78.026 seconds; every non-latency gate
was identical, and C2 p95 was 7.980 -> 7.330 ms. Alternating fresh CLI samples:

| Corpus/sample | Before median/p95 ms | Candidate median/p95 ms |
|---|---:|---:|
| q4, 29 queries x 5 rounds, 145 per binary | 60.660 / 84.284 | 59.229 / 86.386 |
| scale-q4, 114 queries x 3 rounds, 342 per binary | 189.629 / 589.951 | 192.068 / 605.472 |

All paired CLI outputs matched. The roughly 7% neutral wall-time gain did not
establish a fresh-CLI improvement: scale median and p95 were slightly higher,
and q4 was mixed. **Not adopted** for this CLI-latency goal; the extra reuse
state and query-local restoration obligation were not justified by these results.
Product and LLP changes were removed. No full scale-neutral run of this candidate
was performed. Objective rating: **repeated-query benefit, no demonstrated
end-to-end CLI gain**. Patch, binaries, full responses and timing samples remain
under `evals/output/matcher-reuse-20261110/`.

### 2026-11-10: bounded write-time pager cache — adopted

The earlier read-scan cache trial did not test bulk index writing. On an APFS
copy of the scale-q4 store, forgetting s1-r-0001 with a temporary 32 MiB native
pager budget was measured against the unchanged default budget. Both retained
secure deletion, FTS optimize, transaction boundaries and all publication work.
Trials ran in before/after/after/before order, without other heavy project jobs.

| Trial | Whole CLI seconds | Structural publication seconds |
|---|---:|---:|
| Default budget | 86.304 | 29.960 |
| 32 MiB diagnostic | 64.690 | 13.108 |
| 32 MiB diagnostic repeat | 63.704 | 13.116 |
| Default budget repeat | 81.823 | 30.695 |

The production implementation scopes `cache_size=-32768` to rebuild and forget,
restores the prior connection value after success or error, and requests unused
pager memory release. It changes no read-path setting, schema, index contents,
durability setting, model or retrieval algorithm. A final production CLI probe,
including restoration, took 67.590 seconds (command/output phase 66.705 seconds).
Every scratch probe passed integrity; scratch copies were removed afterward.
This is roughly a one-fifth whole-forget improvement over the measured default
trials, not a fresh-query latency improvement.

Tests inspect the active budget inside SQL mutation triggers and verify exact
restoration after successful rebuild/forget and injected failures. A fresh q4
build matched all 6,096 block vectors/POS annotations, form/beam metadata and
arrays, and 35,690 sentence vectors. All 1,470 q4, 1,380 h2 and 1,470 scale-q4
complete responses matched the accepted implementation byte for byte, with
unchanged scores. All q4 neutral non-latency gates, including existing failures,
were unchanged; the full run took 83.619 seconds with C2 p95 7.940 ms, essentially
flat against the preceding default-budget run at 84.059 seconds and 7.980 ms.
Workspace tests, strict clippy, formatting, LLP references and Nix packaging pass.
Objective rating: **verified scale write improvement; no cold-query gain claimed**.
Artifacts are under `evals/output/write-pager-20261110/`.

This latency pass ended at the planned stopping point:
matcher reuse did not justify adoption, and this bounded write improvement is
validated. The latest complete scaled neutral measurement remains 1,292.188
seconds, before this final write-cache change; no newer full-scale total or
tenfold improvement is claimed. The remaining large costs are global numerical
populations and broad structural/fallback regeneration. Further substantial gains
require a separately scoped structural change, rather than another small tuning
pass. No further experiment or full scale run was started at this stopping point.


## 2026-11-10 — Caller-clock lifecycle correction

- The lifecycle measurement covered consistent enforcement and skipped a writer option, and
  considered removing author attribution. New CLI records omit writer; existing immutable
  attribution remains. Added practical agent-use advice without a verification service.
- Apply the caller clock to hints, related pointers, filename uniqueness, heading trees,
  record fills and rare neighbors. Clear clock-dependent query caches when the clock
  changes. Retain historical trees at ingestion; guard every returned card and pointer.
  Ranking arithmetic, thresholds, model and scope penalties stay unchanged.
- q4 dev: 412/439 any, 354/439 first, cross-scope 31/42 and 25/42, no-gold
  35/51, section 55/73; all counts unchanged. Thirty wording responses change across
  26 cases. Ineligible card occurrences fall from 32 to zero.
- h2 all: 341/354 any, 298/354 first, cross-scope 55/62 and 48/62, no-gold
  54/106, section 237/271. Cross-scope-any, cross-scope-first and no-gold each lose
  one: `h2a1-c-0102` previously credited closed/superseded `s2b-r-0032`.
  Twenty-eight wording responses change across 25 cases; ineligible card occurrences
  fall from 28 to zero. `h2a3-c-0082` also loses the expired/closed nongold
  `s5-r-0236` without a scored metric change. No retuning.
- q4 changed wordings (zero-based): `s1b-c-0034:2`, `s1b-c-0041:0`, `s1b-c-0041:2`, `s1b-c-0065:1`, `s2b-c-0032:2`, `s3a-c-0053:0`, `s3a-c-0053:2`, `s3b-c-0013:0`, `s3b-c-0026:0`, `s3b-c-0034:0`, `s3b-c-0046:1`, `s4-c-0080:1`, `s4-c-0084:0`, `s5-c-0084:0`, `s5-c-0085:0`, `s5-c-0124:0`, `s5-c-0128:0`, `s5-c-0128:1`, `s5-c-0130:0`, `s5-c-0151:0`, `s6-c-0020:0`, `s6-c-0022:0`, `s6-c-0028:0`, `s6-c-0030:1`, `s6-c-0041:0`, `s6-c-0049:0`, `s6-c-0055:2`, `s6-c-0083:1`, `s6-c-0083:2`, `s6-c-0119:0`.
- h2 changed wordings (zero-based): `h2a1-c-0049:0`, `h2a1-c-0050:1`, `h2a1-c-0068:1`, `h2a1-c-0100:1`, `h2a1-c-0101:0`, `h2a1-c-0102:0`, `h2a1-c-0106:0`, `h2a1-c-0106:2`, `h2a2-c-0005:1`, `h2a2-c-0031:2`, `h2a2-c-0040:0`, `h2a2-c-0077:0`, `h2a2-c-0091:0`, `h2a2-c-0108:0`, `h2a3-c-0036:0`, `h2a3-c-0045:0`, `h2a3-c-0045:1`, `h2a3-c-0062:1`, `h2a3-c-0073:0`, `h2a3-c-0082:0`, `s7-c-0019:0`, `s7-c-0058:0`, `s7-c-0064:0`, `s7-c-0068:1`, `s7-c-0104:1`, `s7-c-0126:0`, `s7-c-0126:1`, `s7-c-0148:1`.

- Full q4 dev neutral loop: G1 0 violations (previously 26), G2 0.014,
  G3/G5 1.0, Q1 0.686, Q2 0.985, Q3 0.980, Q4 0.822, Q5 1.0,
  Q6 0.945, C1 median 176/p95 287, C2 p95 9.060 ms, R1 0.987,
  R2 0.229. G4 remains failed. Q1 and R2 remain failed; this correction does
  not claim to resolve them. Latency was sampled during validation activity,
  not as a controlled performance comparison.


## 2026-11-11 — Configured Git email attribution

The writer attribution policy was updated: CLI file, stdin and
close records now capture effective Git `user.email`, with null for unavailable
or empty configuration. Email is unverified metadata, never retrieval evidence.
A nullable generated SQL column exposes the immutable envelope field without
duplicating it. Existing attribution and missing legacy fields remain unchanged;
no device identity or backfill is introduced.

## Packed NLP model loading (2026-11-12)

Performance-only change to the embedded NLP asset, version 3: pack chunker
feature slots, outcome ids and float32 parameters as byte arrays, decoding
explicit little-endian values without changing annotation or scoring.
`examples/nlp_load.rs` measures only `Tokenizer::from_compiled`, excluding
annotation, process startup and destruction. Release builds on aarch64-darwin;
30 alternating fresh processes per version, with filesystem caches warm:
unchanged checkout median 29.2275 ms, packed median 26.795 ms (8.3% lower).
Ranges were 28.642–84.730 ms and 26.418–80.813 ms respectively. These are
fresh-process measurements, not cold filesystem measurements. The historical
117 ms probe is not a matched baseline for the current checkout.
An initial 15-process sample during concurrent builds was noisy and slower
(28.709 ms baseline median, 32.971 ms packed); a later 25-process packed sample
had median 29.939 ms. Use the alternating comparison for this decision.
Workspace tests include differential token/tag/chunk/span parity against the
unchanged registry engine. No retrieval tuning or holdout scoring was run.

## 2026-11-13 — Detached runtime and application boundary

Implemented LLP 0007 on Unix without changing retrieval arithmetic, thresholds,
models, indexed evidence or original-byte ingestion. Application operations and
text/JSON rendering are separate; the CLI automatically connects to a detached
same-binary owner. Store handles and immutable model resources are reused.
SSH coordination stays outside the dispatcher, while local replica operations
and remote stdio requests use that dispatcher. No automatic background sync.

Native aarch64-darwin validation through the flake passed formatting, workspace
clippy with all targets/features and warnings denied, workspace tests with all
features, Nix package build/tests and packaged release smoke. The release smoke
also passed with the pre-change 4dc9516 executable as the compatible stdio peer.
Runtime tests cover eight concurrent cold clients, XDG-root changes, symlink
aliases, detached parent-pipe closure, initiator exit, stop/restart, forced crash/restart,
non-socket preservation, build mismatch, per-request diagnostics, external forget,
direct/IPC page parity and simultaneous two-way peer pulls. Protocol unit tests
cover oversized/truncated frames and values larger than a single frame. The
existing optional candidate_vector_corpus_bits test passed on a copied store;
that is arithmetic evidence, not a neutral retrieval gate.

### Closed small-store smoke measurement

Baseline: release build from 4dc9516. Candidate build identity:
`31040666e876a0594d5ec71cba7b2f74a6002b460706e21a0da018bbd6418287`.
This measurement predates the final bootstrap-only readiness/error hardening;
that subsequent change preserves the healthy execution path and retrieval.
Used a consistent online backup of the account's existing store: 22 records,
20,246,528 bytes. No generated surrogate documents were indexed for the search
comparison. Ten varied queries, including an identifier miss and queries reaching
POS/fallback paths, ran in three sequential baseline/candidate rounds. Thirty
full first-page responses matched, including ordered IDs, card text, hints,
related pointers and pagination context. Only captured UTC clock fields were
normalized in cursors. Twelve pages had cards, exposing eleven distinct records.
Four concurrent clients also preserved the expected ordered IDs. List and outline
JSON matched the baseline. Private store copies, ordered IDs, timing samples and diagnostics remain outside
the repository; full responses were compared in memory, not retained as a corpus.
None of these private artifacts is committed.

| Invocation through exit | Median ms | Observed p95 ms |
|---|---:|---:|
| Fresh baseline CLI | 42.064 | 62.452 |
| CLI using resident owner | 24.716 | 31.644 |

The sample includes the first search (51.229 ms), not just repeated hot queries.
A cold daemon-starting list took 132.700 ms end-to-end, with 6.164 ms separately
reported store opening and 6.834 ms application execution. Separate warm diagnostic
probes had application-operation median 3.161 ms and maximum 5.621 ms, excluding
CLI startup, Git scope discovery, transport and rendering. These scopes are not
interchangeable. RSS snapshots were 6,592 KiB after metadata-only startup and
180,080 KiB after the query mix; these are snapshots, not peak-memory bounds.

Earlier completed implementation probes, before final buffered JSON and build
identity hardening, measured baseline/resident medians 39.206/23.564 ms and
39.204/25.701 ms with the same query sequence and accepted algorithm. An initial
harness attempt stopped when a readback assumed an empty query had a first card;
its timings were discarded, and readback selection was fixed without changing
queries or retrieval. No ranker configuration was selected from these timings.

A separate external-write probe ingested the repository's original README through
the baseline into a disposable copy. The following resident search reopened in
0.422 ms and took 31.270 ms end-to-end (7.647 ms application execution), preserving
the baseline JSON response. This probe preceded final JSON buffering and build-ID
hardening; it demonstrates invalidation, not an independently controlled latency
comparison. The captured originals and float-bit check remained intact.

OS caches were warm from backup/preceding work, and builds/tests shared the host.
These measurements are a small operational smoke test, not the curated/scale dev
page, a cold-disk guarantee, a controlled throughput study, an h2 confirmation or
a neutral gate table. Curated/scale corpora were absent from this checkout and the
shared checkout, so those retrieval runs were not performed. No closed retrieval
study was reopened, and no holdout was used for tuning. Native Linux and Windows
execution were not available in this run; Windows remains a separate delivery.


The subsequent thin-client correction moved missing-database creation, import
validation, default label derivation and SSH coordination behind daemon requests.
The timing table above predates that correction and bootstrap readiness hardening;
it is not a fresh measurement of the resulting branch or its subsequent
dispatcher-thread and admission changes. Regression checks cover
readback with a resident handle while client file opens are denied, invalid
imports leaving a missing database absent, SSH subprocess parent PID matching
the authenticated owner, and stop draining network workers plus internal replica
steps. These checks establish process ownership and liveness, not new retrieval
measurements or full neutral validation.


## 2026-11-13: Daemon versus pre-daemon CLI wall latency before descriptor cleanup optimization

Measured the current branch after the dispatcher-thread, admission and response
barrier corrections against pre-daemon commit `4dc9516dc9804efb342e00af43de6ff309df420c`.
Both executables were built with the same flake devshell and release profile,
using `rustc 1.97.1 (8bab26f4f 2026-07-14) (built from a source tarball)`. Candidate build identity:
`f147da108584e4332a2e4bbdd863704da4c663a9bdd8a46d9966bcbb7ece21a0`. Machine: Apple M3 Ultra, macOS-27.0.1-arm64-arm-64bit-Mach-O.

Used separate byte-identical database copies from one read-only online backup
of the account store: 24 original records, 20,537,344 bytes.
The ten query strings are the unchanged mixed hit/miss selection from the earlier
operational probe, listed below. No surrogate documents, tuning, gold scoring
or holdout runs were introduced. This remains a small real-store sample, not
a representative curated/scale benchmark.

Timed ordinary default-text `linn --store <copy> search <query>` from process
creation through exit, including output capture, Git caller context, transport
and rendering. Each invocation executes the binary directly; Nix shell entry
is outside the timer. Both variants used the same repository cwd and caller
environment with diagnostic flags removed.

| Condition | Pairs | Before median / p95 / max, ms | Current median / p95 / max, ms |
|---|---:|---:|---:|
| Resident, repeated queries | 100 | 41.411 / 64.126 / 74.876 | 24.289 / 29.424 / 31.795 |
| Resident, first pass | 10 | 41.679 / 73.123 / 73.123 | 26.181 / 66.770 / 66.770 |
| First search starts daemon | 10 | 40.495 / 71.387 / 71.387 | 128.246 / 162.030 / 162.030 |

For cold searches, stop the owner before each query. For the resident first pass,
start a fresh daemon with a metadata-only list, then measure each query once;
this includes first-use model/query work. Metadata-only startup separately took
99.248 ms. Its diagnostics separately reported store opening and
application execution. The first resident search took 42.667 ms.
Repeated queries followed that measured pass: ten rounds of the same ten
queries, randomized with seed 34709. Alternate baseline/candidate execution order
across pairs. Report every sample without filtering outliers. p95 uses the nearest
index with ties-to-even; with ten samples this selects the maximum. File and code
caches were warm from backup, copies, builds and preceding runs: cold here means
fresh daemon/application resources, not cold disk. No parallel build or test was
running during the timed search loop.

Repeated-query per-query medians, ten samples per variant:

| Query | Before, ms | Current, ms |
|---|---:|---:|
| `git send` | 41.265 | 24.180 |
| `publication marker` | 43.811 | 23.701 |
| `XDG_RUNTIME_DIR` | 24.023 | 21.471 |
| `native routing` | 41.660 | 24.889 |
| `daemon double fork` | 34.726 | 21.971 |
| `Metro transform cache` | 64.578 | 27.633 |
| `configured devshell` | 43.407 | 26.084 |
| `Swift async actor` | 44.329 | 26.253 |
| `worktree remote branch` | 35.563 | 22.520 |
| `nanometerquux42` | 40.109 | 23.810 |

All 120 timed default-text response pairs matched after normalizing only cursor
clocks, with identical exit codes. Ten additional complete JSON response pairs
also matched ordered cards, hints, related pointers and pagination context after
normalizing cursor clocks. Four queries returned cards, covering twelve distinct
records; an honest miss remains a valid invocation. Original bodies and envelopes
were fingerprinted unchanged in both copies after the run. Raw individual timings,
ordered IDs, diagnostics and private store copies remain outside the repository.

This run shows lower resident invocation latency and a higher first-daemon-search
cost on this sample. It does not measure concurrent throughput, establish a
production p95, replace dev/holdout page validation or claim a neutral gate.


## 2026-11-13: Native macOS descriptor cleanup and final CLI latency

The preceding first-start cost exposed an avoidable macOS bootstrap sweep over
RLIMIT_NOFILE. This host's soft limit is 1,048,575. An isolated C probe measured
82.778 ms closing every number from 3 to that limit, versus 0.005 ms listing and
closing actual descriptors with libproc. This probe isolates descriptor cleanup;
it is not the complete application startup timer.

The final implementation enumerates actual macOS descriptors before daemonizing
and before threads are created. It validates the complete list and reports
startup failure on errors; the macOS descriptor-limit fallback is removed.
The pinned source review and differences from zmx's limited-range cleanup are
recorded in src/runtime/README.md; LLP 0007 records the adopted decision.

Final release build identity: `df8a6cbbb421e8c79788a1893523425d3f8565cbdc8e0383edc0f285f1b41a75`. Same toolchain, machine,
cwd, 24-record snapshot (20,537,344 bytes), ten query strings and timing/parity
method as the preceding run. The immutable backup was reused exactly rather than
recapturing a possibly changed live store. Builds and tests finished before the
measurement loops. All 120 paired text responses and ten full JSON response
pairs matched, normalizing cursor clocks only, with equal exit codes. Original
body/envelope fingerprints remained unchanged in both store copies.

| Condition | Pairs | Before daemon: median / p95 / max, ms | Final daemon: median / p95 / max, ms |
|---|---:|---:|---:|
| Resident, repeated queries | 100 | 37.021 / 57.133 / 58.435 | 21.486 / 24.800 / 26.369 |
| Resident, first pass | 10 | 36.705 / 58.133 / 58.133 | 22.490 / 54.627 / 54.627 |
| First search starts daemon | 10 | 38.942 / 71.141 / 71.141 | 42.324 / 76.440 / 76.440 |

Metadata-only daemon startup separately took 12.476 ms,
compared with 99.248 ms in the earlier run. Repeated-query median latency is
42.0% lower than the pre-daemon baseline in this final run. File/code caches
remain warm; first startup means fresh daemon/application resources.

A separate, directly paired startup comparison alternated the saved pre-fix
daemon executable with the final executable on identical snapshot copies. Both
owners were stopped before every search; the ten queries ran twice (20 pairs).
All text responses and exit codes matched after cursor-clock normalization.

| Cold search | Median, ms | Observed p95, ms | Maximum, ms |
|---|---:|---:|---:|
| Daemon before native cleanup | 125.242 | 148.554 | 191.038 |
| Daemon with native cleanup | 40.608 | 62.391 | 66.950 |

Final repeated-query medians, ten samples per query per variant:

| Query | Before daemon, ms | Final daemon, ms |
|---|---:|---:|
| `git send` | 36.275 | 21.488 |
| `publication marker` | 37.998 | 21.167 |
| `XDG_RUNTIME_DIR` | 20.947 | 18.507 |
| `native routing` | 36.878 | 21.602 |
| `daemon double fork` | 30.734 | 19.714 |
| `Metro transform cache` | 57.168 | 23.866 |
| `configured devshell` | 38.792 | 22.985 |
| `Swift async actor` | 39.272 | 22.913 |
| `worktree remote branch` | 31.475 | 19.969 |
| `nanometerquux42` | 36.120 | 21.307 |

The first resident pass is measured, not a hidden warmup; repeated results
reuse warmed query/model state. Reported p95 uses nearest-index ties-to-even
and is the maximum for the ten-sample groups. The separate paired comparison
helps distinguish the startup fix from host drift between the two full runs.
Individual raw samples and private stores remain outside the repository. These
are operational latency diagnostics, not concurrent throughput, a production
latency distribution, full curated/scale parity, gold scoring or a neutral gate.


A separate inherited-pipe probe placed a non-close-on-exec descriptor at 4096,
then reduced the child's soft descriptor limit to 128. The saved pre-fix daemon
kept the caller's pipe open after the CLI exited; the final daemon closed it
while remaining alive. The integration test covers this case. This also shows
why a soft-limit sweep is not a reliable substitute for enumerating open entries.


## 2026-11-13: File-based Git scope and runtime simplification

Compared the saved daemon build `df8a6cbbb421e8c79788a1893523425d3f8565cbdc8e0383edc0f285f1b41a75` with the final git2
implementation `19a5646dd274723faa464350b32d78a7ddc97fe13a8769a9a75150db95b08365`. This is an incremental comparison after
the native descriptor-cleanup fix, not another pre-daemon comparison. Both use
the same toolchain and release settings as the preceding runs.

The final CLI reads caller scope through git2 0.21.0/libgit2 1.9.7 file
configuration, without Git subprocesses or a command fallback. Network features
are disabled. Per explicit decision, injected GIT_CONFIG_COUNT/PARAMETERS remote
configuration is outside caller-scope support. Regression cases preserve legacy
headers, URL rewrites, first effective fetch URL, URL-less remote precedence,
global/includes/worktree config, canonical local roots and quoted metadata.
A fake Git executable confirms search does not execute Git for scope; pagination
uses cursor-pinned scope/clock and omits the discarded caller lookup entirely.
Writer attribution and filter-aware original-file evidence keep their existing
Git operations. LLP 0004 records the adopted boundary.

The direct-read investigation measured repository discovery plus config-byte IO
at about 0.064 ms, excluding a complete semantic resolver. A minimal gix 0.90
remote-API probe handled normal discovery in about 0.3 ms internally, but differed
on legacy/quoted metadata. Its config-based version covered those cases. The
git2 probe handled them directly but omitted command-injected remotes, the
explicitly accepted exception. The probe packages remain outside the product
workspace; only git2 was adopted as a product dependency. These component timings are not
end-to-end search measurements.

The runtime now keeps application/ownership state in its dispatcher, validates
the known bound socket path directly instead of reparsing discovery JSON per
operation, starts sync workers directly in JoinSet blocking tasks, and drops the
extra store LocalSet and unused public replica-over-IPC adapter. Bilateral sync
tests use whole Sync operations. Across the three runtime implementation modules,
source lines decreased from 1,202 to 1,043 (159 fewer); this count excludes the
new integration-test file and dependency sources. The CLI paths module decreased
from 304 to 294 lines including its tests.

Used the unchanged 24-record, 20,537,344-byte original snapshot from the previous
runs and the same ten query strings. Separate identical store copies, actual
repository cwd/caller context, default-text output, direct executable launch
through process exit, alternating pair order, fixed shuffle seed 34709, warm
OS caches and no concurrent build/test load. The measured first resident pass
includes first-use work; repeated queries follow it. No outliers were removed.

| Condition | Pairs | Previous daemon: median / p95 / max, ms | Final: median / p95 / max, ms |
|---|---:|---:|---:|
| Resident repeated search | 100 | 20.723 / 23.169 / 24.010 | 9.752 / 12.371 / 14.544 |
| Resident first pass | 10 | 22.600 / 55.088 / 55.088 | 11.041 / 42.665 / 42.665 |
| First search starts daemon | 10 | 41.526 / 78.740 / 78.740 | 30.119 / 64.049 / 64.049 |
| Cursor replay | 20 | 23.183 / 24.477 / 24.876 | 11.233 / 11.414 / 11.601 |

The repeated-search median is 52.9% lower in these paired samples. Metadata-only
startup was 11.818/12.170 ms before/after, measured separately.
The release executable grew from 102,669,120 to 103,153,408 bytes
(+484,288 bytes, about 473 KiB).

Cursor replay used the first cursor-bearing query in the fixed selection
(`Metro transform cache`), with one identical pinned cursor for both variants.
Twenty default-text replay pairs and an additional exact JSON replay pair
matched. All 120 search text pairs and ten full search JSON pairs matched after
normalizing only cursor clocks, with equal exit codes. Original body/envelope
fingerprints remained unchanged in both copies. Individual timings, ordered IDs,
probe fixtures and copied stores remain outside the repository.

Final repeated-query medians, ten samples per query per variant:

| Query | Previous daemon, ms | Final, ms |
|---|---:|---:|
| `git send` | 20.737 | 9.777 |
| `publication marker` | 20.824 | 9.745 |
| `XDG_RUNTIME_DIR` | 18.663 | 7.771 |
| `native routing` | 20.430 | 9.772 |
| `daemon double fork` | 19.198 | 8.362 |
| `Metro transform cache` | 23.134 | 12.197 |
| `configured devshell` | 22.476 | 11.542 |
| `Swift async actor` | 21.661 | 11.052 |
| `worktree remote branch` | 20.323 | 9.065 |
| `nanometerquux42` | 20.445 | 9.427 |

p95 uses nearest-index ties-to-even; the ten-sample groups therefore report the
maximum as p95. This remains a small, single-client operational latency sample,
not concurrent throughput, a production distribution, a curated/scale retrieval
validation, a neutral gate or holdout scoring. Retrieval algorithms, thresholds,
models and original bytes were not changed or retuned.

## 2026-11-13: End-to-end search with streamed local JSON

Compared the release build before this review (after body-SHA-256 source refs)
with the final release build. Both are Nix packages built with the same pinned
toolchain, release settings and embedded assets. Native Apple M3 Ultra/macOS
27.0.1; the same frozen 24-record, 20,537,344-byte store and ten query strings
as the previous measurements. Separate byte-identical copies, repository cwd
and caller scope, warm OS caches, no concurrent build/test load, alternating
pair order, ten shuffled resident rounds with seed 34709, and no outlier removal.
Timers include direct executable launch through exit, scope lookup, IPC and
default-text rendering; Nix shell entry is outside the timer. Cold means a new
daemon/application, not a cold disk or fresh machine.

| Condition | Pairs | Before: median / p95 / max, ms | Final: median / p95 / max, ms |
|---|---:|---:|---:|
| Resident repeated search | 100 | 9.362 / 11.659 / 12.592 | 9.123 / 11.369 / 11.864 |
| Resident first pass | 10 | 11.199 / 42.362 / 42.362 | 10.553 / 40.727 / 40.727 |
| First search starts daemon | 10 | 29.646 / 50.387 / 50.387 | 28.646 / 61.981 / 61.981 |
| Cursor replay | 20 | 11.103 / 13.317 / 13.542 | 11.033 / 12.765 / 12.941 |

The resident median is 2.5% lower in this paired run (0.238 ms); this is a
small operational improvement. Cold median is about 29 ms. Its ten mixed-query
samples have a higher observed maximum/p95 after the change; that small group
does not establish a production tail distribution. p95 uses the existing
nearest-index ties-to-even rule, so ten-sample p95 is the maximum.

All 120 default-text search pairs and ten full JSON query pairs matched after
normalizing only ordinary cursor clocks. Twenty text cursor-replay pairs and
one exact JSON replay pair matched using the same pinned cursor. Ordered card
IDs, page content and exit codes matched; original body/envelope fingerprints
remained unchanged in both stores. Retrieval algorithms, arithmetic, thresholds,
models and ordering were not changed. This is not curated/scale validation,
a neutral gate or holdout scoring.

The implementation streams JSON over the existing Chunk/End wire format with
no JSON spooling. Every encoder uses a synchronous serde worker bridged by a
two-slot bounded channel. Single-chunk values decode in bounded memory; larger
values use a blocking reader over another two-slot channel. This avoids probing
encoded size on the IPC thread: escaping one large string can scan its entire
contents before a capped writer fills. Channel closure cancels codec workers
on transport loss. Query and replica operations with no byte input pass no input
handle, eliminating the two empty input files as well as four JSON-envelope
spools formerly used by an ordinary query. Original-data application staging
for add/import/export retains its validation and snapshot semantics. Frame
prefix and payload now use one write buffer. Codec code lives in runtime/wire.rs;
no dependencies or protocol variants were added.

Boundary tests cover exact one-chunk JSON, spill into a second chunk, messages
larger than the frame limit, malformed/truncated replies, cancellation and
small/large transfers with an unavailable temporary directory. CLI query tests
use an unavailable temp directory too. Existing ownership, schema fencing,
concurrency, response-delivery/forget barriers and sync behavior remain covered.
All 149 workspace tests passed (two existing fixture-dependent tests ignored),
as did Clippy, formatting, Nix build, release/older-peer checks and LLP references.

A separate diagnostic probe measured process-plus-help median 4.435 ms
and status median 4.676 ms (30 samples each), healthy store opening
0.651 ms, and in-daemon application-operation median 2.752 ms
(one warm diagnostic sample for each of the ten queries). These are illustrative
component probes, not an additive breakdown of the paired timing run. Process/CLI
work is a substantial part of this small-store end-to-end latency.

Other recorded configurations in this review: an initial run against the
previously measured git2 build yielded 9.788/12.133 ms resident median/p95 for
that older build and 9.347/11.707 ms for the build entering this review. Streaming
with a synchronous bounded-size encoding probe, before combining frame writes,
yielded 9.211/12.098 ms for its baseline and 9.002/12.081 ms for streaming. Combining
frame writes with that probe still present yielded 9.365/11.861 ms baseline and
9.024/11.494 ms candidate. The final code removes the encoding probe for large-message
responsiveness and simpler consistent encoding. The small differences across
runs reinforce treating the gain as modest, rather than attributing a precise
standalone benefit to frame-write batching or choosing a size probe for its timing.

Before build: `f7af365fdb3418718d1843aa9bac4f2f3370422455261d6a6d3982da20d37a51`.
Final build: `0b8aab44ba1e944651ac3638fe1ee1860b249cb97f12a394b9e2100f7c5b1e92`.
Release bytes: 105,959,320 before, 106,281,416 final
(+322,096 bytes).
Metadata-only startup was 11.306/11.019 ms before/final, measured separately.
Raw samples, ordered page IDs, copied stores and probe scripts remain outside
the repository.

Final repeated-query medians (ten samples per query per variant):

| Query | Before, ms | Final, ms |
|---|---:|---:|
| `git send` | 9.449 | 9.216 |
| `publication marker` | 9.336 | 9.064 |
| `XDG_RUNTIME_DIR` | 7.300 | 7.037 |
| `native routing` | 9.316 | 9.240 |
| `daemon double fork` | 7.795 | 7.459 |
| `Metro transform cache` | 11.649 | 11.455 |
| `configured devshell` | 11.024 | 10.670 |
| `Swift async actor` | 10.501 | 10.141 |
| `worktree remote branch` | 8.424 | 8.178 |
| `nanometerquux42` | 9.166 | 8.847 |

## 2026-11-13: Bounded resident query work and reusable replica connections

Performance-only cache/ownership changes under LLP 0007. The application acquires
its Store once per operation and holds the checked snapshot around the complete
read. A lazily retained Replica serves internal sync steps and read-only remote
configuration without dropping the warm Store. Mutating replica requests still
invalidate the Store; errors discard both handles. Incoming sync Items move into
the dispatcher without cloning envelopes. Transaction/schema, physical-identity,
publication and original-data delivery guards remain in place.

Clock changes clear eligibility hits and statistics. Query/scope changes clear
ranking work and negative probes, while positive corpus stemming and current-clock
positive document frequencies remain reusable. Immutable FTS-hit and candidate
feature memos, and same-clock eligibility hits, use fixed soft budgets at query
boundaries (256 entries, 64 KiB key payload, 65,536 allocated ID slots per map).
Oversized maps are dropped as a whole. Negative probe cleanup visits recorded
keys, not corpus-wide maps. No runtime cache knob, model change, arithmetic
change, threshold adjustment or ordering change was introduced.

A counting Rust global allocator linked the actual library and repeated
`publication marker` 1,000 times after warming the frozen 24-record store.
Changing the logical clock by one second per call previously retained 3,144,280
additional bytes; the final library retained 188 bytes of probe/working overhead
with no growth across samples. Fixed-clock repeats retained 532 bytes in both
versions. Every serialized Call matched its warmed baseline on every iteration.
These are retained Rust heap deltas, not RSS, excluding native SQLite/model
allocations. Clocks were synthetic; this was not a 1,000-second wall-clock run.
The final probe results are `/tmp/linn-cache-after-vt1_1qrh/results.json`; baseline
results are `/tmp/linn-cache-review-eu06e1ws/results.json`.

Final end-to-end timing compares release binaries built with the same pinned
flake, compiler, profile and embedded assets on native macOS/Apple M3 Ultra.
Separate copies of the same frozen 24-record, 20,537,344-byte store and existing
ten-query sample were used, with repository cwd/caller scope, warm OS caches,
alternating pair order and ten shuffled resident rounds (seed 34709). Each timer
includes a fresh CLI launch through exit, Git context, IPC and default rendering.
No concurrent build, allocator probe or test load; no outlier removal. Cold means
a newly started daemon, not a cold filesystem.

| Condition | Pairs | Before median / p95, ms | Final median / p95, ms |
|---|---:|---:|---:|
| Resident repeated search | 100 | 8.853 / 11.476 | 9.187 / 12.214 |
| Resident first pass | 10 | 11.746 / 43.169 | 11.499 / 42.643 |
| First search starts daemon | 10 | 28.688 / 49.875 | 28.751 / 52.129 |
| Cursor replay | 20 | 10.360 / 11.181 | 10.402 / 11.288 |

Bounding retained state costs 0.334 ms (3.8%) at the resident median in this run;
it is not a query-speedup claim. Cold and cursor medians were nearly unchanged.
The ten-sample groups do not establish a production tail distribution; p95 uses
the existing nearest-index ties-to-even rule. Metadata-only startup measured
10.756/10.804 ms before/final in separate single samples, not a breakdown of
query latency. A diagnostic probe confirmed read-only `sync:remote list` no
longer causes a store-open phase on the following query; its individual operation
timings are not a benchmark of sync speedup.

All 120 text query pairs and ten full JSON query pairs matched after normalizing
only ordinary cursor clocks. Twenty text cursor-replay pairs and one exact JSON
replay matched with the same pinned cursor. Ordered cards, full page contents,
exit codes and original body/envelope fingerprints matched. This small fixture
is not curated/scale validation, a neutral gate or holdout scoring.

Other tested cache configurations, not selected:

- Clearing all query memos, including stemming and statistics, on every query
  change: resident median/p95 9.765/12.853 ms against its paired baseline
  9.012/11.607 ms (`/tmp/lqcachefix-ivhplg06/results.json`).
- Keeping positive corpus stemming and current-clock positive DF while still
  clearing posting/feature memos: 9.448/12.361 ms against 8.724/11.189 ms
  (`/tmp/lqcachefix-av1vrrrf/results.json`).
- Budgeting immutable posting/feature memos but clearing eligibility hits on each
  query change: 9.218/11.496 ms against 8.794/11.207 ms
  (`/tmp/lqcachefix-xkngeyxe/results.json`).

Each configuration preserved page parity. A preliminary timing run of the final
configuration overlapped the allocator probe; its timings are excluded for that
experimental-condition violation, not for sample values. Its raw results remain
at `/tmp/lqcachefix-854m6wo4/results.json`. The isolated final timing results are
`/tmp/lqcachefix-v8_ac_rn/results.json`, produced by
`/tmp/linn-cachefix-latency.py`. Original stores, raw results and temporary probe
sources remain outside the repository.

Binary SHA-256: before
`edd2051ce9da18ac8faba3104ff2ed8ed314e084a3b4b879bf7b645b65b04345`, final
`0f9b8d22b4b8ced89a2406093eda236dda71c9e6296407abe41468a3e2c754b4`.
Release bytes: 106,290,680 before; 106,293,320 final (+2,640).
All 157 workspace tests passed, with two existing fixture-dependent ignores;
Clippy, formatting, LLP references, Nix build and packaged/previous-peer smoke
checks passed. Regression tests cover clock/query/scope changes, bounded memos,
read-only replica reuse and write invalidation. Native Linux/Windows runs and
curated/scale retrieval outcomes remain unverified here.

## Scale CLI latency before and after daemonization (2026-11-13)

Performance-only comparison of pre-daemon `4dc9516dc9804efb342e00af43de6ff309df420c`
against final `ba0ac4df46d64ea7bb5b8ac8c1f55261a7eec461`, using native macOS
on Apple M3 Ultra. Both release packages were built with the pinned Nix flake,
compiler, profile and embedded assets. The existing 29-case stratified
`scale-final-cli.json` sample was reused; all case IDs pass LLP 0005's dev split.
Only original query wordings were used. No holdout scoring or retrieval tuning.

The scale-q4 corpus contains 16,906 records and 189,767 blocks. Records JSONL
SHA-256 is `be42984d615165066a2d3cfc16d863aa010b0a9ec823cde395b7e1c813231a5f`.
A read-only SQLite backup of the existing sentence-dictionary scale store was
prepared with the baseline, then APFS-cloned into independent before/after
stores. Every stored body matches the original corpus bytes. Preparation took
58.168 seconds and is excluded from query timings; both timed stores start with
schema 5 and the same derived state. Snapshot size is 4,488,953,856 bytes.

Each timer includes a fresh CLI subprocess through exit, caller Git context,
store access or IPC, retrieval and default text rendering. Both use the same
repository cwd/caller scope and live CLI eligibility clock, rather than the
case-specific scope and frozen evaluation clock. Pairs alternate execution
order; three resident rounds shuffle the queries with seed 34709. No concurrent
build/test/probe load, no forced OS-cache flush and no outlier removal. Filesystem
caches have been exercised by backup/preparation and earlier calls; this is not
a cold-disk benchmark. New-daemon samples stop only the isolated test owner
before each query. The before binary opens its store/models anew on every call.

| Condition | Pairs | Before median / p95, ms | After median / p95, ms |
|---|---:|---:|---:|
| First query starts daemon | 29 | 213.079 / 531.701 | 211.570 / 503.687 |
| First query pass after metadata-only daemon startup | 29 | 197.614 / 532.586 | 145.785 / 475.765 |
| Resident repeated query | 87 | 182.530 / 464.426 | 133.968 / 347.660 |

Resident median falls 26.6% (48.562 ms; 1.36x ratio of aggregate medians), and
p95 falls 25.1%. Every query's three-repeat median improves. These p95 values
summarize this fixed development sample, not a production tail distribution;
they use the existing nearest-index, ties-to-even rule. First-query medians are
nearly unchanged (0.7% lower); first-query maxima are 4,010.501/4,079.876 ms,
including the first sampled query in both versions. Resident maxima are
758.345/411.211 ms. A separate metadata-only startup sample took 14.252 ms,
with diagnostic store-open time 0.770 ms; it is not a query-latency breakdown.

All 145 paired text pages and exit codes matched. All 29 additional full JSON
pages matched, including ordered cards, spans, text, hints and related pointers;
only ordinary cursor clocks were normalized. Original body/envelope fingerprints
remain identical across both stores and the snapshot. This establishes parity
for the sampled product queries, not a full scale retrieval-score confirmation.

Raw samples, per-query medians, diagnostics, identities and parity results:
`/tmp/linn-scale-x_izk01i/results.json`; benchmark driver:
`/tmp/linn-scale-bench.py` (adapts `/tmp/linn-query-latency.py`). Corpus, stores,
raw output and binaries remain uncommitted. Binary SHA-256: before
`f3d0f8e6f7e0c1b9f88a22562c9d7ec60b569caee31e7063da0cb65e47828a7a`, after
`0f9b8d22b4b8ced89a2406093eda236dda71c9e6296407abe41468a3e2c754b4`.

## Resident libSQL connection experiments (2026-11-13)

The long-lived daemon changes the reuse window from the earlier fresh-process
SQL setting studies. This is a new resident-workload measurement, not a rerun
of those closed trials. Compared the current `ba0ac4d` implementation with four
isolated changes: a 64 MiB connection pager budget, a 1 GiB mapping instead of
256 MiB, a bounded native prepared-read-statement pool, and refreshed planner
statistics. Retrieval algorithms, model arithmetic and stored original bytes
remain unchanged. No holdout data or scores were used.

Native macOS/Apple M3 Ultra; all four source variants and the baseline built
through the same pinned Nix devshell, release profile, toolchain and embedded
assets. The same prepared scale-q4 snapshot from the preceding measurement was
APFS-cloned into five separate store paths. Each daemon had its own connection,
mutable query workspace and caches. The existing 29 stratified dev queries were
used, with the same repository cwd/caller scope and live operation clock.

Each query group invoked all five binaries in shuffled order (seed 34709), with
one complete first traversal then three shuffled resident rounds. Each timer
includes a fresh CLI launch through exit, default text output, Git context, IPC
and store execution. No overlapping builds, tests or probes, no forced OS-cache
flush, and no samples removed. First-traversal samples are reported separately;
they include the initial population of model and query resources. Percentiles
use nearest-index ties-to-even, as in the preceding daemon measurements.

| Variant | Resident samples | Median / p95, ms | Median / p95 change | Final daemon RSS, MiB |
|---|---:|---:|---:|---:|
| Baseline, default pager / 256 MiB mapping | 87 | 132.305 / 342.970 | — | 1,111.3 |
| 64 MiB pager | 87 | 120.610 / 311.112 | -8.8% / -9.3% | 1,192.0 |
| 1 GiB mapping | 87 | 135.165 / 348.967 | +2.2% / +1.7% | 1,835.3 |
| Prepared-statement pool | 87 | 141.975 / 371.943 | +7.3% / +8.4% | 1,110.8 |
| Refreshed planner statistics | 87 | 134.130 / 394.928 | +1.4% / +15.1% | 1,114.8 |

The 64 MiB pager improves all 29 per-query three-repeat medians, with a 10.0%
median of per-query reductions and 9.8% lower mean latency. Its observed RSS
increase is 80.7 MiB, not a promise that total RSS grows by exactly the pager
budget difference. RSS includes native/model allocations and resident mapped
pages; the larger mapping's 724.0 MiB increase is not all private Rust heap.

First-traversal median/p95 in the same order as the table: 167.323/495.368,
151.216/443.978, 165.461/465.339, 178.824/490.578 and 151.438/657.973 ms.
Single metadata-only startup samples were 19.133, 17.904, 16.081, 17.970 and
13.861 ms; these are not controlled startup-speed claims. No whole-database
preload was added.

The statement-pool prototype retained at most 64 idle native read-only SELECT/WITH
statements, with SQL text at most 4 KiB each. Active statement/row leases stayed
outside the pool until the last handle dropped. Returning a native statement
reset it and cleared bindings; failed/finalized statements were not reused.
Idle entries contained no connection references, avoiding reference cycles.
Two temporary integration tests passed for parameter clearing, overlapping live
results, abandoned-result lock release, native schema reprepare and eviction
workloads. This implementation regressed all 29 per-query medians and is not
adopted; the result does not prove that every possible targeted statement-reuse
implementation would regress. The patch and test source remain outside the
repository at `/tmp/linn-resident-sql-statement-experiment.patch` and
`/tmp/linn-statement-tests.rs`.

The pinned native engine reports SQLite 3.45.1. On an unused connection,
`PRAGMA optimize=0x10002` took 0.360 ms and created no planner statistics; its
newer startup-mask behavior is absent in this engine. That no-op was not used
as the statistics timing candidate. The actual statistics candidate ran
`PRAGMA analysis_limit=1000; ANALYZE; PRAGMA optimize;` inside an immediate
transaction after checking schema 5, before daemon startup. It created 51
sqlite_stat1 rows and took 7,511.977 ms, excluded from query timings. For
`Cursor Co-Authored-By setting`, refreshed statistics increased the per-query
median from 45.834 to 639.616 ms. This statistics policy is not adopted.

All 464 candidate/baseline text-page comparisons and exit codes matched across
first traversal and resident rounds. All 116 candidate/baseline full JSON pages
matched, normalizing only ordinary cursor clocks; ordered cards, original spans,
text, hints and related pointers were preserved. Original body/envelope
fingerprints match across all five stores. This is sampled product-page parity,
not full scale retrieval-score or neutral/holdout validation.

Decision: retain only `cache_size=-65536` for ordinary Store connections. Keep
256 MiB mapping, current statement preparation and current planner-statistics
behavior. Rebuild/forget retain their separately measured temporary 32 MiB
write budget and restore the 64 MiB ordinary budget afterward. Secure deletion,
durability, transactions, publication checks, schema fences and journal mode
are unchanged. No runtime algorithm flag or configuration control was added.

Raw timings, per-query values, binary fingerprints, diagnostics and parity:
`/tmp/linn-resident-sql-gypuy5a5/results.json`; driver:
`/tmp/linn-resident-sql-bench.py`. All experimental stores, binaries, prototypes
and raw output remain uncommitted. The baseline and 64 MiB candidate are also
verified separately below as final Nix packages.

### Final packaged pager-cache confirmation

An independent paired run compares the existing Nix release with the final Nix
package containing only the pager setting and its settings assertion. Same
29-query sample, prepared snapshot, caller context and alternating pair order;
three resident rounds shuffled with seed 34709. Both isolated owners are stopped
before every first-request pair. No concurrent build/test/probe load or outlier
removal; a fresh owner is not a cold filesystem.

| Condition | Pairs | Baseline median / p95, ms | 64 MiB pager median / p95, ms |
|---|---:|---:|---:|
| First query, both owners newly started | 29 | 213.014 / 515.544 | 202.548 / 490.113 |
| Resident repeated query | 87 | 133.948 / 348.143 | 120.484 / 315.548 |

Resident median falls 10.1%, p95 9.4%; all 29 per-query medians improve again.
First-request median falls 4.9%. Resident maxima are 412.636/373.257 ms;
first-request maxima are 4,181.287/4,205.449 ms, including the first sampled
query. A separate candidate metadata-only startup sample is 13.661 ms. Final
candidate RSS is 1,188.0 MiB. These sampled p95s are not production tail bounds.

All 145 paired text pages and exit codes matched, as did 29 additional full JSON
pages after normalizing only ordinary cursor clocks. The 29 first-traversal
pairs are included in parity but not in the above timing table: only the
candidate had metadata-only initialization before that traversal; baseline
started its owner with its first query. Both were warm before resident timing.
Original body/envelope fingerprints are unchanged. Raw confirmation:
`/tmp/linn-resident-sql-confirm-jr8du1bk/results.json`; driver:
`/tmp/linn-resident-sql-confirm.py`.

A preliminary packaged run at `/tmp/linn-resident-sql-confirm-1t4_f37t/results.json`
used the earlier pre-daemon comparator's stop routine, stopping only the
candidate between first requests. Its first-request and first-traversal timing
comparisons are invalid and excluded for that setup error. Its valid warm run
measured 133.685/345.814 ms versus 120.454/319.871 ms, with full sampled parity.
The final run above corrects teardown of both owners; neither run selected a new
configuration or removed sample outliers.

Final package: `/nix/store/ddxrms0q9idbpyn88fp6wi0zjz5f43p8-linn-0.1.1/bin/linn`.
SHA-256 baseline
`0f9b8d22b4b8ced89a2406093eda236dda71c9e6296407abe41468a3e2c754b4`, final
`06985a4cc77a1790b93490bc7fdc031874867553655ae30a78aa0197f540463e`.
Both packaged executables are 106,293,320 bytes. All 157 workspace tests passed,
with two existing fixture-dependent ignores. Formatting, strict workspace
Clippy, LLP references, diff checks, Nix build and packaged release smoke with
the previous compatible peer passed. No experimental cache/library/statistics
code remains in the final implementation. Only native macOS scale CLI behavior
was measured; full retrieval scoring and other platforms were not rerun.

## 2026-11-13: denser SQLite pages for new stores

Performance-only physical-layout study after `74cae10`. The production change
requests `page_size=16384` before initial persistent DDL. Existing files retain
their current page size. No compaction command, automatic VACUUM, logical schema
migration, RPC change, new table, codec or index removal is adopted. Retrieval,
original envelopes/bodies, exact float32 arrays and all numerical kernels remain
unchanged. SQL schema 5 and application compatibility 1 remain appropriate.

### Space attribution and isolated physical layouts

Read-only dbstat profiling of the prepared 16,906-record/189,767-block scale-q4
snapshot found 4,488,953,856 file bytes, 4,096-byte pages and 326,082 free pages
(1,335,631,872 bytes). Those free pages are reusable high-water allocation, not
additional live records or proof of continuing logical growth. The vector table
uses 779,231,232 bytes for 391,301,689 bytes of row payload, leaving 383,402,464
bytes unused; section means leave another 153,610,987 bytes unused. Exact
2,048-byte vectors plus row overhead cannot fit two rows on a 4 KiB table page.

To isolate geometry, independent APFS clones were repacked with the pinned native
libSQL engine at 4, 8 and 16 KiB. Native repacking took 25.975, 23.552 and 23.185
seconds respectively, before query timings. This is diagnostic preparation only;
it is not a delivered maintenance operation or a reason to compact on startup.
The source snapshot and shared evaluation stores were never modified.

| Physical layout, same logical contents | File bytes | Decimal GB |
|---|---:|---:|
| Historical 4 KiB snapshot, including free pages | 4,488,953,856 | 4.489 |
| Repacked 4 KiB | 3,118,706,688 | 3.119 |
| Repacked 8 KiB | 2,719,309,824 | 2.719 |
| Repacked 16 KiB | 2,605,088,768 | 2.605 |

The comparable 16 KiB layout saves 513,617,920 bytes (16.5%) versus repacked
4 KiB. The 42.0% reduction from the historical file also reclaims old free pages;
that is not the new-store page-size saving, and existing files are not shrunk by
this change. Block vectors shrink to 444,465,152 bytes and section means to
177,553,408 bytes: together they explain 468,467,712 bytes of the comparable
saving. Larger pages pack those rows more densely without changing vector bits.
Fallback chunks already pack efficiently and change little.

All values and row counts in all 37 persistent real tables match across all four
files, checked by ordered type-tagged SHA-256 digests. This includes original
bytes/hashes, lifecycle/publication state, FTS shadow data, block vectors, mean
vectors and fallback chunks. Virtual tables are checked through their persisted
shadow tables and query outputs. No embeddings were quantized or re-encoded for
this comparison.

### Scale query latency and parity

Native macOS/Apple M3 Ultra, same pinned Nix release executable and assets on all
four independent files, with the adopted 64 MiB pager and 256 MiB mapping limits.
Used the existing 29 stratified dev query wordings, repository cwd/caller scope
and live CLI clock. Each timer includes a fresh CLI subprocess through exit,
Git context, IPC, execution and default rendering. All four variants run in
shuffled order per query; seed 34709. No concurrent build/test/probe load, forced
filesystem-cache flush or outlier removal. Percentiles use nearest-index
 ties-to-even. Fresh owner means initialized daemon resources, not cold disk.

| Layout | Resident samples | Median / p95, ms | Fresh-owner median / p95, ms (29 each) |
|---|---:|---:|---:|
| Historical 4 KiB | 87 | 121.102 / 313.558 | 166.846 / 413.581 |
| Repacked 4 KiB | 87 | 119.565 / 323.895 | 167.676 / 413.937 |
| Repacked 8 KiB | 87 | 120.636 / 325.900 | 169.291 / 409.280 |
| Repacked 16 KiB | 87 | 117.623 / 320.429 | 167.356 / 400.484 |

The screening p95s and a few per-query medians were mixed, so a longer paired
confirmation compared the historical 4 KiB file with 16 KiB using the same
queries, ten shuffled resident rounds and randomized pair order. No settings
were retuned between screening and confirmation.

| Confirmation | Samples each | Median / p95, ms | Mean / maximum, ms |
|---|---:|---:|---:|
| 4 KiB | 290 | 120.173 / 312.810 | 143.624 / 359.931 |
| 16 KiB | 290 | 117.227 / 307.260 | 140.585 / 358.441 |

Median/p95 fall 2.5%/1.8%. Twenty-eight of 29 per-query medians improve; the
remaining query changes from 18.431 to 18.792 ms (+0.361 ms). This is modest
sampled performance improvement, not a universal per-query or tail guarantee.
Final screening RSS is 1,185.7 MiB for historical 4 KiB and 1,129.4 MiB for
16 KiB; mapped/native allocations are included, not only Rust heap.

All 435 candidate/baseline text-page comparisons from the four-way fresh-owner,
first-traversal and resident screening match, with 87 additional full JSON page
comparisons. The longer confirmation adds 319 matching text comparisons and
29 matching JSON pages. Only ordinary cursor clocks are normalized; ordered
cards, spans, text, hints, related pointers and exit codes match. These are
sampled product pages, not a full retrieval-score/neutral/holdout rerun.

### Fresh builds, write growth and small stores

Three fresh builds per layout used all 1,376 original q4 records, alternating
order and separate files. The 4 KiB files were initialized at 4 KiB before
Store::build; 16 KiB files used the new default. Native integrity checks passed.

| Fresh q4 builds | Median file bytes | Median build time, ms |
|---|---:|---:|
| 4 KiB | 196,186,112 | 12,571.776 |
| 16 KiB | 176,128,000 | 12,470.910 |

Fresh q4 files are 10.2% smaller; build time is roughly unchanged in this small
three-sample comparison. File sizes vary by a few pages across independent
builds; the table reports medians rather than selecting a favorable run.

A separate growth trial started from identical q4 state repacked at each page
size, then added the same three original corpus bodies (311, 1,884 and 9,300
bytes) under fixed test IDs and forgot the first added record. The native Store
remained open across the sequence; each add exercises the existing complete
rebuild/publication path. No unrelated benchmark/build load overlapped.

| Page size | Initial / final file bytes | Three-add median, ms | Forget, ms |
|---|---:|---:|---:|
| 4 KiB | 194,789,376 / 196,718,592 | 16,352.600 | 2,132.906 |
| 8 KiB | 178,544,640 / 180,822,016 | 16,067.538 | 2,013.725 |
| 16 KiB | 174,440,448 / 176,488,448 | 15,675.180 | 1,883.253 |

The smaller 16 KiB layout persists through these writes; add/forget do not slow
in this trial. Per-step allocation is rounded and workload-dependent: the net
increases are 1,929,216 and 2,048,000 bytes at 4/16 KiB, so this does not claim
that every individual append allocates fewer bytes. No full scale ingest-time
or long-duration growth guarantee is claimed from q4 writes.

Independent fresh builds of the frozen 24-record small fixture use 20,516,864
bytes at 4 KiB and 20,086,784 at 16 KiB (2.1% smaller). Empty stores instead use
290,816 and 1,097,728 bytes: the larger minimum allocation costs 806,912 bytes
before enough data accumulates. This is the explicit small-store space tradeoff.
Original body/envelope bytes and every block, mean and fallback vector payload
match across the fresh q4 builds, all three write histories and the small fixture.

Decision: use fixed 16 KiB pages for newly created stores. Keep existing page
sizes, native durability, secure deletion, journal mode, cache byte budgets and
all indexes. No automatic maintenance is added. The production diff is one
connection setting, its existing settings assertion and a regression test that
opening a legacy 4 KiB file twice preserves its layout.

Raw artifacts under `/tmp/linn-layout-z76qchdp/`: `profile.json`, `results.json`,
`confirmation.json`, `writes.json`, `fresh-rounds.json`, `small.json` and
`write-fidelity.json`. Drivers are `/tmp/linn-layout-prepare.py`,
`/tmp/linn-layout-bench.py`, `/tmp/linn-layout-confirm.py`,
`/tmp/linn-layout-writes.py`, `/tmp/linn-layout-fresh-repeat.py` and
`/tmp/linn-layout-small.py`. The temporary native diagnostic is archived at
`/tmp/linn-store-layout-probe-final.rs`; it is not part of the product or repo.
Corpora, databases and raw output remain uncommitted. The physical analysis uses
SQLite's dbstat accounting and page geometry; native repacking/query/build/write
operations all use the pinned libSQL engine rather than Python's SQLite engine.

Final validation: all 158 workspace tests pass, with two existing fixture-dependent
ignores. Formatting, strict all-target/all-feature workspace Clippy, LLP references,
whitespace checks and Nix build pass. Packaged release smoke with the previous
compatible binary passes search/show, exact backup restore, retry, schema upgrade,
truncation and interrupted publication. An additional final-package check matches
all 29 full JSON search pages and verifies physical page sizes: old baseline
4 KiB, comparison store 16 KiB, newly created store 16 KiB. All 1,376 pre-existing
q4 records retain their exact original bytes through the tested write histories.

Final package `/nix/store/z0msnad4lx60mqkywfyrbrp9wb73vfv0-linn-0.1.1/bin/linn`,
SHA-256 `c570bbd6c941e7bc1c54684dea88bcba6d2090d9a65f627d56cdec25ab46b6eb`;
baseline SHA-256 `06985a4cc77a1790b93490bc7fdc031874867553655ae30a78aa0197f540463e`.
The final-package comparison is recorded in `package-check.json` in the same raw
artifact directory. Native timings are macOS only; other-platform performance
and full scale ingest were not measured. No partial compaction prototype remains.
