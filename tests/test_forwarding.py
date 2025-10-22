#!/usr/bin/env python3
"""
P4 IP Forwarding Test Script
P4スイッチのIP転送機能をテストするスクリプト
"""

import socket
import struct
import time
import argparse
from scapy.all import *

def create_ethernet_frame(src_mac, dst_mac, ethertype=0x0800):
    """イーサネットフレームを作成"""
    return Ether(src=src_mac, dst=dst_mac, type=ethertype)

def create_ip_packet(src_ip, dst_ip, payload="Hello P4!"):
    """IPパケットを作成"""
    return IP(src=src_ip, dst=dst_ip, ttl=64) / Raw(payload)

def create_test_packet(src_mac, dst_mac, src_ip, dst_ip, payload="Hello P4!"):
    """テスト用の完全なパケットを作成"""
    eth = create_ethernet_frame(src_mac, dst_mac)
    ip = create_ip_packet(src_ip, dst_ip, payload)
    return eth / ip

def send_packet(interface, packet):
    """パケットを送信"""
    try:
        sendp(packet, iface=interface, verbose=False)
        print(f"Sent packet: {packet[IP].src} -> {packet[IP].dst}")
        return True
    except Exception as e:
        print(f"Error sending packet: {e}")
        return False

def test_ip_forwarding():
    """IP転送のテストを実行"""
    print("=== P4 IP Forwarding Test ===")
    
    # テストケース
    test_cases = [
        {
            "name": "192.168.1.0/24 network",
            "src_mac": "00:00:00:00:00:01",
            "dst_mac": "08:00:00:00:00:01",  # Switch MAC
            "src_ip": "192.168.1.10",
            "dst_ip": "192.168.1.20",
            "expected_port": 1
        },
        {
            "name": "192.168.2.0/24 network",
            "src_mac": "00:00:00:00:00:02",
            "dst_mac": "08:00:00:00:00:01",
            "src_ip": "192.168.2.10",
            "dst_ip": "192.168.2.20",
            "expected_port": 2
        },
        {
            "name": "10.0.0.0/8 network",
            "src_mac": "00:00:00:00:00:03",
            "dst_mac": "08:00:00:00:00:01",
            "src_ip": "10.0.0.10",
            "dst_ip": "10.0.0.20",
            "expected_port": 3
        }
    ]
    
    for i, test_case in enumerate(test_cases, 1):
        print(f"\nTest {i}: {test_case['name']}")
        
        # パケットを作成
        packet = create_test_packet(
            test_case['src_mac'],
            test_case['dst_mac'],
            test_case['src_ip'],
            test_case['dst_ip'],
            f"Test packet {i}"
        )
        
        # パケットを送信
        if send_packet("veth1", packet):
            print(f"  Expected forwarding to port {test_case['expected_port']}")
            print(f"  Packet: {test_case['src_ip']} -> {test_case['dst_ip']}")
        
        time.sleep(1)  # パケット間の間隔

def test_ttl_decrement():
    """TTLデクリメントのテスト"""
    print("\n=== TTL Decrement Test ===")
    
    packet = create_test_packet(
        "00:00:00:00:00:01",
        "08:00:00:00:00:01",
        "192.168.1.10",
        "192.168.1.20",
        "TTL test"
    )
    
    # TTLを64に設定
    packet[IP].ttl = 64
    
    print(f"Original TTL: {packet[IP].ttl}")
    print("Sending packet...")
    
    if send_packet("veth1", packet):
        print("Packet sent. Check if TTL was decremented by 1.")
        print("Expected TTL after forwarding: 63")

def main():
    parser = argparse.ArgumentParser(description='P4 IP Forwarding Test')
    parser.add_argument('--interface', default='veth1', help='Network interface to use')
    parser.add_argument('--test', choices=['forwarding', 'ttl', 'all'], default='all',
                       help='Test to run')
    
    args = parser.parse_args()
    
    print(f"Using interface: {args.interface}")
    print("Make sure the P4 switch is running and routes are installed.")
    print("Press Ctrl+C to stop the test.\n")
    
    try:
        if args.test in ['forwarding', 'all']:
            test_ip_forwarding()
        
        if args.test in ['ttl', 'all']:
            test_ttl_decrement()
            
    except KeyboardInterrupt:
        print("\nTest stopped by user.")
    except Exception as e:
        print(f"Test error: {e}")

if __name__ == "__main__":
    main()
