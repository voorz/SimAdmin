<a href="https://github.com/3899/SimAdmin">
  <img src="https://socialify.git.ci/3899/SimAdmin/image?description=1&descriptionEditable=%E9%9D%A2%E5%90%91%20Debian%20%E5%B9%B3%E5%8F%B0%E6%94%AF%E6%8C%81%20SIM%2FeSIM%20%E8%9C%82%E7%AA%9D%E8%AE%BE%E5%A4%87%E7%9A%84%E5%BC%80%E6%BA%90%20Web%20%E7%AE%A1%E7%90%86%E7%B3%BB%E7%BB%9F&font=Source%20Code%20Pro&logo=https%3A%2F%2Fgithub.com%2F3899%2FSimAdmin%2Fblob%2Fmain%2Ffrontend%2Fpublic%2Fsimadmin-logo.svg%3Fraw%3Dtrue&name=1&owner=1&pattern=Floating%20Cogs&theme=Auto" alt="SimAdmin" />
</a>

<div align="center">
  <br/>

  <div>
    <a href="https://github.com/3899/SimAdmin/releases">
      <img 
        alt="Debian"
        src="https://img.shields.io/badge/Debian-%23D70A53?logo=debian&logoColor=white&style=flat-square" 
      />
    </a >
    <a href="./LICENSE">
      <img
        src="https://img.shields.io/github/license/3899/SimAdmin?style=flat-square"
      />
    </a >
    <a href="https://github.com/3899/SimAdmin/releases">
      <img
        src="https://img.shields.io/github/v/release/3899/SimAdmin?style=flat-square"
      />
    </a >
    <a href="https://github.com/3899/SimAdmin/releases">
      <img
        src="https://img.shields.io/github/downloads/3899/SimAdmin/total?style=flat-square"
      />  
    </a >
  </div>

  <br/>

  <picture>
    <img src="./static/Dashboard.png" width="100%" alt="Dashboard" />
	<br/><br/>
	<img src="./static/Device_Information.png" width="100%" alt="Device_Information" />
	<br/><br/>
	<img src="./static/eSIM.png" width="100%" alt="eSIM" />
	<br/><br/>
	<img src="./static/Cellular_Network.png" width="100%" alt="Cellular_Network" />
	<br/><br/>
	<img src="./static/WLAN.png" width="100%" alt="WLAN" />
	<br/><br/>
	<img src="./static/DDNS.png" width="100%" alt="DDNS" />
	<br/><br/>
	<img src="./static/SMS.png" width="100%" alt="SMS" />
	<br/><br/>
	<img src="./static/Notification.png" width="100%" alt="Notification" />
	<br/><br/>
	<img src="./static/Basic_Configuration.png" width="100%" alt="Basic_Configuration" />
	<br/><br/>
	<img src="./static/Security_Settings.png" width="100%" alt="Security_Settings" />
	<br/><br/>
	<img src="./static/OTA.png" width="100%" alt="OTA" />
	<br/><br/>
	<img src="./static/Dashboard_Dark.png" width="100%" alt="Dashboard_Dark" />
	<br/><br/>
  </picture>
  
</div>

# SimAdmin - SIM/eSIM 中枢

SimAdmin 是一套面向 Debian 蜂窝 CPE、随身 WiFi、软路由类设备的 SIM/eSIM、蜂窝网络、短信和系统状态管理系统。

当前项目由 Rust 后端和 React 前端组成：

- 后端：Rust + Axum + zbus，主要通过 ModemManager D-Bus 接口管理 modem，并在部分场景使用 `mmcli`、`qmicli` 或 AT 直连兜底。
- 前端：React + Vite + Material UI，提供仪表盘、设备信息、蜂窝网络、设备网络、短信、通知配置和 OTA 更新页面。
- 部署形态：后端二进制同进程托管前端 SPA，默认安装到 `/opt/simadmin`，通过 systemd 运行。

健康检查整体按支持 ModemManager 的 Linux 蜂窝设备组织，不同 modem 固件、内核、ModemManager 版本暴露的能力不同，具体功能以实际设备为准。

## 免责声明

本项目会直接操作蜂窝 modem、SIM 注册、数据拨号、APN、频段、飞行模式、NetworkManager、systemd 服务、系统重启和 OTA 文件替换；iptables/ip6tables 仅用于只读诊断，不会自动清空宿主机防火墙规则。

请仅在你拥有控制权的设备上使用。错误配置可能导致断网、无法注册网络、SIM 漫游计费、设备需要手动恢复，甚至 OTA 后服务无法启动。任何使用本项目造成的后果由使用者自行承担。

部分接口受硬件和 ModemManager 能力限制：

- 频段锁定依赖 ModemManager 暴露的 `SupportedBands` / `CurrentBands` / `SetCurrentBands`。
- 小区锁定当前为后端内存态展示，不会下发真实硬件锁小区命令。

## 开源协议声明

本项目采用 GNU General Public License v3.0 (GPLv3) 开源协议。

你可以：

- 自由使用、研究、修改本软件。
- 分发本软件副本。
- 分发修改后的版本。

但你必须：

1. 保留版权声明和许可证声明。
2. 分发本软件或修改版本时，以 GPLv3 协议公开完整源代码。
3. 基于本项目的衍生作品继续使用 GPLv3 协议。
4. 明确标注修改内容和修改日期。
5. 分发时附带完整 GPLv3 许可证文本。

严禁将本项目或其衍生版本闭源后作为专有软件分发。

## 社区交流

⚠️ 温馨提示：群聊仅限日常讨论和经验分享，如需反馈问题或提交新需求。

<table>
  <thead>
    <tr>
      <th width="50%">QQ 群</th>
      <th width="50%">微信群</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td>
        <picture>
          <source media="(prefers-color-scheme: dark)" srcset="./static/Community/Community_QQ_Dark.png" />
          <source media="(prefers-color-scheme: light)" srcset="./static/Community/Community_QQ_Light.png" />
          <img src="./static/Community/Community_QQ_Light.png" />
        </picture>
      </td>
      <td>
        <picture>
          <source media="(prefers-color-scheme: dark)" srcset="https://raw.giteeusercontent.com/v2599/Notify/raw/simadmin/Community_Wechat_Dark.png" />
          <source media="(prefers-color-scheme: light)" srcset="https://raw.giteeusercontent.com/v2599/Notify/raw/simadmin/Community_Wechat_Light.png" />
          <img src="https://raw.giteeusercontent.com/v2599/Notify/raw/simadmin/Community_Wechat_Light.png" />
        </picture>
      </td>
    </tr>
  </tbody>
