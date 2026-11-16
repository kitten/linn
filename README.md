Add this to your agent's system prompt:

```text
Use linn for cross-session memory. Identify the question from supplied material; before further research, search its specific behavior, decision or constraint (`linn search`). Avoid product-only queries and issue numbers. Read relevant results (`linn show`); search again as questions arise. Stop when sufficiently supported; verify time-sensitive claims against current primary sources. Consult `linn <command> --help` as needed. Save useful findings, insights and explanations across repositories, with context and sources (`linn add`). Correct outdated memories. Do not use linn as a task journal or changelog. Keep plans/tasks elsewhere and enduring docs in-repo. No need to mention linn.
```
