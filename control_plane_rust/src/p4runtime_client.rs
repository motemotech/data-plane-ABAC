use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::{Channel, Endpoint};

/// P4Runtime gRPCクライアント
#[derive(Debug)]
pub struct P4RuntimeClient {
    device_id: DeviceId,
    client: tonic::client::Grpc<Channel>,
}

impl P4RuntimeClient {
    /// 新しいP4Runtimeクライアントを作成
    pub async fn new(device_id: DeviceId, endpoint: &str) -> Result<Self> {
        let channel = Endpoint::from_shared(endpoint.to_string())?
            .connect()
            .await?;
        
        let client = tonic::client::Grpc::new(channel);
        
        Ok(Self {
            device_id,
            client,
        })
    }
    
    /// デバイスに接続を確立
    pub async fn connect(&mut self) -> Result<()> {
        // 実際のP4Runtimeでは、MasterArbitrationUpdateを送信してマスター権を取得
        // ここでは簡略化して接続成功とみなす
        tracing::info!("Connected to device {}", self.device_id);
        Ok(())
    }
    
    /// テーブルエントリを書き込み
    pub async fn write_table_entries(&mut self, entries: &[TableEntry]) -> Result<()> {
        for entry in entries {
            self.write_table_entry(entry).await?;
        }
        Ok(())
    }
    
    /// 単一のテーブルエントリを書き込み
    pub async fn write_table_entry(&mut self, entry: &TableEntry) -> Result<()> {
        // 実際のP4Runtimeでは、WriteRequestを送信
        // ここでは簡略化してログ出力
        tracing::info!(
            "Writing table entry: {} -> {:?}",
            entry.key.ipv4_dst,
            entry.action
        );
        Ok(())
    }
    
    /// テーブルエントリを削除
    pub async fn delete_table_entry(&mut self, key: &TableKey) -> Result<()> {
        tracing::info!("Deleting table entry: {}", key.ipv4_dst);
        Ok(())
    }
    
    /// テーブルエントリを読み取り
    pub async fn read_table_entries(&mut self) -> Result<Vec<TableEntry>> {
        // 実際のP4Runtimeでは、ReadRequestを送信
        // ここでは簡略化して空のベクターを返す
        Ok(Vec::new())
    }
    
    /// 統計情報を取得
    pub async fn get_statistics(&mut self) -> Result<Statistics> {
        // 実際のP4Runtimeでは、ReadRequestで統計情報を取得
        // ここでは簡略化してデフォルト値を返す
        Ok(Statistics::default())
    }
}

/// デバイスマネージャー
#[derive(Debug)]
pub struct DeviceManager {
    clients: Arc<RwLock<HashMap<DeviceId, P4RuntimeClient>>>,
    devices: Arc<RwLock<HashMap<DeviceId, DeviceInfo>>>,
}

impl DeviceManager {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            devices: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// デバイスを追加
    pub async fn add_device(&self, device_info: DeviceInfo) -> Result<()> {
        let device_id = device_info.device_id;
        let endpoint = device_info.grpc_endpoint.clone();
        
        // クライアントを作成
        let mut client = P4RuntimeClient::new(device_id, &endpoint).await?;
        client.connect().await?;
        
        // クライアントとデバイス情報を保存
        {
            let mut clients = self.clients.write().await;
            clients.insert(device_id, client);
        }
        
        {
            let mut devices = self.devices.write().await;
            devices.insert(device_id, device_info);
        }
        
        tracing::info!("Added device {} to manager", device_id);
        Ok(())
    }
    
    /// デバイスを削除
    pub async fn remove_device(&self, device_id: DeviceId) -> Result<()> {
        {
            let mut clients = self.clients.write().await;
            clients.remove(&device_id);
        }
        
        {
            let mut devices = self.devices.write().await;
            devices.remove(&device_id);
        }
        
        tracing::info!("Removed device {} from manager", device_id);
        Ok(())
    }
    
    /// デバイス一覧を取得
    pub async fn list_devices(&self) -> Vec<DeviceInfo> {
        let devices = self.devices.read().await;
        devices.values().cloned().collect()
    }
    
    /// 特定のデバイスにテーブルエントリを書き込み
    pub async fn write_table_entries_to_device(
        &self,
        device_id: DeviceId,
        entries: &[TableEntry],
    ) -> Result<()> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(&device_id) {
            client.write_table_entries(entries).await?;
        } else {
            return Err(P4RuntimeError::DeviceNotFound { device_id }.into());
        }
        Ok(())
    }
    
    /// 全デバイスにテーブルエントリを書き込み
    pub async fn write_table_entries_to_all_devices(
        &self,
        entries: &[TableEntry],
    ) -> Result<()> {
        let mut clients = self.clients.write().await;
        for (device_id, client) in clients.iter_mut() {
            if let Err(e) = client.write_table_entries(entries).await {
                tracing::error!("Failed to write entries to device {}: {}", device_id, e);
            }
        }
        Ok(())
    }
    
    /// デバイスから統計情報を取得
    pub async fn get_device_statistics(&self, device_id: DeviceId) -> Result<Statistics> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(&device_id) {
            client.get_statistics().await
        } else {
            Err(P4RuntimeError::DeviceNotFound { device_id }.into())
        }
    }
    
    /// 全デバイスの統計情報を取得
    pub async fn get_all_device_statistics(&self) -> HashMap<DeviceId, Statistics> {
        let mut clients = self.clients.write().await;
        let mut stats = HashMap::new();
        
        for (device_id, client) in clients.iter_mut() {
            match client.get_statistics().await {
                Ok(stat) => {
                    stats.insert(*device_id, stat);
                }
                Err(e) => {
                    tracing::error!("Failed to get statistics from device {}: {}", device_id, e);
                }
            }
        }
        
        stats
    }
}

impl Default for DeviceManager {
    fn default() -> Self {
        Self::new()
    }
}