</table>

## 快速开始

### 项目结构

```text
.
├── backend/          # Rust + Axum 后端，ModemManager、SQLite、OTA、通知、系统接口
├── frontend/         # React + Vite + MUI 前端
├── bruno-api/        # Bruno API 调试集合
├── scripts/          # 构建、部署、systemd、modem 恢复脚本
├── install_latest.sh # 设备侧一键安装 / 升级脚本
├── uninstall.sh      # 设备侧一键卸载脚本
├── VERSION           # 项目版本号
└── LICENSE           # GPLv3 许可证
```

### 前端开发

```bash
cd frontend
pnpm install
pnpm dev
```

构建前端：

```bash
cd frontend
pnpm run build
```

前端构建产物输出到 `frontend/dist/`，部署后会复制为 `/opt/simadmin/www/`。

### 后端开发

```bash
cd backend
cargo check
cargo run -- --host :: --port 3000
```

参数和环境变量：

| 参数 | 环境变量 | 默认值 | 说明 |
|------|----------|--------|------|
| `--host` / `-H` | `HOST` | `::` | 监听地址，默认双栈 IPv4/IPv6 |
| `--port` / `-p` | `PORT` | `3000` | HTTP 监听端口 |

在普通开发机上运行后端时，如果没有 system D-Bus、ModemManager 或 modem，硬件相关接口会返回错误，这是预期行为。

### 登录与管理员密码

SimAdmin 采用后台式单管理员密码登录，不包含用户名和多账号权限系统。首次打开 Web 后台时会进入 `/login` 的“设置管理员密码”页面；设置成功后会自动建立会话并进入管理后台。

密码规则：

- 8-64 个字符。
- 只能使用英文字母、数字和符号，不允许空格或中文。
- 至少包含两类字符，例如字母 + 数字、字母 + 符号或数字 + 符号。

登录与接口保护：

- 管理后台页面和 `/api/*` 业务接口默认需要登录；`/api/health`、`/api/auth/status`、`/api/auth/setup`、`/api/auth/login` 为公开接口。
- 未登录访问受保护页面会跳转到 `/login`；前端 API 请求遇到 `401` 会自动进入登录页，直接调用 API 时返回标准 JSON 错误。
- 会话使用 `simadmin_session` HttpOnly Cookie，默认有效期 7 天。重置或清除管理员密码会清空所有 Web 会话。
- 当前不提供手动登出入口，适合单管理员设备后台场景。

忘记密码时，可通过 SSH 登录目标设备后执行交互式重置：

```bash
/opt/simadmin/simadmin auth reset-password
```

如需清除管理员密码并让 Web UI 下次重新进入首次设置：

```bash
/opt/simadmin/simadmin auth clear
```

如果使用了自定义安装目录，请将 `/opt/simadmin/simadmin` 替换为实际后端二进制路径。

### 构建完整 OTA 包

```bash
./scripts/build.sh
```

常用选项：

```bash
./scripts/build.sh --backend-only
./scripts/build.sh --frontend-only
./scripts/build.sh --no-upx
./scripts/build.sh --no-ota
```

Windows 下建议在 WSL2 Ubuntu 中执行完整 OTA 构建。原生 PowerShell 不能直接运行 Bash 脚本；Git Bash 容易受 Node/npm/pnpm PATH 影响，完整 OTA 仍需要 `aarch64-unknown-linux-musl-gcc` 等 Linux 交叉编译工具链：

```bash
./scripts/build.sh --no-upx
```

构建脚本会：

- 同步 `VERSION` 到 `backend/Cargo.toml` 和 `frontend/package.json`。
- 使用 `pnpm-lock.yaml` 时通过 `pnpm install --frozen-lockfile`、`pnpm run lint` 和 `pnpm exec vite build` 构建前端到 `frontend/dist/`。
- 交叉编译后端到 `backend/target/aarch64-unknown-linux-musl/release/simadmin`。
- 可选使用 UPX 压缩后端二进制；未安装 UPX 时会自动跳过压缩。
- 生成 `release/simadmin_<version>.tar.gz` OTA 包。

### 通过 ADB 部署

```bash
./scripts/deploy.sh
```

常用选项：

```bash
./scripts/deploy.sh --backend-only
./scripts/deploy.sh --frontend-only
./scripts/deploy.sh --no-restart
./scripts/deploy.sh --target=/opt/simadmin
```

### 设备侧一键安装 / 升级

在目标设备上以 root 执行：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/install_latest.sh | sh
```

国内网络环境：

```bash
curl -fsSL https://gh-proxy.com/https://raw.githubusercontent.com/3899/SimAdmin/main/install_latest.sh | sh
```

可选环境变量：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/install_latest.sh \
  | REPO=3899/SimAdmin INSTALL_DIR=/opt/simadmin SERVICE_NAME=simadmin sh
```

脚本会：

- 从 GitHub Release 下载 `simadmin.tar.gz`。
- 安装后端二进制到 `/opt/simadmin/simadmin`。
- 安装前端到 `/opt/simadmin/www`。
- 安装并启用 `simadmin.service`。
- 安装并启用 `simadmin-modem-recovery.service`。
- 配置 NetworkManager 忽略 `wwan*` 接口，避免与 SimAdmin 抢占蜂窝连接管理。

### 设备侧一键卸载

默认彻底卸载，删除服务、程序文件、前端文件、OTA 临时目录、NetworkManager 配置以及用户数据：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/uninstall.sh | sh
```

国内网络环境：

```bash
curl -fsSL https://gh-proxy.com/https://raw.githubusercontent.com/3899/SimAdmin/main/uninstall.sh | sh
```

如需保留短信数据库和配置文件：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/uninstall.sh \
  | sh -s -- --keep-user-data
```

自定义安装路径或服务名时，需要和安装时保持一致：

