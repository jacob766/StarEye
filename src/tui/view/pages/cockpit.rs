use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::tui::model::types::App;
use crate::tui::view::{chrome, layout, palette};

pub(super) fn render_cockpit(app: &App, frame: &mut Frame, area: Rect) {
    // 纵向切分：上部攻击链+战果，下部快捷指令提示带（高 3 行）
    let [main, hints] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(area);
    let [chain, stats] = layout::cockpit(main);
    // ratatui 的组合方式是 builder：每个 widget 都有 .block(Block) 方法，
    // chrome::panel() 是统一边框的唯一入口。这里不需要也不存在 block_()——
    // 部件（phase_chain 产出）与边框（panel 产出）在调用点组装。
    frame.render_widget(chrome::phase_chain(app).block(chrome::panel("攻击链")), chain);
    frame.render_widget(chrome::severity_stats(app), stats);

    // 快捷指令带：把最常用的几条命令摆在明处。
    // 它不是装饰——是命令行的"发现性"保障：用户先看到能敲什么，再学会敲。
    let hint = Paragraph::new(Line::from(vec![
        Span::raw(" "),
        Span::styled("❯ ", palette::accent()),
        Span::raw("scan 10.0.0.5     "),
        Span::styled("❯ ", palette::accent()),
        Span::raw(":config llm     "),
        Span::styled("❯ ", palette::accent()),
        Span::raw("llm test     "),
        Span::styled("❯ ", palette::accent()),
        Span::raw("export report.md"),
    ]))
    .block(chrome::panel("快捷指令 · 按 : 在底部命令行输入"));
    frame.render_widget(hint, hints);
}
