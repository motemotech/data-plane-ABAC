use crate::types::*;
use crate::p4runtime_client::DeviceManager;
use crate::table_manager::TableManager;
use crate::routing_manager::RoutingManager;
use anyhow::Result;
use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, error};

/// P4コントローラーのメインアプリケーション
#[derive(Debug)]
pub struct P4Controller {
    device_manager: Arc<DeviceManager>,
    table_manager: Arc<TableManager>,
    routing_manager: Arc<RoutingManager>,
    state: Arc<RwLock<ControllerState>>,
}

impl P4Controller {
    pub fn new() -> Self {
        Self {
            device_manager: Arc::new(DeviceManager::new()),
            table_manager: Arc::new(TableManager::new()),
            routing_manager: Arc::new(RoutingManager::new()),
            state: Arc::new(RwLock::new(ControllerState::default())),
        }
    }
    
    /// コントローラーを初期化
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing P4 Controller...");
        
        // デフォルト設定を読み込み
        self.load_default_config().await?;
        
        // デフォルトルートを追加
        self.setup_default_routes().await?;
        
        // デフォルトARPエントリを追加
        self.setup_default_arp_entries().await?;
        
        info!("P4 Controller initialized successfully");
        Ok(())
    }
    
    /// デバイスを追加
    pub async fn add_device(&self, device_info: DeviceInfo) -> Result<()> {
        info!("Adding device: {} ({})", device_info.device_id, device_info.name);
        
        // デバイスマネージャーに追加
        self.device_manager.add_device(device_info.clone()).await?;
        
        // テーブルマネージャーでデバイスを初期化
        self.table_manager.initialize_device_tables(device_info.device_id).await;
        
        // 状態を更新
        let device_id = device_info.device_id;
        {
            let mut state = self.state.write().await;
            state.connected_devices.insert(device_id, device_info);
        }
        
        // ルーティングテーブルをデバイスに適用
        self.apply_routing_table_to_device(device_id).await?;
        
        info!("Device added successfully");
        Ok(())
    }
    
    /// デバイスを削除
    pub async fn remove_device(&self, device_id: DeviceId) -> Result<()> {
        info!("Removing device: {}", device_id);
        
        // 各マネージャーからデバイスを削除
        self.device_manager.remove_device(device_id).await?;
        self.table_manager.remove_device(device_id).await;
        
        // 状態を更新
        {
            let mut state = self.state.write().await;
            state.connected_devices.remove(&device_id);
        }
        
        info!("Device removed successfully");
        Ok(())
    }
    
    /// ルートを追加
    pub async fn add_route(&self, route: RouteEntry) -> Result<()> {
        info!("Adding route: {}/{}", route.prefix, route.prefix_len);
        
        // ルーティングマネージャーに追加
        self.routing_manager.add_route(route.clone()).await?;
        
        // 全接続デバイスにルートを適用
        self.apply_route_to_all_devices(&route).await?;
        
        info!("Route added successfully");
        Ok(())
    }
    
    /// ルートを削除
    pub async fn remove_route(&self, prefix: Ipv4Address, prefix_len: u8) -> Result<()> {
        info!("Removing route: {}/{}", prefix, prefix_len);
        
        // ルーティングマネージャーから削除
        self.routing_manager.remove_route(prefix, prefix_len).await?;
        
        // 全接続デバイスからルートを削除
        self.remove_route_from_all_devices(prefix, prefix_len).await?;
        
        info!("Route removed successfully");
        Ok(())
    }
    
    /// ARPエントリを追加
    pub async fn add_arp_entry(&self, arp_entry: ArpEntry) -> Result<()> {
        info!("Adding ARP entry: {} -> {}", arp_entry.ip, arp_entry.mac);
        
        // ルーティングマネージャーに追加
        self.routing_manager.add_arp_entry(arp_entry).await;
        
        // ルーティングテーブルを再適用（MACアドレスが変更された可能性があるため）
        self.apply_routing_table_to_all_devices().await?;
        
        info!("ARP entry added successfully");
        Ok(())
    }
    
    /// ポートを追加
    pub async fn add_port(&self, port: PortInfo) -> Result<()> {
        info!("Adding port: {} ({})", port.port_id, port.name);
        
        // ルーティングマネージャーに追加
        self.routing_manager.add_port(port).await;
        
        info!("Port added successfully");
        Ok(())
    }
    
    /// ポートの状態を更新
    pub async fn update_port_status(&self, port_id: PortId, is_up: bool) -> Result<()> {
        info!("Updating port {} status: {}", port_id, if is_up { "UP" } else { "DOWN" });
        
        // ルーティングマネージャーで状態を更新
        self.routing_manager.update_port_status(port_id, is_up).await?;
        
        info!("Port status updated successfully");
        Ok(())
    }
    
    /// ルートを特定のデバイスに適用
    async fn apply_route_to_device(&self, device_id: DeviceId, route: &RouteEntry) -> Result<()> {
        if let Some(table_entry) = self.routing_manager.convert_route_to_table_entry(route, device_id).await? {
            self.table_manager.add_ipv4_lpm_entry(
                device_id,
                table_entry.key.ipv4_dst,
                table_entry.key.prefix_len,
                table_entry.action.clone(),
                table_entry.priority,
            ).await?;
            
            // デバイスにテーブルエントリを書き込み
            self.device_manager.write_table_entries_to_device(
                device_id,
                &[table_entry],
            ).await?;
        }
        
        Ok(())
    }
    
    /// ルートを全デバイスに適用
    async fn apply_route_to_all_devices(&self, route: &RouteEntry) -> Result<()> {
        let devices = self.device_manager.list_devices().await;
        
        for device in devices {
            if let Err(e) = self.apply_route_to_device(device.device_id, route).await {
                error!("Failed to apply route to device {}: {}", device.device_id, e);
            }
        }
        
        Ok(())
    }
    
    /// デバイスからルートを削除
    async fn remove_route_from_device(&self, device_id: DeviceId, prefix: Ipv4Address, prefix_len: u8) -> Result<()> {
        self.table_manager.remove_ipv4_lpm_entry(device_id, prefix, prefix_len).await?;
        
        // デバイスからテーブルエントリを削除
        // 実際のP4Runtimeでは、WriteRequestでDELETE操作を送信
        info!("Removed route {}/{} from device {}", prefix, prefix_len, device_id);
        
        Ok(())
    }
    
    /// 全デバイスからルートを削除
    async fn remove_route_from_all_devices(&self, prefix: Ipv4Address, prefix_len: u8) -> Result<()> {
        let devices = self.device_manager.list_devices().await;
        
        for device in devices {
            if let Err(e) = self.remove_route_from_device(device.device_id, prefix, prefix_len).await {
                error!("Failed to remove route from device {}: {}", device.device_id, e);
            }
        }
        
        Ok(())
    }
    
    /// ルーティングテーブルを特定のデバイスに適用
    async fn apply_routing_table_to_device(&self, device_id: DeviceId) -> Result<()> {
        let table_entries = self.routing_manager.convert_all_routes_to_table_entries(device_id).await?;
        
        if !table_entries.is_empty() {
            // テーブルマネージャーに追加
            for entry in &table_entries {
                self.table_manager.add_ipv4_lpm_entry(
                    device_id,
                    entry.key.ipv4_dst,
                    entry.key.prefix_len,
                    entry.action.clone(),
                    entry.priority,
                ).await?;
            }
            
            // デバイスにテーブルエントリを書き込み
            self.device_manager.write_table_entries_to_device(device_id, &table_entries).await?;
        }
        
        Ok(())
    }
    
    /// ルーティングテーブルを全デバイスに適用
    async fn apply_routing_table_to_all_devices(&self) -> Result<()> {
        let devices = self.device_manager.list_devices().await;
        
        for device in devices {
            if let Err(e) = self.apply_routing_table_to_device(device.device_id).await {
                error!("Failed to apply routing table to device {}: {}", device.device_id, e);
            }
        }
        
        Ok(())
    }
    
    /// デフォルト設定を読み込み
    async fn load_default_config(&self) -> Result<()> {
        // 実際の実装では、設定ファイルから読み込む
        info!("Loading default configuration");
        Ok(())
    }
    
    /// デフォルトルートを設定
    async fn setup_default_routes(&self) -> Result<()> {
        info!("Setting up default routes");
        
        // デフォルトゲートウェイルート
        let default_route = RouteEntry {
            prefix: Ipv4Address::from_u32(0), // 0.0.0.0
            prefix_len: 0,
            next_hop: Some(Ipv4Address::new(Ipv4Addr::new(192, 168, 1, 1))),
            interface: "eth0".to_string(),
            metric: 1,
        };
        
        self.routing_manager.add_route(default_route).await?;
        
        // ローカルネットワークルート
        let local_route = RouteEntry {
            prefix: Ipv4Address::new(Ipv4Addr::new(192, 168, 1, 0)),
            prefix_len: 24,
            next_hop: None, // 直接接続
            interface: "eth0".to_string(),
            metric: 0,
        };
        
        self.routing_manager.add_route(local_route).await?;
        
        Ok(())
    }
    
    /// デフォルトARPエントリを設定
    async fn setup_default_arp_entries(&self) -> Result<()> {
        info!("Setting up default ARP entries");
        
        // デフォルトゲートウェイのARPエントリ
        let gateway_arp = ArpEntry {
            ip: Ipv4Address::new(Ipv4Addr::new(192, 168, 1, 1)),
            mac: MacAddress::new([0x00, 0x11, 0x22, 0x33, 0x44, 0x55]),
            interface: "eth0".to_string(),
        };
        
        self.routing_manager.add_arp_entry(gateway_arp).await;
        
        Ok(())
    }
    
    /// 統計情報を取得
    pub async fn get_statistics(&self) -> Result<HashMap<DeviceId, Statistics>> {
        Ok(self.device_manager.get_all_device_statistics().await)
    }
    
    /// コントローラー状態を取得
    pub async fn get_state(&self) -> ControllerState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// デバイス一覧を取得
    pub async fn list_devices(&self) -> Vec<DeviceInfo> {
        self.device_manager.list_devices().await
    }
    
    /// ルート一覧を取得
    pub async fn list_routes(&self) -> Vec<RouteEntry> {
        self.routing_manager.get_all_routes().await
    }
    
    /// ARPエントリ一覧を取得
    pub async fn list_arp_entries(&self) -> Vec<ArpEntry> {
        self.routing_manager.get_all_arp_entries().await
    }
    
    /// ポート一覧を取得
    pub async fn list_ports(&self) -> Vec<PortInfo> {
        self.routing_manager.get_all_ports().await
    }
}

impl Default for P4Controller {
    fn default() -> Self {
        Self::new()
    }
}
