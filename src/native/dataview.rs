use std::{
    env,
    ffi::{OsStr, OsString},
    fs,
    io::{self, Read},
    iter,
    path::PathBuf,
    process::{Command, Output},
    thread,
    time::Duration,
};

use clap::{
    builder::{NonEmptyStringValueParser, OsStringValueParser},
    error::ErrorKind,
    Arg, ArgAction, ArgGroup, ArgMatches, Command as ClapCommand,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::env as bob_env;

const COMMAND_NAME: &str = "bob dataview";
const ENV_OBSIDIAN_COMMAND: &str = "BOB_DATAVIEW_OBSIDIAN_COMMAND";
const ENV_VAULT: &str = "BOB_DATAVIEW_VAULT";
const RESULT_PREFIX: &str = "BOB_DATAVIEW_RESULT\t";
const OBSIDIAN_EVAL_SCRIPT: &str = r#"
(async () => {
  function plain(value, seen = new WeakSet()) {
    if (value == null || typeof value === "string" || typeof value === "number" || typeof value === "boolean") {
      return value;
    }
    if (typeof value === "bigint") {
      return value.toString();
    }
    if (Array.isArray(value)) {
      return value.map((item) => plain(item, seen));
    }
    if (typeof value !== "object") {
      return String(value);
    }
    if (seen.has(value)) {
      return "[Circular]";
    }
    seen.add(value);
    if (typeof value.path === "string" && ("display" in value || "embed" in value || "type" in value)) {
      return {
        type: "link",
        path: value.path,
        display: value.display ?? null,
        embed: Boolean(value.embed),
      };
    }
    if (typeof value.toISO === "function") {
      try {
        return value.toISO();
      } catch (_error) {
      }
    }
    if (typeof value.array === "function") {
      try {
        return plain(value.array(), seen);
      } catch (_error) {
      }
    }
    const output = {};
    for (const [key, item] of Object.entries(value)) {
      if (typeof item !== "function") {
        output[key] = plain(item, seen);
      }
    }
    return output;
  }

  function messageFor(error) {
    if (error == null) {
      return "unknown error";
    }
    if (typeof error === "string") {
      return error;
    }
    if (typeof error.message === "string" && error.message.length > 0) {
      return error.message;
    }
    return JSON.stringify(plain(error));
  }

  function dataviewApi() {
    return globalThis.app?.plugins?.plugins?.dataview?.api
      ?? globalThis.window?.DataviewAPI
      ?? globalThis.DataviewAPI;
  }

  async function sleep(milliseconds) {
    await new Promise((resolve) => setTimeout(resolve, milliseconds));
  }

  async function waitForDataview() {
    for (let attempt = 0; attempt < 50; attempt += 1) {
      const api = dataviewApi();
      if (api) {
        return api;
      }
      await sleep(100);
    }
    const error = new Error("Dataview is disabled, missing, or not loaded in this Obsidian vault");
    error.bobCode = "DATAVIEW_MISSING";
    throw error;
  }

  async function waitForIndexReady() {
    if (globalThis.app?.metadataCache?.on) {
      await Promise.race([
        new Promise((resolve) => {
          const off = globalThis.app.metadataCache.on("dataview:index-ready", () => {
            if (typeof off === "function") {
              off();
            }
            resolve();
          });
        }),
        sleep(1500),
      ]);
    } else {
      await sleep(250);
    }
  }

  function unwrapDataviewResult(result) {
    if (result && typeof result === "object" && result.successful === false) {
      const error = new Error(messageFor(result.error ?? result));
      error.bobCode = "DATAVIEW_QUERY_ERROR";
      error.details = result.error ?? result;
      throw error;
    }
    if (result && typeof result === "object" && result.successful === true && "value" in result) {
      return result.value;
    }
    return result;
  }

  function emit(payload) {
    console.log(resultPrefix + JSON.stringify(payload));
  }

  try {
    const api = await waitForDataview();
    await waitForIndexReady();

    if (request.query.kind === "source") {
      const paths = Array.from(await api.pagePaths(request.query.source) ?? []);
      emit({
        status: "ok",
        kind: "source_paths",
        paths: plain(paths),
        warnings: [],
      });
      return;
    }

    const origin = request.origin ?? undefined;
    if (request.format === "markdown") {
      const markdown = unwrapDataviewResult(await api.tryQueryMarkdown(request.query.query, origin));
      emit({
        status: "ok",
        kind: "markdown",
        markdown: String(markdown ?? ""),
        warnings: [],
      });
      return;
    }

    const result = unwrapDataviewResult(await api.tryQuery(request.query.query, origin, { forceId: true }));
    emit({
      status: "ok",
      kind: "dql_json",
      result: plain(result),
      warnings: [],
    });
  } catch (error) {
    emit({
      status: "error",
      code: error?.bobCode ?? "ENGINE_ERROR",
      message: messageFor(error),
      details: plain(error?.details ?? error),
    });
  }
})();
"#;

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

    match run_request(&request) {
        Ok(()) => 0,
        Err(error) => {
            error.report();
            error.exit_code()
        }
    }
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

fn run_request(request: &Request) -> Result<(), DataviewError> {
    match request.engine {
        Engine::Obsidian => run_obsidian(request),
        Engine::Dynomark => Err(DataviewError::EngineNotImplemented {
            engine: request.engine,
            query: request.query.summary(),
            format: request.format,
        }),
    }
}

fn run_obsidian(request: &Request) -> Result<(), DataviewError> {
    if request.sync {
        return Err(DataviewError::SyncNotImplemented);
    }

    let eval_request = request.obsidian_eval_request()?;
    let javascript = build_obsidian_javascript(&eval_request)?;
    let output = run_obsidian_eval(&request.vault, &javascript)?;
    let engine_output = parse_protocol_stdout(&output.stdout)?;
    emit_engine_output(request, engine_output)
}

fn run_obsidian_eval(
    vault: &VaultConfig,
    javascript: &str,
) -> Result<Output, DataviewError> {
    let command = obsidian_command();
    let output =
        run_obsidian_process(&command, vault, javascript).map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                DataviewError::MissingObsidianCommand {
                    command: command.clone(),
                }
            } else {
                DataviewError::RunObsidian {
                    command: command.clone(),
                    error,
                }
            }
        })?;

    if output.status.success() {
        Ok(output)
    } else {
        Err(obsidian_failure(output))
    }
}

