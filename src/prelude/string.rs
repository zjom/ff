use crate::interop::FfResult;
use crate::interpreter::Value;
use crate::members;

// Data is the *last* parameter on every multi-arg function so calls compose
// naturally under `|>` (e.g. `s |> String.contains(needle)`).
members! {
    "String",
    len => |s: String| -> usize { s.chars().count() },
    upper => |s: String| -> String { s.to_uppercase() },
    lower => |s: String| -> String { s.to_lowercase() },
    trim => |s: String| -> String { s.trim().to_string() },
    trim_start => |s: String| -> String { s.trim_start().to_string() },
    trim_end => |s: String| -> String { s.trim_end().to_string() },
    contains => |needle: String, s: String| -> bool { s.contains(&needle) },
    starts_with => |prefix: String, s: String| -> bool { s.starts_with(&prefix) },
    ends_with => |suffix: String, s: String| -> bool { s.ends_with(&suffix) },
    replace => |from: String, to: String, s: String| -> String { s.replace(&from, &to) },
    split => |sep: String, s: String| -> Vec<String> {
        s.split(&sep).map(|p| p.to_string()).collect()
    },
    lines => |s: String| -> Vec<String> {
        s.lines().map(|l| l.to_string()).collect()
    },
    join => |sep: String, parts: Vec<String>| -> String { parts.join(&sep) },
    chars => |s: String| -> Vec<String> {
        s.chars().map(|c| c.to_string()).collect()
    },
    repeat => |n: usize, s: String| -> String { s.repeat(n) },
    reverse => |s: String| -> String { s.chars().rev().collect() },
    // Char-position slice; bounds are clamped to [0, len] and a reversed range
    // yields the empty string.
    slice => |start: i64, end: i64, s: String| -> String {
        let chars: Vec<char> = s.chars().collect();
        let n = chars.len() as i64;
        let start = start.clamp(0, n) as usize;
        let end = end.clamp(0, n) as usize;
        if start >= end { String::new() } else { chars[start..end].iter().collect() }
    },
    // Returns the char-index of the first occurrence, or `()` if not found.
    index_of => |needle: String, s: String| -> Option<usize> {
        s.find(&needle).map(|byte_pos| s[..byte_pos].chars().count())
    },
    parse_int => |s: String| -> FfResult<i64> {
        s.trim().parse::<i64>().map_err(|e| e.to_string()).into()
    },
    parse_float => |s: String| -> FfResult<f64> {
        s.trim().parse::<f64>().map_err(|e| e.to_string()).into()
    },
}
