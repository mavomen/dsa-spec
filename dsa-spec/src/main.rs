//! CLI entrypoint for dsa-spec.
//! Handles argument parsing, backend dispatch, and file I/O.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

mod assertion;
mod ast;
mod backend;
mod casing;
mod complexity;
mod contracts;
mod csharp_backend;
mod error;
mod go_backend;
mod migrate;
mod parser;
mod python_backend;
mod rust_backend;
mod spec_schema;
mod template_engine;
mod typescript_backend;
mod validator;
mod visualization;

mod doc_gen;

use backend::Backend;

/// Result type for backend instantiation across one or more languages.
type BackendResult = Result<Vec<(&'static str, Box<dyn Backend>)>, Box<dyn std::error::Error>>;

#[derive(Parser)]
#[command(name = "dsa-spec")]
#[command(about = "Generate code skeletons from DSA specifications")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Verbosity level (-v for info, -vv for debug)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    verbose: u8,

    /// Machine-readable JSON output
    #[arg(long, global = true)]
    json: bool,
}

/// CLI subcommands.
#[derive(Subcommand)]
enum Command {
    /// Generate code stubs
    Generate {
        /// Path to the spec YAML file
        spec: PathBuf,

        /// Target language: rust, python, csharp, typescript, go, or all
        #[arg(short, long, default_value = "rust")]
        lang: String,

        /// Output file (single language) — stdout if omitted
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output directory (for --lang all). Defaults to "generated/"
        #[arg(long)]
        output_dir: Option<PathBuf>,

        /// Inject contract assertions into generated stubs
        #[arg(long)]
        contracts: bool,
    },
    /// Validate a specification
    Validate {
        /// Path to the spec YAML file
        spec: PathBuf,
    },
    /// Analyze complexity across DSA specifications
    Analyze {
        /// Path to specs directory
        #[arg(default_value = "specs")]
        dir: PathBuf,

        /// Output format: table, markdown, json, chart
        #[arg(short, long, default_value = "table")]
        format: String,
    },
    /// Visualize data structures or algorithms as diagrams
    Visualize {
        /// Path to the spec YAML file
        spec: PathBuf,

        /// Output format: dot/graphviz or mermaid
        #[arg(short, long, default_value = "dot")]
        format: String,
    },
    /// Migrate a spec file to a newer spec version
    Migrate {
        /// Path to the spec YAML file
        spec: PathBuf,

        /// Target spec version (default: latest)
        #[arg(short, long, default_value = "2.0")]
        target_version: String,
    },
    /// Generate markdown documentation from a spec
    Doc {
        /// Path to the spec YAML file
        spec: PathBuf,

        /// Output file (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Verify contracts — generates stubs with contract assertions
    Verify {
        /// Path to the spec YAML file
        spec: PathBuf,

        /// Target language: rust, python, csharp, typescript, go, or all
        #[arg(short, long, default_value = "rust")]
        lang: String,

        /// Verification backend (runtime is the only supported option)
        #[arg(long, default_value = "runtime")]
        backend: String,
    },
}

/// Program entrypoint. Parses arguments, dispatches to the appropriate
/// command handler, and prints errors to stderr.
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let filter = match cli.verbose {
        0 => "dsa_spec=warn",
        1 => "dsa_spec=info",
        _ => "dsa_spec=debug",
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_new(filter).unwrap_or_else(|_| EnvFilter::new("warn")))
        .try_init();

    let use_json = cli.json;

