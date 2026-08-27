#!/bin/sh
set -eu

# 验证参数格式
case "$1" in
  "https://github.com/"*) ;;
  *)
    echo "Usage: $0 <https://github.com/owner/repo>" >&2
    exit 1
    ;;
esac

# 提取 owner/repo
repo_path="${1#https://github.com/}"
# 去除可能的尾部斜杠
repo_path="${repo_path%/}"

# 构建 API URL（可调整 per_page 为所需数量）
api_url="https://api.github.com/repos/${repo_path}/releases?per_page=6"

echo $api_url
# 请求并过滤下载链接
curl -s -f "$api_url" | jq -r '.[] | .assets[]? | .browser_download_url'
