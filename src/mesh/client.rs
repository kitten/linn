//! Bounded private coordinator API; owns no database or endpoint secret.
use super::membership::{Acceptance, Proposal, canonical, endpoint, hex, random_id, unhex};
use iroh::SecretKey;
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

const READ_DOMAIN: &[u8] = b"linn mesh read v1\0";
const MAX_RESPONSE: usize = 512 * 1024;

#[derive(Debug, thiserror::Error)]
pub enum ControlError {
    #[error("mesh service is unavailable; the mutation outcome may be uncertain")]
    Unavailable,
    #[error("mesh membership changed; fetch verified history before retrying")]
    StaleHead,
    #[error("mesh service authorization denied")]
    Unauthorized,
    #[error("mesh service rejected the request (HTTP {0})")]
    Rejected(u16),
    #[error("invalid mesh service response: {0}")]
    Protocol(&'static str),
}
type Result<T> = std::result::Result<T, ControlError>;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub version: u8,
    pub service: String,
    pub receipt_key: String,
}
#[derive(Serialize)]
struct ReadRequest<'a> {
    version: u8,
    service: &'a str,
    key: String,
    expires: u64,
    nonce: String,
    after: Option<&'a str>,
    invitation: Option<&'a str>,
}
#[derive(Serialize)]
struct SignedRead<'a> {
    request: ReadRequest<'a>,
    signature: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryPage {
    pub version: u8,
    pub head: String,
    pub events: Vec<Acceptance>,
    pub after: Option<String>,
    pub more: bool,
}

#[derive(Clone)]
pub struct ControlClient {
    origin: String,
    timeout: Duration,
}
impl ControlClient {
    pub fn new(origin: &str) -> crate::error::Result<Self> {
        Self::with_timeout(origin, Duration::from_secs(30))
    }

    pub(crate) fn for_sync(origin: &str) -> crate::error::Result<Self> {
        Self::with_timeout(origin, Duration::from_secs(5))
    }

    fn with_timeout(origin: &str, timeout: Duration) -> crate::error::Result<Self> {
        Ok(Self {
            origin: crate::storage::mesh::service_origin(origin)?,
            timeout,
        })
    }

    fn http(&self) -> Result<reqwest::Client> {
        reqwest::Client::builder()
            .timeout(self.timeout)
            .redirect(reqwest::redirect::Policy::none())
            .build()
            .map_err(|_| ControlError::Unavailable)
    }

    async fn response<T: DeserializeOwned + Serialize>(
        &self,
        request: reqwest::RequestBuilder,
    ) -> Result<T> {
        let mut response = request
            .send()
            .await
            .map_err(|_| ControlError::Unavailable)?;
        match response.status().as_u16() {
            200 => (),
            401 | 403 => return Err(ControlError::Unauthorized),
            409 => return Err(ControlError::StaleHead),
            429 | 500..=599 => return Err(ControlError::Unavailable),
            status @ 400..=499 => return Err(ControlError::Rejected(status)),
            _ => return Err(ControlError::Protocol("unexpected HTTP status")),
        }
        if response
            .content_length()
            .is_some_and(|n| n > MAX_RESPONSE as u64)
        {
            return Err(ControlError::Protocol("oversized body"));
        }
        let mut body = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|_| ControlError::Unavailable)?
        {
            if body.len() + chunk.len() > MAX_RESPONSE {
                return Err(ControlError::Protocol("oversized body"));
            }
            body.extend_from_slice(&chunk);
        }
        let value =
            serde_json::from_slice(&body).map_err(|_| ControlError::Protocol("malformed JSON"))?;
        if canonical(&value).map_err(|_| ControlError::Protocol("invalid JSON"))? != body {
            return Err(ControlError::Protocol("canonical JSON required"));
        }
        Ok(value)
    }

    pub async fn service(&self) -> Result<Service> {
        let service: Service = self
            .response(self.http()?.get(format!("{}/v1/service", self.origin)))
            .await?;
        if service.version != 1
            || service.service != self.origin
            || unhex::<32>(&service.receipt_key).is_err()
        {
            return Err(ControlError::Protocol("service identity mismatch"));
        }
        Ok(service)
    }

    /// The caller persists this exact proposal before issuing the request and
    /// verifies the returned acceptance against its locally trusted history.
    pub async fn commit(&self, proposal: &Proposal) -> Result<Acceptance> {
        if proposal.event.service != self.origin {
            return Err(ControlError::Protocol("proposal names another service"));
        }
        let bytes = canonical(proposal).map_err(|_| ControlError::Protocol("invalid proposal"))?;
        let request = self
            .http()?
            .post(format!("{}/v1/events", self.origin))
            .header("content-type", "application/json")
            .body(bytes);
        let accepted: Acceptance = self.response(request).await?;
        if &accepted.proposal != proposal {
            return Err(ControlError::Protocol("receipt names another proposal"));
        }
        Ok(accepted)
    }

    pub async fn history(
        &self,
        key: &SecretKey,
        invitation: Option<&str>,
        after: Option<&str>,
        known_head: Option<&str>,
    ) -> Result<HistoryPage> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|_| ControlError::Protocol("invalid local clock"))?
            .as_secs();
        let request = ReadRequest {
            version: 1,
            service: &self.origin,
            key: endpoint(key),
            expires: now + 60,
            nonce: random_id(),
            after,
            invitation,
        };
        let mut message = READ_DOMAIN.to_vec();
        message.extend(canonical(&request).map_err(|_| ControlError::Protocol("invalid request"))?);
        let signature = hex(&key.sign(&message).to_bytes());
        let bytes = canonical(&SignedRead { request, signature })
            .map_err(|_| ControlError::Protocol("invalid request"))?;
        let mut http = self
            .http()?
            .post(format!("{}/v1/history", self.origin))
            .header("content-type", "application/json")
            .body(bytes);
        if let Some(head) = known_head {
            http = http.header("x-linn-known-head", head);
        }
        let page: HistoryPage = self.response(http).await?;
        if page.version != 1
            || page.events.len() > 16
            || unhex::<32>(&page.head).is_err()
            || page
                .after
                .as_ref()
                .is_some_and(|s| s.len() != 16 || !s.bytes().all(|b| b.is_ascii_digit()))
            || (page.more
                && page
                    .after
                    .as_deref()
                    .is_none_or(|cursor| after.is_some_and(|old| cursor <= old)))
        {
            return Err(ControlError::Protocol("invalid history page or cursor"));
        }
        Ok(page)
    }
}
