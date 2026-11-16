use crate::{
    error::{Error, Result},
    mesh::pairing::{Command, Output},
};
use clap::Args;
use std::io::{IsTerminal, Read, Write};

#[derive(Args)]
pub(super) struct PairArgs {
    #[arg(long, num_args = 0..=1, default_missing_value = "", conflicts_with = "cancel", help = "Join using a token argument, piped stdin, or an interactive prompt")]
    join: Option<String>,
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
    eprintln!();
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
    eprint!("Admitted. Start the first sync now? [Y/n] ");
    std::io::stderr().flush()?;
    let mut answer = String::new();
    std::io::stdin().lock().read_line(&mut answer)?;
    Ok(!matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "n" | "no"
    ))
}

#[derive(Default)]
pub(super) struct ProgressRenderer {
    last: Option<String>,
    active: bool,
}
impl ProgressRenderer {
    pub(super) fn render(&mut self, progress: &crate::mesh::status::Progress) {
        let line = match progress {
            crate::mesh::status::Progress::Admitted(_) => {
                "Joined the mesh. Starting the requested first sync.".into()
            }
            crate::mesh::status::Progress::Sync(job) => progress_line(job),
        };
        if self.last.as_ref() == Some(&line) {
            return;
        }
        if std::io::stderr().is_terminal() {
            eprint!("\r\x1b[2K{line}");
            let _ = std::io::stderr().flush();
            self.active = true;
        } else {
            eprintln!("{line}");
        }
        self.last = Some(line);
    }
    pub(super) fn finish(&mut self) {
        if self.active {
            eprint!("\r\x1b[2K");
            self.active = false;
        }
        self.last = None;
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
    format!(
        "{action}: {count}/{} peers (includes this peer); local changes: {}",
        r.cohort.len(),
        r.received
    )
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
            "Sync incomplete: collected from {}/{} peers; distributed to {}/{} peers (includes this peer).",
            report.collected.len(),
            report.cohort.len(),
            report.distributed.len(),
            report.cohort.len()
        )
    };
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
    println!("{}", summary(report));
    if !report.complete {
        for peer in &report.cohort {
            let collected = report.collected.contains(peer);
            let distributed = report.distributed.contains(peer);
            if !collected || !distributed {
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
            "Accepted partial progress is durable. Retry with linn sync when peers are available."
        );
    }
}

pub(super) fn render(output: Output, json: bool) -> Result<()> {
    if let Output::Changed {
        initial_sync_error: Some(error),
        ..
    } = &output
    {
        eprintln!("Admitted; initial sync failed: {error}. Retry with linn sync.");
    }
    if json {
        println!("{}", serde_json::to_string(&output)?);
        return Ok(());
    }
    match output {
        Output::Synced(report) => render_report(&report),
        Output::Invitation {
            token,
            invitation,
            expires,
            members,
        } => {
            eprintln!("Invitation {invitation}; expires at Unix time {expires}. Known members:");
            for member in members {
                eprintln!("  {member}");
            }
            println!("{token}");
        }
        Output::Status(status) => {
            if let Some(endpoint) = status.endpoint {
                println!("This store's peer ID: {endpoint}");
                if let Some(service) = status.service {
                    println!("Mesh service: {service}");
                }
                if let Some(head) = status.head {
                    println!("Membership revision: {head}");
                } else {
                    println!("Enrollment incomplete.");
                }
                if status.conflict {
                    println!("Conflicting signed histories; network authorization suspended.");
                }
                if status.pending_leave {
                    println!(
                        "Local transport disabled; remote revocation outstanding. Retry linn sync:leave when the service is available."
                    );
                } else if !status.enabled {
                    println!("Local transport disabled.");
                }
                println!(
                    "Local transport: {}; relay {}; local discovery {}.",
                    if status.transport.ready {
                        "ready"
                    } else {
                        "stopped"
                    },
                    if status.transport.relay_connected {
                        "connected"
                    } else {
                        "disconnected"
                    },
                    if status.transport.local_discovery {
                        "ready"
                    } else {
                        "unavailable"
                    }
                );
                if let Some(error) = status.transport.error {
                    eprintln!("Transport: {error}");
                }
                println!("Peers (IDs identify store instances, not machines):");
                for member in status.members {
                    let reachability = status
                        .reachability
                        .as_ref()
                        .and_then(|r| r.get(&member))
                        .map(String::as_str)
                        .unwrap_or("unknown (not checked)");
                    if member == endpoint {
                        println!("  {member} (this store)");
                    } else {
                        println!("  {member}: {reachability}");
                    }
                }
                if status.reachability.is_none() {
                    println!("Use linn sync:status --check to check peer transport reachability.");
                } else {
                    println!(
                        "Reachability is a transport snapshot; it does not guarantee a successful sync."
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
            initial_sync_error: _,
        } => {
            println!("{operation}");
            if let Some(head) = status.head {
                println!("Membership: {head}");
            }
            if let Some(report) = initial_sync {
                render(Output::Synced(*report), false)?;
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
        assert!(line.contains("local changes: 0"));
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
