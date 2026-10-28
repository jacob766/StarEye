use crate::tui::model::types::{App, InputMode, Page};

use super::form::set_focus;

/// 页面切换的唯一入口。副作用只有两条，都是不变量维护：
/// 1. 进入设置页时记下来源页（Esc 返回用）；
/// 2. 设置页永远是表单态，其余页永远是 Normal 态——态与页绑定，
///    不存在"在设置页打字却敲成了翻页"的中间地带。
pub(super) fn goto(app: &mut App, page: Page) {
    if app.page != Page::Settings && page == Page::Settings {
        app.last_page = app.page;
    }
    app.page = page;
    app.mode = match page {
        Page::Settings => InputMode::Form,
        _ => InputMode::Normal,
    };
    if page == Page::Settings {
        set_focus(app, app.form.focus); // 仅归位光标，不改变焦点字段
    }
}