fn run_obsidian_process(
    command: &OsString,
    vault: &VaultConfig,
    javascript: &str,
) -> io::Result<Output> {
    let first = obsidian_process(command, vault, javascript).output();
    if first.as_ref().is_err_and(is_text_file_busy) {
        thread::sleep(Duration::from_millis(10));
        return obsidian_process(command, vault, javascript).output();
    }

    first
}

fn obsidian_process(
    command: &OsString,
    vault: &VaultConfig,
    javascript: &str,
) -> Command {
    let mut process = Command::new(command);
    if let Some(obsidian_vault) = &vault.obsidian_vault {
        process.arg(format!("vault={obsidian_vault}"));
    }
    process.arg("eval").arg(format!("code={javascript}"));
    process
}

fn is_text_file_busy(error: &io::Error) -> bool {
    error.raw_os_error() == Some(26)
}

fn obsidian_command() -> OsString {
    env::var_os(ENV_OBSIDIAN_COMMAND)
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| OsString::from("obsidian"))
}

fn obsidian_failure(output: Output) -> DataviewError {
    let exit_code = bob_env::exit_code(output.status);
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let combined = format!("{stderr}\n{stdout}");
    let lower = combined.to_lowercase();

    if lower.contains("unable to find obsidian")
        || lower.contains("make sure obsidian is running")
        || lower.contains("could not connect")
    {
        return DataviewError::ObsidianNotRunning {
            exit_code,
            output: child_output_excerpt(&stdout, &stderr),
        };
    }

    DataviewError::ObsidianFailed {
        exit_code,
        output: child_output_excerpt(&stdout, &stderr),
    }
}

fn build_obsidian_javascript(
    request: &ObsidianEvalRequest,
) -> Result<String, DataviewError> {
    let request_json = serde_json::to_string(request)
        .map_err(DataviewError::SerializeRequest)?;
    let prefix_json = serde_json::to_string(RESULT_PREFIX)
        .map_err(DataviewError::SerializeRequest)?;

    Ok(format!(
        "const request = {request_json};\n\
         const resultPrefix = {prefix_json};\n\
         {OBSIDIAN_EVAL_SCRIPT}"
    ))
}

