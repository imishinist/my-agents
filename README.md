# my-agents

Kiro CLI カスタムエージェントの管理リポジトリ。

## セットアップ

```bash
make install      # ~/.kiro/agents/ にシンボリックリンクを作成
make uninstall    # シンボリックリンクを削除
make install-copy # シンボリックリンクが動かない場合のフォールバック（コピー）
make list         # ソースとインストール済みエージェントの一覧
```

## 使い方

```bash
# create-agent でリポジトリに合ったエージェントを対話的に生成
cd /path/to/your-project
kiro-cli --agent create-agent
```

## 構成

```
agents/
├── create-agent.json      # メタエージェント（エージェント生成用）
└── prompts/
    └── create-agent.md    # create-agent のプロンプト定義
steering-templates/            # create-agent が使うテンプレート
├── doc-sync.md                # ドキュメント同期ルール
├── coding-standards.md        # コーディング規約
├── architecture-decisions.md  # アーキテクチャ決定記録
├── self-update.md             # ガイドライン自己更新ルール
└── sandbox-awareness.md       # sandbox環境の権限エラー検知
.kiro/steering/                # このリポジトリ用の steering
├── doc-sync.md
└── self-update.md
```

## エージェント一覧

| エージェント | 用途 |
|---|---|
| create-agent | リポジトリを解析し、適切なカスタムエージェントを対話的に生成するメタエージェント |

## steering テンプレート一覧

| テンプレート | 用途 |
|---|---|
| doc-sync | コード変更時にドキュメントも更新するルール |
| coding-standards | コーディング規約（言語・FW検出に基づいて生成） |
| architecture-decisions | 設計変更時にADR等を更新するルール |
| self-update | ガイドライン自体を変更に応じて更新するメタルール |
| sandbox-awareness | sandbox環境での権限エラーを検知しユーザーに伝えるルール |
