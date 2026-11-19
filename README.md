**linn** is a lab notebook that your agent uses to record and recall findings, insights, and research with evidence.

Your agents already have short-term memory, represented in their context, plans, or tasks. Your repos already have long-term memory in skills, documentation, code comments, tests, or other files.

Medium-term memory is what agents often lose between tasks: research, insights, and short notes not settled enough to commit yet. Without medium-term memory, threads repeat work another thread already did, and you're monitoring threads to prevent them from going off track.

linn sits in this medium-term memory niche. You trust your agents to write your code. Models are intelligent enough to record what's worth writing down and find it later. They only need a tool to do it deliberately.

- **Deliberate, evidence-backed notes**. Agents write a note when they learn something worth keeping, link it to files and other sources it came from.
- **Append-only ledger.** Nothing is deleted and notes are never edited in place. Newer notes may supersede older ones, and forgotten notes are stashed away.
- **Optionally scoped.** Notes can be searched plainly, or queried using filters, such as a specific git remote with `scope:org/repo`.
- **Progressive disclosure.** Search results come back as short summary cards, and agents can either expand them or refine their search step by step, so irrelevant notes don't fill their context.
- **Fast and fully local.** Search runs on SQLite with a small, built-in embedding model. Queries are always local and answered in milliseconds.
- **P2P mesh sync.** Paired devices can sync notes privately, either on a local network, peer to peer, or via a relay with end-to-end encryption.
- **Just a CLI.** Agents interact with a simple, clear CLI that will work in any harness.

[Skip to setup steps](#agent-setup)

## How agents use linn

Medium-term memory should be automatic. Your agent interacts with linn; you don't have to. Its goal is for agents to write and read memories automatically.

Suppose you ask your agent to investigate why a worker keeps timing out. Before tracing the code, it checks whether an earlier session learned anything useful:

```sh
linn search 'worker timeout configuration'
```

It'll get short cards with headings, excerpts, and references. One mentions that timeout settings are read only at startup. The agent opens that result to read the original note and its sources:

```sh
linn show [--all] <fetch-id>
```

During a search, the agent sees enough to choose what is worth reading, then loads the evidence it needs. To complete a search it may:
- refine the search query
- open another section if the first note leaves a question unanswered
- get an outline of the document
- or; get the full document or an excerpt

During a search, it may also see related pointers and near misses as hints. When nothing is found, the agent may see different hints to pivot to a different search.

This is progressive disclosure. The agent stops when it has enough context, without having to search too broadly, traverse full documents, or perform too many individual steps. 
The agent sees just enough to dive deeper, into the correct note, or move sideways to pivot to another search.

As an agent then continues its work, it may do research, discover new findings, or learn something new, which it can then save.

```sh
echo "My new finding" | linn add - [--parent <old-id>]
```

The commands also account for labelling (with `--label` or for corrections with `--parent`). This allows an agent to correct or forget memories. While it can pipe notes into `linn add` as text via stdin, it can also save full documents with file paths. Any new notes may also be automatically tagged with the current repository, and an author email for later filtering.

For the full command list and options, run:

```sh
linn --help
```

## How search works

A notebook of memories is only useful if your agent can find what it needs, as it needs it. If search misses it, the agent repeats the work; if it returns the wrong note, the agent acts on it.
linn's search aims to put the right note first, never return closed or superseded notes, and say plainly when nothing matches, so the agent knows to stop looking.

![Diagram of linn's search pipeline in five phases: the query is rewritten and parsed, matched by exact and stemmed BM25 full-text search with a part-of-speech word-pair bonus, assembled into a tiered first page (with query rewrites and embedding neighbours when it's empty, plus filename matches), refined by lexical and vector competition, passage selection and an eligibility filter, then returned as up to five results with related pointers, or as "Nothing qualified." with near-miss hints.](./evals/docs/search-algorithm.png)

**Match.** linn runs two BM25 full-text searches, one on exact words and one on stemmed words, so "caching" also matches "cache". Full-text search comes first because most queries contain a specific name or phrase, which exact matching finds reliably. Part-of-speech (POS) word pairs add a bonus when a note uses the query's words in order. A note must contain every name and about half of the other words. Matches are grouped into tiers first, then sorted by score: whole phrase, every word, then the rest. Notes from the current repository rank above other repositories' notes, which stay searchable.

**Page.** The first page combines the best matches from both searches. If only a few notes match, linn adds matching headings and one close embedding neighbour. If nothing matches, it tries query rewrites and embedding neighbours in turn and keeps the first that returns a result. A query naming a file puts that file's note first.

**Refine.** Embeddings find notes related in meaning, but an exact match is usually stronger. In the lexical and vector competition, a related note replaces a full-text match only if it scores higher on both. Passage selection picks each note's best-matching part, and closed or superseded notes are dropped, so only current notes are returned.

