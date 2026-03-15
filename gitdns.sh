#!/bin/sh

set -eu

# 检查sudo权限
if [ "$(id -u)" -ne 0 ]; then
    echo "This script requires sudo privileges."
    exit 1
fi

cleanup_github_hosts() {
    hosts=$1
    # 移除所有Github Hosts相关条目
    sed -i -E '/^[[:space:]]*#[[:space:]]*[gG]ithub[[:space:]]+[hH]osts[[:space:]]+[sS]tart[[:space:]]*$/,/^[[:space:]]*#[[:space:]]*[gG]ithub[[:space:]]+[hH]osts[[:space:]]+[eE]nd[[:space:]]*$/d' "$hosts"
}

add_github_hosts() {
    hosts=$1
    dns=$2

    # 确保文件末尾有换行符
    if [ -s "$hosts" ]; then
        if [ "$(tail -c 1 "$hosts")" != "\n" ]; then
            echo >> "$hosts"
        fi
    fi


    # 添加DNS条目
    {
        echo "# Github Hosts Start"
        echo "$dns"
        echo "# Github Hosts End"
    } >> "$hosts"

    # 移除多余的空行
    awk 'NF {n=0; print} !NF {if (!n) print; n=1}' "$hosts" | tee "$hosts"
}

get_github_dns() {
    uri_a=https://raw.githubusercontent.com/521xueweihan/GitHub520/refs/heads/main/hosts
    uri_b=https://githubhosts.xuanyuan.me
    # 从主要源获取
    dns=$(curl -s "$uri_a" | grep -ioE '(([0-9]+\.){3}[0-9]+|[0-9a-f:]{8,}:[0-9a-f]{1,4})\s+(\w+\.)+\w+')

   # 如果主要源失败，从备用源获取
    if [ -z "$dns" ]; then
        dns=$(wget -q -O - "$uri_b" | grep -ioE '#Github\s+Hosts\s+Start.*?#Github\s+Hosts\s+End' | grep -ioE '(([0-9]+\.){3}[0-9]+|[0-9a-f:]{8,}:[0-9a-f]{1,4})\s+(\w+\.)+\w+')
    fi

    echo "$dns"
}

check_github_hosts() {
    hosts=$1
    if grep -q -iE '^\s*#\s*Github\s+Hosts\s+Start\s*$' "$hosts"; then
        return 0  # 存在
    else
        return 1  # 不存在
    fi
}

# 主函数
main() {
    hosts=/etc/hosts

    # 检查hosts文件是否存在
    if [ ! -f "$hosts" ]; then
        echo "Error: Hosts file not found at $hosts"
        exit 1
    fi

    echo "Getting latest GitHub DNS entries..."
    # shellcheck disable=SC2068
    dns_github=$(get_github_dns $@) && echo "GitHub DNS entries retrieved successfully."

    # 检查是否获取到DNS条目
    if [ -z "$dns_github" ]; then
        echo "GitHub DNS entries is empty. Please check the source."
        exit 1
    fi

    dns_local=$(sed -nE '/^[[:space:]]*#[[:space:]]*[gG]ithub[[:space:]]+[hH]osts[[:space:]]+[sS]tart[[:space:]]*$/,/^[[:space:]]*#[[:space:]]*[gG]ithub[[:space:]]+[hH]osts[[:space:]]+[eE]nd[[:space:]]*$/p' "$hosts" | grep -vE '^\s*#' || true)
    [ "$dns_local" = "$dns_github" ] && {
        echo "GitHub DNS entries are already up to date. No changes needed."
        exit 0
    }

    if check_github_hosts "$hosts"; then
        echo "GitHub entries found in hosts file."
        echo "Cleaning up existing GitHub entries..."
        cleanup_github_hosts "$hosts"
    else
        echo "No GitHub entries found in hosts file."
    fi

    echo "Adding latest GitHub entries..."
    add_github_hosts "$hosts" "$dns_github"

    echo "GitHub DNS entries updated successfully!"
}

# 执行主函数

# shellcheck disable=SC2068
main $@
