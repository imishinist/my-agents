# --- Kiro ---
AGENTS_SRC := $(wildcard agents/*.json)
AGENTS_DST := $(patsubst agents/%.json,$(HOME)/.kiro/agents/%.json,$(AGENTS_SRC))
KIRO_DIR   := $(HOME)/.kiro/agents

# --- Claude Code ---
CLAUDE_SRC := $(wildcard claude-commands/*.md)
CLAUDE_DST := $(patsubst claude-commands/%.md,$(HOME)/.claude/commands/%.md,$(CLAUDE_SRC))
CLAUDE_DIR := $(HOME)/.claude/commands

.PHONY: install uninstall install-copy list
.PHONY: claude-install claude-uninstall claude-install-copy claude-list
.PHONY: all-install all-uninstall all-list

# ============================
# Kiro targets
# ============================

install: $(AGENTS_DST)
	@# promptsディレクトリもリンク（file:// 相対パス解決のため）
	@if [ -d agents/prompts ] && [ ! -e "$(KIRO_DIR)/prompts" ]; then \
		ln -sf $(abspath agents/prompts) $(KIRO_DIR)/prompts; \
	fi
	@echo "✅ Kiro symlinks created:"
	@ls -la $(KIRO_DIR)/ | grep -- '->'

$(KIRO_DIR)/%.json: agents/%.json | $(KIRO_DIR)
	ln -sf $(abspath $<) $@

$(KIRO_DIR):
	mkdir -p $@

uninstall:
	@for f in $(AGENTS_SRC); do \
		target="$(KIRO_DIR)/$$(basename $$f)"; \
		if [ -L "$$target" ]; then rm "$$target" && echo "Removed $$target"; fi; \
	done
	@if [ -L "$(KIRO_DIR)/prompts" ]; then rm "$(KIRO_DIR)/prompts" && echo "Removed prompts link"; fi

install-copy:
	@mkdir -p $(KIRO_DIR)/prompts
	@for f in $(AGENTS_SRC); do \
		cp "$$f" "$(KIRO_DIR)/$$(basename $$f)"; \
		echo "Copied $$f -> $(KIRO_DIR)/$$(basename $$f)"; \
	done
	@cp -r agents/prompts/* $(KIRO_DIR)/prompts/ 2>/dev/null || true
	@echo "⚠️  Copied (not symlinked). Edits here won't auto-reflect."

list:
	@echo "=== Source agents (Kiro) ==="
	@ls agents/*.json 2>/dev/null || echo "(none)"
	@echo "\n=== Installed (Kiro) ==="
	@ls -la $(KIRO_DIR)/*.json 2>/dev/null | grep -v example || echo "(none)"

# ============================
# Claude Code targets
# ============================

claude-install: $(CLAUDE_DST)
	@# claude-templatesディレクトリもリンク（テンプレート参照用）
	@if [ -d claude-templates ] && [ ! -e "$(CLAUDE_DIR)/../claude-templates" ]; then \
		ln -sf $(abspath claude-templates) $(CLAUDE_DIR)/../claude-templates; \
	fi
	@echo "✅ Claude Code symlinks created:"
	@ls -la $(CLAUDE_DIR)/ | grep -- '->'

$(CLAUDE_DIR)/%.md: claude-commands/%.md | $(CLAUDE_DIR)
	ln -sf $(abspath $<) $@

$(CLAUDE_DIR):
	mkdir -p $@

claude-uninstall:
	@for f in $(CLAUDE_SRC); do \
		target="$(CLAUDE_DIR)/$$(basename $$f)"; \
		if [ -L "$$target" ]; then rm "$$target" && echo "Removed $$target"; fi; \
	done
	@if [ -L "$(CLAUDE_DIR)/../claude-templates" ]; then rm "$(CLAUDE_DIR)/../claude-templates" && echo "Removed templates link"; fi

claude-install-copy:
	@mkdir -p $(CLAUDE_DIR)
	@for f in $(CLAUDE_SRC); do \
		cp "$$f" "$(CLAUDE_DIR)/$$(basename $$f)"; \
		echo "Copied $$f -> $(CLAUDE_DIR)/$$(basename $$f)"; \
	done
	@echo "⚠️  Copied (not symlinked). Edits here won't auto-reflect."

claude-list:
	@echo "=== Source commands (Claude Code) ==="
	@ls claude-commands/*.md 2>/dev/null || echo "(none)"
	@echo "\n=== Installed (Claude Code) ==="
	@ls -la $(CLAUDE_DIR)/*.md 2>/dev/null | grep -v example || echo "(none)"

# ============================
# Combined targets
# ============================

all-install: install claude-install
all-uninstall: uninstall claude-uninstall
all-list: list claude-list
