[*中文*](#egg-support-介绍) | [*English*](#egg-support-readme)

# egg-support 介绍

基于 egg 驱动的编写时代码优化插件

现在还在开发阶段，但是已经可以勉强使用了。

## 特性

## 依赖

## 设置

此扩展提供以下设置:

* 'myExtension.enable': 启用/禁用此扩展。
* 'myExtension.thing': 设置为 'blah' 做某事。

## 开发

### 结构

```
.
├── client // 语言客户端
│   ├── src
│   │   ├── test // 语言客户端/服务器的端到端测试
│   │   └── extension.ts // 语言客户端入口点
├── package.json // 扩展清单
└── server // 语言服务器
    └── src
        └── main.rs // 语言服务器入口点
```

### 运行

0. 在此文件夹上打开 VS Code
1. 在此文件夹中运行 `npm install` , 这将在客户端文件夹中安装所有必要的 npm 模块
2. `cargo build`
3. press <kbd>F5</kbd> or
   1. 切换到侧栏中的运行和调试视图 (Ctrl+Shift+D).
   2. Select `Launch Client` from the drop down (if it is not already).
   3. Press ▷ to run the launch config (F5).


## Known Issues

## Release Notes

---

*English*

# egg-support README

## Features

## Requirements

## Extension Settings

## Development

### Structure

```
.
├── client // Language Client
│   ├── src
│   │   ├── test // End to End tests for Language Client / Server
│   │   └── extension.ts // Language Client entry point
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
