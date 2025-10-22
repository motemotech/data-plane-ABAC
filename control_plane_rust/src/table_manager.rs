use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// テーブルエントリマネージャー
#[derive(Debug)]
pub struct TableManager {
    /// デバイスごとのテーブルエントリ
    device_tables: Arc<RwLock<HashMap<DeviceId, HashMap<String, Vec<TableEntry>>>>>,
    /// テーブル名のマッピング
    table_names: Arc<RwLock<HashMap<String, String>>>,
}

impl TableManager {
    pub fn new() -> Self {
        Self {
            device_tables: Arc::new(RwLock::new(HashMap::new())),
            table_names: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// デバイスのテーブルを初期化
    pub async fn initialize_device_tables(&self, device_id: DeviceId) {
        let mut tables = self.device_tables.write().await;
        tables.insert(device_id, HashMap::new());
        tracing::info!("Initialized tables for device {}", device_id);
    }
    
    /// テーブル名を登録
    pub async fn register_table_name(&self, table_name: &str, p4_table_name: &str) {
        let mut names = self.table_names.write().await;
        names.insert(table_name.to_string(), p4_table_name.to_string());
    }
    
    /// IPv4 LPMテーブルにエントリを追加
    pub async fn add_ipv4_lpm_entry(
        &self,
        device_id: DeviceId,
        prefix: Ipv4Address,
        prefix_len: u8,
        action: TableAction,
        priority: u32,
    ) -> Result<()> {
        let key = TableKey {
            ipv4_dst: prefix,
            prefix_len,
        };
        
        let entry = TableEntry {
            key,
            action,
            priority,
        };
        
        let mut tables = self.device_tables.write().await;
        if let Some(device_tables) = tables.get_mut(&device_id) {
            let table_name = "ipv4_lpm".to_string();
            let table_entries = device_tables.entry(table_name).or_insert_with(Vec::new);
            
            // 既存のエントリをチェックして重複を避ける
            if let Some(existing_index) = table_entries.iter().position(|e| e.key == entry.key) {
                table_entries[existing_index] = entry.clone();
                tracing::info!("Updated existing LPM entry for {} on device {}", prefix, device_id);
            } else {
                table_entries.push(entry.clone());
                tracing::info!("Added new LPM entry for {} on device {}", prefix, device_id);
            }
        } else {
            return Err(P4RuntimeError::DeviceNotFound { device_id }.into());
        }
        
        Ok(())
    }
    
    /// IPv4 LPMテーブルからエントリを削除
    pub async fn remove_ipv4_lpm_entry(
        &self,
        device_id: DeviceId,
        prefix: Ipv4Address,
        prefix_len: u8,
    ) -> Result<()> {
        let key = TableKey {
            ipv4_dst: prefix,
            prefix_len,
        };
        
        let mut tables = self.device_tables.write().await;
        if let Some(device_tables) = tables.get_mut(&device_id) {
            if let Some(table_entries) = device_tables.get_mut("ipv4_lpm") {
                if let Some(index) = table_entries.iter().position(|e| e.key == key) {
                    table_entries.remove(index);
                    tracing::info!("Removed LPM entry for {} from device {}", prefix, device_id);
                } else {
                    tracing::warn!("LPM entry for {} not found on device {}", prefix, device_id);
                }
            }
        } else {
            return Err(P4RuntimeError::DeviceNotFound { device_id }.into());
        }
        
        Ok(())
    }
    
    /// デバイスのIPv4 LPMテーブルエントリを取得
    pub async fn get_ipv4_lpm_entries(&self, device_id: DeviceId) -> Result<Vec<TableEntry>> {
        let tables = self.device_tables.read().await;
        if let Some(device_tables) = tables.get(&device_id) {
            Ok(device_tables.get("ipv4_lpm").cloned().unwrap_or_default())
        } else {
            Err(P4RuntimeError::DeviceNotFound { device_id }.into())
        }
    }
    
    /// 全デバイスのIPv4 LPMテーブルエントリを取得
    pub async fn get_all_ipv4_lpm_entries(&self) -> HashMap<DeviceId, Vec<TableEntry>> {
        let tables = self.device_tables.read().await;
        let mut result = HashMap::new();
        
        for (device_id, device_tables) in tables.iter() {
            if let Some(entries) = device_tables.get("ipv4_lpm") {
                result.insert(*device_id, entries.clone());
            }
        }
        
        result
    }
    
    /// デバイスの全テーブルエントリを取得
    pub async fn get_all_device_entries(&self, device_id: DeviceId) -> Result<HashMap<String, Vec<TableEntry>>> {
        let tables = self.device_tables.read().await;
        if let Some(device_tables) = tables.get(&device_id) {
            Ok(device_tables.clone())
        } else {
            Err(P4RuntimeError::DeviceNotFound { device_id }.into())
        }
    }
    
    /// デバイスのテーブルをクリア
    pub async fn clear_device_tables(&self, device_id: DeviceId) -> Result<()> {
        let mut tables = self.device_tables.write().await;
        if let Some(device_tables) = tables.get_mut(&device_id) {
            device_tables.clear();
            tracing::info!("Cleared all tables for device {}", device_id);
        } else {
            return Err(P4RuntimeError::DeviceNotFound { device_id }.into());
        }
        Ok(())
    }
    
    /// デバイスを削除
    pub async fn remove_device(&self, device_id: DeviceId) {
        let mut tables = self.device_tables.write().await;
        tables.remove(&device_id);
        tracing::info!("Removed device {} from table manager", device_id);
    }
    
    /// テーブルエントリの検索（最長プレフィックスマッチ）
    pub async fn find_lpm_entry(
        &self,
        device_id: DeviceId,
        dst_ip: Ipv4Address,
    ) -> Result<Option<TableEntry>> {
        let entries = self.get_ipv4_lpm_entries(device_id).await?;
        
        let mut best_match: Option<TableEntry> = None;
        let mut best_prefix_len = 0;
        
        for entry in entries {
            let prefix = entry.key.ipv4_dst.as_u32();
            let prefix_len = entry.key.prefix_len;
            let dst_ip_u32 = dst_ip.as_u32();
            
            // プレフィックスマスクを作成
            let mask = if prefix_len == 0 {
                0
            } else {
                !((1u32 << (32 - prefix_len)) - 1)
            };
            
            // プレフィックスマッチをチェック
            if (prefix & mask) == (dst_ip_u32 & mask) && prefix_len >= best_prefix_len {
                best_match = Some(entry);
                best_prefix_len = prefix_len;
            }
        }
        
        Ok(best_match)
    }
    
    /// テーブル統計情報を取得
    pub async fn get_table_statistics(&self, device_id: DeviceId) -> Result<HashMap<String, usize>> {
        let tables = self.device_tables.read().await;
        if let Some(device_tables) = tables.get(&device_id) {
            let mut stats = HashMap::new();
            for (table_name, entries) in device_tables.iter() {
                stats.insert(table_name.clone(), entries.len());
            }
            Ok(stats)
        } else {
            Err(P4RuntimeError::DeviceNotFound { device_id }.into())
        }
    }
}

impl Default for TableManager {
    fn default() -> Self {
        Self::new()
    }
}

/// テーブルエントリビルダー
#[derive(Debug)]
pub struct TableEntryBuilder {
    device_id: Option<DeviceId>,
    prefix: Option<Ipv4Address>,
    prefix_len: Option<u8>,
    action: Option<TableAction>,
    priority: u32,
}

impl TableEntryBuilder {
    pub fn new() -> Self {
        Self {
            device_id: None,
            prefix: None,
            prefix_len: None,
            action: None,
            priority: 0,
        }
    }
    
    pub fn device_id(mut self, device_id: DeviceId) -> Self {
        self.device_id = Some(device_id);
        self
    }
    
    pub fn prefix(mut self, prefix: Ipv4Address) -> Self {
        self.prefix = Some(prefix);
        self
    }
    
    pub fn prefix_len(mut self, prefix_len: u8) -> Self {
        self.prefix_len = Some(prefix_len);
        self
    }
    
    pub fn action(mut self, action: TableAction) -> Self {
        self.action = Some(action);
        self
    }
    
    pub fn priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }
    
    pub fn build(self) -> Result<TableEntry> {
        let prefix = self.prefix.ok_or_else(|| P4RuntimeError::InvalidTableEntry("Missing prefix".to_string()))?;
        let prefix_len = self.prefix_len.ok_or_else(|| P4RuntimeError::InvalidTableEntry("Missing prefix length".to_string()))?;
        let action = self.action.ok_or_else(|| P4RuntimeError::InvalidTableEntry("Missing action".to_string()))?;
        
        Ok(TableEntry {
            key: TableKey {
                ipv4_dst: prefix,
                prefix_len,
            },
            action,
            priority: self.priority,
        })
    }
}

impl Default for TableEntryBuilder {
    fn default() -> Self {
        Self::new()
    }
}
