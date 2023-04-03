use log::*;

// 实现的 lsp 功能
use egg_language_server::egg_support::optimize::egg_violence;
use egg_language_server::python::py_parser;

// 依赖
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use serde_json::Value;


// 设置 TODO
#[derive(Debug)]
struct Settings {
    maxNumberOfProblems: u32,
}

#[derive(Debug)]
struct Backend {
    client: Client,
}

// 这里应该是 自己需实现的 LSP 后端接口
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
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;
        self.get_client_settings().await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file opened!")
            .await;
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
            version: params.text_document.version,
        })
        .await
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        self.on_change(TextDocumentItem {
            uri: params.text_document.uri,
            text: std::mem::take(&mut params.content_changes[0].text),
            version: params.text_document.version,
        })
        .await
    }

    async fn did_save(&self, _: DidSaveTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file saved!")
            .await;
    }
    async fn did_close(&self, _: DidCloseTextDocumentParams) {
        self.client
            .log_message(MessageType::INFO, "file closed!")
            .await;
    }

    async fn did_change_configuration(&self, _: DidChangeConfigurationParams) {
        self.client
            .log_message(MessageType::INFO, "configuration changed!")
            .await;
    }

    async fn did_change_workspace_folders(&self, _: DidChangeWorkspaceFoldersParams) {
        self.client
            .log_message(MessageType::INFO, "workspace folders changed!")
            .await;
    }

    async fn did_change_watched_files(&self, _: DidChangeWatchedFilesParams) {
        self.client
            .log_message(MessageType::INFO, "watched files have changed!")
            .await;
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
        // 根据拓展名选择解析器 
        // TODO 临时的方式。 更好是侦测客户端语言配置
        let target_language = match params
            .uri
            .to_file_path()
            .unwrap()
            .extension()
            .unwrap()
            .to_str()
        {
            Some(ext) => match ext {
                "py" => "python",
                "lisp" | "scm" => "lisp",
                _ => {
                    self.client
                        .log_message(MessageType::ERROR, "文件拓展名错误")
                        .await;
                    return;
                }
            },
            None => {
                self.client
                    .log_message(MessageType::ERROR, "文件拓展名错误")
                    .await;
                return;
            }
        };

        let f_parser;
        /* if target_language == "lisp" {
            f_parser = |s| egg_violence(s);
        } else */ if target_language == "python" {
            f_parser = py_parser;
        } else {
            return self.client.log_message(MessageType::ERROR, format!("暂不支持{target_language}")).await;
        }

        // egg
        let (m, diagnostic_type) = match f_parser(&params.text) {
            Ok(s) => (format!("{}", s), DiagnosticSeverity::INFORMATION),
            Err(s) => (format!("{}", s), DiagnosticSeverity::ERROR),
        };

        debug!("Egg: {} => {}", params.text.trim(), m);
        if params.text.trim() != m {
            let start_position = Position::new(0, 0);
            let lines = params.text.lines();
            let end_position = match (lines.clone().count(), lines.last()) {
                (count, Some(last_line)) => {
                    Position::new(count as u32 - 1, last_line.len() as u32)
                }
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


    // TODO 获取客户端设置的函数
    async fn get_client_settings(&self) {
        let settings = self.client
            .configuration(vec![ConfigurationItem {
                scope_uri: None,
                section: Some("maxNumberOfProblems".to_string()),
            }])
            .await;
        self.client
            .log_message(MessageType::INFO, format!("获取到客户端设置{:?}",settings))
            .await;
    }

}

#[tokio::main]
async fn main() {
    env_logger::init();

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::build(|client| Backend { client })
        // .custom_method("custom/inlay_hint", Backend::inlay_hint)
        .finish();
    Server::new(stdin, stdout, socket).serve(service).await;
}



