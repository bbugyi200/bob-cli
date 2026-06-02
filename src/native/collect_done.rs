use std::{
    ffi::{OsStr, OsString},
    fs, io,
    path::{Component, Path, PathBuf},
};

use super::env as bob_env;

const COMMAND_NAME: &str = "bob collect-done";
const DEFAULT_THRESHOLD: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Args {
    threshold: usize,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            threshold: DEFAULT_THRESHOLD,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CollectionPlan {
    scanned_files: usize,
    files: Vec<FilePlan>,
}

impl CollectionPlan {
    fn total_task_count(&self) -> usize {
        self.files.iter().map(|file| file.task_count).sum()
    }

    fn planned_bytes(&self) -> usize {
        self.files
            .iter()
            .map(|file| file.source_contents.len() + file.archive_append.len())
            .sum()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FilePlan {
    relative_source_path: PathBuf,
    relative_archive_path: PathBuf,
    task_count: usize,
    source_contents: String,
    archive_append: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Transform {
    task_count: usize,
    source_contents: String,
    archive_append: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TaskLine {
    indent: usize,
}

pub(crate) fn run(args: Vec<OsString>) -> i32 {
    match parse_args(args) {
        ParseResult::Run(args) => run_collect_done(args),
        ParseResult::Help => {
            print_help();
            0
        }
        ParseResult::Error(message) => {
            eprintln!("{COMMAND_NAME}: {message}");
            eprintln!("Try '{COMMAND_NAME} --help' for more information.");
            2
        }
    }
}

fn run_collect_done(args: Args) -> i32 {
    let vault = bob_env::bob_dir();
    let plan = match build_collection_plan(&vault, args.threshold) {
        Ok(plan) => plan,
        Err(error) => {
            eprintln!(
                "{COMMAND_NAME}: failed to scan {}: {error}",
                vault.display()
            );
            return 1;
        }
    };

    println!("Collect done tasks");
    println!("vault: {}", vault.display());
    println!("threshold: {}", args.threshold);
    println!("scan:");
    println!("  markdown files: {}", plan.scanned_files);
    println!("  files meeting threshold: {}", plan.files.len());
    println!("  task blocks: {}", plan.total_task_count());
    println!("  planned bytes: {}", plan.planned_bytes());

    if plan.files.is_empty() {
        println!(
            "summary: no task blocks met the threshold; no vault changes made."
        );
        return 0;
    }

    println!("moves:");
    for file in &plan.files {
        println!(
            "  {} -> {} ({} task blocks)",
            file.relative_source_path.display(),
            file.relative_archive_path.display(),
            file.task_count
        );
    }
    println!("summary: scan plan built in memory; no vault changes made.");
    0
}

fn build_collection_plan(
    vault: &Path,
    threshold: usize,
) -> io::Result<CollectionPlan> {
    let markdown_files = markdown_files(vault)?;
    let mut files = Vec::new();

    for path in &markdown_files {
        let contents = fs::read_to_string(path)?;
        let transform = transform_markdown(&contents);
        if transform.task_count < threshold {
            continue;
        }

        let relative_source_path = path
            .strip_prefix(vault)
            .map_err(|error| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!(
                        "source path {} is outside vault {}: {error}",
                        path.display(),
                        vault.display()
                    ),
                )
            })?
            .to_path_buf();
        let relative_archive_path =
            archive_relative_path(&relative_source_path)?;

        files.push(FilePlan {
            relative_source_path,
            relative_archive_path,
            task_count: transform.task_count,
            source_contents: transform.source_contents,
            archive_append: transform.archive_append,
        });
    }

    Ok(CollectionPlan {
        scanned_files: markdown_files.len(),
        files,
    })
}

fn markdown_files(vault: &Path) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_markdown_files(vault, vault, &mut files)?;
    files.sort();
    Ok(files)
}

fn collect_markdown_files(
    vault: &Path,
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> io::Result<()> {
    let mut entries =
        fs::read_dir(directory)?.collect::<Result<Vec<_>, io::Error>>()?;
    entries.sort_by_key(|entry| entry.path());

    for entry in entries {
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            if should_skip_directory(vault, &path) {
                continue;
            }
            collect_markdown_files(vault, &path, files)?;
        } else if file_type.is_file() && is_markdown_file(&path) {
            files.push(path);
        }
    }

    Ok(())
}

fn should_skip_directory(vault: &Path, directory: &Path) -> bool {
    let relative = directory.strip_prefix(vault).unwrap_or(directory);
    relative.components().any(|component| {
        matches!(
            component,
            Component::Normal(name)
                if name == OsStr::new("done")
                    || name == OsStr::new(".git")
                    || name == OsStr::new(".obsidian")
        )
    })
}

fn is_markdown_file(path: &Path) -> bool {
    path.extension()
        .and_then(OsStr::to_str)
        .map(|extension| extension.eq_ignore_ascii_case("md"))
        .unwrap_or(false)
}

fn archive_relative_path(source_relative_path: &Path) -> io::Result<PathBuf> {
    let stem = source_relative_path.file_stem().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "source path has no file stem: {}",
                source_relative_path.display()
            ),
        )
    })?;
    let mut archive_name = OsString::from(stem);
    archive_name.push("_done.md");

    let mut archive_path = PathBuf::from("done");
    if let Some(parent) = source_relative_path.parent()
        && !parent.as_os_str().is_empty()
    {
        archive_path.push(parent);
    }
    archive_path.push(archive_name);
    Ok(archive_path)
}

