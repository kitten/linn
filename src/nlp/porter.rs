//! Direct access to the pinned libSQL Porter tokenizers, without scratch FTS writes.
//! @ref LLP 0002#ordered-fallbacks
use libsql::ffi;
use std::{
    cell::RefCell,
    ffi::{CStr, c_char, c_int, c_void},
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
    ptr::{self, NonNull},
    rc::Rc,
};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("SQLite tokenizer initialization failed: {0}")]
    Initialization(String),
    #[error("Porter {operation} failed (SQLite code {code})")]
    Sql {
        operation: &'static str,
        code: c_int,
    },
    #[error("Porter API unavailable: {0}")]
    Unavailable(&'static str),
    #[error("Porter input exceeds the tokenizer byte limit")]
    InputTooLong,
    #[error("Porter returned an invalid UTF-8 token")]
    InvalidToken,
    #[error("Porter token callback panicked")]
    CallbackPanicked,
    #[error("Porter tokenizer is already in use on this thread")]
    Reentrant,
}
type Result<T> = std::result::Result<T, Error>;
type Callback =
    unsafe extern "C" fn(*mut c_void, c_int, *const c_char, c_int, c_int, c_int) -> c_int;
type Tokenize = unsafe extern "C" fn(
    *mut ffi::Fts5Tokenizer,
    *mut c_void,
    c_int,
    *const c_char,
    c_int,
    Option<Callback>,
) -> c_int;

fn check(code: c_int, operation: &'static str) -> Result<()> {
    if code == ffi::SQLITE_OK {
        Ok(())
    } else {
        Err(Error::Sql { operation, code })
    }
}
fn input_len(len: usize) -> Result<c_int> {
    len.try_into().map_err(|_| Error::InputTooLong)
}

struct Database {
    raw: NonNull<ffi::sqlite3>,
    // All handles and tokenizer instances stay on their creating thread.
    _thread: PhantomData<Rc<()>>,
}
struct Statement(NonNull<ffi::sqlite3_stmt>);
impl Drop for Statement {
    fn drop(&mut self) {
        // SAFETY: this wrapper exclusively owns a statement from sqlite3_prepare_v2.
        unsafe {
            ffi::sqlite3_finalize(self.0.as_ptr());
        }
    }
}
impl Database {
    fn open() -> Result<Self> {
        // Enter libSQL's own process-wide initialization guard before raw FFI
        // can initialize SQLite. Configuring threading after that is too late.
        #[allow(deprecated)] // The pinned crate's synchronous initialization API.
        let _initialized =
            libsql::Database::open_in_memory().map_err(|e| Error::Initialization(e.to_string()))?;
        let mut raw = ptr::null_mut();
        // SAFETY: the static filename is NUL-terminated; raw is a valid output slot.
        let code = unsafe { ffi::sqlite3_open(c":memory:".as_ptr(), &mut raw) };
        let db = Self {
            raw: NonNull::new(raw).ok_or(Error::Unavailable("database handle"))?,
            _thread: PhantomData,
        };
        check(code, "open")?;
        Ok(db)
    }
    fn api(&self) -> Result<NonNull<ffi::fts5_api>> {
        let mut raw = ptr::null_mut();
        // SAFETY: the connection is live and thread-owned; SQL and output slots are valid.
        let code = unsafe {
            ffi::sqlite3_prepare_v2(
                self.raw.as_ptr(),
                c"SELECT fts5(?1)".as_ptr(),
                -1,
                &mut raw,
                ptr::null_mut(),
            )
        };
        let statement = NonNull::new(raw).map(Statement);
        check(code, "prepare API lookup")?;
        let statement = statement.ok_or(Error::Unavailable("API statement"))?;
        let mut api: *mut ffi::fts5_api = ptr::null_mut();
        // SAFETY: api stays alive through step and finalization. SQLite writes the API
        // pointer synchronously, and the binding has no destructor or retained owner.
        check(
            unsafe {
                ffi::sqlite3_bind_pointer(
                    statement.0.as_ptr(),
                    1,
                    (&mut api as *mut *mut ffi::fts5_api).cast(),
                    c"fts5_api_ptr".as_ptr(),
                    None,
                )
            },
            "bind API lookup",
        )?;
        // SAFETY: the prepared statement and its binding are live.
        let code = unsafe { ffi::sqlite3_step(statement.0.as_ptr()) };
        if code != ffi::SQLITE_ROW {
            return Err(Error::Sql {
                operation: "read API",
                code,
            });
        }
        NonNull::new(api).ok_or(Error::Unavailable("FTS5 API"))
    }
}
impl Drop for Database {
    fn drop(&mut self) {
        // SAFETY: statements are finalized locally and Tokenizers drops both instances
        // before this connection. close_v2 also safely defers teardown if SQLite is busy.
        unsafe {
            ffi::sqlite3_close_v2(self.raw.as_ptr());
        }
    }
}

