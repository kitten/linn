use crate::{
    error::{Error, Result},
    mesh::pairing::{Command, Output},
};
use clap::Args;
use std::io::{IsTerminal, Read, Write};

// Prompt input echo may go to a different terminal than stderr, or be disabled.
// End the prompt even on early errors before the caller prints a diagnostic.
struct PromptLine;
impl Drop for PromptLine {
    fn drop(&mut self) {
        let mut stderr = std::io::stderr().lock();
        let _ = writeln!(stderr);
        let _ = stderr.flush();
    }
}

#[derive(Args)]
pub(super) struct PairArgs {
    #[arg(long, num_args = 0..=1, default_missing_value = "", conflicts_with = "cancel", help = "Join using a token argument, piped stdin, or an interactive prompt")]
    join: Option<String>,
    #[arg(
        short = 'y',
        long,
        help = "Confirm whole-store invitation creation or redemption without prompting"
    )]
    yes: bool,
    #[arg(
        long,
        conflicts_with = "no_sync",
        help = "Run the first sync after successful admission"
    )]
    sync: bool,
    #[arg(long, help = "Skip the interactive first-sync offer")]
    no_sync: bool,
    #[arg(
        long,
        help = "Mesh service origin for a new mesh (default: https://linn.kitten.sh)"
    )]
    service: Option<String>,
    #[arg(
        long,
        help = "Cancel an unused invitation by its full ID",
        conflicts_with = "service"
    )]
    cancel: Option<String>,
}
impl PairArgs {
    pub(super) fn first_sync(&self) -> Option<bool> {
        if self.sync {
            Some(true)
        } else if self.no_sync || self.join.is_none() {
            Some(false)
        } else {
            None
        }
    }
    pub(super) fn cancel_context(&self) -> Option<String> {
        self.cancel
            .as_ref()
            .map(|id| format!("Cancelled invitation ID: {id}"))
    }
    pub(super) fn needs_confirmation(&self) -> bool {
        self.cancel.is_none()
    }
    pub(super) fn confirm(&self, enrolled: bool) -> Result<()> {
        if !self.needs_confirmation() {
            return Ok(());
        }
        eprintln!(
            "{}",
            super::presentation::current().err.warning(PAIR_NOTICE)
        );
        if enrolled {
            eprintln!(
                "This store already belongs to a mesh. Pairing can invite another peer or merge meshes. For a copied store that must run alongside its original, use linn sync:reset-identity first; it removes local enrollment while preserving records and deletions."
            );
        }
        if self.yes {
            return Ok(());
        }
        if !std::io::stdin().is_terminal() {
            return Err(Error::Invalid("pairing requires confirmation; pass -y to confirm when using piped input or running unattended".into()));
        }
        use std::io::BufRead;
        let prompt_line = PromptLine;
        eprint!("Continue with pairing? [y/N] ");
        std::io::stderr().flush()?;
        let mut answer = String::new();
        std::io::stdin().lock().read_line(&mut answer)?;
        drop(prompt_line);
        if matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            Ok(())
        } else {
            Err(Error::Invalid(
                "pairing cancelled; no invitation created or redeemed".into(),
            ))
        }
    }
    pub(super) fn resolve(self) -> Result<Command> {
        let value = self.join;
        let invitation = match value.as_deref() {
            Some("") if std::io::stdin().is_terminal() => {
                let token = prompt_invitation()?;
                Some(token.trim().to_owned())
            }
            Some("") | Some("-") => {
                let mut token = String::new();
                std::io::stdin()
                    .lock()
                    .take(32769)
                    .read_to_string(&mut token)?;
                if token.len() > 32768 {
                    return Err(Error::Invalid("invitation token is too large".into()));
                }
                Some(token.trim().to_owned())
            }
            _ => value,
        };
        let command = Command::Pair {
            service: self.service,
            invitation,
            cancel: self.cancel,
            initial_sync: self.sync,
        };
        command.validate()?;
        Ok(command)
    }
}

