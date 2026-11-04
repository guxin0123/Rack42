# Phase 3 技术验证报告

> 日期：2026-11-04
> 范围：Xshell 7.x、Xftp 7.x、Windows OpenSSH、PuTTY/psftp 参数
> 说明：测试机信息来自本地 `.secrets`，本报告不记录 IP、用户名或密码。

## 1. 验证环境

- Windows 11
- Xshell 7.0.0026（文件版本 7.0.0.31）
- Xftp 7.0.0026（文件版本 7.0.0.30）
- Windows OpenSSH Client 9.5p2
- 测试机 OpenSSH 10.2，22 端口可达
- PuTTY / psftp 未安装

## 2. 验证结果

### Xshell 7.x

- 使用 `-url ssh://<user>@<host>:<port>` 启动，能打开连接并提示主机指纹；
- 使用带 URL 编码密码的 `-url ssh://<user>:<password>@<host>:<port>`，接受主机指纹后成功登录，窗口标题显示远端用户@主机；
- 说明 Xshell 7.x 的 `-url` 与 URL 编码方式可用；
- 自动填充模式下密码会出现在进程命令行，因此产品默认关闭。

### Xftp 7.x

- 使用 `-url sftp://<user>:<password>@<host>:<port>` 启动；
- 成功连接，远端面板显示用户主目录，底部状态为已连接；
- 说明 Xftp 7.x 支持 `-url sftp://`，可作为 SFTP 客户端。

### Windows OpenSSH

- `ssh.exe -p <port> -l <user> <host>` 在新控制台中成功登录；
- `sftp.exe -P <port> <user>@<host>` 在新控制台中成功登录，`pwd` 返回用户主目录；
- 密码由控制台提示输入，命令行不含密码；
- 主机指纹提示正常出现，说明新控制台方案可用。

### PuTTY / psftp

- 本机未安装，无法做实际连接；
- 已通过单元测试验证参数：
  - PuTTY：`-ssh -P <port> -l <user> <host>`，自动填充时追加 `-pw`；
  - psftp：`-P <port> -l <user> <host>`，自动填充时追加 `-pw`；
- 需在安装 PuTTY 的环境补一次实测。

## 3. 安全结论

- 默认 `autoFillPassword = false`：Xshell/Xftp 只收到主机、端口、用户名，密码由客户端提示；
- OpenSSH 始终不接收密码参数，由新控制台提示；
- 开启自动填充后，Xshell/PuTTY/Xftp/psftp 的密码会出现在命令行，设置页已给出红色风险提示；
- 客户端路径保存时校验文件名与存在性；绿色版 Xshell/Xftp 不在 PATH 时需手动配置路径。

## 4. 未覆盖项

- PuTTY / psftp 实际连接（本机未安装）；
- 非默认 SSH 端口、域账号、密钥登录；
- 多凭据选择菜单（Phase 4）；
- 设备编辑弹窗内的凭据标签页（Phase 4）。

## 5. 结论

Phase 3 的 SSH/SFTP 客户端接入方案通过验证。建议默认使用 Windows OpenSSH，已安装 Xshell/Xftp 的用户在设置页选择并配置绿色版路径。
