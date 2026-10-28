use crate::tui::model::types::{App, Field, LineKind, LlmConfig, PROVIDERS};

/// 表单焦点移动。光标归位到新字段末尾——
/// 否则从长字段跳到短字段，光标会指向不存在的位置。
pub(super) fn set_focus(app: &mut App, f: Field) {
    app.form.focus = f;
    let len = app.form.focused_text().map_or(0, str::chars_count);
    app.form.cursor = len;
}

/// ↑↓/Tab：在字段间移动焦点。Field::ALL 的顺序即 Tab 顺序，取余循环。
pub(super) fn move_focus(app: &mut App, d: i32) {
    let focus = if d < 0 {
        app.form.focus.prev()
    } else {
        app.form.focus.next()
    };
    set_focus(app, focus);
}

/// ←→ 的语义取决于聚焦字段：服务商行循环选项，文本行移动字符光标。
/// 这一个函数就是"同一按键、不同字段、不同行为"的收容所。
pub(super) fn cursor_left(app: &mut App) {
    if app.form.focus == Field::Provider {
        cycle_provider(app, -1);
    } else {
        app.form.cursor = app.form.cursor.saturating_sub(1);
    }
}

pub(super) fn cursor_right(app: &mut App) {
    if app.form.focus == Field::Provider {
        cycle_provider(app, 1);
    } else {
        let len = app.form.focused_text().map_or(0, str::chars_count);
        if app.form.cursor < len {
            app.form.cursor += 1;
        }
    }
}

fn cycle_provider(app: &mut App, d: isize) {
    let n = PROVIDERS.len() as isize;
    app.form.provider = (app.form.provider as isize + d).rem_euclid(n) as usize;
}

/// 表单态字符插入。光标按 UTF-8 字符计，按字节定位——
/// 中文服务商名、中文路径都不会把字符串切坏。
/// 先取光标快照再借字段：focused_text_mut 借走的是整个 form，
/// 边借边读 app.form.cursor 会被借用检查器拦下（E0503）。
pub(super) fn form_insert(app: &mut App, c: char) {
    if app.form.focus == Field::ReadOnly {
        return; // 开关不吃字符
    }
    let cursor = app.form.cursor;
    let inserted = if let Some(buf) = app.form.focused_text_mut() {
        let byte: usize = buf.chars().take(cursor).map(|ch| ch.len_utf8()).sum();
        buf.insert(byte, c);
        true
    } else {
        false
    };
    if inserted {
        app.form.cursor = cursor + 1;
    }
}

pub(super) fn form_backspace(app: &mut App) {
    let cursor = app.form.cursor;
    if cursor == 0 {
        return;
    }
    let deleted = if let Some(buf) = app.form.focused_text_mut() {
        let byte: usize = buf
            .chars()
            .take(cursor - 1)
            .map(|ch| ch.len_utf8())
            .sum();
        let ch_len = buf[byte..]
            .chars()
            .next()
            .map_or(1, |ch| ch.len_utf8());
        buf.replace_range(byte..byte + ch_len, "");
        true
    } else {
        false
    };
    if deleted {
        app.form.cursor = cursor - 1;
    }
}

/// Enter 激活聚焦元素。按钮三分：测试（演示环境无真实网络，直接给结论）、
/// 保存（草稿 -> config）、恢复默认（config -> 草稿）。方向互逆，可对测。
pub(super) fn activate_field(app: &mut App) {
    match app.form.focus {
        Field::BtnTest => {
            let testing = format!("↻ 正在测试 {} ...", app.form.base_url);
            app.push_console(LineKind::Sys, testing.clone());
            app.form.status = testing;
            app.form.status_ok = true;
            let done = format!("✓ 连接成功 · 模型 {} 可用 · 延迟 312ms", app.form.model);
            app.push_console(LineKind::Ok, done.clone());
            app.form.status = done;
        }
        Field::BtnSave => {
            app.config = LlmConfig {
                provider: PROVIDERS[app.form.provider].to_string(),
                model: app.form.model.clone(),
                api_key: app.form.api_key.clone(),
                base_url: app.form.base_url.clone(),
                temperature: app.form.temperature.clone(),
            };
            let msg = format!(
                "✓ LLM 配置已保存: {} / {}（演示：仅内存）",
                PROVIDERS[app.form.provider], app.config.model
            );
            app.push_console(LineKind::Ok, msg);
            app.form.status = "✓ LLM 配置已保存".into();
            app.form.status_ok = true;
        }
        Field::BtnReset => {
            let cfg = &app.config;
            app.form.provider = PROVIDERS
                .iter()
                .position(|p| *p == cfg.provider.as_str())
                .unwrap_or(0);
            app.form.model = cfg.model.clone();
            app.form.base_url = cfg.base_url.clone();
            app.form.api_key = cfg.api_key.clone();
            app.form.temperature = cfg.temperature.clone();
            set_focus(app, Field::BtnReset);
            app.form.status = "已恢复上次保存的配置".into();
            app.form.status_ok = true;
        }
        Field::BtnScopeSave => {
            app.run.scope_to = app.form.scope_to.clone();
            app.run.scope_ex = app.form.scope_ex.clone();
            let msg = format!("✓ 作用域已更新: {}", app.run.scope_note());
            app.push_console(LineKind::Ok, msg);
            app.form.scope_status = "✓ 作用域已保存（顶部横幅已更新）".into();
            app.form.scope_status_ok = true;
        }
        Field::ReadOnly => app.form.read_only = !app.form.read_only,
        _ => {} // 文本字段 / 下拉：Enter 无操作（编辑用字符键，选择用 ←→）
    }
}

