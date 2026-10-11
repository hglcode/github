#!/bin/sh
set -eu
help() {
    echo "Usage: $0 or $(basename "$0") <https://github.com/owner/repo>" >&2
}
need() {
    command -v "$1" >/dev/null 2>&1 || {
        echo "Error: $1 is not installed." >&2
        exit 1
    }
}
need curl
need jq

[ $# -lt 1 ] && {
    help
    exit 1
}

case "$1" in
"https://github.com/"*) ;;
*)
    help
    exit 1
    ;;
esac

repo_path="${1#https://github.com/}"
repo_path="${repo_path%/}"
uri="https://api.github.com/repos/${repo_path}/releases?per_page=6"

# 攒 curl 参数：无 token 时匿名，有 token 时带鉴权头
set -- -s -f
if [ -n "${GITHUB_TOKEN:-}" ]; then
  set -- "$@" \
    -H "Authorization: Bearer $GITHUB_TOKEN" \
    -H "Accept: application/vnd.github+json"
fi

if ! rsp=$(curl "$@" "$uri"); then
  echo "Error: Failed to fetch releases for ${uri}" >&2
  exit 1
fi

printf '%s\n' "$rsp" | jq -r '.[] | .assets[]? | .browser_download_url'
