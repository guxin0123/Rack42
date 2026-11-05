# Rack42 凭据类型与远程连接改造计划

> 状态：Phase 0/1/2/3/4/5 已完成（审计、安全基线、手动测试矩阵与文档）
>
> 验证报告：[Phase 0](phase0-report.md) · [Phase 3](phase3-report.md)
> 手动测试：[manual-test-matrix.md](manual-test-matrix.md)
> 版本：v0.1（定稿）
> 范围：凭据类型、凭据地址/端口、设备内凭据管理、设备列表连接入口、RDP/SSH/SFTP/浏览器启动器
> 目标平台：Windows 10/11（Tauri 2 + Rust + Vue 3）

## 1. 结论与原则

`更改.txt` 的产品方向合理：把“连接方式”从设备字段升级为凭据类型，把地址、端口、账号、密码放在同一条凭据里，可以让一台设备同时拥有 RDP、SSH、iDRAC、ESXi、Web 等多个入口。

但原稿不能按字面直接实现，必须先解决三个问题：

1. **密码不能进入命令行**：`cmdkey /pass`、`putty -pw`、`Xshell ssh://user:pass@...` 都会让明文密码出现在进程命令行、EDR 和 Windows 审计日志中。
2. **数据模型不完整**：缺类型表、端口、空密码、域、凭据默认项，以及 `config_addr` 的移除方案。
3. **RDP 凭据生命周期不可靠**：`cmdkey` 是用户级全局凭据，直接覆盖/删除会影响用户已有凭据，崩溃后也可能残留。

设计原则：

- 密码只在 Rust 侧解密，**不返回给 WebView**，更不进入 shell/命令行（默认策略）。
- 所有外部程序用参数数组启动，禁止 `cmd /c`、`powershell -Command` 拼接。
- 删除设备字段前先迁移数据，不做静默丢字段。
- 连接器是白名单枚举，不允许在类型里保存任意命令。
- 连接行为可审计，但审计日志永远不记录密码。

## 2. 目标数据模型

### 2.1 新增/调整表

```sql
CREATE TABLE credential_types (
    id           INTEGER PRIMARY KEY AUTOINCREMENT,
    slug         TEXT NOT NULL UNIQUE,   -- rdp / ssh / idrac / esxi / web / other
    name         TEXT NOT NULL,
    icon         TEXT NOT NULL DEFAULT 'key',  -- 内置图标 key，后续支持自定义
    actions      TEXT NOT NULL DEFAULT '[]',  -- JSON: ["rdp"] / ["ssh","sftp"] / ["web"]
    default_port INTEGER,
    is_builtin   INTEGER NOT NULL DEFAULT 0,
    sort_order   INTEGER NOT NULL DEFAULT 0
);

ALTER TABLE credentials ADD COLUMN type_id INTEGER REFERENCES credential_types(id);
ALTER TABLE credentials ADD COLUMN address TEXT;
ALTER TABLE credentials ADD COLUMN port INTEGER;
ALTER TABLE credentials ADD COLUMN domain TEXT;
ALTER TABLE credentials ADD COLUMN is_default INTEGER NOT NULL DEFAULT 0;

CREATE UNIQUE INDEX idx_credentials_default
    ON credentials(device_id, type_id) WHERE is_default = 1;
```

说明：

- `password` 继续使用现有加密文本列，但允许 `NULL`，表示“没有保存密码，由客户端弹窗输入”。
- `address` 允许为空：为空时连接时动态取 `device.ips[0]`，不要创建时复制，避免设备主 IP 变更后凭据仍指向旧地址。
- `port` 为空时使用类型默认端口（RDP 3389、SSH 22、Web 443）。
- `domain` 主要给 RDP 使用，也允许 `DOMAIN\user` 直接写在 `username` 中。
- `actions` 用 JSON 数组，避免为几个动作单独建关联表；后端负责解析和校验。

### 2.2 内置类型种子

