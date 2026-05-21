//! 工具函数模块
//!
//! 包含系统状态、资源统计和网络接口读取等工具函数

use crate::models::{IpAddress, NetworkInterfaceInfo};
use std::net::IpAddr;

/// 从 /proc/meminfo 读取内存信息
///
/// # Returns
/// (total, available, cached, buffers) in bytes
pub fn read_memory_info() -> Result<(u64, u64, u64, u64), String> {
    use std::fs;

    let content = fs::read_to_string("/proc/meminfo")
        .map_err(|e| format!("Failed to read /proc/meminfo: {}", e))?;

    let mut total = 0u64;
    let mut available = 0u64;
    let mut cached = 0u64;
    let mut buffers = 0u64;

    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 2 {
            continue;
        }

        let value = parts[1].parse::<u64>().unwrap_or(0) * 1024; // Convert KB to bytes

        match parts[0] {
            "MemTotal:" => total = value,
            "MemAvailable:" => available = value,
            "Cached:" => cached = value,
            "Buffers:" => buffers = value,
            _ => {}
        }
    }

    Ok((total, available, cached, buffers))
}

/// 读取磁盘/分区使用情况
///
/// 自适应检测分区，去重处理（相同设备的多个挂载点只保留一个）
///
/// # Returns
/// 包含各个分区信息的 Vec<DiskInfo>
#[cfg(unix)]
pub fn read_disk_info() -> Vec<crate::models::DiskInfo> {
    use std::collections::HashMap;
    use std::ffi::CString;
    use std::fs;

    // 读取 /proc/mounts
    let mounts = match fs::read_to_string("/proc/mounts") {
        Ok(content) => content,
        Err(_) => return Vec::new(),
    };

    // 用于设备去重：设备名 -> (挂载点, 文件系统类型, 优先级)
    // 优先级越低越优先显示
    let mut device_map: HashMap<String, (String, String, u8)> = HashMap::new();

    // 挂载点优先级（数字越小优先级越高）
    let get_priority = |mount: &str| -> u8 {
        match mount {
            "/" => 0,
            "/home" => 1,
            "/mnt/userdata" => 2,
            "/var" => 3,
            "/run" => 4,
            "/tmp" => 5,
            _ if mount.starts_with("/mnt/") => 10,
            _ if mount.starts_with("/var/") => 15,
            _ => 20,
        }
    };

    // 跳过的虚拟文件系统和挂载点
    let skip_fs = [
        "proc",
        "sysfs",
        "devtmpfs",
        "devpts",
        "cgroup",
        "cgroup2",
        "pstore",
        "bpf",
        "tracefs",
        "debugfs",
        "securityfs",
        "configfs",
        "fusectl",
        "hugetlbfs",
        "mqueue",
        "rpc_pipefs",
        "autofs",
        "functionfs",
    ];

    let skip_mounts = [
        "/dev",
        "/dev/pts",
        "/sys",
        "/proc",
        "/sys/kernel/config",
        "/dev/usb-ffs/adb",
    ];

    for line in mounts.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() < 3 {
            continue;
        }

        let device = parts[0];
        let mount_point = parts[1];
        let fs_type = parts[2];

        // 跳过虚拟文件系统
        if skip_fs.contains(&fs_type) {
            continue;
        }

        // 跳过特定挂载点
        if skip_mounts.contains(&mount_point) {
            continue;
        }

        let priority = get_priority(mount_point);

        // 设备去重：同一设备保留优先级最高的挂载点
        let key = device.to_string();
        if let Some((_, _, existing_priority)) = device_map.get(&key) {
            if priority >= *existing_priority {
                continue; // 已有更高优先级的挂载点
            }
        }

        device_map.insert(
            key,
            (mount_point.to_string(), fs_type.to_string(), priority),
        );
    }

    // 收集磁盘信息
    let mut disks = Vec::new();

    for (_, (mount_point, fs_type, _)) in device_map {
        let c_path = match CString::new(mount_point.as_str()) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let mut stat: libc::statvfs = unsafe { std::mem::zeroed() };
        let result = unsafe { libc::statvfs(c_path.as_ptr(), &mut stat) };

        if result != 0 {
            continue;
        }

        let block_size = stat.f_frsize as u64;
        let total = stat.f_blocks as u64 * block_size;
        let available = stat.f_bavail as u64 * block_size;
        let free = stat.f_bfree as u64 * block_size;
        let used = total.saturating_sub(free);

        // 跳过太小的分区（< 1MB）
        if total < 1024 * 1024 {
            continue;
        }

        let used_percent = (used as f64 / total as f64) * 100.0;

        disks.push(crate::models::DiskInfo {
            mount_point,
            fs_type,
            total_bytes: total,
            used_bytes: used,
            available_bytes: available,
            used_percent,
        });
    }

    // 按挂载点排序：根目录优先，然后按名称
    disks.sort_by(|a, b| {
        let pa = get_priority(&a.mount_point);
        let pb = get_priority(&b.mount_point);
        if pa != pb {
            pa.cmp(&pb)
        } else {
            a.mount_point.cmp(&b.mount_point)
        }
    });

    disks
}

