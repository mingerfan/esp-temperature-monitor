use anyhow::Result;
use esp_idf_svc::sntp::{EspSntp, SntpConf, SyncStatus};
use log::{info, warn};
use std::time::Duration;

/// NTP 时间同步配置
pub struct NtpConfig {
    /// NTP 服务器列表
    pub servers: Vec<String>,
    /// 同步超时时间（秒）
    pub timeout_secs: u64,
    /// 是否等待同步完成
    pub wait_for_sync: bool,
}

impl Default for NtpConfig {
    fn default() -> Self {
        Self {
            // 使用常用的 NTP 服务器
            servers: vec![
                "pool.ntp.org".to_string(),
                "time.google.com".to_string(),
                "time.cloudflare.com".to_string(),
            ],
            timeout_secs: 30,
            wait_for_sync: true,
        }
    }
}

impl NtpConfig {
    /// 创建新的 NTP 配置
    pub fn new() -> Self {
        Self::default()
    }

    // /// 设置 NTP 服务器列表
    // pub fn servers(mut self, servers: Vec<String>) -> Self {
    //     self.servers = servers;
    //     self
    // }

    // /// 设置单个 NTP 服务器
    // pub fn server(mut self, server: impl Into<String>) -> Self {
    //     self.servers = vec![server.into()];
    //     self
    // }

    /// 设置中国常用的 NTP 服务器
    pub fn china_servers(mut self) -> Self {
        self.servers = vec![
            "ntp.aliyun.com".to_string(),
            "ntp1.aliyun.com".to_string(),
            "time.pool.aliyun.com".to_string(),
            "cn.ntp.org.cn".to_string(),
        ];
        self
    }

    // /// 使用全球通用的 NTP 服务器（更可靠）
    // pub fn global_servers(mut self) -> Self {
    //     self.servers = vec![
    //         "pool.ntp.org".to_string(),
    //         "time.google.com".to_string(),
    //         "time.cloudflare.com".to_string(),
    //         "time.apple.com".to_string(),
    //     ];
    //     self
    // }

    /// 设置超时时间
    pub fn timeout(mut self, secs: u64) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// 设置是否等待同步完成
    pub fn wait_for_sync(mut self, wait: bool) -> Self {
        self.wait_for_sync = wait;
        self
    }

    /// 初始化并启动 NTP 时间同步
    pub fn init(self) -> Result<EspSntp<'static>> {
        info!("正在初始化 NTP 时间同步...");
        info!("NTP 服务器: {:?}", self.servers);

        // 创建 SNTP 配置
        let sntp_conf = SntpConf {
            servers: [
                self.servers.first().map(|s| s.as_str()).unwrap_or("pool.ntp.org"),
            ],
            ..Default::default()
        };

        // 初始化 SNTP
        let sntp = EspSntp::new(&sntp_conf)?;
        info!("NTP 客户端已启动");

        // 如果需要等待同步
        if self.wait_for_sync {
            info!("正在同步时间，请稍候...");
            
            // 给 SNTP 服务一些时间来启动
            std::thread::sleep(Duration::from_millis(500));
            
            let start = std::time::Instant::now();
            let timeout = Duration::from_secs(self.timeout_secs);
            let mut last_status_print = std::time::Instant::now();
            let mut reset_count = 0;

            loop {
                let status = sntp.get_sync_status();
                let elapsed = start.elapsed();
                
                match status {
                    SyncStatus::Completed => {
                        info!("✅ 时间同步完成！耗时 {:.1} 秒", elapsed.as_secs_f32());
                        print_current_time();
                        break;
                    }
                    SyncStatus::InProgress => {
                        // 每 5 秒打印一次进度
                        if last_status_print.elapsed() > Duration::from_secs(5) {
                            info!("⏳ 同步中... 已等待 {:.1} 秒", elapsed.as_secs_f32());
                            last_status_print = std::time::Instant::now();
                        }
                        
                        if elapsed > timeout {
                            warn!("⚠️  时间同步超时（{} 秒），将在后台继续同步", self.timeout_secs);
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(500));
                    }
                    SyncStatus::Reset => {
                        reset_count += 1;
                        
                        // Reset 状态通常表示还没开始同步，给更多时间
                        if reset_count == 1 {
                            info!("⏳ 正在初始化同步连接...");
                        } else if reset_count % 10 == 0 {
                            // 每 10 次（约 5 秒）打印一次
                            warn!("⏳ 正在尝试连接 NTP 服务器... ({:.1}秒)", elapsed.as_secs_f32());
                        }
                        
                        if elapsed > timeout {
                            warn!("⚠️  无法连接到 NTP 服务器（超时 {} 秒）", self.timeout_secs);
                            warn!("💡 建议：");
                            warn!("  1. 检查网络连接是否正常");
                            warn!("  2. 尝试更换 NTP 服务器（使用 .china_servers() 或 .server()）");
                            warn!("  3. 检查防火墙是否阻止 UDP 123 端口");
                            break;
                        }
                        std::thread::sleep(Duration::from_millis(500));
                    }
                }
            }
        } else {
            info!("NTP 同步已启动（后台运行）");
        }

