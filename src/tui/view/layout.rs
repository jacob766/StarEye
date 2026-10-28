use ratatui::layout::{Constraint, Layout, Rect};

/// 本工具全部主界面的唯一纵向配方。数值即文档：
/// 页头1行 + 作用域横幅2行 + 页签1行 + 主体弹性 + 命令行1行 + 页脚1行。
/// 命令行是常驻输入通道（配置 LLM 等操作的入口），和页头一样"永在"。
/// 审批条未来上线时，在这里加一条 Length(n)（n 由 Model 计算），
/// 主体自动让位——改布局只碰这一个文件（易扩写的物理保障）。
pub fn root(area: Rect) -> RootAreas {
    let [header, scope, tabs, body, cmdbar, footer] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Length(1),
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    RootAreas { header, scope, tabs, body, cmdbar, footer }
}

/// 用结构体代替"裸数组返回值"：area[3] 读不出含义，body.footer 一眼懂。
/// 这就是布局层的"具名化"——可解释性的最小成本投入。
pub struct RootAreas {
    pub header: Rect,
    pub scope: Rect,
    pub tabs: Rect,
    pub body: Rect,
    pub cmdbar: Rect,
    pub footer: Rect,
}

/// 总览页内部配方：左侧攻击链(定宽34)，右侧战果统计，弹性填剩余。
/// 每页内部怎么再切，归该页自己的 render 调用这个函数——
/// 全局配方和页面配方分离，互不污染。
pub fn cockpit(area: Rect) -> [Rect; 2] {
    // 0.29+ 的 areas() 直接返回定长数组，编译期校验数量；
    // 旧写法 split().to_vec().try_into().unwrap() 运行期才会 panic。
    Layout::horizontal([
        Constraint::Length(34),
        Constraint::Min(0),
    ]).areas(area)
}