#[cfg(not(unix))]
pub fn read_disk_info() -> Vec<crate::models::DiskInfo> {
    Vec::new()
}

/// 从 /proc/uptime 读取系统运行时间
///
/// # Returns
/// (uptime_seconds, idle_seconds)
pub fn read_uptime() -> Result<(u64, u64), String> {
    use std::fs;

    let content = fs::read_to_string("/proc/uptime")
        .map_err(|e| format!("Failed to read /proc/uptime: {}", e))?;

    let parts: Vec<&str> = content.trim().split_whitespace().collect();
    if parts.len() < 2 {
        return Err("Invalid /proc/uptime format".to_string());
    }

    let uptime = parts[0]
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse uptime: {}", e))? as u64;

    let idle = parts[1]
        .parse::<f64>()
        .map_err(|e| format!("Failed to parse idle time: {}", e))? as u64;

    Ok((uptime, idle))
}

/// 格式化运行时间为人类可读格式
///
/// # Arguments
/// * `seconds` - 总秒数
///
/// # Returns
/// 格式化的字符串，如 "2天 3小时 45分钟"
pub fn format_uptime(seconds: u64) -> String {
    let days = seconds / 86400;
    let hours = (seconds % 86400) / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    let mut parts = Vec::new();

    if days > 0 {
        parts.push(format!("{}天", days));
    }
    if hours > 0 {
        parts.push(format!("{}小时", hours));
    }
    if minutes > 0 {
        parts.push(format!("{}分钟", minutes));
    }
    if parts.is_empty() || secs > 0 {
        parts.push(format!("{}秒", secs));
    }

    parts.join(" ")
}

/// 读取网络接口的流量统计
///
/// # Arguments
/// * `interface` - 网络接口名称（如 usb0, eth0）
/// * `conn` - 可选的 D-Bus 连接用于蜂窝接口流量补足 fallback
///
/// # Returns
/// (rx_bytes, tx_bytes)
pub async fn read_interface_stats(
    interface: &str,
    conn: Option<&zbus::Connection>,
) -> Result<(u64, u64), String> {
    use std::fs;

    let rx_path = format!("/sys/class/net/{}/statistics/rx_bytes", interface);
    let tx_path = format!("/sys/class/net/{}/statistics/tx_bytes", interface);

    let mut rx_bytes = fs::read_to_string(&rx_path)
        .map_err(|e| format!("Failed to read {}: {}", rx_path, e))?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse rx_bytes: {}", e))?;

    let mut tx_bytes = fs::read_to_string(&tx_path)
        .map_err(|e| format!("Failed to read {}: {}", tx_path, e))?
        .trim()
        .parse::<u64>()
        .map_err(|e| format!("Failed to parse tx_bytes: {}", e))?;

    if let Some(c) = conn {
        if let Ok(Some(mm_stats)) =
            crate::modem_manager::get_bearer_stats_for_interface(c, interface).await
        {
            rx_bytes = std::cmp::max(rx_bytes, mm_stats.rx_bytes);
            tx_bytes = std::cmp::max(tx_bytes, mm_stats.tx_bytes);
        }
    }

    Ok((rx_bytes, tx_bytes))
}

