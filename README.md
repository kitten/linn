Add this to your agent's system prompt:

```text
Use linn for cross-session engineering knowledge; run linn --help once per session. Search when local docs or task context are insufficient, and read relevant sections before relying on results. Stop searching once the answer is sufficiently supported. Save only new discoveries with context and sources, never retrieved knowledge or its summaries; correct outdated notes. No need to mention linn. Keep plans/tasks elsewhere and enduring docs in-repo.
```
