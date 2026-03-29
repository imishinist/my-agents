# my-agents

Kiro CLI / Claude Code カスタムエージェントの管理リポジトリ。

## セットアップ

### Kiro CLI

```bash
make install      # ~/.kiro/agents/ にシンボリックリンクを作成
make uninstall    # シンボリックリンクを削除
make install-copy # シンボリックリンクが動かない場合のフォールバック（コピー）
make list         # ソースとインストール済みエージェントの一覧
```

### Claude Code

```bash
make claude-install      # ~/.claude/commands/ にシンボリックリンクを作成
make claude-uninstall    # シンボリックリンクを削除
make claude-install-copy # フォールバック（コピー）
make claude-list         # ソースとインストール済みコマンドの一覧
```

### 両方まとめて

```bash
make all-install    # Kiro + Claude Code 両方インストール
make all-uninstall  # 両方アンインストール
make all-list       # 両方の一覧
```

## 使い方

### Kiro CLI

```bash
cd /path/to/your-project
kiro-cli --agent create-agent
```

### Claude Code

```bash
cd /path/to/your-project
claude
# Claude Code 内で /user:create-agent を実行
```

## 構成

```
agents/                        # Kiro CLI 用
├── create-agent.json          # メタエージェント（エージェント生成用）
└── prompts/
    └── create-agent.md        # create-agent のプロンプト定義
steering-templates/            # Kiro create-agent が使うテンプレート
├── doc-sync.md
├── coding-standards.md
├── architecture-decisions.md
├── self-update.md
└── sandbox-awareness.md
claude-commands/               # Claude Code 用
└── create-agent.md            # メタコマンド（CLAUDE.md・コマンド生成用）
claude-templates/              # Claude create-agent が使うテンプレート
├── doc-sync.md
├── coding-standards.md
├── architecture-decisions.md
├── self-update.md
└── sandbox-awareness.md
.kiro/steering/                # このリポジトリ用の steering
├── doc-sync.md
└── self-update.md
```

## エージェント / コマンド一覧

### Kiro CLI エージェント

| エージェント | 用途 |
|---|---|
| create-agent | リポジトリを解析し、適切なカスタムエージェントを対話的に生成するメタエージェント |

### Claude Code カスタムコマンド

| コマンド | 呼び出し | 用途 |
|---|---|---|
| create-agent | `/user:create-agent` | リポジトリを解析し、CLAUDE.md とカスタムコマンドを対話的に生成するメタコマンド |

## テンプレート一覧

Kiro 用 (`steering-templates/`) と Claude Code 用 (`claude-templates/`) で同じプリセットを提供:

| テンプレート | 用途 |
|---|---|
| doc-sync | コード変更時にドキュメントも更新するルール |
| coding-standards | コーディング規約（言語・FW検出に基づいて生成） |
| architecture-decisions | 設計変更時にADR等を更新するルール |
| self-update | ガイドライン自体を変更に応じて更新するメタルール |
| sandbox-awareness | sandbox環境での権限エラーを検知しユーザーに伝えるルール |