/// 获取所有活跃的网络接口列表
///
/// # Returns
/// 网络接口名称列表（排除 lo）
pub fn get_active_interfaces() -> Result<Vec<String>, String> {
    use std::fs;

    let entries = fs::read_dir("/sys/class/net")
        .map_err(|e| format!("Failed to read /sys/class/net: {}", e))?;

    let mut interfaces = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let name = entry.file_name().to_string_lossy().to_string();

        // 排除回环接口
        if name != "lo" {
            // 检查接口是否 up
            let operstate_path = format!("/sys/class/net/{}/operstate", name);
            if let Ok(state) = fs::read_to_string(&operstate_path) {
                let state = state.trim();
                // 包含 up 和 unknown 状态的接口（unknown 可能是某些虚拟接口）
                if state == "up" || state == "unknown" {
                    interfaces.push(name);
                }
            }
        }
    }

    Ok(interfaces)
}

/// 从 /proc/stat 解析 CPU 时间
/// 返回 (total, idle)
fn parse_cpu_stat() -> Result<(u64, u64), String> {
    use std::fs;

    let stat = fs::read_to_string("/proc/stat")
        .map_err(|e| format!("Failed to read /proc/stat: {}", e))?;

    for line in stat.lines() {
        if line.starts_with("cpu ") {
            let values: Vec<u64> = line
                .split_whitespace()
                .skip(1) // 跳过 "cpu"
                .filter_map(|s| s.parse::<u64>().ok())
                .collect();

            if values.len() >= 4 {
                // user + nice + system + idle + iowait + irq + softirq + steal
                let user = values.first().copied().unwrap_or(0);
                let nice = values.get(1).copied().unwrap_or(0);
                let system = values.get(2).copied().unwrap_or(0);
                let idle = values.get(3).copied().unwrap_or(0);
                let iowait = values.get(4).copied().unwrap_or(0);
                let irq = values.get(5).copied().unwrap_or(0);
                let softirq = values.get(6).copied().unwrap_or(0);
                let steal = values.get(7).copied().unwrap_or(0);

                let total = user + nice + system + idle + iowait + irq + softirq + steal;
                let idle_total = idle + iowait;

                return Ok((total, idle_total));
            }
        }
    }

    Err("Failed to parse /proc/stat".to_string())
}

/// 从 /proc/loadavg 读取负载信息，CPU 使用率需要异步采样
///
/// # Returns
/// CpuLoadInfo 结构（不含实时 CPU 使用率）
pub fn read_cpu_load_sync() -> Result<crate::models::CpuLoadInfo, String> {
    use crate::models::CpuLoadInfo;
    use std::fs;

    // 读取 /proc/loadavg 获取负载平均值
    let loadavg = fs::read_to_string("/proc/loadavg")
        .map_err(|e| format!("Failed to read /proc/loadavg: {}", e))?;

    let parts: Vec<&str> = loadavg.split_whitespace().collect();
    let load_1min = parts
        .first()
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let load_5min = parts
        .get(1)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);
    let load_15min = parts
        .get(2)
        .and_then(|s| s.parse::<f64>().ok())
        .unwrap_or(0.0);

    // 获取 CPU 核心数
    let core_count = std::thread::available_parallelism()
        .map(|p| p.get() as u32)
        .unwrap_or(1);

    Ok(CpuLoadInfo {
        load_1min,
        load_5min,
        load_15min,
        core_count,
        load_percent: 0.0, // 需要异步采样
    })
}

/// 异步采样 CPU 使用率（需要两次采样计算差值）
///
/// # Returns
/// CPU 使用率百分比 (0.0 - 100.0)
pub async fn sample_cpu_usage() -> Result<f64, String> {
    use tokio::time::{sleep, Duration};

    // 第一次采样
    let (total1, idle1) = parse_cpu_stat()?;

    // 等待 200ms
    sleep(Duration::from_millis(200)).await;

    // 第二次采样
    let (total2, idle2) = parse_cpu_stat()?;

    // 计算差值
    let total_diff = total2.saturating_sub(total1);
    let idle_diff = idle2.saturating_sub(idle1);

    if total_diff == 0 {
        return Ok(0.0);
    }

    // 计算 CPU 使用率
    let usage = ((total_diff - idle_diff) as f64 / total_diff as f64) * 100.0;

    Ok(usage.clamp(0.0, 100.0))
}