/// str::chars().count() 的语法糖——测试里断言光标位置时更易读。
trait CharsCount {
    fn chars_count(&self) -> usize;
}
impl CharsCount for str {
    fn chars_count(&self) -> usize {
        self.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tui::controller::{apply, bind};
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
    fn form_focus_cycles_all_fields() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('5')); // 设置页 -> 表单态
        assert_eq!(app.form.focus, Field::Provider);
        for expect in Field::ALL.iter().skip(1) {
            press(&mut app, KeyCode::Tab);
            assert_eq!(app.form.focus, *expect);
        }
        press(&mut app, KeyCode::Tab); // 绕回第一个
        assert_eq!(app.form.focus, Field::Provider);
    }

    #[test]
    fn form_edits_and_saves() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char(':'));
        type_str(&mut app, "config llm"); // 焦点直接落在模型字段
        press(&mut app, KeyCode::Enter);
        for _ in 0.."gpt-4o".chars().count() {
            press(&mut app, KeyCode::Backspace);
        }
        assert_eq!(app.form.model, "");
        type_str(&mut app, "gpt-4o-mini");
        assert_eq!(app.form.model, "gpt-4o-mini");
        // Model -> BtnSave 共 5 次 Tab
        for _ in 0..5 {
            press(&mut app, KeyCode::Tab);
        }
        assert_eq!(app.form.focus, Field::BtnSave);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.config.model, "gpt-4o-mini");
        assert!(app.form.status_ok);
        assert!(app.form.status.contains("已保存"));
    }

    #[test]
    fn form_q_types_not_quits() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('5')); // 表单态
        press(&mut app, KeyCode::Tab); // 焦点 -> Model
        type_str(&mut app, "q");
        assert!(!app.quit);
        assert_eq!(app.form.model, "gpt-4oq");
    }

    #[test]
    fn form_cursor_moves_and_inserts_midway() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('5'));
        press(&mut app, KeyCode::Tab); // -> Model "gpt-4o"，光标初始在末尾(6)
        for _ in 0..3 {
            press(&mut app, KeyCode::Left);
        }
        // 光标 6 -> 3，落在 '-' 之前：g p t | - 4 o
        type_str(&mut app, "X");
        assert_eq!(app.form.model, "gptX-4o");
        press(&mut app, KeyCode::Backspace);
        assert_eq!(app.form.model, "gpt-4o");
    }

    #[test]
    fn form_provider_cycles_with_arrows() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('5')); // 焦点在 Provider
        press(&mut app, KeyCode::Right);
        assert_eq!(PROVIDERS[app.form.provider], "Anthropic");
        press(&mut app, KeyCode::Left);
        assert_eq!(PROVIDERS[app.form.provider], "OpenAI");
        press(&mut app, KeyCode::Left);
        assert_eq!(PROVIDERS[app.form.provider], "自定义");
    }

    #[test]
    fn scope_save_updates_banner_source() {
        let mut app = App::new();
        press(&mut app, KeyCode::Char('5'));
        // Provider(0) 之后 8 次 Tab -> ScopeTo(8)
        for _ in 0..8 {
            press(&mut app, KeyCode::Tab);
        }
        assert_eq!(app.form.focus, Field::ScopeTo);
        for _ in 0.."2026-11-01".chars().count() {
            press(&mut app, KeyCode::Backspace);
        }
        assert_eq!(app.form.scope_to, "");
        type_str(&mut app, "2026-12-31");
        for _ in 0..3 {
            press(&mut app, KeyCode::Tab);
        }
        assert_eq!(app.form.focus, Field::BtnScopeSave);
        press(&mut app, KeyCode::Enter);
        assert_eq!(app.run.scope_to, "2026-12-31");
        assert!(app.run.scope_note().contains("2026-12-31"));
    }
}
