use ratatui::{
    prelude::*,
    widgets::{Block, BorderType, Clear, List, ListItem, Paragraph, Tabs},
};

use crate::orchestration::command::COMMANDS;
use crate::tui::model::types::{App, InputMode, Page, Phase, PhaseStatus, Severity};
use super::palette;

/// 顶栏：品牌 + 任务核心事实 + LLM 状态位。参数取 &App 是为了"读"，全程无 mut（可验证）。
/// 渗透工具顶栏铁律：状态、目标、计时三件事必须同屏——作用域敬畏。
/// LLM 状态位是新增的第四件事：配置错误要第一时间可见，不用翻设置页确认。
pub fn header<'a>(app: &'a App) -> Paragraph<'a> {
    let dot = match app.phase_status(app.run.phase) {
        PhaseStatus::Running { .. } => "●",
        PhaseStatus::Done => "✓",
        PhaseStatus::Pending => "·",
    };
    let line = Line::from(vec![
        Span::styled(" StarEye ", palette::accent()),
        Span::raw("  "),
        Span::styled(format!("{dot} {} · ", app.run.phase.label()), palette::chrome()),
        // 目标用粗体：出事故时，回看截图最先要被读到的就是这个字段
        Span::styled(format!("目标 {}", app.run.target), Style::new().bold()),
        Span::raw("  "),
        Span::styled(app.elapsed_str(), palette::muted()),
        Span::raw("  "),
        Span::styled(format!("LLM·{}", app.config.model), palette::muted()),
    ]);
    Paragraph::new(line)
}

/// 作用域横幅：独立于页头的第二道安全线。永远显示 scope_note，
/// 任何覆盖层（将来）也不许遮盖它——把这条约定写成注释钉在代码里。
/// scope_note 由 RunState 的字段拼装：设置页改字段，横幅自动跟随。
pub fn scope_banner<'a>(app: &'a App) -> Paragraph<'a> {
    Paragraph::new(Span::styled(
        format!(" 作用域: {} ", app.run.scope_note()),
        Style::new().fg(Color::Yellow).bg(Color::Rgb(60, 50, 20)),
    ))
}

/// 页签条。页签清单来自 Model 的 Page::ALL 唯一来源，零件不知道
/// 具体有哪些页签——它只负责"把给它的标签摆成一条"。
/// 加第 6 个页签时这里零改动（收敛重复：不再本地硬编码）。
pub fn tabs_bar(app: &App) -> Tabs<'static> {
    let titles: Vec<&str> = Page::ALL.iter().map(|p| p.label()).collect();
    let current = Page::ALL.iter().position(|p| *p == app.page).unwrap_or(0);
    Tabs::new(titles)
        .select(current)
        .highlight_style(palette::selected())
        .divider(" · ")
}

/// 攻击链横向进度条：把 [PhaseStatus;4] 渲染成 "侦察 ✓ › 扫描 ●62 › 验证 · › 报告 ·"。
/// 纯函数：输入 Model 字段，输出 widget，无副作用。
pub fn phase_chain(app: &App) -> Paragraph<'_> {
    let mut spans = Vec::new();
    for (i, p) in Phase::CHAIN.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled(" › ", palette::muted()));
        }
        let st = app.phase_status(*p);
        let text = match st {
            PhaseStatus::Running { progress } => format!("{} {}{}%", p.label(), st.icon(), progress),
            _ => format!("{} {}", p.label(), st.icon()),
        };
        spans.push(Span::styled(text, palette::phase_status(st)));
    }
    Paragraph::new(Line::from(spans))
}

/// 底部快捷键提示栏。提示文案随输入态变化——
/// 用户永远能看到"现在按什么键有用"，这是命令行交互的路标。
pub fn footer(app: &App) -> Paragraph<'_> {
    let hints = match app.mode {
        InputMode::Normal => "Tab/←→ 翻页 · : 命令 · ? 帮助 · q 退出",
        InputMode::Command => "命令行 · Tab 补全 · ↑↓ 选择 · Enter 执行 · Esc 退出",
        InputMode::Form => "Tab/↑↓ 焦点 · ←→ 选项 · Enter 确认 · Esc 返回",
    };
    let text = if app.phase_status(app.run.phase).is_active() {
        format!(" {hints}   ● 进行中")
    } else {
        format!(" {hints}")
    };
    Paragraph::new(Span::styled(text, palette::chrome_dim()))
}

/// 底部命令行（常驻输入通道）。三段式：❯ 提示符 + 已输入 + 幽灵补全 + 块光标。
/// 幽灵文本（灰）展示选中补全项的剩余部分——zsh/fish 的自动建议，
/// 让"不用记全命令"成为默认体验而非进阶技巧。
pub fn cmdbar(app: &App) -> Paragraph<'_> {
    let mut spans: Vec<Span> = vec![Span::styled("❯ ", palette::accent())];
    if app.cmd.buffer.is_empty() {
        spans.push(Span::styled(
            "按 : 输入命令 · Tab 补全 · ? 查看全部命令",
            palette::muted(),
        ));
    } else {
        spans.push(Span::raw(app.cmd.buffer.as_str()));
        // 比对时剥掉 buffer 的 : 或 / 前缀——否则带前缀的 buffer 永远
        // strip 不掉命令名，幽灵文本就成了死代码。
        let typed = app.cmd.buffer.trim_start_matches([':', '/']);
        if app.cmd.open
            && let Some(&mi) = app.cmd.matches.get(app.cmd.selected)
            && let Some(rest) = COMMANDS[mi].name.strip_prefix(typed)
        {
            spans.push(Span::styled(rest, palette::muted()));
        }
    }
    spans.push(Span::styled("█", palette::accent()));
    Paragraph::new(Line::from(spans))
}