/// 从 /proc/cpuinfo 读取 CPU 信息
///
/// # Returns
/// CpuInfo 结构
pub fn read_cpu_info() -> Result<crate::models::CpuInfo, String> {
    use crate::models::{CpuCore, CpuInfo};
    use std::fs;

    let content = fs::read_to_string("/proc/cpuinfo")
        .map_err(|e| format!("Failed to read /proc/cpuinfo: {}", e))?;

    let mut cores = Vec::new();
    let mut current_core = CpuCore::default();
    let mut hardware = String::new();
    let mut serial = String::new();

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() {
            // 空行表示一个 processor 块结束
            if current_core.processor > 0 || !current_core.bogomips.is_empty() {
                cores.push(current_core.clone());
                current_core = CpuCore::default();
            }
            continue;
        }

        if let Some((key, value)) = line.split_once(':') {
            let key = key.trim();
            let value = value.trim();

            match key {
                "processor" => {
                    if let Ok(num) = value.parse::<u32>() {
                        current_core.processor = num;
                    }
                }
                "BogoMIPS" => {
                    current_core.bogomips = value.to_string();
                }
                "Features" => {
                    current_core.features =
                        value.split_whitespace().map(|s| s.to_string()).collect();
                }
                "CPU implementer" => {
                    current_core.implementer = value.to_string();
                }
                "CPU architecture" => {
                    current_core.architecture = value.to_string();
                }
                "CPU variant" => {
                    current_core.variant = value.to_string();
                }
                "CPU part" => {
                    current_core.part = value.to_string();
                }
                "CPU revision" => {
                    current_core.revision = value.to_string();
                }
                "Hardware" => {
                    hardware = value.to_string();
                }
                "Serial" => {
                    serial = value.to_string();
                }
                _ => {}
            }
        }
    }

    // 处理最后一个核心（如果文件不以空行结尾）
    if current_core.processor > 0 || !current_core.bogomips.is_empty() {
        cores.push(current_core);
    }

    // 识别 CPU 型号
    let model_name = if !cores.is_empty() {
        identify_cpu_model(&cores[0].implementer, &cores[0].part)
    } else {
        "Unknown".to_string()
    };

    Ok(CpuInfo {
        core_count: cores.len() as u32,
        cores,
        hardware,
        serial,
        model_name,
    })
}

/// 从 uname 系统调用读取系统信息
///
/// # Returns
/// SystemInfo 结构
#[cfg(unix)]
pub fn read_system_info() -> Result<crate::models::SystemInfo, String> {
    use crate::models::SystemInfo;
    use std::ffi::CStr;

    unsafe {
        let mut utsname: libc::utsname = std::mem::zeroed();

        if libc::uname(&mut utsname) != 0 {
            return Err("Failed to call uname system call".to_string());
        }

        // 将 C 字符串转换为 Rust String
        let sysname = CStr::from_ptr(utsname.sysname.as_ptr())
            .to_string_lossy()
            .to_string();

        let nodename = CStr::from_ptr(utsname.nodename.as_ptr())
            .to_string_lossy()
            .to_string();

        let release = CStr::from_ptr(utsname.release.as_ptr())
            .to_string_lossy()
            .to_string();

        let version = CStr::from_ptr(utsname.version.as_ptr())
            .to_string_lossy()
            .to_string();

        let machine = CStr::from_ptr(utsname.machine.as_ptr())
            .to_string_lossy()
            .to_string();

        // 注意：domainname 字段在某些平台上不可用，这里留空
        let domainname = String::new();

        // 构造类似 uname -a 的完整输出
        let full_info = format!(
            "{} {} {} {} {}",
            sysname, nodename, release, version, machine
        );

        Ok(SystemInfo {
            sysname,
            nodename,
            release,
            version,
            machine,
            domainname,
            full_info,
        })
    }
}

#[cfg(not(unix))]
pub fn read_system_info() -> Result<crate::models::SystemInfo, String> {
    use crate::models::SystemInfo;

    let sysname = std::env::consts::OS.to_string();
    let machine = std::env::consts::ARCH.to_string();
    let nodename = std::env::var("COMPUTERNAME").unwrap_or_else(|_| "unknown".to_string());
    let release = String::new();
    let version = String::new();
    let domainname = String::new();
    let full_info = format!("{} {} {}", sysname, nodename, machine)
        .trim()
        .to_string();

    Ok(SystemInfo {
        sysname,
        nodename,
        release,
        version,
        machine,
        domainname,
        full_info,
    })
}