fn parse_protocol_stdout(stdout: &[u8]) -> Result<EngineOutput, DataviewError> {
    let stdout_text = String::from_utf8_lossy(stdout);
    let payloads = stdout_text
        .lines()
        .filter_map(|line| line.strip_prefix(RESULT_PREFIX))
        .collect::<Vec<_>>();

    match payloads.as_slice() {
        [] => Err(DataviewError::MissingProtocolSentinel {
            output: stdout_excerpt(&stdout_text),
        }),
        [payload] => parse_protocol_payload(payload),
        _ => Err(DataviewError::MalformedProtocolResponse {
            reason: "multiple sentinel responses found".to_string(),
        }),
    }
}

fn parse_protocol_payload(
    payload: &str,
) -> Result<EngineOutput, DataviewError> {
    let envelope: ProtocolEnvelope =
        serde_json::from_str(payload).map_err(|error| {
            DataviewError::MalformedProtocolResponse {
                reason: format!("invalid sentinel JSON: {error}"),
            }
        })?;

    envelope.into_engine_output()
}

fn emit_engine_output(
    request: &Request,
    output: EngineOutput,
) -> Result<(), DataviewError> {
    for warning in &output.warnings {
        eprintln!("{COMMAND_NAME}: warning: {warning}");
    }

    match (&output.response, request.format) {
        (EngineResponse::SourcePaths(paths), OutputFormat::Paths) => {
            if !paths.is_empty() {
                println!("{}", paths.join("\n"));
            }
            Ok(())
        }
        (EngineResponse::SourcePaths(paths), OutputFormat::Json) => {
            print_json(serde_json::json!({
                "engine": "obsidian",
                "query_kind": "source",
                "format": request.format.as_str(),
                "paths": paths,
                "warnings": output.warnings,
            }))
        }
        (EngineResponse::DqlJson(result), OutputFormat::Json) => {
            print_json(serde_json::json!({
                "engine": "obsidian",
                "query_kind": "dql",
                "format": request.format.as_str(),
                "result": result,
                "warnings": output.warnings,
            }))
        }
        (EngineResponse::DqlJson(_), OutputFormat::Paths) => {
            Err(DataviewError::DqlPathsNotImplemented)
        }
        (EngineResponse::DqlJson(_), OutputFormat::Markdown) => Err(
            DataviewError::MalformedProtocolResponse {
                reason:
                    "DQL JSON protocol response did not match requested format"
                        .to_string(),
            },
        ),
        (EngineResponse::Markdown(markdown), OutputFormat::Markdown) => {
            print!("{markdown}");
            Ok(())
        }
        (EngineResponse::Markdown(_), _) => Err(
            DataviewError::MalformedProtocolResponse {
                reason:
                    "markdown protocol response did not match requested format"
                        .to_string(),
            },
        ),
        (EngineResponse::SourcePaths(_), OutputFormat::Markdown) => Err(
            DataviewError::MalformedProtocolResponse {
                reason:
                    "source path protocol response did not match requested format"
                        .to_string(),
            },
        ),
    }
}

fn print_json(value: Value) -> Result<(), DataviewError> {
    let json = serde_json::to_string(&value)
        .map_err(DataviewError::SerializeOutput)?;
    println!("{json}");
    Ok(())
}

#[derive(Debug, Serialize)]
struct ObsidianEvalRequest {
    format: &'static str,
    origin: Option<String>,
    query: ObsidianEvalQuery,
}

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum ObsidianEvalQuery {
    Source { source: String },
    Dql { query: String },
}

#[derive(Debug)]
struct EngineOutput {
    response: EngineResponse,
    warnings: Vec<String>,
}

#[derive(Debug)]
enum EngineResponse {
    SourcePaths(Vec<String>),
    DqlJson(Value),
    Markdown(String),
}

#[derive(Debug, Deserialize)]
struct ProtocolEnvelope {
    status: String,
    #[serde(default)]
    kind: Option<String>,
    #[serde(default)]
    paths: Option<Vec<String>>,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    markdown: Option<String>,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(default)]
    code: Option<String>,
    #[serde(default)]
    message: Option<String>,
}

impl ProtocolEnvelope {
    fn into_engine_output(self) -> Result<EngineOutput, DataviewError> {
        match self.status.as_str() {
            "ok" => self.ok_output(),
            "error" => Err(protocol_error(self.code, self.message)),
            other => Err(DataviewError::MalformedProtocolResponse {
                reason: format!("unexpected protocol status: {other}"),
            }),
        }
    }

