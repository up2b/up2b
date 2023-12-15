![Code](https://s2.loli.net/2023/12/15/ClyB1w7RNnP4EF8.png)

# UP2B

图床管理器。

## 介绍

此程序基于 [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) 开发，适配多平台（Windows、macOS 和 Linux桌面发行版）。

与 PicGo 不同的是，由于核心业务逻辑是用 Rust 实现，暂时无法实现 API 插件功能，只能逐个适配。

写此程序的初衷就是觉得作为一个图床管理器，PicGo 的体积太大了，当然，这也是 Electron 程序的通病。

### 0 下面是与 PicGo 的体积对比

基于 0.2.0beta 版。

|             | PicGo | UP2B |
| ----------- | ----- | ---- |
| Windows x64 |       |      |
| macOS arm64 |       |      |
| Linux x64   |       |      |

### 1 图片上传

![截屏2023-12-15 22.19.46](https://s2.loli.net/2023/12/15/42YRjUmPckleJx9.png)

### 2 图片列表及删除图片

![截屏2023-12-15 22.28.44](https://s2.loli.net/2023/12/15/pvlGhXcr6dZHntR.png)

## CLI

与 PicGo 类似，UP2B 也提供了一个上传图片的 CLI 命令，下面是帮助信息：

```
up2b 0.2.0
thepoy
图床管理客户端

USAGE:
    up2b [SUBCOMMAND]

OPTIONS:
    -h, --help       Print help information
    -V, --version    Print version information

SUBCOMMANDS:
    help      Print this message or the help of the given subcommand(s)
    upload    上传一张或多张图片
```

只有一条有效命令`up2b`。

你可以通过此命令在任何支持图片上传的文本编辑器中上传图片到图床，比如在 Typora 中如此设置：

![截屏2023-12-15 22.25.54](https://s2.loli.net/2023/12/15/i7gSByjX4FtmKxv.png)

就可以直接上传图片了。
