use std::{
    env,
    ffi::{OsStr, OsString},
    iter,
    path::PathBuf,
};

use clap::{
    builder::{NonEmptyStringValueParser, OsStringValueParser},
    error::ErrorKind,
    Arg, ArgAction, ArgGroup, ArgMatches, Command as ClapCommand,
};

use super::env as bob_env;

const COMMAND_NAME: &str = "bob dataview";
const ENV_VAULT: &str = "BOB_DATAVIEW_VAULT";

#[derive(Debug, Clone, PartialEq, Eq)]
struct Request {
    query: QueryInput,
    format: OutputFormat,
    engine: Engine,
    vault: VaultConfig,
    strict_paths: bool,
    sync: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum QueryInput {
    Source(String),
    Dql(DqlInput),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DqlInput {
    Inline(String),
    File(PathBuf),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutputFormat {
    Json,
    Markdown,
    Paths,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Engine {
    Dynomark,
    Obsidian,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct VaultConfig {
    bob_dir: PathBuf,
    origin: Option<PathBuf>,
    obsidian_vault: Option<String>,
}

pub(crate) fn run(args: Vec<OsString>) -> i32 {
    let mut command = build_cli();
    let matches = match command.try_get_matches_from_mut(
        iter::once(OsString::from(COMMAND_NAME)).chain(args),
    ) {
        Ok(matches) => matches,
        Err(error) => return print_clap_error(error),
    };

    let request = match Request::from_matches(&matches, &mut command) {
        Ok(request) => request,
        Err(error) => return print_clap_error(error),
    };

    report_engine_not_implemented(&request);
    1
}

fn print_clap_error(error: clap::Error) -> i32 {
    let exit_code = error.exit_code();
    if let Err(print_error) = error.print() {
        eprintln!(
            "{COMMAND_NAME}: failed to print command-line error: {print_error}"
        );
    }
    exit_code
}

fn report_engine_not_implemented(request: &Request) {
    eprintln!("{COMMAND_NAME}: engine execution is not implemented yet");
    eprintln!("engine: {}", request.engine.as_str());
    eprintln!("query: {}", request.query.summary());
    eprintln!("format: {}", request.format.as_str());
    eprintln!("bob_dir: {}", request.vault.bob_dir.display());
    eprintln!(
        "origin: {}",
        optional_path_label(request.vault.origin.as_ref())
    );
    eprintln!(
        "vault: {}",
        request
            .vault
            .obsidian_vault
            .as_deref()
            .unwrap_or("(default)")
    );
    eprintln!("strict_paths: {}", request.strict_paths);
    eprintln!("sync: {}", request.sync);
}

fn optional_path_label(path: Option<&PathBuf>) -> String {
    path.map(|path| path.display().to_string())
        .unwrap_or_else(|| "(none)".to_string())
}

fn build_cli() -> ClapCommand {
    ClapCommand::new(COMMAND_NAME)
        .about("Run Dataview queries against the Bob Obsidian vault")
        .long_about(
            "Run Dataview source expressions or DQL queries against the Bob \
Obsidian vault.\n\n\
Source expressions return matching page paths. DQL queries support path, JSON, \
and markdown output modes.",
        )
        .after_help(
            "Examples:\n  bob dataview --source '#project and -\"archive\"'\n  bob dataview --query 'LIST FROM #waiting'\n  bob dataview --format json --query-file ~/queries/projects.dql",
        )
        .arg_required_else_help(true)
        .group(
            ArgGroup::new("query-input")
                .required(true)
                .multiple(false)
                .args(["source", "query", "query-file"]),
        )
        .arg(bob_dir_arg())
        .arg(engine_arg())
        .arg(format_arg())
        .arg(origin_arg())
        .arg(query_arg())
        .arg(query_file_arg())
        .arg(source_arg())
        .arg(strict_paths_arg())
        .arg(sync_arg())
        .arg(vault_arg())
}

fn bob_dir_arg() -> Arg {
    Arg::new("bob-dir")
        .long("bob-dir")
        .value_name("PATH")
        .value_parser(OsStringValueParser::new())
        .help("Bob vault root; defaults to BOB_DIR or ~/bob")
}

fn engine_arg() -> Arg {
    Arg::new("engine")
        .long("engine")
        .value_name("ENGINE")
        .default_value("obsidian")
        .value_parser(["dynomark", "obsidian"])
        .help("Query engine to use")
}

fn format_arg() -> Arg {
    Arg::new("format")
        .long("format")
        .value_name("FORMAT")
        .default_value("paths")
        .value_parser(["json", "markdown", "paths"])
        .help("Output format; markdown is available only for DQL")
}

fn origin_arg() -> Arg {
    Arg::new("origin")
        .long("origin")
        .value_name("VAULT_RELATIVE_PATH")
        .value_parser(OsStringValueParser::new())
        .help("Origin note for relative links and this")
}

fn query_arg() -> Arg {
    Arg::new("query")
        .long("query")
        .value_name("DQL")
        .value_parser(NonEmptyStringValueParser::new())
        .help("Full Dataview DQL query")
}

fn query_file_arg() -> Arg {
    Arg::new("query-file")
        .long("query-file")
        .value_name("PATH")
        .value_parser(OsStringValueParser::new())
        .help("Read a Dataview DQL query from a file; use - for stdin")
}

fn source_arg() -> Arg {
    Arg::new("source")
        .long("source")
        .value_name("SOURCE")
        .value_parser(NonEmptyStringValueParser::new())
        .help("Dataview source expression for page path lookup")
}

fn strict_paths_arg() -> Arg {
    Arg::new("strict-paths")
        .long("strict-paths")
        .action(ArgAction::SetTrue)
        .help("Fail when paths output cannot derive clean note paths")
}

fn sync_arg() -> Arg {
    Arg::new("sync")
        .long("sync")
        .action(ArgAction::SetTrue)
        .help("Run ob sync before querying, keeping sync logs off stdout")
}

fn vault_arg() -> Arg {
    Arg::new("vault")
        .long("vault")
        .value_name("NAME_OR_ID")
        .value_parser(NonEmptyStringValueParser::new())
        .help("Obsidian vault name or ID; defaults to BOB_DATAVIEW_VAULT")
}

impl Request {
    fn from_matches(
        matches: &ArgMatches,
        command: &mut ClapCommand,
    ) -> Result<Self, clap::Error> {
        let query = QueryInput::from_matches(matches);
        let format = OutputFormat::from_matches(matches);
        let strict_paths = matches.get_flag("strict-paths");

        if query.is_source() && format == OutputFormat::Markdown {
            return Err(command.error(
                ErrorKind::ArgumentConflict,
                "--format markdown requires a DQL query",
            ));
        }

        if strict_paths && format != OutputFormat::Paths {
            return Err(command.error(
                ErrorKind::ArgumentConflict,
                "--strict-paths can only be used with --format paths",
            ));
        }

        Ok(Self {
            query,
            format,
            engine: Engine::from_matches(matches),
            vault: VaultConfig::from_matches(matches),
            strict_paths,
            sync: matches.get_flag("sync"),
        })
    }
}

impl QueryInput {
    fn from_matches(matches: &ArgMatches) -> Self {
        if let Some(source) = matches.get_one::<String>("source") {
            return Self::Source(source.clone());
        }

        if let Some(query) = matches.get_one::<String>("query") {
            return Self::Dql(DqlInput::Inline(query.clone()));
        }

        let query_file = matches
            .get_one::<OsString>("query-file")
            .expect("clap query-input group requires query-file")
            .into();
        Self::Dql(DqlInput::File(query_file))
    }

    fn is_source(&self) -> bool {
        matches!(self, Self::Source(_))
    }

    fn summary(&self) -> String {
        match self {
            Self::Source(source) => {
                format!("source expression ({} bytes)", source.len())
            }
            Self::Dql(DqlInput::Inline(query)) => {
                format!("inline DQL ({} bytes)", query.len())
            }
            Self::Dql(DqlInput::File(path))
                if path.as_os_str() == OsStr::new("-") =>
            {
                "DQL from stdin".to_string()
            }
            Self::Dql(DqlInput::File(path)) => {
                format!("DQL file {}", path.display())
            }
        }
    }
}

impl OutputFormat {
    fn from_matches(matches: &ArgMatches) -> Self {
        match matches
            .get_one::<String>("format")
            .expect("clap provides a default format")
            .as_str()
        {
            "json" => Self::Json,
            "markdown" => Self::Markdown,
            "paths" => Self::Paths,
            value => unreachable!("unexpected format value from clap: {value}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Markdown => "markdown",
            Self::Paths => "paths",
        }
    }
}

impl Engine {
    fn from_matches(matches: &ArgMatches) -> Self {
        match matches
            .get_one::<String>("engine")
            .expect("clap provides a default engine")
            .as_str()
        {
            "dynomark" => Self::Dynomark,
            "obsidian" => Self::Obsidian,
            value => unreachable!("unexpected engine value from clap: {value}"),
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Dynomark => "dynomark",
            Self::Obsidian => "obsidian",
        }
    }
}

impl VaultConfig {
    fn from_matches(matches: &ArgMatches) -> Self {
        let bob_dir = matches
            .get_one::<OsString>("bob-dir")
            .map(PathBuf::from)
            .map(|path| bob_env::expand_tilde(&path))
            .unwrap_or_else(bob_env::bob_dir);
        let origin = matches.get_one::<OsString>("origin").map(PathBuf::from);
        let obsidian_vault = matches
            .get_one::<String>("vault")
            .cloned()
            .or_else(default_vault_from_env);

        Self {
            bob_dir,
            origin,
            obsidian_vault,
        }
    }
}

fn default_vault_from_env() -> Option<String> {
    env::var(ENV_VAULT).ok().filter(|value| !value.is_empty())
}
