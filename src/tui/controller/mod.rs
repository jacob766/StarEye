use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

use crate::tui::model::types::{App, InputMode, Page};

use cmdline::{
    backspace, complete, enter_command_mode, exit_command_mode, insert_char, move_selection,
    submit_command,
};
use form::{activate_field, cursor_left, cursor_right, form_backspace, form_insert, move_focus};
use nav::goto;

mod cmdline;
mod form;
mod nav;

/// Controller 的唯一职责：把"物理输入"翻译成"意图"，再作用到 Model。
/// 三种输入态（Normal/Command/Form）共享同一份 Cmd 词汇表：
/// 按键 -> Cmd 的映射随态而变，Cmd -> Model 的作用同样随态而变，
/// 但词汇表封闭——新增交互 = 加变体，编译器逼你补全 bind 与 apply。
///
/// 本文件是调度核心：Cmd 词汇表 + bind（翻译）+ apply（分发）。
/// 具体动作按职责拆到三个子模块：
/// - nav.rs     页面导航与"态页绑定"不变量
/// - cmdline.rs 底部命令行的全部行为（服务控制台页）
/// - form.rs    设置页表单的全部交互
#[derive(Debug, PartialEq, Eq)]
pub enum Cmd {
    SwitchTo(Page),
    NextPage,
    PrevPage,
    Quit,
    Noop,              // 未绑定的按键必须显式吸收，防止"什么都不做"成为隐式行为
    ToggleHelp,
    EnterCommandMode,  // ':' 或 '/' 唤出底部命令行
    InsertChar(char),
    Backspace,
    CursorLeft,
    CursorRight,
    Complete,          // Tab：补全（仅命令行态）
    MoveSelection(i32), // ↑↓：命令行态移补全选项 / 表单态移焦点
    Submit,            // Enter：执行命令 / 激活表单元素
    ExitMode,          // Esc：命令行态清空退回 / 表单态返回来源页
}

/// 纯函数：输入输出可单测，无需终端（可验证性的关键设计）。
/// 注意签名带上 &App：键位映射依赖当前输入态，态是 Model 的事实。
pub fn bind(app: &App, key: &KeyEvent) -> Cmd {
    if key.kind != KeyEventKind::Press {
        return Cmd::Noop; // Windows 三连事件在此拦截
    }
    // Ctrl+C 是最后的安全绳：任何态下都必须能退出。
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        return match key.code {
            KeyCode::Char('c') => Cmd::Quit,
            _ => Cmd::Noop,
        };
    }
    // 帮助浮层是模态的：只认 Esc/?/Enter，其余全部吸收。
    if app.help_open {
        return match key.code {
            KeyCode::Esc | KeyCode::Char('?') | KeyCode::Enter => Cmd::ToggleHelp,
            _ => Cmd::Noop,
        };
    }
    match app.mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => Cmd::Quit,
            KeyCode::Tab | KeyCode::Right => Cmd::NextPage,
            KeyCode::BackTab | KeyCode::Left => Cmd::PrevPage,
            KeyCode::Char(':') | KeyCode::Char('/') => Cmd::EnterCommandMode,
            KeyCode::Char('?') => Cmd::ToggleHelp,
            KeyCode::Char('1') => Cmd::SwitchTo(Page::Cockpit),
            KeyCode::Char('2') => Cmd::SwitchTo(Page::Activity),
            KeyCode::Char('3') => Cmd::SwitchTo(Page::Findings),
            KeyCode::Char('4') => Cmd::SwitchTo(Page::Console),
            KeyCode::Char('5') => Cmd::SwitchTo(Page::Settings),
            _ => Cmd::Noop,
        },
        InputMode::Command => match key.code {
            // 命令行态下 q 是普通字符——"想输命令却把程序退了"从此绝迹。
            KeyCode::Esc => Cmd::ExitMode,
            KeyCode::Enter => Cmd::Submit,
            KeyCode::Backspace => Cmd::Backspace,
            KeyCode::Tab => Cmd::Complete,
            KeyCode::Up => Cmd::MoveSelection(-1),
            KeyCode::Down => Cmd::MoveSelection(1),
            KeyCode::Char(c) => Cmd::InsertChar(c),
            _ => Cmd::Noop,
        },
        InputMode::Form => match key.code {
            KeyCode::Esc => Cmd::ExitMode,
            KeyCode::Tab | KeyCode::Down => Cmd::MoveSelection(1),
            KeyCode::BackTab | KeyCode::Up => Cmd::MoveSelection(-1),
            KeyCode::Left => Cmd::CursorLeft,
            KeyCode::Right => Cmd::CursorRight,
            KeyCode::Enter => Cmd::Submit,
            KeyCode::Backspace => Cmd::Backspace,
            KeyCode::Char(c) => Cmd::InsertChar(c),
            _ => Cmd::Noop,
        },
    }
}

