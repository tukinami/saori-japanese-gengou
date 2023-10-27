use std::{
    collections::HashMap,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use chrono::NaiveDate;

const SPAN_DIR_PATH: &str = "gengou_lists";

pub(crate) type SpanListArray = Vec<(NaiveDateSpan, HashMap<char, SpanList>)>;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SpanList {
    span: NaiveDateSpan,
    initial: char,
    spans: Vec<Span>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Span {
    gengou: String,
    span: NaiveDateSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct NaiveDateSpan {
    start: NaiveDate,
    end: NaiveDate,
}

impl SpanList {
    pub fn new(span: NaiveDateSpan, initial: char, spans: Vec<Span>) -> SpanList {
        SpanList {
            span,
            initial,
            spans,
        }
    }

    pub fn span(&self) -> &NaiveDateSpan {
        &self.span
    }

    pub fn initial(&self) -> &char {
        &self.initial
    }

    pub fn spans(&self) -> &Vec<Span> {
        &self.spans
    }
}

impl Span {
    pub fn new(gengou: String, start: NaiveDate, end: NaiveDate) -> Span {
        Span {
            gengou,
            span: NaiveDateSpan::new(start, end),
        }
    }

    pub fn gengou(&self) -> &str {
        &self.gengou
    }

    pub fn span(&self) -> &NaiveDateSpan {
        &self.span
    }
}

impl NaiveDateSpan {
    pub fn new(start: NaiveDate, end: NaiveDate) -> NaiveDateSpan {
        NaiveDateSpan { start, end }
    }

    pub fn start(&self) -> &NaiveDate {
        &self.start
    }

    pub fn end(&self) -> &NaiveDate {
        &self.end
    }
}

impl PartialOrd for NaiveDateSpan {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for NaiveDateSpan {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.start().cmp(other.start()) {
            std::cmp::Ordering::Equal => self.end().cmp(other.end()),
            o => o,
        }
    }
}

pub(crate) fn load_spans(base_path: &Path) -> Result<SpanListArray, std::io::Error> {
    let path = PathBuf::from(base_path).join(SPAN_DIR_PATH);
    let mut span_list_array: SpanListArray = Vec::new();

    for entry in path.read_dir()? {
        let dir = entry?;
        if let Some(span_list) = parse_span_list_file(&dir.path())? {
            let naive_date_span = span_list.span().clone();
            let initial = *span_list.initial();

            match span_list_array.binary_search_by(|v| v.0.cmp(&naive_date_span)) {
                Ok(i) => {
                    let target = span_list_array.get_mut(i).expect("already searched");
                    target.1.insert(initial, span_list);
                }
                Err(i) => {
                    let mut target = HashMap::new();
                    target.insert(initial, span_list);
                    span_list_array.insert(i, (naive_date_span, target));
                }
            };
        }
    }

    Ok(span_list_array)
}

fn parse_span_list_file(path: &Path) -> Result<Option<SpanList>, std::io::Error> {
    let filestem = if let Some(s) = path.file_stem() {
        s.to_string_lossy()
    } else {
        return Ok(None);
    };
    // ファイル名の確認
    if !is_target_filestem(&filestem) {
        return Ok(None);
    }

    let initial = if let Some(i) = get_initial_char_after_underbar(&filestem) {
        i
    } else {
        return Ok(None);
    };

    let mut fs = File::open(path)?;
    let mut contents = String::new();
    fs.read_to_string(&mut contents)?;

    let (span, spans) = if let Some(v) = parse_contents(&contents)? {
        v
    } else {
        return Ok(None);
    };

    Ok(Some(SpanList::new(span, initial, spans)))
}

fn is_target_filestem(s: &str) -> bool {
    // '00'で始まること
    if !s.starts_with("00") {
        return false;
    }
    // '_'があること
    let underbar_point = if let Some(p) = s.find('_') {
        p + 1
    } else {
        return false;
    };
    // '_'の次になにか文字があること
    if underbar_point >= s.len() || !s.is_char_boundary(underbar_point) {
        return false;
    }

    true
}

fn get_initial_char_after_underbar(s: &str) -> Option<char> {
    s.split_once('_').and_then(|(_lhs, rhs)| rhs.chars().next())
}

fn parse_contents(contents: &str) -> Result<Option<(NaiveDateSpan, Vec<Span>)>, std::io::Error> {
    let lines = contents.lines();
    let mut spans = Vec::new();
    let method = |target: &Span, value: &Span| target.span() < value.span();
    for line in lines {
        if let Some(line_span) = parse_line(line)? {
            insertion_sort(&mut spans, line_span, method);
        }
    }

    let (start, end) = if let (Some(first), Some(last)) = (spans.first(), spans.last()) {
        (*first.span().start(), *last.span.end())
    } else {
        return Ok(None);
    };
    let span = NaiveDateSpan::new(start, end);

    Ok(Some((span, spans)))
}

fn parse_line(s: &str) -> Result<Option<Span>, std::io::Error> {
    // コメント処理
    let comment_point = s.find("//");
    let body = if let Some(point) = comment_point {
        let (lhs, _rhs) = s.split_at(point);
        lhs.trim()
    } else {
        s.trim()
    };
    // 空行ならNoneを返す
    if body.is_empty() {
        return Ok(None);
    }

    let mut splited = body.split(',');

    if let (Some(gengou), Some(start_str), Some(end_str)) =
        (splited.next(), splited.next(), splited.next())
    {
        let start = parse_datetime(start_str.trim())?;
        let end = parse_datetime(end_str.trim())?;

        // 逆になっていたら直す
        if end < start {
            Ok(Some(Span::new(gengou.trim().to_string(), end, start)))
        } else {
            Ok(Some(Span::new(gengou.trim().to_string(), start, end)))
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            "line format is invalid. the format is 'gengou,%Y-%m-%d,%Y-%m-%d'.",
        ))
    }
}

fn parse_datetime(s: &str) -> Result<NaiveDate, std::io::Error> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").or_else(|_| {
        if s.eq("****") {
            Ok(chrono::Local::now().date_naive())
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "date format is invalid. the format is '%Y-%m-%d' or '****' (current date)",
            ))
        }
    })
}