fn transform_markdown(contents: &str) -> Transform {
    let lines: Vec<&str> = contents.split_inclusive('\n').collect();
    let mut source_contents = String::with_capacity(contents.len());
    let mut archive_append = String::new();
    let mut task_count = 0;
    let mut index = 0;

    while index < lines.len() {
        let Some(task_line) = collectible_task_line(lines[index]) else {
            source_contents.push_str(lines[index]);
            index += 1;
            continue;
        };

        let end = task_block_end(&lines, index, task_line.indent);
        task_count += 1;
        for line in &lines[index..end] {
            archive_append.push_str(line);
        }
        index = end;
    }

    Transform {
        task_count,
        source_contents,
        archive_append,
    }
}

fn task_block_end(lines: &[&str], start: usize, task_indent: usize) -> usize {
    let mut index = start + 1;
    let mut include_end = start + 1;
    let mut pending_blank = false;

    while index < lines.len() {
        let (content, _) = split_line_ending(lines[index]);
        if content.trim().is_empty() {
            pending_blank = true;
            index += 1;
            continue;
        }

        if leading_indent_len(content) > task_indent {
            pending_blank = false;
            index += 1;
            include_end = index;
            continue;
        }

        break;
    }

    if pending_blank && index == lines.len() {
        include_end = index;
    }

    include_end
}

fn collectible_task_line(line: &str) -> Option<TaskLine> {
    let (content, _) = split_line_ending(line);
    let indent = leading_indent_len(content);
    let rest = &content[indent..];
    let rest = strip_list_marker(rest)?.trim_start();
    let checkbox = rest.get(..3)?;

    if !matches!(checkbox, "[x]" | "[X]" | "[-]") {
        return None;
    }

    let after_checkbox = &rest[3..];
    if !after_checkbox.is_empty()
        && !after_checkbox.starts_with(char::is_whitespace)
    {
        return None;
    }

    has_task_tag(content).then_some(TaskLine { indent })
}

fn strip_list_marker(line: &str) -> Option<&str> {
    let first = line.chars().next()?;
    if matches!(first, '-' | '*' | '+') {
        let after_marker = &line[first.len_utf8()..];
        if after_marker.starts_with(char::is_whitespace) {
            return Some(after_marker);
        }
    }

    let digit_len = line
        .bytes()
        .take_while(|byte| byte.is_ascii_digit())
        .count();
    if digit_len == 0 {
        return None;
    }

    let after_digits = &line[digit_len..];
    let marker = after_digits.chars().next()?;
    if !matches!(marker, '.' | ')') {
        return None;
    }

    let after_marker = &after_digits[marker.len_utf8()..];
    after_marker
        .starts_with(char::is_whitespace)
        .then_some(after_marker)
}

fn has_task_tag(text: &str) -> bool {
    let mut rest = text;
    while let Some(index) = rest.find("#task") {
        let after_index = index + "#task".len();
        let after = rest[after_index..].chars().next();
        if after.map(is_task_tag_boundary).unwrap_or(true) {
            return true;
        }
        rest = &rest[after_index..];
    }

    false
}

fn is_task_tag_boundary(character: char) -> bool {
    !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
}

fn leading_indent_len(line: &str) -> usize {
    line.char_indices()
        .find_map(|(index, character)| {
            (!matches!(character, ' ' | '\t')).then_some(index)
        })
        .unwrap_or(line.len())
}

fn split_line_ending(line: &str) -> (&str, &str) {
    if let Some(content) = line.strip_suffix("\r\n") {
        return (content, "\r\n");
    }
    if let Some(content) = line.strip_suffix('\n') {
        return (content, "\n");
    }
    (line, "")
}

