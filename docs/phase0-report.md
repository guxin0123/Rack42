# Phase 0 技术验证报告

> 日期：2026-11-04
> 范围：RDP 临时 `.rdp`、Xshell/Xftp 7.x、Windows OpenSSH 兜底
> 说明：测试机地址与账号来自本地 `.secrets`，本报告不记录任何 IP、用户名或密码。

## 1. 验证环境

- Windows 11，系统自带 `mstsc.exe`
- Windows OpenSSH Client `9.5p2`（`ssh.exe`、`sftp.exe`）
- Xshell 7.0.0026（文件版本 7.0.0.31）
- Xftp 7.0.0026（文件版本 7.0.0.30）
- Xshell/Xftp 为绿色版，**不在 PATH 中**，程序路径需要在设置中手动配置
- PuTTY / psftp 未安装
- 测试机：RDP 3389 端口可达，SSH 22 端口未开放

## 2. RDP：临时 `.rdp` + DPAPI

### 2.1 验证方法

1. 使用当前 Windows 用户的 DPAPI（`ProtectedData.Protect`，`CurrentUser`）加密 UTF-16LE 密码；
2. 生成临时 `.rdp`：
   - `full address:s:<host>`
   - `username:s:<user>`
   - `password 51:b:<DPAPI blob>`
   - `prompt for credentials:i:0`
3. 通过 `mstsc.exe "<temp.rdp>"` 启动，命令行不包含密码；
4. 连接成功后关闭 mstsc，删除临时文件。

### 2.2 结果

- 连接成功，mstsc 窗口标题显示测试机地址，画面进入远程桌面；
- 没有出现 Windows 凭据输入框；
- `.rdp` 文件中没有明文密码，只有 DPAPI 密文；
- 未在 Credential Manager 中新增测试机的 `TERMSRV` 条目；
- 用户已有的其他 `TERMSRV` 凭据没有被覆盖或删除；
- 测试结束后临时 `.rdp` 已删除。

### 2.3 空密码场景

生成不含 `password 51` 的 `.rdp` 后启动：

- mstsc 弹出“Windows 安全中心 / 输入你的凭据”；
- 用户名已预填，密码留空；
- 行为符合“密码为空则跳过自动填充、由客户端询问”的预期。

### 2.4 生产实现注意事项

- 本次自动化测试使用了 `authentication level:i:0` 避免证书警告；正式实现应使用 `authentication level:i:2`，并让 mstsc 正常展示证书确认。
- `password 51` 在部分 Windows 策略/版本上可能被忽略，需要保留 `CredWriteW` 备选路径。
- 需要验证非默认端口和域账号；本次测试机为默认 3389、本地账号，未覆盖这两项。
- Rust 侧需用 `CryptProtectData`（空 entropy、CurrentUser）生成与 .NET 等价的 blob，并补一个逐字节/可连接性验证。
- 临时 `.rdp` 生命周期、mstsc 进程等待、异常退出后的启动清理，仍需在 Phase 2 实现并测试。

### 2.5 安全结论

- **通过**：默认配置下密码不进入命令行；
- **通过**：不写全局凭据、不覆盖用户已有 RDP 凭据；
- 临时 `.rdp` 含 DPAPI 密文，必须放在用户专属临时目录、限制 ACL，并在会话结束/下次启动清理；
- 不建议在生产中回退到 `cmdkey /pass`。

## 3. SSH / SFTP

### 3.1 Xshell 7.x

验证命令（无密码，符合默认“不自动填充”策略）：

```text
Xshell.exe -url ssh://<user>@<host>:<port>
```

结果：

- Xshell 成功接收 `-url` 参数；
- 打开连接标签并尝试连接测试机 22 端口；
- 因测试机 SSH 未开放，显示 `Connection failed`；
- 说明 7.x 的 `-url` 语法可用，用户名/主机/端口解析正常；
- 密码字段未验证，也不建议通过 URL 传递密码。

### 3.2 Xftp 7.x

验证命令：

```text
Xftp.exe -url sftp://<user>@<host>:<port>
```

结果：

- Xftp 成功接收 `-url` 参数；
- 打开文件传输窗口，标题显示测试机 `(Disconnected)`；
- 因测试机 SSH/SFTP 未开放，连接失败；
- 说明 Xftp 7.x 支持 `-url sftp://` 形式，可以用于密码留空、由客户端询问的场景。

### 3.3 Windows OpenSSH 兜底

- `ssh.exe` 9.5p2、`sftp.exe` 均存在；
- 可用作未安装 Xshell/Xftp 时的兜底；
- 本次测试机无 SSH 服务，未做端到端登录验证；
- 建议以新控制台启动，让 OpenSSH 自己提示密码，不传 `-pw`。

### 3.4 PuTTY / psftp

- 本机未安装，未做命令验证；
- 实施时仍按计划支持 `-P/-l/-pw` 参数，但默认不传密码；
- 需要在有 PuTTY 的环境补一次验证。

## 4. 结论与决策

1. **RDP 采用临时 `.rdp` + DPAPI 方案**，通过本次验证；保留 `CredWriteW` 作为兼容备选。
2. `authentication level` 生产值改为 2，测试值 0 仅用于自动化。
3. **Xshell/Xftp 7.x 的 `-url` 语法确认可用**（无密码形式）；自动填充密码仍默认关闭。
4. 绿色版客户端不在 PATH，设置页必须支持手动选择路径，并在连接前校验存在性。
5. Windows OpenSSH 作为兜底方案保留。
6. 临时文件和 `rdp_sessions` 清理逻辑是实现阶段必须补上的部分。

## 5. 未覆盖项

- 非默认 RDP 端口、域账号；
- 测试机 SSH/SFTP 端到端（当前 22 端口关闭）；
- Xshell/Xftp 7.x 在 URL 中携带密码的特殊字符编码（不建议启用）；
- PuTTY/psftp 实际参数验证；
- mstsc 进程退出/崩溃清理的实际行为；
- Rust `CryptProtectData` 与 .NET DPAPI blob 的等价性（需实现后补测）。

## 6. 下一步

- Phase 2：实现 RDP 临时文件生成、DPAPI、mstsc 启动/等待/清理、浏览器打开；
- Phase 3：接入 Xshell/Xftp 7.x 与 Windows OpenSSH，设置页配置客户端路径；
- 建议准备一台可 SSH/SFTP 的 Linux 测试机，补全 Xshell/Xftp/OpenSSH 端到端验证。
