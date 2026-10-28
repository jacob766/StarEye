//! 命令词表与分发。
//!
//! 这个模块是"命令"的唯一真相来源：名字、描述、用法示例、补全规则、
//! 执行后果都在这里。它不认识 App——执行结果以 `Outcome`（输出行 + 动作）
//! 表达，由 Controller 作用到 Model。于是：
//! - 补全（View 读）、分发（Controller 调）、测试（直接调 run）三方消费同一份数据；
//! - 加命令 = 加一条 COMMANDS + 加一个 match 分支，编译器逼你补全。

use crate::tui::model::types::{ConsoleLine, Field, LineKind, Page};

/// 一条命令的静态描述。name 是补全匹配的主键，usage 是参数示例。
pub struct CommandSpec {
    pub name: &'static str,
    pub desc: &'static str,
    pub usage: Option<&'static str>,
}

/// 命令词表。顺序即补全浮层的稳定顺序（不做花哨排序，可预期性优先）。
pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec { name: "help",       desc: "查看全部命令与快捷键", usage: None },
    CommandSpec { name: "config",     desc: "打开设置页", usage: None },
    CommandSpec { name: "config llm", desc: "打开设置页并定位 LLM 配置", usage: None },
    CommandSpec { name: "scan",       desc: "启动扫描", usage: Some("scan 10.0.0.5") },
    CommandSpec { name: "llm model",  desc: "切换模型", usage: Some("llm model gpt-4o-mini") },
    CommandSpec { name: "llm key",    desc: "设置 API Key", usage: Some("llm key sk-...") },
    CommandSpec { name: "llm test",   desc: "测试 LLM 连通性", usage: None },
    CommandSpec { name: "findings",   desc: "查看 / 过滤发现", usage: Some("findings --sev crit") },
    CommandSpec { name: "activity",   desc: "查看活动流", usage: None },
    CommandSpec { name: "scope",      desc: "查看当前授权作用域", usage: None },
    CommandSpec { name: "export",     desc: "导出报告", usage: Some("export report.md") },
    CommandSpec { name: "quit",       desc: "退出", usage: None },
];

/// 补全：返回 COMMANDS 下标的列表。
/// 规则——前缀命中优先（按词表顺序），其后包含命中；空前缀返回全部；最多 7 条
/// （浮层高度预算，超出截断）。纯函数：同样的输入永远得到同样的补全。
pub fn complete_indices(prefix: &str) -> Vec<usize> {
    let q = prefix
        .trim()
        .trim_start_matches([':', '/'])
        .trim()
        .to_lowercase();
    let mut starts = Vec::new();
    let mut contains = Vec::new();
    for (i, c) in COMMANDS.iter().enumerate() {
        let name = c.name.to_lowercase();
        if q.is_empty() || name.starts_with(&q) {
            starts.push(i);
        } else if name.contains(&q) {
            contains.push(i);
        }
    }
    starts.extend(contains);
    starts.truncate(7);
    starts
}

/// 命令执行所需的只读上下文。从 App.run / App.config 借道而来——
/// 本模块因此不需要认识 App，也不可变更任何状态。
pub struct CommandCtx<'a> {
    pub target: &'a str,
    pub scope_note: &'a str,
    pub model: &'a str,
    pub base_url: &'a str,
}

/// 命令的副作用。命令本身不改 Model，只"请求"一个动作，
/// 由 Controller 决定如何落地（保持"唯一合法变更点"的约定）。
pub enum Action {
    None,
    OpenHelp,
    Goto(Page),
    FocusField(Page, Field),
    SetModel(String),
    SetApiKey(String),
    TestLlm,
    Quit,
}

/// 一次命令执行的全部结果：若干输出行 + 至一个动作。
pub struct Outcome {
    pub lines: Vec<ConsoleLine>,
    pub action: Action,
}

impl Outcome {
    fn line(kind: LineKind, text: impl Into<String>) -> Self {
        Self {
            lines: vec![ConsoleLine {
                kind,
                text: text.into(),
            }],
            action: Action::None,
        }
    }
    fn ok(text: impl Into<String>) -> Self {
        Self::line(LineKind::Ok, text)
    }
    fn err(text: impl Into<String>) -> Self {
        Self::line(LineKind::Err, text)
    }
    fn sys(text: impl Into<String>) -> Self {
        Self::line(LineKind::Sys, text)
    }
    fn with(mut self, action: Action) -> Self {
        self.action = action;
        self
    }
}

