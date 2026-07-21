<div align="center">
  <br/>

  <div>
    <a href="https://github.com/voorz/SimAdmin">
      <img
        alt="Debian"
        src="https://img.shields.io/badge/Debian-%23D70A53?logo=debian&logoColor=white&style=flat-square"
      />
    </a>
    <a href="./LICENSE">
      <img
        src="https://img.shields.io/github/license/voorz/SimAdmin?style=flat-square"
      />
    </a>
    <a href="https://github.com/voorz/SimAdmin/releases">
      <img
        src="https://img.shields.io/github/v/release/voorz/SimAdmin?style=flat-square"
      />
    </a>
    <a href="https://github.com/voorz/SimAdmin/releases">
      <img
        src="https://img.shields.io/github/downloads/voorz/SimAdmin/total?style=flat-square"
      />
    </a>
  </div>

  <br/>

  <picture>
    <img src="./static/Dashboard_Dark.png" width="100%" alt="Dashboard (Dark)" />
  </picture>

</div>

# SimAdmin

SimAdmin is a web-based management system for Debian-based cellular CPEs, portable Wi-Fi hotspots, and software routers. It provides comprehensive control over SIM/eSIM cards, cellular networks, SMS, DDNS, device status, and OTA updates.

The project consists of a Rust backend and a React frontend:

- **Backend**: Rust + Axum + zbus, communicates with ModemManager via D-Bus, with fallbacks to `mmcli` and direct AT commands.
- **Frontend**: React + Vite + Material UI, providing a dashboard, SIM management, cellular network, SMS, notification center, automation, and OTA update pages.
- **Deployment**: The backend binary serves the frontend SPA in-process. Installed to `/opt/simadmin`, managed by systemd.

## Documentation

- [Installation & Deployment](./docs/install.md) — One-click install/uninstall, default access address, and initial admin password setup.
- [Changelog](./docs/changelog.md) — Detailed version history and update notes.
- [Environment & System Management](./docs/environment.md) — Hardware requirements, dependencies, install paths, eSIM management, systemd service, and data persistence.
- [Developer Guide](./docs/developer.md) — Project structure, frontend/backend development, OTA build, ADB deployment, and D-Bus interface reference.
- [REST API Documentation](./bruno-api/README.md) — REST API route map, request/response schemas, and Bruno API debugging collections.

## Disclaimer

This project directly operates cellular modems, SIM registration, data dialing, APN, frequency bands, airplane mode, NetworkManager, systemd services, system reboots, and OTA file replacement. iptables/ip6tables are used for read-only network diagnostics only and will not automatically clear host firewall rules.

Use only on devices you own and control. Misconfiguration may result in network disconnection, SIM roaming charges, or service failure requiring manual recovery. The user assumes all responsibility for any consequences arising from the use of this project.

Some interfaces are limited by hardware and ModemManager capabilities:

- Band locking depends on ModemManager's `SupportedBands` / `CurrentBands` / `SetCurrentBands`.
- Cell locking is currently an in-memory display only and does not send real hardware lock commands.

## License

This project is licensed under the GNU General Public License v3.0 (GPLv3).

## Features

### Web Management Pages

| Page | Route | Description |
|------|-------|-------------|
| Login | `/login` | Initial admin password setup and login |
| Dashboard | `/` | Online status, carrier, signal, latency, quick toggles, system resources, temperature, traffic, and device info |
| SIM Management | `/sim` | SIM status, identifiers, unlock counters, phone number & SMSC editing, eSIM profile management |
| Cellular Network | `/network` | Network registration, serving/neighbor cells, operator scan, APN, radio mode, band locking |
| Device Network | `/device-network` | WLAN client connectivity, wireless scanning, DDNS configuration and sync logs |
| SMS | `/sms` | Send/receive SMS, conversation view, statistics, delete |
| Notifications | `/notifications` | Forwarding logs, rules, channels, multi-channel test sending |
| Automation | `/automation` | Task scheduling, execution logs with search and cleanup |
| Configuration | `/config` | System settings, data connection, roaming, airplane mode |
| Security | `/config/security` | Admin password, password policy, login protection, session timeout |
| OTA Update | `/ota` | Upload OTA package, fetch releases, verify, apply or cancel |

### Backend Capabilities

- Single-admin authentication with first-time setup, session cookies, protected API interception, and SSH recovery.
- Device info, SIM info, and network registration readout.
- Data connection toggle and roaming policy persistence.
- Airplane mode control.
- Baseband reboot flow with progress tracking.
- Data connection watchdog (15-second interval): checks connection status, iptables rules, and modem availability. Detects host firewall rules without clearing them.
- ModemManager loss recovery via `mmcli --scan-modems`, with ModemManager restart on consecutive failures.
- NetworkManager `wwan*` unmanaged configuration.
- WLAN client management via NetworkManager/nmcli, with WLAN prioritized as default route when online.
- Native DDNS sync supporting Tencent Cloud DNSPod, Alibaba Cloud AliDNS, and Cloudflare, with independent IPv4/IPv6 configuration.
- SMS send/receive, SQLite persistence, and multi-channel notification forwarding.
- Dual-track automation task scheduler: fixed-time (weekly + specific time) and interval-based (minutes/hours/days).
- Automation actions: baseband reboot, safe system reboot (with delay), SMS sending (with random delay, anti-intercept random content, and failure retry).
- Automation event notifications and execution logs with SQLite storage, keyword search, date filtering, and cleanup policies.
- APN list read and modification.
- Operator list, scan, manual/automatic registration.
- eSIM mode: on-demand `lpac` integration for eUICC profile management; inactive in normal SIM mode.
- OTA upload, online download, verification, binary and frontend resource replacement.

## References

- [project-cpe](https://github.com/1orz/project-cpe)
- [SmsForwarder](https://github.com/pppscn/SmsForwarder)
- [ddns-go](https://github.com/jeessy2/ddns-go)
- [lpac](https://github.com/estkme-group/lpac)
