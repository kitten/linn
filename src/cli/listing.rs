use super::*;
#[derive(clap::Args)]
pub(super) struct ListArgs {
    #[arg(long, num_args = 0..=1, default_missing_value = ".", conflicts_with = "cursor",
        help = "Exact repository scope; omit the value or use . for the current repository")]
    scope: Option<String>,
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

impl ListArgs {
    pub fn resolve(self) -> Result<crate::application::ListOptions> {
        let scope = self
            .scope
            .map(|s| {
                if s == "." {
                    paths::scope().ok_or_else(|| {
                        Error::Invalid(
                            "current directory has no repository scope; use --scope <repository>"
                                .into(),
                        )
                    })
                } else {
                    Ok(s)
                }
            })
            .transpose()?;
        Ok(crate::application::ListOptions {
            scope,
            writer: self.writer,
            kind: self.kind,
            since: self.since,
            expiring_within: self.expiring_within,
            limit: self.limit,
            cursor: self.cursor,
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn scope_argument_and_cursor_conflicts() {
        use clap::Parser;
        for (flags, expected) in [
            (vec!["--scope"], Some(".")),
            (vec!["--scope", "."], Some(".")),
            (vec!["--scope", "owner/repo"], Some("owner/repo")),
            (vec![], None),
        ] {
            let cli = Args::try_parse_from([vec!["linn", "list"], flags].concat()).unwrap();
            let Command::List(args) = cli.command else {
                panic!("expected list")
            };
            assert_eq!(args.scope.as_deref(), expected);
        }
        assert!(Args::try_parse_from(["linn", "list", "--scope", "--cursor", "token"]).is_err());
    }

    #[test]
    fn duration_validation() {
        assert_eq!(
            crate::application::listing::duration("7d")
                .unwrap()
                .whole_seconds(),
            604800
        );
        for invalid in ["", "0d", "-1h", "1.5d", "999999999999999999d", "🦀"] {
            assert!(crate::application::listing::duration(invalid).is_err());
        }
    }
}
