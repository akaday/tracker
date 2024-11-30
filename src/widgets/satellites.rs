use std::time::Instant;

use anyhow::Result;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Margin, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, List, ListItem, ListState, Scrollbar, ScrollbarState, StatefulWidget},
};
use strum::IntoEnumIterator;

use crate::{app::App, object::Object, satellite::Satellite};

#[derive(Default)]
pub struct Satellites;

pub struct SatellitesState {
    pub objects: Vec<Object>,

    pub items: Vec<Item>,
    pub list_state: ListState,

    pub area: Rect,

    pub last_object_update: Instant,
}

impl SatellitesState {
    pub fn update_objects(&mut self) {
        let mut objects = Vec::new();
        for item in &self.items {
            if !item.selected {
                continue;
            }
            for elements in item.satellite.get_elements() {
                objects.push(Object::from_elements(elements));
            }
        }
        self.objects = objects;
    }
}

impl Default for SatellitesState {
    fn default() -> Self {
        Self {
            objects: Vec::new(),
            items: Satellite::iter().map(Item::from).collect(),
            list_state: Default::default(),
            area: Default::default(),
            last_object_update: Instant::now(),
        }
    }
}

impl StatefulWidget for Satellites {
    type State = SatellitesState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let items = state.items.iter().map(|item| {
            let style = if item.selected {
                Style::default().fg(Color::White)
            } else {
                Style::default()
            };
            let text: String = if item.selected {
                format!("✓ {}", item.satellite)
            } else {
                format!("☐ {}", item.satellite)
            };
            ListItem::new(Text::styled(text, style))
        });

        let list = List::new(items)
            .block(Block::bordered().title("Satellites".blue()))
            .highlight_style(Style::default().add_modifier(Modifier::REVERSED));

        list.render(area, buf, &mut state.list_state);

        let inner_area = area.inner(Margin::new(0, 1));
        let mut scrollbar_state =
            ScrollbarState::new(state.items.len().saturating_sub(inner_area.height as usize))
                .position(state.list_state.offset());
        Scrollbar::default().render(area, buf, &mut scrollbar_state);
    }
}

pub struct Item {
    pub satellite: Satellite,
    selected: bool,
}

impl From<Satellite> for Item {
    fn from(satellite: Satellite) -> Self {
        Self {
            satellite,
            selected: false,
        }
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.satellites_state.area.inner(Margin::new(1, 1));
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.satellites_state.list_state.select(None);
        return Ok(());
    }

    match event.kind {
        MouseEventKind::Down(MouseButton::Left) => {
            if let Some(index) = app.satellites_state.list_state.selected() {
                app.satellites_state.items[index].selected =
                    !app.satellites_state.items[index].selected;
                app.track_map_state.selected_object = None;
                app.satellites_state.update_objects();
            }
        }
        MouseEventKind::ScrollDown => {
            let max_offset = app
                .satellites_state
                .items
                .len()
                .saturating_sub(inner_area.height as usize);
            *app.satellites_state.list_state.offset_mut() =
                (app.satellites_state.list_state.offset() + 1).min(max_offset);
        }
        MouseEventKind::ScrollUp => {
            *app.satellites_state.list_state.offset_mut() =
                app.satellites_state.list_state.offset().saturating_sub(1);
        }
        _ => {}
    }
    let row = (event.row - inner_area.y) as usize + app.satellites_state.list_state.offset();
    let index = if row < app.satellites_state.items.len() {
        Some(row)
    } else {
        None
    };
    app.satellites_state.list_state.select(index);

    Ok(())
}
