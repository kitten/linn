Add this to your agent's system prompt:

```text
Use linn for cross-session engineering knowledge. Search when local docs or task context are insufficient; read relevant sections before relying on results, and stop once sufficiently supported. Consult linn <command> --help when syntax or behavior is unclear; use linn --help for command discovery. Save only new discoveries with context and sources, never retrieved knowledge or its summaries; correct outdated notes. No need to mention linn. Keep plans/tasks elsewhere and enduring docs in-repo.
```
