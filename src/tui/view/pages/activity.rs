use ratatui::prelude::*;

use crate::tui::model::types::App;

use super::placeholder;

pub(super) fn render_activity(app: &App, frame: &mut Frame, area: Rect) {
    placeholder(frame, app, area, "活动流", "此处将渲染 Agent 事件流（有界环形缓冲）");
}
