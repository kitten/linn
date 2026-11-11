//! @ref LLP 0006#file-export-and-import
use super::*;
use crate::storage::{
    bundle::{Selection, import_bundle, stage},
    sync::Replica,
};
use std::io::{BufReader, Seek, SeekFrom, Write};
use std::path::Path;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};

#[derive(clap::Args)]
#[command(
    after_help = "Filters are ANDed; repeated --id references are ORed. References export whole records.\nDefault: current records. Full backup: export --all-states (no filters).\nFiltered history must include its lifecycle witnesses; references never pull in other bodies.\nFiltered exports omit unrelated tombstones. Import only bundles from trusted writers."
)]
pub(super) struct ExportArgs {
    #[arg(long, help = "Exact original repository scope (case-insensitive)")]
    scope: Option<String>,
    #[arg(long, value_parser = ["note", "doc"])]
    kind: Option<String>,
    #[arg(long, help = "Created since a duration (7d, 24h) or RFC3339 timestamp")]
    since: Option<String>,
    #[arg(
        long,
        help = "Select a whole record by reference; repeat to select more"
    )]
    id: Vec<String>,
    #[arg(
        long,
        help = "Include history; unfiltered exports also include all deletions"
    )]
    all_states: bool,
    #[arg(long, help = "Create a new bundle file; default or - writes stdout")]
    output: Option<PathBuf>,
}

pub(super) async fn export(
    store: &mut Store,
    args: ExportArgs,
    now: &str,
    caller: Option<&str>,
) -> Result<i32> {
    let clock = OffsetDateTime::parse(now, &Rfc3339).map_err(|e| Error::Invalid(e.to_string()))?;
    let since = args
        .since
        .as_deref()
        .map(|value| {
            let date = if value.contains('T') {
                OffsetDateTime::parse(value, &Rfc3339).map_err(|e| Error::Invalid(e.to_string()))?
            } else {
                clock
                    .checked_sub(listing::duration(value)?)
                    .ok_or_else(|| Error::Invalid("duration out of range".into()))?
            };
            listing::timestamp(date)
        })
        .transpose()?;
    let mut ids = Vec::new();
    for reference in args.id {
        ids.push(store.export_reference(&reference, caller).await?);
    }
    let selection = Selection {
        scope: args.scope,
        kind: args.kind,
        since,
        ids,
        all_states: args.all_states,
        now: now.into(),
    };
    // Stage before opening the destination: a failed selection never leaves a partial file.
    let mut spool = tempfile::tempfile()?;
    let count = store.export_bundle(&selection, &mut spool).await?;
    spool.seek(SeekFrom::Start(0))?;
    if let Some(path) = args.output.filter(|p| p != Path::new("-")) {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        io::copy(&mut spool, file.as_file_mut())?;
        file.as_file().sync_all()?;
        file.persist_noclobber(&path)
            .map_err(|e| Error::Io(e.error))?;
    } else {
        let mut out = io::stdout().lock();
        io::copy(&mut spool, &mut out)?;
        out.flush()?;
    }
    eprintln!("Exported {count} items.");
    Ok(0)
}

pub(super) async fn import(path: &Path, file: &str, json: bool) -> Result<i32> {
    // Validate even the end marker before opening or changing the destination store.
    let staged = if file == "-" {
        stage(&mut io::stdin().lock())?
    } else {
        stage(&mut BufReader::new(std::fs::File::open(file)?))?
    };
    let replica = Replica::open(path).await?;
    let counts = import_bundle(&replica, staged).await?;
    if json {
        println!("{}", serde_json::to_string(&counts)?);
    } else {
        println!(
            "Imported {} items; {} unchanged.",
            counts.changed, counts.unchanged
        );
    }
    Ok(0)
}