/// 根据 implementer 和 part 识别 CPU 型号
///
/// # Arguments
/// * `implementer` - CPU 实现者 ID（如 0x41 表示 ARM）
/// * `part` - CPU 部件号（如 0xd05 表示 Cortex-A55）
///
/// # Returns
/// CPU 型号名称
fn identify_cpu_model(implementer: &str, part: &str) -> String {
    // ARM implementer (0x41)
    if implementer == "0x41" {
        return match part {
            "0xd05" => "ARM Cortex-A55".to_string(),
            "0xd0a" => "ARM Cortex-A75".to_string(),
            "0xd0b" => "ARM Cortex-A76".to_string(),
            "0xd0c" => "ARM Neoverse N1".to_string(),
            "0xd0d" => "ARM Cortex-A77".to_string(),
            "0xd0e" => "ARM Cortex-A76AE".to_string(),
            "0xd40" => "ARM Neoverse V1".to_string(),
            "0xd41" => "ARM Cortex-A78".to_string(),
            "0xd44" => "ARM Cortex-X1".to_string(),
            "0xd46" => "ARM Cortex-A510".to_string(),
            "0xd47" => "ARM Cortex-A710".to_string(),
            "0xd48" => "ARM Cortex-X2".to_string(),
            "0xd49" => "ARM Neoverse N2".to_string(),
            "0xd4a" => "ARM Neoverse E1".to_string(),
            "0xd4b" => "ARM Cortex-A78AE".to_string(),
            "0xd4c" => "ARM Cortex-X1C".to_string(),
            "0xd4d" => "ARM Cortex-A715".to_string(),
            "0xd4e" => "ARM Cortex-X3".to_string(),
            _ => format!("ARM CPU (part: {})", part),
        };
    }

    format!("CPU (implementer: {}, part: {})", implementer, part)
}

/// 判断IP地址范围（公网/内网/回环/链路本地）
#[cfg_attr(not(unix), allow(dead_code))]
fn get_ip_scope(ip: &IpAddr) -> String {
    match ip {
        IpAddr::V4(ipv4) => {
            let octets = ipv4.octets();
            if ipv4.is_loopback() {
                "loopback".to_string()
            } else if ipv4.is_private()
                || (octets[0] == 10)
                || (octets[0] == 172 && octets[1] >= 16 && octets[1] <= 31)
                || (octets[0] == 192 && octets[1] == 168)
            {
                "private".to_string()
            } else if ipv4.is_link_local() || (octets[0] == 169 && octets[1] == 254) {
                "link-local".to_string()
            } else {
                "public".to_string()
            }
        }
        IpAddr::V6(ipv6) => {
            if ipv6.is_loopback() {
                "loopback".to_string()
            } else if ipv6.is_unicast_link_local() {
                "link-local".to_string()
            } else if ipv6.segments()[0] & 0xfe00 == 0xfc00 {
                // fc00::/7 - Unique Local Address (ULA)
                "private".to_string()
            } else if ipv6.segments()[0] & 0xff00 == 0xfe00 {
                // fe80::/10 - Link-Local
                "link-local".to_string()
            } else {
                "public".to_string()
            }
        }
    }
}

/// 读取网络接口的IP地址信息
#[cfg(unix)]
fn read_interface_ip_addresses(
    interface: &str,
    allow_command_fallback: bool,
) -> Result<Vec<IpAddress>, String> {
    let mut errors = Vec::new();

    match read_interface_ip_addresses_getifaddrs(interface) {
        Ok(addresses) if !addresses.is_empty() => return Ok(addresses),
        Ok(addresses) if !allow_command_fallback => return Ok(addresses),
        Ok(_) => {}
        Err(err) => errors.push(err),
    }

    match read_interface_ip_addresses_with_ip(interface) {
        Ok(addresses) if !addresses.is_empty() => return Ok(addresses),
        Ok(_) => {}
        Err(err) => errors.push(err),
    }

    match read_interface_ip_addresses_with_ifconfig(interface) {
        Ok(addresses) if !addresses.is_empty() => return Ok(addresses),
        Ok(addresses) => Ok(addresses),
        Err(err) => {
            errors.push(err);
            Err(errors.join("; "))
        }
    }
}

