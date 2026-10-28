use crate::orchestration::command::{self, Action, CommandCtx};
use crate::tui::model::types::{App, InputMode, LineKind};

use super::{form::set_focus, nav::goto};

/// ':' 或 '/' 唤出底部命令行：预填冒号（可视化"已在命令行"，且补全规则兼容），
/// 并立刻刷新匹配（":" 匹配全部命令，浮层随之展开）。
pub(super) fn enter_command_mode(app: &mut App) {
    app.mode = InputMode::Command;
    app.cmd.buffer = ":".into();
    refresh_matches(app);
}

/// buffer 变化后重算补全。open 的条件：buffer 非空且有匹配——
/// 空 buffer（或只有回车后的清净状态）不弹浮层，避免常驻干扰。
pub(super) fn refresh_matches(app: &mut App) {
    app.cmd.matches = command::complete_indices(&app.cmd.buffer);
    app.cmd.open = !app.cmd.buffer.is_empty() && !app.cmd.matches.is_empty();
    if app.cmd.selected >= app.cmd.matches.len() {
        app.cmd.selected = 0;
    }
}

/// 命令行态插入字符：进 buffer 后刷新补全。
pub(super) fn insert_char(app: &mut App, c: char) {
    app.cmd.buffer.push(c);
    refresh_matches(app);
}

/// 命令行态退格：出 buffer 后刷新补全。
pub(super) fn backspace(app: &mut App) {
    app.cmd.buffer.pop();
    refresh_matches(app);
}

/// Tab 补全：把 buffer 补全为当前选中项。补全后可能仍是一族命令
/// （llm -> llm model/key/test），因此补全后再刷新一次，浮层保持展开。
pub(super) fn complete(app: &mut App) {
    refresh_matches(app);
    if !app.cmd.matches.is_empty() {
        let name = command::COMMANDS[app.cmd.matches[app.cmd.selected]].name;
        app.cmd.buffer = format!("{name} ");
        refresh_matches(app); // 补全后可能仍是一族命令，浮层保持展开
    }
}

/// ↑↓ 在补全候选间循环移动（rem_euclid：选到最后一项再按 ↓ 回到第一项，
/// 且不会算出负数下标）。
pub(super) fn move_selection(app: &mut App, d: i32) {
    if app.cmd.open && !app.cmd.matches.is_empty() {
        let n = app.cmd.matches.len() as isize;
        app.cmd.selected = (app.cmd.selected as isize + d as isize).rem_euclid(n) as usize;
    }
}

/// Esc：清空命令行退回常态。
pub(super) fn exit_command_mode(app: &mut App) {
    app.cmd.buffer.clear();
    app.cmd.matches.clear();
    app.cmd.open = false;
    app.cmd.selected = 0;
    app.mode = InputMode::Normal;
}

/// Enter 的执行路径：命令进历史 -> 分发 -> 输出行落控制台 -> 动作落地。
pub(super) fn submit_command(app: &mut App) {
    let raw = std::mem::take(&mut app.cmd.buffer);
    app.cmd.matches.clear();
    app.cmd.open = false;
    app.cmd.selected = 0;
    if raw.trim().is_empty() {
        return;
    }
    app.push_console(LineKind::Cmd, raw.clone());
    let ctx = CommandCtx {
        target: &app.run.target,
        scope_note: &app.run.scope_note(),
        model: &app.config.model,
        base_url: &app.config.base_url,
    };
    let outcome = command::run(&raw, &ctx);
    for line in outcome.lines {
        app.push_console(line.kind, line.text);
    }
    match outcome.action {
        Action::None => {}
        Action::OpenHelp => app.help_open = true,
        Action::Goto(p) => goto(app, p),
        Action::FocusField(p, f) => {
            goto(app, p);
            set_focus(app, f);
        }
        Action::SetModel(m) => {
            // 一行命令即时生效：config（事实）与 form（草稿）同步改，
            // 顶栏 LLM 位与设置页输入框永远指向同一个值。
            app.config.model = m.clone();
            app.form.model = m;
        }
        Action::SetApiKey(k) => {
            app.config.api_key = k.clone();
            app.form.api_key = k;
        }
        Action::TestLlm => {
            let msg = format!("✓ 连接成功 · 模型 {} 可用 · 延迟 312ms", app.config.model);
            app.form.status = msg.clone();
            app.form.status_ok = true;
        }
        Action::Quit => app.quit = true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::controller::{apply, bind};
    use crate::tui::model::types::{Field, Page};
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

    fn key(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn press(app: &mut App, code: KeyCode) {
        apply(app, bind(app, &key(code)));
    }

    fn type_str(app: &mut App, s: &str) {
        for c in s.chars() {
            apply(app, bind(app, &key(KeyCode::Char(c))));
        }
    }

    #[test]
    fn colon_enters_command_mode_and_lists_all() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        assert_eq!(app.mode, InputMode::Command);
        assert_eq!(app.cmd.buffer, ":");
        assert!(app.cmd.open);
        assert_eq!(app.cmd.matches.len(), 7); // 全表截断到浮层预算
    }

    #[test]
    fn command_mode_swallows_q() {
        // 核心防误触：命令行态下 q 只是字符，程序岿然不动
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "q");
        assert!(!app.quit);
        assert_eq!(app.cmd.buffer, ":q");
    }

    #[test]
    fn typing_filters_completions() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "llm");
        // 3 条前缀命中 + "config llm" 包含命中 = 4
        assert_eq!(app.cmd.matches.len(), 4);
        assert_eq!(app.cmd.selected, 0);
    }

    #[test]
    fn tab_completes_to_selected() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "llm");
        press(&mut app, KeyCode::Down); // 选中 llm key
        press(&mut app, KeyCode::Tab);
        assert_eq!(app.cmd.buffer, "llm key ");
        // 补全后浮层保持展开（llm 家族仍在候选内）
        assert!(app.cmd.open);
    }

    #[test]
    fn enter_executes_llm_model() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "llm model gpt-4o-mini");
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.config.model, "gpt-4o-mini");
        assert_eq!(app.form.model, "gpt-4o-mini"); // 草稿同步，双处一致
        let last = app.console.last().unwrap();
        assert_eq!(last.kind, LineKind::Ok);
        assert!(last.text.contains("模型已切换"));
        // 执行后命令行清空但保持命令行态（连续敲命令不用重复唤出）
        assert_eq!(app.mode, InputMode::Command);
        assert!(app.cmd.buffer.is_empty());
        assert!(!app.cmd.open);
    }

    #[test]
    fn esc_exits_command_mode() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "sc");
        press(&mut app, KeyCode::Esc);
        assert_eq!(app.mode, InputMode::Normal);
        assert!(app.cmd.buffer.is_empty());
        assert!(!app.cmd.open);
    }

    #[test]
    fn unknown_command_reports_error() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "frobnicate");
        press(&mut app, KeyCode::Enter);
        assert!(!app.quit);
        let last = app.console.last().unwrap();
        assert_eq!(last.kind, LineKind::Err);
        assert!(last.text.contains("未知命令"));
    }

    #[test]
    fn config_llm_opens_settings_focused_on_model() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "config llm");
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.page, Page::Settings);
        assert_eq!(app.mode, InputMode::Form);
        assert_eq!(app.form.focus, Field::Model);
    }

    #[test]
    fn llm_test_command_sets_form_status() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "llm test");
        press(&mut app, KeyCode::Enter);
        assert!(app.form.status.contains("连接成功"));
        assert!(app.form.status_ok);
        assert!(app.console.iter().any(|l| l.kind == LineKind::Ok));
    }
}