```bash
curl -fsSL https://raw.githubusercontent.com/3899/SimAdmin/main/uninstall.sh \
  | INSTALL_DIR=/opt/simadmin SERVICE_NAME=simadmin sh -s -- --keep-user-data
```

可选参数：

| 参数 | 说明 |
|------|------|
| `--purge` | 删除全部 SimAdmin 文件和用户数据，默认行为 |
| `--keep-user-data` | 保留 `/opt/simadmin/data.db`、SQLite sidecar 文件和配置文件 |
| `--install-dir PATH` | 指定安装目录，默认 `/opt/simadmin` |
| `--service-name NAME` | 指定主服务名，默认 `simadmin` |

脚本会：

- 停止并禁用 `simadmin.service`。
- 停止并禁用 `simadmin-modem-recovery.service`。
- 删除 systemd 单元文件并执行 `daemon-reload` / `reset-failed`。
- 删除 `/usr/local/bin/simadmin-modem-recovery.sh`。
- 删除 `/etc/NetworkManager/conf.d/99-simadmin-unmanaged-modem.conf`，并在 NetworkManager 运行时重启它。
- 删除 `/tmp/ota_staging`。
- 默认删除 `/opt/simadmin` 和 `/data/config.json`；使用 `--keep-user-data` 时保留用户数据。

## 运行环境

### 目标设备要求

- Linux / Debian 系统。
- systemd。
- root 权限。
- system D-Bus。
- ModemManager 和 `mmcli`。
- 建议安装 `qmicli`，用于基站信息兜底读取。
- `iptables` / `ip6tables`，用于只读网络诊断；SimAdmin 不会自动清空宿主机防火墙规则。
- `tar`，OTA 安装必需。
- `unzip`、`busybox unzip` 或 `python3`，用于安装脚本解压自动下载的 `lpac`；`unzip` 也用于上传 zip 格式 OTA 包。
- eSIM 管理依赖 `lpac`。使用 `install_latest.sh` 安装时会先校验设备上的 `lpac`，缺失、不可用或版本较旧时才按设备架构下载最新匹配版本；普通 SIM 模式下不会调用。

### 安装路径

| 路径 | 说明 |
|------|------|
| `/opt/simadmin/simadmin` | 后端二进制 |
| `/opt/simadmin/www/` | 前端静态文件 |
| `/opt/simadmin/lpac/` | 安装脚本按设备架构下载的私有 `lpac` 运行文件，后端优先使用 |
| `/opt/simadmin/data.db` | SQLite 数据库，保存短信、登录认证配置和 Web 会话等数据 |
| `/opt/simadmin/meta.json` | 当前安装包元数据 |
| `/data/config.json` | 优先使用的持久化配置文件 |
| `/opt/simadmin/config.json` | `/data` 不存在时的配置文件回退路径 |
| `/tmp/ota_staging` | OTA 上传后的临时验证目录 |
| `/etc/systemd/system/simadmin.service` | 主服务 |
| `/etc/systemd/system/simadmin-modem-recovery.service` | 开机 modem 自检恢复服务 |
| `/usr/local/bin/simadmin-modem-recovery.sh` | 开机 modem 自检恢复脚本 |
| `/etc/NetworkManager/conf.d/99-simadmin-unmanaged-modem.conf` | NetworkManager 忽略 `wwan*` 的配置 |

### eSIM 管理

本项目中的 eSIM 指写入了 Profiles 的实体 eUICC SIM 卡，插入设备 SIM 卡槽后仍按普通 SIM 使用。SimAdmin 的“eSIM 模式”只控制 eSIM 管理页面和接口是否开放，不切换设备板载硬件。

普通 SIM 模式下，侧边栏隐藏 eSIM 管理入口，`/api/esim/*` 返回 `403`，前端不会下载 eSIM 页面 chunk，后端也不会主动调用 `lpac`。切换到 eSIM 模式后，只有打开 eSIM 管理页面或执行 Profile 操作时才会按需调用 `lpac chip info`、`lpac profile list`、`lpac profile enable`、`lpac profile nickname` 和 `lpac profile delete`。

OTA 包不内置固定架构的 `lpac`。`install_latest.sh` 会在目标设备上根据 `uname -m` 和 glibc 版本优先选择兼容资产，已安装最新版本时跳过，否则安装到 `/opt/simadmin/lpac/lpac`；兼容资产按 SimAdmin 的 `lpac` release manifest 比对版本，官方资产按 `estkme-group/lpac` latest release 比对版本。后端优先使用该私有路径，找不到时再回退到 PATH 中的 `lpac`。如需跳过自动安装，可设置 `SIMADMIN_INSTALL_LPAC=0`；如需固定版本或使用镜像，可设置 `LPAC_TARGET_VERSION`、`LPAC_ASSET_URL`、`LPAC_ASSET_NAME`、`LPAC_RELEASE_BASE_URL` 或 `LPAC_COMPAT_RELEASE_BASE_URL`。仅手动上传 OTA 包不会自动安装或更新 `lpac`，首次部署建议先执行安装脚本一次。

如果设备已经通过 OTA 更新到支持 eSIM 的版本，但还没有安装 `lpac`，eSIM 管理页面会显示状态提示并提供“安装/修复 lpac”入口。修复过程只在 eSIM 模式下由用户手动触发，会根据设备 `uname -m` 和 glibc 版本优先下载兼容资产，失败后再尝试官方资产，并支持选择 GitHub 代理前缀；页面内修复由后端内置 ZIP 解压完成。

### systemd 服务

主服务单元位于 `scripts/simadmin.service`，默认：

- `WorkingDirectory=/opt/simadmin`
- `ExecStart=/opt/simadmin/simadmin`
- `Restart=always`
- `DBUS_SYSTEM_BUS_ADDRESS=unix:path=/var/run/dbus/system_bus_socket`

查看状态：

```bash
systemctl status simadmin --no-pager
journalctl -u simadmin -f
```

## 核心功能

### Web 管理页面

