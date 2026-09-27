# NeNe Ruler のビルドと品質ゲート。
#
# 🔴 判定はここと xtask にある。CI は `make ci` を呼ぶだけで、CI 側にだけ存在する検査を作らない（QLT-005）。
# 🔴 道具の版は rust-toolchain.toml が決める。ここにも CI にも版を書かない（QLT-011）。
# 🔴 全件（make check）は既定ではない。差分に必要な検証を個別の target で選び、
#    全件は限定した検証で影響を覆えない理由を REASON= に添えたときだけ回す（QLT-001 / ADR 0021 の継承）。
CARGO ?= cargo

.PHONY: all check ci fmt fmt-check lint test conformance validate-git coverage deny doc-check build prove doctor clean

all: doctor

## check — 全件。REASON が空なら実行前に拒む。
check:
	@if [ -z "$(REASON)" ]; then \
	  echo "QLT-001: 全件検証には理由が要る。例: make check REASON='共通の lint 集合を変えたので全クレートに影響する'"; \
	  exit 2; \
	fi
	@echo "全件検証: $(REASON)"
	$(MAKE) fmt-check lint test conformance deny doc-check build

## ci — CI の必須 check。Git 規約・PR の検証記録・差分の空白だけを見る（QLT-012）。
ci: validate-git
	git diff --check origin/main...HEAD

## fmt — rustfmt に設定ファイルを置かない（QLT-004）。
fmt:
	$(CARGO) fmt --all

fmt-check:
	$(CARGO) fmt --all --check

## lint — 規則との対応は docs/CODING_RULES.md。
## 🔴 --all-targets を外さないこと。外すとテストコードが検査対象から落ちる。
## 🔴 --locked は Cargo.lock が更新される状態でのゲート通過を拒む（QLT-011）。
lint:
	$(CARGO) clippy --workspace --all-targets --locked -- -D warnings

test:
	$(CARGO) test --workspace --locked

## conformance — このリポジトリ固有の規約検査（CNF-0xx / xtask）。
conformance:
	$(CARGO) run --quiet -p xtask --locked -- conformance

## validate-git — GIT-001〜004。ローカルは origin/main..HEAD、CI は PR のメタデータ。
validate-git:
	$(CARGO) run --quiet -p xtask --locked -- validate-git

## coverage — 中核の分岐カバレッジ（QLT-009）。中核クレートが入った Issue で下限を置く。
## 導入: cargo install cargo-llvm-cov --locked
coverage:
	@command -v cargo-llvm-cov >/dev/null || { echo "coverage: cargo-llvm-cov が無い。cargo install cargo-llvm-cov --locked"; exit 2; }
	$(CARGO) llvm-cov --workspace --locked --branch --summary-only

## deny — 依存の許可制（ARC-013）。方針の正本は deny.toml。
## 🔴 --deny license-not-encountered を外さないこと。
## 導入: cargo install cargo-deny --locked
deny:
	@command -v cargo-deny >/dev/null || { echo "deny: cargo-deny が無い。cargo install cargo-deny --locked"; exit 2; }
	$(CARGO) deny --locked check --deny license-not-encountered

## doc-check — rustdoc の警告（壊れた intra-doc link 等）を失敗にする。
doc-check:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --workspace --no-deps --locked

build:
	$(CARGO) build --workspace --release --locked

## prove — ゲートが意図した違反で発火することを実測する（QLT-007）。結果は docs/quality/gate-proofs.md へ転記する。
prove:
	$(CARGO) run --quiet -p xtask --locked -- prove

## doctor — 開発環境（クラウドのコンテナ・WSLg）に道具が揃っているかを見る。検査ではない。
doctor:
	@rustc --version; cargo --version; cargo clippy --version; cargo fmt --version
	@command -v cargo-deny >/dev/null && cargo deny --version || echo "cargo-deny: 無い（cargo install cargo-deny --locked）"
	@command -v cargo-llvm-cov >/dev/null && cargo llvm-cov --version || echo "cargo-llvm-cov: 無い（cargo install cargo-llvm-cov --locked）"
	@pkg-config --modversion cairo >/dev/null 2>&1 && echo "cairo $$(pkg-config --modversion cairo)" || echo "cairo: 無い（apt-get install libcairo2-dev）"
	@command -v Xvfb >/dev/null && echo "Xvfb: ある" || echo "Xvfb: 無い（apt-get install xvfb）"
	@command -v import >/dev/null && echo "ImageMagick: ある" || echo "ImageMagick: 無い（apt-get install imagemagick）"

clean:
	$(CARGO) clean
