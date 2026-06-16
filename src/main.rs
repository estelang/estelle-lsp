use dashmap::DashMap;
use ropey::Rope;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

mod analysis;
mod builtins;
mod handlers;
mod lexer;
mod parser;

use analysis::{DocStore, DocumentState, analyze};

#[derive(Debug)]
struct Backend {
    client: Client,
    docs: DocStore,
}

impl Backend {
    fn new(client: Client) -> Self {
        Backend {
            client,
            docs: DashMap::new(),
        }
    }

    async fn on_change(&self, uri: Uri, text: String, version: i32) {
        let rope = Rope::from_str(&text);
        let decls = analyze(&text);

        let diags = decls.diagnostics.clone();
        self.docs.insert(
            uri.clone(),
            DocumentState {
                rope,
                decls,
                version,
            },
        );
        self.client
            .publish_diagnostics(uri, diags, Some(version))
            .await;
    }
}

impl LanguageServer for Backend {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "estelle-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        ".".to_string(),
                        ":".to_string(),
                        "|".to_string(),
                    ]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "estelle-lsp started")
            .await;
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(
            params.text_document.uri,
            params.text_document.text,
            params.text_document.version,
        )
        .await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        let full_text =
            if params.content_changes.len() == 1 && params.content_changes[0].range.is_none() {
                params.content_changes.remove(0).text
            } else if let Some(mut doc) = self.docs.get_mut(&uri) {
                for change in params.content_changes {
                    if let Some(range) = change.range {
                        apply_incremental(&mut doc.rope, range, &change.text);
                    }
                }
                doc.rope.to_string()
            } else {
                return;
            };

        self.on_change(uri, full_text, version).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.docs.remove(&params.text_document.uri);
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        Ok(handlers::hover(&self.docs, params).await)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(handlers::completion(&self.docs, params).await)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(handlers::goto_definition(&self.docs, params).await)
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        Ok(handlers::document_symbol(&self.docs, params).await)
    }
}

fn apply_incremental(rope: &mut Rope, range: Range, new_text: &str) {
    let start_char = rope
        .try_line_to_char(range.start.line as usize)
        .unwrap_or(0)
        + range.start.character as usize;
    let end_char =
        rope.try_line_to_char(range.end.line as usize).unwrap_or(0) + range.end.character as usize;

    rope.remove(start_char..end_char);
    rope.insert(start_char, new_text);
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let (service, socket) = LspService::new(Backend::new);
    Server::new(stdin, stdout, socket).serve(service).await;
}

#[cfg(test)]
mod tests {
    use crate::analysis::analyze;

    #[test]
    fn analyzes_imports_and_functions_and_reports_structural_errors() {
        let src = r#"
import "Module:Foo" as bar
pub fnc main str {
  x = 1
  return
}
fnc helper(n num) num {
  return n + 1
}
bad top level token
"#;
        let res = analyze(src);
        assert!(res.imports.iter().any(|i| i.alias == "bar"));
        assert!(res.functions.iter().any(|f| f.name == "main" && f.is_pub));
        assert!(res.functions.iter().any(|f| f.name == "helper"));

        assert!(!res.diagnostics.is_empty());
        assert!(
            res.diagnostics
                .iter()
                .any(|d| d.message.contains("Unexpected token"))
        );
    }

    #[test]
    fn hover_completions_data_present() {
        assert!(crate::builtins::find_builtin("trim").is_some());
        assert!(crate::builtins::find_builtin("page").is_some());
        assert!(crate::builtins::KEYWORDS.iter().any(|k| *k == "fnc"));
    }
}