#[cfg(unix)]
fn read_interface_ip_addresses_getifaddrs(interface: &str) -> Result<Vec<IpAddress>, String> {
    let mut ifaddrs: *mut libc::ifaddrs = std::ptr::null_mut();

    // Read addresses directly from libc so minimal systems do not need iproute2.
    if unsafe { libc::getifaddrs(&mut ifaddrs) } != 0 {
        return Err(format!(
            "Failed to get interface addresses: {}",
            std::io::Error::last_os_error()
        ));
    }

    let addresses = collect_interface_ip_addresses(interface, ifaddrs);
    unsafe { libc::freeifaddrs(ifaddrs) };

    Ok(addresses)
}

#[cfg(unix)]
fn collect_interface_ip_addresses(interface: &str, ifaddrs: *mut libc::ifaddrs) -> Vec<IpAddress> {
    use std::ffi::CStr;
    use std::net::{Ipv4Addr, Ipv6Addr};

    let mut addresses = Vec::new();
    let mut current = ifaddrs;

    while !current.is_null() {
        let ifaddr = unsafe { &*current };

        if !ifaddr.ifa_name.is_null() && !ifaddr.ifa_addr.is_null() {
            let name = unsafe { CStr::from_ptr(ifaddr.ifa_name) }.to_string_lossy();
            if name == interface {
                let family = unsafe { (*ifaddr.ifa_addr).sa_family as i32 };

                match family {
                    libc::AF_INET => {
                        let addr = unsafe { &*(ifaddr.ifa_addr as *const libc::sockaddr_in) };
                        let ip = IpAddr::V4(Ipv4Addr::from(addr.sin_addr.s_addr.to_ne_bytes()));
                        let prefix_len = ipv4_netmask_prefix_len(ifaddr.ifa_netmask);

                        addresses.push(IpAddress {
                            address: ip.to_string(),
                            prefix_len,
                            ip_type: "ipv4".to_string(),
                            scope: get_ip_scope(&ip),
                        });
                    }
                    libc::AF_INET6 => {
                        let addr = unsafe { &*(ifaddr.ifa_addr as *const libc::sockaddr_in6) };
                        let ip = IpAddr::V6(Ipv6Addr::from(addr.sin6_addr.s6_addr));
                        let prefix_len = ipv6_netmask_prefix_len(ifaddr.ifa_netmask);

                        addresses.push(IpAddress {
                            address: ip.to_string(),
                            prefix_len,
                            ip_type: "ipv6".to_string(),
                            scope: get_ip_scope(&ip),
                        });
                    }
                    _ => {}
                }
            }
        }

        current = ifaddr.ifa_next;
    }

    addresses
}

#[cfg(unix)]
fn read_interface_ip_addresses_with_ip(interface: &str) -> Result<Vec<IpAddress>, String> {
    for command in ["ip", "/sbin/ip", "/usr/sbin/ip"] {
        match std::process::Command::new(command)
            .args(["-o", "addr", "show", "dev", interface])
            .output()
        {
            Ok(output) if output.status.success() => {
                return Ok(parse_ip_addr_output(&String::from_utf8_lossy(
                    &output.stdout,
                )));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.trim().is_empty() {
                    return Err(format!("{} failed: {}", command, stderr.trim()));
                }
            }
            Err(_) => {}
        }
    }

    Err("ip command not found or failed".to_string())
}

#[cfg(unix)]
fn parse_ip_addr_output(output: &str) -> Vec<IpAddress> {
    output
        .lines()
        .filter_map(|line| {
            let parts: Vec<&str> = line.split_whitespace().collect();
            let family_index = parts
                .iter()
                .position(|part| *part == "inet" || *part == "inet6")?;
            let family = *parts.get(family_index)?;
            let addr_with_prefix = *parts.get(family_index + 1)?;
            let (addr, prefix_len) = parse_address_with_prefix(addr_with_prefix)?;
            let ip = addr.parse::<IpAddr>().ok()?;

            Some(IpAddress {
                address: addr.to_string(),
                prefix_len,
                ip_type: if family == "inet" { "ipv4" } else { "ipv6" }.to_string(),
                scope: get_ip_scope(&ip),
            })
        })
        .collect()
}

