#!/bin/sh

# 检查sudo权限
if [ "$(id -u)" -ne 0 ]; then
    echo "This script requires sudo privileges."
    exit 1
fi

cleanup_github_hosts() {
    hosts=$1
    # 移除所有Github Hosts相关条目
    sed -i '/^\s*#\s*Github Hosts Start/,/^\s*#\s*Github Hosts End/d' "$hosts" 2>/dev/null
    # 移除多余的空行
    awk 'NF {n=0; print} !NF {if (!n) print; n=1}' "$hosts" > "$hosts.tmp" && mv "$hosts.tmp" "$hosts"
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
    awk 'NF {n=0; print} !NF {if (!n) print; n=1}' "$hosts" > "$hosts.tmp" && mv "$hosts.tmp" "$hosts"
}

get_github_dns() {
    uri_a=https://raw.githubusercontent.com/521xueweihan/GitHub520/refs/heads/main/hosts
    uri_b=https://githubhosts.xuanyuan.me
    # 从主要源获取
    dns=$(curl -s "$uri_a" | grep -oiE '(([0-9]+\.){3}[0-9]+|[0-9a-f:]{8,}:[0-9a-f]{1,4})\s+(\w+\.)+\w+')

   # 如果主要源失败，从备用源获取
    if [ -z "$dns" ]; then
        dns=$(wget -q -O - "$uri_b" | grep -oE '#Github Hosts Start.*?#Github Hosts End' | grep -ioE '(([0-9]+\.){3}[0-9]+|[0-9a-f:]{8,}:[0-9a-f]{1,4})\s+(\w+\.)+\w+')
    fi
    # 提取包含IPv4和IPv6的Github Hosts记录
dns=$(wget -q -O - "$uri_b" | grep -oE '#Github Hosts Start.*?#Github Hosts End' | grep -oE '(([0-9]+\.){3}[0-9]+|(([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])))\s+(\w+\.)+\w+')

    echo "$dns"
}

check_github_hosts() {
    hosts=$1
    if grep -q -E "^\s*#\s*Github Hosts Start" "$hosts"; then
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
    dns_github=$(get_github_dns $@)

    # 检查是否获取到DNS条目
    if [ -z "$dns_github" ]; then
        echo "Error: Failed to get GitHub DNS entries"
        exit 1
    fi

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
