use std::{fs::read_to_string, io};
use ratatui::{
    backend::Backend, buffer::Buffer, crossterm::
        event::{self, Event, KeyCode, KeyEvent}, 
        layout::{Constraint, Layout, Rect}, style::{
        Styled,
        Color, Modifier, Style, Stylize,
    }, text::Line, widgets::{Block, Borders, Tabs, List, ListItem, ListState, Paragraph, StatefulWidget, Widget}, Terminal
};
use std::collections::HashMap;

use crate::test::Test;

pub struct App {
    // results is a list of [ "test_name": ["patch": "res", ...], ..]
    results: HashMap<String, HashMap<u8, i64>>,
    should_exit: bool,
    selected: ListState,
    patch_selected: Option<usize>,
    path: String,
    scroll: u16
}

impl App {
    pub fn from_results(res:  &mut Vec<Test>, p: &String) -> Self {
        return App{should_exit: false, selected: ListState::default(), path: p.clone(), patch_selected: None, scroll: 0,
            results: {
                let mut foo: HashMap<String, HashMap<u8, i64>> = HashMap::new();
                for x in res.iter() {
                    match foo.get_mut(&x.test) {
                        Some(entry) => {entry.insert(x.patch, x.result);},
                        None => {foo.insert(x.test.clone(), HashMap::from([(x.patch, x.result)]));}
                    }
                }
               foo
            }
        }
    }

    pub fn run(& mut self, mut term: Terminal<impl Backend>) -> io::Result<()> {
        while !self.should_exit {
            term.draw(|f| f.render_widget(&mut *self, f.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        return Ok(())
    }

    fn handle_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char('q') | KeyCode::Esc => self.should_exit = true,
            KeyCode::Up => {
                self.scroll = 0;
                self.patch_selected = Some(0);
                match self.selected.selected() {
                    Some(i) => self.selected.select(Some(i.max(1) - 1)),
                    None => self.selected.select(Some(0))
                }
            },
            KeyCode::Down => {
                self.scroll = 0;
                self.patch_selected = Some(0);
                match self.selected.selected() {
                    Some(i) => self.selected.select(Some(i.min(self.results.keys().len() - 2) + 1)),
                    None => self.selected.select(Some(0))
                }
            },
            KeyCode::Left => {
                self.scroll = 0;
                match self.selected.selected() {
                    Some(_) => {
                        match self.patch_selected {
                            Some(p) => self.patch_selected = Some(p.max(1) - 1),
                            None => self.patch_selected = Some(0)
                        }
                    }
                    None => {
                        self.selected.select(Some(0));
                        self.patch_selected = Some(0)
                    }
                }
            },
            KeyCode::Right => {
                self.scroll = 0;
                match self.selected.selected() {
                    Some(i) => {
                        match self.patch_selected {
                            Some(p) => self.patch_selected = Some((p+1).min(self.results.get(self.results.keys().nth(i).unwrap()).unwrap().keys().len() - 1)),
                            None => self.patch_selected = Some(0)
                        }
                    }
                    None => {
                        self.selected.select(Some(0));
                        self.patch_selected = Some(0);
                    }
                }
            },
            KeyCode::Enter => {
                self.scroll += 1;
            },
            KeyCode::Backspace => {
                self.scroll = self.scroll.max(1) - 1;
            } 
            _ => {}

        }
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title(Line::raw(self.path.clone()).centered()).borders(Borders::ALL);
        let items : Vec<ListItem> = self.results.keys().map(
            |k| {
                let totals = self.results.get(k).unwrap().values().len();
                let pass = self.results.get(k).unwrap().values().filter(|r| **r == 0).collect::<Vec<_>>().len();

                if pass == 0 {
                    ListItem::from(format!("{k}  {pass}/{totals}")).on_light_red().black()
                } else if pass == totals {
                    ListItem::from(format!("{k}  {pass}/{totals}")).on_light_green().black()
                }
                else {
                    ListItem::from(format!("{k}  {pass}/{totals}")).on_light_yellow().black()
                }
            }
        ).collect();

        let mut selected_style = Style::new().add_modifier(Modifier::BOLD).on_white().green();
        match self.selected.selected() {
            Some(i) => {
                let old_bg = Styled::style(items.get(i).unwrap()).bg.unwrap();
                selected_style = selected_style.fg(old_bg);
            },
            _ => {}
        }
        let list = List::new(items).highlight_style(selected_style).block(block);
        StatefulWidget::render(list, area, buf, &mut self.selected);
    }

    fn render_selected_item(&mut self, area: Rect, buf: &mut Buffer) {
        let selected_key_op = self.results.keys().nth(self.selected.selected().unwrap_or(0));
        
        if selected_key_op.is_none() || self.patch_selected.is_none() {
            let block = Block::new().borders(Borders::ALL);
            Widget::render(block, area, buf);
            return
        }
        let vertical = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]);
        let [header_area, inner_area] = vertical.areas(area);

        let horizontal = Layout::horizontal([Constraint::Min(0), Constraint::Fill(1)]);
        let [title_area, tabs_area] = horizontal.areas(header_area);
    
        let selected_key = selected_key_op.unwrap();

        let mut patches : Vec<_> = self.results.get(selected_key).unwrap().keys().map(|x| x.to_string()).collect::<Vec<String>>();
        patches.sort();
        let style ;
        let patch_selected = patches[self.patch_selected.unwrap()].parse::<u8>().unwrap();
        if self.results[selected_key][&patch_selected] == 0 {
            style = Color::Green;
        } else {
            style = Color::Red;
        }
        let tabs = Tabs::new(patches).highlight_style(style).select(self.patch_selected.unwrap());

        tabs.render(tabs_area, buf);
        let part_file = format!("{}/{}/summary", patch_selected, selected_key);
        let file = format!("{}/{}", self.path, part_file);
        Line::raw(&part_file).bold().render(title_area, buf);

        // now render the text file
        let output = read_to_string(&file).unwrap_or(format!("Cannot read {}", file));
        let lines = output.lines().count();
        let block = Block::new().borders(Borders::ALL);
        let text = Paragraph::new(output).scroll((self.scroll % lines as u16, 0)).block(block);
        text.render(inner_area,buf);
    }
}


impl Widget for &mut App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let [main_area, footer_area] = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .areas(area);

        let [list_area, item_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(main_area);

        //App::render_header(header_area, buf);
        //App::render_footer(footer_area, buf);
        self.render_list(list_area, buf);
        self.render_selected_item(item_area, buf);
        Block::new().borders(Borders::TOP).title("Use: arrows, ENTER/BACKSPC, q").render(footer_area, buf);
    }
}