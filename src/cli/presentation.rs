//! @ref LLP 0009#presentation-policy
use std::{borrow::Cow, ffi::OsString, io::IsTerminal, sync::OnceLock};

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Style(pub bool);
impl Style {
    fn paint<'a>(self, text: &'a str, code: &str) -> Cow<'a, str> {
        if self.0 && !text.is_empty() {
            format!("\x1b[{code}m{text}\x1b[0m").into()
        } else {
            text.into()
        }
    }
    pub fn title(self, text: &str) -> Cow<'_, str> {
        self.paint(text, "1")
    }
    pub fn accent(self, text: &str) -> Cow<'_, str> {
        self.paint(text, "35")
    }
    pub fn success(self, text: &str) -> Cow<'_, str> {
        self.paint(text, "32")
    }
    pub fn quiet(self, text: &str) -> Cow<'_, str> {
        self.paint(text, "2")
    }
    pub fn warning(self, text: &str) -> Cow<'_, str> {
        self.paint(text, "33")
    }
    pub fn error(self, text: &str) -> Cow<'_, str> {
        self.paint(text, "31")
    }
    pub fn labelled(self, text: &str) -> Cow<'_, str> {
        if !self.0 {
            return text.into();
        }
        let split = text.find(':').map_or(text.len(), |i| i + 1);
        format!("{}{}", self.title(&text[..split]), &text[split..]).into()
    }
    pub fn excerpt(self, text: &str) -> Cow<'_, str> {
        if self.0 {
            text.replace('\n', "\n    ").into()
        } else {
            text.into()
        }
    }
    pub fn gap(self) {
        if self.0 {
            println!();
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct Presentation {
    pub out: Style,
    pub err: Style,
}
impl Presentation {
    fn resolve(
        stdout: bool,
        stderr: bool,
        json: bool,
        env: impl Fn(&str) -> Option<OsString>,
    ) -> Self {
        let equals = |name: &str, value: &str| {
            env(name).is_some_and(|v| v.to_str().is_some_and(|v| v.eq_ignore_ascii_case(value)))
        };
        let disabled = json
            || env("NO_COLOR").is_some_and(|v| !v.is_empty())
            || equals("TERM", "dumb")
            || equals("TERM", "unknown")
            || equals("CI", "1")
            || equals("CI", "true")
            || ["CLICOLOR", "COLOR", "FORCE_COLOR"]
                .iter()
                .any(|name| equals(name, "0"));
        Self {
            out: Style(stdout && !disabled),
            err: Style(stderr && !disabled),
        }
    }
}

pub(super) fn current() -> &'static Presentation {
    static POLICY: OnceLock<Presentation> = OnceLock::new();
    POLICY.get_or_init(|| {
        // Stop at the option terminator: a literal query named --json is content.
        let json = std::env::args_os()
            .skip(1)
            .take_while(|a| a != "--")
            .any(|a| a == "--json");
        Presentation::resolve(
            std::io::stdout().is_terminal(),
            std::io::stderr().is_terminal(),
            json,
            |key| std::env::var_os(key),
        )
    })
}

// Generated progress consists only of fixed ASCII phase names and integer counts.
// Native width inspection does not send escape queries or read terminal input.
pub(super) fn stderr_columns() -> Option<usize> {
    #[cfg(unix)]
    {
        let mut size = std::mem::MaybeUninit::<libc::winsize>::uninit();
        if unsafe { libc::ioctl(libc::STDERR_FILENO, libc::TIOCGWINSZ, size.as_mut_ptr()) } == 0 {
            let columns = unsafe { size.assume_init() }.ws_col;
            return (columns > 1).then_some(usize::from(columns));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    fn policy(out: bool, err: bool, json: bool, vars: &[(&str, &str)]) -> Presentation {
        Presentation::resolve(out, err, json, |key| {
            vars.iter()
                .find(|(k, _)| *k == key)
                .map(|(_, v)| OsString::from(*v))
        })
    }
    #[test]
    fn disable_signals_win_over_force_settings_even_in_a_pty() {
        for signal in [
            ("NO_COLOR", "1"),
            ("NO_COLOR", "0"),
            ("TERM", "dumb"),
            ("TERM", "unknown"),
            ("CI", "1"),
            ("CI", "TRUE"),
            ("CLICOLOR", "0"),
            ("COLOR", "0"),
            ("FORCE_COLOR", "0"),
        ] {
            let p = policy(true, true, false, &[signal, ("CLICOLOR_FORCE", "1")]);
            assert!(!p.out.0 && !p.err.0, "{signal:?}");
        }
        let p = policy(true, true, true, &[("FORCE_COLOR", "3")]);
        assert!(!p.out.0 && !p.err.0);
        assert!(
            policy(true, true, false, &[("NO_COLOR", ""), ("CI", "false")])
                .out
                .0
        );
    }
    #[test]
    fn streams_are_independent_and_force_never_creates_a_terminal() {
        let p = policy(false, true, false, &[("FORCE_COLOR", "1")]);
        assert!(!p.out.0 && p.err.0);
        let p = policy(true, false, false, &[]);
        assert!(p.out.0 && !p.err.0);
        let p = policy(false, false, false, &[("CLICOLOR_FORCE", "1")]);
        assert!(!p.out.0 && !p.err.0);
    }
    #[test]
    fn plain_rendering_borrows_original_text_and_human_excerpts_keep_all_lines() {
        assert!(matches!(
            Style(false).title("label"),
            Cow::Borrowed("label")
        ));
        assert_eq!(Style(false).excerpt("one\ntwo"), "one\ntwo");
        assert_eq!(Style(true).excerpt("one\ntwo"), "one\n    two");
        assert_eq!(Style(true).title("label"), "\x1b[1mlabel\x1b[0m");
    }
}