| slug | 名称 | actions | 默认端口 | 备注 |
|---|---|---|---|---|
| rdp | RDP | `["rdp"]` | 3389 | mstsc |
| ssh | SSH | `["ssh","sftp"]` | 22 | 终端 + 文件传输两个动作 |
| idrac | iDRAC | `["web"]` | 443 | 默认 https |
| esxi | ESXi | `["web"]` | 443 | 默认 https |
| web | Web | `["web"]` | 443 | 补上原稿缺失的 Web 类型 |
| other | 其他 | `[]` | — | 默认不提供连接动作，可自行配置 |

内置类型允许修改名称、图标、默认端口、动作集合，但**不允许删除**；自定义类型删除时，其凭据统一迁移到 `other`。

### 2.3 地址与端口规则

- 新建凭据时默认勾选“跟随设备主 IP”，此时 `address` 为空，界面显示设备主 IP 占位。
- 用户关闭“跟随”后才写入固定地址，适用于 NAT、跳板、独立管理网等场景。
- 连接时统一解析实际地址：`credential.address` 优先，否则 `device.ips[0]`；两者都没有则报错“未配置地址”。
- 端口按“凭据端口 → 类型默认端口 → 协议默认端口”回退。
- Web 地址：如果用户填完整 URL（`https://host/path`），保留 scheme 和 path；如果只填主机，则按类型默认 scheme 拼成 `https://host:port`。只允许 `http`/`https`。

## 3. 连接器设计

### 3.1 通用规则

- 连接入口统一为 Rust 命令：`connect_credential(credential_id, action)`。
- 命令内部解密密码，前端只拿到 `{ started: true }` 或错误信息，永不拿到密码。
- 使用 `std::process::Command` 参数数组，禁止 shell；所有参数先做长度/字符/端口校验。
- 启动外部程序后立刻返回，后台任务负责等待进程退出和清理。
- 密钥使用 `zeroize` 清理；日志和错误信息脱敏。
- 客户端路径必须存在、必须是预期文件名（如 `Xshell.exe`、`putty.exe`），启动前再次校验。

### 3.2 RDP（mstsc）

原稿的 `cmdkey` 流程存在三个问题：

1. 明文密码出现在 `cmdkey` 命令行；
2. 会覆盖并删除用户已有的 `TERMSRV/<host>` 凭据；
3. 应用崩溃后只能等下次启动清理，期间凭据一直存在。

**建议方案（优先）**：临时 `.rdp` 文件 + DPAPI 加密密码。

- 生成临时 `.rdp` 到 `%TEMP%\Rack42\`，写入：
  - `full address:s:<host>:<port>`
  - `username:s:<user>`
  - `domain:s:<domain>`
  - `password 51:b:<DPAPI 加密后的 UTF-16LE 密码>`
  - 空密码时省略 `password 51`，并设置 `prompt for credentials:i:1`
- 用 `mstsc.exe "<temp.rdp>"` 启动，参数数组，不拼 shell。
- 后台等待 mstsc 退出，删除临时文件；启动时扫描并清理遗留 `.rdp`。
- 优点：不写全局 Credential Manager，不覆盖用户已有凭据，命令行无密码。
- 风险：部分 Windows 版本/策略可能忽略 `password 51`，需要用 **Phase 0 spike** 在目标 Win10/11 上验证；若不生效，退回方案 B。

**备选方案 B**：`CredWriteW` / `CredDeleteW`。

- 使用 `CRED_TYPE_GENERIC`、target `TERMSRV/<host>`，优先 `CRED_PERSIST_SESSION`。
- 写入前 `CredReadW` 检查已有凭据，提示“将覆盖已有 RDP 凭据”，并尽量备份/恢复。
- 需要实际测试非默认端口下 target 是 `TERMSRV/host` 还是 `TERMSRV/host:port`。

**生命周期**：

- `rdp_sessions(id, target, temp_path, created_at)` 记录本次连接资源。
- 正常退出：删除临时文件/凭据并删除记录。
- 异常退出：下次启动按记录清理；清理失败保留记录并写日志。
- 密码为空：跳过凭据写入，由 mstsc 自己弹登录框。

### 3.3 SSH 终端

**默认策略：不自动传密码**（`autoFillPassword = false`）：

- 启动客户端时只传主机、端口、用户名，让客户端弹密码框；
- 同时提供“复制密码到剪贴板（30 秒后清空）”的按钮/动作，由用户粘贴；
- 这样密码不会出现在进程命令行，保持现有加密凭据模型的安全边界。

**可选策略：自动填充**（用户显式开启，界面红字警告）：

- PuTTY：`putty.exe -ssh -P <port> -l <user> -pw <password> <host>`
- Xshell 7.x：`Xshell.exe -url ssh://<user>:<urlencoded-password>@<host>:<port>`（需验证 7.x 是否支持及特殊字符编码）
- Windows OpenSSH 兜底：`ssh.exe -p <port> <user>@<host>`，在新控制台启动，由系统提示密码（不传 `-pw`）
- 自动填充时密码会出现在子进程命令行；设置页必须明确提示，默认关闭。