| 页面 | 路由 | 说明 |
|------|------|------|
| 登录认证 | `/login` | 首次设置管理员密码、登录后台 |
| 仪表盘 | `/` | 在线状态、运营商、信号、网络延迟、数据/漫游/飞行模式快捷开关、系统资源、温度、流量 |
| 设备信息 | `/device` | IMEI、厂商、型号、固件、SIM、系统信息 |
| eSIM 管理 | `/esim` | eSIM 模式下显示，管理插入设备的实体 eUICC SIM 卡 Profiles |
| 蜂窝网络 | `/network` | 网络注册、服务小区和邻区、运营商扫描、APN、射频模式、频段锁定、小区锁定状态 |
| 设备网络 | `/device-network` | WLAN 客户端联网、无线网络扫描和连接、DDNS 动态解析配置和同步日志 |
| 短信管理 | `/sms` | 接收短信、发送短信、短信列表、会话、统计、删除对话、删除短信 |
| 通知中心 | `/notifications` | 多渠道通知配置、短信转发、DDNS转发、版本更新提醒、测试发送 |
| 系统配置 | `/config` | 设备操作、数据连接、漫游、飞行模式、基带重启、服务重启、系统重启等 |
| OTA 更新 | `/ota` | 上传 OTA 包、在线获取 Release、验证、应用或取消更新 |

### 后端能力

- 单管理员密码登录，支持首次设置、会话 Cookie、受保护 API 拦截和 SSH 本机恢复。
- 设备信息、SIM 信息、网络注册信息读取。
- 数据连接开关和漫游策略持久化。
- 飞行模式控制。
- 基带重启流程和进度查询。
- 数据连接 watchdog，每 15 秒检查连接状态、iptables 规则数量和 modem 可用性；检测到宿主机防火墙规则时仅记录诊断日志，不自动清空规则。
- ModemManager 丢失时触发 `mmcli --scan-modems`，连续失败后重启 ModemManager。
- NetworkManager `wwan*` unmanaged 配置。
- 设备侧 WLAN 客户端连接管理，通过 NetworkManager/nmcli 扫描和连接无线局域网，WLAN 在线时优先作为设备默认出口。
- 原生 DDNS 同步，支持腾讯云 DNSPod、阿里云 AliDNS 和 Cloudflare，支持 IPv4/IPv6 独立配置、API/网卡取 IP 和变更/失败事件通知；默认通过网卡取 IP，可切换为内置多接口 API fallback。
- 短信发送、接收监听、SQLite 持久化和多渠道通知转发。
- APN 列表读取和 APN 修改。
- 运营商列表、扫描、手动注册、自动注册。
- eSIM 模式下按需调用 `lpac` 管理实体 eUICC SIM 卡 Profiles；普通 SIM 模式下不调用 eSIM 能力。
- 安装脚本按设备架构自动准备私有 `lpac`；OTA 包本身不绑定 `lpac` 架构或版本。
- OTA 上传、在线下载、校验、替换二进制和前端资源。

## 🚀 版本更新记录

### 📌 v1.0.8

#### ✨ 新增功能

- 短信管理加入常驻搜索框，支持按号码、短信内容检索会话；匹配项可自动高亮检索关键词。
- 短信统计新增推送数据统计，直观展示推送成功、尝试推送条数。
- 新增系统配置「安全性」页面，支持启用密码保护、修改管理员密码、密码策略、会话有效期和空闲超时设置。

#### 💫 体验优化

- 精简短信统计面板，隐藏总计数据，保留收发统计，新增独立推送统计卡片，支持悬停查看数据说明。
- 持久化存储短信转发状态，过滤无效异常数据，保证推送统计结果准确可靠。
- 优化登录页和首次设置密码页视觉效果，补充忘记密码提示。
- 支持按已保存的密码策略校验新密码，并提供密码强度提示。

### 📌 v1.0.7

#### ✨ 新增功能

- 新增管理员密码登录能力，支持初始配置、会话校验，防护页面与接口访问访问保护。
- 支持 SSH 本地命令交互式重置密码、清空认证配置后重新设置。
- 增加密码复杂度校验，过滤非法格式字符。

#### 💫 体验优化

- 优化本机号码变量展示，国内号码精简为 11 位格式，境外号码保留或补全国际前缀。
- 版本检测改为北京时间每天 09:00 和 18:00 执行，并新增超时限制、多节点降级重试，提升更新通知获取成功率。

### 📌 v1.0.6

#### ✨ 新增功能

- 数据连接管控从 ModemManager D-Bus 迁移至 NetworkManager，通过nmcli实现联网启停。
- 交由 NetworkManager 自动完成接口启用、IP、DNS、IPv6 及路由全流程配置。
- 新增双轨流量统计机制，精准获取蜂窝网口真实上下行流量。
- 新增统一网络地址选择能力，仪表盘与 DDNS 共用同一套 IPv4 / IPv6 网口判断逻辑。
- 在未自动识别到 SIM 卡的本机号码或短信中心（SMSC）时，允许用户在界面手动输入并持久化缓存。
- 手动输入的本机号码立即生效，自动支持并映射通知中心的短信转发变量。

#### 💫 体验优化

- 职责拆分，ModemManager 仅负责信号、注册状态等只读查询。
- 程序启动自动清理旧 `unmanaged` 配置，自动生成蜂窝网络配置。
- 移除 `wwan0` 等网口名称硬编码，由 `NetworkManager` 自动适配识别。
- APN 配置界面默认选中并展示当前已连接的 APN 槽位。
- 优化 DDNS 公网 IP 获取逻辑，非蜂窝网卡跳过冗余 D-Bus 请求，减少资源开销。
- 完善蜂窝接口流量展示，支持展示真实上下行流量字节与数据包数量。
- 优化仪表盘与 DDNS 的 IP 展示和获取优先级，有无线网络时优先使用无线网口，无无线网络时自动回落到蜂窝数据网口。
- 自动识别无线网口、蜂窝网口和默认路由，不再依赖 `wlan0`、`wwan0` 等固定网口名称。
- 在仪表盘和设备信息页增加 SIM 号码空值识别，无缝集成在线编辑组件及输入格式校验。

#### 🐞 bug 修复