struct Tokenizer {
    raw: NonNull<ffi::Fts5Tokenizer>,
    tokenize: Tokenize,
    delete: unsafe extern "C" fn(*mut ffi::Fts5Tokenizer),
}
impl Tokenizer {
    fn new(api: NonNull<ffi::fts5_api>, name: &CStr, args: &mut [*const c_char]) -> Result<Self> {
        let mut user = ptr::null_mut();
        // SAFETY: the API is supplied by the live owning connection. Function pointer
        // fields are nullable; zero initialization is valid for this C struct.
        let mut methods: ffi::fts5_tokenizer = unsafe { std::mem::zeroed() };
        let find =
            unsafe { api.as_ref().xFindTokenizer }.ok_or(Error::Unavailable("xFindTokenizer"))?;
        // SAFETY: name is a valid C string; methods and user are writable output slots.
        check(
            unsafe { find(api.as_ptr(), name.as_ptr(), &mut user, &mut methods) },
            "find tokenizer",
        )?;
        let create = methods.xCreate.ok_or(Error::Unavailable("xCreate"))?;
        let delete = methods.xDelete.ok_or(Error::Unavailable("xDelete"))?;
        let tokenize = methods.xTokenize.ok_or(Error::Unavailable("xTokenize"))?;
        let mut raw = ptr::null_mut();
        // SAFETY: all arguments are static C strings and user belongs to this registration.
        // On failure the tokenizer constructor owns cleanup, as required by the FTS5 API.
        check(
            unsafe { create(user, args.as_mut_ptr(), input_len(args.len())?, &mut raw) },
            "create tokenizer",
        )?;
        Ok(Self {
            raw: NonNull::new(raw).ok_or(Error::Unavailable("tokenizer instance"))?,
            tokenize,
            delete,
        })
    }
    fn tokens(&mut self, text: &str) -> Result<Vec<String>> {
        let mut tokens = self.occurrences(text)?;
        tokens.sort_unstable();
        tokens.dedup();
        Ok(tokens)
    }
    fn occurrences(&mut self, text: &str) -> Result<Vec<String>> {
        let length = input_len(text.len())?;
        let mut output = Output::default();
        // SAFETY: instance is live and exclusively borrowed. Input and output remain
        // valid for this synchronous call. The callback catches unwinds and copies tokens.
        let code = unsafe {
            (self.tokenize)(
                self.raw.as_ptr(),
                (&mut output as *mut Output).cast(),
                ffi::FTS5_TOKENIZE_DOCUMENT as c_int,
                text.as_ptr().cast(),
                length,
                Some(collect_token),
            )
        };
        if let Some(error) = output.error {
            return Err(error);
        }
        check(code, "tokenize")?;
        Ok(output.tokens)
    }
}
impl Drop for Tokenizer {
    fn drop(&mut self) {
        // SAFETY: this wrapper owns the instance; its registration's connection outlives it.
        unsafe {
            (self.delete)(self.raw.as_ptr());
        }
    }
}
#[derive(Default)]
struct Output {
    tokens: Vec<String>,
    error: Option<Error>,
}
unsafe extern "C" fn collect_token(
    ctx: *mut c_void,
    _flags: c_int,
    token: *const c_char,
    len: c_int,
    _start: c_int,
    _end: c_int,
) -> c_int {
    // SAFETY: only Tokenizer::tokens supplies this callback and a live Output pointer.
    let output = unsafe { &mut *ctx.cast::<Output>() };
    let result = catch_unwind(AssertUnwindSafe(|| {
        if len < 0 || (len > 0 && token.is_null()) {
            return Err(Error::InvalidToken);
        }
        let bytes = if len == 0 {
            &[]
        } else {
            // SAFETY: FTS5 lends len bytes until the callback returns; never retain its pointer.
            unsafe { std::slice::from_raw_parts(token.cast::<u8>(), len as usize) }
        };
        let text = std::str::from_utf8(bytes).map_err(|_| Error::InvalidToken)?;
        output.tokens.push(text.to_owned());
        Ok(())
    }));
    match result {
        Ok(Ok(())) => ffi::SQLITE_OK,
        Ok(Err(error)) => {
            output.error = Some(error);
            ffi::SQLITE_ERROR
        }
        Err(_) => {
            output.error = Some(Error::CallbackPanicked);
            ffi::SQLITE_ERROR
        }
    }
}

