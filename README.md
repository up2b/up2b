![logo](docs/images/logo.png)

# UP2B

图床管理器。

## 介绍

此程序基于 [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) 开发，适配多平台（Windows、macOS 和 Linux桌面发行版）。

图片的管理和配置完全图形化，上传在支持图形化的同时还支持 CLI 。

与 PicGo 不同的是，由于核心业务逻辑是用 Rust 实现，暂时无法实现 API 插件功能，只能逐个适配。

写此程序的初衷就是觉得作为一个图床管理器，PicGo 的体积太大了，当然，这也是 Electron 程序的通病。

### 0 下面是与 PicGo 的体积对比

|                    | PicGo(2.3.1) | UP2B(0.2.0beta) |
| ------------------ | ------------ | --------------- |
| Windows x64 exe    | 60.6 MB      | 3.84 MB         |
| Windows x64 msi    | 不支持       | 4.66 MB         |
| macOS arm64        | 88.2 MB      | 5.4 MB          |
| Linux x64 AppImage | 101 MB       | 73.8 MB         |
| Linux x64 deb      | 不支持       | 7.3 MB          |

### 1 图片上传

![上传](docs/images/upload.avif)

### 2 图片列表及删除图片

![截屏2023-12-15 22.28.44](docs/images/list.avif)

### 3 设置

![截屏2023-12-15 22.47.26](https://s2.loli.net/2023/12/15/esQrwN8KhnomBTx.png)

### 4 自动压缩（体验）

本程序的特色功能，可以将超过图床体积限制的图片压缩后上传，但此功能尚不稳定，还需改进，而且此功能会导致程序体积增加，考虑到不是所有用户都有此需求，故而我会为增加此功能的程序单独打包一个版本。

![截屏2023-12-15 22.55.49](https://s2.loli.net/2023/12/15/5xbHVlOpwMmtrXe.png)

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

只有一条有效命令`upload`。

你可以通过此命令在任何支持图片上传的文本编辑器中上传图片到图床，比如在 Typora 中如此设置：

![截屏2023-12-15 22.25.54](https://s2.loli.net/2023/12/15/i7gSByjX4FtmKxv.png)

就可以直接上传图片了。