- 彻底修复数据连接假开关问题，解决前台显示已联网、实际接口未启用无 IP 无法上网的故障。
- 修复正常网口因内核限制被误判为 `UNKNOWN` 的问题，优化状态自动纠正逻辑。
- 彻底解决高通平台蜂窝网口流量统计恒为 0 问题，同时兼容联发科、展锐等多类模组设备。
- 解决内核网口统计数据不准，造成前端无法展示真实蜂窝流量的问题。
- 修复仪表盘和 DDNS 在部分场景下误选本地回环、管理桥或链路本地地址，导致显示或解析 IP 不符合实际上网出口的问题。

### 📌 v1.0.5

#### ✨ 新增功能

- 新增短信中心号码、本机号码多源读取 + 缓存能力，多通道静默提取、命中即止。
- 新增缓存表，接收短信时自动学习并持久化存储短信中心号码、本机号码。
- 新增 SIM / eSIM 工作模式功能开关，默认保持普通 SIM 模式。
- 依托 `lpac` 实现实体 eUICC 卡轻量 eSIM 管理，配套管理页面支持卡信息查看、配置文件切换、重命名与删除。
- 通知模板新增本机号码变量，适配短信转发场景。

#### 💫 体验优化

- 短信中心号码、本机号码读取失败均静默处理，不影响功能使用与页面展示。
- 优化短信接收可靠性，服务启动、Profile 切换和基带恢复后会主动检查未同步短信。
- 优化短信去重策略，减少设备重启或短信编号复用导致的误判漏收。
- 采用命中即停策略，减少冗余调制解调器指令调用，降低日志干扰。
- 优化系统配置页面 UI，调整布局并新增工作模式卡片。
- 侧边栏随工作模式动态隐藏 eSIM 相关入口，普通 SIM 模式不加载对应页面资源。
- 优化 eSIM 调用逻辑，仅使用对应功能时调用 `lpac`，无需常驻后台进程。
- 优化设备开机及 SIM 搜网注册阶段前端界面频繁报错的问题，同时将用户操作触发的错误提示改为常显，解决此前错误信息自动消失过快导致用户无法阅读的问题。

#### 🐞 bug 修复

- 修复 PushPlus 部分配置场景报错问题，提升通知渠道兼容性。
- 修复短信中心号码读取时，ModemManager 非调试模式的未授权告警问题。
- 修复实体 eSIM 卡在 Profile 切换或基带恢复后，短信已到设备但 Web 页面未显示的问题。

#### 📚 接口与文档更新

- 数据库新增号码缓存表，持久化存储 SMSC 与本机号码。
- 新增工作模式、eSIM 管理全套 API 接口。
- eSIM 接口适配 `lpac` 标准 JSON 输出，适配各类 eSIM 管理操作。
- 完善文档，补充相关使用说明。

### 📌 v1.0.4

#### ✨ 新增功能

- 通知中心新增 PushPlus 通知渠道，支持配置 Token、标题模板、群组编码、消息模板、发送渠道、渠道参数和回调地址，并接入短信、DDNS 事件转发及测试发送。
- 通知中心新增版本更新提醒：后台每天北京时间 09:00 和 18:00 检测，发现新版本时按渠道首次提醒一次，不对检测失败或无更新场景推送通知。

#### 💫 体验优化

- 抽象封装前端通用 IPv6 公网地址筛选工具，仪表盘连接展示、设备网络 DDNS 接口统一复用同一套判定规则。
- 数据连接看门狗保留蜂窝状态巡检与 Modem 自愈能力，不再自动清空宿主机 iptables/ip6tables 规则，避免影响 Docker、VPN 及容器端口映射等业务。
- 切换蜂窝数据连接时取消全局防火墙规则刷新，减少对宿主机网络栈、容器转发链路的副作用。
- 通知中心统一在模板渲染层将短信、DDNS 的时间变量格式化为北京时间，避免各渠道收到 UTC 原始时间。

#### 🐞 bug 修复

- 修正 DDNS 模块 IPv6 公网地址判断逻辑，前后端统一以公网域 IPv6 作为 AAAA 记录候选地址，避免误选链路本地地址、内网 ULA 地址。
- IPv6 地址候选按 `/128` 优先排序，提升多 IPv6 地址场景下 DDNS 自动选择地址的准确性。
- 修复宿主机 `FORWARD` 默认策略为 `DROP` 时，清空 filter 表规则引发 Docker 网桥转发失效、外部无法访问容器映射端口的问题。
- 修复企业微信应用消息可能报错的问题。

#### 📚 接口与文档更新

- 明确 NetworkManager `wwan*` 非托管配置仅用于防止蜂窝接口被双重管理，不会直接修改 iptables 规则。
- 同步更新 README 及 Bruno 接口示例，标注 `iptables/ip6tables` 仅作只读诊断使用，程序不会自动清空宿主机防火墙规则。

### 📌 v1.0.3

#### ✨ 新增功能

- 新增“设备网络”模块，用于配置设备侧 WLAN 客户端联网、 DDNS 动态解析，以及后期其他高级功能的扩充。
- 新增设备侧 WLAN 管理能力：
  - 开启 / 关闭 WLAN。
  - 扫描附近无线网络。
  - 连接、断开和忘记已保存 WLAN。
  - 保存自动加入和 IPv4 地址配置。
- 新增原生 DDNS 同步能力：
  - 支持腾讯云 DNSPod、阿里云 AliDNS 和 Cloudflare。
  - 支持 IPv4 / IPv6 独立解析配置。
  - 支持通过网卡或多接口 API 获取公网 IP。
  - 支持立即同步、后台周期同步、同步状态和最近 50 条运行日志。
- DDNS 变更和失败事件接入通知中心，可按通知渠道独立开启“DDNS 变更”转发。

#### 💫 体验优化

- 侧边栏和页面分为“蜂窝网络”和“设备网络”，职责更清晰。
- OTA 更新页面优化待安装更新和验证结果展示，在线更新支持 GitHub Release 检查、下载加速节点和下载进度。
- 仪表盘温度展示改为冷到热连续热力色，并同时标识最低和最高温度探头。

#### 🐞 bug 修复

- 运营商扫描和手动 / 自动注册增加超时处理，避免 ModemManager D-Bus 调用长时间阻塞。
- 数据连接 watchdog 增强长时间 `searching` 状态恢复逻辑，可自动尝试重新注册或循环射频状态。
- 注册遇到 QMI network selection internal 类错误时统一走射频恢复流程，减少注册异常后的卡死概率。

