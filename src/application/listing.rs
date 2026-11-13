use super::*;
use crate::storage::listing::ListQuery;
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};
#[derive(Serialize, Deserialize)]
pub struct ListOptions {
    pub scope: Option<String>,
    pub writer: Option<String>,
    pub kind: Option<String>,
    pub since: Option<String>,
    pub expiring_within: Option<String>,
    pub limit: u32,
    pub cursor: Option<String>,
}
#[derive(Serialize, Deserialize)]
struct Cursor {
    query: ListQuery,
    after: (String, String),
    publication: Vec<u8>,
}

pub fn duration(value: &str) -> Result<Duration> {
    let (digits, unit) = value.split_at(value.char_indices().last().map_or(0, |(i, _)| i));
    let multiplier = match unit {
        "m" => 60,
        "h" => 3600,
        "d" => 86400,
        "w" => 604800,
        _ => {
            return Err(Error::Invalid(
                "duration must use m, h, d or w (for example 7d)".into(),
            ));
        }
    };
    let seconds = digits
        .parse::<i64>()
        .ok()
        .filter(|n| *n > 0)
        .and_then(|n| n.checked_mul(multiplier))
        .ok_or_else(|| {
            Error::Invalid("duration must be a positive whole number within range".into())
        })?;
    Ok(Duration::seconds(seconds))
}

pub fn timestamp(value: OffsetDateTime) -> Result<String> {
    value
        .to_offset(time::UtcOffset::UTC)
        .replace_nanosecond(0)
        .unwrap()
        .format(&Rfc3339)
        .map_err(|e| Error::Invalid(e.to_string()))
}

pub async fn run(
    store: &Store,
    args: ListOptions,
    now: &str,
) -> Result<(Vec<crate::storage::listing::ListedRecord>, Option<String>)> {
    let (query, after, publication) = if let Some(cursor) = args.cursor {
        let decoded = cursor
            .strip_prefix("l1.")
            .and_then(|s| URL_SAFE_NO_PAD.decode(s).ok())
            .and_then(|bytes| serde_json::from_slice::<Cursor>(&bytes).ok())
            .ok_or_else(|| Error::Invalid("invalid list cursor".into()))?;
        (
            decoded.query,
            Some(decoded.after),
            Some(decoded.publication),
        )
    } else {
        let clock =
            OffsetDateTime::parse(now, &Rfc3339).map_err(|e| Error::Invalid(e.to_string()))?;
        let since = args
            .since
            .as_deref()
            .map(|value| {
                let date = if value.contains('T') {
                    OffsetDateTime::parse(value, &Rfc3339)
                        .map_err(|e| Error::Invalid(e.to_string()))?
                } else {
                    clock
                        .checked_sub(duration(value)?)
                        .ok_or_else(|| Error::Invalid("duration out of range".into()))?
                };
                timestamp(date)
            })
            .transpose()?;
        let expires_before = args
            .expiring_within
            .as_deref()
            .map(|value| {
                timestamp(
                    clock
                        .checked_add(duration(value)?)
                        .ok_or_else(|| Error::Invalid("duration out of range".into()))?,
                )
            })
            .transpose()?;
        (
            ListQuery {
                now: now.into(),
                kind: args.kind,
                writer: args.writer,
                scope: args.scope,
                since,
                expires_before,
            },
            None,
            None,
        )
    };
    let page = store
        .list_records(
            &query,
            after.as_ref(),
            args.limit as usize,
            publication.as_deref(),
        )
        .await?;
    let next = page
        .next
        .map(|after| -> Result<String> {
            Ok(format!(
                "l1.{}",
                URL_SAFE_NO_PAD.encode(serde_json::to_vec(&Cursor {
                    query,
                    after,
                    publication: page.publication
                })?)
            ))
        })
        .transpose()?;
    Ok((page.records, next))
}
pub fn since(value: Option<&str>, now: &str) -> Result<Option<String>> {
    let clock = OffsetDateTime::parse(now, &Rfc3339).map_err(|e| Error::Invalid(e.to_string()))?;
    value
        .map(|v| {
            let date = if v.contains('T') {
                OffsetDateTime::parse(v, &Rfc3339).map_err(|e| Error::Invalid(e.to_string()))?
            } else {
                clock
                    .checked_sub(duration(v)?)
                    .ok_or_else(|| Error::Invalid("duration out of range".into()))?
            };
            timestamp(date)
        })
        .transpose()
}