#[cfg(unix)]
fn read_interface_ip_addresses_with_ifconfig(interface: &str) -> Result<Vec<IpAddress>, String> {
    for command in ["ifconfig", "/sbin/ifconfig", "/usr/sbin/ifconfig"] {
        match std::process::Command::new(command).arg(interface).output() {
            Ok(output) if output.status.success() => {
                return Ok(parse_ifconfig_output(&String::from_utf8_lossy(
                    &output.stdout,
                )));
            }
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if !stderr.trim().is_empty() {
                    return Err(format!("{} failed: {}", command, stderr.trim()));
                }
            }
            Err(_) => {}
        }
    }

    Err("ifconfig command not found or failed".to_string())
}

#[cfg(unix)]
fn parse_ifconfig_output(output: &str) -> Vec<IpAddress> {
    output
        .lines()
        .flat_map(|line| {
            let normalized = line
                .replace("addr:", "addr ")
                .replace("Mask:", "netmask ")
                .replace("prefixlen ", "prefixlen ");
            let parts: Vec<&str> = normalized.split_whitespace().collect();

            if parts.first() == Some(&"inet") {
                parse_ifconfig_ipv4_line(&parts).into_iter().collect()
            } else if parts.first() == Some(&"inet6") {
                parse_ifconfig_ipv6_line(&parts).into_iter().collect()
            } else {
                Vec::new()
            }
        })
        .collect()
}

#[cfg(unix)]
fn parse_ifconfig_ipv4_line(parts: &[&str]) -> Option<IpAddress> {
    let addr = if parts.get(1) == Some(&"addr") {
        *parts.get(2)?
    } else {
        *parts.get(1)?
    };

    let ip = addr.parse::<IpAddr>().ok()?;
    let prefix_len = parts
        .iter()
        .position(|part| *part == "netmask")
        .and_then(|index| parts.get(index + 1))
        .and_then(|mask| ipv4_netmask_to_prefix_len(mask))
        .unwrap_or(0);

    Some(IpAddress {
        address: addr.to_string(),
        prefix_len,
        ip_type: "ipv4".to_string(),
        scope: get_ip_scope(&ip),
    })
}

#[cfg(unix)]
fn parse_ifconfig_ipv6_line(parts: &[&str]) -> Option<IpAddress> {
    let addr = if parts.get(1) == Some(&"addr") {
        *parts.get(2)?
    } else {
        *parts.get(1)?
    };

    let (addr, prefix_len) = parse_address_with_prefix(addr).unwrap_or_else(|| {
        let prefix_len = parts
            .iter()
            .position(|part| *part == "prefixlen")
            .and_then(|index| parts.get(index + 1))
            .and_then(|prefix| prefix.parse::<u8>().ok())
            .unwrap_or(0);
        (addr, prefix_len)
    });
    let ip = addr.parse::<IpAddr>().ok()?;

    Some(IpAddress {
        address: addr.to_string(),
        prefix_len,
        ip_type: "ipv6".to_string(),
        scope: get_ip_scope(&ip),
    })
}

#[cfg(unix)]
fn parse_address_with_prefix(addr_with_prefix: &str) -> Option<(&str, u8)> {
    let (addr, prefix_len) = addr_with_prefix.split_once('/')?;
    Some((addr, prefix_len.parse::<u8>().ok()?))
}

#[cfg(unix)]
fn ipv4_netmask_to_prefix_len(netmask: &str) -> Option<u8> {
    let octets = netmask.parse::<std::net::Ipv4Addr>().ok()?.octets();

    Some(octets.iter().map(|octet| octet.count_ones() as u8).sum())
}

#[cfg(unix)]
fn ipv4_netmask_prefix_len(netmask: *mut libc::sockaddr) -> u8 {
    if netmask.is_null() {
        return 0;
    }

    let mask = unsafe { &*(netmask as *const libc::sockaddr_in) };
    mask.sin_addr
        .s_addr
        .to_ne_bytes()
        .iter()
        .map(|b| b.count_ones() as u8)
        .sum()
}

#[cfg(unix)]
fn ipv6_netmask_prefix_len(netmask: *mut libc::sockaddr) -> u8 {
    if netmask.is_null() {
        return 0;
    }

    let mask = unsafe { &*(netmask as *const libc::sockaddr_in6) };
    mask.sin6_addr
        .s6_addr
        .iter()
        .map(|b| b.count_ones() as u8)
        .sum()
}

