use std::ops::Range;

use egui::{Color32, Response, RichText, Ui, Widget};

use crate::git::{Header, Line};

pub struct LineNumbersWidget {
    longest_line: usize,
    lines: Vec<Line>,
    headers: Vec<Header>,
    range: Range<usize>,
}

impl LineNumbersWidget {
    pub fn new(
        longest_line: usize,
        lines: Vec<Line>,
        headers: Vec<Header>,
        range: Range<usize>,
    ) -> LineNumbersWidget {
        LineNumbersWidget {
            longest_line,
            lines,
            headers,
            range,
        }
    }
}

impl Widget for LineNumbersWidget {
    fn ui(self, ui: &mut Ui) -> Response {
        puffin::profile_function!("LineNumbersWidget");

        let mut content = "".to_owned();

        let Range { start, end } = self.range;
        let end = std::cmp::min(end, self.lines.len());

        for line in &self.lines[start..end] {
            for header in &self.headers {
                if header.line == line.new_lineno.unwrap_or(0)
                    && line.origin != '+'
                    && line.origin != '-'
                {
                    content.push_str(" \n");
                }
            }
            let mut line_no = match line.origin {
                '+' => line.new_lineno.unwrap_or(0).to_string(),
                '-' => line.old_lineno.unwrap_or(0).to_string(),
                _ => line.new_lineno.unwrap_or(0).to_string(),
            };

            while line_no.len() != self.longest_line {
                line_no = format!(" {}", line_no);
            }

            content.push_str(format!("{}\n", line_no).as_str());
        }
        ui.vertical(|ui| {
            ui.add_space(3.0);
            ui.label(RichText::new(content).monospace().color(Color32::GRAY))
        })
        .response
    }
}
