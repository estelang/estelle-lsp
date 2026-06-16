use crate::analysis::DocStore;
use crate::builtins;
use crate::parser::FunctionDecl;
use ropey::Rope;
use tower_lsp_server::ls_types::*;

fn extract_word_at(rope: &Rope, pos: Position) -> Option<String> {
    let line_idx = pos.line as usize;
    if line_idx >= rope.len_lines() {
        return None;
    }
    let line = rope.line(line_idx);
    let line_str = line.to_string();
    let char_idx = pos.character as usize;

    if char_idx > line_str.len() {
        return None;
    }

    let mut start = char_idx;
    while start > 0 {
        let prev = line_str[..start].chars().next_back()?;
        if prev.is_ascii_alphanumeric() || prev == '_' {
            start -= prev.len_utf8();
        } else {
            break;
        }
    }

    let mut end = char_idx;
    while end < line_str.len() {
        let ch = line_str[end..].chars().next()?;
        if ch.is_ascii_alphanumeric() || ch == '_' {
            end += ch.len_utf8();
        } else {
            break;
        }
    }

    if start == end {
        return None;
    }
    Some(line_str[start..end].to_string())
}

pub async fn hover(docs: &DocStore, params: HoverParams) -> Option<Hover> {
    let uri = &params.text_document_position_params.text_document.uri;
    let pos = params.text_document_position_params.position;

    let doc = docs.get(uri)?;
    let word = extract_word_at(&doc.rope, pos)?;

    if let Some(b) = builtins::find_builtin(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```estelle\n{}\n```\n\n{}", b.signature, b.doc),
            }),
            range: None,
        });
    }

    if let Some(desc) = builtins::keyword_doc(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::PlainText,
                value: desc.to_string(),
            }),
            range: None,
        });
    }

    if let Some(fnc) = doc.decls.functions.iter().find(|f| f.name == word) {
        let sig = format_signature(fnc);
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("```estelle\n{}\n```", sig),
            }),
            range: None,
        });
    }

    None
}

fn format_signature(f: &FunctionDecl) -> String {
    let params: Vec<String> = f
        .params
        .iter()
        .map(|p| {
            let tn = type_name(p.typ);
            format!("{} {}{}", p.name, tn, if p.nullable { "?" } else { "" })
        })
        .collect();
    let ret = f
        .return_type
        .map(|t| format!(" {}", type_name(t)))
        .unwrap_or_default();
    format!(
        "{}fnc {}({}){}",
        if f.is_pub { "pub " } else { "" },
        f.name,
        params.join(", "),
        ret
    )
}

fn type_name(t: crate::parser::EstelleType) -> &'static str {
    use crate::parser::EstelleType::*;
    match t {
        Str => "str",
        Num => "num",
        Bool => "bool",
        List => "list",
        Map => "map",
    }
}

pub async fn goto_definition(
    docs: &DocStore,
    params: GotoDefinitionParams,
) -> Option<GotoDefinitionResponse> {
    let uri = &params.text_document_position_params.text_document.uri;
    let pos = params.text_document_position_params.position;
    let doc = docs.get(uri)?;
    let word = extract_word_at(&doc.rope, pos)?;

    if let Some(fnc) = doc.decls.functions.iter().find(|f| f.name == word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: fnc.name_span,
        }));
    }

    if let Some(imp) = doc.decls.imports.iter().find(|i| i.alias == word) {
        return Some(GotoDefinitionResponse::Scalar(Location {
            uri: uri.clone(),
            range: imp.name_span,
        }));
    }

    None
}

pub async fn document_symbol(
    docs: &DocStore,
    params: DocumentSymbolParams,
) -> Option<DocumentSymbolResponse> {
    let doc = docs.get(&params.text_document.uri)?;

    let symbols: Vec<DocumentSymbol> = doc
        .decls
        .functions
        .iter()
        .map(|f| {
            #[allow(deprecated)]
            DocumentSymbol {
                name: f.name.clone(),
                detail: Some(format_signature(f)),
                kind: SymbolKind::FUNCTION,
                tags: None,
                deprecated: None, // required by ls-types despite deprecation
                range: f.body_span,
                selection_range: f.name_span,
                children: None,
            }
        })
        .collect();

    Some(DocumentSymbolResponse::Nested(symbols))
}

pub async fn completion(docs: &DocStore, params: CompletionParams) -> Option<CompletionResponse> {
    let uri = &params.text_document_position.text_document.uri;

    let mut items: Vec<CompletionItem> = Vec::new();

    for kw in builtins::KEYWORDS {
        items.push(CompletionItem {
            label: (*kw).to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        });
    }

    for b in builtins::BUILTINS {
        items.push(CompletionItem {
            label: b.name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(b.signature.to_string()),
            documentation: Some(Documentation::String(b.doc.to_string())),
            insert_text: Some(format!("{}(", b.name)),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            ..Default::default()
        });
    }

    if let Some(doc) = docs.get(uri) {
        for f in &doc.decls.functions {
            items.push(CompletionItem {
                label: f.name.clone(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(format_signature(f)),
                ..Default::default()
            });
        }

        for imp in &doc.decls.imports {
            items.push(CompletionItem {
                label: imp.alias.clone(),
                kind: Some(CompletionItemKind::MODULE),
                detail: Some(format!("import \"{}\"", imp.path)),
                ..Default::default()
            });
        }
    }

    Some(CompletionResponse::Array(items))
}
