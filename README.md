# AutoCommit

AutoCommit 是一个自动生成 Git 提交信息的命令行工具。它使用 AI 模型分析您的 Git diff，并生成符合约定的提交信息。

## 功能

- 自动分析 Git diff
- 使用 AI 模型生成提交信息
- 支持自定义提示词配置
- 支持从文件导入 diff
- 支持流式输出生成的提交信息

## 命令行参数

AutoCommit 支持以下命令行参数：

```
Usage: autocommit [OPTIONS]

Options:
  -c, --config    配置文件路径（可选）, 缺省时会从当前目录和autodiff程序所在的目录中寻找"config.json"
  -p, --prompt    选择预设提示词（可选）, 缺省时为"default"
  -f, --file      从文件导入 diff（可选）, 缺省时使用当前目录的`git diff --cached`结果
  -h, --help      显示帮助信息
```

## 配置文件结构

配置文件使用 JSON 格式，包含以下字段：

```json
{
  "api_url": "API 端点 URL",
  "api_key": "API 密钥",
  "model": "使用的 AI 模型名称",
  "prompts": [
    {
      "name": "预设名称",
      "prompt": [
        {
          "role": "角色（system/user）",
          "content": [
            "提示词内容，可以是多行"
          ]
        }
      ]
    }
  ]
}
```

### 字段说明

- `api_url`: AI API 的端点 URL
- `api_key`: 用于认证的 API 密钥
- `model`: 要使用的 AI 模型名称
- `prompts`: 预设提示词数组
  - `name`: 预设名称，用于命令行参数中选择
  - `prompt`: 提示词数组
    - `role`: 消息角色，可以是 "system" 或 "user"
    - `content`: 提示词内容，可以是多行字符串数组

您可以在配置文件中定义多个预设提示词，并通过命令行参数 `-p` 或 `--prompt` 来选择使用哪个预设。
