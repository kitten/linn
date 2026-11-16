//! Implements logic for abstracting over a websocket stream that allows sending only [`Bytes`]-based
//! messages.
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bytes::Bytes;
use n0_error::AnyError;
#[cfg(not(wasm_browser))]
use n0_error::anyerr;
use n0_future::{Sink, Stream, ready};
#[cfg(not(wasm_browser))]
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::warn;

use crate::ExportKeyingMaterial;

#[cfg(not(wasm_browser))]
#[derive(derive_more::Debug)]
pub(crate) struct WsBytesFramed<T, const CONTROL_HEALTH: bool = false> {
    #[debug("WebSocketStream<T>")]
    pub(crate) io: tokio_websockets::WebSocketStream<T>,
}

#[cfg(wasm_browser)]
#[derive(derive_more::Debug)]
pub(crate) struct WsBytesFramed {
    #[debug("WebSocketStream")]
    pub(crate) io: ws_stream_wasm::WsStream,
}

/// Error type for WebSocket stream operations.
///
/// This is a generic [`AnyError`] and represents errors that can occur during WebSocket
/// communication. The underlying error type depends on the platform:
/// - On non-browser platforms: `tokio_websockets::Error`
/// - On browser WASM: `ws_stream_wasm::WsErr`
///
/// You can use [`AnyError::downcast_ref`] to downcast to the concrete error. However, the
/// downcast is not covered by any semver guarantees and the underlying error may change.
pub type StreamError = AnyError;

/// Shorthand for a type that implements both a websocket-based stream & sink for [`Bytes`].
pub trait BytesStreamSink:
    Stream<Item = Result<Bytes, StreamError>> + Sink<Bytes, Error = StreamError> + Unpin
{
}

impl<T> BytesStreamSink for T where
    T: Stream<Item = Result<Bytes, StreamError>> + Sink<Bytes, Error = StreamError> + Unpin
{
}

#[cfg(not(wasm_browser))]
impl<IO: ExportKeyingMaterial + AsyncRead + AsyncWrite + Unpin, const CONTROL_HEALTH: bool>
    ExportKeyingMaterial for WsBytesFramed<IO, CONTROL_HEALTH>
{
    fn export_keying_material<T: AsMut<[u8]>>(
        &self,
        output: T,
        label: &[u8],
        context: Option<&[u8]>,
    ) -> Option<T> {
        self.io
            .get_ref()
            .export_keying_material(output, label, context)
    }
}

#[cfg(wasm_browser)]
impl ExportKeyingMaterial for WsBytesFramed {
    fn export_keying_material<T: AsMut<[u8]>>(
        &self,
        _output: T,
        _label: &[u8],
        _context: Option<&[u8]>,
    ) -> Option<T> {
        None
    }
}

#[cfg(not(wasm_browser))]
impl<T: AsyncRead + AsyncWrite + Unpin, const CONTROL_HEALTH: bool> Stream
    for WsBytesFramed<T, CONTROL_HEALTH>
{
    type Item = Result<Bytes, StreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match ready!(Pin::new(&mut self.io).poll_next(cx)) {
                None => return Poll::Ready(None),
                Some(Err(e)) => return Poll::Ready(Some(Err(anyerr!(e)))),
                Some(Ok(msg)) => {
                    if msg.is_close() {
                        // Indicate the stream is done when we receive a close message.
                        // Note: We don't have to poll the stream to completion for it to close gracefully.
                        return Poll::Ready(None);
                    }
                    if CONTROL_HEALTH && msg.is_pong() && msg.as_payload().len() == 8 {
                        // Only the client health tracker sees control Pongs. Server
                        // application checkpoints retain their ordered binary replies.
                        let mut pong = Vec::with_capacity(9);
                        pong.push(10);
                        pong.extend_from_slice(msg.as_payload());
                        return Poll::Ready(Some(Ok(pong.into())));
                    }
                    if msg.is_ping() || msg.is_pong() {
                        continue; // Responding appropriately to these is done inside of tokio_websockets/browser impls
                    }
                    if !msg.is_binary() {
                        warn!(?msg, "Got websocket message of unsupported type, skipping.");
                        continue;
                    }
                    return Poll::Ready(Some(Ok(msg.into_payload().into())));
                }
            }
        }
    }
}

