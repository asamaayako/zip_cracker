use std::collections::HashSet;

use crate::cli::Charset;

/// 获取单个字符集
pub fn get_single_charset(charset: &Charset) -> (&'static str, Vec<char>) {
    match charset {
        Charset::Pinyin => (
            "拼音声母",
            vec![
                'b', 'p', 'm', 'f', 'd', 't', 'n', 'l', 'g', 'k', 'h', 'j', 'q', 'x', 'z', 'c',
                's', 'r', 'y', 'w',
            ],
        ),
        Charset::Lower => ("小写字母", ('a'..='z').collect()),
        Charset::Upper => ("大写字母", ('A'..='Z').collect()),
        Charset::Digit => ("数字", ('0'..='9').collect()),
        Charset::Symbol => ("ASCII符号", get_ascii_symbols()),
        Charset::Ascii => (
            "可打印ASCII",
            (' '..='~').collect(), // ASCII 32-126
        ),
        Charset::Fullwidth => ("全角符号", get_fullwidth_symbols()),
        Charset::Chinese => ("常用汉字 (GB2312一级)", get_chinese_charset()),
    }
}

/// 获取 ASCII 符号字符集（不包括字母和数字）
fn get_ascii_symbols() -> Vec<char> {
    let mut symbols = Vec::new();

    // 空格
    symbols.push(' ');

    // !"#$%&'()*+,-./ (ASCII 33-47)
    symbols.extend('!'..='/');
    symbols.push('/');

    // :;<=>?@ (ASCII 58-64)
    symbols.extend(':'..='@');

    // [\]^_` (ASCII 91-96)
    symbols.extend('['..='`');

    // {|}~ (ASCII 123-126)
    symbols.extend('{'..='~');

    symbols
}

/// 获取全角符号字符集
fn get_fullwidth_symbols() -> Vec<char> {
    vec![
        // 全角标点符号
        '，', '。', '、', '；', '：', '？', '！', '…', '—', '·', '"', '"', '\u{2018}', '\u{2019}',
        '（', '）', '【', '】', '《', '》', '『', '』', '「', '」', '〈', '〉', '￥', '※', '〃',
        '々', // 全角数学和其他符号
        '＋', '－', '×', '÷', '＝', '≠', '＜', '＞', '≤', '≥', '％', '‰', '°', '℃', '＄', '￡',
        '￠', '＠', '＃', '＆', '＊', '§', '〒', '〓', '□', '■', '△', '▲', '○', '●', '◎', '☆',
        '★', '◇', '◆', '〔', '〕', '〖', '〗',
    ]
}

/// 获取 GB2312 一级汉字字符集 (3500个常用汉字)
fn get_chinese_charset() -> Vec<char> {
    let mut chars = Vec::with_capacity(3500);

    // GB2312 一级汉字 Unicode 范围（常用汉字）
    for code in 0x4e00..=0x9fa5_u32 {
        if let Some(c) = char::from_u32(code) {
            if chars.len() < 3500 {
                chars.push(c);
            } else {
                break;
            }
        }
    }

    chars
}

/// 合并多个字符集（去重）
pub fn get_combined_charset(charsets: &[Charset]) -> (String, Vec<char>) {
    // 去重字符集选择
    let unique_charsets: Vec<_> = charsets.iter().collect::<HashSet<_>>().into_iter().collect();

    if unique_charsets.is_empty() {
        eprintln!("错误: 必须至少选择一个字符集");
        std::process::exit(1);
    }

    // 合并所有字符集
    let mut char_set = HashSet::new();
    let mut names = Vec::new();

    for charset in &unique_charsets {
        let (name, chars) = get_single_charset(charset);
        names.push(name);
        char_set.extend(chars);
    }

    let name = names.join(" + ");
    let mut chars: Vec<char> = char_set.into_iter().collect();
    chars.sort_unstable(); // 排序以保证顺序一致性

    (name, chars)
}

/// 将索引转换为密码字符串
pub fn index_to_password(mut index: u64, chars: &[char], length: usize) -> String {
    let base = chars.len() as u64;
    let mut result = String::with_capacity(length);

    for _ in 0..length {
        result.push(chars[(index % base) as usize]);
        index /= base;
    }
    result
}