fn insertion_sort<T, F>(list: &mut Vec<T>, value: T, method: F)
where
    F: Fn(&T, &T) -> bool,
{
    let mut target_index = list.len();
    while target_index > 0 {
        if method(&list[target_index - 1], &value) {
            break;
        }
        target_index -= 1;
    }

    list.insert(target_index, value);
}

#[cfg(test)]
mod tests {
    use super::*;

    mod load_spans {
        use super::*;

        #[test]
        fn success_when_valid_path_to_dir() {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            let result = load_spans(&path).unwrap();

            let expect = vec![
                (
                    1,
                    NaiveDateSpan::new(
                        NaiveDate::from_ymd_opt(645, 8, 1).unwrap(),
                        NaiveDate::from_ymd_opt(1329, 9, 30).unwrap(),
                    ),
                ),
                (
                    2,
                    NaiveDateSpan::new(
                        NaiveDate::from_ymd_opt(1329, 09, 30).unwrap(),
                        NaiveDate::from_ymd_opt(1394, 8, 10).unwrap(),
                    ),
                ),
                (
                    1,
                    NaiveDateSpan::new(
                        NaiveDate::from_ymd_opt(1394, 8, 10).unwrap(),
                        NaiveDate::from_ymd_opt(1573, 9, 4).unwrap(),
                    ),
                ),
                (
                    1,
                    NaiveDateSpan::new(
                        NaiveDate::from_ymd_opt(1573, 9, 4).unwrap(),
                        NaiveDate::from_ymd_opt(1868, 10, 23).unwrap(),
                    ),
                ),
            ];

            assert_eq!(result.len(), 5);

            for (i, (n, span)) in expect.iter().enumerate() {
                let target = result.get(i).unwrap();
                assert_eq!(target.1.len(), *n);
                assert_eq!(&target.0, span);
            }
        }
    }

    mod parse_span_list_file {
        use super::*;

