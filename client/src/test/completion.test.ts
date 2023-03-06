/* --------------------------------------------------------------------------------------------
 * Copyright (c) Microsoft Corporation. All rights reserved.
 * Licensed under the MIT License. See License.txt in the project root for license information.
 * ------------------------------------------------------------------------------------------ */

import * as vscode from 'vscode';
import * as assert from 'assert';
import { getDocUri, activate } from './helper';
/* 
在这个测试中，我们：

激活了插件
带上了一个URI和位置模拟信息，然后运行了vscode.executeCompletionItemProvider去触发补全
断言返回的补全项是不是达到了我们的预期
*/
/*
这段代码用于测试，它会首先通过 getDocUri() 函数得到一个文档 URI，
然后调用 testCompletion() 并提供这个 URIL 和位置作为参数。
函数 testCompletion() 会检查在文档的这个位置上有哪些补全项，
这里应该有 JavaSccript 和 TypeScript。 
所以它会创建两个 vscode.CompletionItem，
每个都有 label 和 kind 属性，并使用这两个条目去测试代码补全。
*/
suite('Should do completion', () => {
	const docUri = getDocUri('completion.txt');

	test('Completes JS/TS in txt file', async () => {
		await testCompletion(docUri, new vscode.Position(0, 0), {
			items: [
				{ label: 'JavaScript', kind: vscode.CompletionItemKind.Text },
				{ label: 'TypeScript', kind: vscode.CompletionItemKind.Text }
			]
		});
	});
});

async function testCompletion(
	docUri: vscode.Uri,
	position: vscode.Position,
	expectedCompletionList: vscode.CompletionList
) {
	await activate(docUri);  // 它被定义在client/src/test/helper.ts中

	// Executing the command `vscode.executeCompletionItemProvider` to simulate triggering completion
	// 执行 `vscode.executeCompletionItemProvider` 命令，模拟激活代码补全功能
	const actualCompletionList = (await vscode.commands.executeCommand(
		'vscode.executeCompletionItemProvider',
		docUri,
		position
	)) as vscode.CompletionList;

	assert.ok(actualCompletionList.items.length >= 2);
	expectedCompletionList.items.forEach((expectedItem, i) => {
		const actualItem = actualCompletionList.items[i];
		assert.equal(actualItem.label, expectedItem.label);
		assert.equal(actualItem.kind, expectedItem.kind);
	});
}
