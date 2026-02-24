use clap::{Parser, ValueEnum};

#[derive(Parser)]
#[command(name = "archive_cracker")]
#[command(about = "压缩包密码破解器 - 支持 ZIP/7z/RAR (多线程)", long_about = None)]
pub struct Args {
    /// 压缩包文件路径 (支持 .zip, .7z, .rar)
    pub archive_path: String,

    /// 字典文件路径 (默认: ~/.zip_cracker/dictionary.txt)
    #[arg(short = 'D', long)]
    pub dictionary: Option<String>,

    /// 字符集选择 (可多选，用逗号分隔，如: lower,upper,digit)
    #[arg(short, long, value_delimiter = ',', default_value = "lower,upper,digit")]
    pub charset: Vec<Charset>,

    /// 密码长度 (固定长度模式，暴力破解时必需)
    #[arg(short, long)]
    pub length: Option<usize>,

    /// 最大密码长度 (递增模式: 从 min-length 开始逐一尝试到此长度)
    #[arg(short, long)]
    pub max_length: Option<usize>,

    /// 最小密码长度 (递增模式下的起始长度, 默认为1)
    #[arg(long, default_value = "1")]
    pub min_length: usize,

    /// 跳过字典攻击，直接暴力破解
    #[arg(long)]
    pub skip_dictionary: bool,
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