/// 补全浮层：锚定命令行向上展开的面板。
/// anchor 是命令行所在 Rect；高度 = 候选数 + 标题/边框，且绝不向上越过页签
/// （anchor.y 即上方可用空间）。铁律的物理保障：横幅、页签永不被遮。
pub fn render_completion(frame: &mut Frame, app: &App, anchor: Rect) {
    if !app.cmd.open || app.cmd.matches.is_empty() {
        return;
    }
    let want = app.cmd.matches.len() as u16 + 2; // 候选行 + 标题行 + 边框
    let h = want.min(anchor.y); // anchor.y = 浮层上方可用行数
    if h < 3 {
        return; // 终端太矮，宁可不弹也不画出残缺浮层
    }
    let area = Rect {
        x: anchor.x,
        y: anchor.y - h,
        width: anchor.width,
        height: h,
    };
    // 先清屏再画：ratatui 的 widget 只写自己的 cell，不清背景——
    // 不清的话，被浮层盖住的旧内容会从文字旁边的空 cell 里透出来。
    frame.render_widget(Clear, area);
    let items: Vec<ListItem> = app
        .cmd
        .matches
        .iter()
        .enumerate()
        .map(|(i, &mi)| {
            let spec = &COMMANDS[mi];
            let sel = i == app.cmd.selected;
            // 选中行整行反色（黑字黄底，复用 selected()）；未选中=青色命令名 + 灰描述
            let name_style = if sel {
                palette::selected().add_modifier(Modifier::BOLD)
            } else {
                palette::accent()
            };
            let rest_style = if sel {
                palette::selected()
            } else {
                palette::muted()
            };
            let mut spans = vec![
                Span::styled(format!("{:<12}", spec.name), name_style),
                Span::styled(spec.desc, rest_style),
            ];
            if let Some(u) = spec.usage {
                spans.push(Span::styled(format!("  eg: {u}"), rest_style));
            }
            ListItem::new(Line::from(spans))
        })
        .collect();
    let list = List::new(items).block(panel("命令补全"));
    frame.render_widget(list, area);
}

/// 帮助浮层：只覆盖主体区的模态面板。
/// 注意它刻意不覆盖页签/横幅——安全信息特权高于一切浮层。
pub fn render_help(frame: &mut Frame, body: Rect) {
    let area = Rect {
        x: body.x + 1,
        y: body.y + 1,
        width: body.width.saturating_sub(2),
        height: body.height.saturating_sub(2),
    };
    if area.width < 20 || area.height < 8 {
        return; // 终端过小时帮助浮层让位，不画残缺内容
    }
    // 先清屏：帮助是模态的，底下的页面内容不该从面板空隙里透出来
    frame.render_widget(Clear, area);
    let head = palette::accent();
    let lines = vec![
        Line::from(Span::styled("命令（按 : 呼出命令行，输入即补全）", head)),
        Line::from("  help            查看全部命令与快捷键"),
        Line::from("  config [llm]    打开设置页；跟 llm 直接定位 LLM 配置"),
        Line::from("  scan <cidr|host>    启动扫描，如 scan 10.0.0.5"),
        Line::from("  llm model <name>    切换模型，如 llm model gpt-4o-mini"),
        Line::from("  llm key <sk-...>    设置 API Key"),
        Line::from("  llm test        测试 LLM 连通性"),
        Line::from("  findings [--sev x]  查看 / 过滤发现"),
        Line::from("  scope           查看当前授权作用域"),
        Line::from("  export <file>   导出报告，如 export report.md"),
        Line::from("  quit            退出"),
        Line::from(""),
        Line::from(Span::styled("快捷键", head)),
        Line::from("  : 或 /    呼出命令行          Tab/←→   翻页（浮层中为补全）"),
        Line::from("  ↑↓        选择补全项          Enter     执行 / 确认"),
        Line::from("  Esc       关闭浮层 / 后退     ?         帮助"),
        Line::from(""),
        Line::from(Span::styled(
            "规则：命令行激活时字符键一律进输入框（q 不会误退出）；Esc 后退。",
            palette::muted(),
        )),
    ];
    let p = Paragraph::new(lines).block(panel("帮助 · ? / Esc 关闭"));
    frame.render_widget(p, area);
}

/// 统一边框容器。所有面板经它产出，保证圆角/标题风格全局一致（设计系统入口）。
pub fn panel<'a>(title: &'a str) -> Block<'a> {
    Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(palette::panel_border())
        .title(Line::from(format!(" {title} ")).style(palette::muted()))
}

/// 统计零件：按严重度计数，供总览页右侧"战果"区使用。
pub fn severity_stats(app: &App) -> List<'_> {
    let mut items = Vec::new();
    for s in [Severity::Critical, Severity::High, Severity::Medium, Severity::Low, Severity::Info] {
        let count = app.run.findings.iter().filter(|f| f.severity == s).count();
        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{:>4} ", count), palette::severity(s)),
            Span::styled(s.label(), palette::severity(s)),
        ])));
    }
    List::new(items).block(panel("战果"))
}
