use clap::{Parser, Subcommand};
use dirs::config_dir;
use dotenv::dotenv;
use regex::Regex;
use reqwest;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    error::Error,
    fs,
    io::{self, Write},
    path::PathBuf,
    process::Command,
    time::Duration,
};
use whoami;

// ----------------- 配置相关 -----------------

/// 单个 profile 对应的字段
#[derive(Debug, Deserialize)]
struct Profile {
    #[serde(rename = "API_KEY")]
    api_key: String,
    #[serde(rename = "MODEL_NAME")]
    model_name: String,
    #[serde(rename = "API_ENDPOINT")]
    api_endpoint: String,
}

/// 命令别名配置
#[derive(Debug, Deserialize)]
struct CommandAlias {
    prompt: String,
    profile: Option<String>,
}

/// 场景配置
#[derive(Debug, Deserialize)]
struct Scenarios {
    dialogue: String,
    multifunction: String,
}

/// 整个 TOML 的反序列化目标
#[derive(Debug, Deserialize)]
struct ChatstepConfig {
    #[serde(flatten)]
    profiles: HashMap<String, Profile>,
    #[serde(default)]
    aliases: HashMap<String, CommandAlias>,
    scenarios: Scenarios,
}

/// 读取 ~/.config/chatstep/chatstep.conf
fn load_config() -> Result<ChatstepConfig, Box<dyn Error>> {
    let mut path: PathBuf = config_dir().ok_or("无法定位 config 目录")?;
    path.push("chatstep");
    path.push("chatstep.conf");

    // 如果配置文件不存在，创建默认配置
    if !path.exists() {
        // 确保目录存在
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // 默认配置内容
        let default_config = r#"###############################
# QwQ-32B （多功能场景推荐）  #
###############################
[QwQ-32B]
API_KEY      = "sk-W0rpStc95T7JVYVwDYc29IyirjtpPPby6SozFMQr17m8KWeo"
MODEL_NAME   = "free:QwQ-32B"
API_ENDPOINT = "https://api.suanli.cn/v1/chat/completions"

###############################
# glm-4-flash （对话场景推荐） #
###############################
[glm-4-flash]
API_KEY      = "b6e23abec9b746adb20cb0f3ae116f8e.LIlANJZWSUCbXLnX"
MODEL_NAME   = "glm-4-flash"
API_ENDPOINT = "https://open.bigmodel.cn/api/paas/v4/chat/completions"

#####################################
# 场景 —— 默认选哪个 profile 的 key #
#####################################
[scenarios]
# 仅对话 
dialogue       = "glm-4-flash"
# 多功能
multifunction  = "glm-4-flash"

#####################################
# 命令别名 —— 可以用 chatstep <代号> 代替 chatstep run <prompt> #
#####################################
[aliases]
# 示例1: 简单别名
example1 = { prompt = "生成一个随机数并保存到文件" }

# 示例2: 带参数的别名
example2 = { prompt = "创建一个名为{0}的Python项目，包含setup.py和README.md", profile = "QwQ-32B" }"#;

        fs::write(&path, default_config)?;
    }

    let s = fs::read_to_string(&path)?;
    let cfg: ChatstepConfig = toml::from_str(&s)?;
    Ok(cfg)
}

// ----------------- AI 接口调用 -----------------

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    top_p: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i32>,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: Message,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

/// 去掉 <think> 标签及其中内容
fn remove_think_tags(input: &str) -> String {
    let re = Regex::new(r"(?s)<think>.*?</think>").unwrap();
    re.replace_all(input, "").to_string()
}

/// 发送请求到 AI 接口，使用指定的 profile
async fn call_ai_api(
    messages: Vec<Message>,
    profile: &Profile,
) -> Result<String, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1200))
        .build()?;

    let req = ChatRequest {
        model: profile.model_name.clone(),
        messages,
        temperature: Some(0.7),
        top_p: Some(0.7),
        max_tokens: Some(512),
    };

    let resp_text = client
        .post(&profile.api_endpoint)
        .header("Authorization", format!("Bearer {}", profile.api_key))
        .header("Content-Type", "application/json")
        .json(&req)
        .send()
        .await?
        .text()
        .await?;

    let parsed: ChatResponse =
        serde_json::from_str(&resp_text).map_err(|e| format!("{}\n<<<{}>>>", e, resp_text))?;

    Ok(remove_think_tags(parsed.choices[0].message.content.trim()))
}

// ----------------- Shell 执行相关 -----------------

/// 系统信息
#[derive(Debug, Serialize)]
struct SystemInfo {
    os: String,
    os_version: String,
    shell: String,
    shell_version: String,
}