        #[test]
        fn success_when_valid_path_to_file() {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(SPAN_DIR_PATH)
                .join("002_nantyou.txt");
            let result = parse_span_list_file(&path).unwrap().unwrap();

            assert_eq!(result.initial(), &'n');
            assert_eq!(
                result.span().start(),
                &NaiveDate::from_ymd_opt(1329, 09, 30).unwrap()
            );
            assert_eq!(
                result.span().end(),
                &NaiveDate::from_ymd_opt(1394, 08, 10).unwrap(),
            );

            let expect = vec![
                Span::new(
                    "元徳".to_string(),
                    NaiveDate::from_ymd_opt(1329, 09, 30).unwrap(),
                    NaiveDate::from_ymd_opt(1331, 09, 18).unwrap(),
                ),
                Span::new(
                    "元弘".to_string(),
                    NaiveDate::from_ymd_opt(1331, 09, 18).unwrap(),
                    NaiveDate::from_ymd_opt(1334, 03, 13).unwrap(),
                ),
                Span::new(
                    "建武".to_string(),
                    NaiveDate::from_ymd_opt(1334, 03, 13).unwrap(),
                    NaiveDate::from_ymd_opt(1336, 04, 19).unwrap(),
                ),
                Span::new(
                    "延元".to_string(),
                    NaiveDate::from_ymd_opt(1336, 04, 19).unwrap(),
                    NaiveDate::from_ymd_opt(1340, 06, 02).unwrap(),
                ),
                Span::new(
                    "興国".to_string(),
                    NaiveDate::from_ymd_opt(1340, 06, 02).unwrap(),
                    NaiveDate::from_ymd_opt(1347, 01, 28).unwrap(),
                ),
                Span::new(
                    "正平".to_string(),
                    NaiveDate::from_ymd_opt(1347, 01, 28).unwrap(),
                    NaiveDate::from_ymd_opt(1370, 08, 24).unwrap(),
                ),
                Span::new(
                    "建徳".to_string(),
                    NaiveDate::from_ymd_opt(1370, 08, 24).unwrap(),
                    NaiveDate::from_ymd_opt(1372, 05, 09).unwrap(),
                ),
                Span::new(
                    "文中".to_string(),
                    NaiveDate::from_ymd_opt(1372, 05, 09).unwrap(),
                    NaiveDate::from_ymd_opt(1375, 07, 04).unwrap(),
                ),
                Span::new(
                    "天授".to_string(),
                    NaiveDate::from_ymd_opt(1375, 07, 04).unwrap(),
                    NaiveDate::from_ymd_opt(1381, 03, 14).unwrap(),
                ),
                Span::new(
                    "弘和".to_string(),
                    NaiveDate::from_ymd_opt(1381, 03, 14).unwrap(),
                    NaiveDate::from_ymd_opt(1384, 05, 26).unwrap(),
                ),
                Span::new(
                    "元中".to_string(),
                    NaiveDate::from_ymd_opt(1384, 05, 26).unwrap(),
                    NaiveDate::from_ymd_opt(1392, 11, 27).unwrap(),
                ),
                Span::new(
                    "明徳".to_string(),
                    NaiveDate::from_ymd_opt(1392, 11, 27).unwrap(),
                    NaiveDate::from_ymd_opt(1394, 08, 10).unwrap(),
                ),
            ];

            for (r, e) in result.spans().iter().zip(expect.iter()) {
                assert_eq!(r.gengou(), e.gengou());
                assert_eq!(r.span().start(), e.span().start());
                assert_eq!(r.span().end(), e.span().end());
            }
        }

        #[test]
        fn success_when_valid_path_to_file_checking_value() {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(SPAN_DIR_PATH)
                .join("002_hokutyou.txt");
            let result = parse_span_list_file(&path).unwrap().unwrap();

            assert_eq!(result.initial(), &'h');
            assert_eq!(
                result.span().start(),
                &NaiveDate::from_ymd_opt(1329, 09, 30).unwrap()
            );
            assert_eq!(
                result.span().end(),
                &NaiveDate::from_ymd_opt(1394, 08, 10).unwrap(),
            );
        }

        #[test]
        fn success_and_return_none_when_the_file_is_not_target() {
            let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join(SPAN_DIR_PATH)
                .join("README.md");
            assert!(parse_span_list_file(&path).unwrap().is_none());
        }
    }

    mod is_target_filestem {
        use super::*;

        #[test]
        fn return_true_if_it_is_target_filename() {
            assert!(is_target_filestem("001_asuka_nara_heian"));
        }

        #[test]
        fn return_false_if_it_does_not_start_with_0() {
            assert!(!is_target_filestem("1_asuka_nara_heian"));
        }

        #[test]
        fn return_false_if_it_does_not_contain_underbar() {
            assert!(!is_target_filestem("001-asuka-nara-heian"));
        }

        #[test]
        fn return_false_if_there_are_no_char_next_underbar() {
            assert!(!is_target_filestem("001_"));
        }
    }

    mod get_initial_char_after_underbar {
        use super::*;

        #[test]
        fn return_char_when_next_char_exists() {
            let result = get_initial_char_after_underbar("001_asuka_nara_heian");
            assert_eq!(result, Some('a'));
        }