/// 分发入口。raw 是用户原样输入（允许带 : 或 / 前缀），ctx 提供只读事实。
pub fn run(raw: &str, ctx: &CommandCtx) -> Outcome {
    let norm = raw.trim().trim_start_matches([':', '/']).trim();
    if norm.is_empty() {
        return Outcome::line(LineKind::Out, "");
    }
    let (head, arg) = match norm.split_once(' ') {
        Some((h, a)) => (h, a.trim()),
        None => (norm, ""),
    };
    match head {
        "help" => Outcome::sys("已显示命令帮助（Esc 关闭）").with(Action::OpenHelp),
        "config" => {
            if arg == "llm" {
                Outcome::sys("→ 设置 · LLM 配置")
                    .with(Action::FocusField(Page::Settings, Field::Model))
            } else {
                Outcome::sys("→ 已打开设置页").with(Action::Goto(Page::Settings))
            }
        }
        "scan" => {
            if arg.is_empty() {
                Outcome::err("✗ 缺少目标 · 用法: scan <cidr|host>")
            } else {
                Outcome::sys(format!("↻ 已入队: nmap -sV --top-ports 1000 {arg}"))
                    .with(Action::Goto(Page::Activity))
            }
        }
        "llm" => {
            let (sub, val) = match arg.split_once(' ') {
                Some((s, v)) => (s, v.trim()),
                None => (arg, ""),
            };
            match sub {
                "model" if !val.is_empty() => {
                    Outcome::ok(format!("✓ 模型已切换: {} → {}", ctx.model, val))
                        .with(Action::SetModel(val.to_string()))
                }
                "key" if !val.is_empty() => {
                    Outcome::ok("✓ API Key 已更新（演示：仅内存）")
                        .with(Action::SetApiKey(val.to_string()))
                }
                "test" => Outcome {
                    lines: vec![
                        ConsoleLine {
                            kind: LineKind::Sys,
                            text: format!("↻ 正在测试 {} ...", ctx.base_url),
                        },
                        ConsoleLine {
                            kind: LineKind::Ok,
                            text: format!("✓ 连接成功 · 模型 {} 可用 · 延迟 312ms", ctx.model),
                        },
                    ],
                    action: Action::TestLlm,
                },
                _ => Outcome::err("✗ 用法: llm model <name> | llm key <sk-...> | llm test"),
            }
        }
        "findings" => {
            let note = if arg.is_empty() {
                "→ 发现列表".to_string()
            } else {
                format!("→ 发现 · 过滤 {arg}")
            };
            Outcome::sys(note).with(Action::Goto(Page::Findings))
        }
        "activity" => Outcome::sys("→ 活动流").with(Action::Goto(Page::Activity)),
        "scope" => Outcome::line(
            LineKind::Out,
            format!("{} · 目标 {}", ctx.scope_note, ctx.target),
        ),
        "export" => {
            if arg.is_empty() {
                Outcome::err("✗ 用法: export <file>")
            } else {
                Outcome::ok(format!("✓ 报告已导出: ./{arg}"))
            }
        }
        "quit" | "q" => Outcome::sys("（演示原型不会真正退出）").with(Action::Quit),
        _ => Outcome::err(format!("✗ 未知命令: {head} · 输入 help 查看可用命令")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> CommandCtx<'static> {
        CommandCtx {
            target: "10.0.0.0/24",
            scope_note: "授权: 2026-10-01 ~ 2026-11-01 · 排除 *.internal",
            model: "gpt-4o",
            base_url: "https://api.openai.com/v1",
        }
    }

    #[test]
    fn empty_prefix_lists_all_capped() {
        // ':' 唤出命令行时应看到全表（截断到 7 条，匹配浮层高度预算）
        let idx = complete_indices(":");
        assert_eq!(idx.len(), 7);
        assert_eq!(COMMANDS[idx[0]].name, "help");
    }

    #[test]
    fn prefix_match_beats_contains() {
        // "llm" 前缀命中 3 条 llm 家族命令；"config llm" 作为包含命中排在
        // 它们之后——前缀优先于包含，且两组都不打乱词表原始顺序。
        let idx = complete_indices("llm");
        assert_eq!(idx.len(), 4);
        assert_eq!(COMMANDS[idx[0]].name, "llm model");
        assert_eq!(COMMANDS[idx[1]].name, "llm key");
        assert_eq!(COMMANDS[idx[2]].name, "llm test");
        assert_eq!(COMMANDS[idx[3]].name, "config llm");
    }

    #[test]
    fn contains_fallback() {
        // "port" 不是任何命令的前缀，但 "export" 包含它——兜底分支生效
        let idx = complete_indices("port");
        assert_eq!(idx.len(), 1);
        assert_eq!(COMMANDS[idx[0]].name, "export");
    }

    #[test]
    fn run_llm_model_switches() {
        let out = run("llm model gpt-4o-mini", &ctx());
        assert!(matches!(out.action, Action::SetModel(ref m) if m == "gpt-4o-mini"));
        assert_eq!(out.lines[0].kind, LineKind::Ok);
        assert!(out.lines[0].text.contains("模型已切换"));
    }

    #[test]
    fn run_unknown_reports_error() {
        let out = run("foo bar", &ctx());
        assert!(matches!(out.action, Action::None));
        assert_eq!(out.lines[0].kind, LineKind::Err);
        assert!(out.lines[0].text.contains("未知命令"));
    }

    #[test]
    fn run_empty_is_silent() {
        let out = run("  :  ", &ctx());
        assert!(out.lines[0].text.is_empty());
        assert!(matches!(out.action, Action::None));
    }

    #[test]
    fn run_quit() {
        assert!(matches!(run("quit", &ctx()).action, Action::Quit));
        assert!(matches!(run("q", &ctx()).action, Action::Quit));
    }

    #[test]
    fn run_scan_requires_arg() {
        let bad = run("scan", &ctx());
        assert!(matches!(bad.action, Action::None));
        assert_eq!(bad.lines[0].kind, LineKind::Err);

        let good = run("scan 10.0.0.5", &ctx());
        assert!(matches!(good.action, Action::Goto(Page::Activity)));
        assert!(good.lines[0].text.contains("10.0.0.5"));
    }

    #[test]
    fn run_config_llm_focuses_model() {
        let out = run(":config llm", &ctx());
        assert!(matches!(
            out.action,
            Action::FocusField(Page::Settings, Field::Model)
        ));
    }

    #[test]
    fn run_help_opens_overlay() {
        let out = run("help", &ctx());
        assert!(matches!(out.action, Action::OpenHelp));
    }

    #[test]
    fn run_llm_test_uses_ctx() {
        let out = run("llm test", &ctx());
        assert!(matches!(out.action, Action::TestLlm));
        assert_eq!(out.lines.len(), 2);
        assert!(out.lines[1].text.contains("gpt-4o"));
    }
}
