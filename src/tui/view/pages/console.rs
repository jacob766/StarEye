use ratatui::prelude::*;
use ratatui::widgets::{List, ListItem};

use crate::tui::model::types::{App, LineKind};
use crate::tui::view::{chrome, palette};

pub(super) fn render_console(app: &App, frame: &mut Frame, area: Rect) {
    // 面板吃掉上下各 1 行边框，可见行数据此计算；超出部分从顶部裁掉，
    // 保证最新输出永远可见（终端 tail 行为）。
    let visible = area.height.saturating_sub(2) as usize;
    let items: Vec<ListItem> = if app.console.is_empty() {
        vec![ListItem::new(Line::from(Span::styled(
            "（暂无输出 · 按 : 输入命令）",
            palette::muted(),
        )))]
    } else {
        let start = app.console.len().saturating_sub(visible);
        app.console[start..]
            .iter()
            .map(|l| {
                let mut spans = Vec::new();
                if l.kind == LineKind::Cmd {
                    spans.push(Span::styled("❯ ", palette::line_kind(LineKind::Cmd)));
                }
                spans.push(Span::styled(l.text.as_str(), palette::line_kind(l.kind)));
                ListItem::new(Line::from(spans))
            })
            .collect()
    };
    let list = List::new(items).block(chrome::panel("控制台 · 命令输出历史"));
    frame.render_widget(list, area);
}