        #[test]
        fn return_none_when_next_char_does_not_exist() {
            let result = get_initial_char_after_underbar("001_");
            assert_eq!(result, None);
        }
    }

    mod parse_contents {
        use super::*;

        #[test]
        fn success_when_valid_contents() {
            let case = r#"// 1868-10-23_****
    // コメント
    // 書式:
    // 元号(なかった時期は空文字),yyyy-mm-dd(始期),yyyy-mm-dd(終期)(改行)
    // 先頭から順に処理されます
    // ユリウス暦であった1573-08-25以前の日付は、グレゴリオ暦に変換してあります。
    // 明治以降
    明治,1868-01-25,1912-07-29
    大正,1912-07-30,1926-12-24
    昭和,1926-12-25,1989-01-07
    平成,1989-01-08,2019-04-30
    令和,2019-05-01,****
    "#;
            let (span, spans) = parse_contents(&case).unwrap().unwrap();
            let now = chrono::Local::now().date_naive();

            assert_eq!(span.start(), &NaiveDate::from_ymd_opt(1868, 1, 25).unwrap());
            assert_eq!(span.end(), &now);

            let expect = vec![
                Span::new(
                    "明治".to_string(),
                    NaiveDate::from_ymd_opt(1868, 1, 25).unwrap(),
                    NaiveDate::from_ymd_opt(1912, 7, 29).unwrap(),
                ),
                Span::new(
                    "大正".to_string(),
                    NaiveDate::from_ymd_opt(1912, 7, 30).unwrap(),
                    NaiveDate::from_ymd_opt(1926, 12, 24).unwrap(),
                ),
                Span::new(
                    "昭和".to_string(),
                    NaiveDate::from_ymd_opt(1926, 12, 25).unwrap(),
                    NaiveDate::from_ymd_opt(1989, 1, 7).unwrap(),
                ),
                Span::new(
                    "平成".to_string(),
                    NaiveDate::from_ymd_opt(1989, 1, 8).unwrap(),
                    NaiveDate::from_ymd_opt(2019, 4, 30).unwrap(),
                ),
                Span::new(
                    "令和".to_string(),
                    NaiveDate::from_ymd_opt(2019, 5, 1).unwrap(),
                    now.clone(),
                ),
            ];

            for (r, e) in spans.iter().zip(expect.iter()) {
                assert_eq!(r.gengou(), e.gengou());
                assert_eq!(r.span().start(), e.span().start());
                assert_eq!(r.span().end(), e.span().end());
            }
        }

        #[test]
        fn success_when_valid_randomize_contents() {
            let case = r#"
令和,****,2019-05-01
平成,2019-04-30,1989-01-08
明治,1912-07-29,1868-01-25
昭和,1989-01-07,1926-12-25
大正,1926-12-24,1912-07-30
    "#;
            let (span, spans) = parse_contents(&case).unwrap().unwrap();
            let now = chrono::Local::now().date_naive();

            assert_eq!(span.start(), &NaiveDate::from_ymd_opt(1868, 1, 25).unwrap());
            assert_eq!(span.end(), &now);

            let expect = vec![
                Span::new(
                    "明治".to_string(),
                    NaiveDate::from_ymd_opt(1868, 1, 25).unwrap(),
                    NaiveDate::from_ymd_opt(1912, 7, 29).unwrap(),
                ),
                Span::new(
                    "大正".to_string(),
                    NaiveDate::from_ymd_opt(1912, 7, 30).unwrap(),
                    NaiveDate::from_ymd_opt(1926, 12, 24).unwrap(),
                ),
                Span::new(
                    "昭和".to_string(),
                    NaiveDate::from_ymd_opt(1926, 12, 25).unwrap(),
                    NaiveDate::from_ymd_opt(1989, 1, 7).unwrap(),
                ),
                Span::new(
                    "平成".to_string(),
                    NaiveDate::from_ymd_opt(1989, 1, 8).unwrap(),
                    NaiveDate::from_ymd_opt(2019, 4, 30).unwrap(),
                ),
                Span::new(
                    "令和".to_string(),
                    NaiveDate::from_ymd_opt(2019, 5, 1).unwrap(),
                    now.clone(),
                ),
            ];

            for (r, e) in spans.iter().zip(expect.iter()) {
                assert_eq!(r.gengou(), e.gengou());
                assert_eq!(r.span().start(), e.span().start());
                assert_eq!(r.span().end(), e.span().end());
            }
        }