#### 📚 接口与文档更新

- 新增设备网络 API 文档和 Bruno 示例，覆盖 DDNS 配置、状态、同步、日志以及 WLAN 状态、扫描、连接、断开、忘记和配置保存。
- API 契约新增 DDNS、WLAN 和 DDNS 同步日志相关类型。
- 配置持久化新增 `device_network.ddns`，通知渠道配置新增 `forward_ddns`。

### 📌 v1.0.2

#### ✨ 新增功能

- 频段锁定状态新增后端报告的真实支持频段：
  - `supported_lte_fdd_bands`
  - `supported_lte_tdd_bands`
  - `supported_nr_fdd_bands`
  - `supported_nr_tdd_bands`
- 小区与信号数据增加 `qmicli` 兜底解析，提升 ModemManager `GetCellInfo` 不可用场景下的可观测性。

#### 💫 体验优化

- 优化移动网络注册问题的诊断路径，重点区分 SIM 归属运营商、当前注册网络、漫游状态、射频模式、频段限制和小区数据。
- 前端自定义频段选项改为基于后端 `SupportedBands` 动态显示，不再依赖固定写死的 LTE/NR 频段列表。
- 从“未锁定”切换到“自定义”时，默认勾选当前设备真实支持的全部频段，更符合使用预期。
- 移除网络页面中不必要的“验证说明”文案，简化频段锁定 UI。

#### 🐞 bug 修复

- SIM 信息优先从 IMSI 推导 MCC/MNC，避免注册到漫游网络时把当前网络运营商误当成 SIM 归属运营商。
- 后端设置自定义频段时会校验用户选择是否被当前 modem 支持；发现不支持的频段会返回明确错误，不再静默忽略。
- 自动注册遇到 QMI Internal 类错误时增强恢复处理，降低调制解调器处于异常注册状态时的卡死概率。

#### 📚 接口与文档更新

- 修正 README、Bruno API 示例和前端开发代理中的默认访问地址为 `http://192.168.68.1:3000/`。

### 📌 v1.0.1

#### ✨ 新增功能

- 新增“通知中心”多渠道通知配置能力。
  - 通知配置从单一 Webhook 扩展为多渠道：
    - Webhook
    - Bark
    - 企业微信应用消息
    - 企业微信群机器人
    - 钉钉群自定义机器人
    - 钉钉企业内机器人
    - 飞书机器人
    - Telegram 机器人
  - 新增通知中心后端接口：
    - `GET /api/notifications/config`
    - `POST /api/notifications/config`
    - `POST /api/notifications/test/{channel}`
  - 新增各通知渠道的测试发送能力，可按渠道发送模拟短信测试。
- 新增短信批量管理能力。
  - 删除短信：
    - 删除单条短信
    - 删除整个短信对话
    - 批量删除短信或多个对话
  - 新增短信删除相关后端接口：
    - `DELETE /api/sms/message/{id}`
    - `DELETE /api/sms/conversation/{phone_number}`
    - `POST /api/sms/batch-delete`
- 新增设备侧一键卸载脚本 `uninstall.sh`，支持彻底卸载和保留用户数据卸载。

#### 💫 体验优化

- Dashboard 仪表盘加载性能优化。
- 通知中心页面重构为左右分栏布局：
  - 左侧展示通知渠道列表和启用统计
  - 右侧展示当前渠道配置表单
- 通知中心支持按渠道独立配置启用状态、短信转发、模板和渠道专属参数。
- 通知模板编辑支持中文变量标签，前端显示中文变量，保存时映射为后端变量。
- 通知渠道测试按钮会先保存当前配置，再执行测试，减少“配置未保存导致测试不生效”的误操作。
- 短信页面优化为更完整的对话管理体验：
  - 对话列表支持删除按钮
  - 聊天消息支持单条删除
  - 支持批量管理，有清晰的选择数量提示
  - 删除这种不可逆的危险操作增加二次确认
- 短信列表加载数量提升，减少历史短信遗漏。

#### 📚 接口与文档更新

- README 中 Webhook 章节改为“通知中心”，补充多渠道通知能力说明。
- Bruno 文档同步更新通知中心接口说明。
- README 新增卸载脚本使用说明、参数说明和清理范围说明。
- API 类型定义新增完整通知中心配置类型，包括各渠道配置结构。

## ModemManager D-Bus 接口

当前实现以 ModemManager 为主。

### 核心接口

| 接口 | 说明 |
|------|------|
| `org.freedesktop.ModemManager1` | ModemManager 根服务 |
| `org.freedesktop.ModemManager1.Modem` | Modem 状态、开关、模式、频段 |
| `org.freedesktop.ModemManager1.Modem.Modem3gpp` | 运营商、注册、扫描 |
| `org.freedesktop.ModemManager1.Modem.Simple` | 简化连接和断开 |
| `org.freedesktop.ModemManager1.Modem.Messaging` | 短信发送和接收 |
| `org.freedesktop.ModemManager1.Sim` | SIM 属性 |
| `org.freedesktop.ModemManager1.Bearer` | 数据连接 bearer |

### 常用调试命令

```bash
# 查看 modem 列表
mmcli -L

# 查看 modem 详情
mmcli -m any

# 查看注册和连接简要状态
mmcli -m any --simple-status

# 查看 3GPP 定位信息
mmcli -m any --location-get

# 查看信号指标
mmcli -m any --signal-get

# 发送 AT 指令
mmcli -m any --command='AT+CGSN'
```

### D-Bus 监控

```bash
# 监听 ModemManager 信号
dbus-monitor --system "sender='org.freedesktop.ModemManager1'"

# 查看 modem 0 暴露的接口
busctl introspect org.freedesktop.ModemManager1 /org/freedesktop/ModemManager1/Modem/0
```

## 频段与小区控制

### 射频模式

`/api/radio-mode` 支持：

| 值 | 说明 |
|----|------|
| `auto` | LTE/NR 自动 |
| `lte` | LTE only |
| `nr` | NR only |

实际是否可切换取决于 modem 的 `SupportedModes`。

### 频段锁定

