//! Block flow: stacks blocks vertically, indents lists and quotes, lays out tables.

use iced::advanced::graphics::text::cosmic_text as ct;
use iced::{Color, Font, Rectangle};

use super::{Fill, Page};
use crate::model::{Align, Block, Item, Span};
use crate::theme::{Theme, bold};

/// Base look of a text block before per-span styling.
#[derive(Debug, Clone, Copy)]
pub(super) struct Ink {
    pub font: Font,
    pub size: f32,
    pub color: Color,
}

pub(super) struct Flow<'a> {
    pub theme: &'a Theme,
    pub fonts: &'a mut ct::FontSystem,
    pub page: Page,
    pub y: f32,
    pub line: usize,
}

/// Bullets come from the bundled mono face: it is the one family that has all three.
const BULLETS: [&str; 3] = ["•", "◦", "▪"];

pub(super) fn run(blocks: &[Block], theme: &Theme, fonts: &mut ct::FontSystem, width: f32) -> Page {
    let content = (width - 2.0 * theme.gutter).min(theme.measure).max(80.0);
    let x = (width - content) / 2.0;
    let mut flow = Flow { theme, fonts, page: Page::default(), y: theme.gutter, line: 0 };
    let ink = Ink { font: theme.body_font, size: theme.body_size, color: theme.ink };
    flow.blocks(blocks, x, content, ink, 0);
    Page { height: flow.y + theme.gutter * 2.0, ..flow.page }
}

