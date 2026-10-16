# Nauterm-A

Nauterm-A 是基于 [korvect/Nauterm](https://github.com/korvect/nauterm) 的 Android 实验性移植，保留原项目的 Flutter + Rust 架构以及桌面式主机管理界面。建议在安卓大屏幕上配合键盘鼠标使用。

> 本项目是 fork；原作者与原项目的版权及 [许可证](LICENSE) 保持不变。本仓库的 Android 适配尚未得到上游认可，也不是上游官方 Android 版本。

## 当前状态

- 安卓端已可启动、管理主机、加载 Rust 终端核心和 SQLCipher 加密数据库；数据库密钥由 Android Keystore 保护。
- 保留 SSH 终端、SFTP 和桌面布局的现有代码；**远程连接及文件传输仍需更多实机验证**。
- Android 暂用 Alacritty 终端后端；本地 Shell、串口、FIDO2 HID 和 Mosh 暂不支持。
- 当前以 Android x86_64 模拟器测试；Rust 构建脚本也支持 arm64。桌面版仍可使用原项目的入口和构建流程。
- MCP 尚未实现，原项目将其列为后续计划。

## 下载与构建

Android 测试包请到 [Releases](https://github.com/noerror6/nauterm-a/releases) 下载；如果暂无包，可按 [安卓构建说明](android/README.md) 自行构建。安卓预发布由 `android-v*` 标签触发 GitHub Actions 自动构建，不依赖上游桌面版发布流程。测试包使用调试签名，不适合作为正式商店发行版。

使用安卓模拟器时，可指定大屏显示器运行：

```sh
adb -s emulator-5554 install -r build/app/outputs/apk/debug/app-debug.apk
adb -s emulator-5554 shell am start --display 2 -n com.korvect.nauterm/.MainActivity
```

## 上游文档

- [上游中文说明](README.zh-CN.md)：完整桌面功能与安全设计。其“仅支持桌面平台”等描述针对上游，不代表本 fork 的安卓实验版本。
- [上游项目](https://github.com/korvect/nauterm)

问题反馈请使用本仓库的 [Issues](https://github.com/noerror6/nauterm-a/issues)。报告问题时请勿上传服务器密码、私钥、数据库或终端日志。
