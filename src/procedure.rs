use std::path::PathBuf;

use chrono::NaiveDate;

use crate::represent::represent_by_gregorian;
use crate::request::*;
use crate::response::*;
use crate::search::search_from_array;
use crate::span;

/// load時に呼ばれる関数
pub fn load(_path: &str) {}

/// unload時に呼ばれる関数
pub fn unload(_path: &str) {}

/// request GET Version時に呼ばれる関数
pub fn get_version(_path: &str, _request: &SaoriRequest, response: &mut SaoriResponse) {
    response.set_result(String::from(env!("CARGO_PKG_VERSION")));
}

/// request EXECUTE時に呼ばれる関数
/// メインの処理はここに記述する
pub fn execute(path: &str, request: &SaoriRequest, response: &mut SaoriResponse) {
    let args = request.argument();
    let mut path = PathBuf::from(path);
    if !path.is_dir() {
        path.pop();
    }

    let span_list_array = match span::load_spans(&path) {
        Ok(r) => r,
        Err(e) => {
            response.set_result(format!("Error: {}", e));
            return;
        }
    };

    let mut args_iter = args.iter();

    let (year, month, day) = match (args_iter.next(), args_iter.next(), args_iter.next()) {
        (Some(year_str), Some(month_str), Some(day_str)) => {
            match (
                year_str.parse::<i32>(),
                month_str.parse::<u32>(),
                day_str.parse::<u32>(),
            ) {
                (Ok(y), Ok(m), Ok(d)) => (y, m, d),
                (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => {
                    response.set_result(format!("Error: {}", e));
                    return;
                }
            }
        }
        _ => {
            response.set_result(
                "Error: arguments are not enough. arguments are required >= 3.".to_string(),
            );
            return;
        }
    };

    let date = match NaiveDate::from_ymd_opt(year, month, day) {
        Some(d) => d,
        None => {
            response.set_result("Error: target date is invalid.".to_string());
            return;
        }
    };

    let mode_str = args_iter.next().map_or("Gi*", |s| s.as_str());
    let mut should_search_future = true;
    if mode_str.contains('*') {
        should_search_future = true;
    } else if mode_str.contains('!') {
        should_search_future = false;
    }

    let mut is_kansuuji = false;
    if mode_str.contains('i') {
        is_kansuuji = false;
    } else if mode_str.contains('k') {
        is_kansuuji = true;
    }

    // TODO:
    // let mut taiinreki_mode = false;
    // if mode_str.contains('G') {
    //     taiinreki_mode = false;
    // } else if mode_str.contains('T') {
    //     taiinreki_mode = true;
    // }

    let selector: Vec<&str> = args_iter
        .next()
        .map(|s| s.as_str())
        .unwrap_or("")
        .split('_')
        .collect();

    let now = chrono::Local::now().date_naive();
    let search_target_date = if should_search_future && date > now {
        &now
    } else {
        &date
    };
    let span_list = search_from_array(&span_list_array, search_target_date, &selector);

    let (r_date, r_spans) = represent_by_gregorian(&span_list, &date, is_kansuuji);

    let result = r_date.first().unwrap_or(&"".to_string()).clone();
    let value_1 = r_date.join(",");
    let value_2 = r_spans.join(",");

    response.set_result(result);
    if !value_1.is_empty() {
        response.set_value(vec![value_1, value_2]);
    }
}