// Invitations can exceed the terminal's canonical line buffer. Read a bounded
// line without canonical buffering; keyboard interrupts return through the guard.
fn prompt_invitation() -> Result<String> {
    use std::os::fd::AsRawFd;
    struct Restore(i32, libc::termios);
    impl Drop for Restore {
        fn drop(&mut self) {
            unsafe {
                libc::tcsetattr(self.0, libc::TCSANOW, &self.1);
            }
        }
    }
    let stdin = std::io::stdin();
    let fd = stdin.as_raw_fd();
    let mut original = std::mem::MaybeUninit::uninit();
    if unsafe { libc::tcgetattr(fd, original.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let original = unsafe { original.assume_init() };
    let mut input = original;
    input.c_lflag &= !(libc::ICANON | libc::ECHO | libc::ISIG | libc::IEXTEN);
    input.c_cc[libc::VMIN] = 1;
    input.c_cc[libc::VTIME] = 0;
    if unsafe { libc::tcsetattr(fd, libc::TCSANOW, &input) } != 0 {
        return Err(std::io::Error::last_os_error().into());
    }
    let _restore = Restore(fd, original);
    let prompt_line = PromptLine;
    eprint!("Invitation: ");
    std::io::stderr().flush()?;
    let mut bytes = Vec::new();
    let mut reader = stdin.lock();
    loop {
        let mut byte = [0];
        reader.read_exact(&mut byte)?;
        match byte[0] {
            b'\n' | b'\r' => break,
            3 | 4 => return Err(Error::Invalid("invitation input cancelled".into())),
            8 | 127 => {
                bytes.pop();
            }
            value if value.is_ascii_graphic() || value == b' ' => {
                if bytes.len() == 32768 {
                    return Err(Error::Invalid("invitation token is too large".into()));
                }
                bytes.push(value);
            }
            _ => return Err(Error::Invalid("invalid invitation input".into())),
        }
    }
    drop(prompt_line);
    if bytes.is_empty() {
        return Err(Error::Invalid("empty invitation input".into()));
    }
    Ok(String::from_utf8(bytes).expect("ASCII input"))
}

pub(super) fn offer_sync() -> Result<bool> {
    use std::io::BufRead;
    if !std::io::stdin().is_terminal() {
        return Ok(false);
    }
    loop {
        let prompt_line = PromptLine;
        eprint!("This store joined the mesh. Start the first sync now? [Y/n] ");
        std::io::stderr().flush()?;
        let mut answer = String::new();
        let read = std::io::stdin().lock().read_line(&mut answer)?;
        drop(prompt_line);
        if read == 0 {
            return Ok(false);
        }
        match answer.trim().to_ascii_lowercase().as_str() {
            "" | "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => eprintln!("Enter yes or no."),
        }
    }
}

const PAIR_NOTICE: &str = "An invitation authorizes merging both entire stores and their meshes, including offline members. Share it only with a trusted peer. Pairing does not itself transfer records.";

// @ref LLP 0009#transient-progress
pub(super) struct ProgressRenderer {
    last: Option<String>,
    phase: Option<u8>,
    drawn: Option<std::time::Instant>,
    columns: Option<usize>,
    interactive: bool,
    active: bool,
    message: Option<(u8, String)>,
    frame: usize,
}
impl Default for ProgressRenderer {
    fn default() -> Self {
        Self::new(super::presentation::current().err.0)
    }
}
impl ProgressRenderer {
    fn new(interactive: bool) -> Self {
        Self {
            last: None,
            phase: None,
            drawn: None,
            columns: None,
            interactive,
            active: false,
            message: None,
            frame: 0,
        }
    }
    pub(super) fn render(&mut self, progress: &crate::mesh::status::Progress) {
        let (phase, line) = match progress {
            crate::mesh::status::Progress::Admitted(_) => (
                0,
                "Joined the mesh. Starting the requested first sync.".into(),
            ),
            crate::mesh::status::Progress::Sync(job) => (
                job.phase as u8 + 1,
                if self.interactive {
                    human_progress_line(job)
                } else {
                    progress_line(job)
                },
            ),
            crate::mesh::status::Progress::Checked(probe) => (
                self.phase.unwrap_or(0).wrapping_add(1),
                if self.interactive {
                    format!(
                        "{} ({})",
                        sentence_case(&probe.result.to_string()),
                        probe.peer
                    )
                } else {
                    format!("{}: {}", probe.peer, probe.result)
                },
            ),
        };
        self.message = Some((phase, line.clone()));
        let _ = self.draw(
            &line,
            phase,
            std::time::Instant::now(),
            if self.interactive {
                super::presentation::stderr_columns()
            } else {
                None
            },
            &mut std::io::stderr().lock(),
        );
    }
    pub(super) async fn execute(
        &mut self,
        path: &std::path::Path,
        op: crate::application::Operation,
        caller: crate::application::Caller,
        input: Option<std::fs::File>,
    ) -> Result<crate::application::Reply> {
        if !self.interactive || !matches!(&op, crate::application::Operation::Mesh(_)) {
            return crate::runtime::execute_with_progress(path, op, caller, input, |event| {
                self.render(event)
            })
            .await;
        }
        let renderer = std::cell::RefCell::new(self);
        let operation = crate::runtime::execute_with_progress(path, op, caller, input, |event| {
            renderer.borrow_mut().render(event);
        });
        tokio::pin!(operation);
        let mut ticks = tokio::time::interval(std::time::Duration::from_millis(200));
        ticks.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tokio::select! {
                result = &mut operation => return result,
                _ = ticks.tick() => renderer.borrow_mut().tick(),
            }
        }
    }
    fn tick(&mut self) {
        let _ = self.spin(
            std::time::Instant::now(),
            super::presentation::stderr_columns(),
            &mut std::io::stderr().lock(),
        );
    }
    fn spin(
        &mut self,
        now: std::time::Instant,
        width: Option<usize>,
        out: &mut impl Write,
    ) -> std::io::Result<()> {
        if !self.interactive {
            return Ok(());
        }
        let Some((phase, message)) = self.message.clone() else {
            return Ok(());
        };
        let Some(columns) = width else {
            return Ok(());
        };
        let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        let line = format!("{} {message}", frames[self.frame % frames.len()]);
        self.frame += 1;
        self.draw(&line, phase, now, Some(columns), out)
    }
    fn draw(
        &mut self,
        line: &str,
        phase: u8,
        now: std::time::Instant,
        width: Option<usize>,
        out: &mut impl Write,
    ) -> std::io::Result<()> {
        if self.active && self.columns != width {
            // Reflow may have moved the old status across rows. Do not erase a row
            // whose contents we can no longer identify; use line output from here.
            writeln!(out)?;
            out.flush()?;
            self.active = false;
            self.interactive = false;
        }
        if self.last.as_deref() == Some(line) {
            return Ok(());
        }
        let columns = width.filter(|w| *w > 1);
        if self.interactive
            && let Some(columns) = columns
        {
            if self.phase == Some(phase)
                && self.drawn.is_some_and(|last| {
                    now.duration_since(last) < std::time::Duration::from_millis(200)
                })
            {
                return Ok(());
            }
            // The Braille frame occupies one column, like each ASCII message character.
            // Reserve the final column and slice only at UTF-8 boundaries.
            let end = line
                .char_indices()
                .nth(columns - 1)
                .map_or(line.len(), |(i, _)| i);
            let visible = &line[..end];
            write!(out, "\r\x1b[2K{visible}")?;
            self.active = true;
            self.columns = Some(columns);
            self.drawn = Some(now);
        } else {
            writeln!(out, "{line}")?;
        }
        out.flush()?;
        self.last = Some(line.into());
        self.phase = Some(phase);
        Ok(())
    }
    fn clear(&mut self, width: Option<usize>, out: &mut impl Write) -> std::io::Result<()> {
        if self.active {
            if self.columns == width {
                write!(out, "\r\x1b[2K")?;
            } else {
                writeln!(out)?;
            }
            out.flush()?;
            self.active = false;
        }
        self.last = None;
        self.message = None;
        self.phase = None;
        self.drawn = None;
        Ok(())
    }
    pub(super) fn finish(&mut self) {
        let width = if self.active {
            super::presentation::stderr_columns()
        } else {
            None
        };
        let _ = self.clear(width, &mut std::io::stderr().lock());
    }
}
impl Drop for ProgressRenderer {
    fn drop(&mut self) {
        self.finish();
    }
}
fn phase_name(phase: crate::mesh::jobs::Phase) -> &'static str {
    use crate::mesh::jobs::Phase::*;
    match phase {
        Collecting => "Collecting from peers",
        Collected => "Collection finished",
        Distributing => "Distributing to peers",
        Complete => "Finishing sync",
        Failed => "Sync failed",
        Cancelled => "Sync cancelled",
    }
}
fn progress_line(job: &crate::mesh::status::Job) -> String {
    use crate::mesh::jobs::Phase;
    let r = &job.report;
    let (action, count) = match job.phase {
        Phase::Collecting => ("Collecting", r.collected.len()),
        Phase::Collected => ("Collected", r.collected.len()),
        Phase::Distributing => ("Distributing", r.distributed.len()),
        Phase::Complete => ("Finishing sync", r.distributed.len()),
        Phase::Failed => ("Sync failed", r.distributed.len()),
        Phase::Cancelled => ("Sync cancelled", r.distributed.len()),
    };
    let traversal = r.traversal.as_ref().map_or(String::new(), |state| {
        format!(
            "; round {}, L{}, direct {}, {} reached, {} contacts pending{}",
            state.round,
            state.level,
            if state.direct_complete {
                "complete"
            } else {
                "pending"
            },
            state.reached,
            state.pending.len(),
            if state.discovery_expired {
                "; discovery window expired"
            } else {
                ""
            }
        )
    });
    format!(
        "{action}: {count}/{} peers (includes this peer); changes applied here: {}{traversal}",
        r.cohort.len(),
        r.received
    )
}

