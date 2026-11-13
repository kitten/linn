use super::*;
use std::{fs::File, io::Write, path::Path};
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

impl ExportArgs {
    pub fn resolve(self) -> (crate::application::ExportOptions, Option<PathBuf>) {
        (
            crate::application::ExportOptions {
                scope: self.scope,
                kind: self.kind,
                since: self.since,
                id: self.id,
                all_states: self.all_states,
            },
            self.output,
        )
    }
}
pub fn write_output(spool: &mut File, path: Option<PathBuf>) -> Result<()> {
    if let Some(path) = path.filter(|p| p != Path::new("-")) {
        let parent = path
            .parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
        let mut file = tempfile::NamedTempFile::new_in(parent)?;
        io::copy(spool, file.as_file_mut())?;
        file.as_file().sync_all()?;
        file.persist_noclobber(path)
            .map_err(|e| Error::Io(e.error))?;
    } else {
        let mut out = io::stdout().lock();
        io::copy(spool, &mut out)?;
        out.flush()?;
    }
    Ok(())
}