频段锁定通过 ModemManager 的 `SetCurrentBands` 实现。API 使用用户熟悉的物理频段号，后端内部转换为 ModemManager band id：

- LTE：`30 + Bn`
- NR：`300 + Nn`

示例，锁定 LTE B1 + B3：

```json
{
  "lte_fdd_bands": [1, 3],
  "lte_tdd_bands": [],
  "nr_fdd_bands": [],
  "nr_tdd_bands": []
}
```

示例，锁定 NR N78：

```json
{
  "lte_fdd_bands": [],
  "lte_tdd_bands": [],
  "nr_fdd_bands": [],
  "nr_tdd_bands": [78]
}
```

示例，混合锁定 LTE 和 NR：

```json
{
  "lte_fdd_bands": [1, 3],
  "lte_tdd_bands": [38, 40, 41],
  "nr_fdd_bands": [],
  "nr_tdd_bands": [78, 79]
}
```

解锁所有频段时传空数组：

```json
{
  "lte_fdd_bands": [],
  "lte_tdd_bands": [],
  "nr_fdd_bands": [],
  "nr_tdd_bands": []
}
```

后端会与 modem 的 `SupportedBands` 取交集。如果所选频段不被当前 modem 支持，会返回错误。

### 小区锁定

当前 `/api/cell-lock` 只维护内存状态，支持记录 LTE 和 NR 的 ARFCN/PCI：

```json
{
  "rat": 16,
  "enable": true,
  "pci": 123,
  "arfcn": 627264
}
```

`rat=12` 表示 LTE，`rat=16` 表示 NR。这个接口当前不会下发真实 AT 或 QMI 小区锁定命令。

## API 接口文档

所有主要业务接口返回统一 JSON 包装：

```json
{
  "status": "ok",
  "message": "Success",
  "data": {}
}
```

错误响应：

```json
{
  "status": "error",
  "message": "错误信息"
}
```

除 `/api/health` 和认证初始化接口外，业务接口需要携带登录后由后端设置的 `simadmin_session` Cookie。

### 登录认证

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/auth/status` | GET | 查询是否已设置管理员密码以及当前会话是否已登录 |
| `/api/auth/setup` | POST | 首次设置管理员密码，仅在尚未配置密码时可用 |
| `/api/auth/login` | POST | 使用管理员密码登录，成功后写入 `simadmin_session` Cookie |
| `/api/auth/password` | POST | 已登录后修改管理员密码，并清空旧 Web 会话 |

`/api/auth/setup` 和 `/api/auth/login` 请求体：

```json
{
  "password": "AdminPassword123!"
}
```

`/api/auth/password` 请求体：

```json
{
  "new_password": "NewPassword123!"
}
```

### 基础信息

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/health` | GET | 健康检查，返回平台和版本 |
| `/api/device` | GET | 设备信息 |
| `/api/sim` | GET | SIM 卡信息 |

### 网络状态

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/network` | GET | 网络注册信息、运营商、信号百分比 |
| `/api/cells` | GET | 服务小区和邻区信息 |
| `/api/cell-monitor/start` | POST | 启动小区监控 |
| `/api/cell-monitor/stop` | POST | 停止小区监控 |
| `/api/network/interfaces` | GET | 网络接口、IP、流量统计 |
| `/api/device-network/ddns/config` | GET/POST | DDNS 配置读取和保存 |
| `/api/device-network/ddns/status` | GET | DDNS 当前状态和最近同步结果 |
| `/api/device-network/ddns/sync` | POST | 立即执行 DDNS 同步 |
| `/api/device-network/ddns/logs` | GET | 最近 50 条 DDNS 同步日志 |
| `/api/device-network/ddns/logs/clear` | POST | 清空 DDNS 同步日志 |
| `/api/device-network/wlan/status` | GET | WLAN 设备、开关、连接和 IP 状态 |
| `/api/device-network/wlan/enabled` | POST | 开启或关闭设备 WLAN |
| `/api/device-network/wlan/scan` | POST | 扫描附近 WLAN 热点 |
| `/api/device-network/wlan/profiles` | GET | 获取已保存 WLAN 网络 |
| `/api/device-network/wlan/forget` | POST | 忘记已保存 WLAN 网络 |
| `/api/device-network/wlan/connect` | POST | 连接指定 WLAN 热点 |
| `/api/device-network/wlan/disconnect` | POST | 断开当前 WLAN 连接 |
| `/api/device-network/wlan/profile` | POST | 保存 WLAN 自动加入和 IPv4 配置 |
| `/api/network/signal-strength` | GET | 信号强度 |
| `/api/location/cell-info` | GET | 基站定位参数 |
| `/api/network/operators` | GET | 当前和已知运营商 |
| `/api/network/operators/scan` | GET | 扫描可用运营商，可能耗时较长 |
| `/api/network/register-manual` | POST | 手动注册运营商 |
| `/api/network/register-auto` | POST | 自动注册运营商 |
| `/api/connectivity` | GET | IPv4 / IPv6 连通性检测 |

### 模块控制

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/data` | GET/POST | 数据连接状态和开关 |
| `/api/roaming` | GET/POST | 漫游策略和当前漫游状态 |
| `/api/airplane-mode` | GET/POST | 飞行模式 |
| `/api/radio-mode` | GET/POST | 射频模式 |
| `/api/band-lock` | GET/POST | 频段锁定 |
| `/api/cell-lock` | GET/POST | 小区锁定内存状态 |
| `/api/cell-lock/unlock-all` | POST | 清空小区锁定状态 |
| `/api/apn` | GET/POST | APN 列表和配置 |
| `/api/baseband/restart` | POST | 重启基带并尝试恢复网络 |
| `/api/baseband/restart/status` | GET | 基带重启进度 |
| `/api/work-mode` | GET/POST | 读取或切换普通 SIM / eSIM 工作模式 |

