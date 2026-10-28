use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::tui::model::types::{App, Page};
use crate::tui::view::{chrome, palette};

mod cockpit;
mod activity;
mod findings;
mod console;
mod settings;

/// "页面渲染器"的签名。所有页面视图共享这一个形状。
/// 注意：传 &App（只读）；页面若有内部滚动等状态，从 Model 的 focus 字段
/// 里拿——视图层永远不 new 状态。
pub type PageRender = fn(&App, &mut Frame, Rect);

/// 注册表：Page -> 渲染器。这就是策略模式的"表"。
/// 新增一个"资产图谱"页的完整成本：Page 加变体 + 这里加一行 +
/// 写一个 render_assets 函数。其他文件零改动——开闭原则。
pub fn renderer(page: Page) -> PageRender {
    match page {
        Page::Cockpit => cockpit::render_cockpit,
        Page::Activity => activity::render_activity,
        Page::Findings => findings::render_findings,
        Page::Console => console::render_console,
        Page::Settings => settings::render_settings,
    }
}

/// 通用占位：告诉观者"这里将来是什么"，并显示该页关键数据摘要，
/// 让布局验证有意义。活动流/发现两页共用这一个助手。
pub(super) fn placeholder(frame: &mut Frame, app: &App, area: Rect, title: &str, note: &str) {
    let body = format!(
        "{note}\n\n当前数据: {} 个发现 / 阶段 {} / 耗时 {}",
        app.run.findings.len(),
        app.run.phase.label(),
        app.elapsed_str(),
    );
    let p = Paragraph::new(body)
        .block(chrome::panel(title))
        .style(palette::muted());
    frame.render_widget(p, area);
}