struct Tokenizers {
    // Field destruction order is significant: instances precede their connection.
    word: Tokenizer,
    index: Tokenizer,
    exact: Tokenizer,
    _db: Database,
}
impl Tokenizers {
    fn new() -> Result<Self> {
        let db = Database::open()?;
        let api = db.api()?;
        let word = Tokenizer::new(api, c"porter", &mut [c"unicode61".as_ptr()])?;
        let index = Tokenizer::new(
            api,
            c"porter",
            &mut [
                c"unicode61".as_ptr(),
                c"tokenchars".as_ptr(),
                c"_-".as_ptr(),
            ],
        )?;
        let exact = Tokenizer::new(
            api,
            c"unicode61",
            &mut [c"tokenchars".as_ptr(), c"_-".as_ptr()],
        )?;
        Ok(Self {
            word,
            index,
            exact,
            _db: db,
        })
    }
}
thread_local! {
    static TOKENIZERS: RefCell<Option<Tokenizers>> = const { RefCell::new(None) };
}
fn with_tokenizers<T>(f: impl FnOnce(&mut Tokenizers) -> Result<T>) -> Result<T> {
    TOKENIZERS.with(|cell| {
        let mut slot = cell.try_borrow_mut().map_err(|_| Error::Reentrant)?;
        if slot.is_none() {
            *slot = Some(Tokenizers::new()?);
        }
        f(slot.as_mut().expect("initialized tokenizers"))
    })
}

/// Lowercase one independent input and return its first lexicographic Porter stem.
/// An input without tokens returns its lowercased original bytes.
pub fn word(text: &str) -> Result<String> {
    word_lowercased(&text.to_lowercase())
}
/// The store already lowercases its cache key; avoid repeating that allocation.
pub(crate) fn word_lowercased(lower: &str) -> Result<String> {
    with_tokenizers(|t| {
        Ok(t.word
            .tokens(lower)?
            .into_iter()
            .next()
            .unwrap_or_else(|| lower.to_owned()))
    })
}
/// Unique sorted stems using the index's underscore/hyphen-preserving tokenizer.
pub fn index_stems(text: &str) -> Result<Vec<String>> {
    with_tokenizers(|t| t.index.tokens(text))
}

