use crate::opts::SortOrder;
use crate::Instantiations;
use regex::Regex;
use std::collections::HashMap as Map;
use std::io::{self, Write};

pub(crate) fn print(
    instantiations: Map<String, Instantiations>,
    sort_order: SortOrder,
    function_filter: Option<&Regex>,
) {
    let mut data = instantiations.into_iter().collect::<Vec<_>>();

    let mut total = Instantiations {
        copies: 0,
        total_lines: 0,
    };
    for row in &data {
        total.copies += row.1.copies;
        total.total_lines += row.1.total_lines;
    }

    match sort_order {
        SortOrder::Lines => {
            data.sort_by(|a, b| {
                let key_lo = (b.1.total_lines, b.1.copies, &a.0);
                let key_hi = (a.1.total_lines, a.1.copies, &b.0);
                key_lo.cmp(&key_hi)
            });
        }
        SortOrder::Copies => {
            data.sort_by(|a, b| {
                let key_lo = (b.1.copies, b.1.total_lines, &a.0);
                let key_hi = (a.1.copies, a.1.total_lines, &b.0);
                key_lo.cmp(&key_hi)
            });
        }
    }

    let lines_width = total.total_lines.to_string().len();
    let copies_width = total.copies.to_string().len();

    let stdout = io::stdout();
    let mut handle = stdout.lock();
    let _ = writeln!(
        handle,
        "  Lines{0:1$}           Copies{0:2$}          Function name",
        "", lines_width, copies_width,
    );
    let _ = writeln!(
        handle,
        "  -----{0:1$}           ------{0:2$}          -------------",
        "", lines_width, copies_width,
    );
    let _ = writeln!(
        handle,
        "  {0:1$}                {2:3$}                (TOTAL)",
        total.total_lines, lines_width, total.copies, copies_width,
    );
    let mut cumul_lines = 0;
    let mut cumul_copies = 0;
    let perc = |m, cumul_m: &mut _, n| {
        *cumul_m += m;
        format!(
            "({:3.1}%,{:5.1}%)",
            m as f64 / n as f64 * 100f64,
            *cumul_m as f64 / n as f64 * 100f64,
        )
    };
    for row in data {
        if function_filter.map_or(true, |ff| ff.is_match(&row.0)) {
            let _ = writeln!(
                handle,
                "  {0:1$} {2:<14} {3:4$} {5:<14} {6}",
                row.1.total_lines,
                lines_width,
                perc(row.1.total_lines, &mut cumul_lines, total.total_lines),
                row.1.copies,
                copies_width,
                perc(row.1.copies, &mut cumul_copies, total.copies),
                row.0,
            );
        }
    }
}
