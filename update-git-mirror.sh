#!/bin/sh
set -eu

HERE=$(CDPATH=; cd -- "$(dirname -- "$0")" && pwd -P)

# 1. 获取最优镜像
BEST_MIRROR=$("$HERE/gmirror" 2>/dev/null | awk '{print $1}')

if [ -z "$BEST_MIRROR" ]; then
    echo "❌ gmirror 未返回可用镜像，跳过更新" >&2
    exit 1
fi

BEST_MIRROR="${BEST_MIRROR%/}/"
NEW_KEY="url.${BEST_MIRROR}https://github.com/.insteadOf"

# 2. 检查是否已经配置了该最优镜像（精确匹配单条）
if git config --global --get "$NEW_KEY" >/dev/null 2>&1; then
    echo "✅ 当前 Git 镜像已是最优: ${BEST_MIRROR}"
    exit 0
fi

# 3. 清除所有残留的 github.com insteadOf 规则
# 注意：--unset-all 配合正则可能因版本而异，这里用 get-regexp + while read 循环安全删除
git config --global --get-regexp '^url\..*\.insteadof$' | grep 'https://github.com/$' | while IFS=' ' read -r key _value; do
    git config --global --unset "$key" 2>/dev/null || true
done

# 4. 写入新规则
git config --global "$NEW_KEY" "https://github.com/"

echo "🚀 Git 镜像已更新为: ${BEST_MIRROR}"
echo "   验证: git config --global --get-regexp insteadOf"