/// Exact FTS tokens in occurrence order, including repetitions. No stemming or
/// extra lowercasing: identical to the search index's Unicode61 configuration.
pub fn exact_tokens(text: &str) -> Result<Vec<String>> {
    with_tokenizers(|t| t.exact.occurrences(text))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn both_modes_match_the_sql_tokenizers() {
        let dir = tempfile::tempdir().unwrap();
        let database = libsql::Builder::new_local(dir.path().join("tokens.db"))
            .build()
            .await
            .unwrap();
        let db = database.connect().unwrap();
        db.execute_batch("CREATE VIRTUAL TABLE temp.words USING fts5(text,tokenize='porter unicode61');
            CREATE VIRTUAL TABLE temp.word_vocab USING fts5vocab(temp,words,'instance');
            CREATE VIRTUAL TABLE temp.index_words USING fts5(text,tokenize=\"porter unicode61 tokenchars '_-'\");
            CREATE VIRTUAL TABLE temp.index_vocab USING fts5vocab(temp,index_words,'row');
            CREATE VIRTUAL TABLE temp.exact_words USING fts5(text,tokenize=\"unicode61 tokenchars '_-'\");
            CREATE VIRTUAL TABLE temp.exact_vocab USING fts5vocab(temp,exact_words,'instance');").await.unwrap();
        let mut probes: Vec<String> = [
            "",
            "---",
            "foo-bar",
            "snake_case",
            "dogs cats",
            "Running running runs",
            "CamelCase",
            "résumés",
            "İstanbul",
            "Straße",
            "ὈΔΥΣΣΕΎΣ",
            "ABC\0DEF",
            "é\u{301}",
            "中文 emojis🙂",
            "𞤀𞤢",
            "a.b",
            "a\r\nb",
            "caresses ponies ties agreed disabled relational conditional",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        for prefix in [
            "a", "ae", "y", "oy", "by", "relat", "feed", "run", "happy", "é", "1", "_", "-",
        ] {
            for suffix in [
                "", "s", "sses", "ies", "eed", "ed", "ing", "ational", "izer", "fulness", "icate",
                "al", "ion", "e", "ll",
            ] {
                probes.push(format!("{prefix}{suffix}"));
            }
        }
        for length in [2, 3, 62, 63, 64, 65, 66, 128] {
            probes.push(format!("{}ing", "a".repeat(length)));
            probes.push(format!("{}ing", "é".repeat(length)));
        }
        for text in probes {
            db.execute("DELETE FROM exact_words", ()).await.unwrap();
            db.execute("INSERT INTO exact_words VALUES(?)", [text.as_str()])
                .await
                .unwrap();
            let mut rows = db
                .query("SELECT term FROM exact_vocab ORDER BY offset", ())
                .await
                .unwrap();
            let mut expected = Vec::new();
            while let Some(row) = rows.next().await.unwrap() {
                expected.push(row.get::<String>(0).unwrap());
            }
            assert_eq!(exact_tokens(&text).unwrap(), expected, "exact {text:?}");
            let lower = text.to_lowercase();
            db.execute("DELETE FROM words", ()).await.unwrap();
            db.execute("INSERT INTO words VALUES(?)", [lower.clone()])
                .await
                .unwrap();
            let mut rows = db
                .query("SELECT term FROM word_vocab LIMIT 1", ())
                .await
                .unwrap();
            let expected = rows
                .next()
                .await
                .unwrap()
                .map(|r| r.get::<String>(0).unwrap())
                .unwrap_or(lower);
            assert_eq!(word(&text).unwrap(), expected, "word {text:?}");
            db.execute("DELETE FROM index_words", ()).await.unwrap();
            db.execute("INSERT INTO index_words VALUES(?)", [text.clone()])
                .await
                .unwrap();
            let mut rows = db
                .query("SELECT term FROM index_vocab ORDER BY term", ())
                .await
                .unwrap();
            let mut expected = Vec::new();
            while let Some(row) = rows.next().await.unwrap() {
                expected.push(row.get::<String>(0).unwrap());
            }
            assert_eq!(index_stems(&text).unwrap(), expected, "index {text:?}");
        }
        assert_eq!(word("dogs cats").unwrap(), "cat");
        assert_eq!(word("foo-bar").unwrap(), "bar");
        assert_eq!(index_stems("foo-bar").unwrap(), vec!["foo-bar"]);
    }

    #[test]
    fn callbacks_and_reentrancy_report_errors() {
        assert!(matches!(input_len(usize::MAX), Err(Error::InputTooLong)));
        let mut output = Output::default();
        let invalid = [0xffu8];
        // SAFETY: valid output context and one initialized byte; UTF-8 failure is deliberate.
        let code = unsafe {
            collect_token(
                (&mut output as *mut Output).cast(),
                0,
                invalid.as_ptr().cast(),
                1,
                0,
                1,
            )
        };
        assert_eq!(code, ffi::SQLITE_ERROR);
        assert!(matches!(output.error, Some(Error::InvalidToken)));
        assert!(output.tokens.is_empty());
        TOKENIZERS.with(|cell| {
            let _borrow = cell.borrow_mut();
            assert!(matches!(word("running"), Err(Error::Reentrant)));
        });
        assert_eq!(word("running").unwrap(), "run");
    }

    #[test]
    fn each_thread_owns_and_releases_its_tokenizers() {
        let threads: Vec<_> = (0..4)
            .map(|_| {
                std::thread::spawn(|| {
                    for _ in 0..32 {
                        assert_eq!(word("Running").unwrap(), "run");
                        assert_eq!(
                            index_stems("foo-bar running").unwrap(),
                            vec!["foo-bar", "run"]
                        );
                    }
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
    }
}
