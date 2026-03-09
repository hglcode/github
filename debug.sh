#!/bin/sh

# 检查sudo权限
if [ "$(id -u)" -ne 0 ]; then
    echo "This script requires sudo privileges."
    exit 1
fi

# 清理hosts文件中的GitHub520条目
cleanup_github_hosts() {
    hosts=$1
    # 移除所有GitHub520相关条目
    sed -i '/^\s*#\s*GitHub520/,/^\s*#\s*GitHub520.*End/d' "$hosts" 2>/dev/null
    # 移除多余的空行
    awk 'NF {n=0; print} !NF {if (!n) print; n=1}' "$hosts" > "$hosts.tmp" && mv "$hosts.tmp" "$hosts"
}

# 添加GitHub520条目
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
    echo "$dns" >> "$hosts"

    # 移除多余的空行
    awk 'NF {n=0; print} !NF {if (!n) print; n=1}' "$hosts" > "$hosts.tmp" && mv "$hosts.tmp" "$hosts"
}

# 获取GitHub520 DNS条目
get_github_dns() {
    # 从主要源获取
    dns=$(curl -s https://raw.githubusercontent.com/521xueweihan/GitHub520/refs/heads/main/hosts | grep -v '^\s*$')

   # 如果主要源失败，从备用源获取
    if [ -z "$dns" ]; then
        dns=$(curl -s https://githubhosts.xuanyuan.me | sed -n '/^\s*#\s*GitHub.*Start/,/^\s*#\s*GitHub.*End/p')
    fi

    echo "$dns"
}

# 检查GitHub520条目是否已存在
check_github_hosts() {
    hosts=$1
    # 检查是否存在GitHub520条目
    if grep -q "^\s*#\s*GitHub520" "$hosts"; then
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

    # 获取最新的GitHub520 DNS条目
    echo "Getting latest GitHub520 DNS entries..."
    dns_github=$(get_github_dns)

    # 检查是否获取到DNS条目
    if [ -z "$dns_github" ]; then
        echo "Error: Failed to get GitHub520 DNS entries"
        exit 1
    fi

    # 检查本地是否已有GitHub520条目
    if check_github_hosts "$hosts"; then
        echo "GitHub520 entries found in hosts file."

        # 清理现有的GitHub520条目
        echo "Cleaning up existing GitHub520 entries..."
        cleanup_github_hosts "$hosts"
    else
        echo "No GitHub520 entries found in hosts file."
    fi

    # 添加新的GitHub520条目
    echo "Adding latest GitHub520 entries..."
    add_github_hosts "$hosts" "$dns_github"

    echo "GitHub520 DNS entries updated successfully!"
}

# 执行主函数
main
