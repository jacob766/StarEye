use super::task::RunState;

/// LLM 连接配置。temperature 刻意用 String：表单是它的第一现场，
/// 原样编辑、原样展示，避免 f32 解析把 UI 层变复杂（真正调用时再 parse）。
#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub base_url: String,
    pub temperature: String,
}

impl LlmConfig {
    pub fn demo() -> Self {
        Self {
            provider: "OpenAI".into(),
            model: "gpt-4o".into(),
            api_key: "sk-demo1234567890abcd".into(),
            base_url: "https://api.openai.com/v1".into(),
            temperature: "0.2".into(),
        }
    }
}

/// 服务商清单。设置页的 ←→ 循环与保存回查都以它为唯一来源。
pub const PROVIDERS: [&str; 6] = [
    "OpenAI",
    "Anthropic",
    "DeepSeek",
    "通义千问",
    "Ollama（本地）",
    "自定义",
];

/// 设置页的可聚焦元素。ALL 的顺序即 Tab 顺序——
/// 加字段 = 加变体 + 加进 ALL，next/prev 与渲染自动跟上。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Provider,
    Model,
    BaseUrl,
    ApiKey,
    Temp,
    BtnTest,
    BtnSave,
    BtnReset,
    ScopeTo,
    ScopeEx,
    ReadOnly,
    BtnScopeSave,
}

impl Field {
    pub const ALL: [Field; 12] = [
        Field::Provider,
        Field::Model,
        Field::BaseUrl,
        Field::ApiKey,
        Field::Temp,
        Field::BtnTest,
        Field::BtnSave,
        Field::BtnReset,
        Field::ScopeTo,
        Field::ScopeEx,
        Field::ReadOnly,
        Field::BtnScopeSave,
    ];

    pub fn idx(self) -> usize {
        Field::ALL.iter().position(|f| *f == self).unwrap()
    }

    pub fn next(self) -> Field {
        Field::ALL[(self.idx() + 1) % Field::ALL.len()]
    }

    pub fn prev(self) -> Field {
        Field::ALL[(self.idx() + Field::ALL.len() - 1) % Field::ALL.len()]
    }

    pub fn label(self) -> &'static str {
        match self {
            Field::Provider => "服务商",
            Field::Model => "模型",
            Field::BaseUrl => "Base URL",
            Field::ApiKey => "API Key",
            Field::Temp => "温度",
            Field::BtnTest => "测试连接",
            Field::BtnSave => "保存",
            Field::BtnReset => "恢复默认",
            Field::ScopeTo => "授权到期",
            Field::ScopeEx => "排除规则",
            Field::ReadOnly => "只读模式",
            Field::BtnScopeSave => "保存作用域",
        }
    }
}

/// 设置页表单状态。草稿与已保存配置（App.config）分离：
/// 只有按「保存」才落库，Esc 离开不丢草稿——和网页表单一个心智模型。
#[derive(Debug, Clone)]
pub struct LlmForm {
    pub provider: usize, // PROVIDERS 下标
    pub model: String,
    pub base_url: String,
    pub api_key: String,
    pub temperature: String,
    pub scope_to: String,
    pub scope_ex: String,
    pub read_only: bool,
    pub focus: Field,
    pub cursor: usize, // 聚焦文本字段内的字符光标（UTF-8 字符数，不是字节）
    pub status: String,
    pub status_ok: bool,
    pub scope_status: String,
    pub scope_status_ok: bool,
}

impl LlmForm {
    pub fn from_run(run: &RunState, cfg: &LlmConfig) -> Self {
        Self {
            provider: PROVIDERS
                .iter()
                .position(|p| *p == cfg.provider.as_str())
                .unwrap_or(0),
            model: cfg.model.clone(),
            base_url: cfg.base_url.clone(),
            api_key: cfg.api_key.clone(),
            temperature: cfg.temperature.clone(),
            scope_to: run.scope_to.clone(),
            scope_ex: run.scope_ex.clone(),
            read_only: false,
            focus: Field::Provider,
            cursor: 0,
            status: "尚未测试".into(),
            status_ok: true,
            scope_status: "保存后即时更新顶部作用域横幅".into(),
            scope_status_ok: true,
        }
    }

    /// 聚焦字段的文本内容。按钮/开关/下拉没有文本，返回 None——
    /// 调用方（Controller 的编辑逻辑）因此不需要 match 每个字段。
    pub fn focused_text(&self) -> Option<&str> {
        match self.focus {
            Field::Model => Some(&self.model),
            Field::BaseUrl => Some(&self.base_url),
            Field::ApiKey => Some(&self.api_key),
            Field::Temp => Some(&self.temperature),
            Field::ScopeTo => Some(&self.scope_to),
            Field::ScopeEx => Some(&self.scope_ex),
            _ => None,
        }
    }

    pub fn focused_text_mut(&mut self) -> Option<&mut String> {
        match self.focus {
            Field::Model => Some(&mut self.model),
            Field::BaseUrl => Some(&mut self.base_url),
            Field::ApiKey => Some(&mut self.api_key),
            Field::Temp => Some(&mut self.temperature),
            Field::ScopeTo => Some(&mut self.scope_to),
            Field::ScopeEx => Some(&mut self.scope_ex),
            _ => None,
        }
    }

    /// API Key 的展示值：聚焦时显示明文（方便修改），否则打码——
    /// 和网页密码框一个规则，但不需要额外的"显示/隐藏"按钮。
    pub fn masked_api_key(&self) -> String {
        if self.focus == Field::ApiKey {
            self.api_key.clone()
        } else {
            mask_key(&self.api_key)
        }
    }
}

/// Key 打码：保留头 6 尾 4，中间 bullet。短 key 全打码（没有可保留的辨识度）。
pub fn mask_key(key: &str) -> String {
    let chars: Vec<char> = key.chars().collect();
    if chars.len() <= 8 {
        return "•".repeat(chars.len());
    }
    let head: String = chars.iter().take(6).collect();
    let tail: String = chars.iter().skip(chars.len() - 4).collect();
    format!("{head}••••••••{tail}")
}
