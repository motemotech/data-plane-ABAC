# P4 Controller (Rust Implementation)

このプロジェクトは、P4Runtimeプロトコルを使用してP4スイッチを制御するRustベースのコントロールプレーン実装です。

## 機能

- **デバイス管理**: P4スイッチの接続と管理
- **ルーティングテーブル管理**: IPv4ルーティングテーブルの管理
- **ARPテーブル管理**: ARPエントリの管理
- **ポート管理**: スイッチポートの状態管理
- **統計情報**: パケット処理統計の取得
- **CLIインターフェース**: コマンドラインからの操作

## アーキテクチャ

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   CLI Handler   │────│   Controller    │────│ Device Manager  │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                              │
                       ┌──────┴──────┐
                       │             │
                ┌──────▼──────┐ ┌────▼──────┐
                │Table Manager│ │Routing    │
                │             │ │Manager    │
                └─────────────┘ └───────────┘
```

## ビルドと実行

### 前提条件

- Rust 1.70以上
- Cargo

### ビルド

```bash
cargo build --release
```

### 実行

```bash
cargo run -- --help
```

## 使用方法

### デバイス管理

#### デバイスを追加
```bash
cargo run -- device add --device-id 1 --name "switch1" --endpoint "127.0.0.1:50051"
```

#### デバイス一覧を表示
```bash
cargo run -- device list
```

#### デバイスを削除
```bash
cargo run -- device remove --device-id 1
```

### ルーティング管理

#### ルートを追加
```bash
cargo run -- route add --prefix "192.168.1.0" --prefix-len 24 --next-hop "192.168.1.1" --interface "eth0"
```

#### ルート一覧を表示
```bash
cargo run -- route list
```

#### ルートを検索
```bash
cargo run -- route lookup --ip "192.168.1.100"
```

#### ルートを削除
```bash
cargo run -- route remove --prefix "192.168.1.0" --prefix-len 24
```

### ARP管理

#### ARPエントリを追加
```bash
cargo run -- arp add --ip "192.168.1.1" --mac "00:11:22:33:44:55" --interface "eth0"
```

#### ARPエントリ一覧を表示
```bash
cargo run -- arp list
```

#### ARPエントリを検索
```bash
cargo run -- arp lookup --ip "192.168.1.1"
```

### ポート管理

#### ポートを追加
```bash
cargo run -- port add --port-id 1 --name "eth0" --mac "00:11:22:33:44:55" --ip "192.168.1.10"
```

#### ポート一覧を表示
```bash
cargo run -- port list
```

#### ポートの状態を更新
```bash
cargo run -- port update --port-id 1 --status "up"
```

### 統計情報と状態

#### 統計情報を表示
```bash
cargo run -- stats
```

#### コントローラー状態を表示
```bash
cargo run -- status
```

## 設定

デフォルトでは、以下の設定が自動的に適用されます：

- デフォルトゲートウェイルート: `0.0.0.0/0` → `192.168.1.1`
- ローカルネットワークルート: `192.168.1.0/24` → 直接接続
- デフォルトゲートウェイのARPエントリ: `192.168.1.1` → `00:11:22:33:44:55`

## 実装の詳細

### 型定義 (`types.rs`)

- `DeviceInfo`: デバイス情報
- `RouteEntry`: ルーティングテーブルエントリ
- `ArpEntry`: ARPテーブルエントリ
- `PortInfo`: ポート情報
- `TableEntry`: P4テーブルエントリ
- `Statistics`: 統計情報

### P4Runtimeクライアント (`p4runtime_client.rs`)

- `P4RuntimeClient`: gRPCクライアント
- `DeviceManager`: デバイス管理

### テーブル管理 (`table_manager.rs`)

- `TableManager`: P4テーブルエントリの管理
- `TableEntryBuilder`: テーブルエントリのビルダー

### ルーティング管理 (`routing_manager.rs`)

- `RoutingManager`: ルーティングテーブルとARPテーブルの管理
- `RouteBuilder`: ルートエントリのビルダー

### コントローラー (`controller.rs`)

- `P4Controller`: メインコントローラーアプリケーション

### CLI (`cli.rs`)

- `Cli`: コマンドライン引数の定義
- `CliHandler`: CLIコマンドの処理

## 拡張性

この実装は以下の点で拡張可能です：

1. **新しいテーブルタイプの追加**: `TableManager`を拡張
2. **新しいプロトコルのサポート**: `types.rs`に新しい型を追加
3. **設定ファイルのサポート**: JSONやYAMLファイルからの設定読み込み
4. **REST API**: HTTPサーバーの追加
5. **イベント通知**: デバイス状態変更の通知機能

## 注意事項

- この実装は教育目的のサンプルです
- 実際のP4Runtimeプロトコルとの完全な互換性は保証されません
- 本番環境での使用前に十分なテストを行ってください

## ライセンス

このプロジェクトはMITライセンスの下で公開されています。