注意：原稿 `putty.exe -ssh 用户名@服务器IP 端口号 -pw 密码` 的参数是错的，PuTTY 端口必须用 `-P`。

### 3.4 SFTP / 文件传输

与 SSH 分开配置、分开动作：

- `sftp` 客户端可选：`psftp` / `Xftp` / Windows OpenSSH `sftp.exe`。
- psftp：`psftp.exe -P <port> -l <user> -pw <password> <host>`
- Xftp 7.x：需实测其命令行是否支持 `-url sftp://...`；若不支持，不强行复用 SSH 规则，改用 psftp/OpenSSH。
- Windows OpenSSH：`sftp.exe -P <port> <user>@<host>`，密码由系统提示。
- 默认同样不自动传密码。

### 3.5 浏览器

- 只打开凭据配置的地址，不传用户名密码。
- 只允许 `http` / `https`；拒绝 `file:`、`javascript:`、`ms-*` 等协议。
- 使用 `ShellExecuteW` 或 Tauri opener 打开，不经过 `cmd /c start`。
- 地址拼接规则与 2.3 一致。

## 4. 后端接口

新增/调整 Tauri 命令（示意）：

```text
list_credential_types() -> Vec<CredentialType>
create_credential_type(input) -> CredentialType
update_credential_type(id, input) -> CredentialType
delete_credential_type(id) -> ()

list_credentials(device_id) -> Vec<CredentialSummary>   // 增加 type/address/port/hasPassword
create_credential(input) -> CredentialSummary
update_credential(id, input) -> CredentialSummary
delete_credential(id) -> ()

list_device_connections() -> Vec<DeviceConnectionSummary>
  // { deviceId, types: [{ typeId, slug, icon, actions, credentialCount, defaultCredentialId }] }

connect_credential(credential_id, action) -> ConnectionStarted
  // 解密、校验、启动外部程序、登记清理任务；不返回密码

get_connection_settings() -> ConnectionSettings
update_connection_settings(settings) -> ConnectionSettings
detect_client_path(client) -> Option<String>
copy_credential_password(credential_id) -> ()   // 可选：写剪贴板，不返回明文
```

`ConnectionSettings`：

```json
{
  "sshClient": "putty",
  "sshPath": "C:\\Program Files\\PuTTY\\putty.exe",
  "sftpClient": "psftp",
  "sftpPath": "C:\\Program Files\\PuTTY\\psftp.exe",
  "autoFillPassword": false
}
```

设置项存入现有 `settings` 表（key = `connection`，value = JSON），不新增表。

## 5. 前端交互

### 5.1 设备编辑

- 弹窗改为两个标签页：**基本信息**、**凭据**。
- 基本信息移除“配置地址”字段；显示迁移说明或“原配置地址已迁移到凭据”。
- 凭据页：
  - 列表展示类型图标、标签、地址、用户名、端口、是否默认；
  - 新增/编辑凭据使用子弹窗或内联表单；
  - 类型选择后自动带出默认端口和动作；
  - 地址输入框带“跟随设备主 IP”开关，默认开启；
  - 密码允许留空，提示“留空则连接时由客户端询问”；
  - 同设备同类型可设置一条“默认凭据”。
- 新设备没有 `device_id`：凭据页先提示“保存设备后可添加凭据”，保存后再打开凭据页；或在保存设备的同时提交待创建凭据列表（二选一，推荐先保存）。

### 5.2 设备列表

