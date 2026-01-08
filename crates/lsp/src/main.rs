use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use coding_guide_helper_core::{Lexer, Parser, diagnose, DiagnosticSeverity, LoadedProjectConfig};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
struct Backend {
    client: Client,
    config: Arc<RwLock<LoadedProjectConfig>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // ワークスペースルートから設定を読み込む
        if let Some(workspace_folders) = params.workspace_folders {
            if let Some(folder) = workspace_folders.first() {
                if let Ok(path) = folder.uri.to_file_path() {
                    let loaded_config = LoadedProjectConfig::find_and_load_with_root(&path);
                    let mut config = self.config.write().await;
                    *config = loaded_config;
                    
                    self.client
                        .log_message(
                            MessageType::INFO,
                            format!(
                                "Loaded config from workspace: {}",
                                path.display()
                            ),
                        )
                        .await;
                }
            }
        }
        
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "Coding Guide Helper".to_string(),
                version: Some("0.1.0".to_string()),
            }),
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Coding Guide Helper LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.on_change(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.first() {
            self.on_change(params.text_document.uri, change.text.clone())
                .await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        if let Some(text) = params.text {
            self.on_change(params.text_document.uri, text).await;
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        
        // URIからファイルパスを取得してファイルを読み込む
        if let Ok(path) = uri.to_file_path() {
            if let Ok(source) = std::fs::read_to_string(&path) {
                let config = self.config.read().await;
                let mut parser = Parser::new_with_config(
                    Lexer::new(&source),
                    config.to_preprocessor_config(),
                );
                if let Some(parent) = path.parent() {
                    parser.set_current_file_dir(parent);
                }
                let tu = parser.parse();
                
                let formatter = coding_guide_helper_core::Formatter::new();
                let formatted = formatter.format_tu(&tu);
                
                // 全体を置換するTextEditを返す
                let lines: Vec<&str> = source.lines().collect();
                let end_line = lines.len().saturating_sub(1);
                let end_char = lines.last().map(|l| l.len()).unwrap_or(0);
                
                return Ok(Some(vec![TextEdit {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: end_line as u32,
                            character: end_char as u32,
                        },
                    },
                    new_text: formatted,
                }]));
            }
        }
        
        Ok(None)
    }
}

impl Backend {
    async fn on_change(&self, uri: Url, text: String) {
        // パースして診断を実行
        let config = self.config.read().await;

        let mut parser = Parser::new_with_config(Lexer::new(&text), config.to_preprocessor_config());
        let source_path = uri.to_file_path().ok();
        if let Some(path) = &source_path {
            if let Some(parent) = path.parent() {
                parser.set_current_file_dir(parent);
            }
        }
        let tu = parser.parse();
        
        // プロジェクト設定から診断設定を取得
        let diag_config = config.to_diagnostic_config_with_path(source_path.as_ref());
        
        // デバッグ用ログ
        self.client
            .log_message(
                MessageType::INFO,
                format!(
                    "Diagnosing file: {:?}, Project root: {:?}",
                    source_path,
                    diag_config.project_root
                ),
            )
            .await;
        
        let diagnostics = diagnose(&tu, &diag_config);
        
        // LSP Diagnosticに変換
        let lsp_diagnostics: Vec<Diagnostic> = diagnostics
            .iter()
            .map(|diag| {
                let severity = match diag.severity {
                    DiagnosticSeverity::Error => Some(tower_lsp::lsp_types::DiagnosticSeverity::ERROR),
                    DiagnosticSeverity::Warning => Some(tower_lsp::lsp_types::DiagnosticSeverity::WARNING),
                    DiagnosticSeverity::Information => Some(tower_lsp::lsp_types::DiagnosticSeverity::INFORMATION),
                    DiagnosticSeverity::Hint => Some(tower_lsp::lsp_types::DiagnosticSeverity::HINT),
                };
                
                Diagnostic {
                    range: Range {
                        start: Position {
                            line: diag.span.start_line as u32,
                            character: diag.span.start_column as u32,
                        },
                        end: Position {
                            line: diag.span.end_line as u32,
                            character: diag.span.end_column as u32,
                        },
                    },
                    severity,
                    code: Some(NumberOrString::String(diag.code.to_string())),
                    source: Some("coding-guide-helper".to_string()),
                    message: diag.message.clone(),
                    ..Default::default()
                }
            })
            .collect();
        
        self.client
            .publish_diagnostics(uri, lsp_diagnostics, None)
            .await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { 
        client,
        config: Arc::new(RwLock::new(LoadedProjectConfig::find_and_load_with_root(
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from(".")),
        ))),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}
