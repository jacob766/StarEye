use ratatui::style::{Color, Modifier, Style};

use crate::tui::model::types::{LineKind, PhaseStatus, Severity};

/// 主题基色。换主题 = 只改这个函数群，全 app 视觉一致变化（可复用性）。
pub fn chrome() -> Style { Style::new().fg(Color::Black).bg(Color::Rgb(90, 140, 160)) }
pub fn chrome_dim() -> Style { Style::new().fg(Color::DarkGray) }
pub fn panel_border() -> Style { Style::new().fg(Color::DarkGray) }
pub fn selected() -> Style { Style::new().fg(Color::Black).bg(Color::Yellow).add_modifier(Modifier::BOLD) }
pub fn muted() -> Style { Style::new().fg(Color::DarkGray) }
pub fn accent() -> Style { Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD) }

/// 严重度 -> 样式。CVSS 色阶是安全行业约定，集中定义一次。
pub fn severity(s: Severity) -> Style {
    match s {
        Severity::Critical => Style::new().fg(Color::White).bg(Color::Rgb(150, 30, 30)).add_modifier(Modifier::BOLD),
        Severity::High     => Style::new().fg(Color::LightRed),
        Severity::Medium   => Style::new().fg(Color::Yellow),
        Severity::Low      => Style::new().fg(Color::LightBlue),
        Severity::Info     => Style::new().dark_gray(),
    }
}

/// 阶段三态的着色：进行中高亮、完成正常、未开始压暗——
/// 用户扫一眼就知道"球在哪个阶段"。
pub fn phase_status(s: PhaseStatus) -> Style {
    match s {
        PhaseStatus::Running { .. } => Style::new().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        PhaseStatus::Done => Style::new().fg(Color::Green),
        PhaseStatus::Pending => Style::new().dark_gray(),
    }
}

/// 控制台行类型的着色。与严重度色阶并列的第二个"语义色表"：
/// 命令=强调（复用 accent）、输出=压暗、成功=绿、失败=红、系统=青。
/// 命令行交互的全部视觉反馈都收敛到这一个函数。
pub fn line_kind(k: LineKind) -> Style {
    match k {
        LineKind::Cmd => accent(),
        LineKind::Out => muted(),
        LineKind::Ok => Style::new().fg(Color::Green),
        LineKind::Err => Style::new().fg(Color::LightRed),
        LineKind::Sys => Style::new().fg(Color::Cyan),
    }
}
