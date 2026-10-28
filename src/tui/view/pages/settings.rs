use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

use crate::tui::model::types::{App, Field, LineKind, PROVIDERS};
use crate::tui::view::{chrome, palette};

pub(super) fn render_settings(app: &App, frame: &mut Frame, area: Rect) {
    let [left, right] = Layout::horizontal([
        Constraint::Percentage(56),
        Constraint::Percentage(44),
    ])
    .areas(area);
    frame.render_widget(llm_panel(app), left);
    frame.render_widget(scope_panel(app), right);
}

const LBL_W: usize = 9;  // 标签列宽（"Base URL" 刚好 8 字符）
const VAL_W: usize = 26; // 值框内宽

fn llm_panel(app: &App) -> Paragraph<'_> {
    let f = &app.form;
    let lines = vec![
        select_row(app, Field::Provider, &format!("{} ▾", PROVIDERS[f.provider])),
        text_row(app, Field::Model, &f.model),
        text_row(app, Field::BaseUrl, &f.base_url),
        text_row(app, Field::ApiKey, &f.masked_api_key()),
        text_row(app, Field::Temp, &f.temperature),
        Line::from(""),
        buttons_row(app, &[(Field::BtnTest, "测试连接"), (Field::BtnSave, "保存"), (Field::BtnReset, "恢复默认")]),
        Line::from(Span::styled(
            f.status.as_str(),
            if f.status_ok { palette::line_kind(LineKind::Ok) } else { palette::line_kind(LineKind::Err) },
        )),
    ];
    Paragraph::new(lines).block(chrome::panel("LLM 配置"))
}

fn scope_panel(app: &App) -> Paragraph<'_> {
    let f = &app.form;
    let lines = vec![
        text_row(app, Field::ScopeTo, &f.scope_to),
        text_row(app, Field::ScopeEx, &f.scope_ex),
        toggle_row(app, Field::ReadOnly, f.read_only, "只探测，不执行验证"),
        Line::from(""),
        buttons_row(app, &[(Field::BtnScopeSave, "保存作用域")]),
        Line::from(Span::styled(
            f.scope_status.as_str(),
            if f.scope_status_ok {
                palette::line_kind(LineKind::Ok)
            } else {
                palette::line_kind(LineKind::Err)
            },
        )),
    ];
    Paragraph::new(lines).block(chrome::panel("作用域与授权"))
}

/// 文本字段行。聚焦行整行反色（黑字黄底 = selected()），光标是行内唯一的
/// 黑色块——黄底上的黑 █，不引入第二种聚焦样式。
fn text_row(app: &App, field: Field, value: &str) -> Line<'static> {
    let label = format!("{:<width$}", field.label(), width = LBL_W);
    let focused = app.form.focus == field;
    if !focused {
        return Line::from(vec![
            Span::styled(label, palette::muted()),
            Span::raw(" ["),
            Span::raw(fit(value, VAL_W)),
            Span::raw("]"),
        ]);
    }
    let cur = app.form.cursor.min(VAL_W);
    let chars: Vec<char> = value.chars().collect();
    let head: String = chars.iter().take(cur).collect();
    let tail: String = chars.iter().skip(cur).collect();
    let sel = palette::selected();
    Line::from(vec![
        Span::styled(label, sel),
        Span::styled(" [", sel),
        Span::styled(fit(&head, VAL_W), sel),
        Span::styled("█", Style::new().fg(Color::Black)), // 光标：黄底上的黑块
        Span::styled(fit(&tail, VAL_W.saturating_sub(head.chars().count())), sel),
        Span::styled("]", sel),
    ])
}

/// 下拉字段行：值后带 ▾ 提示可循环；←→ 在 Controller 里循环选项。
fn select_row(app: &App, field: Field, value: &str) -> Line<'static> {
    let label = format!("{:<width$}", field.label(), width = LBL_W);
    let focused = app.form.focus == field;
    if focused {
        let sel = palette::selected();
        Line::from(vec![
            Span::styled(label, sel),
            Span::styled(format!(" [{}]", fit(value, VAL_W)), sel),
        ])
    } else {
        Line::from(vec![
            Span::styled(label, palette::muted()),
            Span::raw(" ["),
            Span::raw(fit(value, VAL_W)),
            Span::raw("]"),
        ])
    }
}

/// 开关行：[x]/[ ] + 说明文字。
fn toggle_row(app: &App, field: Field, on: bool, note: &'static str) -> Line<'static> {
    let mark = if on { "[x]" } else { "[ ]" };
    let label = format!("{:<width$}", field.label(), width = LBL_W);
    if app.form.focus == field {
        let sel = palette::selected();
        Line::from(vec![
            Span::styled(label, sel),
            Span::styled(format!("{mark} {note}"), sel),
        ])
    } else {
        Line::from(vec![
            Span::styled(label, palette::muted()),
            Span::raw(format!("{mark} ")),
            Span::styled(note, palette::muted()),
        ])
    }
}

/// 按钮行：每个按钮独立可聚焦，聚焦者整按钮反色。
fn buttons_row(app: &App, buttons: &[(Field, &str)]) -> Line<'static> {
    let mut spans = Vec::new();
    for (i, (field, label)) in buttons.iter().enumerate() {
        if i > 0 {
            spans.push(Span::raw("  "));
        }
        let text = format!("[ {label} ]");
        if app.form.focus == *field {
            spans.push(Span::styled(text, palette::selected()));
        } else {
            spans.push(Span::styled(text, palette::muted()));
        }
    }
    Line::from(spans)
}

/// 定宽裁剪/补齐：值超宽截断（Base URL 常见），不足补空格（对齐）。
fn fit(s: &str, width: usize) -> String {
    let mut out: String = s.chars().take(width).collect();
    while out.chars().count() < width {
        out.push(' ');
    }
    out
}