fn human_progress_line(job: &crate::mesh::status::Job) -> String {
    use crate::mesh::jobs::Phase;
    let r = &job.report;
    let action = match job.phase {
        Phase::Collecting => "Receiving changes",
        Phase::Collected => "Received changes",
        Phase::Distributing => "Sharing changes",
        Phase::Complete => "Finishing sync",
        Phase::Failed => "Sync failed",
        Phase::Cancelled => "Sync cancelled",
    };
    format!(
        "{action} (changes: {} sent / {} received)",
        r.sent, r.received
    )
}

fn human_summary(report: &crate::mesh::jobs::Report) -> String {
    let outcome = if report.cohort.len() <= 1 && report.complete {
        "Sync complete\nNo paired stores to sync with.".into()
    } else if report.complete {
        format!(
            "Sync complete\n{} synced.",
            counted(report.cohort.len().saturating_sub(1), "peer", "peers")
        )
    } else {
        format!(
            "Sync {}\nReceived from {}/{} peers; shared with {}/{} peers.",
            if report.unavailable_only() {
                "partial"
            } else {
                "incomplete"
            },
            report.collected.len().saturating_sub(1),
            report.cohort.len().saturating_sub(1),
            report.distributed.len().saturating_sub(1),
            report.cohort.len().saturating_sub(1)
        )
    };
    let changes = format!(
        "{} {} applied, {} {} sent.",
        report.received,
        if report.received == 1 {
            "change"
        } else {
            "changes"
        },
        report.sent,
        if report.sent == 1 {
            "change"
        } else {
            "changes"
        }
    );
    let mut text = format!("{outcome}\n{changes}");
    if !report.complete
        && report
            .traversal
            .as_ref()
            .is_some_and(|s| s.discovery_expired)
    {
        text.push_str("\nStopped looking for more stores; transfers already started kept running.");
    }
    if let Some(state) = &report.traversal {
        for (source, target) in &state.no_known_direct_routes {
            text.push_str(&format!(
                "\nNo known direct connection between peers (from {source} to {target})"
            ));
        }
    }
    text
}

