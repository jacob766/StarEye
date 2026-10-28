//! Model 根模块：App（应用总状态）+ 四个领域子模块。
//!
//! 按"领域"而非"页面"拆分（页面是 View 的路由事实，不是数据的归属）：
//! - `task`    任务域：阶段/严重度/发现/运行态（总览、发现页消费）
//! - `page`    页签路由：Page（页签条、翻页、注册表消费）
//! - `cmdline` 命令行域：输入态/命令行/控制台行（控制台页与全局键位消费）
//! - `form`    表单域：LLM 配置/服务商/字段/表单草稿（设置页消费）
//!
//! 本文件 `pub use` 再导出全部子模块类型，`crate::tui::model::types::Xxx`
//! 的外部引用路径保持不变——拆模块对 controller / view / orchestration 零侵入。

mod cmdline;
mod form;
mod page;
mod task;

pub use cmdline::*;
pub use form::*;
pub use page::*;
pub use task::*;

use cmdline::CONSOLE_CAP;

/// 应用总状态（M 的根）。本阶段字段少是正常的——它是后续所有动态
/// 数据（AgentEvent、审批队列）的挂载点，现在先把"家"建好。
pub struct App {
    pub page: Page,
    pub run: RunState,
    pub quit: bool,
    pub mode: InputMode,
    pub last_page: Page,     // 进入设置页前的页面，Esc 返回用
    pub cmd: CommandLine,
    pub console: Vec<ConsoleLine>,
    pub help_open: bool,
    pub config: LlmConfig,   // 已保存的 LLM 配置
    pub form: LlmForm,       // 设置页表单草稿
}

impl App {
    pub fn new() -> Self {
        let run = RunState::demo();
        let config = LlmConfig::demo();
        let form = LlmForm::from_run(&run, &config);
        let mut app = Self {
            page: Page::Cockpit,
            run,
            quit: false,
            mode: InputMode::Normal,
            last_page: Page::Cockpit,
            cmd: CommandLine::default(),
            console: Vec::new(),
            help_open: false,
            config,
            form,
        };
        app.seed_console();
        app
    }

    /// 开场白：让控制台页一进来就有"这个工具怎么用"的示范，
    /// 而不是一个空框（示范即文档）。
    fn seed_console(&mut self) {
        self.push_console(LineKind::Sys, "StarEye v0.1 · AI 渗透测试代理（演示数据）");
        self.push_console(LineKind::Sys, "输入 help 查看命令 · 按 : 随时呼出命令行");
        self.push_console(LineKind::Cmd, "scan 10.0.0.0/24");
        self.push_console(LineKind::Out, "↻ nmap -sV --top-ports 1000 10.0.0.0/24 已入队");
        self.push_console(LineKind::Ok, "✓ 侦察完成 · 254 hosts · 扫描进行中 62%");
        self.push_console(LineKind::Cmd, "llm model gpt-4o");
        self.push_console(LineKind::Ok, "✓ 模型已切换: gpt-4o-mini → gpt-4o");
    }

    /// 控制台输出的唯一入口。越界即从头部丢弃——
    /// 调用方不需要知道容量，Capacity 是 Model 自己的家事。
    pub fn push_console(&mut self, kind: LineKind, text: impl Into<String>) {
        self.console.push(ConsoleLine {
            kind,
            text: text.into(),
        });
        if self.console.len() > CONSOLE_CAP {
            let excess = self.console.len() - CONSOLE_CAP;
            self.console.drain(..excess);
        }
    }

    /// 已运行秒数。View 不碰时间源（Instant 不直接暴露给视图），
    /// 由 Model 提供格式化入口，视图只消费字符串——可解释性。
    pub fn elapsed_str(&self) -> String {
        let s = self.run.started_at.elapsed().as_secs();
        format!("{:02}:{:02}:{:02}", s / 3600, (s % 3600) / 60, s % 60)
    }

    pub fn phase_status(&self, p: Phase) -> PhaseStatus {
        // 从 CHAIN 反查下标，而不是到处写 [0],[1],[2]，防止魔法下标
        let idx = Phase::CHAIN.iter().position(|x| *x == p).unwrap();
        self.run.phase_statuses[idx]
    }
}