#[cfg(not(unix))]
fn read_interface_ip_addresses(
    _interface: &str,
    _allow_command_fallback: bool,
) -> Result<Vec<IpAddress>, String> {
    Ok(Vec::new())
}

/// 读取所有网络接口信息
pub async fn read_network_interfaces(
    conn: Option<&zbus::Connection>,
) -> Result<Vec<NetworkInterfaceInfo>, String> {
    use std::fs;
    use std::path::Path;

    let sys_class_net = Path::new("/sys/class/net");

    if !sys_class_net.exists() {
        return Err("Network interface directory not found".to_string());
    }

    let mut interfaces = Vec::new();
    let bearer_stats_by_interface = if let Some(c) = conn {
        crate::modem_manager::get_bearer_stats_by_interface(c)
            .await
            .unwrap_or_default()
    } else {
        std::collections::HashMap::new()
    };

    // 遍历所有网络接口
    let entries = fs::read_dir(sys_class_net)
        .map_err(|e| format!("Failed to read network interfaces: {}", e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
        let interface_name = entry.file_name().to_string_lossy().to_string();
        let interface_path = entry.path();

        // 读取接口状态
        let mut status = fs::read_to_string(interface_path.join("operstate"))
            .unwrap_or_else(|_| "unknown".to_string())
            .trim()
            .to_lowercase();

        // 读取MAC地址
        let mac_address = fs::read_to_string(interface_path.join("address"))
            .ok()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty() && s != "00:00:00:00:00:00");

        // 读取MTU
        let mtu = fs::read_to_string(interface_path.join("mtu"))
            .ok()
            .and_then(|s| s.trim().parse::<u32>().ok())
            .unwrap_or(0);

        // 读取统计信息
        let stats_path = interface_path.join("statistics");
        let mut rx_bytes = fs::read_to_string(stats_path.join("rx_bytes"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let mut tx_bytes = fs::read_to_string(stats_path.join("tx_bytes"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        let bearer_stats = bearer_stats_by_interface.get(&interface_name).copied();
        if let Some(mm_stats) = bearer_stats {
            rx_bytes = std::cmp::max(rx_bytes, mm_stats.rx_bytes);
            tx_bytes = std::cmp::max(tx_bytes, mm_stats.tx_bytes);
        }

        let mut rx_packets = fs::read_to_string(stats_path.join("rx_packets"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let mut tx_packets = fs::read_to_string(stats_path.join("tx_packets"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        if let Some(mm_stats) = bearer_stats {
            rx_packets = std::cmp::max(rx_packets, mm_stats.rx_packets);
            tx_packets = std::cmp::max(tx_packets, mm_stats.tx_packets);
        }

        let rx_errors = fs::read_to_string(stats_path.join("rx_errors"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);
        let tx_errors = fs::read_to_string(stats_path.join("tx_errors"))
            .ok()
            .and_then(|s| s.trim().parse::<u64>().ok())
            .unwrap_or(0);

        // 读取IP地址信息
        let ip_addresses =
            read_interface_ip_addresses(&interface_name, status != "down").unwrap_or_default();

        // 如果操作状态为 unknown，检查 flags 和 carrier/IP 来判定是否实际处于 up 状态
        if status == "unknown" {
            let flags = fs::read_to_string(interface_path.join("flags"))
                .ok()
                .and_then(|s| {
                    let s = s.trim();
                    if s.starts_with("0x") {
                        u32::from_str_radix(&s[2..], 16).ok()
                    } else {
                        s.parse::<u32>().ok()
                    }
                })
                .unwrap_or(0);

            // 检查 IFF_UP (0x1) 标志是否设置
            if (flags & 0x1) != 0 {
                let carrier = fs::read_to_string(interface_path.join("carrier"))
                    .ok()
                    .map(|s| s.trim().to_string())
                    .unwrap_or_default();
                if carrier == "1" || !ip_addresses.is_empty() {
                    status = "up".to_string();
                }
            }
        }

        interfaces.push(NetworkInterfaceInfo {
            name: interface_name,
            status,
            mac_address,
            mtu,
            ip_addresses,
            rx_bytes,
            tx_bytes,
            rx_packets,
            tx_packets,
            rx_errors,
            tx_errors,
        });
    }

    // 按接口名称排序
    interfaces.sort_by(|a, b| a.name.cmp(&b.name));

    Ok(interfaces)
}
