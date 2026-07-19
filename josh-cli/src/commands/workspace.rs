use std::collections::HashSet;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, anyhow};

const WORKSPACE_FILE: &str = "workspace.josh";

#[derive(Debug, clap::Parser)]
pub struct WorkspaceArgs {
    #[command(subcommand)]
    pub command: WorkspaceCommand,
}

#[derive(Debug, clap::Subcommand)]
pub enum WorkspaceCommand {
    /// Create a workspace definition in the current repository
    Create(WorkspaceCreateArgs),
    /// Add a repository path to the current workspace
    Add(WorkspaceAddArgs),
}

#[derive(Debug, clap::Parser)]
pub struct WorkspaceCreateArgs {
    /// Repository directory that will contain workspace.josh
    pub path: String,

    /// Map DESTINATION to a repository SOURCE path
    #[arg(long = "map", value_name = "DESTINATION=SOURCE")]
    pub mappings: Vec<String>,

    /// Validate and show the result without writing it
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Debug, clap::Parser)]
pub struct WorkspaceAddArgs {
    /// Repository path to add
    pub source: String,

    /// Destination in the workspace
    #[arg(long = "as", value_name = "DESTINATION")]
    pub destination: String,

    /// Validate and show the result without writing it
    #[arg(long)]
    pub dry_run: bool,
}

pub fn handle_workspace(
    args: &WorkspaceArgs,
    transaction: &josh_core::cache::Transaction,
) -> anyhow::Result<()> {
    match &args.command {
        WorkspaceCommand::Create(args) => handle_create(args, transaction.repo()),
        WorkspaceCommand::Add(args) => handle_add(args, transaction.repo()),
    }
}

fn validate_repository_path(path: &str, kind: &str) -> anyhow::Result<()> {
    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.contains('\\')
        || path
            .split('/')
            .any(|component| component.is_empty() || matches!(component, "." | ".."))
    {
        return Err(anyhow!(
            "{kind} '{path}' must be a relative repository path"
        ));
    }
    Ok(())
}

fn workspace_directory(path: &str) -> anyhow::Result<PathBuf> {
    if path == "." {
        return Ok(PathBuf::new());
    }
    validate_repository_path(path, "Workspace path")?;
    Ok(PathBuf::from(path))
}

fn parse_mapping(mapping: &str) -> anyhow::Result<(String, josh_core::filter::Filter)> {
    let (destination, source) = mapping
        .split_once('=')
        .ok_or_else(|| anyhow!("Invalid mapping '{mapping}'; expected DESTINATION=SOURCE"))?;
    let destination = destination.trim();
    let source = source.trim();

    validate_repository_path(destination, "Mapping destination")?;
    if source.is_empty() {
        return Err(anyhow!("Mapping '{mapping}' has an empty source"));
    }

    validate_repository_path(source, "Mapping source")?;
    let filter = josh_core::filter::Filter::new().subdir(source);

    Ok((destination.to_string(), filter.prefix(destination)))
}

fn create_content(mappings: &[String]) -> anyhow::Result<String> {
    if mappings.is_empty() {
        return Ok("# Empty Josh workspace\n".to_string());
    }

    let mut destinations = HashSet::new();
    let mut filters = Vec::new();
    for mapping in mappings {
        let (destination, filter) = parse_mapping(mapping)?;
        if !destinations.insert(destination.clone()) {
            return Err(anyhow!(
                "Duplicate workspace mapping destination '{destination}'"
            ));
        }
        filters.push(filter);
    }

    let filter = josh_core::filter::to_filter(josh_core::filter::Op::Compose(filters));
    josh_core::filter::invert(filter).context("Workspace mappings are not reversible")?;
    Ok(josh_core::filter::as_file(filter, 0))
}

fn add_content(content: &str, destination: &str, source: &str) -> anyhow::Result<String> {
    let mapping = format!("{destination}={source}");
    let (_, filter) = parse_mapping(&mapping)?;
    let mut result = content.to_string();
    if !result.is_empty() && !result.ends_with('\n') {
        result.push('\n');
    }
    result.push_str(&josh_core::filter::as_file(filter, 0));

    let combined = josh_core::filter::parse(&result).context("Invalid workspace definition")?;
    josh_core::filter::invert(combined).context("Workspace mappings are not reversible")?;
    Ok(result)
}

fn workspace_name(path: &Path) -> String {
    if path.as_os_str().is_empty() {
        ".".to_string()
    } else {
        path.to_string_lossy().replace('\\', "/")
    }
}

