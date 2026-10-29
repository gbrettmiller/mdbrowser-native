//! Hit-testing, carets and selection over a laid-out page.

use iced::{Point, Rectangle};

use super::{Caret, Fragment, Joint, Page};

fn ordered(a: Caret, b: Caret) -> (Caret, Caret) {
    if a <= b { (a, b) } else { (b, a) }
}

impl Fragment {
    fn x_of(&self, byte: usize) -> f32 {
        self.stops.iter().find(|stop| stop.1 >= byte).map_or(self.bounds.width, |stop| stop.0)
    }

    fn byte_at(&self, x: f32) -> usize {
        let local = x - self.bounds.x;
        self.stops.iter().min_by(|a, b| (a.0 - local).abs().total_cmp(&(b.0 - local).abs())).map_or(0, |stop| stop.1)
    }
}

impl Page {
    pub fn link_at(&self, point: Point) -> Option<&str> {
        self.fragments.iter().find(|f| f.bounds.contains(point)).and_then(|f| f.link.as_deref())
    }

    /// The image a clickable note at `point` loads.
    pub fn action_at(&self, point: Point) -> Option<&str> {
        self.fragments.iter().find(|f| f.bounds.contains(point)).and_then(|f| f.action.as_deref())
    }

    pub fn caret_at(&self, point: Point) -> Option<Caret> {
        let on_row = self
            .fragments
            .iter()
            .enumerate()
            .filter(|(_, f)| point.y >= f.bounds.y && point.y < f.bounds.y + f.bounds.height);
        let distance = |f: &Fragment| (f.bounds.x - point.x).max(point.x - (f.bounds.x + f.bounds.width)).max(0.0);
        if let Some((i, f)) = on_row.min_by(|a, b| distance(a.1).total_cmp(&distance(b.1))) {
            return Some(Caret { fragment: i, byte: f.byte_at(point.x) });
        }
        // Between rows: snap to the start of the next fragment, or the very end.
        match self.fragments.iter().position(|f| f.bounds.y > point.y) {
            Some(i) => Some(Caret { fragment: i, byte: 0 }),
            None => self.fragments.last().map(|f| Caret { fragment: self.fragments.len() - 1, byte: f.text.len() }),
        }
    }

    /// The caret span covering the whole page, for select-all.
    pub fn everything(&self) -> Option<(Caret, Caret)> {
        let last = self.fragments.len().checked_sub(1)?;
        Some((Caret { fragment: 0, byte: 0 }, Caret { fragment: last, byte: self.fragments[last].text.len() }))
    }

    /// Each selected piece: (fragment, start byte, end byte).
    fn pieces(&self, a: Caret, b: Caret) -> impl Iterator<Item = (&Fragment, usize, usize)> {
        let (a, b) = ordered(a, b);
        (a.fragment..=b.fragment).filter_map(move |i| {
            let f = self.fragments.get(i)?;
            let start = if i == a.fragment { a.byte } else { 0 };
            let end = if i == b.fragment { b.byte } else { f.text.len() };
            (start < end).then_some((f, start, end))
        })
    }

    pub fn highlight(&self, a: Caret, b: Caret) -> Vec<Rectangle> {
        self.pieces(a, b)
            .fold(Vec::<(usize, Rectangle)>::new(), |mut rects, (f, start, end)| {
                let left = f.bounds.x + f.x_of(start);
                let rect = Rectangle { x: left, width: f.bounds.x + f.x_of(end) - left, ..f.bounds };
                match rects.last_mut() {
                    Some((line, last)) if *line == f.line => *last = last.union(&rect),
                    _ => rects.push((f.line, rect)),
                }
                rects
            })
            .into_iter()
            .map(|(_, rect)| rect)
            .collect()
    }

    pub fn selected_text(&self, a: Caret, b: Caret) -> String {
        let mut previous: Option<usize> = None;
        self.pieces(a, b).filter(|(f, _, _)| !f.note).fold(String::new(), |text, (f, start, end)| {
            let separator = match (previous, f.joint) {
                (None, _) | (Some(_), Joint::Flush) => "",
                (Some(line), _) if line == f.line => " ",
                (Some(_), Joint::Wrap) => " ",
                (Some(_), Joint::Break) => "\n",
            };
            previous = Some(f.line);
            let separator = if separator == " " && text.ends_with(char::is_whitespace) { "" } else { separator };
            text + separator + &f.text[start..end]
        })
    }
}
