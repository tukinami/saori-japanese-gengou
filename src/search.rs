use chrono::NaiveDate;

use crate::span::{NaiveDateSpan, Span, SpanListArray};

pub(crate) type SearchedSpanList<'a> = Vec<(&'a NaiveDateSpan, char, Vec<&'a Span>)>;

pub(crate) fn search_from_array<'a>(
    spans_array: &'a SpanListArray,
    date: &NaiveDate,
    selector: &[&str],
) -> SearchedSpanList<'a> {
    let mut result = Vec::new();

    let mut selector = selector.iter();
    for (span, spans_map) in spans_array.iter() {
        let s = selector.next().unwrap_or(&"*");

        if span.start() <= date && span.end() >= date {
            let span_list = spans_map
                .iter()
                .filter(|v| s.contains('*') || s.contains(*v.0) || s.is_empty());

            for (i, l) in span_list {
                let searched_list = search_from_list(l.spans(), date);
                let position = result
                    .binary_search_by(|(d, c, _): &(&NaiveDateSpan, char, _)| match d.cmp(&span) {
                        std::cmp::Ordering::Equal => c.cmp(i),
                        o => o,
                    })
                    .unwrap_or_else(|v| v);
                result.insert(position, (span, *i, searched_list));
            }
        }
    }

    result
}

