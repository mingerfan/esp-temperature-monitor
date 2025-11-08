use anyhow::{bail, Result};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    hal::peripheral,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use log::info;

/// WiFi 配置构建器
/// 
/// # 事件循环说明
/// 
/// `EspSystemEventLoop` 是 ESP-IDF 的系统事件循环，用于处理各种系统事件：
/// - WiFi 连接/断开事件
/// - IP 地址分配事件
/// - 网络状态变化事件
/// 
/// 它是 ESP32 异步事件处理的核心机制，WiFi、蓝牙、以太网等模块都依赖它。
pub struct WifiBuilder<'a> {
    ssid: &'a str,
    password: &'a str,
    auth_method: Option<AuthMethod>,
    channel: Option<u8>,
    scan_for_channel: bool,
    auto_connect: bool,
    bssid: Option<[u8; 6]>,
}

impl<'a> WifiBuilder<'a> {
    /// 创建一个新的 WiFi 配置构建器
    ///
    /// # 参数
    /// - `ssid`: WiFi 网络名称
    /// - `password`: WiFi 密码（如果为空，将使用无认证方式）
    pub fn new(ssid: &'a str, password: &'a str) -> Self {
        Self {
            ssid,
            password,
            auth_method: None,
            channel: None,
            scan_for_channel: true,
            auto_connect: true,
            bssid: None,
        }
    }

    // /// 设置认证方法
    // ///
    // /// 如果不设置，将根据密码自动选择：
    // /// - 密码为空：AuthMethod::None
    // /// - 密码不为空：AuthMethod::WPA2Personal
    // pub fn auth_method(mut self, auth_method: AuthMethod) -> Self {
    //     self.auth_method = Some(auth_method);
    //     self
    // }

    // /// 设置指定的 WiFi 频道
    // ///
    // /// 如果设置了频道，将不会进行扫描
    // pub fn channel(mut self, channel: u8) -> Self {
    //     self.channel = Some(channel);
    //     self.scan_for_channel = false;
    //     self
    // }

    // /// 设置是否扫描并自动选择频道
    // ///
    // /// 默认为 true
    // pub fn scan_for_channel(mut self, scan: bool) -> Self {
    //     self.scan_for_channel = scan;
    //     self
    // }

    // /// 设置是否自动连接
    // ///
    // /// 默认为 true。如果设置为 false，需要手动调用连接方法
    // pub fn auto_connect(mut self, auto_connect: bool) -> Self {
    //     self.auto_connect = auto_connect;
    //     self
    // }

    // /// 设置 BSSID（MAC 地址）
    // ///
    // /// 用于连接到特定的接入点
    // pub fn bssid(mut self, bssid: [u8; 6]) -> Self {
    //     self.bssid = Some(bssid);
    //     self
    // }

    /// 构建并初始化 WiFi 连接
    ///
    /// # 参数
    /// - `modem`: WiFi modem 外设
    /// - `sysloop`: 系统事件循环（用于处理 WiFi 事件）
    pub fn build(
        self,
        modem: impl peripheral::Peripheral<P = esp_idf_svc::hal::modem::Modem> + 'static,
        sysloop: EspSystemEventLoop,
    ) -> Result<Box<EspWifi<'static>>> {
        // 验证 SSID
        if self.ssid.is_empty() {
            bail!("Missing WiFi name")
        }

        // 确定认证方法
        let auth_method = if let Some(method) = self.auth_method {
            method
        } else if self.password.is_empty() {
            info!("Wifi password is empty, using AuthMethod::None");
            AuthMethod::None
        } else {
            AuthMethod::WPA2Personal
        };

        // 创建 WiFi 实例
        let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), None)?;
        let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;

        // 设置初始配置
        wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))?;

        info!("Starting wifi...");
        wifi.start()?;

        // 扫描并查找频道（如果需要）
        let channel = if let Some(ch) = self.channel {
            Some(ch)
        } else if self.scan_for_channel {
            info!("Scanning for WiFi networks...");
            let ap_infos = wifi.scan()?;
            let ours = ap_infos.into_iter().find(|a| a.ssid == self.ssid);

            if let Some(ours) = ours {
                info!(
                    "Found configured access point {} on channel {}",
                    self.ssid, ours.channel
                );
                Some(ours.channel)
            } else {
                info!(
                    "Configured access point {} not found during scanning, will go with unknown channel",
                    self.ssid
                );
                None
            }
        } else {
            None
        };

        // 配置 WiFi 客户端
        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: self
                .ssid
                .try_into()
                .expect("Could not parse the given SSID into WiFi config"),
            password: self
                .password
                .try_into()
                .expect("Could not parse the given password into WiFi config"),
            channel,
            auth_method,
            bssid: self.bssid,
            ..Default::default()
        }))?;

        // 自动连接（如果启用）
        if self.auto_connect {
            info!("Connecting to wifi...");
            wifi.connect()?;

            info!("Waiting for DHCP lease...");
            wifi.wait_netif_up()?;

            let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
            info!("Wifi DHCP info: {ip_info:?}");
        }

        Ok(Box::new(esp_wifi))
    }
}