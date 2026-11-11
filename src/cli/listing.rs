//! @ref LLP 0004#linn-list
use super::*;
use crate::storage::listing::ListQuery;
use serde::{Deserialize, Serialize};
use time::{Duration, OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(clap::Args)]
pub(super) struct ListArgs {
    #[arg(long, help = "Exact stored writer email", conflicts_with = "cursor")]
    writer: Option<String>,
    #[arg(long, value_parser = ["note", "doc"], conflicts_with = "cursor")]
    kind: Option<String>,
    #[arg(
        long,
        help = "Created since a duration (7d, 24h) or RFC3339 timestamp",
        conflicts_with = "cursor"
    )]
    since: Option<String>,
    #[arg(
        long,
        help = "Expire within a duration (30d); earliest expiry first",
        conflicts_with = "cursor"
    )]
    expiring_within: Option<String>,
    #[arg(long, default_value_t = 20, value_parser = clap::value_parser!(u32).range(1..=100))]
    limit: u32,
    #[arg(long, help = "Continue a list using its next cursor")]
    cursor: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct Cursor {
    query: ListQuery,
    after: (String, String),
    publication: Vec<u8>,
}

pub(super) fn duration(value: &str) -> Result<Duration> {
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

pub(super) fn timestamp(value: OffsetDateTime) -> Result<String> {
    value
        .to_offset(time::UtcOffset::UTC)
        .replace_nanosecond(0)
        .unwrap()
        .format(&Rfc3339)
        .map_err(|e| Error::Invalid(e.to_string()))
}

pub(super) async fn run(store: &Store, args: ListArgs, now: &str, json: bool) -> Result<i32> {
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
    let empty = page.records.is_empty();
    if json {
        println!("{}", json!({"records":page.records,"next":next}));
    } else {
        for r in page.records {
            println!(
                "{}  {}  created: {}  expires: {}\n  {}",
                r.record,
                r.kind,
                if r.created.is_empty() {
                    "unknown"
                } else {
                    &r.created
                },
                r.stale_after.as_deref().unwrap_or("never"),
                r.label.replace(['\n', '\r', '\t'], " ")
            );
        }
        if empty {
            println!("No records.");
        }
        if let Some(next) = next {
            println!("Next: linn list --cursor {next}");
        }
    }
    Ok(if empty { 1 } else { 0 })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn duration_validation() {
        assert_eq!(duration("7d").unwrap().whole_seconds(), 604800);
        for invalid in ["", "0d", "-1h", "1.5d", "999999999999999999d", "🦀"] {
            assert!(duration(invalid).is_err());
        }
    }
}