fn counted(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

fn summary(report: &crate::mesh::jobs::Report) -> String {
    let mut text = if report.cohort.is_empty() {
        "Sync complete: no enrolled peers to sync with.".into()
    } else if report.complete {
        format!(
            "Sync complete across {}, including this peer.",
            counted(report.cohort.len(), "peer", "peers")
        )
    } else {
        format!(
            "Sync {}: collected from {}/{} peers; distributed to {}/{} peers (includes this peer).",
            if report.unavailable_only() {
                "partial"
            } else {
                "incomplete"
            },
            report.collected.len(),
            report.cohort.len(),
            report.distributed.len(),
            report.cohort.len()
        )
    };
    if !report.complete
        && report
            .traversal
            .as_ref()
            .is_some_and(|state| state.discovery_expired)
    {
        text.push_str(
            "\nThe discovery window expired; admitted work retained its progress deadlines.",
        );
    }
    if let Some(state) = &report.traversal {
        for (source, target) in &state.no_known_direct_routes {
            text.push_str(&format!(
                "\nPeer {source}: no known direct route to peer {target}."
            ));
        }
    }
    if report.received == 0 {
        text.push_str("\nNo changes applied to this store.");
    } else {
        text.push_str(&format!(
            "\n{} {} applied to this store (records and deletions).",
            report.received,
            if report.received == 1 {
                "change"
            } else {
                "changes"
            }
        ));
    }
    text
}
fn render_report(report: &crate::mesh::jobs::Report) {
    let style = super::presentation::current().out;
    let text = if style.0 {
        human_summary(report)
    } else {
        summary(report)
    };
    let complete = report.complete || report.cohort.is_empty();
    let label = if complete {
        "Sync complete"
    } else if report.unavailable_only() {
        "Sync partial"
    } else {
        "Sync incomplete"
    };
    println!(
        "{}{}",
        if complete {
            style.success(label)
        } else if report.unavailable_only() {
            style.warning(label)
        } else {
            style.error(label)
        },
        &text[label.len()..]
    );
    if !report.complete {
        for peer in &report.cohort {
            let collected = report.collected.contains(peer);
            let distributed = report.distributed.contains(peer);
            if (!collected || !distributed)
                && (!style.0 || !report.failures.iter().any(|f| &f.peer == peer))
            {
                if style.0 {
                    eprintln!(
                        "Sync unfinished: receiving {}; sharing {} ({peer})",
                        if collected { "complete" } else { "incomplete" },
                        if distributed {
                            "complete"
                        } else {
                            "incomplete"
                        }
                    );
                    continue;
                }
                eprintln!(
                    "Peer ID {peer}: collection {}; distribution {}.",
                    if collected {
                        "complete"
                    } else {
                        "not completed"
                    },
                    if distributed {
                        "complete"
                    } else {
                        "not completed"
                    }
                );
            }
        }
        for failure in &report.failures {
            if style.0 {
                let err = super::presentation::current().err;
                eprintln!("{}", styled_failure(err, failure));
                continue;
            }
            let kind = serde_json::to_value(failure.kind).expect("failure kind serializes");
            eprintln!(
                "Peer ID {} — {} [{}]: {}",
                failure.peer,
                phase_name(failure.phase),
                kind.as_str().unwrap(),
                failure.message
            );
            use crate::mesh::failure::Kind;
            match failure.kind {
                Kind::Publication => {
                    eprintln!("Repair publication with linn index:rebuild before retrying.")
                }
                Kind::Version => {
                    eprintln!("Use compatible Linn versions on both peers before retrying.")
                }
                Kind::Authentication | Kind::Conflict => eprintln!(
                    "Inspect enrollment with linn sync:status; resolve the authorization or history conflict before retrying."
                ),
                _ => (),
            }
        }
        eprintln!(
            "{}",
            if style.0 {
                "Changes kept. Retry with linn sync when peers are available."
            } else {
                "Accepted partial progress is durable. Retry with linn sync when peers are available."
            }
        );
    }
}

fn human_reachability(
    style: super::presentation::Style,
    state: Option<&crate::mesh::status::Reachability>,
) -> String {
    use crate::mesh::{failure::Kind, status::Reachability};
    let text = sentence_case(
        &state
            .map(ToString::to_string)
            .unwrap_or_else(|| "unknown (not checked)".into()),
    );
    match state {
        Some(Reachability::Reachable) => style.success(&text).into_owned(),
        Some(Reachability::Failed { kind, .. }) if *kind != Kind::Offline => {
            style.error(&text).into_owned()
        }
        _ => style.warning(&text).into_owned(),
    }
}

fn sentence_case(text: &str) -> String {
    let mut chars = text.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().chain(chars).collect(),
        None => String::new(),
    }
}

fn human_failure(failure: &crate::mesh::jobs::Failure) -> String {
    use crate::mesh::failure::Kind;
    let reason = if failure.kind == Kind::Offline {
        "unreachable".to_owned()
    } else {
        failure
            .message
            .replace(['\n', '\r'], " ")
            .trim_end_matches('.')
            .to_owned()
    };
    let recovery = match failure.kind {
        Kind::Publication => " Run linn index:rebuild before retrying.",
        Kind::Version => " Use compatible Linn versions on both peers.",
        Kind::Authentication | Kind::Conflict => " Check enrollment with linn sync:status.",
        _ => "",
    };
    if failure.kind == Kind::Offline {
        format!("Unreachable peer ({})", failure.peer)
    } else {
        format!("{}.{recovery} ({})", sentence_case(&reason), failure.peer)
    }
}

