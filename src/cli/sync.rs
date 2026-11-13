use crate::error::{Error, Result};
use clap::Subcommand;
use std::{io, path::Path};
#[derive(Subcommand)]
pub enum RemoteCommand {
    #[command(
        about = "Save an SSH remote; sync checks the connection",
        after_help = "Origin: ssh://[user@]host[:port][/absolute/store/path]\nBoth hosts need linn on PATH. SSH keys and known hosts must already work.\nAll records are exchanged unless --scope selects an original repository.\nTombstone IDs always propagate. Pairing grants no access beyond your SSH account."
    )]
    Pair {
        name: String,
        origin: String,
        #[arg(long)]
        scope: Option<String>,
    },
    Remove {
        name: String,
    },
    List,
}
pub async fn serve(path: &Path) -> Result<i32> {
    use crate::sync::protocol::{self, Request, Response};
    let mut reader = io::stdin().lock();
    let mut writer = io::stdout().lock();
    let Some(Request::Hello { version }) = protocol::read::<Request>(&mut reader)? else {
        protocol::write(
            &mut writer,
            &Response::Error {
                message: "incompatible sync protocol".into(),
            },
        )?;
        return Err(Error::Invalid("incompatible sync protocol".into()));
    };
    match crate::runtime::execute(
        path,
        crate::application::Operation::SyncHello { version },
        crate::application::Caller {
            diagnostics: Default::default(),
            scope: None,
            writer: None,
        },
        None,
    )
    .await
    {
        Ok(reply) => {
            let crate::application::Value::SyncFrame(response) = reply.value else {
                return Err(Error::Invalid("unexpected sync hello response".into()));
            };
            protocol::write(&mut writer, &response)?;
        }
        Err(error) => {
            protocol::write(
                &mut writer,
                &Response::Error {
                    message: error.to_string(),
                },
            )?;
            return Err(error);
        }
    }
    while let Some(frame) = protocol::read::<Request>(&mut reader)? {
        let reply = crate::runtime::execute(
            path,
            crate::application::Operation::SyncFrame(frame),
            crate::application::Caller {
                diagnostics: Default::default(),
                scope: None,
                writer: None,
            },
            None,
        )
        .await;
        match reply {
            Ok(reply) => {
                let crate::application::Value::SyncFrame(response) = reply.value else {
                    return Err(Error::Invalid("unexpected sync response".into()));
                };
                protocol::write(&mut writer, &response)?;
            }
            Err(e) => {
                protocol::write(
                    &mut writer,
                    &Response::Error {
                        message: e.to_string(),
                    },
                )?;
                return Err(e);
            }
        }
    }
    Ok(0)
}
