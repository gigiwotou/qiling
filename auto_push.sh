#!/bin/bash

# 自动推送脚本 - 检测代码修改并推送到GitHub

echo "=== 自动推送脚本 ==="

# 检查当前目录是否是git仓库
if [ ! -d ".git" ]; then
    echo "错误：当前目录不是git仓库"
    exit 1
fi

# 检查是否有修改
echo "检查代码修改..."
git status

# 检查是否有未提交的修改
if git status | grep -q "modified\|added\|deleted"; then
    echo "发现代码修改，准备提交..."
    
    # 添加所有修改的文件
    git add .
    
    # 生成提交信息
    commit_message="Auto commit: $(date '+%Y-%m-%d %H:%M:%S')"
    
    # 提交代码
    git commit -m "$commit_message"
    
    # 推送到GitHub
    echo "推送代码到GitHub..."
    git push origin HEAD:main
    
    if [ $? -eq 0 ]; then
        echo "✅ 推送成功！"
    else
        echo "❌ 推送失败！"
        exit 1
    fi
else
    echo "✅ 没有发现代码修改，无需推送"
fi

echo "=== 自动推送脚本完成 ==="