#[cfg(wasm_browser)]
impl Stream for WsBytesFramed {
    type Item = Result<Bytes, StreamError>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        loop {
            match ready!(Pin::new(&mut self.io).poll_next(cx)) {
                None => return Poll::Ready(None),
                Some(ws_stream_wasm::WsMessage::Binary(msg)) => {
                    return Poll::Ready(Some(Ok(msg.into())));
                }
                Some(msg) => {
                    warn!(?msg, "Got websocket message of unsupported type, skipping.");
                    continue;
                }
            }
        }
    }
}

#[cfg(not(wasm_browser))]
impl<T: AsyncRead + AsyncWrite + Unpin, const CONTROL_HEALTH: bool> Sink<Bytes>
    for WsBytesFramed<T, CONTROL_HEALTH>
{
    type Error = StreamError;

    fn start_send(mut self: Pin<&mut Self>, bytes: Bytes) -> Result<(), Self::Error> {
        let msg = if CONTROL_HEALTH && bytes.len() == 9 && bytes[0] == 9 {
            tokio_websockets::Message::ping(bytes.slice(1..))
        } else {
            tokio_websockets::Message::binary(tokio_websockets::Payload::from(bytes))
        };
        Pin::new(&mut self.io)
            .start_send(msg)
            .map_err(AnyError::from_std)
    }

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.io)
            .poll_ready(cx)
            .map_err(AnyError::from_std)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.io)
            .poll_flush(cx)
            .map_err(AnyError::from_std)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.io)
            .poll_close(cx)
            .map_err(AnyError::from_std)
    }
}

#[cfg(wasm_browser)]
impl Sink<Bytes> for WsBytesFramed {
    type Error = StreamError;

    fn start_send(mut self: Pin<&mut Self>, bytes: Bytes) -> Result<(), Self::Error> {
        let msg = ws_stream_wasm::WsMessage::Binary(Vec::from(bytes));
        Pin::new(&mut self.io)
            .start_send(msg)
            .map_err(AnyError::from_std)
    }

    fn poll_ready(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.io)
            .poll_ready(cx)
            .map_err(AnyError::from_std)
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.io)
            .poll_flush(cx)
            .map_err(AnyError::from_std)
    }

    fn poll_close(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Pin::new(&mut self.io)
            .poll_close(cx)
            .map_err(AnyError::from_std)
    }
}

#[cfg(all(test, not(wasm_browser)))]
mod control_health_tests {
    use super::*;
    use n0_future::{SinkExt, StreamExt};

    #[tokio::test]
    async fn client_health_uses_control_frames_but_checkpoints_remain_binary() {
        let (a, b) = tokio::io::duplex(4096);
        let mut client = WsBytesFramed::<_, true> {
            io: tokio_websockets::ClientBuilder::new().take_over(a),
        };
        let mut server = tokio_websockets::ServerBuilder::new().serve(b);
        let ping = Bytes::from_static(&[9, 1, 2, 3, 4, 5, 6, 7, 8]);
        client.send(ping.clone()).await.unwrap();
        let control = server.next().await.unwrap().unwrap();
        assert!(control.is_ping());
        assert_eq!(&control.as_payload()[..], &ping[1..]);
        server.flush().await.unwrap(); // Automatic control Pong.
        let pong = client.next().await.unwrap().unwrap();
        assert_eq!(&pong[..], &[10, 1, 2, 3, 4, 5, 6, 7, 8]);

        server
            .send(tokio_websockets::Message::binary(ping.clone()))
            .await
            .unwrap();
        assert_eq!(client.next().await.unwrap().unwrap(), ping);
        client.send(pong.clone()).await.unwrap();
        let checkpoint_reply = server.next().await.unwrap().unwrap();
        assert!(checkpoint_reply.is_binary());
        assert_eq!(&checkpoint_reply.as_payload()[..], &pong[..]);

        // A malformed control Pong cannot reach the fixed-width health tracker.
        server
            .send(tokio_websockets::Message::pong(vec![0; 7]))
            .await
            .unwrap();
        server
            .send(tokio_websockets::Message::binary(vec![2]))
            .await
            .unwrap();
        assert_eq!(&client.next().await.unwrap().unwrap()[..], &[2]);
    }

    #[tokio::test]
    async fn server_adapter_preserves_application_pings() {
        let (a, b) = tokio::io::duplex(4096);
        let mut server = WsBytesFramed::<_> {
            io: tokio_websockets::ServerBuilder::new().serve(a),
        };
        let mut client = tokio_websockets::ClientBuilder::new().take_over(b);
        server
            .send(Bytes::from_static(&[9, 0, 0, 0, 0, 0, 0, 0, 0]))
            .await
            .unwrap();
        assert!(client.next().await.unwrap().unwrap().is_binary());
    }
}
