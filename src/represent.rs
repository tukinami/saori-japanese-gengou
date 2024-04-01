use chrono::{Datelike, NaiveDate};

use crate::{search::SearchedSpanList, span::NaiveDateSpan};

pub(crate) fn represent_by_gregorian(
    searched_list: &SearchedSpanList<'_>,
    date: &NaiveDate,
    is_kansuuji: bool,
) -> (Vec<String>, Vec<String>) {
    let mut r_dates = Vec::new();
    let mut r_spans = Vec::new();

    for (file_span, initial, spans) in searched_list.iter() {
        for span in spans.iter() {
            let year_i = date.year() - span.span().start().year() + 1;
            let year = if year_i == 1 {
                "元".to_string()
            } else if is_kansuuji {
                to_kansuuji(year_i as u32)
            } else {
                format!("{}", year_i)
            };

            let (month, day) = if is_kansuuji {
                (to_kansuuji(date.month()), to_kansuuji(date.day()))
            } else {
                (date.month().to_string(), date.day().to_string())
            };

            let date_str = format!("{}{}年{}月{}日", span.gengou(), year, month, day,);

            r_dates.push(date_str);
            r_spans.push(represent_span(file_span, initial));
        }
    }

    (r_dates, r_spans)
}

fn represent_span(file_span: &NaiveDateSpan, initial: &char) -> String {
    format!(
        "{}_{}_{}",
        file_span.start().format("%Y-%m-%d"),
        file_span.end().format("%Y-%m-%d"),
        initial
    )
}

const NUMS: [&str; 10] = ["", "一", "二", "三", "四", "五", "六", "七", "八", "九"];
const SUBS: [&str; 4] = ["", "十", "百", "千"];
const PARTS: [&str; 18] = [
    "",
    "万",
    "億",
    "兆",
    "京",
    "垓",
    "𥝱",
    "穣",
    "溝",
    "澗",
    "正",
    "載",
    "極",
    "恒河沙",
    "阿僧祇",
    "那由他",
    "不可思議",
    "無量大数",
];

fn to_kansuuji(value: u32) -> String {
    let mut buf: Vec<&'static str> = Vec::new();

    let v_str = value.to_string();
    let v_bytes = v_str.as_bytes();
    let v_bytes_len = v_bytes.len();

    let mut parts_flag = true;
    for (i, v_byte) in v_bytes.iter().enumerate() {
        let code = (v_byte - 48) as usize;
        let class = v_bytes_len - i - 1;

        if code != 0 {
            parts_flag = true;
        }

        // 数字本体
        if !(class % 4 != 0 && code == 1) {
            buf.push(NUMS[code]);
        }
        // 十百千
        if code != 0 {
            buf.push(SUBS[class % 4]);
        }
        // 万億兆……
        if parts_flag && class % 4 == 0 {
            buf.push(PARTS[class / 4]);
            parts_flag = false;
        }
    }

    buf.join("")
}

#[cfg(test)]
mod tests {
    use super::*;

    mod represent_by_gregorian {
        use crate::span::Span;

        use super::*;

        #[test]
        fn checking_value_normal_multiple() {
            let file_span = NaiveDateSpan::new(
                NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            );

            let case_a_1 = Span::new(
                "aa".to_string(),
                NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
            );
            let case_a_2 = Span::new(
                "ab".to_string(),
                NaiveDate::from_ymd_opt(150, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
            );
            let spans_a = vec![&case_a_1, &case_a_2];

            let case_b_1 = Span::new(
                "ba".to_string(),
                NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
            );
            let spans_b = vec![&case_b_1];

            let searched_list = vec![(&file_span, 'a', spans_a), (&file_span, 'b', spans_b)];

            let date = NaiveDate::from_ymd_opt(150, 1, 1).unwrap();
            let is_kansuuji = false;

            let (r_dates, r_spans) = represent_by_gregorian(&searched_list, &date, is_kansuuji);
            assert_eq!(
                r_dates,
                vec![
                    "aa51年1月1日".to_string(),
                    "ab元年1月1日".to_string(),
                    "ba51年1月1日".to_string(),
                ]
            );
            assert_eq!(
                r_spans,
                vec![
                    "0100-01-01_1000-01-01_a".to_string(),
                    "0100-01-01_1000-01-01_a".to_string(),
                    "0100-01-01_1000-01-01_b".to_string(),
                ]
            )
        }
    }

    mod represent_span {
        use super::*;

        #[test]
        fn checking_value() {
            let file_span = NaiveDateSpan::new(
                NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            );
            let initial = 'a';

            let result = represent_span(&file_span, &initial);
            assert_eq!(result, "0100-01-01_1000-01-01_a".to_string());
        }
    }

    mod to_kansuuji {
        use super::*;

        #[test]
        fn checking_value() {
            assert_eq!(to_kansuuji(1), "一");
            assert_eq!(to_kansuuji(9), "九");
            assert_eq!(to_kansuuji(10), "十");
            assert_eq!(to_kansuuji(11), "十一");
            assert_eq!(to_kansuuji(21), "二十一");
            assert_eq!(to_kansuuji(99), "九十九");
            assert_eq!(to_kansuuji(100), "百");
            assert_eq!(to_kansuuji(999), "九百九十九");
            assert_eq!(to_kansuuji(1000), "千");
            assert_eq!(to_kansuuji(9999), "九千九百九十九");
            assert_eq!(to_kansuuji(10000), "一万");
            assert_eq!(to_kansuuji(10020), "一万二十");
            assert_eq!(to_kansuuji(1_000_020), "百万二十");
            assert_eq!(to_kansuuji(100_000_020), "一億二十");
            assert_eq!(to_kansuuji(1_0000_4423), "一億四千四百二十三");
            assert_eq!(to_kansuuji(1_8000_4423), "一億八千万四千四百二十三");
        }
    }
}