        Ok(sntp)
    }
}

/// 测试网络连接（在同步 NTP 前调用）
pub fn test_network_connectivity() -> bool {
    use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream, ToSocketAddrs};
    use std::time::Duration;

    info!("正在测试网络连接...");
    
    // 首先测试直接 IP 连接（不需要 DNS）
    let direct_ips = [
        (IpAddr::V4(Ipv4Addr::new(223, 5, 5, 5)), 80, "阿里云DNS"),  // 阿里 DNS
        (IpAddr::V4(Ipv4Addr::new(119, 29, 29, 29)), 80, "DNSPod"),  // DNSPod
    ];

    for (ip, port, name) in direct_ips.iter() {
        let addr = SocketAddr::new(*ip, *port);
        match TcpStream::connect_timeout(&addr, Duration::from_secs(3)) {
            Ok(_) => {
                info!("✅ 网络连接正常（直连 {name} - {addr}）");
                return true;
            }
            Err(e) => {
                warn!("  无法直连 {name}: {e}");
            }
        }
    }
    
    info!("直连 IP 测试失败，尝试 DNS 解析...");
    
    // 测试 DNS 解析和网络连通性
    let test_targets = [
        ("www.baidu.com", 80),
        ("www.qq.com", 80),
    ];

    for (host, port) in test_targets.iter() {
        info!("尝试解析并连接 {host}:{port}...");
        
        // 测试 DNS 解析
        match format!("{host}:{port}").to_socket_addrs() {
            Ok(mut addrs) => {
                if let Some(addr) = addrs.next() {
                    info!("  DNS 解析成功: {} -> {}", host, addr.ip());
                    
                    // 尝试 TCP 连接
                    match TcpStream::connect_timeout(&addr, Duration::from_secs(5)) {
                        Ok(_) => {
                            info!("✅ 网络连接正常（通过 {host}:{port}）");
                            return true;
                        }
                        Err(e) => {
                            warn!("  TCP 连接失败: {e}");
                        }
                    }
                } else {
                    warn!("  DNS 解析返回空地址");
                }
            }
            Err(e) => {
                warn!("  DNS 解析失败 {host}: {e}");
            }
        }
    }

    warn!("❌ 网络连接测试失败，请检查：");
    warn!("   1. WiFi 是否真的连接成功（查看 IP 地址）");
    warn!("   2. 路由器是否能访问互联网");
    warn!("   3. DNS 设置是否正确");
    warn!("   4. 防火墙是否阻止了连接");
    false
}

/// 打印当前系统时间
pub fn print_current_time() {
    use time::{format_description, OffsetDateTime};

    if let Ok(now) = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        if let Ok(datetime) = OffsetDateTime::from_unix_timestamp(now.as_secs() as i64) {
            if let Ok(format) =
                format_description::parse("[year]-[month]-[day] [hour]:[minute]:[second] UTC")
            {
                if let Ok(time_str) = datetime.format(&format) {
                    info!("当前系统时间: {time_str}");
                }
            }
        }
    }
}

// /// 检查时间是否已同步
// pub fn is_time_synced(sntp: &EspSntp) -> bool {
//     matches!(sntp.get_sync_status(), SyncStatus::Completed)
// }