/// 收集系统信息
fn get_system_info() -> SystemInfo {
    // OS
    let (os, os_version) = match whoami::platform() {
        whoami::Platform::Linux => {
            let data = fs::read_to_string("/etc/os-release").unwrap_or_default();
            let ver = data
                .lines()
                .find(|l| l.starts_with("PRETTY_NAME="))
                .and_then(|l| l.split('=').nth(1))
                .map(|s| s.trim_matches('"').to_string())
                .unwrap_or_else(|| "Linux".into());
            ("Linux".into(), ver)
        }
        whoami::Platform::Windows => ("Windows".into(), "unknown".into()),
        _ => ("unknown".into(), "unknown".into()),
    };

    // Shell
    let shell = std::env::var("SHELL")
        .unwrap_or_else(|_| if cfg!(windows) { "cmd.exe".into() } else { "sh".into() });
    let shell_version = Command::new(&shell)
        .arg("--version")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|_| "Unknown".into());

    SystemInfo {
        os,
        os_version,
        shell,
        shell_version,
    }
}

/// 执行 Shell 命令并抓取输出
fn execute_command(cmd: &str) -> Result<String, String> {
    let info = get_system_info();
    let output = if cfg!(windows) {
        Command::new(&info.shell)
            .args(&["/C", cmd])
            .output()
            .expect("执行失败")
    } else {
        Command::new(&info.shell)
            .arg("-c")
            .arg(cmd)
            .output()
            .expect("执行失败")
    };

    // 检查执行状态和stderr
    if !output.status.success() || !output.stderr.is_empty() {
        let error_msg = String::from_utf8_lossy(&output.stderr).to_string();
        return Err(error_msg);
    }
    
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// 去除 AI 返回脚本的首尾标记，只保留中间内容
fn extract_command_content(full: &str) -> String {
    let mut lines: Vec<&str> = full.lines().collect();
    if lines.len() > 2 {
        lines.remove(0);
        lines.pop();
    }
    lines.join("\n").trim().to_string()
}

// 修改后的函数，现在可以处理脚本修复历史
async fn handle_script_error(
    original_script: &str,
    error: &str,
    fix_history: &Vec<(String, String)>,  // 历史脚本和它们的错误信息
    profile: &Profile
) -> Result<String, Box<dyn Error>> {
    // 构造错误修复prompt，包含历史修复记录
    let mut fix_prompt = format!(
        "你是一个Shell脚本修复专家。以下Shell脚本执行出错，请修复：\n\n## 原始脚本\n```bash\n{}\n```\n\n## 执行错误\n```\n{}\n```\n\n",
        original_script,
        error
    );
    
    // 如果有历史修复记录，将其添加到prompt中
    if !fix_history.is_empty() {
        fix_prompt.push_str("\n## 以前的修复尝试\n");
        for (i, (script, err)) in fix_history.iter().enumerate() {
            fix_prompt.push_str(&format!(
                "### 修复尝试 {}\n```bash\n{}\n```\n\n### 执行错误\n```\n{}\n```\n\n",
                i+1, script, err
            ));
        }
    }
    
    fix_prompt.push_str("## 请求\n请修复上述脚本，使其能够正常执行。仅返回完整的修复后的脚本，不要包含任何解释或额外文本。\n脚本第一行必须包含正确的shebang。");

    // 调用AI生成修复后的脚本
    let result = call_ai_api(
        vec![Message {
            role: "user".into(),
            content: fix_prompt,
        }],
        profile,
    ).await?;

    Ok(extract_command_content(&result))
}

/// 替换prompt中的参数占位符
fn replace_placeholders(prompt: &str, args: &[String]) -> String {
    let mut result = prompt.to_string();
    for (i, arg) in args.iter().enumerate() {
        result = result.replace(&format!("{{{}}}", i), arg);
    }
    result
}

// ----------------- CLI 定义 -----------------

#[derive(Parser)]
#[command(name = "ChatStep", version = "0.1.0")]
struct Cli {
    /// 覆盖使用的 profile（对应 chatstep.conf 里 profiles 的 key）
    #[arg(short = 'p', long = "profile")]
    profile: Option<String>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand, Clone)]
enum Commands {
    /// 生成／执行 Shell 脚本
    Run {
        /// 用户的自然语言请求
        prompt: String,
    },
    /// 纯对话场景
    Ask {
        /// 用户的提问
        prompt: String,
    },
    /// 执行配置文件中定义的别名命令
    #[command(external_subcommand)]
    Alias(Vec<String>),
}

