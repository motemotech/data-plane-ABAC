use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::Ipv4Addr;
use thiserror::Error;

/// P4Runtime関連のエラー型
#[derive(Error, Debug)]
pub enum P4RuntimeError {
    #[error("gRPC error: {0}")]
    GrpcError(#[from] tonic::Status),
    
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    
    #[error("Invalid table entry: {0}")]
    InvalidTableEntry(String),
    
    #[error("Device not found: {device_id}")]
    DeviceNotFound { device_id: u64 },
    
    #[error("Table not found: {table_name}")]
    TableNotFound { table_name: String },
}

/// P4RuntimeデバイスID
pub type DeviceId = u64;

/// P4Runtimeポート番号
pub type PortId = u32;

/// MACアドレス型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MacAddress([u8; 6]);

impl MacAddress {
    pub fn new(bytes: [u8; 6]) -> Self {
        Self(bytes)
    }
    
    pub fn as_bytes(&self) -> &[u8; 6] {
        &self.0
    }
    
    pub fn to_string(&self) -> String {
        format!("{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
                self.0[0], self.0[1], self.0[2], 
                self.0[3], self.0[4], self.0[5])
    }
}

impl std::fmt::Display for MacAddress {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// IPv4アドレス型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ipv4Address(u32);

impl Ipv4Address {
    pub fn new(addr: Ipv4Addr) -> Self {
        Self(addr.into())
    }
    
    pub fn from_u32(addr: u32) -> Self {
        Self(addr)
    }
    
    pub fn as_u32(&self) -> u32 {
        self.0
    }
    
    pub fn as_ipv4(&self) -> Ipv4Addr {
        Ipv4Addr::from(self.0)
    }
}

impl std::fmt::Display for Ipv4Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ipv4())
    }
}

/// P4テーブルエントリのキー
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TableKey {
    pub ipv4_dst: Ipv4Address,
    pub prefix_len: u8,
}

/// P4テーブルエントリのアクション
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableAction {
    /// IPv4フォワーディングアクション
    Ipv4Forward {
        dst_mac: MacAddress,
        port: PortId,
    },
    /// ドロップアクション
    Drop,
}

/// P4テーブルエントリ
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableEntry {
    pub key: TableKey,
    pub action: TableAction,
    pub priority: u32,
}

/// デバイス情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub device_id: DeviceId,
    pub name: String,
    pub grpc_endpoint: String,
    pub p4info: Option<P4Info>,
}

/// P4プログラム情報（簡略化版）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P4Info {
    pub tables: HashMap<String, TableInfo>,
    pub actions: HashMap<String, ActionInfo>,
}

/// テーブル情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub id: u32,
    pub key_fields: Vec<KeyField>,
    pub action_refs: Vec<ActionRef>,
}

/// キーフィールド情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyField {
    pub name: String,
    pub bitwidth: u32,
    pub match_type: MatchType,
}

/// マッチタイプ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MatchType {
    Exact,
    Lpm,  // Longest Prefix Match
    Ternary,
    Range,
}

/// アクション参照
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionRef {
    pub name: String,
    pub id: u32,
}

/// アクション情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionInfo {
    pub name: String,
    pub id: u32,
    pub params: Vec<ActionParam>,
}

/// アクションパラメータ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionParam {
    pub name: String,
    pub bitwidth: u32,
}

/// ルーティングテーブルエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteEntry {
    pub prefix: Ipv4Address,
    pub prefix_len: u8,
    pub next_hop: Option<Ipv4Address>,
    pub interface: String,
    pub metric: u32,
}

/// ARPテーブルエントリ
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArpEntry {
    pub ip: Ipv4Address,
    pub mac: MacAddress,
    pub interface: String,
}

/// スイッチポート情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortInfo {
    pub port_id: PortId,
    pub name: String,
    pub mac_address: MacAddress,
    pub ip_address: Option<Ipv4Address>,
    pub is_up: bool,
}

/// コントローラー設定
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControllerConfig {
    pub devices: Vec<DeviceInfo>,
    pub default_routes: Vec<RouteEntry>,
    pub arp_table: Vec<ArpEntry>,
}

impl Default for ControllerConfig {
    fn default() -> Self {
        Self {
            devices: Vec::new(),
            default_routes: Vec::new(),
            arp_table: Vec::new(),
        }
    }
}

/// P4Runtimeメッセージの簡略化版
#[derive(Debug, Clone)]
pub struct P4RuntimeMessage {
    pub device_id: DeviceId,
    pub table_entries: Vec<TableEntry>,
}

/// 統計情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statistics {
    pub packets_processed: u64,
    pub bytes_processed: u64,
    pub table_hits: HashMap<String, u64>,
    pub table_misses: HashMap<String, u64>,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            packets_processed: 0,
            bytes_processed: 0,
            table_hits: HashMap::new(),
            table_misses: HashMap::new(),
        }
    }
}

/// コントローラー状態
#[derive(Debug, Clone)]
pub struct ControllerState {
    pub config: ControllerConfig,
    pub statistics: Statistics,
    pub connected_devices: HashMap<DeviceId, DeviceInfo>,
}

impl Default for ControllerState {
    fn default() -> Self {
        Self {
            config: ControllerConfig::default(),
            statistics: Statistics::default(),
            connected_devices: HashMap::new(),
        }
    }
}