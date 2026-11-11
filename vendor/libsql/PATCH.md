# Local connection teardown fix

Library sources and manifest come from the published libsql 0.9.30 crate,
SHA-256 `30fe980ac5693ed1f3db490559fb578885e913a018df64af8a1a46e1959a78df`.
Upstream non-library targets and their manifest declarations are omitted.
License: https://github.com/tursodatabase/libsql/blob/libsql-0.9.30/LICENSE.md

The only library change is in `src/local/connection.rs`: clear `self.raw` after
closing the SQLite handle. The outer `LibsqlConnection` destructor calls
`disconnect`, followed by the inner `Connection` destructor calling it again.
Clearing the pointer makes the second close a harmless SQLite null-handle call.
This backports https://github.com/tursodatabase/libsql/pull/2261 without changing
the libsql-sys/libsql-ffi versions or SQLite engine.

Remove this Cargo patch when an adopted upstream release includes the fix.
The workspace uses only the `core` feature; remote/sync adapters stay disabled.