    let make_backends = |lang_lower: &str| -> BackendResult {
        match lang_lower {
            "rust" => Ok(vec![(
                "rust",
                Box::new(rust_backend::RustBackend::new("templates")?) as Box<dyn Backend>,
            )]),
            "python" => Ok(vec![(
                "python",
                Box::new(python_backend::PythonBackend::new("templates")?),
            )]),
            "csharp" | "c#" => Ok(vec![(
                "csharp",
                Box::new(csharp_backend::CSharpBackend::new("templates")?),
            )]),
            "typescript" | "ts" => Ok(vec![(
                "typescript",
                Box::new(typescript_backend::TypeScriptBackend::new("templates")?),
            )]),
            "go" => Ok(vec![(
                "go",
                Box::new(go_backend::GoBackend::new("templates")?),
            )]),
            "all" => Ok(vec![
                (
                    "rust",
                    Box::new(rust_backend::RustBackend::new("templates")?) as Box<dyn Backend>,
                ),
                (
                    "python",
                    Box::new(python_backend::PythonBackend::new("templates")?),
                ),
                (
                    "csharp",
                    Box::new(csharp_backend::CSharpBackend::new("templates")?),
                ),
                (
                    "typescript",
                    Box::new(typescript_backend::TypeScriptBackend::new("templates")?),
                ),
                ("go", Box::new(go_backend::GoBackend::new("templates")?)),
            ]),
            _ => {
                tracing::error!(lang = %lang_lower, "unsupported language");
                eprintln!(
                    "Unsupported language: {lang_lower}. Use rust, python, csharp, typescript, go, or all."
                );
                std::process::exit(1);
            }
        }
    };

    let _lang_ext = |name: &str| -> &str {
        match name {
            "rust" => "rs",
            "python" => "py",
            "csharp" => "cs",
            "typescript" => "ts",
            "go" => "go",
            _ => "txt",
        }
    };

