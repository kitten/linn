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

pub(super) fn render_progress(progress: &crate::mesh::status::Progress) {
    match progress {
        crate::mesh::status::Progress::Admitted(_) => {
            eprintln!("Admitted. Starting the requested first sync.")
        }
        crate::mesh::status::Progress::Sync(job) => eprintln!(
            "Sync {:?}: {} collected, {} distributed; {} received here",
            job.phase,
            job.report.collected.len(),
            job.report.distributed.len(),
            job.report.received
        ),
    }
}

pub(super) fn render(output: Output, json: bool) -> Result<()> {
    let report = match &output {
        Output::Synced(report) => Some(report),
        Output::Changed {
            initial_sync: Some(report),
            ..
        } => Some(report.as_ref()),
        _ => None,
    };
    if let Some(report) = report
        && (json || matches!(output, Output::Synced(_)))
    {
        eprintln!(
            "Sync {}: {} collected, {} distributed; {} received here",
            if report.complete {
                "complete"
            } else {
                "incomplete"
            },
            report.collected.len(),
            report.distributed.len(),
            report.received
        );
    }
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
        Output::Synced(report) => {
            println!(
                "Mesh sync: {} collected, {} distributed; {} received here.",
                report.collected.len(),
                report.distributed.len(),
                report.received
            );
            for failure in report.failures {
                eprintln!("{} {:?}: {}", failure.peer, failure.phase, failure.message);
            }
        }
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
                println!("Peer: {endpoint}");
                if let Some(head) = status.head {
                    println!("Membership: {head}");
                } else {
                    println!("Enrollment incomplete.");
                }
                if status.conflict {
                    println!("Conflicting signed histories; network authorization suspended.");
                }
                if status.pending_leave {
                    println!("Local transport disabled; remote revocation outstanding.");
                }
                println!(
                    "Transport: {}; relay {}; loopback discovery {}.",
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
                    },
                );
                if let Some(error) = status.transport.error {
                    eprintln!("Transport: {error}");
                }
                for member in status.members {
                    println!(
                        "{member}{}",
                        if member == endpoint { " (self)" } else { "" }
                    );
                }
            } else {
                println!("Not paired.");
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
