/// 页签 = 视图路由的封闭集合。新增页面只需在这里加一个变体，
/// 编译器会逼你把注册表和测试补齐——这是"易扩写"的机制保障。
/// 「设置」是本轮新增：LLM 配置等需要表单的场景，复用页面机制而不是
/// 另造覆盖层状态机（与既有 Page 注册表同构，成本最低）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Cockpit,    // 作战室
    Activity,   // 活动流
    Findings,   // 发现
    Console,    // 控制台
    Settings,   // 设置（LLM / 作用域）
}

impl Page {
    /// 全部页签的唯一清单。tabs_bar、测试、补全浮层都从这里取。
    pub const ALL: [Page; 5] = [
        Page::Cockpit,
        Page::Activity,
        Page::Findings,
        Page::Console,
        Page::Settings,
    ];

    pub fn label(self) -> &'static str {
        match self {
            Page::Cockpit => "总览",
            Page::Activity => "活动流",
            Page::Findings => "发现",
            Page::Console => "控制台",
            Page::Settings => "设置",
        }
    }

    /// 左右键循环切换用的"下一个"，写在这里而不是 View 里：
    /// 页签顺序是数据事实，不是呈现细节。
    pub fn next(self) -> Page {
        match self {
            Page::Cockpit => Page::Activity,
            Page::Activity => Page::Findings,
            Page::Findings => Page::Console,
            Page::Console => Page::Settings,
            Page::Settings => Page::Cockpit,
        }
    }

    pub fn prev(self) -> Page {
        match self {
            Page::Cockpit => Page::Settings,
            Page::Settings => Page::Console,
            Page::Console => Page::Findings,
            Page::Findings => Page::Activity,
            Page::Activity => Page::Cockpit,
        }
    }
}