fn search_from_list<'a>(spans: &'a [Span], date: &NaiveDate) -> Vec<&'a Span> {
    let start_point = spans.partition_point(|t| t.span().start() < date && t.span().end() < date);
    let end_point = spans.partition_point(|t| t.span().start() <= date);

    spans[start_point..end_point].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    mod search_from_array {
        use std::collections::HashMap;

        use crate::span::{NaiveDateSpan, SpanList};

        use super::*;

        #[test]
        fn return_single_element_when_only_one_element_matches() {
            let date_span = NaiveDateSpan::new(
                NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            );
            let mut map = HashMap::new();
            map.insert(
                'a',
                SpanList::new(
                    date_span.clone(),
                    'a',
                    vec![
                        Span::new(
                            "a".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "b".to_string(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "c".to_string(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "d".to_string(),
                            NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "e".to_string(),
                            NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "f".to_string(),
                            NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        ),
                    ],
                ),
            );

            map.insert(
                'b',
                SpanList::new(
                    date_span.clone(),
                    'b',
                    vec![
                        Span::new(
                            "A".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "B".to_string(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "C".to_string(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "D".to_string(),
                            NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "E".to_string(),
                            NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "F".to_string(),
                            NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        ),
                    ],
                ),
            );
            let array = vec![(date_span.clone(), map)];

            let date = NaiveDate::from_ymd_opt(150, 1, 1).unwrap();
            let selector = ["b"];

            let result = search_from_array(&array, &date, &selector);

            assert_eq!(
                result,
                vec![(
                    &date_span,
                    'b',
                    vec![&Span::new(
                        "A".to_string(),
                        NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                        NaiveDate::from_ymd_opt(200, 1, 1).unwrap()
                    )]
                )]
            );
        }

        #[test]
        fn return_multiple_elements_when_all_elements_match() {
            let date_span = NaiveDateSpan::new(
                NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            );
            let mut map = HashMap::new();
            map.insert(
                'a',
                SpanList::new(
                    date_span.clone(),
                    'a',
                    vec![
                        Span::new(
                            "a".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "b".to_string(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "c".to_string(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "d".to_string(),
                            NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "e".to_string(),
                            NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "f".to_string(),
                            NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        ),
                    ],
                ),
            );

            map.insert(
                'b',
                SpanList::new(
                    date_span.clone(),
                    'b',
                    vec![
                        Span::new(
                            "A".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "B".to_string(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "C".to_string(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "D".to_string(),
                            NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "E".to_string(),
                            NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "F".to_string(),
                            NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        ),
                    ],
                ),
            );
            let array = vec![(date_span.clone(), map)];

            let date = NaiveDate::from_ymd_opt(150, 1, 1).unwrap();
            let selector = ["*"];

            let result = search_from_array(&array, &date, &selector);

            assert_eq!(
                result,
                vec![
                    (
                        &date_span,
                        'a',
                        vec![&Span::new(
                            "a".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap()
                        )]
                    ),
                    (
                        &date_span,
                        'b',
                        vec![&Span::new(
                            "A".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap()
                        )]
                    ),
                ]
            );
        }

        #[test]
        fn return_multiple_elements_when_selector_default() {
            let date_span = NaiveDateSpan::new(
                NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
            );
            let mut map = HashMap::new();
            map.insert(
                'a',
                SpanList::new(
                    date_span.clone(),
                    'a',
                    vec![
                        Span::new(
                            "a".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "b".to_string(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "c".to_string(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "d".to_string(),
                            NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "e".to_string(),
                            NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "f".to_string(),
                            NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        ),
                    ],
                ),
            );

            map.insert(
                'b',
                SpanList::new(
                    date_span.clone(),
                    'b',
                    vec![
                        Span::new(
                            "A".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "B".to_string(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "C".to_string(),
                            NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "D".to_string(),
                            NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "E".to_string(),
                            NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                        ),
                        Span::new(
                            "F".to_string(),
                            NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                        ),
                    ],
                ),
            );
            let array = vec![(date_span.clone(), map)];

            let date = NaiveDate::from_ymd_opt(150, 1, 1).unwrap();
            let selector = [];

            let result = search_from_array(&array, &date, &selector);

            assert_eq!(
                result,
                vec![
                    (
                        &date_span,
                        'a',
                        vec![&Span::new(
                            "a".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap()
                        )]
                    ),
                    (
                        &date_span,
                        'b',
                        vec![&Span::new(
                            "A".to_string(),
                            NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                            NaiveDate::from_ymd_opt(200, 1, 1).unwrap()
                        )]
                    ),
                ]
            );
        }
    }

    mod search_from_list {
        use super::*;

        #[test]
        fn return_single_element_when_only_one_element_matches_middle() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result = search_from_list(&case_list, &NaiveDate::from_ymd_opt(350, 1, 1).unwrap());

            assert_eq!(
                result,
                vec![&Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap()
                ),]
            );
        }

        #[test]
        fn return_multiple_elements_when_multiple_elements_match_border() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result = search_from_list(&case_list, &NaiveDate::from_ymd_opt(300, 1, 1).unwrap());

            assert_eq!(
                result,
                vec![
                    &Span::new(
                        "b".to_string(),
                        NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                        NaiveDate::from_ymd_opt(300, 1, 1).unwrap()
                    ),
                    &Span::new(
                        "c".to_string(),
                        NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                        NaiveDate::from_ymd_opt(400, 1, 1).unwrap()
                    ),
                ]
            );
        }

        #[test]
        fn return_single_element_when_only_element_matches_border_start() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result = search_from_list(&case_list, &NaiveDate::from_ymd_opt(500, 1, 1).unwrap());

            assert_eq!(
                result,
                vec![&Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap()
                ),]
            );
        }

        #[test]
        fn return_single_element_when_only_element_matches_border_start_top() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result = search_from_list(&case_list, &NaiveDate::from_ymd_opt(100, 1, 1).unwrap());

            assert_eq!(
                result,
                vec![&Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap()
                ),]
            );
        }

        #[test]
        fn return_single_element_when_only_element_matches_border_start_bottom() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result =
                search_from_list(&case_list, &NaiveDate::from_ymd_opt(1000, 1, 1).unwrap());

            assert_eq!(
                result,
                vec![&Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap()
                ),]
            );
        }

        #[test]
        fn return_single_element_when_only_element_matches_border_end() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result = search_from_list(&case_list, &NaiveDate::from_ymd_opt(600, 1, 1).unwrap());

            assert_eq!(
                result,
                vec![&Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap()
                ),]
            );
        }

        #[test]
        fn return_nothing_when_no_match_spans_middle() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result = search_from_list(&case_list, &NaiveDate::from_ymd_opt(450, 1, 1).unwrap());

            assert!(result.is_empty());
        }

        #[test]
        fn return_nothing_when_no_match_spans_top() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result = search_from_list(&case_list, &NaiveDate::from_ymd_opt(50, 1, 1).unwrap());

            assert!(result.is_empty());
        }

        #[test]
        fn return_nothing_when_no_match_spans_bottom() {
            let case_list = vec![
                Span::new(
                    "a".to_string(),
                    NaiveDate::from_ymd_opt(100, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                ),
                Span::new(
                    "b".to_string(),
                    NaiveDate::from_ymd_opt(200, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                ),
                Span::new(
                    "c".to_string(),
                    NaiveDate::from_ymd_opt(300, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(400, 1, 1).unwrap(),
                ),
                Span::new(
                    "d".to_string(),
                    NaiveDate::from_ymd_opt(500, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(600, 1, 1).unwrap(),
                ),
                Span::new(
                    "e".to_string(),
                    NaiveDate::from_ymd_opt(700, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(800, 1, 1).unwrap(),
                ),
                Span::new(
                    "f".to_string(),
                    NaiveDate::from_ymd_opt(900, 1, 1).unwrap(),
                    NaiveDate::from_ymd_opt(1000, 1, 1).unwrap(),
                ),
            ];

            let result =
                search_from_list(&case_list, &NaiveDate::from_ymd_opt(1100, 1, 1).unwrap());

            assert!(result.is_empty());
        }
    }
}
