use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// ルーティングテーブルマネージャー
#[derive(Debug)]
pub struct RoutingManager {
    /// ルーティングテーブル
    routes: Arc<RwLock<Vec<RouteEntry>>>,
    /// ARPテーブル
    arp_table: Arc<RwLock<HashMap<Ipv4Address, ArpEntry>>>,
    /// ポート情報
    ports: Arc<RwLock<HashMap<PortId, PortInfo>>>,
}

impl RoutingManager {
    pub fn new() -> Self {
        Self {
            routes: Arc::new(RwLock::new(Vec::new())),
            arp_table: Arc::new(RwLock::new(HashMap::new())),
            ports: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// ルートを追加
    pub async fn add_route(&self, route: RouteEntry) -> Result<()> {
        let mut routes = self.routes.write().await;
        
        // 既存のルートをチェック
        if let Some(existing_index) = routes.iter().position(|r| 
            r.prefix == route.prefix && r.prefix_len == route.prefix_len) {
            routes[existing_index] = route.clone();
            tracing::info!("Updated route: {}/{}", route.prefix, route.prefix_len);
        } else {
            routes.push(route.clone());
            tracing::info!("Added route: {}/{}", route.prefix, route.prefix_len);
        }
        
        // メトリックでソート（低いメトリックが優先）
        routes.sort_by(|a, b| a.metric.cmp(&b.metric));
        
        Ok(())
    }
    
    /// ルートを削除
    pub async fn remove_route(&self, prefix: Ipv4Address, prefix_len: u8) -> Result<()> {
        let mut routes = self.routes.write().await;
        
        if let Some(index) = routes.iter().position(|r| 
            r.prefix == prefix && r.prefix_len == prefix_len) {
            let removed_route = routes.remove(index);
            tracing::info!("Removed route: {}/{}", removed_route.prefix, removed_route.prefix_len);
        } else {
            tracing::warn!("Route {}/{} not found", prefix, prefix_len);
        }
        
        Ok(())
    }
    
    /// ルートを検索（最長プレフィックスマッチ）
    pub async fn find_route(&self, dst_ip: Ipv4Address) -> Option<RouteEntry> {
        let routes = self.routes.read().await;
        
        let mut best_match: Option<RouteEntry> = None;
        let mut best_prefix_len = 0;
        
        for route in routes.iter() {
            let prefix = route.prefix.as_u32();
            let prefix_len = route.prefix_len;
            let dst_ip_u32 = dst_ip.as_u32();
            
            // プレフィックスマスクを作成
            let mask = if prefix_len == 0 {
                0
            } else {
                !((1u32 << (32 - prefix_len)) - 1)
            };
            
            // プレフィックスマッチをチェック
            if (prefix & mask) == (dst_ip_u32 & mask) && prefix_len >= best_prefix_len {
                best_match = Some(route.clone());
                best_prefix_len = prefix_len;
            }
        }
        
        best_match
    }
    
    /// 全ルートを取得
    pub async fn get_all_routes(&self) -> Vec<RouteEntry> {
        let routes = self.routes.read().await;
        routes.clone()
    }
    
    /// ARPエントリを追加
    pub async fn add_arp_entry(&self, arp_entry: ArpEntry) {
        let mut arp_table = self.arp_table.write().await;
        arp_table.insert(arp_entry.ip, arp_entry.clone());
        tracing::info!("Added ARP entry: {} -> {}", arp_entry.ip, arp_entry.mac);
    }
    
    /// ARPエントリを削除
    pub async fn remove_arp_entry(&self, ip: Ipv4Address) {
        let mut arp_table = self.arp_table.write().await;
        if let Some(entry) = arp_table.remove(&ip) {
            tracing::info!("Removed ARP entry: {} -> {}", entry.ip, entry.mac);
        }
    }
    
    /// ARPエントリを検索
    pub async fn find_arp_entry(&self, ip: Ipv4Address) -> Option<ArpEntry> {
        let arp_table = self.arp_table.read().await;
        arp_table.get(&ip).cloned()
    }
    
    /// 全ARPエントリを取得
    pub async fn get_all_arp_entries(&self) -> Vec<ArpEntry> {
        let arp_table = self.arp_table.read().await;
        arp_table.values().cloned().collect()
    }
    
    /// ポートを追加
    pub async fn add_port(&self, port: PortInfo) {
        let mut ports = self.ports.write().await;
        ports.insert(port.port_id, port.clone());
        tracing::info!("Added port: {} ({})", port.port_id, port.name);
    }
    
    /// ポートを削除
    pub async fn remove_port(&self, port_id: PortId) {
        let mut ports = self.ports.write().await;
        if let Some(port) = ports.remove(&port_id) {
            tracing::info!("Removed port: {} ({})", port.port_id, port.name);
        }
    }
    
    /// ポート情報を取得
    pub async fn get_port(&self, port_id: PortId) -> Option<PortInfo> {
        let ports = self.ports.read().await;
        ports.get(&port_id).cloned()
    }
    
    /// 全ポートを取得
    pub async fn get_all_ports(&self) -> Vec<PortInfo> {
        let ports = self.ports.read().await;
        ports.values().cloned().collect()
    }
    
    /// ポートの状態を更新
    pub async fn update_port_status(&self, port_id: PortId, is_up: bool) -> Result<()> {
        let mut ports = self.ports.write().await;
        if let Some(port) = ports.get_mut(&port_id) {
            port.is_up = is_up;
            tracing::info!("Updated port {} status: {}", port_id, if is_up { "UP" } else { "DOWN" });
        } else {
            tracing::warn!("Port {} not found", port_id);
        }
        Ok(())
    }
    
    /// ルーティングテーブルをクリア
    pub async fn clear_routes(&self) {
        let mut routes = self.routes.write().await;
        routes.clear();
        tracing::info!("Cleared all routes");
    }
    
    /// ARPテーブルをクリア
    pub async fn clear_arp_table(&self) {
        let mut arp_table = self.arp_table.write().await;
        arp_table.clear();
        tracing::info!("Cleared ARP table");
    }
    
    /// ポートテーブルをクリア
    pub async fn clear_ports(&self) {
        let mut ports = self.ports.write().await;
        ports.clear();
        tracing::info!("Cleared all ports");
    }
    
    /// ルートをP4テーブルエントリに変換
    pub async fn convert_route_to_table_entry(
        &self,
        route: &RouteEntry,
        _device_id: DeviceId,
    ) -> Result<Option<TableEntry>> {
        // ネクストホップのMACアドレスを取得
        let next_hop_mac = if let Some(next_hop) = route.next_hop {
            if let Some(arp_entry) = self.find_arp_entry(next_hop).await {
                arp_entry.mac
            } else {
                tracing::warn!("No ARP entry found for next hop {}", next_hop);
                return Ok(None);
            }
        } else {
            // 直接接続されたネットワークの場合、デフォルトゲートウェイのMACを使用
            // 実際の実装では、インターフェースのMACアドレスを使用
            MacAddress::new([0x08, 0x00, 0x00, 0x00, 0x00, 0x01])
        };
        
        // ポートIDを取得（インターフェース名から）
        let port_id = self.get_port_id_by_interface(&route.interface).await
            .unwrap_or(1); // デフォルトポート
        
        let action = TableAction::Ipv4Forward {
            dst_mac: next_hop_mac,
            port: port_id,
        };
        
        Ok(Some(TableEntry {
            key: TableKey {
                ipv4_dst: route.prefix,
                prefix_len: route.prefix_len,
            },
            action,
            priority: route.metric,
        }))
    }
    
    /// インターフェース名からポートIDを取得
    async fn get_port_id_by_interface(&self, interface: &str) -> Option<PortId> {
        let ports = self.ports.read().await;
        for (port_id, port) in ports.iter() {
            if port.name == interface {
                return Some(*port_id);
            }
        }
        None
    }
    
    /// ルーティングテーブルをP4テーブルエントリに一括変換
    pub async fn convert_all_routes_to_table_entries(
        &self,
        device_id: DeviceId,
    ) -> Result<Vec<TableEntry>> {
        let routes = self.routes.read().await;
        let mut table_entries = Vec::new();
        
        for route in routes.iter() {
            if let Some(entry) = self.convert_route_to_table_entry(route, device_id).await? {
                table_entries.push(entry);
            }
        }
        
        Ok(table_entries)
    }
}

impl Default for RoutingManager {
    fn default() -> Self {
        Self::new()
    }
}

/// ルーティングテーブルビルダー
#[derive(Debug)]
pub struct RouteBuilder {
    prefix: Option<Ipv4Address>,
    prefix_len: Option<u8>,
    next_hop: Option<Ipv4Address>,
    interface: Option<String>,
    metric: u32,
}

impl RouteBuilder {
    pub fn new() -> Self {
        Self {
            prefix: None,
            prefix_len: None,
            next_hop: None,
            interface: None,
            metric: 1,
        }
    }
    
    pub fn prefix(mut self, prefix: Ipv4Address) -> Self {
        self.prefix = Some(prefix);
        self
    }
    
    pub fn prefix_len(mut self, prefix_len: u8) -> Self {
        self.prefix_len = Some(prefix_len);
        self
    }
    
    pub fn next_hop(mut self, next_hop: Ipv4Address) -> Self {
        self.next_hop = Some(next_hop);
        self
    }
    
    pub fn interface(mut self, interface: String) -> Self {
        self.interface = Some(interface);
        self
    }
    
    pub fn metric(mut self, metric: u32) -> Self {
        self.metric = metric;
        self
    }
    
    pub fn build(self) -> Result<RouteEntry> {
        let prefix = self.prefix.ok_or_else(|| P4RuntimeError::InvalidTableEntry("Missing prefix".to_string()))?;
        let prefix_len = self.prefix_len.ok_or_else(|| P4RuntimeError::InvalidTableEntry("Missing prefix length".to_string()))?;
        let interface = self.interface.ok_or_else(|| P4RuntimeError::InvalidTableEntry("Missing interface".to_string()))?;
        
        Ok(RouteEntry {
            prefix,
            prefix_len,
            next_hop: self.next_hop,
            interface,
            metric: self.metric,
        })
    }
}

impl Default for RouteBuilder {
    fn default() -> Self {
        Self::new()
    }
}
