# ChatStep

## 重要提示
- README.md和README.en.md里可能有错误

## 介绍
- 支持自然语言生成并执行Shell脚本的CLI工具
- 支持自定义命令别名
- 多AI平台支持
- [QQ群](http://qm.qq.com/cgi-bin/qm/qr?_wv=1027&k=XE6sZHD2Bi1IFKZyZ_hMa4T9UsFTMeBD&authKey=sCB6CvS4j%2BxPSlfh3BNR2%2F%2FTr%2BJ45T6Uku2YvmiyScPISHjiWSMZU%2BQMJJzh1EdW&noverify=0&group_code=1035618715)
- [Discord](https://discord.gg/MepDu9XJnd)
- [matrix](https://matrix.to/#/#chat-step:matrix.org)

## 安装
```bash
cargo install --git https://gitee.com/qwe12345678/chat-step.git
```
或获取源码后：
```bash
bash install.sh
```

## 配置文件
配置文件位于 `~/.config/chatstep/chatstep.conf`，首次运行会自动生成。完整配置结构：

```toml
# API 配置示例
[glm-4-flash]
API_KEY = "your_api_key_here"     # 从AI平台获取
MODEL_NAME = "glm-4-flash"       # 模型标识
API_ENDPOINT = "https://api.example.com/v4/chat/completions"  # API地址

# 场景配置
[scenarios]
dialogue = "glm-4-flash"       # 对话模式默认模型
multifunction = "glm-4-flash"  # 脚本生成默认模型

# 命令别名配置
[aliases]
random = { prompt = "生成一个1-100的随机数" }
create_py = { 
    prompt = "创建名为{0}的Python项目，包含{1}和README.md",
    profile = "QwQ-32B" 
}
```

## 使用方法

### 基础命令
对话模式（不执行命令）：
```bash
chatstep ask "要是基于TypeScript搞一个支持依赖类型的编程语言价值大吗？"
```

生成并执行脚本：
```bash
chatstep run "生成随机数并保存到random.txt"
```

### 别名命令
使用预定义的别名：
```bash
chatstep random  # 等价于 chatstep run "生成一个1-100的随机数"
```

带参数的别名：
```bash
chatstep create_py myproject setup.py
```
等价于：
```bash
chatstep run "创建名为myproject的Python项目，包含setup.py和README.md" -p QwQ-32B
```

### 高级功能
1. 指定AI模型：
```bash
chatstep run "用QwQ-32B生成脚本" -p QwQ-32B
```

3. 自动修复：
当脚本执行出错时，会自动尝试生成修复方案并询问是否执行

## 别名配置指南
在配置文件的 `[aliases]` 部分添加：

```toml
[aliases]
# 简单别名
别名 = { prompt = "固定prompt内容" }

# 带参数的别名（使用{0}、{1}占位符）
别名 = { 
    prompt = "包含{0}个参数和{1}的模板", 
    profile = "指定使用的模型" 
}
```

示例：
```toml
[aliases]
latex_report = { 
    prompt = "生成包含'{0}'主题的LaTeX报告，保存为{1}.tex并编译为PDF",
    profile = "glm-4-flash"
}
```
使用：
```bash
chatstep latex_report "AI发展前景" report_2024
```

## 注意事项
1. API密钥可以手动添加（需要从对应平台申请），自带API和密钥

2. 执行脚本前请仔细检查生成的内容

3. 参数替换遵循从0开始的索引：
   - {0} → 第一个参数
   - {1} → 第二个参数
   - 以此类推

4. 可通过`--profile`参数覆盖别名中指定的模型
