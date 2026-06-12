use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;

mod ast;
mod backend;
mod complexity;
mod contracts;
mod csharp_backend;
mod error;
mod go_backend;
mod parser;
mod python_backend;
mod rust_backend;
mod spec_schema;
mod template_engine;
mod typescript_backend;
mod validator;
mod visualization;

use backend::Backend;

#[derive(Parser)]
#[command(name = "dsa-spec")]
#[command(about = "Generate code skeletons from DSA specifications")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Generate code stubs
    Generate {
        /// Path to the spec YAML file
        spec: PathBuf,

        /// Target language: rust, python, csharp, typescript, go, or all
        #[arg(short, long, default_value = "rust")]
        lang: String,

        /// Output file or directory (stdout if omitted)
        #[arg(short, long)]
        output: Option<PathBuf>,

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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate {
            spec,
            lang,
            output,
            contracts,
        } => {
            let yaml = fs::read_to_string(&spec)?;
            let parsed = parser::parse(&yaml)?;

            let active_spec = if contracts {
                contracts::inject_assertions(&parsed)
            } else {
                parsed
            };

            let spec_path = &spec;
            let lang_lower = lang.to_lowercase();
            let backends: Vec<(&str, Box<dyn Backend>)> = match lang_lower.as_str() {
                "rust" => vec![(
                    "rust",
                    Box::new(rust_backend::RustBackend::new("templates")?),
                )],
                "python" => vec![(
                    "python",
                    Box::new(python_backend::PythonBackend::new("templates")?),
                )],
                "csharp" | "c#" => vec![(
                    "csharp",
                    Box::new(csharp_backend::CSharpBackend::new("templates")?),
                )],
                "typescript" | "ts" => vec![(
                    "typescript",
                    Box::new(typescript_backend::TypeScriptBackend::new("templates")?),
                )],
                "go" => vec![("go", Box::new(go_backend::GoBackend::new("templates")?))],
                "all" => {
                    vec![
                        (
                            "rust",
                            Box::new(rust_backend::RustBackend::new("templates")?)
                                as Box<dyn Backend>,
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
                    ]
                }
                _ => {
                    eprintln!(
                        "Unsupported language: {lang}. Use rust, python, csharp, typescript, go, or all."
                    );
                    std::process::exit(1);
                }
            };

            for (lang_name, backend) in backends {
                let code = backend.generate(&active_spec)?;
                match &output {
                    Some(path) => {
                        if lang_lower == "all" {
                            let ext = match lang_name {
                                "rust" => "rs",
                                "python" => "py",
                                "csharp" => "cs",
                                "typescript" => "ts",
                                "go" => "go",
                                _ => "txt",
                            };
                            let file_name = format!(
                                "{}.{}",
                                spec_path.file_stem().unwrap().to_string_lossy(),
                                ext
                            );
                            let out_path = path.join(file_name);
                            fs::create_dir_all(path)?;
                            fs::write(&out_path, code)?;
                        } else {
                            fs::write(path, code)?;
                        }
                    }
                    None => println!("{code}"),
                }
            }
        }
        Command::Validate { spec } => {
            let yaml = fs::read_to_string(&spec)?;
            let parsed = parser::parse(&yaml)?;
            match validator::validate(&parsed) {
                Ok(()) => println!("Spec is valid."),
                Err(errs) => {
                    eprintln!("Validation errors:");
                    for e in errs {
                        eprintln!("  - {e}");
                    }
                    std::process::exit(1);
                }
            }
        }
        Command::Analyze { dir, format } => {
            let specs = match complexity::load_specs_from_dir(&dir.to_string_lossy()) {
                Ok(s) => s,
                Err(errs) => {
                    for e in &errs {
                        eprintln!("  - {e}");
                    }
                    std::process::exit(1);
                }
            };
            match format.to_lowercase().as_str() {
                "json" => println!("{}", complexity::generate_json_report(&specs)),
                "chart" => println!("{}", complexity::generate_tradeoff_chart(&specs)),
                _ => {
                    // default "table" and "markdown": produce the comparison table
                    println!("{}", complexity::generate_report(&specs));
                }
            }
        }
        Command::Visualize { spec, format } => {
            let yaml = fs::read_to_string(&spec)?;
            let parsed = parser::parse(&yaml)?;
            let output = visualization::generate(&parsed, &format);
            println!("{output}");
        }
        Command::Verify {
            spec,
            lang,
            backend,
        } => {
            if backend != "runtime" {
                eprintln!(
                    "Unsupported verification backend: {backend}. Only 'runtime' is supported."
                );
                std::process::exit(1);
            }

            let yaml = fs::read_to_string(&spec)?;
            let parsed = parser::parse(&yaml)?;
            let spec = contracts::inject_assertions(&parsed);

            let lang_lower = lang.to_lowercase();
            let backends: Vec<(&str, Box<dyn Backend>)> = match lang_lower.as_str() {
                "rust" => vec![(
                    "rust",
                    Box::new(rust_backend::RustBackend::new("templates")?),
                )],
                "python" => vec![(
                    "python",
                    Box::new(python_backend::PythonBackend::new("templates")?),
                )],
                "csharp" | "c#" => vec![(
                    "csharp",
                    Box::new(csharp_backend::CSharpBackend::new("templates")?),
                )],
                "typescript" | "ts" => vec![(
                    "typescript",
                    Box::new(typescript_backend::TypeScriptBackend::new("templates")?),
                )],
                "go" => vec![("go", Box::new(go_backend::GoBackend::new("templates")?))],
                "all" => {
                    vec![
                        (
                            "rust",
                            Box::new(rust_backend::RustBackend::new("templates")?)
                                as Box<dyn Backend>,
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
                    ]
                }
                _ => {
                    eprintln!(
                        "Unsupported language: {lang}. Use rust, python, csharp, typescript, go, or all."
                    );
                    std::process::exit(1);
                }
            };

            for (lang_name, backend) in backends {
                let code = backend.generate(&spec)?;
                println!("--- {lang_name} ---\n{code}");
            }
        }
    }
    Ok(())
}