        #[test]
        fn failed_when_containing_invalid_line() {
            let case = r#"
令和,2019-05-01,****
平成,1989-01-08,2019-04-30
明治,1868_01-25,1912-07-29
昭和,1926-12-25,1989-01-07
大正,1912-07-30,1926-12-24
    "#;
            assert!(parse_contents(case).is_err());
        }

        #[test]
        fn success_and_return_none_when_empty_data() {
            let case = r#"
// 令和,2019-05-01,****
//平成,1989-01-08,2019-04-30
//明治,1868-01-25,1912-07-29
//昭和,1926-12-25,1989-01-07
//大正,1912-07-30,1926-12-24
    "#;
            assert!(parse_contents(&case).unwrap().is_none());
        }
    }

    mod parse_line {
        use super::*;

        #[test]
        fn success_and_return_none_when_only_comment() {
            let case = " // comment";
            let result = parse_line(case).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn success_and_return_none_when_empty_line() {
            let case = "";
            let result = parse_line(case).unwrap();
            assert!(result.is_none());
        }

        #[test]
        fn success_and_return_some_when_valid_line() {
            let case = ",655-02-15,686-08-17 // comment";
            let result = parse_line(case).unwrap().unwrap();
            assert_eq!(result.gengou(), "");
            assert_eq!(
                result.span().start(),
                &NaiveDate::from_ymd_opt(655, 2, 15).unwrap()
            );
            assert_eq!(
                result.span().end(),
                &NaiveDate::from_ymd_opt(686, 8, 17).unwrap()
            );

            let case = "大化,645-08-01,650-03-25";
            let result = parse_line(case).unwrap().unwrap();
            assert_eq!(result.gengou(), "大化");
            assert_eq!(
                result.span().start(),
                &NaiveDate::from_ymd_opt(645, 8, 1).unwrap()
            );
            assert_eq!(
                result.span().end(),
                &NaiveDate::from_ymd_opt(650, 3, 25).unwrap()
            );
        }

        #[test]
        fn failed_when_invalid_lines_parts() {
            let case = ",655-02-15";
            assert!(parse_line(case).is_err());
        }

        #[test]
        fn failed_when_invalid_dateformat() {
            let case = ",655-02-15,686_08-17";
            assert!(parse_line(case).is_err());
        }
    }

    mod parse_datetime {
        use chrono::Datelike;

        use super::*;

        #[test]
        fn success_when_valid_format_str() {
            let result = parse_datetime("645-08-01").unwrap();
            assert_eq!(result.year(), 645);
            assert_eq!(result.month(), 8);
            assert_eq!(result.day(), 1);

            let result = parse_datetime("0645-08-01").unwrap();
            assert_eq!(result.year(), 645);
            assert_eq!(result.month(), 8);
            assert_eq!(result.day(), 1);
        }

        #[test]
        fn success_when_valid_current_date_str() {
            let result = parse_datetime("****").unwrap();
            let now = chrono::Local::now().date_naive();
            assert_eq!(result, now);
        }

        #[test]
        fn failed_when_invalid_alphabetic_format_str() {
            assert!(parse_datetime("aaa-bb-cc").is_err());
        }

        #[test]
        fn failed_when_invalid_symbol_format_str() {
            assert!(parse_datetime("645_08_01").is_err());
        }
    }

    mod insertion_sort {
        use super::*;

        #[test]
        fn when_last_greater_then_value() {
            let mut list = vec![2, 4, 6];
            let value = 5;
            let method = |v1: &i32, v2: &i32| v1 < v2;

            insertion_sort(&mut list, value, method);

            assert_eq!(list, vec![2, 4, 5, 6]);
        }

        #[test]
        fn when_last_less_then_value() {
            let mut list = vec![2, 4, 6];
            let value = 7;
            let method = |v1: &i32, v2: &i32| v1 < v2;

            insertion_sort(&mut list, value, method);

            assert_eq!(list, vec![2, 4, 6, 7]);
        }

        #[test]
        fn when_first_greater_then_value() {
            let mut list = vec![2, 4, 6];
            let value = 1;
            let method = |v1: &i32, v2: &i32| v1 < v2;

            insertion_sort(&mut list, value, method);

            assert_eq!(list, vec![1, 2, 4, 6]);
        }

        #[test]
        fn when_first_less_then_value() {
            let mut list = vec![2, 4, 6];
            let value = 3;
            let method = |v1: &i32, v2: &i32| v1 < v2;

            insertion_sort(&mut list, value, method);

            assert_eq!(list, vec![2, 3, 4, 6]);
        }
    }
}
