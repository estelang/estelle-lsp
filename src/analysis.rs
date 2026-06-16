use crate::lexer;
use crate::parser::{DeclExtractor, FunctionDecl, Import};
use dashmap::DashMap;
use ropey::Rope;
use tower_lsp_server::ls_types::{Diagnostic, DiagnosticSeverity, Uri};

#[derive(Debug)]
pub struct DocumentState {
    pub rope: Rope,
    pub decls: DeclResult,
    #[allow(dead_code)] // version kept for future use
    pub version: i32,
}

#[derive(Debug)]
pub struct DeclResult {
    pub imports: Vec<Import>,
    pub functions: Vec<FunctionDecl>,
    pub diagnostics: Vec<Diagnostic>,
}

pub type DocStore = DashMap<Uri, DocumentState>;

pub fn analyze(text: &str) -> DeclResult {
    let tokens = lexer::lex(text);
    let mut extractor = DeclExtractor::new(&tokens);
    extractor.run();

    let diagnostics: Vec<Diagnostic> = extractor
        .errors
        .iter()
        .map(|e| Diagnostic {
            range: e.range,
            severity: Some(DiagnosticSeverity::ERROR),
            source: Some("estelle-lsp".to_string()),
            message: e.message.clone(),
            ..Default::default()
        })
        .collect();

    DeclResult {
        imports: extractor.imports,
        functions: extractor.functions,
        diagnostics,
    }
}
