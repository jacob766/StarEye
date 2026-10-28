use super::types::*;
use std::time::Instant;

impl RunState {
    /// 演示构造器。注意命名：叫 demo() 而不是 default()，
    /// 明确它只服务于"无真实 Agent 时把界面点亮"，测试与上线都不依赖它的真实性。
    pub fn demo() -> Self {
        Self {
            target: "10.0.0.0/24".into(),
            // 作用域拆成两个字段存，横幅文案由 scope_note() 拼装——
            // 设置页表单改这两个字段，横幅自动跟随（单一真相来源）。
            scope_to: "2026-11-01".into(),
            scope_ex: "*.internal".into(),
            phase: Phase::Scanning,
            phase_statuses: [
                PhaseStatus::Done,
                PhaseStatus::Running { progress: 62 },
                PhaseStatus::Pending,
                PhaseStatus::Pending,
            ],
            started_at: Instant::now(),
            findings: vec![
                Finding { id: 1, severity: Severity::Critical, title: "SQL 注入 /login".into(), target: "10.0.0.5".into() },
                Finding { id: 2, severity: Severity::High,     title: "OpenSSH 8.2p1 已知漏洞".into(), target: "10.0.0.5".into() },
                Finding { id: 3, severity: Severity::Medium,   title: "nginx 版本信息泄露".into(), target: "10.0.0.7".into() },
                Finding { id: 4, severity: Severity::Low,      title: "缺少 CSP 响应头".into(), target: "10.0.0.7".into() },
                Finding { id: 5, severity: Severity::Info,     title: "开放端口 80/443/22".into(), target: "10.0.0.7".into() },
            ],
        }
    }
}
