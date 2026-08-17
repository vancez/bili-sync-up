use regex::Regex;

/// 番剧名称提取器，用于从完整的番剧标题中提取基础系列名称和季度信息
pub struct BangumiNameExtractor;

impl BangumiNameExtractor {
    /// 从番剧标题中提取基础系列名称和季度信息
    ///
    /// # 参数
    /// - `title`: 完整的番剧标题，例如 "灵笼 第二季"
    /// - `season_title`: 可选的季度标题，例如 "第二季"
    ///
    /// # 返回值
    /// 返回元组 (基础系列名称, 季度编号)
    /// 例如：("灵笼", 2)
    pub fn extract_series_name_and_season(title: &str, season_title: Option<&str>) -> (String, u32) {
        // 如果提供了 season_title，优先使用它来提取
        if let Some(season_part) = season_title {
            // 提取基础名称，并标准化空格（将多个连续空格合并为单个空格，去除括号前的空格）
            let base_name = title
                .replace(season_part, "")
                .split_whitespace()  // 分割字符串，自动去除首尾空格并处理连续空格
                .collect::<Vec<_>>()
                .join(" ")           // 用单个空格重新连接
                .replace(" （", "（") // 去除全角括号前的空格
                .replace(" (", "("); // 去除半角括号前的空格
            let season_number = Self::extract_season_number(season_part).unwrap_or(1);
            return (base_name, season_number);
        }

        // 如果没有 season_title，尝试从 title 中识别季度信息
        Self::extract_from_title(title)
    }

