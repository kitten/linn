Add this to your agent's system prompt:

```text
Use linn as cross-session memory. Read supplied material to identify the question, then search for the specific behavior, decision or constraint before further research. Avoid product-only queries and issue numbers. Read relevant results, and search again as questions arise. Stop once sufficiently supported; verify time-sensitive claims against current primary sources. Consult linn <command> --help as needed. Save only new discoveries with context and sources, never retrieved knowledge or summaries; correct outdated memories. Keep plans/tasks elsewhere and enduring docs in-repo. No need to mention linn.
```
