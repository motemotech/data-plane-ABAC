use crate::controller::P4Controller;
use crate::types::*;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::net::Ipv4Addr;
use std::str::FromStr;
use tracing::{info, error};

/// P4コントローラーのCLIアプリケーション
#[derive(Parser)]
#[command(name = "p4-controller")]
#[command(about = "A P4 Runtime Controller implemented in Rust")]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// デバイス管理コマンド
    Device {
        #[command(subcommand)]
        action: DeviceCommands,
    },
    /// ルーティング管理コマンド
    Route {
        #[command(subcommand)]
        action: RouteCommands,
    },
    /// ARP管理コマンド
    Arp {
        #[command(subcommand)]
        action: ArpCommands,
    },
    /// ポート管理コマンド
    Port {
        #[command(subcommand)]
        action: PortCommands,
    },
    /// 統計情報表示コマンド
    Stats,
    /// コントローラー状態表示コマンド
    Status,
}

#[derive(Subcommand)]
pub enum DeviceCommands {
    /// デバイスを追加
    Add {
        /// デバイスID
        #[arg(short, long)]
        device_id: u64,
        /// デバイス名
        #[arg(short, long)]
        name: String,
        /// gRPCエンドポイント
        #[arg(short, long)]
        endpoint: String,
    },
    /// デバイスを削除
    Remove {
        /// デバイスID
        #[arg(short, long)]
        device_id: u64,
    },
    /// デバイス一覧を表示
    List,
}

#[derive(Subcommand)]
pub enum RouteCommands {
    /// ルートを追加
    Add {
        /// プレフィックス (例: 192.168.1.0)
        #[arg(short, long)]
        prefix: String,
        /// プレフィックス長
        #[arg(short, long)]
        prefix_len: u8,
        /// ネクストホップ (例: 192.168.1.1)
        #[arg(short, long)]
        next_hop: Option<String>,
        /// インターフェース名
        #[arg(short, long)]
        interface: String,
        /// メトリック
        #[arg(short, long, default_value = "1")]
        metric: u32,
    },
    /// ルートを削除
    Remove {
        /// プレフィックス (例: 192.168.1.0)
        #[arg(short, long)]
        prefix: String,
        /// プレフィックス長
        #[arg(short, long)]
        prefix_len: u8,
    },
    /// ルート一覧を表示
    List,
    /// ルートを検索
    Lookup {
        /// 検索するIPアドレス
        #[arg(short, long)]
        ip: String,
    },
}

#[derive(Subcommand)]
pub enum ArpCommands {
    /// ARPエントリを追加
    Add {
        /// IPアドレス
        #[arg(short, long)]
        ip: String,
        /// MACアドレス (例: 00:11:22:33:44:55)
        #[arg(short, long)]
        mac: String,
        /// インターフェース名
        #[arg(short, long)]
        interface: String,
    },
    /// ARPエントリを削除
    Remove {
        /// IPアドレス
        #[arg(short, long)]
        ip: String,
    },
    /// ARPエントリ一覧を表示
    List,
    /// ARPエントリを検索
    Lookup {
        /// IPアドレス
        #[arg(short, long)]
        ip: String,
    },
}

#[derive(Subcommand)]
pub enum PortCommands {
    /// ポートを追加
    Add {
        /// ポートID
        #[arg(short, long)]
        port_id: u32,
        /// ポート名
        #[arg(short, long)]
        name: String,
        /// MACアドレス (例: 00:11:22:33:44:55)
        #[arg(short, long)]
        mac: String,
        /// IPアドレス (例: 192.168.1.10)
        #[arg(short, long)]
        ip: Option<String>,
    },
    /// ポートを削除
    Remove {
        /// ポートID
        #[arg(short, long)]
        port_id: u32,
    },
    /// ポート一覧を表示
    List,
    /// ポートの状態を更新
    Update {
        /// ポートID
        #[arg(short, long)]
        port_id: u32,
        /// 状態 (up/down)
        #[arg(short, long)]
        status: String,
    },
}

/// CLIハンドラー
pub struct CliHandler {
    controller: P4Controller,
}

impl CliHandler {
    pub fn new() -> Self {
        Self {
            controller: P4Controller::new(),
        }
    }
    
    /// CLIコマンドを実行
    pub async fn run(&self, cli: Cli) -> Result<()> {
        // コントローラーを初期化
        self.controller.initialize().await?;
        
        match cli.command {
            Commands::Device { action } => {
                self.handle_device_command(action).await?;
            }
            Commands::Route { action } => {
                self.handle_route_command(action).await?;
            }
            Commands::Arp { action } => {
                self.handle_arp_command(action).await?;
            }
            Commands::Port { action } => {
                self.handle_port_command(action).await?;
            }
            Commands::Stats => {
                self.show_statistics().await?;
            }
            Commands::Status => {
                self.show_status().await?;
            }
        }
        
        Ok(())
    }
    