    /// 从完整标题中提取系列名称和季度信息
    fn extract_from_title(title: &str) -> (String, u32) {
        // 季度范围模式：B站合并季，如“海绵宝宝 第一季～第九季”、“海绵宝宝 第1-9季”
        // 取范围起点作为季号，范围本身不进系列名
        let range_patterns = [
            // 第X季～第Y季：第一季～第九季、第1季~第9季、第一季-第二季
            r"(.+?)\s*第([一二三四五六七八九十\d]+)季\s*[～~—–\-至到]\s*第([一二三四五六七八九十\d]+)季\s*(.*)$",
            // 紧凑形式：第1-9季
            r"(.+?)\s*第([一二三四五六七八九十\d]+)\s*[～~—–\-至到]\s*([一二三四五六七八九十\d]+)季\s*(.*)$",
        ];
        for pattern in &range_patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(title) {
                    let base_name_prefix = captures.get(1).map_or("", |m| m.as_str()).trim();
                    let season_str = captures.get(2).map_or("1", |m| m.as_str());
                    let base_name_suffix = captures.get(4).map_or("", |m| m.as_str()).trim();

                    // 合并前缀和后缀，中间用空格连接（如果后缀不为空），并清理括号前的空格
                    let base_name = if !base_name_suffix.is_empty() {
                        format!("{} {}", base_name_prefix, base_name_suffix)
                            .replace(" （", "（")  // 去除全角括号前的空格
                            .replace(" (", "(") // 去除半角括号前的空格
                    } else {
                        base_name_prefix.to_string()
                    };

                    if !base_name.is_empty() {
                        return (base_name, Self::parse_season_number(season_str));
                    }
                }
            }
        }

        // 常见的季度模式
        let patterns = [
            // 中文季度模式：第一季、第二季、第三季等（保留季度后的后缀标签）
            r"(.+?)\s*第([一二三四五六七八九十\d]+)季\s*(.*)$",
            // 英文季度模式：S1、S2、Season 1等
            r"(.+?)\s*S(\d+)\s*$",
            r"(.+?)\s*Season\s*(\d+)\s*$",
            // 日文季度模式
            r"(.+?)\s*第(\d+)期\s*",
            // 其他可能的模式
            r"(.+?)\s*(\d+)\s*$",
        ];

        for pattern in &patterns {
            if let Ok(regex) = Regex::new(pattern) {
                if let Some(captures) = regex.captures(title) {
                    let base_name_prefix = captures.get(1).map_or("", |m| m.as_str()).trim();
                    let season_str = captures.get(2).map_or("1", |m| m.as_str());
                    let base_name_suffix = captures.get(3).map_or("", |m| m.as_str()).trim();

                    // 合并前缀和后缀，中间用空格连接（如果后缀不为空），并清理括号前的空格
                    let base_name = if !base_name_suffix.is_empty() {
                        format!("{} {}", base_name_prefix, base_name_suffix)
                            .replace(" （", "（")  // 去除全角括号前的空格
                            .replace(" (", "(") // 去除半角括号前的空格
                    } else {
                        base_name_prefix.to_string()
                    };

                    let season_number = Self::parse_season_number(season_str);

                    if !base_name.is_empty() {
                        return (base_name, season_number);
                    }
                }
            }
        }

        // 如果没有匹配到任何模式，返回原标题和季度1
        (title.trim().to_string(), 1)
    }

    /// 从季度字符串中提取季度数字
    /// 优先提取“第X季”中的数字片段（避免版本后缀如“中文配音”干扰），再回退到阿拉伯数字
    fn extract_season_number(season_str: &str) -> Option<u32> {
        // 提取“第X季”片段，如 “第十一季 中文配音” -> “十一”
        if let Ok(re) = Regex::new(r"第([一二三四五六七八九十\d]+)季") {
            if let Some(captures) = re.captures(season_str) {
                if let Some(mat) = captures.get(1) {
                    return Some(Self::parse_season_number(mat.as_str()));
                }
            }
        }

        // 无“第X季”片段时回退到阿拉伯数字，如 “Season 2”
        Self::extract_number_from_string(season_str)
    }

    /// 解析季度数字（支持中文和阿拉伯数字）
    fn parse_season_number(season_str: &str) -> u32 {
        // 尝试直接解析阿拉伯数字
        if let Some(number) = Self::extract_number_from_string(season_str) {
            return number;
        }

        // 尝试中文数字（如 十一 -> 11、二十一 -> 21）
        Self::chinese_numeral_to_number(season_str).unwrap_or(1)
    }

    /// 中文数字转阿拉伯数字（十进制解析），支持 一~九十九
    ///
    /// 例如：一→1、十→10、十一→11、二十→20、二十一→21
    /// 遇到非数字字符返回 None
    pub(crate) fn chinese_numeral_to_number(s: &str) -> Option<u32> {
        let mut result: u32 = 0;
        let mut section: u32 = 0;
        let mut found = false;
        for ch in s.chars() {
            match ch {
                '零' => continue,
                '一' => section = 1,
                '二' => section = 2,
                '三' => section = 3,
                '四' => section = 4,
                '五' => section = 5,
                '六' => section = 6,
                '七' => section = 7,
                '八' => section = 8,
                '九' => section = 9,
                '十' => {
                    // “十”单独出现时按 10 处理（如 “十” -> 10）
                    if section == 0 {
                        section = 1;
                    }
                    result += section * 10;
                    section = 0;
                    found = true;
                    continue;
                }
                _ => return None,
            }
            found = true;
        }
        result += section;
        if found && result > 0 {
            Some(result)
        } else {
            None
        }
    }

    /// 从字符串中提取数字
    fn extract_number_from_string(s: &str) -> Option<u32> {
        for part in s.split_whitespace() {
            if let Ok(number) = part.parse::<u32>() {
                return Some(number);
            }
        }

        // 尝试提取字符串中的连续数字
        let re = Regex::new(r"\d+").ok()?;
        if let Some(mat) = re.find(s) {
            return mat.as_str().parse().ok();
        }

        None
    }

    /// 生成标准的季度文件夹名称
    ///
    /// # 参数
    /// - `season_number`: 季度编号
    ///
    /// # 返回值
    /// 标准的季度文件夹名称，例如 "Season 01"、"Season 02"
    pub fn generate_season_folder_name(season_number: u32) -> String {
        format!("Season {:02}", season_number)
    }

    /// 标准化系列名称，仅用于归并判断，不修改真实文件名
    /// 去除常见的版本/介质/分辨率标签，合并多余空白
    pub fn normalize_series_name(input: &str) -> String {
        use regex::Regex;

        let mut name = input.to_string();

        // 1) 去除括号或书名号/方括号内的标签（若命中关键词）
        // 支持 () [] 【】 《》
        let bracket_patterns = vec![
            r"\([^\)]*?(中配|日配|国语|粤语|配音|双语|简中|繁中|中字|外挂|内封|无修|未删减|WEB(?:-DL)?|TV|BD|Blu-?ray|UHD|4K|1080P|720P)[^\)]*?\)",
            r"\[[^\]]*?(中配|日配|国语|粤语|配音|双语|简中|繁中|中字|外挂|内封|无修|未删减|WEB(?:-DL)?|TV|BD|Blu-?ray|UHD|4K|1080P|720P)[^\]]*?\]",
            r"【[^】]*?(中配|日配|国语|粤语|配音|双语|简中|繁中|中字|外挂|内封|无修|未删减|WEB(?:-DL)?|TV|BD|Blu-?ray|UHD|4K|1080P|720P)[^】]*?】",
            r"《[^》]*?(中配|日配|国语|粤语|配音|双语|简中|繁中|中字|外挂|内封|无修|未删减|WEB(?:-DL)?|TV|BD|Blu-?ray|UHD|4K|1080P|720P)[^》]*?》",
        ];
        for pat in bracket_patterns {
            if let Ok(re) = Regex::new(pat) {
                name = re.replace_all(&name, "").to_string();
            }
        }

        // 2) 去除尾部/中间的短标签
        let tail_patterns = vec![
            r"[\s\-·]?(中配版|日配版|国语版|粤语版)$",
            r"[\s\-·]?(中配|日配|国语|粤语)$",
            r"[\s\-·]?(WEB(?:-DL)?|TV|BD|Blu-?ray|UHD)$",
            r"[ \-_·]?(4K|1080P|720P)$",
        ];
        for pat in tail_patterns {
            if let Ok(re) = Regex::new(pat) {
                name = re.replace(&name, "").to_string();
            }
        }

        // 3) 合并多余空白并trim
        if let Ok(re_space) = Regex::new(r"\s+") {
            name = re_space.replace_all(&name, " ").to_string();
        }
        name = name.trim().to_string();

        if name.is_empty() {
            input.to_string()
        } else {
            name
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_with_season_title() {
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("灵笼 第二季", Some("第二季"));
        assert_eq!(base_name, "灵笼");
        assert_eq!(season, 2);
    }

    #[test]
    fn test_extract_chinese_season() {
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("进击的巨人 第三季", None);
        assert_eq!(base_name, "进击的巨人");
        assert_eq!(season, 3);
    }

    #[test]
    fn test_extract_english_season() {
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("Attack on Titan S2", None);
        assert_eq!(base_name, "Attack on Titan");
        assert_eq!(season, 2);
    }

    #[test]
    fn test_extract_season_folder_name() {
        assert_eq!(BangumiNameExtractor::generate_season_folder_name(1), "Season 01");
        assert_eq!(BangumiNameExtractor::generate_season_folder_name(12), "Season 12");
    }

    #[test]
    fn test_no_season_info() {
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("鬼灭之刃", None);
        assert_eq!(base_name, "鬼灭之刃");
        assert_eq!(season, 1);
    }

    #[test]
    fn test_xianwang_seasons() {
        // 测试仙王的日常生活系列的不同季度
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("仙王的日常生活", None);
        assert_eq!(base_name, "仙王的日常生活");
        assert_eq!(season, 1);

        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("仙王的日常生活 第二季", None);
        assert_eq!(base_name, "仙王的日常生活");
        assert_eq!(season, 2);

        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("仙王的日常生活 第三季", None);
        assert_eq!(base_name, "仙王的日常生活");
        assert_eq!(season, 3);
    }

    #[test]
    fn test_kobayashi_seasons() {
        // 测试小林家的龙女仆系列
        let (base_name, season) =
            BangumiNameExtractor::extract_series_name_and_season("小林家的龙女仆 第二季 中配版", None);
        assert_eq!(base_name, "小林家的龙女仆 中配版");
        assert_eq!(season, 2);

        // 测试第一季（没有季度信息）
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("小林家的龙女仆 中配版", None);
        assert_eq!(base_name, "小林家的龙女仆 中配版");
        assert_eq!(season, 1);
    }

    #[test]
    fn test_spongebob_combined_seasons() {
        // B站合并季：第一季～第九季 -> 取范围起点 1，范围不进系列名
        let (base_name, season) =
            BangumiNameExtractor::extract_series_name_and_season("海绵宝宝 第一季～第九季", None);
        assert_eq!(base_name, "海绵宝宝");
        assert_eq!(season, 1);

        // 第十季
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season("海绵宝宝 第十季", None);
        assert_eq!(base_name, "海绵宝宝");
        assert_eq!(season, 10);

        // 第十一季 中文配音：多位中文数字 + 保留后缀标签
        let (base_name, season) =
            BangumiNameExtractor::extract_series_name_and_season("海绵宝宝 第十一季 中文配音", None);
        assert_eq!(base_name, "海绵宝宝 中文配音");
        assert_eq!(season, 11);
    }

    #[test]
    fn test_season_range_variants() {
        // 阿拉伯数字范围
        let (base_name, season) =
            BangumiNameExtractor::extract_series_name_and_season("海绵宝宝 第1季～第9季", None);
        assert_eq!(base_name, "海绵宝宝");
        assert_eq!(season, 1);

        // 紧凑范围形式
        let (base_name, season) =
            BangumiNameExtractor::extract_series_name_and_season("海绵宝宝 第1-9季", None);
        assert_eq!(base_name, "海绵宝宝");
        assert_eq!(season, 1);

        // 半角波浪线
        let (base_name, season) =
            BangumiNameExtractor::extract_series_name_and_season("海绵宝宝 第一季~第九季", None);
        assert_eq!(base_name, "海绵宝宝");
        assert_eq!(season, 1);
    }

    #[test]
    fn test_chinese_numeral_conversion() {
        let cases = [
            ("一", 1),
            ("二", 2),
            ("九", 9),
            ("十", 10),
            ("十一", 11),
            ("十二", 12),
            ("二十", 20),
            ("二十一", 21),
            ("九十九", 99),
        ];
        for (input, expected) in cases {
            assert_eq!(
                BangumiNameExtractor::chinese_numeral_to_number(input),
                Some(expected),
                "中文数字 {} 应转换为 {}",
                input,
                expected
            );
        }
        assert_eq!(BangumiNameExtractor::chinese_numeral_to_number("abc"), None);
        assert_eq!(BangumiNameExtractor::chinese_numeral_to_number(""), None);
    }

    #[test]
    fn test_season_title_with_suffix() {
        // season_title 路径：多位数 + 版本后缀不应干扰季号
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season(
            "海绵宝宝 第十一季 中文配音",
            Some("第十一季 中文配音"),
        );
        assert_eq!(base_name, "海绵宝宝");
        assert_eq!(season, 11);

        // season_title 为范围时取起点
        let (base_name, season) = BangumiNameExtractor::extract_series_name_and_season(
            "海绵宝宝 第一季～第九季",
            Some("第一季～第九季"),
        );
        assert_eq!(base_name, "海绵宝宝");
        assert_eq!(season, 1);
    }
}