fn parse_args(args: Vec<OsString>) -> ParseResult {
    let mut parsed = Args::default();
    let mut args = args.into_iter();

    while let Some(arg) = args.next() {
        let text = bob_env::os_to_string(&arg);
        match text.as_str() {
            "-h" | "--help" => return ParseResult::Help,
            "--threshold" => {
                let Some(value) = args.next() else {
                    return ParseResult::Error(
                        "option --threshold requires a value".to_string(),
                    );
                };
                parsed.threshold = match parse_threshold(&value) {
                    Ok(threshold) => threshold,
                    Err(message) => return ParseResult::Error(message),
                };
            }
            "--" => {
                if let Some(extra) = args.next() {
                    return ParseResult::Error(format!(
                        "unexpected positional argument: {}",
                        bob_env::os_to_string(&extra)
                    ));
                }
            }
            _ if let Some(value) = text.strip_prefix("--threshold=") => {
                parsed.threshold = match parse_threshold_text(value) {
                    Ok(threshold) => threshold,
                    Err(message) => return ParseResult::Error(message),
                };
            }
            _ if text.starts_with('-') => {
                return ParseResult::Error(format!(
                    "unrecognized argument: {text}"
                ));
            }
            _ => {
                return ParseResult::Error(format!(
                    "unexpected positional argument: {text}"
                ));
            }
        }
    }

    ParseResult::Run(parsed)
}

enum ParseResult {
    Run(Args),
    Help,
    Error(String),
}

fn parse_threshold(value: &OsString) -> Result<usize, String> {
    parse_threshold_text(&bob_env::os_to_string(value))
}

fn parse_threshold_text(value: &str) -> Result<usize, String> {
    let threshold = value
        .parse::<usize>()
        .map_err(|_| format!("invalid --threshold value: {value}"))?;
    if threshold == 0 {
        return Err("--threshold must be at least 1".to_string());
    }

    Ok(threshold)
}

fn print_help() {
    println!(
        "\
usage: {COMMAND_NAME} [--threshold N]

Collect done and canceled Bob task blocks into archive notes.

options:
  -h, --help       show this help message and exit
  --threshold N    minimum completed/canceled task count per source note \
(default: {DEFAULT_THRESHOLD})"
    );
}

#[cfg(test)]
mod tests {
    use super::{
        archive_relative_path, build_collection_plan, parse_args,
        transform_markdown, Args, ParseResult, DEFAULT_THRESHOLD,
    };
    use std::{
        ffi::OsString,
        fs, io,
        path::{Path, PathBuf},
        sync::atomic::{AtomicUsize, Ordering},
        time::{SystemTime, UNIX_EPOCH},
    };

    static TEMP_COUNTER: AtomicUsize = AtomicUsize::new(0);

    #[test]
    fn parses_default_threshold() {
        match parse_args(vec![]) {
            ParseResult::Run(args) => {
                assert_eq!(args.threshold, DEFAULT_THRESHOLD);
            }
            _ => panic!("expected runnable args"),
        }
    }

    #[test]
    fn parses_threshold_option() {
        match parse_args(os_args(["--threshold", "15"])) {
            ParseResult::Run(args) => assert_eq!(args, Args { threshold: 15 }),
            _ => panic!("expected runnable args"),
        }
    }

    #[test]
    fn parses_threshold_equals_option() {
        match parse_args(os_args(["--threshold=3"])) {
            ParseResult::Run(args) => assert_eq!(args, Args { threshold: 3 }),
            _ => panic!("expected runnable args"),
        }
    }

    #[test]
    fn rejects_zero_threshold() {
        match parse_args(os_args(["--threshold", "0"])) {
            ParseResult::Error(message) => {
                assert!(message.contains("at least 1"));
            }
            _ => panic!("expected parse error"),
        }
    }

    #[test]
    fn recognizes_done_and_canceled_task_lines_only() {
        let transform = transform_markdown(
            "\
- [x] done #task
- [X] uppercase done #task
- [-] canceled #task
- [ ] active #task
- [/] in progress #task
- [x] done without task tag
- [x] not quite #tasks
",
        );

        assert_eq!(transform.task_count, 3);
        assert_eq!(
            transform.archive_append,
            "\
- [x] done #task
- [X] uppercase done #task
- [-] canceled #task
"
        );
        assert_eq!(
            transform.source_contents,
            "\
- [ ] active #task
- [/] in progress #task
- [x] done without task tag
- [x] not quite #tasks
"
        );
    }

    #[test]
    fn extracts_nested_blocks_and_continuations() {
        let transform = transform_markdown(include_str!(
            "../../tests/fixtures/collect_done/nested_blocks.md"
        ));

        assert_eq!(transform.task_count, 1);
        assert_eq!(
            transform.source_contents,
            include_str!(
                "../../tests/fixtures/collect_done/nested_blocks_source.md"
            )
        );
        assert_eq!(
            transform.archive_append,
            include_str!(
                "../../tests/fixtures/collect_done/nested_blocks_archive.md"
            )
        );
    }

