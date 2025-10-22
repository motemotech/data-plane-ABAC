#!/usr/bin/env python3
"""
P4 IP Forwarding Control Plane
IPアドレス転送表を生成し、P4スイッチに設定するコントロールプレーン
"""

import json
import socket
import struct
import time
from typing import Dict, List, Tuple
import argparse

try:
    from p4runtime_lib import helper
    from p4runtime_lib.switch import ShutdownAllSwitchConnections
    from p4runtime_lib.convert import encodeNum, decodeNum
    P4RUNTIME_AVAILABLE = True
except ImportError:
    print("Warning: p4runtime_lib not available. Using BMv2 CLI instead.")
    P4RUNTIME_AVAILABLE = False
    import subprocess

class IPForwardingController:
    def __init__(self, switch_address: str = "127.0.0.1", switch_port: int = 50051):
        self.switch_address = switch_address
        self.switch_port = switch_port
        self.routing_table = {}
        self.mac_table = {}
        
    def load_routing_table(self, filename: str):
        """JSONファイルからルーティングテーブルを読み込み"""
        try:
            with open(filename, 'r') as f:
                data = json.load(f)
                self.routing_table = data.get('routes', {})
                self.mac_table = data.get('mac_addresses', {})
                print(f"Loaded routing table from {filename}")
                print(f"Routes: {len(self.routing_table)}")
                print(f"MAC addresses: {len(self.mac_table)}")
        except FileNotFoundError:
            print(f"Warning: {filename} not found. Using default routing table.")
            self._create_default_routing_table()
    
    def _create_default_routing_table(self):
        """デフォルトのルーティングテーブルを作成"""
        # 例: 192.168.1.0/24 -> port 1, MAC 02:00:00:00:00:01
        self.routing_table = {
            "192.168.1.0/24": {
                "port": 1,
                "mac": "02:00:00:00:00:01"
            },
            "192.168.2.0/24": {
                "port": 2,
                "mac": "02:00:00:00:00:02"
            },
            "10.0.0.0/8": {
                "port": 3,
                "mac": "02:00:00:00:00:03"
            }
        }
        print("Created default routing table")
    
    def ip_to_int(self, ip: str) -> int:
        """IPアドレス文字列を整数に変換"""
        return struct.unpack("!I", socket.inet_aton(ip))[0]
    
    def int_to_ip(self, ip_int: int) -> str:
        """整数をIPアドレス文字列に変換"""
        return socket.inet_ntoa(struct.pack("!I", ip_int))
    
    def mac_to_bytes(self, mac: str) -> bytes:
        """MACアドレス文字列をバイト列に変換"""
        return bytes.fromhex(mac.replace(':', ''))
    
    def install_routes_p4runtime(self):
        """P4Runtimeを使用してルートをインストール"""
        if not P4RUNTIME_AVAILABLE:
            print("P4Runtime not available, using BMv2 CLI")
            self.install_routes_bmv2_cli()
            return
            
        try:
            # P4Runtime接続を確立
            p4info_helper = helper.P4InfoHelper("build/ip_forwarding.p4info.txt")
            sw = helper.SimpleSwitchConnection(
                device_id=0,
                grpc_addr=f"{self.switch_address}:{self.switch_port}",
                p4info_helper=p4info_helper
            )
            
            # ルーティングテーブルエントリを追加
            for network, info in self.routing_table.items():
                ip, prefix_len = network.split('/')
                prefix_len = int(prefix_len)
                
                # LPMテーブルエントリを作成
                table_entry = p4info_helper.buildTableEntry(
                    table_name="MyIngress.ipv4_lpm",
                    match_fields={
                        "hdr.ipv4.dstAddr": (self.ip_to_int(ip), prefix_len)
                    },
                    action_name="MyIngress.ipv4_forward",
                    action_params={
                        "dstAddr": self.mac_to_bytes(info["mac"]),
                        "port": info["port"]
                    }
                )
                
                sw.WriteTableEntry(table_entry)
                print(f"Installed route: {network} -> port {info['port']}, MAC {info['mac']}")
            
            print("All routes installed successfully")
            
        except Exception as e:
            print(f"Error installing routes with P4Runtime: {e}")
            print("Falling back to BMv2 CLI")
            self.install_routes_bmv2_cli()
    
    def install_routes_bmv2_cli(self):
        """BMv2 CLIを使用してルートをインストール"""
        cli_commands = []
        
        for network, info in self.routing_table.items():
            ip, prefix_len = network.split('/')
            prefix_len = int(prefix_len)
            
            # BMv2 CLIコマンドを生成
            cmd = f"table_add MyIngress.ipv4_lpm MyIngress.ipv4_forward {ip}/{prefix_len} => {info['mac']} {info['port']}"
            cli_commands.append(cmd)
        
        # CLIコマンドをファイルに書き込み
        with open("control_plane/cli_commands.txt", "w") as f:
            for cmd in cli_commands:
                f.write(cmd + "\n")
        
        print("CLI commands written to control_plane/cli_commands.txt")
        print("Run: simple_switch_CLI < control_plane/cli_commands.txt")
    
    def generate_routing_table_json(self, filename: str = "control_plane/routing_table.json"):
        """ルーティングテーブルをJSONファイルに出力"""
        data = {
            "routes": self.routing_table,
            "mac_addresses": self.mac_table,
            "generated_at": time.strftime("%Y-%m-%d %H:%M:%S")
        }
        
        with open(filename, 'w') as f:
            json.dump(data, f, indent=2)
        
        print(f"Routing table saved to {filename}")
    
    def add_route(self, network: str, port: int, mac: str):
        """新しいルートを追加"""
        self.routing_table[network] = {
            "port": port,
            "mac": mac
        }
        print(f"Added route: {network} -> port {port}, MAC {mac}")
    
    def remove_route(self, network: str):
        """ルートを削除"""
        if network in self.routing_table:
            del self.routing_table[network]
            print(f"Removed route: {network}")
        else:
            print(f"Route not found: {network}")
    
    def show_routes(self):
        """ルーティングテーブルを表示"""
        print("\n=== Routing Table ===")
        for network, info in self.routing_table.items():
            print(f"{network:15} -> port {info['port']}, MAC {info['mac']}")
        print("====================\n")

def main():
    parser = argparse.ArgumentParser(description='P4 IP Forwarding Control Plane')
    parser.add_argument('--switch-addr', default='127.0.0.1', help='Switch address')
    parser.add_argument('--switch-port', type=int, default=50051, help='Switch port')
    parser.add_argument('--routing-table', default='control_plane/routing_table.json', 
                       help='Routing table JSON file')
    parser.add_argument('--method', choices=['p4runtime', 'bmv2-cli'], default='bmv2-cli',
                       help='Control plane method')
    
    args = parser.parse_args()
    
    # コントローラーを作成
    controller = IPForwardingController(args.switch_addr, args.switch_port)
    
    # ルーティングテーブルを読み込み
    controller.load_routing_table(args.routing_table)
    
    # ルートを表示
    controller.show_routes()
    
    # ルートをインストール
    if args.method == 'p4runtime':
        controller.install_routes_p4runtime()
    else:
        controller.install_routes_bmv2_cli()
    
    # JSONファイルに保存
    controller.generate_routing_table_json()

if __name__ == "__main__":
    main()