### eSIM 管理

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/esim/lpac/status` | GET | 检测 `lpac` 安装路径、架构匹配和可用状态 |
| `/api/esim/lpac/repair` | POST | 按设备架构下载并安装/修复私有 `lpac`，支持 `proxy_prefix` |
| `/api/esim/euicc` | GET | 读取 eUICC 芯片信息 |
| `/api/esim/profiles` | GET | 读取 Profiles 列表 |
| `/api/esim/profiles/{iccid}/enable` | POST | 启用指定 Profile |
| `/api/esim/profiles/{iccid}/rename` | POST | 重命名指定 Profile |
| `/api/esim/profiles/{iccid}` | DELETE | 删除指定 Profile |

### 短信功能

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/sms/send` | POST | 发送短信 |
| `/api/sms/list` | GET | 短信列表，支持 `limit` / `offset` |
| `/api/sms/conversation` | GET | 指定号码会话，参数 `phone_number` |
| `/api/sms/stats` | GET | 短信统计 |
| `/api/sms/clear` | POST | 清空短信记录 |
| `/api/sms/message/{id}` | DELETE | 删除单条短信 |
| `/api/sms/conversation/{phone_number}` | DELETE | 删除指定号码会话 |
| `/api/sms/batch-delete` | POST | 批量删除短信或多个会话 |

### 系统信息

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/stats` | GET | 网速、内存、磁盘、CPU、运行时间、系统信息、温度 |
| `/api/stats/cpu` | GET | CPU 详细信息 |
| `/api/system/reboot` | POST | 系统重启 |
| `/api/service/restart` | POST | 重启 SimAdmin 服务 |

### 通知中心

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/notifications/config` | GET/POST | 多渠道通知配置 |
| `/api/notifications/test/{channel}` | POST | 测试指定通知渠道 |

通知配置支持：

- Webhook。
- Bark。
- 企业微信应用消息。
- 企业微信群机器人。
- 钉钉群自定义机器人。
- 钉钉企业内机器人。
- 飞书机器人。
- Telegram 机器人。
- `sms_template` 模板。

模板变量包括本机号码、短信号码、内容、方向、状态和时间等字段。版本更新模板支持固件包名、版本号、Commit、构建时间和 OTA 包 MD5；构建时间会按北京时间展示。

### OTA 更新

| 接口 | 方法 | 说明 |
|------|------|------|
| `/api/ota/status` | GET | 当前版本和待安装更新 |
| `/api/ota/upload` | POST | 上传 OTA 包，最大 50 MB |
| `/api/ota/latest-release` | POST | 查询 GitHub latest release |
| `/api/ota/online-prepare` | POST | 在线下载 latest release 中的 OTA 包并验证 |
| `/api/ota/apply` | POST | 应用待安装更新 |
| `/api/ota/cancel` | POST | 取消待安装更新 |

OTA 包结构：

```text
meta.json
simadmin
www/
```

`meta.json` 示例：

```json
{
  "version": "1.0.8",
  "commit": "abcdef0",
  "build_time": "2026-05-06T00:00:00Z",
  "binary_md5": "md5-of-binary",
  "frontend_md5": "md5-of-www",
  "arch": "aarch64-unknown-linux-musl"
}
```

当前验证逻辑会检查：

- OTA 包中存在 `simadmin`。
- OTA 包中存在 `www/`。
- 二进制 MD5 匹配。
- 架构为 `aarch64-unknown-linux-musl`。
- 版本号是否高于当前版本会作为提示字段返回，不是唯一安装条件。

## 开发指南

### D-Bus 操作序列化

会改变 modem 状态的操作应通过 `with_serial` 串行执行，避免 ModemManager 或底层设备出现并发冲突：

```rust
use crate::serial::with_serial;

pub async fn set_some_modem_state(conn: &Connection) -> zbus::Result<()> {
    with_serial(async {
        // D-Bus / modem operation
        Ok(())
    }).await
}
```

### 前后端契约

- 后端模型位于 `backend/src/models.rs`。
- 前端类型位于 `frontend/src/api/contracts.ts`。
- 前端 API 封装位于 `frontend/src/api/current.ts`。
- 路由集中在 `backend/src/main.rs` 和 `frontend/src/App.tsx`。

新增接口时建议同步修改：

1. `backend/src/models.rs`
2. `backend/src/handlers.rs`
3. `backend/src/main.rs`
4. `frontend/src/api/contracts.ts`
5. `frontend/src/api/current.ts`
6. 对应页面或 hook
7. `bruno-api/` 调试请求

### 数据持久化

SQLite 数据库保存：

- `sms_messages`：短信收发记录。
- `auth_config`：管理员密码哈希等登录认证配置。
- `auth_sessions`：Web 会话哈希和过期时间。

管理员密码和会话 token 不以明文存储。修改或清除管理员密码会同步清空旧会话。

配置文件保存：

- 通知中心配置。
- 是否允许漫游。
- 数据连接是否由用户启用。
- 设备网络 DDNS 配置。

### 版本注入

`backend/build.rs` 会在编译期注入：

- `APP_VERSION`
- `GIT_BRANCH`
- `GIT_COMMIT`

其中版本号来自根目录 `VERSION`。

### Bruno API 集合

`bruno-api/` 是 Bruno 调试集合，默认按 `http://IP:3000` 组织请求。导入 Bruno 后可批量验证后端接口。

## 依赖

### 后端

- `axum 0.8`：HTTP 服务。
- `tokio 1.48`：异步运行时。
- `zbus 5`：D-Bus 客户端。
- `rusqlite 0.32`：SQLite。
- `tower-http 0.6`：CORS 和静态文件相关能力。
- `reqwest 0.12`：在线 OTA Release 查询和下载。
- `clap 4.5`：命令行参数。
- `tracing`：日志。

### 前端

- `react 19`
- `vite 7`
- `@mui/material 7`
- `@mui/icons-material`
- `@mui/x-charts`
- `@mui/x-data-grid`
- `@tanstack/react-query`
- `swr`
- `react-router-dom 7`

### 目标设备命令

- `ModemManager`
- `mmcli`
- `NetworkManager` / `nmcli`
- `qmicli`
- `iptables` / `ip6tables`（只读诊断）
- `systemctl`
- `tar`
- `unzip`
- `curl`

##  license 许可证

> GNU General Public License v3.0

## 🎖️ 鸣谢

### 📦 参考项目

- [project-cpe](https://github.com/1orz/project-cpe)
- [SmsForwarder](https://github.com/pppscn/SmsForwarder)
- [ddns-go](https://github.com/jeessy2/ddns-go)
