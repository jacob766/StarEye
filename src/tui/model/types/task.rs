use std::time::Instant;

/// 渗透任务的宏观阶段。用枚举而不是字符串，是为了让"阶段"成为
/// 编译器可检查的封闭集合——漏掉一个分支，match 就报错（可验证性）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    Recon,      // 侦察：资产发现
    Scanning,   // 扫描：端口/服务/漏洞探测
    Exploit,    // 验证：漏洞可利用性确认
    Report,     // 报告：结果汇总导出
}

impl Phase {
    /// 页面上显示的中文名。视图只调这个方法，不 match 枚举——
    /// 把"标签"收进 Model，避免中英文标签在多处漂移。
    pub fn label(self) -> &'static str {
        match self {
            Phase::Recon => "侦察",
            Phase::Scanning => "扫描",
            Phase::Exploit => "验证",
            Phase::Report => "报告",
        }
    }

    /// 攻击链的固定顺序，供进度条渲染。顺序只有一个真相来源。
    pub const CHAIN: [Phase; 4] = [Phase::Recon, Phase::Scanning, Phase::Exploit, Phase::Report];
}

/// 一个阶段可能处于的三种状态。三态而非 bool：
/// "未开始"和"进行中"在视觉上完全不同（灰 vs 高亮）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PhaseStatus {
    Pending,
    Running { progress: u8 },  // 0..=100，进度放进状态里，天然杜绝"Running 却没进度字段"的非法组合
    Done,
}

impl PhaseStatus {
    pub fn icon(self) -> &'static str {
        match self {
            PhaseStatus::Pending => "·",
            PhaseStatus::Running { .. } => "●",
            PhaseStatus::Done => "✓",
        }
    }

    /// 可观测性：非终态需要动效提示，View 用它决定是否闪烁
    pub fn is_active(self) -> bool {
        matches!(self, PhaseStatus::Running { .. })
    }
}

/// 严重度。行业约定必须用枚举承载，后续表格、统计、审批都消费它。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl Severity {
    pub fn label(self) -> &'static str {
        match self {
            Severity::Info => "INFO",
            Severity::Low => "LOW",
            Severity::Medium => "MED",
            Severity::High => "HIGH",
            Severity::Critical => "CRIT",
        }
    }
}

/// 一个发现条目（本阶段只存最小字段，后续往这里加）。
#[derive(Debug, Clone)]
pub struct Finding {
    pub id: u32,
    pub severity: Severity,
    pub title: String,
    pub target: String,
}

/// 任务运行态：把"会变的事实"聚在一个结构体里，App 拥有它。
#[derive(Debug, Clone)]
pub struct RunState {
    pub target: String,              // 作用域横幅的主内容
    pub scope_to: String,            // 授权到期日
    pub scope_ex: String,            // 排除规则
    pub phase: Phase,
    pub phase_statuses: [PhaseStatus; 4], // 与 Phase::CHAIN 一一对应，下标语义固定
    pub started_at: Instant,
    pub findings: Vec<Finding>,
}

/// 授权起始日。演示数据的事实来源之一，和 fixtures 配套。
pub const SCOPE_FROM: &str = "2026-10-01";

impl RunState {
    /// 作用域横幅文案。由字段拼装而不是存一整串——横幅、设置页表单、
    /// scope 命令三处消费同一份字段，杜绝"改了表单、横幅没变"的漂移。
    pub fn scope_note(&self) -> String {
        format!("授权: {SCOPE_FROM} ~ {} · 排除 {}", self.scope_to, self.scope_ex)
    }
}