/// 指令执行器：唯一合法的 App 变更点（View 无权改，命令模块也不直接改）。
/// 本函数只做分发——每个 Cmd 的具体动作在对应子模块。
pub fn apply(app: &mut App, cmd: Cmd) {
    match cmd {
        Cmd::SwitchTo(p) => goto(app, p),
        Cmd::NextPage => {
            let next = app.page.next();
            goto(app, next);
        }
        Cmd::PrevPage => {
            let prev = app.page.prev();
            goto(app, prev);
        }
        Cmd::Quit => app.quit = true,
        Cmd::Noop => {}
        Cmd::ToggleHelp => app.help_open = !app.help_open,
        Cmd::EnterCommandMode => enter_command_mode(app),
        Cmd::InsertChar(c) => match app.mode {
            InputMode::Command => insert_char(app, c),
            InputMode::Form => form_insert(app, c),
            InputMode::Normal => {}
        },
        Cmd::Backspace => match app.mode {
            InputMode::Command => backspace(app),
            InputMode::Form => form_backspace(app),
            InputMode::Normal => {}
        },
        Cmd::CursorLeft => {
            if app.mode == InputMode::Form {
                cursor_left(app);
            }
        }
        Cmd::CursorRight => {
            if app.mode == InputMode::Form {
                cursor_right(app);
            }
        }
        Cmd::Complete => {
            if app.mode == InputMode::Command {
                complete(app);
            }
        }
        Cmd::MoveSelection(d) => match app.mode {
            InputMode::Command => move_selection(app, d),
            InputMode::Form => move_focus(app, d),
            InputMode::Normal => {}
        },
        Cmd::Submit => match app.mode {
            InputMode::Command => submit_command(app),
            InputMode::Form => activate_field(app),
            InputMode::Normal => {}
        },
        Cmd::ExitMode => match app.mode {
            InputMode::Command => exit_command_mode(app),
            InputMode::Form => {
                // 返回来源页（:config 从哪来回哪去），草稿保留
                let back = app.last_page;
                goto(app, back);
            }
            InputMode::Normal => {}
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::model::types::Phase;

    /// 测试辅助：构造一个"按下"状态的按键。
    /// KeyEvent::new 在 crossterm 0.29 默认 kind = Press（正是生产路径的事件）。
    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    /// 模拟连续输入：逐个字符走完整的 bind -> apply 流水线。
    fn type_str(app: &mut App, s: &str) {
        for c in s.chars() {
            apply(app, bind(app, &key(KeyCode::Char(c))));
        }
    }

    fn press(app: &mut App, code: KeyCode) {
        apply(app, bind(app, &key(code)));
    }

    #[test]
    fn tab_cycles_pages() {
        let mut app = App::new();
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.page, Page::Activity);
    }

    #[test]
    fn release_and_repeat_events_are_ignored() {
        // Windows 下每个按键会上报 Press/Repeat/Release 三次，
        // 只有 Press 能触发指令——否则按一下 q 会退出三次交互。
        let mut release = key(KeyCode::Char('q'));
        release.kind = KeyEventKind::Release;
        assert_eq!(bind(&App::new(), &release), Cmd::Noop);

        let mut repeat = key(KeyCode::Char('q'));
        repeat.kind = KeyEventKind::Repeat;
        assert_eq!(bind(&App::new(), &repeat), Cmd::Noop);
    }

    #[test]
    fn q_quits_in_normal_mode() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('q'));
        assert!(app.quit);
    }

    #[test]
    fn digit_keys_switch_pages() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('3'));
        assert_eq!(app.page, Page::Findings);
        press(&mut app, KeyCode::Char('1'));
        assert_eq!(app.page, Page::Cockpit);
        press(&mut app, KeyCode::Char('5'));
        assert_eq!(app.page, Page::Settings);
        // 设置页必须是表单态（态与页绑定的不变量）
        assert_eq!(app.mode, InputMode::Form);
    }

    #[test]
    fn next_prev_are_inverse() {
        // 翻页循环的性质测试：所有页签 next 再 prev 必须回到自身。
        // 这条性质若被破坏（比如加页签时漏改 next/prev），测试立刻红。
        for p in Page::ALL {
            assert_eq!(p.next().prev(), p);
        }
    }

    #[test]
    fn phase_chain_has_four_statuses() {
        let app = App::new();
        assert_eq!(app.run.phase_statuses.len(), Phase::CHAIN.len());
    }

    #[test]
    fn esc_in_form_returns_to_source_page() {
        let mut app = App::new();
        app.page = Page::Findings;
        press(&mut app, KeyCode::Char(':')); // 命令行态
        type_str(&mut app, "config");
        press(&mut app, KeyCode::Enter); // -> 设置页
        assert_eq!(app.page, Page::Settings);
        press(&mut app, KeyCode::Esc); // -> 返回来源页
        assert_eq!(app.page, Page::Findings);
        assert_eq!(app.mode, InputMode::Normal);
    }

    #[test]
    fn ctrl_c_quits_from_any_mode() {
        for mode_key in [KeyCode::Char(':'), KeyCode::Char('5')] {
            let mut app = App::new();
            press(&mut app, mode_key);
            let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
            let cmd = bind(&app, &ctrl_c);
            apply(&mut app, cmd);
            assert!(app.quit);
        }
    }

    #[test]
    fn help_toggles_and_is_modal() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('?'));
        assert!(app.help_open);
        // 模态期间 q 不退出、: 不进命令行
        press(&mut app, KeyCode::Char('q'));
        press(&mut app, KeyCode::Char(':'));
        assert!(!app.quit);
        assert_eq!(app.mode, InputMode::Normal);
        press(&mut app, KeyCode::Esc);
        assert!(!app.help_open);
    }
}
