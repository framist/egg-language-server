/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */
import {
	createConnection,
	TextDocuments,
	Diagnostic,
	DiagnosticSeverity,
	ProposedFeatures,
	InitializeParams,
	DidChangeConfigurationNotification,
	CompletionItem,
	CompletionItemKind,
	TextDocumentPositionParams,
	TextDocumentSyncKind,
	InitializeResult
} from 'vscode-languageserver/node';

import {
	TextDocument
} from 'vscode-languageserver-textdocument';

import { exec, execSync } from 'child_process';

// Create a connection for the server, using Node's IPC as a transport.
// Also include all preview / proposed LSP features.
// 创建一个服务器连接。使用 Node 的 IPC 作为传输方式。
// 也包含所有的预览、建议等LSP特性
const connection = createConnection(ProposedFeatures.all);

// 创建一个简单的文本文档管理器
const documents: TextDocuments<TextDocument> = new TextDocuments(TextDocument);

let hasConfigurationCapability = false;
let hasWorkspaceFolderCapability = false;
let hasDiagnosticRelatedInformationCapability = false;

connection.onInitialize((params: InitializeParams) => {
	const capabilities = params.capabilities;

	// Does the client support the `workspace/configuration` request?
	// If not, we fall back using global settings.
    // 客户端是否支持`workspace/configuration`请求?
    // 如果不是的话，降级到使用全局设置
	hasConfigurationCapability = !!(
		capabilities.workspace && !!capabilities.workspace.configuration
	);
	hasWorkspaceFolderCapability = !!(
		capabilities.workspace && !!capabilities.workspace.workspaceFolders
	);
	hasDiagnosticRelatedInformationCapability = !!(
		capabilities.textDocument &&
		capabilities.textDocument.publishDiagnostics &&
		capabilities.textDocument.publishDiagnostics.relatedInformation
	);

	const result: InitializeResult = {
		capabilities: {
			textDocumentSync: TextDocumentSyncKind.Incremental,
			// 告诉客户端此服务器支持代码完成。
			completionProvider: {
				resolveProvider: true
			}
		}
	};
	if (hasWorkspaceFolderCapability) {
		result.capabilities.workspace = {
			workspaceFolders: {
				supported: true
			}
		};
	}
	return result;
});

connection.onInitialized(() => {
	if (hasConfigurationCapability) {
		// 注册所有配置更改。
		connection.client.register(DidChangeConfigurationNotification.type, undefined);
	}
	if (hasWorkspaceFolderCapability) {
		connection.workspace.onDidChangeWorkspaceFolders(_event => {
			connection.console.log('Workspace folder change event received.');
		});
	}
});

// 示例设置
interface ExampleSettings {
	maxNumberOfProblems: number;
}


// 当客户端不支持`workspace/configuration`请求时，使用global settings
// 请注意，将此服务器与本示例中提供的客户端一起使用时，情况并非如此
// 但可能发生在其他客户端。
const defaultSettings: ExampleSettings = { maxNumberOfProblems: 1000 };
let globalSettings: ExampleSettings = defaultSettings;

// 对所有打开的文档配置进行缓存
const documentSettings: Map<string, Thenable<ExampleSettings>> = new Map();

// 添加一个通知处理函数监听配置文件变动。
connection.onDidChangeConfiguration(change => {
	if (hasConfigurationCapability) {
		// 重置所有已缓存的文档配置
		documentSettings.clear();
	} else {
		globalSettings = <ExampleSettings>(
			(change.settings.lispEgglanguageServer || defaultSettings)
		);
	}

	// 重新验证所有打开的文本文档
	documents.all().forEach(validateTextDocument);
});

// 在服务器中写一段读取客户端配置的代码：
function getDocumentSettings(resource: string): Thenable<ExampleSettings> {
	if (!hasConfigurationCapability) {
		return Promise.resolve(globalSettings);
	}
	let result = documentSettings.get(resource);
	if (!result) {
		result = connection.workspace.getConfiguration({
			scopeUri: resource,
			section: 'lispEgglanguageServer'
		});
		documentSettings.set(resource, result);
	}
	return result;
}

// 只对打开的文档保留设置
documents.onDidClose(e => {
	documentSettings.delete(e.document.uri);
});

// 文档的文本内容发生了改变。
// 这个事件在文档第一次打开或者内容变动时才会触发。
documents.onDidChangeContent(change => {
	validateTextDocument(change.document);
});

async function validateTextDocument(textDocument: TextDocument): Promise<void> {
	// In this simple example we get the settings for every validate run.
	// 在这个简单的示例中，每次校验运行时我们都获取一次配置
	const settings = await getDocumentSettings(textDocument.uri);

	const text = textDocument.getText();
	const pattern = /\(.*\)/g; // 获取固定 pattern
	let m: RegExpExecArray | null;

	let problems = 0;
	const diagnostics: Diagnostic[] = [];
	
	while ((m = pattern.exec(text)) && problems < settings.maxNumberOfProblems) {
		let egg_message = '';
		try {
			egg_message = execSync(`./egg_test "${m[0]}"`).toString().trim();
		} catch (error) {
			console.error(`执行出错：${error}`);
		}

		if (`(${egg_message})` === m[0] || `${egg_message}` === m[0]) {
			continue;
		}
		
		problems++;
		const diagnostic: Diagnostic = {
			severity: DiagnosticSeverity.Warning,
			range: {
				start: textDocument.positionAt(m.index),
				end: textDocument.positionAt(m.index + m[0].length)
			},
			message: `${m[0]} => ${egg_message}`,
			source: 'egg 测试'
		};
		if (hasDiagnosticRelatedInformationCapability) {
			diagnostic.relatedInformation = [
				{
					location: {
						uri: textDocument.uri,
						range: Object.assign({}, diagnostic.range)
					},
					message: '化简表达式很重要'
				},
				{
					location: {
						uri: textDocument.uri,
						range: Object.assign({}, diagnostic.range)
					},
					message: '尤其对于脚本语言'
				}
			];
		}
		diagnostics.push(diagnostic);
	}

	// Send the computed diagnostics to VSCode.
	// 将诊断信息发送给 VS Code
	connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}

connection.onDidChangeWatchedFiles(_change => {
	// Monitored files have change in VSCode
	// 监测VS Code中的文件变动
	connection.console.log('We received an file change event');
});

// 这个处理函数提供了初始补全项列表
connection.onCompletion(
	(_textDocumentPosition: TextDocumentPositionParams): CompletionItem[] => {
		// 传入的变量包含了文本请求代码补全的位置。
		// 在这个示例中我们忽略了这个信息，总是提供相同的补全选项。
		return [
			{
				label: '(+ 0 (* 1 foo))',
				kind: CompletionItemKind.Text,
				data: 1
			},
			{
				label: '(+ 0 bar)',
				kind: CompletionItemKind.Text,
				data: 2
			}
		];
		// data字段用于鉴别处理函数中传入的补全项。
		// 这个属性对协议来说是透明的，因为底层协议信息传输是基于JSON的，
		// 因此data字段只能保留从JSON序列化而来的数据。
	}
);

// 这个函数为补全列表的选中项提供了更多信息
connection.onCompletionResolve(
	(item: CompletionItem): CompletionItem => {
		if (item.data === 1) {
			item.detail = 'TypeScript details';
			item.documentation = 'TypeScript documentation';
		} else if (item.data === 2) {
			item.detail = 'JavaScript details';
			item.documentation = 'JavaScript documentation';
		}
		return item;
	}
);

// 让文档管理器监听文档的打开，变动和关闭事件。
documents.listen(connection);

// 连接后启动监听
connection.listen();
