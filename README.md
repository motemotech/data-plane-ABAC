# P4 IP Forwarding Project

このプロジェクトは、P4プログラミング言語を使用してIP転送機能を実装し、コントロールプレーンでルーティングテーブルを管理するシステムです。

## プロジェクト構成

```
p4prj/
├── src/                    # P4ソースコード
│   └── ip_forwarding.p4   # IP転送用P4プログラム
├── control_plane/          # コントロールプレーン
│   ├── controller.py      # メインコントローラー
│   ├── routing_table.json # ルーティングテーブル設定
│   └── cli_commands.txt   # BMv2 CLIコマンド（自動生成）
├── tests/                  # テストスクリプト
│   └── test_forwarding.py # IP転送テスト
├── build/                  # ビルド出力
├── logs/                   # ログファイル
├── Makefile               # ビルドシステム
├── requirements.txt       # Python依存関係
└── README.md              # このファイル
```

## 機能

### P4データプレーン
- **IP転送**: IPv4パケットのLPM（Longest Prefix Match）ルーティング
- **TTLデクリメント**: IPヘッダーのTTLフィールドを自動的にデクリメント
- **チェックサム更新**: IPヘッダーチェックサムの自動再計算
- **MACアドレス書き換え**: 転送時にイーサネットヘッダーを更新

### コントロールプレーン
- **ルーティングテーブル管理**: JSONファイルからルート情報を読み込み
- **動的ルート追加/削除**: 実行時にルーティングテーブルを更新
- **複数の制御方法**: P4RuntimeとBMv2 CLIの両方をサポート
- **設定の永続化**: ルーティングテーブルをJSONファイルに保存

## 必要な環境

### システム要件
- Ubuntu 18.04+ または Debian 10+
- Python 3.6+
- P4コンパイラ (p4c)
- BMv2 (behavioral-model-v2)
- Scapy (パケット生成・解析用)

### インストール

```bash
# 依存関係をインストール
make install-deps

# または手動でインストール
sudo apt-get update
sudo apt-get install -y p4c bmv2 python3-pip
pip3 install -r requirements.txt
```

## 使用方法

### 1. プロジェクトのビルド

```bash
# P4プログラムをコンパイル
make compile
```

### 2. 仮想インターフェースの設定

```bash
# テスト用の仮想インターフェースを作成
make setup-interfaces
```

### 3. P4スイッチの起動

```bash
# BMv2スイッチを起動
make run
```

### 4. ルーティングテーブルの設定

```bash
# コントロールプレーンを実行
make control
```

または手動でBMv2 CLIを使用：

```bash
simple_switch_CLI < control_plane/cli_commands.txt
```

### 5. テストの実行

```bash
# IP転送テストを実行
make test
```

### 6. クリーンアップ

```bash
# スイッチを停止
make stop

# 仮想インターフェースを削除
make clean-interfaces

# ビルドファイルを削除
make clean
```

## 設定

### ルーティングテーブルの編集

`control_plane/routing_table.json`を編集してルーティングテーブルを設定：

```json
{
  "routes": {
    "192.168.1.0/24": {
      "port": 1,
      "mac": "02:00:00:00:00:01"
    },
    "192.168.2.0/24": {
      "port": 2,
      "mac": "02:00:00:00:00:02"
    }
  }
}
```

### コントロールプレーンのオプション

```bash
# P4Runtimeを使用
python3 control_plane/controller.py --method p4runtime

# カスタムスイッチアドレス
python3 control_plane/controller.py --switch-addr 192.168.1.100

# カスタムルーティングテーブルファイル
python3 control_plane/controller.py --routing-table my_routes.json
```

## テスト

### 手動テスト

```bash
# IP転送テスト
python3 tests/test_forwarding.py --test forwarding

# TTLデクリメントテスト
python3 tests/test_forwarding.py --test ttl

# 全テスト
python3 tests/test_forwarding.py --test all
```

### パケットキャプチャ

```bash
# パケットをキャプチャして転送を確認
sudo tcpdump -i veth1 -n
sudo tcpdump -i veth3 -n
```

## トラブルシューティング

### よくある問題

1. **スイッチが起動しない**
   - P4プログラムのコンパイルエラーを確認
   - ポートが既に使用されていないか確認

2. **ルートがインストールされない**
   - BMv2 CLIコマンドの構文を確認
   - スイッチが起動しているか確認

3. **パケットが転送されない**
   - ルーティングテーブルが正しく設定されているか確認
   - 仮想インターフェースが正しく設定されているか確認

### ログの確認

```bash
# スイッチのログを確認
tail -f logs/switch.log

# BMv2 CLIの出力を確認
simple_switch_CLI
```

## 拡張

### 新しい機能の追加

1. **新しいテーブル**: P4プログラムにテーブルを追加
2. **新しいアクション**: カスタム転送ロジックを実装
3. **新しいヘッダー**: 追加のプロトコルヘッダーを処理
4. **統計情報**: パケットカウンターやバイトカウンターを追加

### コントロールプレーンの拡張

1. **動的ルーティング**: OSPFやBGPプロトコルの実装
2. **負荷分散**: 複数のパス間での負荷分散
3. **QoS**: サービス品質の制御
4. **セキュリティ**: ACL（Access Control List）の実装

## ライセンス

このプロジェクトはMITライセンスの下で公開されています。

## 貢献

バグレポートや機能要求は、GitHubのIssuesページでお知らせください。
Pull Requestも歓迎します。

## 参考資料

- [P4 Language Specification](https://p4.org/p4-spec/)
- [BMv2 Documentation](https://github.com/p4lang/behavioral-model)
- [P4Runtime Documentation](https://p4.org/p4runtime/)
