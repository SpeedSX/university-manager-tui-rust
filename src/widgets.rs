use ratatui::{
    layout::{Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph},
    Frame,
};

// Predefined list of majors for the student dropdown
pub const MAJORS: &[&str] = &[
    "Computer Science",
    "Mathematics",
    "Physics",
    "Chemistry",
    "Biology",
    "Engineering",
    "Economics",
    "Business",
    "Psychology",
    "Sociology",
    "History",
    "English",
    "Philosophy",
    "Political Science",
    "Art",
    "Music",
    "Medicine",
    "Law",
];

// Dropdown state for handling dropdown UI elements
pub struct DropdownState {
    pub is_open: bool,
    pub options: Vec<String>,
    pub list_state: ListState,
}

impl DropdownState {
    pub fn new(options: Vec<String>) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));
        Self {
            is_open: false,
            options,
            list_state,
        }
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn select_next(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i >= self.options.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn select_prev(&mut self) {
        let i = match self.list_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.options.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.list_state.select(Some(i));
    }

    pub fn selected_item(&self) -> Option<&String> {
        match self.list_state.selected() {
            Some(i) => self.options.get(i),
            None => None,
        }
    }

    pub fn set_options(&mut self, options: Vec<String>) {
        self.options = options;
        if self.options.is_empty() {
            self.list_state.select(None);
        } else {
            self.list_state.select(Some(0));
        }
    }

    pub fn select_by_value(&mut self, value: &str) {
        for (i, option) in self.options.iter().enumerate() {
            if option == value {
                self.list_state.select(Some(i));
                return;
            }
        }
        // If no match, select first item
        if !self.options.is_empty() {
            self.list_state.select(Some(0));
        }
    }
}

// Function to render the dropdown list
pub fn render_dropdown(f: &mut Frame, dropdown_state: &mut DropdownState, area: Rect) {
    // Calculate the position for the dropdown - right below the field
    let dropdown_area = Rect::new(
        area.x,
        area.y + 1, // Position right at the bottom edge of the field
        area.width,
        12.min(dropdown_state.options.len() as u16 + 2), // Height based on number of options with max of 12
    );
    
    // Clear the area to prevent visual artifacts
    f.render_widget(Clear, dropdown_area);
    
    // Create the items for the dropdown list
    let items: Vec<ListItem> = dropdown_state
        .options
        .iter()
        .map(|option| {
            ListItem::new(option.as_str())
                .style(Style::default().fg(Color::White))
        })
        .collect();
    
    // Create the list widget with highlighting similar to the screenshot
    let list = List::new(items)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .border_type(BorderType::Plain))
        .highlight_style(
            Style::default()
                .bg(Color::Blue)
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        );
    
    // Render the dropdown list with the current selection state
    f.render_stateful_widget(list, dropdown_area, &mut dropdown_state.list_state);
}