- 操作栏增加连接图标组，按凭据类型展示：
  - RDP：远程桌面图标；
  - SSH：终端 + 文件传输；
  - iDRAC/ESXi/Web：浏览器图标；
  - 其他：若配置了动作则展示，否则不展示。
- 同类型多条凭据时，点击图标弹出小菜单选择具体凭据（显示标签/用户名）。
- 连接中禁用按钮；客户端缺失、地址为空、密码为空等错误用 Toast 提示，并给出“打开设置”链接。
- 设备列表接口只取类型/动作/凭据 ID，不取密码。

### 5.3 设置页

新增“远程连接”区块：

- SSH 客户端：Xshell 7.x / PuTTY / Windows OpenSSH；
- SSH 客户端路径（选择文件，校验存在和文件名）；
- SFTP 客户端：Xftp 7.x / psftp / Windows OpenSSH；
- SFTP 客户端路径；
- “自动填充密码到命令行”开关，默认关闭，开启时显示红色风险提示；
- “测试客户端”按钮：只启动版本/帮助命令，确认路径可用。

文件选择使用原生对话框（新增 `tauri-plugin-dialog` 依赖与权限），或先提供手动路径输入，后续再加选择器。

### 5.4 凭据类型管理

设置页新增“凭据类型”：

- 表格：图标、名称、动作、默认端口、内置/自定义；
- 自定义类型可新增/编辑/删除；内置类型不可删除；
- 动作用复选框（RDP / SSH / SFTP / Web），至少保留一个或允许为空；
- 图标第一阶段用内置图标 key 选择；第二阶段支持上传 PNG/SVG（复制到应用数据目录，限制大小并清洗 SVG）。

## 6. 数据迁移

新增 `0003_credentials_types.sql`：

1. 创建 `credential_types` 并写入 6 条内置种子。
2. 给 `credentials` 增加 `type_id`、`address`、`port`、`domain`、`is_default`。
3. 现有凭据统一设为 `other`（不根据 label 猜测类型，避免误判）；用户可后续修改。
4. **直接删除 `devices.config_addr`**：已确认无历史数据，不做回填、不保留过渡列，避免继续维护两套地址。
5. 增加的索引：`credentials(device_id)`、`credentials(type_id)`、默认凭据唯一索引。

迁移测试必须覆盖：空旧库、已有凭据的旧库（凭据归入 `other`）、删除 `config_addr` 后的设备 CRUD。

## 7. 实施阶段

### Phase 0：技术验证（1–2 天）

> 需要准备：一台可连接的 Windows RDP 目标机、Xshell/Xftp 7.x、PuTTY、Windows OpenSSH。

- 验证 mstsc 对临时 `.rdp` + `password 51` 的支持（Win10/11、默认/非默认端口、域账号、空密码）。
- 验证 `CredWriteW` 方案作为备选。
- 验证 Xshell/Xftp 7.x 命令行能力与特殊字符处理。
- 产出结论，确定 RDP 最终方案。

### Phase 1：数据模型与迁移（2–3 天）

- 新增类型表、凭据字段、默认凭据索引。
- 编写 `0003` 迁移与旧库升级测试。
- 允许空密码，调整创建/更新校验与 `hasPassword`。
- 完成类型/凭据后端接口与单元测试。

### Phase 2：RDP + 浏览器（2–3 天）

- 实现 RDP 启动器、临时文件/凭据清理、启动时残留清理。
- 实现浏览器打开与 URL 校验。
- 设备列表连接入口（先支持单凭据）。
- 手动测试默认/非默认端口、空密码、取消登录、异常退出。

### Phase 3：SSH/SFTP（2–3 天）

- 实现 PuTTY / Xshell / psftp / Xftp / Windows OpenSSH 参数构造。
- 设置页客户端类型与路径、校验、测试按钮。
- 默认不传密码；实现可选自动填充和剪贴板方案。
- 手动测试缺少客户端、路径错误、特殊字符密码、主机密钥提示。

### Phase 4：设备内凭据与类型管理 UI（2–3 天）