    /// デバイスコマンドを処理
    async fn handle_device_command(&self, action: DeviceCommands) -> Result<()> {
        match action {
            DeviceCommands::Add { device_id, name, endpoint } => {
                let device_info = DeviceInfo {
                    device_id,
                    name,
                    grpc_endpoint: endpoint,
                    p4info: None,
                };
                
                self.controller.add_device(device_info).await?;
                info!("Device added successfully");
            }
            DeviceCommands::Remove { device_id } => {
                self.controller.remove_device(device_id).await?;
                info!("Device removed successfully");
            }
            DeviceCommands::List => {
                let devices = self.controller.list_devices().await;
                println!("Connected Devices:");
                println!("{:<10} {:<20} {:<30}", "ID", "Name", "Endpoint");
                println!("{}", "-".repeat(60));
                
                for device in devices {
                    println!("{:<10} {:<20} {:<30}", 
                        device.device_id, 
                        device.name, 
                        device.grpc_endpoint
                    );
                }
            }
        }
        Ok(())
    }
    
    /// ルートコマンドを処理
    async fn handle_route_command(&self, action: RouteCommands) -> Result<()> {
        match action {
            RouteCommands::Add { prefix, prefix_len, next_hop, interface, metric } => {
                let prefix_ip = Ipv4Addr::from_str(&prefix)?;
                let next_hop_ip = if let Some(nh) = next_hop {
                    Some(Ipv4Addr::from_str(&nh)?)
                } else {
                    None
                };
                
                let route = RouteEntry {
                    prefix: Ipv4Address::new(prefix_ip),
                    prefix_len,
                    next_hop: next_hop_ip.map(Ipv4Address::new),
                    interface,
                    metric,
                };
                
                self.controller.add_route(route).await?;
                info!("Route added successfully");
            }
            RouteCommands::Remove { prefix, prefix_len } => {
                let prefix_ip = Ipv4Addr::from_str(&prefix)?;
                self.controller.remove_route(Ipv4Address::new(prefix_ip), prefix_len).await?;
                info!("Route removed successfully");
            }
            RouteCommands::List => {
                let routes = self.controller.list_routes().await;
                println!("Routing Table:");
                println!("{:<18} {:<4} {:<15} {:<10} {:<8}", "Prefix", "Len", "Next Hop", "Interface", "Metric");
                println!("{}", "-".repeat(65));
                
                for route in routes {
                    let next_hop_str = route.next_hop.map(|nh| nh.to_string()).unwrap_or_else(|| "direct".to_string());
                    println!("{:<18} {:<4} {:<15} {:<10} {:<8}", 
                        route.prefix, 
                        route.prefix_len, 
                        next_hop_str,
                        route.interface,
                        route.metric
                    );
                }
            }
            RouteCommands::Lookup { ip } => {
                let lookup_ip = Ipv4Addr::from_str(&ip)?;
                let routes = self.controller.list_routes().await;
                
                println!("Route lookup for {}:", ip);
                println!("{:<18} {:<4} {:<15} {:<10} {:<8}", "Prefix", "Len", "Next Hop", "Interface", "Metric");
                println!("{}", "-".repeat(65));
                
                for route in routes {
                    let prefix_ip = route.prefix.as_ipv4();
                    let prefix_len = route.prefix_len;
                    let lookup_ip_u32: u32 = lookup_ip.into();
                    let prefix_u32: u32 = prefix_ip.into();
                    
                    // プレフィックスマッチをチェック
                    let mask = if prefix_len == 0 {
                        0
                    } else {
                        !((1u32 << (32 - prefix_len)) - 1)
                    };
                    
                    if (prefix_u32 & mask) == (lookup_ip_u32 & mask) {
                        let next_hop_str = route.next_hop.map(|nh| nh.to_string()).unwrap_or_else(|| "direct".to_string());
                        println!("{:<18} {:<4} {:<15} {:<10} {:<8}", 
                            route.prefix, 
                            route.prefix_len, 
                            next_hop_str,
                            route.interface,
                            route.metric
                        );
                    }
                }
            }
        }
        Ok(())
    }
    
