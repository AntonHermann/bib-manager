//! Wegwerf-Language-Server für Schritt 0b. Jede Antwort ist mit „bib-spike" markiert,
//! damit in Zed sichtbar ist, von welchem Server sie stammt.

use std::collections::HashMap;

use lsp_coexist::{KeyHit, find_keys, key_at};
use tokio::sync::Mutex;
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::{Client, LanguageServer, LspService, Server};

struct Spike {
    client: Client,
    docs: Mutex<HashMap<String, String>>,
}

fn range(hit: &KeyHit) -> Range {
    Range::new(Position::new(hit.line, hit.start), Position::new(hit.line, hit.end))
}

impl Spike {
    async fn update(&self, uri: Uri, text: String) {
        let diagnostics = find_keys(&text)
            .iter()
            .map(|hit| Diagnostic {
                range: range(hit),
                severity: Some(DiagnosticSeverity::HINT),
                source: Some("bib-spike".into()),
                message: format!("bib-spike sieht @{}", hit.key),
                ..Default::default()
            })
            .collect();
        self.docs.lock().await.insert(uri.as_str().to_string(), text);
        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn hit(&self, uri: &Uri, position: Position) -> Option<(String, KeyHit)> {
        let docs = self.docs.lock().await;
        let text = docs.get(uri.as_str())?;
        key_at(text, position.line, position.character).map(|hit| (text.clone(), hit))
    }
}

impl LanguageServer for Spike {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo { name: "bib-spike".into(), version: None }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider: Some(OneOf::Left(true)),
                references_provider: Some(OneOf::Left(true)),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec!["@".into()]),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "bib-spike bereit").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.update(params.text_document.uri, params.text_document.text).await;
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.pop() {
            self.update(params.text_document.uri, change.text).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let p = params.text_document_position_params;
        Ok(self.hit(&p.text_document.uri, p.position).await.map(|(_, hit)| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**bib-spike** Hover für `{}`", hit.key),
            }),
            range: Some(range(&hit)),
        }))
    }

    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<Option<GotoDefinitionResponse>> {
        let p = params.text_document_position_params;
        let uri = p.text_document.uri.clone();
        Ok(self.hit(&p.text_document.uri, p.position).await.map(|_| {
            GotoDefinitionResponse::Scalar(Location::new(uri, Range::new(Position::new(0, 0), Position::new(0, 0))))
        }))
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let p = params.text_document_position;
        let uri = p.text_document.uri.clone();
        Ok(self.hit(&p.text_document.uri, p.position).await.map(|(text, hit)| {
            find_keys(&text)
                .iter()
                .filter(|other| other.key == hit.key)
                .map(|other| Location::new(uri.clone(), range(other)))
                .collect()
        }))
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(self.hit(&params.text_document.uri, params.range.start).await.map(|(_, hit)| {
            vec![CodeActionOrCommand::CodeAction(CodeAction {
                title: format!("bib-spike: Aktion für {}", hit.key),
                kind: Some(CodeActionKind::QUICKFIX),
                ..Default::default()
            })]
        }))
    }

    async fn completion(&self, _: CompletionParams) -> Result<Option<CompletionResponse>> {
        Ok(Some(CompletionResponse::Array(vec![CompletionItem::new_simple(
            "bibspike2026".into(),
            "bib-spike Vervollständigung".into(),
        )])))
    }
}

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| Spike { client, docs: Mutex::new(HashMap::new()) });
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket).serve(service).await;
}
