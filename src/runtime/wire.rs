//! Bounded local protocol, separate from the SSH protocol.
use super::{WAIT, protocol::Frame};
use crate::error::{Error, Result};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Serialize, de::DeserializeOwned};
use std::{
    fs::File,
    io::{BufReader, BufWriter, Cursor, Read, Write},
};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::{sync::mpsc, time::timeout};
pub const MAX_FRAME: usize = 16 * 1024 * 1024;
pub(super) async fn send<T: Serialize>(
    stream: &mut (impl AsyncWrite + Unpin),
    value: &T,
) -> Result<()> {
    let mut bytes = Vec::with_capacity(128);
    bytes.extend_from_slice(&[0; 4]);
    serde_json::to_writer(&mut bytes, value)?;
    let n = bytes.len() - 4;
    if n > MAX_FRAME {
        return Err(Error::Invalid("local frame exceeds 16 MiB".into()));
    }
    bytes[..4].copy_from_slice(&(n as u32).to_be_bytes());
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}
pub(super) async fn receive<T: DeserializeOwned>(
    stream: &mut (impl AsyncRead + Unpin),
) -> Result<T> {
    let n = stream.read_u32().await? as usize;
    if n == 0 || n > MAX_FRAME {
        return Err(Error::Invalid("invalid local frame length".into()));
    }
    let mut bytes = vec![0; n];
    stream.read_exact(&mut bytes).await?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub(super) const CHUNK_BYTES: usize = 64 * 1024;

pub(super) async fn send_file(
    stream: &mut (impl AsyncWrite + Unpin),
    file: &mut File,
) -> Result<()> {
    let mut bytes = vec![0; CHUNK_BYTES];
    loop {
        let n = file.read(&mut bytes)?;
        if n == 0 {
            break;
        }
        send(stream, &Frame::Chunk(STANDARD.encode(&bytes[..n]))).await?;
    }
    send(stream, &Frame::End).await
}

pub(super) async fn send_json(
    stream: &mut (impl AsyncWrite + Unpin),
    value: impl Serialize + Send + 'static,
) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(2);
    let encoder = tokio::task::spawn_blocking(move || -> Result<()> {
        let mut writer = BufWriter::with_capacity(CHUNK_BYTES, ChunkWriter(tx));
        serde_json::to_writer(&mut writer, &value)?;
        writer.flush()?;
        Ok(())
    });
    while let Some(bytes) = rx.recv().await {
        send(stream, &Frame::Chunk(STANDARD.encode(bytes))).await?;
    }
    encoder.await.map_err(worker_error)??;
    send(stream, &Frame::End).await
}

pub(super) async fn receive_json<T: DeserializeOwned + Send + 'static>(
    stream: &mut (impl AsyncRead + Unpin),
) -> Result<T> {
    let Some(first) = receive_chunk(stream).await? else {
        return Ok(serde_json::from_slice(&[])?);
    };
    let Some(second) = receive_chunk(stream).await? else {
        return Ok(serde_json::from_slice(&first)?);
    };
    let (tx, rx) = mpsc::channel(2);
    let decoder = tokio::task::spawn_blocking(move || {
        serde_json::from_reader(BufReader::new(ChunkReader {
            rx,
            bytes: Cursor::new(Vec::new()),
        }))
    });
    let transfer = async {
        for bytes in [first, second] {
            if tx.send(bytes).await.is_err() {
                return Ok::<_, Error>(());
            }
        }
        while let Some(bytes) = receive_chunk(stream).await? {
            if tx.send(bytes).await.is_err() {
                break;
            }
        }
        Ok(())
    }
    .await;
    drop(tx);
    let value = decoder.await.map_err(worker_error)?;
    transfer?;
    Ok(value?)
}

pub(super) async fn receive_file(
    stream: &mut (impl AsyncRead + Unpin),
    file: &mut File,
) -> Result<()> {
    while let Some(bytes) = receive_chunk(stream).await? {
        file.write_all(&bytes)?;
    }
    Ok(())
}
async fn receive_chunk(stream: &mut (impl AsyncRead + Unpin)) -> Result<Option<Vec<u8>>> {
    match timeout(WAIT, receive::<Frame>(stream))
        .await
        .map_err(|_| Error::Invalid("local stream stalled".into()))??
    {
        Frame::Chunk(bytes) => {
            if bytes.len() > 128 * 1024 {
                return Err(Error::Invalid("oversized stream chunk".into()));
            }
            Ok(Some(STANDARD.decode(bytes).map_err(|_| {
                Error::Invalid("invalid stream chunk".into())
            })?))
        }
        Frame::End => Ok(None),
        _ => Err(Error::Invalid("incomplete local stream".into())),
    }
}
fn worker_error(error: tokio::task::JoinError) -> Error {
    Error::Invalid(format!("local JSON worker failed: {error}"))
}
struct ChunkWriter(mpsc::Sender<Vec<u8>>);
impl Write for ChunkWriter {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let n = bytes.len().min(CHUNK_BYTES);
        if n != 0 {
            self.0
                .blocking_send(bytes[..n].to_vec())
                .map_err(|_| std::io::Error::from(std::io::ErrorKind::BrokenPipe))?;
        }
        Ok(n)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
struct ChunkReader {
    rx: mpsc::Receiver<Vec<u8>>,
    bytes: Cursor<Vec<u8>>,
}
impl Read for ChunkReader {
    fn read(&mut self, bytes: &mut [u8]) -> std::io::Result<usize> {
        loop {
            let n = Read::read(&mut self.bytes, bytes)?;
            if n != 0 || bytes.is_empty() {
                return Ok(n);
            }
            let Some(chunk) = self.rx.blocking_recv() else {
                return Ok(0);
            };
            self.bytes = Cursor::new(chunk);
        }
    }
}
