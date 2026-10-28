//! Rich-text shaping: one cosmic-text buffer per text block, kept for painting,
//! plus the fragments used for hit-testing and selection.

use std::sync::Arc;

use iced::advanced::graphics::text::{cosmic_text as ct, to_attributes, to_color};
use iced::{Color, Font, Point, Rectangle};

use super::flow::{Flow, Ink};
use super::{Fragment, Joint, LaidText};
use crate::model::{Align, Span};
use crate::theme::{bold, italic};

/// cosmic-text picks one font per word, so a font change mid-word ("(`code`)")
/// shapes the whole word in the first font. A zero-width space splits the word.
const SEPARATOR: &str = "\u{200B}";
const SEPARATOR_ID: usize = usize::MAX;

fn needs_separator(spans: &[Span], looks: &[(Font, Color)], i: usize) -> bool {
    i > 0
        && looks[i - 1].0.family != looks[i].0.family
        && !spans[i - 1].text.ends_with(char::is_whitespace)
        && !spans[i].text.starts_with(char::is_whitespace)
}

impl Flow<'_> {
    fn look(&self, span: &Span, ink: Ink) -> (Font, Color) {
        let theme = self.theme;
        let font = if span.style.code { theme.mono_font } else { ink.font };
        let font = if span.style.strong { bold(font) } else { font };
        let font = if span.style.emphasis { italic(font) } else { font };
        let color = match (&span.link, span.style.code) {
            (Some(_), _) => theme.link,
            (None, true) => theme.code_ink,
            (None, false) => ink.color,
        };
        (font, color)
    }

    /// Shapes and wraps one block of rich text at the current y, then advances y.
    pub(super) fn text(&mut self, spans: &[Span], x: f32, width: f32, ink: Ink, align: Align, wrap: ct::Wrap) {
        if spans.is_empty() {
            return;
        }
        let line_height = (ink.size * self.theme.line_ratio).round();
        let looks: Vec<(Font, Color)> = spans.iter().map(|span| self.look(span, ink)).collect();
        let mut buffer = ct::Buffer::new(self.fonts, ct::Metrics::new(ink.size, line_height));
        buffer.set_wrap(self.fonts, wrap);
        buffer.set_size(self.fonts, Some(width.max(1.0)), None);
        let rich = spans.iter().zip(&looks).enumerate().flat_map(|(i, (span, (font, color)))| {
            let attrs = to_attributes(*font).color(to_color(*color));
            let separator =
                needs_separator(spans, &looks, i).then(|| (SEPARATOR, attrs.clone().metadata(SEPARATOR_ID)));
            separator.into_iter().chain([(span.text.as_str(), attrs.metadata(i))])
        });
        let align = match align {
            Align::Left => ct::Align::Left,
            Align::Center => ct::Align::Center,
            Align::Right => ct::Align::Right,
        };
        buffer.set_rich_text(self.fonts, rich, &to_attributes(ink.font), ct::Shaping::Advanced, Some(align));
        buffer.shape_until_scroll(self.fonts, false);

        let mut height = 0.0;
        let mut previous_line = None;
        for run in buffer.layout_runs() {
            let mut joint = match previous_line {
                Some(line) if line == run.line_i => Joint::Wrap,
                _ => Joint::Break,
            };
            previous_line = Some(run.line_i);
            let top = self.y + run.line_top;
            let baseline = self.y + run.line_y;
            for glyphs in run.glyphs.chunk_by(|a, b| a.metadata == b.metadata) {
                if glyphs[0].metadata == SEPARATOR_ID {
                    continue;
                }
                let span = &spans[glyphs[0].metadata];
                let (font, color) = looks[glyphs[0].metadata];
                let start = glyphs.iter().map(|g| g.start).min().unwrap_or(0);
                let end = glyphs.iter().map(|g| g.end).max().unwrap_or(start);
                let left = glyphs.iter().map(|g| g.x).fold(f32::MAX, f32::min);
                let right = glyphs.iter().map(|g| g.x + g.w).fold(f32::MIN, f32::max);
                let bounds = Rectangle { x: x + left, y: top, width: right - left, height: run.line_height };
                let mut stops: Vec<(f32, usize)> =
                    glyphs.iter().map(|g| (g.x - left, g.start - start)).chain([(right - left, end - start)]).collect();
                stops.sort_by(|a, b| a.0.total_cmp(&b.0));
                if span.style.code && span.link.is_none() {
                    let background = self.theme.code_background;
                    self.fill(bounds.x - 2.0, top + 3.0, bounds.width + 4.0, run.line_height - 6.0, background);
                }
                if span.link.is_some() {
                    self.fill(bounds.x, baseline + 3.0, bounds.width, 1.0, Color { a: 0.45, ..color });
                }
                if span.style.strike {
                    self.fill(bounds.x, baseline - ink.size * 0.3, bounds.width, 1.0, color);
                }
                self.page.fragments.push(Fragment {
                    bounds,
                    text: run.text[start..end].to_string(),
                    font,
                    size: ink.size,
                    color,
                    link: span.link.clone(),
                    stops,
                    line: self.line,
                    joint,
                });
                joint = Joint::Flush;
            }
            self.line += 1;
            height = run.line_top + run.line_height;
        }
        let bounds = Rectangle { x, y: self.y, width, height };
        self.page.texts.push(LaidText { buffer: Arc::new(buffer), position: Point::new(x, self.y), bounds });
        self.y += height;
    }
}