fn styled_failure(
    style: super::presentation::Style,
    failure: &crate::mesh::jobs::Failure,
) -> String {
    let text = human_failure(failure);
    let suffix = format!(" ({})", failure.peer);
    let message = &text[..text.len() - suffix.len()];
    let message = if failure.kind == crate::mesh::failure::Kind::Offline {
        style.warning(message)
    } else {
        style.error(message)
    };
    format!("{message} ({})", style.quiet(&failure.peer))
}

fn transport_text(
    style: super::presentation::Style,
    transport: &crate::mesh::status::Transport,
) -> String {
    let ready = if transport.ready { "ready" } else { "stopped" };
    let relay = if transport.relay_connected {
        "connected"
    } else {
        "disconnected"
    };
    let discovery = if transport.local_discovery {
        "ready"
    } else {
        "unavailable"
    };
    if style.0 {
        format!(
            "{} {ready}\n{} {relay}\n{} {discovery}",
            style.title("Local transport:"),
            style.title("Relay:"),
            style.title("Local discovery:")
        )
    } else {
        format!("Local transport: {ready}; relay {relay}; local discovery {discovery}.")
    }
}

fn peer_text(
    style: super::presentation::Style,
    member: &str,
    reachability: &str,
    local: bool,
) -> String {
    if style.0 {
        format!(
            "  {} ({})",
            if local {
                style.title("This store").into_owned()
            } else {
                sentence_case(reachability)
            },
            style.quiet(member)
        )
    } else if local {
        format!("  {member} (this store)")
    } else {
        format!("  {member}: {reachability}")
    }
}