    fn ok_output(self) -> Result<EngineOutput, DataviewError> {
        let response = match self.kind.as_deref() {
            Some("source_paths") => {
                EngineResponse::SourcePaths(self.paths.ok_or_else(|| {
                    DataviewError::MalformedProtocolResponse {
                        reason: "source_paths response missing paths"
                            .to_string(),
                    }
                })?)
            }
            Some("dql_json") => {
                EngineResponse::DqlJson(self.result.ok_or_else(|| {
                    DataviewError::MalformedProtocolResponse {
                        reason: "dql_json response missing result".to_string(),
                    }
                })?)
            }
            Some("markdown") => {
                EngineResponse::Markdown(self.markdown.ok_or_else(|| {
                    DataviewError::MalformedProtocolResponse {
                        reason: "markdown response missing markdown"
                            .to_string(),
                    }
                })?)
            }
            Some(other) => {
                return Err(DataviewError::MalformedProtocolResponse {
                    reason: format!(
                        "unexpected protocol response kind: {other}"
                    ),
                });
            }
            None => {
                return Err(DataviewError::MalformedProtocolResponse {
                    reason: "protocol response missing kind".to_string(),
                });
            }
        };

        Ok(EngineOutput {
            response,
            warnings: self.warnings,
        })
    }
}

fn protocol_error(
    code: Option<String>,
    message: Option<String>,
) -> DataviewError {
    let code = code.unwrap_or_else(|| "ENGINE_ERROR".to_string());
    let message = message
        .unwrap_or_else(|| "Obsidian Dataview engine failed".to_string());

    match code.as_str() {
        "DATAVIEW_MISSING" => DataviewError::DataviewMissing { message },
        "DATAVIEW_QUERY_ERROR" => DataviewError::DataviewQuery { message },
        _ => DataviewError::ProtocolEngine { code, message },
    }
}

#[derive(Debug)]
enum DataviewError {
    DataviewMissing {
        message: String,
    },
    DataviewQuery {
        message: String,
    },
    DqlPathsNotImplemented,
    EngineNotImplemented {
        engine: Engine,
        query: String,
        format: OutputFormat,
    },
    MalformedProtocolResponse {
        reason: String,
    },
    MissingObsidianCommand {
        command: OsString,
    },
    MissingProtocolSentinel {
        output: String,
    },
    ObsidianFailed {
        exit_code: i32,
        output: String,
    },
    ObsidianNotRunning {
        exit_code: i32,
        output: String,
    },
    ProtocolEngine {
        code: String,
        message: String,
    },
    QueryRead {
        path: Option<PathBuf>,
        error: io::Error,
    },
    RunObsidian {
        command: OsString,
        error: io::Error,
    },
    SerializeOutput(serde_json::Error),
    SerializeRequest(serde_json::Error),
    SyncNotImplemented,
}