impl Flow<'_> {
    fn blocks(&mut self, blocks: &[Block], x: f32, width: f32, ink: Ink, depth: usize) {
        for (i, block) in blocks.iter().enumerate() {
            if i > 0 {
                // A sublist hugs the item text it belongs to.
                let nested = depth > 0 && matches!(block, Block::List { .. });
                self.y += if nested { self.theme.item_gap } else { self.theme.block_gap };
            }
            self.block(block, x, width, ink, depth);
        }
    }

    fn block(&mut self, block: &Block, x: f32, width: f32, ink: Ink, depth: usize) {
        let theme = self.theme;
        match block {
            Block::Paragraph(spans) => self.text(spans, x, width, ink, Align::Left, ct::Wrap::WordOrGlyph),
            Block::Heading { level, spans, slug } => {
                let size = theme.body_size * theme.heading_scales[usize::from(*level).clamp(1, 6) - 1];
                if self.y > theme.gutter {
                    self.y += size * 0.5;
                }
                self.page.anchors.push((slug.clone(), self.y));
                let heading = Ink { font: bold(ink.font), size, ..ink };
                self.text(spans, x, width, heading, Align::Left, ct::Wrap::WordOrGlyph);
                if *level <= 2 {
                    self.y += 6.0;
                    self.fill(x, self.y, width, 1.0, theme.rule);
                }
            }
            Block::Code { text, .. } => {
                let top = self.y;
                let pad = theme.code_padding;
                let code =
                    Ink { font: theme.mono_font, size: theme.body_size * theme.mono_scale, color: theme.code_ink };
                let source = [plain(text.trim_end_matches('\n'))];
                self.y += pad;
                self.text(&source, x + pad, width - 2.0 * pad, code, Align::Left, ct::Wrap::Glyph);
                self.y += pad;
                self.fill(x, top, width, self.y - top, theme.code_background);
            }
            Block::Quote(inner) => {
                let top = self.y;
                let indent = theme.indent * 0.7;
                self.blocks(inner, x + indent, width - indent, Ink { color: theme.muted, ..ink }, depth);
                self.fill(x, top, 3.0, self.y - top, theme.rule);
            }
            Block::List { start, items } => {
                for (n, item) in items.iter().enumerate() {
                    if n > 0 {
                        self.y += theme.item_gap;
                    }
                    let ordinal = start.map(|first| first + n as u64);
                    self.item(item, ordinal, x, width, ink, depth);
                }
            }
            Block::Table { aligns, rows } => self.table(aligns, rows, x, width, ink),
            Block::Rule => {
                self.fill(x, self.y + theme.block_gap / 2.0, width, 1.0, theme.rule);
                self.y += theme.block_gap + 1.0;
            }
        }
    }

    fn item(&mut self, item: &Item, ordinal: Option<u64>, x: f32, width: f32, ink: Ink, depth: usize) {
        let theme = self.theme;
        let (top, line) = (self.y, self.line);
        let line_height = (ink.size * theme.line_ratio).round();
        let marker_ink = Ink { color: theme.muted, ..ink };
        match (item.checked, ordinal) {
            (Some(checked), _) => self.checkbox(x, top, line_height, ink.size, checked),
            (None, Some(n)) => {
                self.text(&[plain(&format!("{n}."))], x, theme.indent, marker_ink, Align::Left, ct::Wrap::None)
            }
            (None, None) => {
                let bullet = [plain(BULLETS[depth % BULLETS.len()])];
                let bullet_ink = Ink { font: theme.mono_font, ..marker_ink };
                self.text(&bullet, x, theme.indent, bullet_ink, Align::Left, ct::Wrap::None)
            }
        }
        // Rewind so the item's first line sits beside (and shares a line id with) the marker.
        let marker_bottom = self.y.max(top + line_height);
        (self.y, self.line) = (top, line);
        self.blocks(&item.blocks, x + theme.indent, width - theme.indent, ink, depth + 1);
        self.y = self.y.max(marker_bottom);
        self.line = self.line.max(line + 1);
    }

    /// Task boxes are drawn, not typed: most reading faces lack ☐ and ☑.
    fn checkbox(&mut self, x: f32, top: f32, line_height: f32, text_size: f32, checked: bool) {
        let theme = self.theme;
        let side = (text_size * 0.75).round();
        let (left, upper) = (x + 1.0, top + ((line_height - side) / 2.0).round());
        let stroke = 1.5;
        self.fill(left, upper, side, stroke, theme.muted);
        self.fill(left, upper + side - stroke, side, stroke, theme.muted);
        self.fill(left, upper, stroke, side, theme.muted);
        self.fill(left + side - stroke, upper, stroke, side, theme.muted);
        if checked {
            let inset = stroke + 2.0;
            self.fill(left + inset, upper + inset, side - 2.0 * inset, side - 2.0 * inset, theme.link);
        }
    }

    fn table(&mut self, aligns: &[Align], rows: &[Vec<Vec<Span>>], x: f32, width: f32, ink: Ink) {
        let theme = self.theme;
        let columns = rows.iter().map(Vec::len).max().unwrap_or(0).max(1);
        let column = width / columns as f32;
        let pad = theme.cell_padding;
        self.fill(x, self.y, width, 1.0, theme.rule);
        for (r, row) in rows.iter().enumerate() {
            let (top, line) = (self.y, self.line);
            let cell_ink = if r == 0 { Ink { font: bold(ink.font), ..ink } } else { ink };
            let mut bottom = top;
            for (c, cell) in row.iter().enumerate() {
                (self.y, self.line) = (top + pad, line);
                let align = aligns.get(c).copied().unwrap_or(Align::Left);
                let left = x + c as f32 * column + pad;
                self.text(cell, left, column - 2.0 * pad, cell_ink, align, ct::Wrap::WordOrGlyph);
                bottom = bottom.max(self.y + pad);
            }
            if r == 0 {
                self.fill(x, top, width, bottom - top, theme.code_background);
            }
            self.y = bottom;
            self.fill(x, self.y, width, 1.0, theme.rule);
        }
    }

    pub(super) fn fill(&mut self, x: f32, y: f32, width: f32, height: f32, color: Color) {
        self.page.fills.push(Fill { bounds: Rectangle { x, y, width, height }, color });
    }
}

fn plain(text: &str) -> Span {
    Span { text: text.to_string(), style: Default::default(), link: None }
}