- 设备编辑弹窗标签页、凭据增删改、默认凭据、跟随主 IP。
- 设备列表多凭据菜单与错误提示。
- 凭据类型管理页面、内置图标选择。
- 自定义图标上传（可放 Phase 5）。

### Phase 5：加固与验收（1–2 天）

- 连接审计日志（不记录密码）。
- RDP 凭据备份/恢复或临时文件清理的异常场景测试。
- 安全检查：确认默认配置下密码不出现在命令行。
- README、更新说明、迁移说明。

总计约 10–14 个开发日；若只做 MVP（类型 + RDP + 浏览器 + PuTTY + 空密码），约 5–7 天。

## 8. 验收标准

- 更改.txt 中所有条目有对应实现，且类型/地址/端口/空密码语义明确。
- 默认设置下，任何连接动作的进程命令行中都不出现密码。
- 设备列表只根据凭据类型显示动作，不暴露密码。
- 设备表不再包含 `config_addr`，旧库升级不因删列失败，设备 CRUD 正常。
- RDP 连接结束后临时文件/凭据被清理；应用崩溃后下次启动能清理。
- 用户已有的 RDP 凭据不会被静默覆盖或删除。
- SSH/SFTP 客户端缺失或路径错误时有明确提示。
- 多凭据同类型时能选择默认凭据。
- `cargo test`、`cargo clippy -D warnings`、`npm run build` 全部通过。

## 9. 主要风险与对策

| 风险 | 对策 |
|---|---|
| mstsc 忽略 `.rdp` 的 password 51 | Phase 0 验证；失败则改用 `CredWriteW` + 备份恢复 |
| 命令行泄漏密码 | 默认不自动填充；自动填充需显式开启并警告 |
| 覆盖用户已有 RDP 凭据 | 优先临时 `.rdp`；若用 Credential Manager，先检测/备份/恢复 |
| Xshell/Xftp 版本差异 | 参数构造按客户端版本做适配；不支持则回退 Windows OpenSSH |
| 自定义客户端路径被滥用 | 白名单文件名 + 路径校验 + 参数数组，禁止 shell |
| 主密码最低 4 位 | 连接前二次确认；自动填充默认关闭 |
| 旧库删除 `config_addr` 失败 | 升级前自动备份数据库；在旧库快照上测试 `0003`；失败则中止迁移并保留原库 |

## 10. 已确认的决策

1. SSH/SFTP **默认不自动填充密码**，需要显式开启；默认通过客户端登录框或剪贴板完成。
2. RDP 按 Phase 0 验证结果优先采用**临时 `.rdp` + DPAPI**，备选 `CredWriteW`/`CredDeleteW`。
3. 支持 **Xshell / Xftp 7.x**；同时接受 **Windows OpenSSH** 作为 SSH/SFTP 兜底。
4. `config_addr` **直接删除**，无需保留过渡列；当前没有历史数据。
5. **自定义图标在本期实现**，随凭据类型管理一起交付。

## 11. 任务清单

- [x] `0003` 迁移：凭据类型种子、凭据地址/端口/域/默认项、删除 `config_addr`
- [x] 凭据类型表/种子/后端 CRUD
- [x] 凭据字段扩展与空密码
- [x] Phase 0 技术验证报告（RDP/SSH/SFTP 均完成实操）
- [x] 凭据表单支持类型、地址、端口、域、默认项
- [x] 地址跟随主 IP、端口回退
- [x] RDP 启动器与清理
- [x] 浏览器打开与 URL 校验
- [x] PuTTY/Xshell/psftp/Xftp/OpenSSH 参数构造
- [x] 连接设置页与路径校验
- [x] 设备列表连接按钮（RDP/Web/SSH/SFTP，同类型多凭据可弹出选择）
- [x] 设备编辑凭据标签页（增删改、默认凭据、跟随主 IP）
- [x] 类型管理与图标 UI（动作、默认端口、内置图标）
- [x] 自定义图标（PNG/JPEG/WebP，100KB 上限，不支持 SVG）
- [x] 审计日志（动作/设备/凭据标签/目标/结果，不含密码与用户名）
- [x] 安全检查与手动测试矩阵
- [x] README/迁移说明
- [ ] RDP 凭据备份/恢复（可选）