// ----------------- 主函数 -----------------

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // 加载 .env（如果有）
    dotenv().ok();
    // 加载 TOML 配置
    let cfg = load_config()?;
    // 解析命令行
    let cli = Cli::parse();

    // 处理别名命令
    let (command, profile_override) = match &cli.command {
        Some(Commands::Alias(args)) if !args.is_empty() => {
            let alias_name = &args[0];
            let alias_args = &args[1..];
            
            let alias = cfg.aliases.get(alias_name)
                .ok_or_else(|| format!("别名 '{}' 未定义", alias_name))?;
            
            let prompt = replace_placeholders(&alias.prompt, alias_args);
            let profile_override = alias.profile.clone();
            
            (Some(Commands::Run { prompt }), profile_override)
        }
        cmd => (cmd.clone(), None),
    };

    // 根据子命令决定默认 profile key，并取出命令内容
    let (default_key, command) = match command {
        Some(Commands::Run { .. }) => {
            let profile_key = profile_override
                .or_else(|| cli.profile.clone())
                .unwrap_or_else(|| cfg.scenarios.multifunction.clone());
            (profile_key, command.unwrap())
        }
        Some(Commands::Ask { .. }) => {
            let profile_key = profile_override
                .or_else(|| cli.profile.clone())
                .unwrap_or_else(|| cfg.scenarios.dialogue.clone());
            (profile_key, command.unwrap())
        }
        Some(Commands::Alias(_)) => unreachable!(), // 已经在上面处理了
        None => {
            eprintln!("请指定子命令：Run 或 Ask");
            std::process::exit(1);
        }
    };

    // 从配置里取 Profile
    let prof = cfg
        .profiles
        .get(&default_key)
        .ok_or_else(|| format!("Profile '{}' 不存在，请检查 chatstep.conf", default_key))?;

    // 根据命令路由
    match command {
        Commands::Run { prompt } => {
            // 构造 Shell 脚本生成 prompt
            let sys = get_system_info();
                        let step_prompt = format!(
    "你是一个Shell脚本生成助手。请根据以下信息处理用户请求:

## 系统环境
- 操作系统: {}
- 系统版本: {}
- Shell程序: {}
- Shell版本: {}

## 用户请求
\"{}\"

## 处理规则
1. 如果请求需要执行Shell命令来完成,请生成一个完整的Shell脚本:
   - Shell是{},Shell版本是{}
   - 只输出脚本内容,不要包含任何解释或额外文本
2. 如果请求不需要Shell命令完成,请严格返回:
\"不需要执行Shell命令\"
3. 生成的Shell脚本第一行必须是正确的shebang(例如#!/bin/bash或#!/usr/bin/zsh)
4. 如果要制作文档和PPT的话，要生成一个完整的Shell脚本,用LaTeX,先生成.tex文件再生成PDF,生成PDF之后不要自动删除.tex文件

请根据上述规则处理请求。",
    sys.os,
    sys.os_version,
    sys.shell,
    sys.shell_version,
    prompt,
    sys.shell,
    sys.shell_version
);

            // 第一次尝试：生成 Shell 脚本
            let result = call_ai_api(
                vec![Message {
                    role: "user".into(),
                    content: step_prompt,
                }],
                prof,
            )
            .await?;

            if result.trim() == "不需要执行Shell命令" {
                let ask_key = cfg.scenarios.dialogue;
                let ask_prof = cfg
                .profiles
                .get(&ask_key)
                .ok_or("对话场景对应的 profile 不存在")?;
                // 转为普通对话
                let resp = call_ai_api(
                    vec![Message {
                        role: "user".into(),
                        content: prompt.clone(),
                    }],
                    ask_prof,
                )
                .await?;

                println!("{}", resp);
                return Ok(());
            }

            // 提取脚本主体
            let original_script = extract_command_content(&result);
            println!("将要执行的脚本\n{}", original_script);
            print!("确认执行？(Y/n)");
            io::stdout().flush().unwrap();
            let mut answer = String::new();
            io::stdin().read_line(&mut answer).unwrap();
            
            if answer.trim().is_empty() || answer.trim().eq_ignore_ascii_case("y") {
                // 创建一个历史记录，用于跟踪所有修复尝试
                let mut fix_history: Vec<(String, String)> = Vec::new();
                let mut current_script = original_script.clone();
                
                // 尝试执行脚本的循环
                loop {
                    match execute_command(&current_script) {
                        Ok(out) => {
                            // 成功执行
                            print!("{}", out);
                            break;
                        },
                        Err(error) => {
                            eprintln!("脚本执行出错: {}", error);
                            println!("正在生成修复脚本...");
                            
                            // 将当前失败的脚本添加到历史记录
                            fix_history.push((current_script.clone(), error.clone()));
                            
                            // 调用LLM生成修复脚本
                            match handle_script_error(&original_script, &error, &fix_history, prof).await {
                                Ok(fixed_script) => {
                                    println!("修复后的脚本:\n{}", fixed_script);
                                    print!("是否执行修复后的脚本？(Y/n/q退出)");
                                    io::stdout().flush().unwrap();
                                    let mut fix_answer = String::new();
                                    io::stdin().read_line(&mut fix_answer).unwrap();
                                    
                                    if fix_answer.trim().eq_ignore_ascii_case("q") {
                                        println!("已取消修复过程");
                                        break;
                                    } else if fix_answer.trim().is_empty() || fix_answer.trim().eq_ignore_ascii_case("y") {
                                        // 更新当前脚本为修复后的脚本，继续循环
                                        current_script = fixed_script;
                                        // 循环会继续，尝试执行新的修复脚本
                                    } else {
                                        println!("已取消执行修复脚本");
                                        break;
                                    }
                                },
                                Err(e) => {
                                    eprintln!("生成修复脚本失败: {}", e);
                                    break;
                                }
                            }
                        }
                    }
                }
            } else {
                println!("已取消执行");
            }
        },
        Commands::Ask { prompt } => {
            // 纯对话模式
            let resp = call_ai_api(
                vec![Message {
                    role: "user".into(),
                    content: prompt,
                }],
                prof,
            )
            .await?;

            println!("{}", resp);
        },
        Commands::Alias(_) => unreachable!(), // 已经在上面处理了
    }

    Ok(())
}