use anyhow::Result;
use chrono::Utc;
use crossterm::event::{MouseEvent, MouseEventKind};
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Layout, Margin, Position, Rect},
    style::{palette::tailwind, Modifier, Style, Stylize},
    text::Text,
    widgets::{
        Block, Cell, Paragraph, Row, Scrollbar, ScrollbarState, StatefulWidget, Table, TableState,
        Widget, Wrap,
    },
};
use reverse_geocoder::ReverseGeocoder;
use unicode_width::UnicodeWidthStr;

use crate::app::App;

use super::{satellites::SatellitesState, track_map::TrackMapState};

pub struct ObjectInformation<'a> {
    pub satellites_state: &'a SatellitesState,
    pub track_map_state: &'a TrackMapState,
}

pub struct ObjectInformationState {
    pub table_state: TableState,
    pub table_size: usize,
    pub area: Rect,
    geocoder: ReverseGeocoder,
}

impl Default for ObjectInformationState {
    fn default() -> Self {
        Self {
            table_state: Default::default(),
            table_size: Default::default(),
            area: Default::default(),
            geocoder: ReverseGeocoder::new(),
        }
    }
}

impl StatefulWidget for ObjectInformation<'_> {
    type State = ObjectInformationState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.area = area;

        let block = Block::bordered().title("Object information".blue());
        if let Some(index) = self.track_map_state.selected_object {
            let object = &self.satellites_state.objects[index];
            let object_state = object.predict(Utc::now()).unwrap();

            let result = state
                .geocoder
                .search((object_state.latitude(), object_state.longitude()));
            let city = result.record.name.clone();
            let country = isocountry::CountryCode::for_alpha2(&result.record.cc)
                .unwrap()
                .name();

            let items = [
                ("Name", object.name().clone()),
                ("COSPAR ID", object.cospar_id().clone()),
                ("NORAD ID", object.norad_id().to_string()),
                ("Longitude", format!("{:10.5}", object_state.longitude())),
                ("Latitude", format!("{:10.5}", object_state.latitude())),
                ("Altitude", format!("{:.5} km", object_state.altitude())),
                ("Speed", format!("{:.2} km/s", object_state.speed())),
                ("Location", format!("{}, {}", city, country)),
                ("Epoch", object.epoch().to_string()),
                (
                    "Period",
                    format!(
                        "{} hr {} min {} ({:.2} min)",
                        object.orbital_period().num_hours(),
                        object.orbital_period().num_minutes() % 60,
                        object.orbital_period().num_seconds() % 60,
                        object.orbital_period().num_seconds() as f64 / 60.0
                    ),
                ),
                ("Inc", object.inclination().to_string()),
                ("R.A.", object.right_ascension().to_string()),
                ("Ecc", object.eccentricity().to_string()),
                ("M. anomaly", object.mean_anomaly().to_string()),
                ("M. motion", object.mean_motion().to_string()),
                ("Rev. #", object.revolution_number().to_string()),
            ];
            state.table_size = items.len();

            let inner_area = area.inner(Margin::new(1, 1));

            let (max_key_width, _max_value_width) = items
                .iter()
                .map(|(key, value)| (key.width() as u16, value.width() as u16))
                .fold((0, 0), |acc, (key_width, value_width)| {
                    (acc.0.max(key_width), acc.1.max(value_width))
                });

            let widths = [Constraint::Max(max_key_width), Constraint::Fill(1)];
            let [_left, right] = Layout::horizontal(widths)
                .areas(inner_area)
                .map(|rect| rect.width);
            let right = right.saturating_sub(1);

            let rows = items.iter().enumerate().map(|(i, (key, value))| {
                let color = match i % 2 {
                    0 => tailwind::SLATE.c950,
                    _ => tailwind::SLATE.c900,
                };
                let value = if value.width() as u16 > right {
                    value[..right as usize - 3.min(right as usize)].to_string() + "..."
                } else {
                    value.to_string()
                };
                Row::new([
                    Cell::from(Text::from(key.bold())),
                    Cell::from(Text::from(value)),
                ])
                .style(Style::new().bg(color))
                .height(1)
            });

            let table = Table::new(rows, widths)
                .block(block)
                .row_highlight_style(Style::default().add_modifier(Modifier::REVERSED));
            StatefulWidget::render(table, area, buf, &mut state.table_state);

            let inner_area = area.inner(Margin::new(0, 1));
            let mut scrollbar_state =
                ScrollbarState::new(items.len().saturating_sub(inner_area.height as usize))
                    .position(state.table_state.offset());
            Scrollbar::default().render(area, buf, &mut scrollbar_state);
        } else {
            let paragraph = Paragraph::new("No object selected".dark_gray())
                .block(block)
                .centered()
                .wrap(Wrap { trim: true });

            paragraph.render(area, buf);
        }
    }
}

pub fn handle_mouse_events(event: MouseEvent, app: &mut App) -> Result<()> {
    let inner_area = app.object_information_state.area.inner(Margin::new(1, 1));
    if !inner_area.contains(Position::new(event.column, event.row)) {
        app.object_information_state.table_state.select(None);
        return Ok(());
    }

    match event.kind {
        MouseEventKind::ScrollDown => {
            let max_offset = app
                .object_information_state
                .table_size
                .saturating_sub(inner_area.height as usize);
            *app.object_information_state.table_state.offset_mut() =
                (*app.object_information_state.table_state.offset_mut() + 1).min(max_offset);
        }
        MouseEventKind::ScrollUp => {
            *app.object_information_state.table_state.offset_mut() = app
                .object_information_state
                .table_state
                .offset()
                .saturating_sub(1);
        }
        _ => {}
    }
    let row =
        (event.row - inner_area.y) as usize + app.object_information_state.table_state.offset();
    let index = if row < app.object_information_state.table_size {
        Some(row)
    } else {
        None
    };
    app.object_information_state.table_state.select(index);

    Ok(())
}

#[allow(dead_code)]
fn format_longitude(longitude: f64) -> String {
    if longitude >= 0.0 {
        format!("{:.5}°E", longitude)
    } else {
        format!("{:.5}°W", longitude.abs())
    }
}

#[allow(dead_code)]
fn format_latitude(latitude: f64) -> String {
    if latitude >= 0.0 {
        format!("{:.5}°N", latitude)
    } else {
        format!("{:.5}°S", latitude.abs())
    }
}