pub(super) fn render(output: Output, json: bool) -> Result<()> {
    if let Output::Changed {
        initial_sync_error: Some(error),
        ..
    } = &output
    {
        eprintln!("Joined the mesh; initial sync failed: {error}. Retry with linn sync.");
    }
    if json {
        println!("{}", serde_json::to_string(&output)?);
        return Ok(());
    }
    let style = super::presentation::current().out;
    match output {
        Output::Synced(report) => render_report(&report),
        Output::Invitation {
            token,
            invitation,
            expires,
            members,
        } => {
            eprintln!(
                "{}",
                super::presentation::current().err.warning(PAIR_NOTICE)
            );
            let expiry = i64::try_from(expires)
                .ok()
                .and_then(|t| time::OffsetDateTime::from_unix_timestamp(t).ok())
                .and_then(|t| {
                    t.format(&time::format_description::well_known::Rfc3339)
                        .ok()
                })
                .unwrap_or_else(|| format!("Unix time {expires}"));
            let err = super::presentation::current().err;
            eprintln!(
                "{} {invitation}\n{} {expiry} (single use)\n{}",
                err.title(if err.0 {
                    "Invitation:"
                } else {
                    "Invitation ID:"
                }),
                err.title("Expires:"),
                err.title(if err.0 {
                    "Known peers:"
                } else {
                    "Known peer IDs (store instances, not machines):"
                })
            );
            for member in members {
                if err.0 {
                    eprintln!("  Known peer ({member})");
                } else {
                    eprintln!("  {member}");
                }
            }
            eprintln!("{}", err.title("On the other installation, run linn sync:pair --join and paste the invitation token."));
            if super::presentation::current().out.0 {
                println!(
                    "\n{}",
                    super::presentation::current()
                        .out
                        .title("Invitation token:")
                );
            }
            println!("{token}");
        }
        Output::Status(status) => {
            if let Some(endpoint) = status.endpoint {
                println!(
                    "{}",
                    if style.0 {
                        format!("This store ({})", style.quiet(&endpoint))
                    } else {
                        format!("This store's peer ID: {endpoint}")
                    }
                );
                if let Some(service) = status.service {
                    println!("{} {service}", style.title("Mesh service:"));
                }
                if let Some(head) = status.head {
                    if !style.0 {
                        println!("Membership revision: {head}");
                    }
                } else {
                    println!(
                        "{}",
                        style.warning(if style.0 {
                            "Pairing incomplete."
                        } else {
                            "Enrollment incomplete."
                        })
                    );
                }
                if status.conflict {
                    println!(
                        "{}",
                        style.error(if style.0 {
                            "Sync blocked: membership histories conflict."
                        } else {
                            "Conflicting signed histories; network authorization suspended."
                        })
                    );
                }
                if status.pending_leave {
                    println!(
                        "{}", style.warning(if style.0 { "Leaving paused: sync is disabled locally. Retry with linn sync:leave when the service is available." } else { "Local transport disabled; remote revocation outstanding. Retry linn sync:leave when the service is available." })
                    );
                } else if !status.enabled {
                    println!(
                        "{}",
                        style.warning(if style.0 {
                            "Sync disabled on this store."
                        } else {
                            "Local transport disabled."
                        })
                    );
                }
                println!("{}", transport_text(style, &status.transport));
                if let Some(error) = status.transport.error {
                    eprintln!(
                        "{}",
                        super::presentation::current().err.error(&format!(
                            "{}: {error}",
                            if super::presentation::current().err.0 {
                                "Connection error"
                            } else {
                                "Transport"
                            }
                        ))
                    );
                }
                println!(
                    "{}",
                    style.labelled(if style.0 {
                        "Peers:"
                    } else {
                        "Peers (IDs identify store instances, not machines):"
                    })
                );
                style.gap();
                for member in status.members {
                    let state = status.reachability.as_ref().and_then(|r| r.get(&member));
                    let reachability = if style.0 {
                        human_reachability(style, state)
                    } else {
                        state
                            .map(ToString::to_string)
                            .unwrap_or_else(|| "unknown (not checked)".into())
                    };
                    println!(
                        "{}",
                        peer_text(style, &member, &reachability, member == endpoint)
                    );
                }
                if status.reachability.is_none() {
                    println!(
                        "{}",
                        style.quiet(if style.0 {
                            "Check connections: linn sync:status --check"
                        } else {
                            "Use linn sync:status --check to check peer transport reachability."
                        })
                    );
                } else {
                    println!(
                        "{}", style.quiet(if style.0 { "Connections checked; a successful sync is not guaranteed." } else { "Reachability is a transport snapshot; it does not guarantee a successful sync." })
                    );
                }
            } else {
                println!("This store has not joined a mesh. Pair with linn sync:pair.");
            }
        }
        Output::Changed {
            operation,
            status,
            initial_sync,
            initial_sync_error,
        } => {
            use crate::mesh::pairing::Change;
            match operation {
                Change::Paired => {
                    println!("{}", style.success(if style.0 { "Pairing complete" } else { "This store joined the mesh." }));
                    if initial_sync.is_none() && initial_sync_error.is_none() {
                        println!("Run linn sync to exchange records and deletions.");
                    }
                }
                Change::Revoked => println!("{}", style.success(if style.0 { "Peer removed. Offline peers will learn this when they reconnect." } else { "Peer revoked from the mesh. Offline peers learn the revocation when they reconnect." })),
                Change::Left => {
                    println!(
                        "{}",
                        style.title(if style.0 { "Sync disabled on this store." } else { "This store's peer transport is disabled." })
                    );
                    if status.pending_leave {
                        println!(
                            "{}", style.warning(if style.0 { "Leaving paused. Retry with linn sync:leave when the service is available." } else { "Remote revocation is outstanding. Retry linn sync:leave when the service is available." })
                        );
                    } else {
                        println!(
                            "{}",
                            style.success(if style.0 { "Left the mesh." } else { "This store's peer ID has been revoked from the mesh." })
                        );
                    }
                }
                Change::Cancelled => println!("{}", style.success("Unused invitation cancelled.")),
                Change::IdentityReset => {
                    if style.0 { println!("{}\nRecords and deletions kept. Pair again with linn sync:pair.", style.success("Identity reset")); }
                    else { println!("This store's identity and local enrollment were removed. Records and deletions were preserved. Enroll as a new peer with linn sync:pair."); }
                },
            }
            if let Some(endpoint) = status.endpoint {
                println!(
                    "{}",
                    if style.0 {
                        format!("This store ({})", style.quiet(&endpoint))
                    } else {
                        format!("This store's peer ID: {endpoint}")
                    }
                );
            }
            if let Some(head) = status.head
                && !style.0
            {
                println!("Membership revision: {head}");
            }
            if let Some(report) = initial_sync {
                render_report(&report);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mesh::{
        jobs::{Phase, Report},
        status::Job,
    };

    #[test]
    fn human_sync_reports_local_changes_without_claiming_global_freshness() {
        let mut report = Report {
            cohort: vec!["local".into(), "remote".into()],
            complete: true,
            ..Default::default()
        };
        assert_eq!(
            human_summary(&report),
            "Sync complete\n1 peer synced.\n0 changes applied, 0 changes sent."
        );
        report.complete = false;
        report.received = 1;
        let text = human_summary(&report);
        assert!(text.starts_with("Sync incomplete\n"));
        assert!(text.contains("1 change applied, 0 changes sent."));
        assert!(!text.contains("up to date"));
    }

    #[test]
    fn human_failures_are_single_line_and_keep_recovery_advice() {
        let mut failure = crate::mesh::jobs::Failure {
            peer: "remote".into(),
            phase: Phase::Collecting,
            kind: crate::mesh::failure::Kind::Offline,
            message: "protocol details".into(),
        };
        assert_eq!(human_failure(&failure), "Unreachable peer (remote)");
        failure.kind = crate::mesh::failure::Kind::Publication;
        failure.message = "publication\nfailed.".into();
        assert_eq!(
            human_failure(&failure),
            "Publication failed. Run linn index:rebuild before retrying. (remote)"
        );
    }

    #[test]
    fn human_progress_keeps_transfer_counters_in_every_phase() {
        let mut job = Job {
            id: String::new(),
            phase: Phase::Collecting,
            report: Report {
                sent: 6,
                received: 5,
                ..Default::default()
            },
        };
        assert_eq!(
            human_progress_line(&job),
            "Receiving changes (changes: 6 sent / 5 received)"
        );
        job.phase = Phase::Complete;
        assert_eq!(
            human_progress_line(&job),
            "Finishing sync (changes: 6 sent / 5 received)"
        );
    }

    #[test]
    fn peer_colors_follow_semantics_and_keep_ids_dim() {
        use super::super::presentation::Style;
        use crate::mesh::{failure::Kind, status::Reachability};
        assert_eq!(
            human_reachability(Style(true), Some(&Reachability::Reachable)),
            "\x1b[32mReachable (transport only)\x1b[0m"
        );
        assert_eq!(
            human_reachability(Style(true), None),
            "\x1b[33mUnknown (not checked)\x1b[0m"
        );
        let failure = Reachability::Failed {
            kind: Kind::Authentication,
            message: "not authorized".into(),
        };
        assert_eq!(
            human_reachability(Style(true), Some(&failure)),
            "\x1b[31mNot authorized\x1b[0m"
        );
        assert_eq!(
            human_reachability(Style(false), Some(&failure)),
            "Not authorized"
        );
        let failure = crate::mesh::jobs::Failure {
            peer: "Ab09".into(),
            phase: Phase::Collecting,
            kind: Kind::Offline,
            message: String::new(),
        };
        assert_eq!(
            styled_failure(Style(true), &failure),
            "\x1b[33mUnreachable peer\x1b[0m (\x1b[2mAb09\x1b[0m)"
        );
        assert_eq!(
            styled_failure(Style(false), &failure),
            "Unreachable peer (Ab09)"
        );
    }

    #[test]
    fn human_status_uses_sentence_case_without_changing_identifiers() {
        assert_eq!(
            sentence_case("reachable (transport only)"),
            "Reachable (transport only)"
        );
        assert_eq!(
            sentence_case("unknown: TLS handshake failed"),
            "Unknown: TLS handshake failed"
        );
        assert_eq!(sentence_case(""), "");
        let id = "aB09";
        let text = peer_text(super::super::presentation::Style(true), id, "", true);
        assert!(text.contains("This store"));
        assert!(text.ends_with("aB09\x1b[0m)"));
    }

    #[test]
    fn human_status_prioritizes_state_while_plain_status_keeps_its_format() {
        use super::super::presentation::Style;
        let transport = crate::mesh::status::Transport {
            ready: true,
            relay_connected: false,
            local_discovery: true,
            ..Default::default()
        };
        assert_eq!(
            super::transport_text(Style(false), &transport),
            "Local transport: ready; relay disconnected; local discovery ready."
        );
        let human = super::transport_text(Style(true), &transport)
            .replace("\x1b[1m", "")
            .replace("\x1b[0m", "");
        assert_eq!(
            human,
            "Local transport: ready\nRelay: disconnected\nLocal discovery: ready"
        );
        let id = "a".repeat(64);
        assert_eq!(
            super::peer_text(Style(false), &id, "unknown (not checked)", false),
            format!("  {id}: unknown (not checked)")
        );
        assert_eq!(
            super::peer_text(Style(false), &id, "", true),
            format!("  {id} (this store)")
        );
        let human = super::peer_text(Style(true), &id, "unknown (not checked)", false)
            .replace("\x1b[1m", "")
            .replace("\x1b[2m", "")
            .replace("\x1b[0m", "");
        assert_eq!(human, format!("  Unknown (not checked) ({id})"));
    }
    #[test]
    fn peer_phase_counts_do_not_imply_record_changes() {
        let report = Report {
            cohort: vec!["local".into(), "remote".into()],
            collected: vec!["local".into(), "remote".into()],
            distributed: vec!["local".into(), "remote".into()],
            complete: true,
            ..Default::default()
        };
        let text = summary(&report);
        assert!(text.contains("across 2 peers"));
        assert!(text.contains("No changes applied to this store"));
        assert!(!text.contains("up to date"));
        let line = progress_line(&Job {
            id: String::new(),
            phase: Phase::Collecting,
            report,
        });
        assert!(line.contains("Collecting: 2/2 peers"));
        assert!(line.contains("includes this peer"));
        assert!(line.contains("changes applied here: 0"));
    }

    #[test]
    fn incomplete_and_unpaired_summaries_are_distinct() {
        let mut report = Report {
            complete: true,
            ..Default::default()
        };
        assert!(summary(&report).contains("no enrolled peers"));
        report.cohort = vec!["local".into(), "offline".into()];
        report.collected = vec!["local".into()];
        report.complete = false;
        report.received = 3;
        let text = summary(&report);
        assert!(text.contains("Sync incomplete"));
        assert!(text.contains("1/2 peers"));
        assert!(text.contains("3 changes applied to this store (records and deletions)"));
    }
}

#[cfg(test)]
mod progress_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[derive(Default)]
    struct Screen {
        row: Vec<u8>,
        column: usize,
        completed: Vec<String>,
        flushes: usize,
    }
    impl Write for Screen {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            let mut i = 0;
            while i < bytes.len() {
                if bytes[i..].starts_with(b"\x1b[2K") {
                    self.row.clear();
                    i += 4;
                    continue;
                }
                match bytes[i] {
                    b'\r' => self.column = 0,
                    b'\n' => {
                        self.completed
                            .push(String::from_utf8(self.row.clone()).unwrap());
                        self.row.clear();
                        self.column = 0;
                    }
                    byte => {
                        self.row.resize(self.row.len().max(self.column + 1), b' ');
                        self.row[self.column] = byte;
                        self.column += 1;
                    }
                }
                i += 1;
            }
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }
    #[test]
    fn spinner_advances_without_events_and_stops_after_finish() {
        let now = Instant::now();
        let mut renderer = ProgressRenderer::new(true);
        renderer.message = Some((1, "Receiving changes (changes: 6 sent / 5 received)".into()));
        let mut screen = Screen::default();
        renderer.spin(now, Some(80), &mut screen).unwrap();
        assert!(screen.row.starts_with("⠋ Receiving changes".as_bytes()));
        renderer
            .spin(now + Duration::from_millis(200), Some(80), &mut screen)
            .unwrap();
        assert!(screen.row.starts_with("⠙ Receiving changes".as_bytes()));
        renderer.clear(Some(80), &mut screen).unwrap();
        renderer
            .spin(now + Duration::from_secs(1), Some(80), &mut screen)
            .unwrap();
        assert!(screen.row.is_empty());
        for (interactive, width) in [(false, Some(80)), (true, None)] {
            let mut renderer = ProgressRenderer::new(interactive);
            renderer.message = Some((1, "Waiting".into()));
            let mut output = Vec::new();
            renderer.spin(now, width, &mut output).unwrap();
            assert!(output.is_empty());
        }
    }

    #[test]
    fn braille_spinner_cycles_and_fits_a_narrow_terminal() {
        let mut renderer = ProgressRenderer::new(true);
        renderer.message = Some((1, "Waiting".into()));
        let mut output = Vec::new();
        let now = Instant::now();
        for (index, frame) in ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "⠋"]
            .iter()
            .enumerate()
        {
            output.clear();
            renderer
                .spin(
                    now + Duration::from_millis(index as u64 * 200),
                    Some(2),
                    &mut output,
                )
                .unwrap();
            assert_eq!(
                String::from_utf8(output.clone()).unwrap(),
                format!("\r\x1b[2K{frame}")
            );
        }
    }

    #[test]
    fn shorter_progress_and_completion_do_not_leave_suffixes_or_erase_results() {
        let mut renderer = ProgressRenderer::new(true);
        let mut screen = Screen::default();
        let now = Instant::now();
        renderer
            .draw("Collecting: 12/20 peers", 1, now, Some(80), &mut screen)
            .unwrap();
        renderer
            .draw("Done", 2, now, Some(80), &mut screen)
            .unwrap();
        assert_eq!(screen.row, b"Done");
        renderer.clear(Some(80), &mut screen).unwrap();
        assert!(screen.row.is_empty());
        assert_eq!(screen.column, 0);
        assert_eq!(screen.flushes, 3);
        writeln!(screen, "Result\nWarning").unwrap();
        renderer.clear(Some(80), &mut screen).unwrap();
        assert_eq!(screen.completed, ["Result", "Warning"]);
        assert_eq!(screen.flushes, 3);
    }
    #[test]
    fn updates_are_coalesced_but_phase_changes_are_immediate() {
        let mut renderer = ProgressRenderer::new(true);
        let mut screen = Screen::default();
        let now = Instant::now();
        renderer.draw("one", 1, now, Some(40), &mut screen).unwrap();
        renderer
            .draw(
                "two",
                1,
                now + Duration::from_millis(50),
                Some(40),
                &mut screen,
            )
            .unwrap();
        assert_eq!(screen.row, b"one");
        renderer
            .draw(
                "three",
                1,
                now + Duration::from_millis(200),
                Some(40),
                &mut screen,
            )
            .unwrap();
        assert_eq!(screen.row, b"three");
        renderer
            .draw(
                "final",
                2,
                now + Duration::from_millis(201),
                Some(40),
                &mut screen,
            )
            .unwrap();
        assert_eq!(screen.row, b"final");
        renderer.clear(Some(40), &mut screen).unwrap();
    }
    #[test]
    fn identical_status_still_observes_terminal_resize() {
        let mut renderer = ProgressRenderer::new(true);
        let mut screen = Screen::default();
        let now = Instant::now();
        renderer
            .draw("Collecting", 1, now, Some(80), &mut screen)
            .unwrap();
        renderer
            .draw("Collecting", 1, now, Some(20), &mut screen)
            .unwrap();
        assert!(!renderer.active && !renderer.interactive);
        assert_eq!(screen.completed, ["Collecting"]);
        renderer.clear(Some(20), &mut screen).unwrap();
        assert_eq!(screen.completed, ["Collecting"]);
    }
    #[test]
    fn narrow_unknown_and_resized_terminals_never_erase_unowned_rows() {
        let now = Instant::now();
        let mut renderer = ProgressRenderer::new(true);
        let mut screen = Screen::default();
        renderer
            .draw("Collecting peers", 1, now, Some(8), &mut screen)
            .unwrap();
        assert_eq!(screen.row.len(), 7);
        renderer
            .draw("Distributing", 2, now, Some(4), &mut screen)
            .unwrap();
        assert!(!renderer.active && !renderer.interactive);
        assert_eq!(screen.completed, ["Collect", "Distributing"]);
        renderer.clear(Some(4), &mut screen).unwrap();
        for width in [None, Some(0), Some(1)] {
            let mut renderer = ProgressRenderer::new(true);
            let mut output = Vec::new();
            renderer.draw("status", 1, now, width, &mut output).unwrap();
            renderer.clear(width, &mut output).unwrap();
            assert_eq!(output, b"status\n");
        }
        let mut renderer = ProgressRenderer::new(false);
        let mut output = Vec::new();
        renderer
            .draw("status", 1, now, Some(80), &mut output)
            .unwrap();
        renderer
            .draw("status", 1, now, Some(80), &mut output)
            .unwrap();
        renderer.clear(Some(80), &mut output).unwrap();
        assert_eq!(output, b"status\n");
    }
}
