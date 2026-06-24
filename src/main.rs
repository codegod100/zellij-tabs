use std::io::Write;
use unicode_width::UnicodeWidthStr;
use zellij_tile::prelude::*;

#[derive(Debug, Clone)]
struct TabSeg { tab_position: usize, width: usize, part: Vec<u8> }

#[derive(Default)]
struct State { tabs: Vec<TabInfo>, segs: Vec<TabSeg>, palette: Option<Styling>, perm_ok: bool }

register_plugin!(State);

fn rgb(c: PaletteColor) -> (u8, u8, u8) { match c { PaletteColor::Rgb(r) => r, _ => (128, 128, 128) } }

fn ansi(w: &mut Vec<u8>, fg: (u8, u8, u8), bg: (u8, u8, u8), bold: bool) {
    let _ = write!(w, "\x1b[0m");
    if bold { let _ = write!(w, "\x1b[1m"); }
    let _ = write!(w, "\x1b[38;2;{};{};{}m", fg.0, fg.1, fg.2);
    let _ = write!(w, "\x1b[48;2;{};{};{}m", bg.0, bg.1, bg.2);
}

fn build_tab(tab: &TabInfo, p: &Styling) -> TabSeg {
    let (fg, bg) = if tab.active {
        (rgb(p.ribbon_selected.base), rgb(p.ribbon_selected.background))
    } else {
        (rgb(p.ribbon_unselected.base), rgb(p.ribbon_unselected.background))
    };
    let sb = rgb(p.text_unselected.background);
    let close = (255u8, 80, 80);
    let lbl = format!(" {} ", tab.name);
    let mut v = Vec::new();
    ansi(&mut v, sb, bg, false); write!(&mut v, "").ok();
    ansi(&mut v, fg, bg, true);  write!(&mut v, "{}", lbl).ok();
    ansi(&mut v, close, bg, true); write!(&mut v, " ×").ok();
    ansi(&mut v, bg, sb, false); write!(&mut v, "").ok();
    TabSeg { tab_position: tab.position, width: tab.name.width() + 6, part: v }
}

fn tab_at(segs: &[TabSeg], col: usize) -> Option<&TabSeg> {
    let mut c = 0;
    for s in segs { if col >= c && col < c + s.width { return Some(s); } c += s.width; }
    None
}

fn close_hit(segs: &[TabSeg], col: usize) -> Option<usize> {
    let mut c = 0;
    for s in segs {
        if col >= c + s.width.saturating_sub(3) && col < c + s.width.saturating_sub(1) { return Some(s.tab_position); }
        c += s.width;
    }
    None
}

fn act(tabs: &[TabInfo]) -> usize { tabs.iter().position(|t| t.active).unwrap_or(0) }

fn fit(segs: &[TabSeg], cols: usize, act: usize) -> Vec<TabSeg> {
    if segs.iter().map(|s| s.width).sum::<usize>() <= cols { return segs.to_vec(); }
    let mut out = vec![segs[act].clone()];
    let mut used = segs[act].width;
    let (mut l, mut r) = (act.wrapping_sub(1), act + 1);
    loop {
        let (ld, rd) = (l >= segs.len(), r >= segs.len());
        if ld && rd { break; }
        if !ld && used + segs[l].width <= cols { out.insert(0, segs[l].clone()); used += segs[l].width; l = l.wrapping_sub(1); } else { l = segs.len(); }
        if !rd && used + segs[r].width <= cols { out.push(segs[r].clone()); used += segs[r].width; r += 1; } else { r = segs.len(); }
    }
    out
}

impl ZellijPlugin for State {
    fn load(&mut self, _c: std::collections::BTreeMap<String, String>) {
        set_selectable(true);
        request_permission(&[PermissionType::ReadApplicationState, PermissionType::ChangeApplicationState]);
        subscribe(&[EventType::TabUpdate, EventType::ModeUpdate, EventType::Mouse, EventType::PermissionRequestResult]);
    }

    fn update(&mut self, event: Event) -> bool {
        match event {
            Event::PermissionRequestResult(s) => { if s == PermissionStatus::Granted { self.perm_ok = true; } false }
            Event::ModeUpdate(m) => { self.palette = Some(m.style.colors); false }
            Event::TabUpdate(tabs) => { self.tabs = tabs; true }
            Event::Mouse(me) => {
                if !self.perm_ok { return false; }
                match me {
                    Mouse::LeftClick(_, col) => {
                        let col = col as usize;
                        // Close on ×, switch on name, fallback to close_focused_tab
                        if let Some(p) = close_hit(&self.segs, col) {
                            // Focus tab first, then close
                            if p != act(&self.tabs) { switch_tab_to((p + 1) as u32); }
                            close_focused_tab();
                        } else if let Some(s) = tab_at(&self.segs, col) {
                            if s.tab_position != act(&self.tabs) {
                                go_to_tab((s.tab_position + 1) as u32);
                            }
                        }
                        true
                    }
                    Mouse::RightClick(_, col) => {
                        let col = col as usize;
                        if let Some(s) = tab_at(&self.segs, col) {
                            if s.tab_position != act(&self.tabs) { go_to_tab((s.tab_position + 1) as u32); }
                            switch_to_input_mode(&InputMode::RenameTab);
                        }
                        true
                    }
                    Mouse::ScrollUp(_) => { go_to_next_tab(); true }
                    Mouse::ScrollDown(_) => { go_to_previous_tab(); true }
                    _ => false,
                }
            }
            _ => false,
        }
    }

    fn render(&mut self, _rows: usize, cols: usize) {
        if self.tabs.is_empty() {
            if !self.perm_ok { let _ = std::io::stdout().write_all(b"need perm"); }
            return;
        }
        let p = match &self.palette { Some(p) => p.clone(), None => return };
        let segs: Vec<TabSeg> = self.tabs.iter().map(|t| build_tab(t, &p)).collect();
        let segs = fit(&segs, cols, act(&self.tabs));
        let mut out = Vec::new();
        for s in &segs { out.extend_from_slice(&s.part); }
        let bg = rgb(p.text_unselected.background);
        let _ = write!(out, "\x1b[48;2;{};{};{}m\x1b[0K", bg.0, bg.1, bg.2);
        let _ = std::io::stdout().write_all(&out);
        self.segs = segs;
    }
}
