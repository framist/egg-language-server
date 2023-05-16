[*中文*](#egg-language-server-介绍) | [*English*](#egg-language-server-readme)


**仍在开发中**

<p align="center">
  <img src="./doc/asserts/cog-egg.png" alt="temp logo" width="10%" />
</p>


# egg-language-server 介绍

egg 驱动的编写时代码优化 语言服务器及 Visual Studio Code 插件。

## 特性

![demo](./doc/asserts/example1.png)

egg-language-server 包括一个语言服务器及一个 Visual Studio Code 插件。目前支持 lisp、python、JavaScript 语言的**子集**，未来预计会支持更多语言。目前，它在 Python 上工作最好

egg-language-server 可以帮助您：

- 优化程序结构，提升代码性能。
- 简化源码本身。
- 提升您的能力和代码质量。

egg 的源码优化主要分为以下过程：

1. Code -> AST：基于 Tree-sitter
2. AST -> IR：针对特定目标语言分别实现的 `ast_to_sexpr`
3. IR <-> IR: 构造基本元素抽象、过程抽象和数据抽象的 `CommonLanguage`。通过 egg 进行 Rewrite。
4. IR -> AST：Common Language 自动派生方法
5. AST -> Code：针对特定目标语言分别实现的 `rpn_to_human`

## 依赖

参见 `Cargo.toml` 文件

## 设置

此扩展提供以下设置:

- `EgglanguageServer.maxNumberOfProblems`: 
  - 类型：`number`
  - 描述：控制最多报告问题的数量
  - 默认值：`100`
  - 最小值：`1`

- `EgglanguageServer.ifExplanations`: 
  - 类型：`boolean`
  - 描述：控制 是否显示 egg 重写方案解释
  - 默认值：`true`

- `EgglanguageServer.ExplanationWithLet`: 
  - 类型：`boolean`
  - 描述：控制 是否显示 egg let 风格的重写方案解释
  - 默认值：`true`
  - 依赖项：`EgglanguageServer.ifExplanations: true`

- `EgglanguageServer.ifEggIR`: 
  - 类型：`boolean`
  - 描述：控制 是否显示egg的中间表示
  - 默认值：`true`

- `EgglanguageServer.outLanguage`: 
  - 类型：`string`
  - 枚举：
    - `same as source`
    - `debug`
    - `python`
    - `C`
    - `lisp`
    - `javascript`
  - 描述：控制 输出的优化结果参考的伪代码语言类型
  - 默认值：`same as source`

- `EgglanguageServer.trace.server`: 
  - 类型：`string`
  - 枚举：
    - `off`
    - `messages`
    - `verbose`
  - 描述：跟踪 VS Code 和语言服务器之间的通信
  - 默认值：`off`


## 开发

### 结构

```
.
├── client // 语言客户端
│   └── src
│       ├── test // 语言客户端/服务器的端到端测试
│       └── extension.ts // 语言客户端入口点
├── package.json // 扩展清单
└── server // 语言服务器
    └── src
        └── main.rs // 语言服务器入口点
```

### 运行

0. 在此文件夹上打开 VS Code
1. 在此文件夹中运行 `npm install` , 这将在客户端文件夹中安装所有必要的 npm 模块
2. `cargo build`
3. 按下 <kbd>F5</kbd> 或者
   1. 切换到侧栏中的运行和调试视图 (Ctrl+Shift+D).
   2. 从下拉列表中选择 `Run Extension (Debug Build)` (如果尚未默认选择)。
   3. 点击 ▷ 运行启动配置 (F5).


### 基准测试

要获取每个测试运行时的简单csv，您可以设置环境变量
将 “EGG_BENCH_CSV” 添加到要将每个测试附加到csv的行的内容。

示例:

```bash
EGG_BENCH_CSV=common.csv cargo test --package egg-language-server --lib -- egg_support::common --nocapture --test --test-threads=1
```


## 已知问题

many

## 发行说明

暂未发行

## 参考

- 本项目同时也是作者的毕业设计，但是论文还未完成。
- 本项目作者对 egg 的论文《Egg: Fast and Extensible Equality Saturation》进行了中文翻译，可以在 [这里](https://www.overleaf.com/read/jhnbztftxwhm) 查看目前翻译的进度。待翻译完成后，会将其放在本项目的 `doc` 文件夹中。 [预览](./doc/asserts/Fast_and_Extensible_Equality_Saturation_zh_cn.pdf)
- 作为本项目的学习基础，作者有以下笔记：
  - [Software-Foundations-Note](https://github.com/framist/Software-Foundations-Note) 
  - [CS61a-Note](https://framist.github.io/2022/12/19/CS61a-Note/)

---

*English*

# egg-language-server README

## Features

## Requirements

## Extension Settings

* 'myExtension.enable': 启用/禁用此扩展。
* 'myExtension.thing': 设置为 'blah' 做某事。


## Development

### Structure

```
.
├── client // Language Client
│   └── src
│       ├── test // End to End tests for Language Client / Server
│       └── extension.ts // Language Client entry point
├── package.json // The extension manifest.
└── server // Language Server
    └── src
        └── main.rs // Language Server entry point
```

### Running

0. Open VS Code on this folder.
1. Run `npm install` in this folder. This installs all necessary npm modules in both the client and server folder
2. `cargo build`
3. press <kbd>F5</kbd> or 
   1. Switch to the Run and Debug View in the Sidebar (Ctrl+Shift+D).
   2. Select `Launch Client` from the drop down (if it is not already).
   3. Press ▷ to run the launch config (F5).

## Known Issues

## Release Notes

## References

---

⚡ Visitor count

![](https://profile-counter.glitch.me/framist-egg-language-server/count.svg)
