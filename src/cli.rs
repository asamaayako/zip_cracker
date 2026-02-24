use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "zip_cracker")]
#[command(about = "ZIP 密码破解器 (多线程)", long_about = None)]
pub struct Args {
    /// ZIP 文件路径
    #[arg(default_value = "../default.zip")]
    pub zip_path: String,

    /// 攻击模式
    #[arg(short = 'M', long, value_enum, default_value = "bruteforce")]
    pub mode: AttackMode,

    /// 字典文件路径 (默认: ~/.zip_cracker/dictionary.txt)
    #[arg(short = 'D', long)]
    pub dictionary: Option<String>,

    /// 字符集选择 (可多选，用逗号分隔，如: lower,upper,digit)
    #[arg(short, long, value_delimiter = ',', default_value = "lower,upper,digit")]
    pub charset: Vec<Charset>,

    /// 密码长度 (固定长度模式)
    #[arg(short, long)]
    pub length: Option<usize>,

    /// 最大密码长度 (递增模式: 从1开始逐一尝试到此长度)
    #[arg(short, long)]
    pub max_length: Option<usize>,

    /// 最小密码长度 (递增模式下的起始长度, 默认为1)
    #[arg(long, default_value = "1")]
    pub min_length: usize,
}

#[derive(Clone, ValueEnum)]
pub enum AttackMode {
    /// 暴力破解
    Bruteforce,
    /// 字典攻击
    Dictionary,
}

#[derive(Clone, ValueEnum, PartialEq, Eq, Hash)]
pub enum Charset {
    /// 拼音声母 (20字符)
    Pinyin,
    /// 小写字母 (a-z, 26字符)
    Lower,
    /// 大写字母 (A-Z, 26字符)
    Upper,
    /// 数字 (0-9, 10字符)
    Digit,
    /// ASCII符号 (32字符)
    Symbol,
    /// 全部可打印 ASCII (95字符)
    Ascii,
    /// 全角符号 (常用全角标点)
    Fullwidth,
    /// 常用汉字 (3500字符 - GB2312一级汉字)
    Chinese,
}
