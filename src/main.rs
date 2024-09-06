use clap::Parser;
use futures::stream::StreamExt;
use reqwest::{self, Client};
use serde::{Deserialize, Serialize};
use std::env::current_exe;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// 配置文件
    #[arg(short, long)]
    config: Option<std::path::PathBuf>,

    /// 选择预设
    #[arg(short, long)]
    prompt: Option<String>,

    /// 从文件导入diff (Optional)
    #[arg(short, long)]
    file: Option<std::path::PathBuf>,
}

#[derive(Deserialize, Debug)]
struct Prompt {
    role: String,
    content: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct PromptConfig {
    name: String,
    prompt: Vec<Prompt>,
}

#[derive(Deserialize, Debug)]
struct Config {
    api_url: String,
    api_key: String,
    model: String,
    prompts: Vec<PromptConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct RequestBody {
    model: String,
    messages: Vec<Message>,
    stream: bool,
}

#[tokio::main]
async fn main() -> Result<(), ()> {
    let args = Args::parse();

    let mut config_file;
    if let Some(config) = args.config {
        config_file = config;
    } else {
        config_file = PathBuf::from("config.json");
        if !config_file.exists() {
            config_file = current_exe()
                .unwrap()
                .parent()
                .unwrap()
                .to_path_buf()
                .join("config.json");
        }
    }

    if !config_file.exists() {
        panic!("can't find config file");
    }

    // 获得 diff
    let diff;
    if let Some(diff_file) = args.file {
        diff = get_diff_from_file(&diff_file);
    } else {
        diff = get_diff_from_context();
    }
    // println!("diff: {}", diff);

    // 打开配置文件
    let mut file = File::open(config_file).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    // 解析配置文件数据
    let config: Config = serde_json::from_str(&data).unwrap();

    // 选择prompt
    let mut messages = get_messages(
        &config,
        args.prompt.unwrap_or("default".to_string()).as_str(),
    );

    // 替换diff内容
    for message in messages.iter_mut() {
        message.content = message.content.replace("%{diff}", diff.as_str());
        // println!("role: {}", message.role);
        // println!("content: {}", message.content);
    }

    // 创建请求
    let request_body = RequestBody {
        model: config.model.clone(),
        messages,
        stream: true,
    };

    // 生成commit message
    generate_commit(&config, &request_body).await;

    Ok(())
}

fn get_messages(config: &Config, name: &str) -> Vec<Message> {
    if let Some(found) = config.prompts.iter().find(|prompt| prompt.name == name) {
        let mut result: Vec<Message> = Vec::new();
        for prompt in found.prompt.iter() {
            result.push(Message {
                role: prompt.role.clone(),
                content: prompt.content.join("\n"),
            })
        }
        return result;
    }
    panic!("can't find prompt {}", name);
}

fn get_diff_from_context() -> String {
    let output = std::process::Command::new("git")
        .arg("diff")
        .arg("--cached")
        .output()
        .expect("Failed to execute git diff command");

    let diff = String::from_utf8_lossy(&output.stdout).to_string();
    if diff.is_empty() {
        panic!("no diff content");
    }

    diff
}

fn get_diff_from_file(diff_file: &PathBuf) -> String {
    let mut file = File::open(diff_file).unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    data
}

async fn generate_commit(config: &Config, request_body: &RequestBody) -> String {
    // 创建 HTTP 客户端
    let client = Client::new();

    // 创建请求正文
    let request_body = serde_json::to_string(&request_body).unwrap();
    // println!("{}", request_body.as_str());

    // 发起 POST 请求
    let response = client
        .post(config.api_url.as_str())
        .header(
            "Authorization",
            format!("Bearer {}", config.api_key.as_str()),
        )
        .header("Content-Type", "application/json")
        .body(request_body)
        .send()
        .await
        .unwrap();

    let mut result = String::new();

    // 检查响应状态
    if response.status().is_success() {
        // 逐块读取响应体
        let mut stream = response.bytes_stream();

        while let Some(chunk) = stream.next().await {
            match chunk {
                Ok(data) => {
                    // 将收到的字节流解码为字符串
                    let response = String::from_utf8_lossy(&data).trim().to_string();
                    if !response.is_empty() {
                        for text in response.split('\n') {
                            let text = text.trim();
                            if text.is_empty() || !text.starts_with("data:") {
                                continue;
                            }
                            let sub = text[5..text.len()].trim();
                            if sub == "[DONE]" {
                                println!("");
                                continue;
                            }
                            let json: serde_json::Value = serde_json::from_str(sub).unwrap();
                            // println!("{}", sub);
                            if let Some(choices) = json["choices"].as_array() {
                                for choice in choices.iter() {
                                    if let Some(delta) = choice["delta"].as_object() {
                                        if !delta.is_empty() {
                                            if let Some(content) = delta["content"].as_str() {
                                                if !content.is_empty() {
                                                    result.push_str(content);
                                                    print!("{}", content);
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error reading chunk: {:?}", err);
                }
            }
        }
    } else {
        eprintln!("Error: {}", response.status());
    }

    result
}
