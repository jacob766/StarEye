/// 输入态。界面同时存在三种"键盘归属"：
/// Normal（翻页/快捷键）、Command（底部命令行）、Form（设置页表单）。
/// 用枚举而不是若干 bool：Command + Form 同时激活这类非法组合无法表示。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode {
    Normal,
    Command,
    Form,
}

/// 底部命令行的全部状态。matches 存的是命令词表的下标而不是引用——
/// Model 因此不需要认识 orchestration 的类型，"指向词表的索引"即解耦。
/// 补全结果由 Controller 在每次 buffer 变化时刷新，View 只消费。
#[derive(Default)]
pub struct CommandLine {
    pub buffer: String,
    pub open: bool,       // 补全浮层是否展开
    pub selected: usize,  // 浮层中高亮第几项（循环移动）
    pub matches: Vec<usize>,
}

/// 一行控制台输出。kind 决定着色，着色策略收敛在 palette::line_kind——
/// 和严重度一样，"行类型 -> 颜色"只允许有一个定义处。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    Cmd,    // 用户输入的命令（❯ 前缀）
    Out,    // 普通输出
    Ok,     // 成功
    Err,    // 失败
    Sys,    // 系统提示
}

#[derive(Debug, Clone)]
pub struct ConsoleLine {
    pub kind: LineKind,
    pub text: String,
}

/// 控制台历史容量。有界缓冲：命令行是长跑交互，绝不能无限增长。
/// pub(crate)：只有 App::push_console 消费，不对外暴露。
pub(crate) const CONSOLE_CAP: usize = 200;
