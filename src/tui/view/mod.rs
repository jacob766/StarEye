mod palette;
mod chrome;
mod layout;
mod pages;

use ratatui::prelude::*;
use crate::tui::model::types::App;

// 注意：上面 `mod palette; mod chrome; ...` 已经把这些模块引入本作用域，
// 不能再写 use {chrome, ...}，否则同一命名空间出现两个 chrome（E0255）。
// 下面的路径（chrome::xxx / layout::xxx / pages::renderer）直接可用。

/// 唯一的视图入口。签名即契约：只读 App，画满 frame。
/// 整个函数的行数应当接近"清单"——它只描述装配顺序，不含任何业务。
pub fn draw(app: &App, frame: &mut Frame) {
    let a = layout::root(frame.area());

    frame.render_widget(chrome::header(app), a.header);
    frame.render_widget(chrome::scope_banner(app), a.scope);
    frame.render_widget(chrome::tabs_bar(app), a.tabs);
    frame.render_widget(chrome::footer(app), a.footer);
    frame.render_widget(chrome::cmdbar(app), a.cmdbar);

    // 主体：查注册表拿渲染器（策略），执行
    let render = pages::renderer(app.page);
    render(app, frame, a.body);

    // 浮层后画（ratatui 里后画的 widget 覆盖先画的）：
    // 补全浮层锚定命令行向上展开，帮助浮层只覆盖主体区——
    // 两者在几何上都不可越过页签，作用域横幅永不被遮（铁律的物理保障）。
    if app.cmd.open {
        chrome::render_completion(frame, app, a.cmdbar);
    }
    if app.help_open {
        chrome::render_help(frame, a.body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::model::types::{App, InputMode, Page};
    use ratatui::{Terminal, backend::TestBackend};

    /// 把整个界面渲染进 TestBackend，返回拼接好的文本。
    /// 目的不是断言像素，而是保证"每个页面、每种态都能画完不 panic"——
    /// 布局算术（浮层高度、裁切、焦点行）的边界都由它兜底。
    fn render(app: &App) -> String {
        let mut terminal = Terminal::new(TestBackend::new(100, 30)).unwrap();
        terminal.draw(|frame| draw(app, frame)).unwrap();
        let buffer = terminal.backend().buffer().clone();
        let mut out = String::new();
        for y in 0..buffer.area.height {
            for x in 0..buffer.area.width {
                out.push_str(buffer[(x, y)].symbol());
            }
            out.push('\n');
        }
        out
    }

    /// 宽松包含：缓冲区里每个 CJK 宽字符后面跟一个占位空格 cell
    /// （ratatui 对宽字形的存储方式），直接 contains 中文会假阴性。
    /// 比对双方都去掉空格后再匹配。
    fn has(text: &str, needle: &str) -> bool {
        text.replace(' ', "").contains(&needle.replace(' ', ""))
    }

    #[test]
    fn every_page_renders() {
        for page in Page::ALL {
            let mut app = App::new();
            app.page = page;
            if page == Page::Settings {
                app.mode = InputMode::Form;
            }
            let text = render(&app);
            assert!(has(&text, "StarEye"), "{page:?} 顶栏缺失");
            assert!(has(&text, "作用域"), "{page:?} 作用域横幅缺失");
            assert!(text.contains('❯'), "{page:?} 命令行缺失");
        }
    }

    #[test]
    fn completion_popup_renders_above_cmdbar() {
        let mut app = App::new();
        app.mode = InputMode::Command;
        app.cmd.buffer = ":llm".into();
        // 正常路径里 matches 由 Controller 维护；测试直接借词表 API 构造同态
        app.cmd.matches = crate::orchestration::command::complete_indices(":llm");
        app.cmd.open = true;
        let text = render(&app);
        assert!(has(&text, "命令补全"));
        assert!(has(&text, "llm model"));
    }

    #[test]
    fn help_overlay_renders_in_body() {
        let mut app = App::new();
        app.help_open = true;
        let text = render(&app);
        assert!(has(&text, "帮助"));
        assert!(has(&text, "llm test"));
        // 铁律：浮层不遮作用域横幅
        assert!(has(&text, "作用域"));
    }

    #[test]
    fn console_page_shows_history() {
        let mut app = App::new();
        app.page = Page::Console;
        let text = render(&app);
        assert!(has(&text, "StarEye v0.1"));
        assert!(has(&text, "扫描进行中"));
    }

    #[test]
    fn settings_page_shows_form() {
        let mut app = App::new();
        app.page = Page::Settings;
        app.mode = InputMode::Form;
        let text = render(&app);
        assert!(has(&text, "LLM 配置"));
        assert!(has(&text, "Base URL"));
        assert!(has(&text, "gpt-4o"));
        // 未聚焦的 Key 必须打码，明文不出现在界面上
        assert!(!text.contains("sk-demo1234567890abcd"));
    }
}