    let json_str = |val: &serde_json::Value| -> String {
        serde_json::to_string(val).unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.into())
    };

    match cli.command {
        Command::Generate {
            spec,
            lang,
            output,
            output_dir,
            contracts,
        } => {
            let yaml = fs::read_to_string(&spec)?;
            tracing::info!(path = %spec.display(), "parsing spec");
            let parsed = parser::parse(&yaml)?;

            let active_spec = if contracts {
                tracing::info!("injecting contract assertions");
                contracts::inject_assertions(&parsed)
            } else {
                parsed
            };

            let lang_lower = lang.to_lowercase();
            let backends = make_backends(&lang_lower)?;
            let _spec_stem = spec.file_stem().unwrap().to_string_lossy().to_string();

            let mut results: Vec<(&str, Vec<(String, String)>)> = Vec::new();
            for (lang_name, backend) in &backends {
                tracing::info!(lang = %lang_name, "generating code");
                let code = backend.generate(&active_spec)?;
                results.push((lang_name, code));
            }

            let out_dir = output_dir.unwrap_or_else(|| {
                if lang_lower == "all" {
                    PathBuf::from("generated")
                } else {
                    PathBuf::new()
                }
            });

            for (lang_name, files) in &results {
                match &output {
                    Some(path) if lang_lower != "all" => {
                        let combined: String = files
                            .iter()
                            .map(|(_, code)| code.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        fs::write(path, combined)?;
                        tracing::info!(out = %path.display(), "wrote output");
                    }
                    _ if lang_lower == "all" || !out_dir.as_os_str().is_empty() => {
                        fs::create_dir_all(&out_dir)?;
                        for (file_name, file_code) in files {
                            let out_path = out_dir.join(file_name);
                            fs::write(&out_path, file_code)?;
                            tracing::info!(out = %out_path.display(), "wrote output");
                        }
                    }
                    _ => {
                        let combined: String = files
                            .iter()
                            .map(|(_, code)| code.as_str())
                            .collect::<Vec<_>>()
                            .join("\n");
                        if use_json {
                            let entry = serde_json::json!({"lang": lang_name, "code": combined});
                            println!("{}", json_str(&entry));
                        } else {
                            println!("--- {lang_name} ---\n{combined}");
                        }
                    }
                }
            }
        }
        Command::Validate { spec } => {
            let yaml = fs::read_to_string(&spec)?;
            tracing::info!(path = %spec.display(), "validating spec");
            let parsed = parser::parse(&yaml)?;

            let mut warnings: Vec<String> = Vec::new();
            if let Some(warn) = validator::validate_category_dir(&parsed, &spec) {
                warnings.push(warn.to_string());
            }

            match validator::validate(&parsed) {
                Ok(()) => {
                    for w in &warnings {
                        tracing::warn!(warning = %w, "category/directory mismatch");
                        eprintln!("Warning: {w}");
                    }
                    if use_json {
                        println!(
                            r#"{{"valid":true,"warnings":{}}}"#,
                            serde_json::to_string(&warnings).unwrap_or_default()
                        );
                    } else {
                        println!("Spec is valid.");
                    }
                }
                Err(errs) => {
                    for e in &errs {
                        tracing::error!(error = %e, "validation error");
                    }
                    if use_json {
                        let err_list: Vec<String> = errs.iter().map(|e| e.to_string()).collect();
                        println!(
                            r#"{{"valid":false,"errors":{}}}"#,
                            serde_json::to_string(&err_list).unwrap()
                        );
                    } else {
                        eprintln!("Validation errors:");
                        for e in errs {
                            eprintln!("  - {e}");
                        }
                    }
                    std::process::exit(1);
                }
            }
        }
        Command::Analyze { dir, format } => {
            tracing::info!(dir = %dir.display(), "analyzing specs");
            let specs = match complexity::load_specs_from_dir(&dir.to_string_lossy()) {
                Ok(s) => s,
                Err(errs) => {
                    for e in &errs {
                        tracing::error!(error = %e, "analyze error");
                        eprintln!("  - {e}");
                    }
                    std::process::exit(1);
                }
            };
            if use_json || format.to_lowercase() == "json" {
                println!("{}", complexity::generate_json_report(&specs));
            } else if format.to_lowercase() == "chart" {
                println!("{}", complexity::generate_tradeoff_chart(&specs));
            } else {
                println!("{}", complexity::generate_report(&specs));
            }
        }
        Command::Visualize { spec, format } => {
            let yaml = fs::read_to_string(&spec)?;
            let parsed = parser::parse(&yaml)?;
            let output = visualization::generate(&parsed, &format);
            if use_json {
                let entry = serde_json::json!({"format": format, "content": output});
                println!("{}", json_str(&entry));
            } else {
                println!("{output}");
            }
        }
        Command::Migrate {
            spec,
            target_version,
        } => {
            tracing::info!(path = %spec.display(), target = %target_version, "migrating spec");
            match migrate::migrate_spec_file(&spec.to_string_lossy(), &target_version) {
                Ok(()) => {
                    let bak_path = format!("{}.bak", spec.display());
                    if use_json {
                        println!(
                            r#"{{"status":"ok","path":"{}","backup":"{}"}}"#,
                            spec.display(),
                            bak_path
                        );
                    } else {
                        println!("Migrated {} to version {}", spec.display(), target_version);
                        println!("Backup saved as {bak_path}");
                    }
                }
                Err(e) => {
                    tracing::error!(error = %e, "migration failed");
                    eprintln!("Migration error: {e}");
                    std::process::exit(1);
                }
            }
        }
        Command::Doc { spec, output } => {
            let yaml = fs::read_to_string(&spec)?;
            let parsed = parser::parse(&yaml)?;
            let doc = doc_gen::generate_doc(&parsed);
            match output {
                Some(path) => {
                    fs::write(&path, &doc)?;
                    if use_json {
                        let entry = serde_json::json!({"path": path});
                        println!("{}", json_str(&entry));
                    } else {
                        println!("Documentation written to {}", path.display());
                    }
                }
                None => {
                    if use_json {
                        let entry = serde_json::json!({"doc": doc});
                        println!("{}", json_str(&entry));
                    } else {
                        println!("{}", doc);
                    }
                }
            }
        }
        Command::Verify {
            spec,
            lang,
            backend,
        } => {
            if backend != "runtime" {
                tracing::error!(backend = %backend, "unsupported verification backend");
                eprintln!(
                    "Unsupported verification backend: {backend}. Only 'runtime' is supported."
                );
                std::process::exit(1);
            }

            let yaml = fs::read_to_string(&spec)?;
            let parsed = parser::parse(&yaml)?;
            let spec = contracts::inject_assertions(&parsed);

            let lang_lower = lang.to_lowercase();
            let backends = make_backends(&lang_lower)?;

            for (lang_name, backend) in &backends {
                let files = backend.generate(&spec)?;
                let combined: String = files
                    .iter()
                    .map(|(_, code)| code.as_str())
                    .collect::<Vec<_>>()
                    .join("\n");
                if use_json {
                    let entry = serde_json::json!({"lang": lang_name, "code": combined});
                    println!("{}", json_str(&entry));
                } else {
                    println!("--- {lang_name} ---\n{combined}");
                }
            }
        }
    }
    Ok(())
}
