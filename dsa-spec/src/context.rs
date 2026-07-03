//! Shared context structs and helpers for language backends.
//!
//! All backends except Go use borrowed `&str` / `&[String]` context
//! structs.  Go is the outlier (owned `String` / `Vec<String>`) and
//! keeps its own context definitions.
//!
//! The helpers in this module eliminate the ~12‑line metadata +
//! contracts insertion block and the ~15‑line test‑case building
//! block that were copy‑pasted into all three context‑building
//! functions of every borrowed‑type backend.

use crate::ast::Spec;
use serde::Serialize;
use tera::Context;

/// Template context carrying spec name and complexity (borrowed form).
#[derive(Serialize)]
pub struct MetadataContext<'a> {
    pub name: &'a str,
    pub complexity: ComplexityContext<'a>,
}

/// Template context for Big-O complexity annotations (borrowed form).
#[derive(Serialize)]
pub struct ComplexityContext<'a> {
    pub time: Option<&'a str>,
    pub space: Option<&'a str>,
}

/// Template context for invariants (borrowed form).
#[derive(Serialize)]
pub struct ContractsContext<'a> {
    pub invariants: &'a [String],
}

/// Template context for test case collections (borrowed form).
#[derive(Serialize)]
pub struct VerificationContext<'a> {
    pub test_cases: Vec<TestContext<'a>>,
}

/// Template context for a single test scenario (borrowed form).
#[derive(Serialize)]
pub struct TestContext<'a> {
    pub name: &'a str,
    pub setup: Option<&'a str>,
    pub actions: &'a [String],
    pub assertions: &'a [String],
}

/// Insert metadata, complexity and contracts into a Tera context.
///
/// Suitable for Rust, Python, C# and TypeScript (all use borrowed
/// `&str` / `&[String]`).  Go handles these fields on its own
/// because its context structs own `String` values.
pub fn add_metadata_and_contracts(ctx: &mut Context, spec: &Spec) {
    ctx.insert(
        "metadata",
        &MetadataContext {
            name: &spec.metadata.name,
            complexity: ComplexityContext {
                time: spec.metadata.complexity.time.as_deref(),
                space: spec.metadata.complexity.space.as_deref(),
            },
        },
    );
    ctx.insert(
        "contracts",
        &ContractsContext {
            invariants: &spec.contracts.invariants,
        },
    );
}

/// Insert test cases *without* assertion translation (Rust pattern).
pub fn add_test_cases_raw(ctx: &mut Context, spec: &Spec) {
    let tests: Vec<TestContext> = spec
        .verification
        .test_cases
        .iter()
        .map(|t| TestContext {
            name: &t.name,
            setup: t.setup.as_deref(),
            actions: &t.actions,
            assertions: &t.assertions,
        })
        .collect();
    ctx.insert("verification", &VerificationContext { test_cases: tests });
}

/// Insert test cases with assertion translation (Python/C#/TS pattern).
///
/// The `translator` function converts each raw assertion string into
/// the target language's assertion syntax.
pub fn add_test_cases_translated(ctx: &mut Context, spec: &Spec, translator: fn(&str) -> String) {
    let translated_assertions: Vec<Vec<String>> = spec
        .verification
        .test_cases
        .iter()
        .map(|t| t.assertions.iter().map(|a| translator(a)).collect())
        .collect();

    let tests: Vec<TestContext> = spec
        .verification
        .test_cases
        .iter()
        .enumerate()
        .map(|(i, t)| TestContext {
            name: &t.name,
            setup: t.setup.as_deref(),
            actions: &t.actions,
            assertions: &translated_assertions[i],
        })
        .collect();
    ctx.insert("verification", &VerificationContext { test_cases: tests });
}
