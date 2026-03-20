# ドキュメント同期ルール

## 目的

エージェント設定やテンプレートを変更した際、関連ドキュメントも同時に更新する。

## ルール

### 変更時にドキュメント更新が必要なケース

- **エージェントの追加・変更**: `agents/` 配下のファイルを変更したら README.md のエージェント一覧を更新
- **steering テンプレートの追加・変更**: `steering-templates/` を変更したら README.md と create-agent プロンプトのテンプレート一覧を更新
- **Makefile の変更**: install/uninstall の手順が変わったら README.md のセットアップ手順を更新
- **ディレクトリ構成の変更**: README.md の構成セクションを更新

### 対象ドキュメント

- `README.md`
- `agents/prompts/create-agent.md`（テンプレート一覧やフロー定義）