**Output.** Every result costs context, so only five short cards are returned, each with a heading and excerpt, plus a few related pointers. `show` returns a note's original text, `show:outline` lists its headings, and `search:more` returns more results. If no note qualifies, a few near misses are listed instead. An empty result means the notes don't cover the question; linn returns nothing rather than a weak match.

Search runs entirely on your machine. The embedding model is compiled into the binary, so no query makes a model or network call.

## Agent setup

### 1. Install linn

Install with [Homebrew](https://github.com/kitten/homebrew-tap):

```sh
brew install kitten/tap/linn
```

Available for macOS 14+ on Apple silicon and Linux on ARM64 or x86_64.
To install manually, download the latest [release](https://github.com/kitten/linn/releases/latest) and place the single-binary CLI in a folder on your `PATH`.

### 2. Add system prompt

Add the following instructions to your harness:

> Use linn for cross-session memory. Identify the question from supplied material; before further research, search its specific behavior, decision or constraint (`linn search`). Avoid product-only queries and issue numbers. Read relevant results (`linn show`); search again as questions arise. Stop when sufficiently supported; verify time-sensitive claims against current primary sources. Consult `linn <command> --help` as needed. Save useful findings, insights and explanations across repositories, with context and sources (`linn add`). Correct outdated memories. Do not use linn as a task journal or changelog. Keep plans/tasks elsewhere and enduring docs in-repo. No need to mention linn.

Edit it as needed. This is the recommended prompt, and it may change over time as it's tuned.

<details>
<summary>Claude</summary>

Add the prompt to `~/.claude/CLAUDE.md` to apply it across projects.

</details>

<details>
<summary>Codex</summary>

Add the prompt to `~/.codex/AGENTS.md` to apply it across projects.

</details>

<details>
<summary>Cursor</summary>

Add the prompt to `AGENTS.md` in your project directory or any parent directory. For example, `~/git/AGENTS.md` applies across projects under `~/git/`. Cursor CLI also reads `CLAUDE.md` and rules in `.cursor/rules/`. See [Cursor CLI's rules documentation](https://cursor.com/docs/cli/using#rules).

</details>

### 3. Disable your harness's default memory system

You should also disable your harness's built-in memory system, so there's no competing system that fills up your context. _(This is optional, but recommended)_

<details>
<summary>Claude</summary>

Add these entries to `~/.claude/settings.json`:

```plaintext
{
  "autoMemoryEnabled": false,
  "env": {
    "CLAUDE_CODE_DISABLE_AUTO_MEMORY": "1"
  }
}
```

After disabling auto memory, back up anything you want to keep, then delete each project's `~/.claude/projects/<project>/memory/` directory, including `MEMORY.md` and its topic files.

If you set `CLAUDE_CONFIG_DIR` or `autoMemoryDirectory`, use that location instead. See [Claude's memory storage documentation](https://code.claude.com/docs/en/memory#storage-location).

</details>

<details>
<summary>Codex</summary>

Add this section to `~/.codex/config.toml`:

```plaintext
[features]
external_agent_memory_import = false
memories = false

[memories]
generate_memories = false
use_memories = false
```

After disabling memories and closing Codex, back up anything you want to keep, then delete `~/.codex/memories/` and the memory database `~/.codex/memories_1.sqlite`, including its `-wal` and `-shm` sidecar files if present.

If you set `CODEX_HOME`, use that directory instead of `~/.codex`. See [Codex's memory artifacts](https://github.com/openai/codex/blob/main/codex-rs/memories/README.md) and [database paths](https://github.com/openai/codex/blob/main/codex-rs/state/src/sqlite.rs).

</details>

<details>
<summary>Cursor</summary>

Cursor CLI has no built-in cross-session memory system to disable. No additional setup is needed. See [Cursor CLI's configuration reference](https://cursor.com/docs/cli/reference/configuration).

</details>

## Sync setup

If you work across machines or accounts, you can pair `linn` into a mesh, allowing you to sync your notebook between them. Each machine keeps its own local copy, and sync brings their notes, history, and deletions together.

To get started, create an invitation on one machine:

```sh
linn sync:pair
```

Then, on the other machine, run this and paste the invitation when prompted:

```sh
linn sync:pair --join
```

The invitation works once and expires after ten minutes. You'll be offered a first sync after pairing. After that, whenever you want to bring your notebooks up to date, run:

```sh
linn sync
```

**Sync is manual for now.** You only need to run it on one machine. It'll gather changes from the peers it can reach and share the combined notebook back with them. If a machine is asleep or offline, you can sync again when it's available.

### How it works

linn starts a small local daemon automatically. It keeps the store ready between commands and handles incoming sync requests, so you don't have to open a terminal on every machine. There's nothing extra to install or start; the daemon just needs to be running on the machines you want to sync with.

Paired machines and accounts form a peer-to-peer mesh. You can start a sync from any member, and peers can pass changes through to members you can't reach directly. Peers connect directly when they can, but fall back to a relay to carry traffic when they can't. Already-paired peers can still sync on a local network when the relay isn't reachable.

All notes travel end-to-end encrypted between paired devices, including when they pass through the relay. **The relay or anyone else on your network can't read your notes.**

## Sharing notes

linn can also be used to share knowledge with a teammate. This is done by exporting a bundle of notes and sending it to them to import:

```sh
linn export --scope org/repo --since 30d --output repo-notes.jsonl
```

By default, an export contains current records only. You can filter it with `--scope`, `--kind note|doc`, `--since`, or `--id` for specific records, and add `--all-states` to include history and corrections. Your teammate then merges the bundle into their own notebook:

```sh
linn import repo-notes.jsonl
```

Import keeps each record's original ID, so importing the same bundle twice doesn't create duplicates, and later corrections still link to the notes they replace. Deletions in a bundle are permanent, so only import bundles from people you trust.

Alternatively, ask your agent to:

- "Export everything we've learned about the payments service this month so I can send it to Sam."
- "Bundle the notes from this investigation for the teammate taking it over."
- "Import Alex's bundle and summarise anything that contradicts our notes."
- "Make a full backup of my linn notebook, including its history."

## Example prompts

The system prompt covers everyday use, but you can also ask your agent about notes directly.

**Before starting work:**

- "Check linn for anything we've learned about the release workflow before you start."
- "Has an earlier session already looked into why CI is slow on Linux? Search linn before profiling anything."
- "We're upgrading the database driver. Search linn for past upgrade problems before you plan it."

**Research and investigation:**

- "Compare the three rate-limiting libraries we discussed, and save the trade-offs to linn with sources."
- "Find out why this test only flakes on Fridays, and record each dead end in linn so we don't chase this again."
- "Read the upstream changelog for the next major version and note anything that affects us."

**Corrections and upkeep:**

- "The linn note on worker timeouts is out of date since the config reload change. Correct it."
- "Review this repo's linn notes from the last month and correct anything the code no longer supports."
- "Move anything settled in linn about the build system into AGENTS.md, and close the notes."

**Handoffs between threads:**

- "Save where this investigation stands to linn so another thread can pick it up."
- "Before you continue, check linn for what the previous thread found about this bug."

## Acknowledgements

### Sources

- [nlprule](https://github.com/bminixhofer/nlprule)
- [LanguageTool](https://github.com/languagetool-org/languagetool/tree/v5.2)
- [Apache OpenNLP](https://opennlp.apache.org/)
- [Stable Static Embedding for Retrieval MRL v2, RikkaBotan](https://huggingface.co/RikkaBotan/stable-static-embedding-fast-retrieval-mrl-en-v2)
- [SQLite](https://www.sqlite.org/)
- [libSQL](https://github.com/tursodatabase/libsql)
- [Porter stemming algorithm, Martin Porter](https://tartarus.org/martin/PorterStemmer/)
- [iroh](https://www.iroh.computer/)
- [Lv and Zhai (2011). When Documents Are Very Long, BM25 Fails!](https://lintool.github.io/robust04-analysis-papers/p1103-lv.pdf)

### Influences

- [Zhai (1997). Fast Statistical Parsing of Noun Phrases for Document Indexing.](https://aclanthology.org/A97-1046.pdf)
- [Wang et al. (2026). BM25 Wins at Scale: A Scaling Study of Retrieval-Augmented Generation Paradigms.](https://arxiv.org/abs/2607.26497)
- [Gloaguen et al. (2026). Evaluating AGENTS.md: Are Repository-Level Context Files Helpful for Coding Agents?](https://arxiv.org/abs/2602.11988)
- [Chen et al. (2026). MemDev-Bench: Does Memory Help Code Agents? Benchmarking Memory Systems over Real Repository Histories.](https://arxiv.org/abs/2610.27524)
- [Fan et al. (2026). VibeMemBench: Evaluating Memory Systems for Coding Agents on Real Repository Coding Tasks.](https://arxiv.org/abs/2609.23570)
- [J. Zhao et al. (2026). Beyond Retrieval: Bi-Temporal State Arbitration for Longitudinal Healthcare Agents.](https://aclanthology.org/2026.knowfm-1.10/)
- [S. Zhou et al. (2026). Filesystem-Based Memory for LLM Agents: Organization, Evolution, and Sustainability.](https://arxiv.org/abs/2607.26637)
- [W. Zhou et al. (2026). Are We Ready For An Agent-Native Memory System?](https://arxiv.org/abs/2606.24775)
