/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */


import path = require('path');
import {
    workspace,
    ExtensionContext,
    window,
    commands
} from "vscode";

import {
    Executable,
    LanguageClient,
    LanguageClientOptions,
    ServerOptions
} from 'vscode-languageclient/node';

let client: LanguageClient;


// 启动函数
export function activate(context: ExtensionContext) {
    // * 启动语言服务器

    let ls_path = '';
    const platform = process.platform;
    if (platform === 'win32') {
        ls_path = path.join(context.extensionPath, 'target', 'x86_64-pc-windows-gnu', 'release', 'egg-language-server.exe');
    } else if (platform === 'darwin') {
        // ls_path = path.join(context.extensionPath, 'bin', 'macos', 'executable');
    } else if (platform === 'linux') {
        ls_path = path.join(context.extensionPath, 'target', 'x86_64-unknown-linux-gnu', 'release', 'egg-language-server');
    }

    if (!ls_path) {
        window.showErrorMessage('Egg Language Server : Unsupported platform. 😢');
        return;
    }
    
    // 创建一个输出通道，用于显示语言服务器的跟踪信息
    const traceOutputChannel = window.createOutputChannel("egg Language Server trace");
    const run: Executable = {
        command: process.env.SERVER_PATH || ls_path,
        options: {
            env: {
                ...process.env,                    // 继承当前进程环境变量，并添加或覆盖其中的环境变量
                RUST_LOG: "egg_language_server=info,egg=off",   // rust 日志级别；egg 似乎会匹配 egg*
                // RUST_BACKTRACE: 1                  // 开启 Rust panic 时的 backtrace 功能
            },
        },
    };
    const debug: Executable = {
        // 取得要运行的语言服务器的命令路径
        command: process.env.SERVER_PATH || ls_path,
        options: {
            env: {
                ...process.env,                    // 继承当前进程环境变量，并添加或覆盖其中的环境变量
                RUST_LOG: "egg_language_server=debug,egg=off",   // rust 日志级别；egg 似乎会匹配 egg*
                RUST_BACKTRACE: 1                  // 开启 Rust panic 时的 backtrace 功能
            },
        },
    };
    const serverOptions: ServerOptions = {
        run,
        debug,
    };


    // 控制语言客户端的选项
    const clientOptions: LanguageClientOptions = {
        // 注册 多 语言 服务器，注意还需要更改 activationEvents, in package.json
        // TODO The use of a string as a document filter is deprecated @since 3.16.0.
        documentSelector: [
            { scheme: 'file', language: 'lisp' },
            { scheme: 'file', language: 'scheme' },
            { scheme: 'file', language: 'c' },
            { scheme: 'file', language: 'python' },
            { scheme: 'file', language: 'javascript' },
            { scheme: 'file', language: 'rust' },
        ],
        synchronize: {
            // 当文件变动为'.clientrc'中那样时，通知服务器
            fileEvents: workspace.createFileSystemWatcher('**/.clientrc')
        },
        traceOutputChannel,
    };

    // 创建语言客户端并启动客户端。
    client = new LanguageClient(
        'EgglanguageServer', // 它是客户端的名称，与服务端配置文件中指定的名称相同。
        'Egg Language Server', // 对客户端的描述，将会在用户界面中显示。
        serverOptions,
        clientOptions
    );

    // 启动客户端。这也将启动服务器
    client.start();

    // * 注册命令 注意服务器和客户端的命令分开注册就行了

    // 该命令已在 package.json 文件中定义
    // 现在用 registerCommand 提供命令的实现
    // commandId 参数必须与 package.json 中的命令字段匹配
    const disposable = commands.registerCommand('EgglanguageServer.restart', () => {
        window.showInformationMessage('EgglanguageServer.restart! 但是是未实现的命令 QAQ');
    });

    context.subscriptions.push(disposable);
    window.showInformationMessage('Welcome to use Egg Language Server! 😊');
}

// 消动函数
export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