    /// ARPコマンドを処理
    async fn handle_arp_command(&self, action: ArpCommands) -> Result<()> {
        match action {
            ArpCommands::Add { ip, mac, interface } => {
                let ip_addr = Ipv4Addr::from_str(&ip)?;
                let mac_bytes = parse_mac_address(&mac)?;
                
                let arp_entry = ArpEntry {
                    ip: Ipv4Address::new(ip_addr),
                    mac: MacAddress::new(mac_bytes),
                    interface,
                };
                
                self.controller.add_arp_entry(arp_entry).await?;
                info!("ARP entry added successfully");
            }
            ArpCommands::Remove { ip } => {
                let _ip_addr = Ipv4Addr::from_str(&ip)?;
                // ARPエントリの削除は直接ルーティングマネージャーから行う
                // self.controller.remove_arp_entry(Ipv4Address::new(ip_addr)).await?;
                info!("ARP entry removal not implemented yet");
            }
            ArpCommands::List => {
                let arp_entries = self.controller.list_arp_entries().await;
                println!("ARP Table:");
                println!("{:<15} {:<17} {:<10}", "IP Address", "MAC Address", "Interface");
                println!("{}", "-".repeat(42));
                
                for entry in arp_entries {
                    println!("{:<15} {:<17} {:<10}", 
                        entry.ip, 
                        entry.mac, 
                        entry.interface
                    );
                }
            }
            ArpCommands::Lookup { ip } => {
                let ip_addr = Ipv4Addr::from_str(&ip)?;
                let arp_entries = self.controller.list_arp_entries().await;
                
                println!("ARP lookup for {}:", ip);
                println!("{:<15} {:<17} {:<10}", "IP Address", "MAC Address", "Interface");
                println!("{}", "-".repeat(42));
                
                for entry in arp_entries {
                    if entry.ip.as_ipv4() == ip_addr {
                        println!("{:<15} {:<17} {:<10}", 
                            entry.ip, 
                            entry.mac, 
                            entry.interface
                        );
                        return Ok(());
                    }
                }
                
                println!("No ARP entry found for {}", ip);
            }
        }
        Ok(())
    }
    
    /// ポートコマンドを処理
    async fn handle_port_command(&self, action: PortCommands) -> Result<()> {
        match action {
            PortCommands::Add { port_id, name, mac, ip } => {
                let mac_bytes = parse_mac_address(&mac)?;
                let ip_addr = if let Some(ip_str) = ip {
                    Some(Ipv4Addr::from_str(&ip_str)?)
                } else {
                    None
                };
                
                let port = PortInfo {
                    port_id,
                    name,
                    mac_address: MacAddress::new(mac_bytes),
                    ip_address: ip_addr.map(Ipv4Address::new),
                    is_up: true,
                };
                
                self.controller.add_port(port).await?;
                info!("Port added successfully");
            }
            PortCommands::Remove { port_id: _port_id } => {
                // ポートの削除は直接ルーティングマネージャーから行う
                // self.controller.remove_port(port_id).await?;
                info!("Port removal not implemented yet");
            }
            PortCommands::List => {
                let ports = self.controller.list_ports().await;
                println!("Port Table:");
                println!("{:<8} {:<15} {:<17} {:<15} {:<6}", "Port ID", "Name", "MAC Address", "IP Address", "Status");
                println!("{}", "-".repeat(71));
                
                for port in ports {
                    let ip_str = port.ip_address.map(|ip| ip.to_string()).unwrap_or_else(|| "N/A".to_string());
                    let status = if port.is_up { "UP" } else { "DOWN" };
                    
                    println!("{:<8} {:<15} {:<17} {:<15} {:<6}", 
                        port.port_id, 
                        port.name, 
                        port.mac_address, 
                        ip_str,
                        status
                    );
                }
            }
            PortCommands::Update { port_id, status } => {
                let is_up = match status.to_lowercase().as_str() {
                    "up" => true,
                    "down" => false,
                    _ => {
                        error!("Invalid status: {}. Use 'up' or 'down'", status);
                        return Ok(());
                    }
                };
                
                self.controller.update_port_status(port_id, is_up).await?;
                info!("Port {} status updated to {}", port_id, status);
            }
        }
        Ok(())
    }
    
    /// 統計情報を表示
    async fn show_statistics(&self) -> Result<()> {
        let stats = self.controller.get_statistics().await?;
        
        println!("Device Statistics:");
        println!("{}", "-".repeat(50));
        
        for (device_id, stat) in stats {
            println!("Device {}:", device_id);
            println!("  Packets processed: {}", stat.packets_processed);
            println!("  Bytes processed: {}", stat.bytes_processed);
            println!("  Table hits:");
            for (table_name, hits) in &stat.table_hits {
                println!("    {}: {}", table_name, hits);
            }
            println!("  Table misses:");
            for (table_name, misses) in &stat.table_misses {
                println!("    {}: {}", table_name, misses);
            }
            println!();
        }
        
        Ok(())
    }
    
    /// コントローラー状態を表示
    async fn show_status(&self) -> Result<()> {
        let state = self.controller.get_state().await;
        
        println!("Controller Status:");
        println!("{}", "-".repeat(50));
        println!("Connected devices: {}", state.connected_devices.len());
        println!("Total packets processed: {}", state.statistics.packets_processed);
        println!("Total bytes processed: {}", state.statistics.bytes_processed);
        
        Ok(())
    }
}

impl Default for CliHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// MACアドレス文字列をパース
fn parse_mac_address(mac_str: &str) -> Result<[u8; 6]> {
    let parts: Vec<&str> = mac_str.split(':').collect();
    if parts.len() != 6 {
        return Err(anyhow::anyhow!("Invalid MAC address format"));
    }
    
    let mut bytes = [0u8; 6];
    for (i, part) in parts.iter().enumerate() {
        bytes[i] = u8::from_str_radix(part, 16)?;
    }
    
    Ok(bytes)
}