fn handle_create(args: &WorkspaceCreateArgs, repo: &git2::Repository) -> anyhow::Result<()> {
    let root = repo
        .workdir()
        .context("Workspace commands require a non-bare Git repository")?;
    let workspace = workspace_directory(&args.path)?;
    let file = root.join(&workspace).join(WORKSPACE_FILE);
    if file.exists() {
        return Err(anyhow!(
            "Workspace '{}' already exists",
            workspace_name(&workspace)
        ));
    }

    let content = create_content(&args.mappings)?;
    let relative_file = file.strip_prefix(root).unwrap_or(&file);
    if args.dry_run {
        println!("Would create workspace '{}'", workspace_name(&workspace));
        println!("Definition: {}", relative_file.display());
        if !args.mappings.is_empty() {
            println!("{}", content.trim_end());
        }
        return Ok(());
    }

    let parent = file.parent().context("Workspace file has no parent")?;
    std::fs::create_dir_all(parent)?;
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&file)
        .with_context(|| format!("Failed to create workspace '{}'", file.display()))?;
    output.write_all(content.as_bytes())?;

    println!("Created workspace '{}'", workspace_name(&workspace));
    println!("Definition: {}", relative_file.display());
    Ok(())
}

fn current_workspace_file(repo: &git2::Repository) -> anyhow::Result<PathBuf> {
    let root = repo
        .workdir()
        .context("Workspace commands require a non-bare Git repository")?;
    let current = std::env::current_dir()?;
    if !current.starts_with(root) {
        return Err(anyhow!("Current directory is outside the repository"));
    }

    let mut directory = current.clone();
    loop {
        let file = directory.join(WORKSPACE_FILE);
        if file.is_file() {
            return Ok(file);
        }
        if directory == root {
            break;
        }
        directory.pop();
    }
    Ok(current.join(WORKSPACE_FILE))
}

fn handle_add(args: &WorkspaceAddArgs, repo: &git2::Repository) -> anyhow::Result<()> {
    let root = repo
        .workdir()
        .context("Workspace commands require a non-bare Git repository")?;
    let file = current_workspace_file(repo)?;
    let existing = match std::fs::read_to_string(&file) {
        Ok(content) => content,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => String::new(),
        Err(error) => return Err(error.into()),
    };
    let content = add_content(&existing, &args.destination, &args.source)?;
    let relative_file = file.strip_prefix(root).unwrap_or(&file);

    if args.dry_run {
        println!("Would map '{}' to '{}'", args.source, args.destination);
        println!("Definition: {}", relative_file.display());
        println!("{}", content.trim_end());
        return Ok(());
    }

    let parent = file.parent().context("Workspace file has no parent")?;
    std::fs::create_dir_all(parent)?;
    std::fs::write(&file, content)
        .with_context(|| format!("Failed to update workspace '{}'", file.display()))?;

    println!("Mapped '{}' to '{}'", args.source, args.destination);
    println!("Definition: {}", relative_file.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_path_mappings() {
        let content = create_content(&[
            "app=apps/frontend".to_string(),
            "shared=libs/shared".to_string(),
        ])
        .unwrap();

        assert_eq!(content, "app = :/apps/frontend\nshared = :/libs/shared\n");
        let filter = josh_core::filter::parse(&content).unwrap();
        assert!(josh_core::filter::invert(filter).is_ok());
    }

    #[test]
    fn empty_workspace_is_valid() {
        let content = create_content(&[]).unwrap();
        let filter = josh_core::filter::parse(&content).unwrap();

        assert_eq!(content, "# Empty Josh workspace\n");
        assert!(josh_core::filter::invert(filter).is_ok());
    }

    #[test]
    fn repository_paths_cannot_inject_filters() {
        let source = "apps/frontend:prefix=elsewhere";
        let content = create_content(&[format!("app={source}")]).unwrap();
        let actual = josh_core::filter::parse(&content).unwrap();
        let expected = josh_core::filter::Filter::new()
            .subdir(source)
            .prefix("app");

        assert_eq!(actual, expected);
    }

    #[test]
    fn adds_mapping_without_rewriting_existing_content() {
        let existing = "# Application workspace\napp = :/apps/frontend\n";
        let content = add_content(existing, "shared", "libs/shared").unwrap();

        assert_eq!(
            content,
            "# Application workspace\napp = :/apps/frontend\nshared = :/libs/shared\n"
        );
    }

    #[test]
    fn rejects_invalid_mappings() {
        for mapping in [
            "app",
            "app=",
            "../app=apps/frontend",
            "app=../apps/frontend",
        ] {
            assert!(parse_mapping(mapping).is_err());
        }
    }

    #[test]
    fn workspace_path_cannot_escape_repository() {
        assert_eq!(
            workspace_directory("workspaces/app").unwrap(),
            Path::new("workspaces/app")
        );
        assert!(workspace_directory("../app").is_err());
        assert!(workspace_directory("/app").is_err());
        assert!(workspace_directory("workspaces\\app").is_err());
    }
}
