/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as path from 'path';
import { workspace, ExtensionContext } from 'vscode';

import {
	LanguageClient,
	LanguageClientOptions,
	ServerOptions,
	TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;


// 启动函数
export function activate(context: ExtensionContext) {
	// 服务器由 node 实现
	const serverModule = context.asAbsolutePath(
		path.join('server', 'out', 'server.js')  // **服务器路径**
	);

    // 如果插件运行在调试模式那么就会使用debug server options
    // 不然就使用 run options
	// IPC（Inter-Process Communication，进程间通信）
	const serverOptions: ServerOptions = {
		run: { module: serverModule, transport: TransportKind.ipc },
		debug: {
			module: serverModule,
			transport: TransportKind.ipc,
		}
	};

	// 控制语言客户端的选项
	const clientOptions: LanguageClientOptions = {
		// 注册 lisp 语言 服务器
		documentSelector: [{ scheme: 'file', language: 'lisp' }],
		synchronize: {
			// 当文件变动为'.clientrc'中那样时，通知服务器
			fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
		}
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
