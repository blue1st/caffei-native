# ☕ Caffei Native

**Caffei Native** は、特定のアプリケーションの動作に合わせてMacのスリープを自動的に抑制する、モダンなmacOS用ユーティリティです。macOS標準の `caffeinate` コマンドを、直感的なGUIとスマートなプロセス監視機能でより使いやすくしました。

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Platform](https://img.shields.io/badge/platform-macOS-lightgrey.svg)
![Tauri](https://img.shields.io/badge/built%20with-Tauri-red.svg)

## ✨ 主な機能

- **📂 ネイティブなアプリ選択**: macOS標準のファイルピッカーを使用して、システム上のアプリケーション（`.app` バンドル）を直接選択できます。
- **🔍 スマートなプロセス監視**: 監視リストに登録したアプリが起動している間、自動的にスリープ抑制を開始します。
- **☕ 状況がわかるシステムトレイ**: 
  - **動的アイコン**: スリープ抑制が有効なときは「湯気の立つコーヒーカップ」にアイコンが変化します。
  - **詳細メニュー**: メニューバーから現在どのアプリを検知して動作しているか一目で確認できます。
- **🎨 モダンなUI**: React と Vanilla CSS を使用した、シンプルで美しいインターフェース。
- **🚀 軽量 & ネイティブ**: Rust と Tauri を採用し、システムリソースを最小限に抑えつつネイティブに近い動作を実現。

## 🚀 使い方

### 動作要件

- macOS (11.0 以降を推奨)
- [Rust](https://www.rust-lang.org/tools/install) (開発・ビルド時)
- [Node.js](https://nodejs.org/) (開発・ビルド時)

### インストール・実行

#### Homebrew (macOS)

[blue1st/homebrew-taps](https://github.com/blue1st/homebrew-taps) を利用して、以下のコマンドでインストールできます：

```bash
brew install --cask blue1st/taps/caffei-native
```

#### ソースからビルド

1. リポジトリをクローン:
   ```bash
   git clone https://github.com/blue1st/caffei-native.git
   cd caffei-native
   ```

2. 依存関係のインストール:
   ```bash
   npm install
   ```

3. 開発モードで実行:
   ```bash
   npm run tauri dev
   ```

   ```bash
   npm run tauri build
   ```

#### 異なるアーキテクチャ向けのビルド (Intel Mac / Universal)

Apple Silicon MacからIntel Mac向けのバイナリや、両方に対応した Universal Binary をビルドするには、まず Rust のターゲットを追加する必要があります：

```bash
# Intel Mac 用ターゲットの追加
rustup target add x86_64-apple-darwin
# Apple Silicon 用ターゲットの追加 (必要に応じて)
rustup target add aarch64-apple-darwin
```

その後、以下のコマンドを実行します：

```bash
# Intel Mac (x86_64) 用のビルド
npm run build:macos-x86_64

# Universal Binary (Intel & Apple Silicon 両対応) のビルド
npm run build:macos-universal
```

### トラブルシューティング: 「壊れているため開けません」と表示される場合

GitHub Releases からダウンロードした `.app` や `.dmg` を開く際に、「“Caffei Native”は壊れているため開けません。ゴミ箱に入れる必要があります。」という警告が出ることがあります。
これは、macOS のGatekeeperが未署名のアプリケーションをブロックする仕様によるものです。

**解決方法:**
1. ダウンロードした `Caffei Native.app` を「アプリケーション」フォルダ (`/Applications`) に配置します。
2. ターミナルを開き、以下のコマンドを実行して隔離属性 (Quarantine) を解除します。
   ```bash
   xattr -cr "/Applications/Caffei Native.app"
   ```
3. その後、通常通りダブルクリックで起動できるようになります。


## 🛠️ 技術スタック

- **フレームワーク**: [Tauri v2](https://tauri.app/)
- **フロントエンド**: [React](https://reactjs.org/), [TypeScript](https://www.typescriptlang.org/)
- **バックエンド**: [Rust](https://www.rust-lang.org/)
- **スタイリング**: Vanilla CSS (独自のデザインシステム)

## 📋 仕組み

Caffei Native は、バックグラウンドで5秒ごとに実行中のプロセスをチェックします。監視リストに登録されたアプリが検出されると、自動的に `caffeinate -w <PID>` プロセスを背後で起動し、対象アプリが動作している間スリープを防止します。アプリ本体が不意に強制終了されても、依存する `caffeinate` コマンドがOSレベルで確実に連動して終了する（ゾンビプロセスを防止する）安全設計となっています。

また、メインウィンドウを閉じてもアプリはシステムトレイ（メニューバー）に常駐し、ワークスペースを邪魔することなくスリープの管理を続けます。

## 💾 設定とデータ保存

監視対象のアプリリストや動作ログなどの設定は、ユーザーのアプリケーション設定ディレクトリ（`~/Library/Application Support/com.blue1st.caffei-native/config.json`）に永続化されます。
アプリを再起動したり、Macを再起動したりしても、一度設定した監視アプリのリストが毎回自動的に復元されます。

*   **GUIでのアプリ追加**: AppleScriptを利用したネイティブピッカー、または内部プロセスの自動読み込みリストからアプリ名を登録できます。
*   **注記**: 「プロセスからの自動検知」などは、AppleScriptを通じた状態取得 (`System Events` へのアクセス許可) を利用して安全に行われます。

## 🤖 自動ビルドとリリース (CI/CD)

GitHub Actionsが構成されており、新しいタグ（例: `v0.1.2`）をプッシュするか手動でワークフローをトリガーすることで、macOS向けの `.app` とインストーラ (`.dmg`) が自動的にビルドされ、GitHub Releasesにアップロードされます。

## 📄 ライセンス

このプロジェクトは MIT ライセンスの下で公開されています。詳細は [LICENSE](LICENSE) ファイルをご覧ください。

---

Created by [blue1st](https://github.com/blue1st)
