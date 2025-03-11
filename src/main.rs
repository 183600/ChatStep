use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::process::Command;

// 命令行参数解析结构
#[derive(Parser)]
#[command(name = "AI Shell Assistant")]
#[command(version = "1.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>, // 支持子命令
}

// 子命令定义
#[derive(Subcommand)]
enum Commands {
    Run { // 执行命令
        prompt: String, // 用户输入的指令
    },
}

// AI 消息结构（实现序列化/反序列化）
#[derive(Debug, Serialize, Deserialize)]
struct Message {
    role: String,    // 角色：user/assistant
    content: String,  // 消息内容
}

// 调用AI API的核心函数
async fn call_ai_api(messages: Vec<Message>) -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let api_key = env::var("QWQ_API_KEY").expect("QWQ_API_KEY必须设置");
    
    // 构造请求体
    let response = client
        .post("https://api.suanli.cn/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&ChatRequest {
            model: "free:QwQ-32B".to_string(),
            messages,
        })
        .send()
        .await?
        .json::<ChatResponse>()
        .await?;

    Ok(response.choices[0].message.content.trim().to_string())
}

// 执行shell命令的函数
fn execute_command(command: &str) -> String {
    let output = if cfg!(target_os = "windows") { // 根据系统选择shell
        Command::new("cmd").args(["/C", command]).output()
    } else {
        Command::new("sh").arg("-c").arg(command).output()
    };

    String::from_utf8_lossy(&output.stdout).to_string()
}

// 主函数
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok(); // 加载.env文件
    let cli = Cli::parse(); // 解析命令行参数

    if let Some(Commands::Run { prompt }) = cli.command {
        let is_command = Regex::new(r"(获取|分析|执行|检查|查看)")?.is_match(&prompt);
        
        if is_command {
            // 分步处理逻辑
            let step_prompt = format!("將以下指令拆解為具體步驟...");
            let steps_response = call_ai_api(vec![/* 构造消息 */]).await?;
            
            let steps = parse_steps(&steps_response); // 解析步骤
            let mut context = String::new(); // 保存执行上下文
            
            for step in steps {
                if step.needs_execution {
                    // 生成并执行命令
                    let command = call_ai_api(/* 构造命令请求 */).await?;
                    let output = execute_command(&command);
                    context.push_str(&output); // 保存输出
                } else {
                    // 直接发送分析请求
                    let result = call_ai_api(/* 构造分析请求 */).await?;
                    context = result; // 更新上下文
                }
            }
        } else {
            // 普通对话模式
            let result = call_ai_api(vec![Message {
                role: "user".to_string(),
                content: prompt,
            }]).await?;
            println!("AI回复：\n{}", result);
        }
    }
    Ok(())
}