impl DataviewError {
    fn report(&self) {
        match self {
            Self::DataviewMissing { message } => {
                eprintln!(
                    "{COMMAND_NAME}: Dataview is disabled, missing, or not \
                     ready in Obsidian"
                );
                eprintln!("Dataview reported: {message}");
            }
            Self::DataviewQuery { message } => {
                eprintln!("{COMMAND_NAME}: Dataview query failed");
                eprintln!("Dataview reported: {message}");
            }
            Self::DqlPathsNotImplemented => {
                eprintln!(
                    "{COMMAND_NAME}: DQL paths output is not implemented yet"
                );
                eprintln!("Use --format json or --format markdown for now.");
            }
            Self::EngineNotImplemented {
                engine,
                query,
                format,
            } => {
                eprintln!(
                    "{COMMAND_NAME}: {} engine execution is not implemented yet",
                    engine.as_str()
                );
                eprintln!("query: {query}");
                eprintln!("format: {}", format.as_str());
            }
            Self::MalformedProtocolResponse { reason } => {
                eprintln!(
                    "{COMMAND_NAME}: malformed Obsidian protocol response"
                );
                eprintln!("{reason}");
            }
            Self::MissingObsidianCommand { command } => {
                eprintln!(
                    "{COMMAND_NAME}: Obsidian command not found: {}",
                    bob_env::os_to_string(command)
                );
                eprintln!(
                    "Install the Obsidian CLI, start Obsidian, or set \
                     {ENV_OBSIDIAN_COMMAND} to an executable path."
                );
            }
            Self::MissingProtocolSentinel { output } => {
                eprintln!("{COMMAND_NAME}: missing Obsidian protocol response");
                eprintln!(
                    "Expected a {RESULT_PREFIX:?}-prefixed JSON line from \
                     `obsidian eval`."
                );
                if !output.is_empty() {
                    eprintln!("obsidian stdout excerpt: {output}");
                }
            }
            Self::ObsidianFailed { exit_code, output } => {
                eprintln!(
                    "{COMMAND_NAME}: Obsidian CLI eval failed with exit code \
                     {exit_code}"
                );
                if !output.is_empty() {
                    eprintln!("obsidian output excerpt: {output}");
                }
            }
            Self::ObsidianNotRunning { exit_code, output } => {
                eprintln!(
                    "{COMMAND_NAME}: Obsidian is not running or the CLI could \
                     not connect to it (exit code {exit_code})"
                );
                if !output.is_empty() {
                    eprintln!("obsidian output excerpt: {output}");
                }
            }
            Self::ProtocolEngine { code, message } => {
                eprintln!("{COMMAND_NAME}: Obsidian Dataview engine failed");
                eprintln!("{code}: {message}");
            }
            Self::QueryRead {
                path: Some(path),
                error,
            } => {
                eprintln!(
                    "{COMMAND_NAME}: failed to read query file {}: {error}",
                    path.display()
                );
            }
            Self::QueryRead { path: None, error } => {
                eprintln!(
                    "{COMMAND_NAME}: failed to read query from stdin: {error}"
                );
            }
            Self::RunObsidian { command, error } => {
                eprintln!(
                    "{COMMAND_NAME}: failed to run Obsidian command {}: {error}",
                    bob_env::os_to_string(command)
                );
            }
            Self::SerializeOutput(error) => {
                eprintln!(
                    "{COMMAND_NAME}: failed to serialize output JSON: {error}"
                );
            }
            Self::SerializeRequest(error) => {
                eprintln!(
                    "{COMMAND_NAME}: failed to serialize Obsidian eval request: \
                     {error}"
                );
            }
            Self::SyncNotImplemented => {
                eprintln!("{COMMAND_NAME}: --sync is not implemented yet");
                eprintln!("Run sync separately before querying for now.");
            }
        }
    }

    fn exit_code(&self) -> i32 {
        match self {
            Self::ObsidianFailed { exit_code, .. }
            | Self::ObsidianNotRunning { exit_code, .. } => *exit_code,
            _ => 1,
        }
    }
}

fn child_output_excerpt(stdout: &str, stderr: &str) -> String {
    let output = if stderr.trim().is_empty() {
        stdout.trim()
    } else {
        stderr.trim()
    };
    output_excerpt(output)
}

fn stdout_excerpt(stdout: &str) -> String {
    output_excerpt(stdout.trim())
}

fn output_excerpt(output: &str) -> String {
    let redacted = redact_generated_code(output);
    let mut excerpt = redacted.chars().take(600).collect::<String>();
    if redacted.chars().count() > 600 {
        excerpt.push_str("...");
    }
    excerpt
}

fn redact_generated_code(output: &str) -> String {
    if let Some(position) = output.find("code=") {
        let mut redacted = output[..position + "code=".len()].to_string();
        redacted.push_str("<generated JavaScript>");
        return redacted;
    }
    output.to_string()
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

    fn obsidian_eval_request(
        &self,
    ) -> Result<ObsidianEvalRequest, DataviewError> {
        let query = match &self.query {
            QueryInput::Source(source) => ObsidianEvalQuery::Source {
                source: source.clone(),
            },
            QueryInput::Dql(input) => ObsidianEvalQuery::Dql {
                query: input.read_query()?,
            },
        };

        Ok(ObsidianEvalRequest {
            format: self.format.as_str(),
            origin: self
                .vault
                .origin
                .as_ref()
                .map(|path| path.to_string_lossy().into_owned()),
            query,
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

impl DqlInput {
    fn read_query(&self) -> Result<String, DataviewError> {
        match self {
            Self::Inline(query) => Ok(query.clone()),
            Self::File(path) if path.as_os_str() == OsStr::new("-") => {
                let mut query = String::new();
                io::stdin().read_to_string(&mut query).map_err(|error| {
                    DataviewError::QueryRead { path: None, error }
                })?;
                Ok(query)
            }
            Self::File(path) => fs::read_to_string(path).map_err(|error| {
                DataviewError::QueryRead {
                    path: Some(path.clone()),
                    error,
                }
            }),
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