    #[test]
    fn completed_child_moves_without_collecting_active_parent() {
        let transform = transform_markdown(
            "\
- [ ] active parent #task
  - [x] done child #task
    child continuation
  - [/] active child #task
",
        );

        assert_eq!(transform.task_count, 1);
        assert_eq!(
            transform.source_contents,
            "\
- [ ] active parent #task
  - [/] active child #task
"
        );
        assert_eq!(
            transform.archive_append,
            "  - [x] done child #task\n    child continuation\n"
        );
    }

    #[test]
    fn preserves_line_endings_in_source_and_archive() {
        let transform = transform_markdown(
            "- [x] done #task\r\n  detail\r\n- [ ] keep #task\r\n",
        );

        assert_eq!(transform.task_count, 1);
        assert_eq!(transform.source_contents, "- [ ] keep #task\r\n");
        assert_eq!(
            transform.archive_append,
            "- [x] done #task\r\n  detail\r\n"
        );
    }

    #[test]
    fn maps_source_notes_to_archive_notes() {
        assert_eq!(
            archive_relative_path(Path::new("obsidian.md")).unwrap(),
            PathBuf::from("done/obsidian_done.md")
        );
        assert_eq!(
            archive_relative_path(Path::new("foo/bar.md")).unwrap(),
            PathBuf::from("done/foo/bar_done.md")
        );
    }

    #[test]
    fn scans_markdown_files_with_exclusions_and_threshold() {
        let vault = TempDir::new("bob-cli-collect-done-vault");
        write_file(
            &vault.path().join("obsidian.md"),
            "\
- [x] one #task
- [-] two #task
",
        );
        write_file(&vault.path().join("foo/bar.md"), "- [x] nested #task\n");
        write_file(&vault.path().join("foo/not-markdown.txt"), "#task\n");
        write_file(&vault.path().join("done/old.md"), "- [x] archived #task\n");
        write_file(&vault.path().join(".git/config.md"), "- [x] git #task\n");
        write_file(
            &vault.path().join(".obsidian/settings.md"),
            "- [x] settings #task\n",
        );

        let plan = build_collection_plan(vault.path(), 2).expect("build plan");

        assert_eq!(plan.scanned_files, 2);
        assert_eq!(plan.files.len(), 1);
        let file = &plan.files[0];
        assert_eq!(file.relative_source_path, PathBuf::from("obsidian.md"));
        assert_eq!(
            file.relative_archive_path,
            PathBuf::from("done/obsidian_done.md")
        );
        assert_eq!(file.task_count, 2);
        assert!(file.source_contents.is_empty());
        assert_eq!(
            file.archive_append,
            "\
- [x] one #task
- [-] two #task
"
        );
    }

    #[test]
    fn includes_nested_path_note_when_it_meets_threshold() {
        let vault = TempDir::new("bob-cli-collect-done-nested-vault");
        write_file(&vault.path().join("foo/bar.md"), "- [x] nested #task\n");

        let plan = build_collection_plan(vault.path(), 1).expect("build plan");

        assert_eq!(plan.files.len(), 1);
        assert_eq!(
            plan.files[0].relative_source_path,
            PathBuf::from("foo/bar.md")
        );
        assert_eq!(
            plan.files[0].relative_archive_path,
            PathBuf::from("done/foo/bar_done.md")
        );
    }

    fn os_args<const N: usize>(args: [&str; N]) -> Vec<OsString> {
        args.into_iter().map(OsString::from).collect()
    }

    fn write_file(path: &Path, contents: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap_or_else(|error| {
                panic!("create parent {}: {error}", parent.display())
            });
        }
        fs::write(path, contents).unwrap_or_else(|error| {
            panic!("write {}: {error}", path.display())
        });
    }

    struct TempDir {
        path: PathBuf,
    }

    impl TempDir {
        fn new(prefix: &str) -> Self {
            let mut path = std::env::temp_dir();
            path.push(format!(
                "{}-{}-{}-{}",
                prefix,
                std::process::id(),
                current_time_nanos(),
                TEMP_COUNTER.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap_or_else(|error| {
                panic!("create temp dir {}: {error}", path.display())
            });
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            if let Err(error) = remove_dir_all_if_exists(&self.path) {
                eprintln!(
                    "failed to remove temp dir {}: {error}",
                    self.path.display()
                );
            }
        }
    }

    fn current_time_nanos() -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before epoch")
            .as_nanos()
    }

    fn remove_dir_all_if_exists(path: &Path) -> io::Result<()> {
        match fs::remove_dir_all(path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }
}
