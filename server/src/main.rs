use log::*;

// 实现的 lsp 功能
use egg_language_server::*;

// 依赖
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use serde_json::Value;
use std::sync::RwLock; // TODO 是否会出现死锁？

#[allow(dead_code)]
struct Settings {
    // 语言客户端配置
    max_number_of_problems: usize,
    if_explanations: bool,
    explanation_with_let: bool,
    if_egg_ir: bool,
    out_language: String,
    // 编辑器配置
    target_language: String,
    // 内部
    f_parser: fn(&str) -> Vec<EggDiagnostic>,
    f_reparser: fn(&String) -> std::result::Result<String, String>,
}
impl Settings {
    fn new() -> Self {
        Settings {
            max_number_of_problems: 100,
            if_explanations: false,
            explanation_with_let: false,
            if_egg_ir: false,
            out_language: String::from(""),
            target_language: String::from("lisp"),
            // 内部
            f_parser: lisp_parser,
            f_reparser: lisp_reparser,
        }
    }
}

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
        debug!("DEBUG: initialized!");
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
        self.log_info(format!("command executed! {:?}", p.command))
            .await;

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
        self.log_info(format!("file changed! {:?}", params.uri))
            .await;

        let diagnostics = (self.settings.read().unwrap().f_parser)(&params.text)
            .into_iter()
            .map(|d| {
                Diagnostic::new(
                    d.span,
                    Some(d.label),
                    None,
                    Some("egg-language-server".to_string()), // 可选字段，用于指定 linter 的名称或标识符等
                    match d.sexpr {
                        Some(s) => format!(
                            "{} => {}\npseudo code ({}-like):\n{}",
                            d.reason,
                            s,
                            self.settings.read().unwrap().out_language,
                            (self.settings.read().unwrap().f_reparser)(&s).unwrap()
                        ),
                        None => d.reason,
                    },
                    None,
                    None,
                )
            })
            .take(self.settings.read().unwrap().max_number_of_problems)
            .collect::<Vec<_>>();

        // 发送诊断信息
        self.client
            .publish_diagnostics(params.uri.clone(), diagnostics, Some(params.version))
            .await;

        debug!("诊断已发送 version={}", params.version);
    }

    async fn get_ext(&self, uri: &Url) -> Option<&str> {
        let target_language = match uri.to_file_path().ok()?.extension()?.to_str()? {
            "py" => "python",
            "lisp" | "scm" => "lisp",
            "js" => "javascript",
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

        self.log_info(format!("获取到客户端设置{:?}", settings))
            .await;

        match settings {
            Ok(settings) => {
                let mut s = self.settings.write().unwrap();
                s.max_number_of_problems =
                    settings[0]["maxNumberOfProblems"].as_u64().unwrap_or(100) as usize;
                s.if_explanations = settings[0]["ifExplanations"].as_bool().unwrap_or(true);
                s.if_egg_ir = settings[0]["ifEggIR"].as_bool().unwrap_or(true);
                s.explanation_with_let =
                    settings[0]["ExplanationWithLet"].as_bool().unwrap_or(true);
                s.out_language = settings[0]["outLanguage"]
                    .as_str()
                    .unwrap_or("lisp")
                    .to_string();
            }
            Err(_) => {
                self.log_error("获取客户端设置失败".to_string()).await;
            }
        };

        // TODO 临时实现的目标语言检测
        let target_language = match self.get_ext(&uri).await {
            Some(value) => value,
            None => {
                self.log_error(format!("不支持的文件类型: {}", uri)).await;
                return;
            }
        };
        self.log_info(format!("目标语言: {}", target_language))
            .await;
        self.settings.write().unwrap().target_language = target_language.to_string();

        // 根据设置配置内部设置
        let f_parser: fn(&str) -> Vec<EggDiagnostic>;
        let mut f_reparser: fn(&String) -> std::result::Result<String, String>;

        match target_language {
            "lisp" => {
                f_parser = lisp_parser;
                f_reparser = lisp_reparser;
            }
            "python" => {
                f_parser = py_parser;
                f_reparser = py_reparser;
            }
            "javascript" => {
                f_parser = js_parser;
                f_reparser = js_reparser;
            }
            _ => {
                return self
                    .log_warn(format!("不支持的语言: {}", target_language))
                    .await;
            }
        };
        // 根据配置选择输出方式
        let out_language = self.settings.read().unwrap().out_language.clone();

        f_reparser = match out_language.as_str() {
            "lisp" => lisp_reparser,
            "python" => py_reparser,
            "javascript" => js_reparser,
            "debug" => debug_reparser,
            "same as source" => f_reparser,
            _ => {
                self
                    .log_warn(format!("不支持的输出语言: {}", out_language))
                    .await;
                debug_reparser
            }
        };
        // 更新配置 f_parser
        self.settings.write().unwrap().f_parser = f_parser;
        self.settings.write().unwrap().f_reparser = f_reparser;
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
    // 自定义日志格式
    // std::env::set_var("RUST_LOG", "egg_language_server=debug,egg=off"); // 在客户端已设置环境变量
    use std::io::Write;
    env_logger::builder()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} - {}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
        .init();
    // env_logger::init();  // 使用默认配置而非自定义

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
