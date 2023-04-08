use log::*;

// 实现的 lsp 功能
use egg_language_server::egg_support::optimize::egg_violence;
use egg_language_server::python::py_parser;

// 依赖
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use serde_json::Value;
use std::sync::RwLock;

#[allow(dead_code)]
#[derive(Debug)]
struct Settings {
    max_number_of_problems: u32,
    if_explanations: bool,
    explanation_with_let: bool,
    explanation_with_high_level_pl: String,
    if_egg_ir: bool,
    out_language: String,
}
impl Settings {
    fn new() -> Self {
        Settings {
            max_number_of_problems: 100,
            if_explanations: false,
            explanation_with_let: false,
            explanation_with_high_level_pl: String::from(""),
            if_egg_ir: false,
            out_language: String::from(""),
        }
    }
}
    

#[derive(Debug)]
struct Backend {
    client: Client,
    settings: RwLock<Settings>,
}

// 需实现的 LSP 后端接口
#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                execute_command_provider: Some(ExecuteCommandOptions {
                    commands: vec!["EgglanguageServer.helloWorld".to_string()],
                    work_done_progress_options: Default::default(),
                }),

                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    file_operations: None,
                }),
                ..ServerCapabilities::default()
            },
        })
    }
    async fn initialized(&self, _: InitializedParams) {
        self.log_info(format!("initialized!")).await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.log_info(format!("file opened! {:?}", params.text_document.uri))
            .await;
        // 获取配置
        self.get_client_settings(&params.text_document.uri).await;

        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        // 获取配置
        self.get_client_settings(&params.text_document.uri).await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
        })
        .await
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.log_info("file saved!").await;
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.log_info("file closed!").await;
    }

    async fn did_change_configuration(&self, p: DidChangeConfigurationParams) {
        // TODO 没获取到配置
        self.log_info(format!("configuration changed! {:?}", p.settings))
            .await;
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.log_info("workspace folders changed!").await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.log_info("watched files have changed!").await;
    }

    async fn execute_command(&self, p: ExecuteCommandParams) -> Result<Option<Value>> {
        self.log_info(format!("command executed! {:?}", p.command)).await;

        // match self.client.apply_edit(WorkspaceEdit::default()).await {
        //     Ok(res) if res.applied => self.client.log_message(MessageType::INFO, "applied").await,
        //     Ok(_) => self.client.log_message(MessageType::INFO, "rejected").await,
        //     Err(err) => self.client.log_message(MessageType::ERROR, err).await,
        // }

        Ok(None)
    }
}

struct TextDocumentItem {
    uri: Url,
    text: String,
    version: i32,
}

/// 后端的一些方法实现
/// 目前只实现了
/// - `on_change`
///   - 被 `did_change` 和 `did_open` 接口引用
/// TODO 增量更新方式
impl Backend {
    async fn on_change(&self, params: TextDocumentItem) {
        let target_language = match self.get_ext(&params).await {
            Some(value) => value,
            None => {
                self.log_error(format!("不支持的文件类型: {}", params.uri))
                    .await;
                return;
            }
        };
        self.client
            .log_message(
                MessageType::INFO,
                format!("target_language: {}", target_language),
            )
            .await;

        let f_parser: fn(&str) -> std::result::Result<String, String>;
        if target_language == "lisp" {
            f_parser = egg_violence;
        } else if target_language == "python" {
            f_parser = py_parser;
        } else {
            return self
                .log_warn(format!("不支持的语言: {}", target_language))
                .await;
        }

        let (m, diagnostic_type) = match f_parser(&params.text) {
            Ok(s) => (format!("{}", s), DiagnosticSeverity::INFORMATION),
            Err(s) => (format!("{}", s), DiagnosticSeverity::ERROR),
        };

        debug!("Egg: {} => {}", params.text.trim(), m);
        if params.text.trim() != m {
            let start_position = Position::new(0, 0);
            let lines = params.text.lines();
            let end_position = match (lines.clone().count(), lines.last()) {
                (count, Some(last_line)) => Position::new(count as u32 - 1, last_line.len() as u32),
                _ => Position::new(0, 0),
            };

            let diagnostic = Diagnostic::new(
                Range::new(start_position, end_position), // 设置诊断范围
                Some(diagnostic_type),                    // 设置诊断级别为 "Information"
                None,
                Some("egg-support".to_string()), // 可选字段，用于指定 linter 的名称或标识符等
                format!("可以优化为 => {}", m),
                None,
                None,
            );
            let diagnostics = vec![diagnostic];

            // 发送诊断信息
            self.client
                .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
                .await;

            debug!("诊断已发送！{}", params.version);
        } else {
            // 否则，发送空诊断
            self.client
                .publish_diagnostics(params.uri.clone(), vec![], Some(params.version))
                .await;
        }
    }

    async fn get_ext(&self, params: &TextDocumentItem) -> Option<&str> {
        let target_language = match params.uri.to_file_path().ok()?.extension()?.to_str()? {
            "py" => "python",
            "lisp" | "scm" => "lisp",
            _ => {
                return None;
            }
        };
        Some(target_language)
    }

    // 获取客户端设置
    async fn get_client_settings(&self, uri: &Url) {
        let settings = self
            .client
            .configuration(vec![ConfigurationItem {
                scope_uri: Some(uri.clone()),
                section: Some("EgglanguageServer".to_string()),
            }])
            .await;
        let old_set = self.settings.read().unwrap().max_number_of_problems;
        self.log_info(format!("旧的客户端设置: {}", old_set)).await;
        self.log_info(format!("获取到客户端设置{:?}", settings))
            .await;
        // 例如
        // Ok([Object {"maxNumberOfProblems": Number(100), "trace": Object {"server": String("verbose")}}])
        match settings {
            Ok(settings) => {
                self.settings.write().unwrap().max_number_of_problems =
                    settings[0]["maxNumberOfProblems"].as_u64().unwrap_or(100) as u32;
            }
            Err(_) => {
                self.log_error("获取客户端设置失败".to_string()).await;
            }
        };
    }

    #[inline]
    async fn log_error<M: std::fmt::Display>(&self, message: M) {
        self.client.log_message(MessageType::ERROR, message).await;
    }
    #[inline]
    async fn log_info<M: std::fmt::Display>(&self, message: M) {
        self.client.log_message(MessageType::INFO, message).await;
    }
    #[inline]
    async fn log_warn<M: std::fmt::Display>(&self, message: M) {
        self.client.log_message(MessageType::WARNING, message).await;
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend {
        client,
        settings: RwLock::new(Settings::new()),
    })
    // .custom_method("custom/inlay_hint", Backend::inlay_hint)
    .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}
