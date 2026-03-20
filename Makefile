AGENTS_SRC := $(wildcard agents/*.json)
AGENTS_DST := $(patsubst agents/%.json,$(HOME)/.kiro/agents/%.json,$(AGENTS_SRC))
KIRO_DIR   := $(HOME)/.kiro/agents

.PHONY: install uninstall install-copy list

install: $(AGENTS_DST)
	@# promptsディレクトリもリンク（file:// 相対パス解決のため）
	@if [ -d agents/prompts ] && [ ! -e "$(KIRO_DIR)/prompts" ]; then \
		ln -sf $(abspath agents/prompts) $(KIRO_DIR)/prompts; \
	fi
	@echo "✅ Symlinks created:"
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
	@echo "=== Source agents ==="
	@ls agents/*.json 2>/dev/null || echo "(none)"
	@echo "\n=== Installed (global) ==="
	@ls -la $(KIRO_DIR)/*.json 2>/dev/null | grep -v example || echo "(none)"
