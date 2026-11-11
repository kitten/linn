use crate::error::{Error, Result};
use std::{
    io::{BufReader, BufWriter},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
};
#[derive(Debug, PartialEq)]
pub struct Origin {
    pub destination: String,
    pub path: Option<String>,
}
impl Origin {
    pub fn parse(value: &str) -> Result<Self> {
        let rest = value
            .strip_prefix("ssh://")
            .ok_or_else(|| Error::Invalid("sync origins must use ssh://".into()))?;
        if rest.chars().any(|c| c.is_control()) || rest.contains(['%', '?', '#']) {
            return Err(Error::Invalid("unsupported character in SSH origin".into()));
        }
        let (authority, path) = rest
            .split_once('/')
            .map_or((rest, None), |(a, p)| (a, Some(format!("/{p}"))));
        let (user, host) = authority
            .rsplit_once('@')
            .map_or((None, authority), |(u, h)| (Some(u), h));
        if user.is_some_and(|s| {
            s.is_empty()
                || !s
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        }) {
            return Err(Error::Invalid("invalid SSH user".into()));
        }
        let (host, port) = if let Some(v) = host.strip_prefix('[') {
            let (ip, tail) = v
                .split_once(']')
                .ok_or_else(|| Error::Invalid("invalid IPv6 host".into()))?;
            if ip.parse::<std::net::Ipv6Addr>().is_err() {
                return Err(Error::Invalid("invalid IPv6 host".into()));
            }
            (
                format!("[{ip}]"),
                if tail.is_empty() {
                    None
                } else {
                    Some(
                        tail.strip_prefix(':')
                            .ok_or_else(|| Error::Invalid("invalid SSH port".into()))?,
                    )
                },
            )
        } else {
            let (h, p) = host
                .split_once(':')
                .map_or((host, None), |(h, p)| (h, Some(p)));
            if h.is_empty()
                || h.starts_with('-')
                || !h
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
            {
                return Err(Error::Invalid("invalid SSH host".into()));
            }
            (h.to_owned(), p)
        };
        if port.is_some_and(|p| p.parse::<u16>().map_or(true, |port| port == 0)) {
            return Err(Error::Invalid("invalid SSH port".into()));
        }
        let destination = format!(
            "ssh://{}{}{}",
            user.map(|u| format!("{u}@")).unwrap_or_default(),
            host,
            port.map(|p| format!(":{p}")).unwrap_or_default()
        );
        if path.as_deref() == Some("/") {
            return Err(Error::Invalid("SSH store path must name a file".into()));
        }
        Ok(Self { destination, path })
    }
    pub fn remote_command(&self) -> String {
        match &self.path {
            None => "linn sync:serve --stdio".into(),
            Some(path) => format!(
                "linn --store '{}' sync:serve --stdio",
                path.replace('\'', "'\\''")
            ),
        }
    }
}
pub struct Session {
    child: Child,
    pub reader: BufReader<ChildStdout>,
    pub writer: Option<BufWriter<ChildStdin>>,
}
impl Session {
    pub fn open(origin: &Origin) -> Result<Self> {
        let mut child = Command::new("ssh")
            .args([
                "-T",
                "-oBatchMode=yes",
                "-oConnectTimeout=15",
                "-oServerAliveInterval=15",
                "-oServerAliveCountMax=3",
                "--",
                &origin.destination,
                &origin.remote_command(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;
        let reader = BufReader::new(child.stdout.take().unwrap());
        let writer = Some(BufWriter::new(child.stdin.take().unwrap()));
        Ok(Self {
            child,
            reader,
            writer,
        })
    }
    pub fn close(mut self) -> Result<()> {
        self.writer.take();
        let status = self.child.wait()?;
        if !status.success() {
            return Err(Error::Invalid(format!("SSH sync exited with {status}")));
        }
        Ok(())
    }
}
impl Drop for Session {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
