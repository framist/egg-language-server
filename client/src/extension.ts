/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from 'path';

import {
	languages,
	workspace,
	EventEmitter,
	ExtensionContext,
	window,
	TextDocument,
	CancellationToken,
	Range,
	TextDocumentChangeEvent,
	ProviderResult,
	commands,
	WorkspaceEdit,
	TextEdit,
	Selection,
	Uri,
} from "vscode";

import {
	Disposable,
	Executable,
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;


// 启动函数
export function activate(context: ExtensionContext) {
	//* old
	// // 服务器由 node 实现
	// const serverModule = context.asAbsolutePath(
	// 	path.join('server', 'out', 'server.js')  // **服务器路径**
	// );

	// // 如果插件运行在调试模式那么就会使用debug server options
	// // 不然就使用 run options
	// // IPC（Inter-Process Communication，进程间通信）
	// const serverOptions: ServerOptions = {
	// 	run: { module: serverModule, transport: TransportKind.ipc },
	// 	debug: {
	// 		module: serverModule,
	// 		transport: TransportKind.ipc,
	// 	}
	// };
	//* old end


	const traceOutputChannel = window.createOutputChannel("Lisp-egg Language Server trace");
	const command = process.env.SERVER_PATH || "lisp-egg-language-server";
	const run: Executable = {
		command,
		options: {
			env: {
				...process.env,
				// eslint-disable-next-line @typescript-eslint/naming-convention
				RUST_LOG: "debug",
			},
		},
	};
	const serverOptions: ServerOptions = {
		run,
		debug: run,
	};


	// 控制语言客户端的选项
	const clientOptions: LanguageClientOptions = {
		// 注册 lisp 语言 服务器
		documentSelector: [{ scheme: 'file', language: 'lisp' }],
		synchronize: {
			// 当文件变动为'.clientrc'中那样时，通知服务器
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		},
		traceOutputChannel,
	};

	// 创建语言客户端并启动客户端。
	client = new LanguageClient(
		'lispEgglanguageServer', // 它是客户端的名称，与服务端配置文件中指定的名称相同。
		'Language Server Example', // 对客户端的描述, 将会在用户界面中显示。
		serverOptions,
		clientOptions
	);

	// 启动客户端。这也将启动服务器
	client.start();
}

// 消动函数
export function deactivate(): Thenable<void> | undefined {
	if (!client) {
		return undefined;
	}
	return client.stop();
}

// TODO 暂未实现的附加功能：行内 hints 
export function activateInlayHints(ctx: ExtensionContext) {
	const maybeUpdater = {
		hintsProvider: null as Disposable | null,
		updateHintsEventEmitter: new EventEmitter<void>(),

		async onConfigChange() {
			this.dispose();

			const event = this.updateHintsEventEmitter.event;
			// this.hintsProvider = languages.registerInlayHintsProvider(
			//   { scheme: "file", language: "nrs" },
			//   // new (class implements InlayHintsProvider {
			//   //   onDidChangeInlayHints = event;
			//   //   resolveInlayHint(hint: InlayHint, token: CancellationToken): ProviderResult<InlayHint> {
			//   //     const ret = {
			//   //       label: hint.label,
			//   //       ...hint,
			//   //     };
			//   //     return ret;
			//   //   }
			//   //   async provideInlayHints(
			//   //     document: TextDocument,
			//   //     range: Range,
			//   //     token: CancellationToken
			//   //   ): Promise<InlayHint[]> {
			//   //     const hints = (await client
			//   //       .sendRequest("custom/inlay_hint", { path: document.uri.toString() })
			//   //       .catch(err => null)) as [number, number, string][];
			//   //     if (hints == null) {
			//   //       return [];
			//   //     } else {
			//   //       return hints.map(item => {
			//   //         const [start, end, label] = item;
			//   //         let startPosition = document.positionAt(start);
			//   //         let endPosition = document.positionAt(end);
			//   //         return {
			//   //           position: endPosition,
			//   //           paddingLeft: true,
			//   //           label: [
			//   //             {
			//   //               value: `${label}`,
			//   //               // location: {
			//   //               //   uri: document.uri,
			//   //               //   range: new Range(1, 0, 1, 0)
			//   //               // }
			//   //               command: {
			//   //                 title: "hello world",
			//   //                 command: "helloworld.helloWorld",
			//   //                 arguments: [document.uri],
			//   //               },
			//   //             },
			//   //           ],
			//   //         };
			//   //       });
			//   //     }
			//   //   }
			//   // })()
			// );
		},

		onDidChangeTextDocument({ contentChanges, document }: TextDocumentChangeEvent) {
			// debugger
			// this.updateHintsEventEmitter.fire();
		},

		dispose() {
			this.hintsProvider?.dispose();
			this.hintsProvider = null;
			this.updateHintsEventEmitter.dispose();
		},
	};

	workspace.onDidChangeConfiguration(maybeUpdater.onConfigChange, maybeUpdater, ctx.subscriptions);
	workspace.onDidChangeTextDocument(maybeUpdater.onDidChangeTextDocument, maybeUpdater, ctx.subscriptions);

	maybeUpdater.onConfigChange().catch(console.error);
}


