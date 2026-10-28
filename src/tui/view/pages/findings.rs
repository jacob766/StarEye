use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem};

use crate::tui::model::types::App;
use crate::tui::view::{chrome, palette};

use super::placeholder;

pub(super) fn render_findings(app: &App, frame: &mut Frame, area: Rect) {
    // 空态设计：没有数据时显示引导语，而不是空白框（决策点 3 的预演）。
    if app.run.findings.is_empty() {
        placeholder(frame, app, area, "发现", "暂无发现。Agent 确认的漏洞会在此列出。");
        return;
    }
    // 静态阶段就用 demo 数据渲染真实列表：Severity 着色在这里被亲眼验证，
    // Finding 的每个字段都被真实消费（不留给"以后再说"的死代码）。
    let items: Vec<ListItem> = app
        .run
        .findings
        .iter()
        .map(|f| {
            ListItem::new(Line::from(vec![
                Span::styled(format!("{:>4} ", f.id), palette::muted()),
                Span::styled(format!("{:<4} ", f.severity.label()), palette::severity(f.severity)),
                Span::raw(format!("{:<28} ", f.title)),
                Span::styled(f.target.as_str(), palette::muted()),
            ]))
        })
        .collect();
    let list = List::new(items).block(chrome::panel("发现"));
    frame.render_widget(list, area);
}
