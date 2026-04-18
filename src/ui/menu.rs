// Menu system for Connect Four UI
// Provides menu bars, context menus, and navigation menus

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
    Frame,
};
use std::collections::HashMap;
use crate::ui::{Theme, PaletteColor};
use crate::ui::popup::{PopupPosition, PopupSize};

// ============================================================================
// Menu Item
// ============================================================================

/// Individual menu item
#[derive(Debug, Clone)]
pub struct MenuItem {
    /// Item ID
    pub id: String,
    /// Item label
    pub label: String,
    /// Keyboard shortcut
    pub shortcut: Option<String>,
    /// Is this item a separator
    pub separator: bool,
    /// Is this item a submenu
    pub submenu: bool,
    /// Is this item checked (for toggle items)
    pub checked: bool,
    /// Is this item enabled
    pub enabled: bool,
    /// Item icon/emoji
    pub icon: Option<String>,
}

impl MenuItem {
    pub fn new(id: String, label: String) -> Self {
        Self {
            id,
            label,
            shortcut: None,
            separator: false,
            submenu: false,
            checked: false,
            enabled: true,
            icon: None,
        }
    }

    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            shortcut: None,
            separator: true,
            submenu: false,
            checked: false,
            enabled: false,
            icon: None,
        }
    }

    pub fn with_shortcut(mut self, shortcut: String) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    pub fn with_icon(mut self, icon: String) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_submenu(mut self, submenu: bool) -> Self {
        self.submenu = submenu;
        self
    }

    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

// ============================================================================
// Menu
// ============================================================================

/// A collection of menu items
#[derive(Debug, Clone)]
pub struct Menu {
    /// Menu ID
    pub id: String,
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Menu title (for menu bar)
    pub title: String,
}

impl Menu {
    pub fn new(id: String, title: String) -> Self {
        Self {
            id,
            items: Vec::new(),
            title,
        }
    }

    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self
    }

    pub fn add_item(&mut self, item: MenuItem) {
        self.items.push(item);
    }

    pub fn add_separator(&mut self) {
        self.items.push(MenuItem::separator());
    }

    pub fn get_item(&self, id: &str) -> Option<&MenuItem> {
        self.items.iter().find(|i| i.id == id)
    }

    pub fn get_item_mut(&mut self, id: &str) -> Option<&mut MenuItem> {
        self.items.iter_mut().find(|i| i.id == id)
    }

    pub fn find_item_by_index(&self, index: usize) -> Option<&MenuItem> {
        self.items.get(index).filter(|i| !i.separator)
    }
}

// ============================================================================
// Menu Bar
// ============================================================================

/// Horizontal menu bar at top of screen
pub struct MenuBar {
    /// Menus in the bar
    menus: Vec<Menu>,
    /// Currently open menu index
    open_menu: Option<usize>,
    /// Theme
    theme: Theme,
    /// Show keyboard shortcuts
    show_shortcuts: bool,
}

impl MenuBar {
    pub fn new(theme: Theme) -> Self {
        Self {
            menus: Vec::new(),
            open_menu: None,
            theme,
            show_shortcuts: true,
        }
    }

    pub fn add_menu(&mut self, menu: Menu) {
        self.menus.push(menu);
    }

    pub fn remove_menu(&mut self, id: &str) -> bool {
        if let Some(pos) = self.menus.iter().position(|m| m.id == id) {
            self.menus.remove(pos);
            if self.open_menu == Some(pos) {
                self.open_menu = None;
            } else if self.open_menu > Some(pos) {
                self.open_menu = self.open_menu.map(|i| i - 1);
            }
            true
        } else {
            false
        }
    }

    pub fn get_menu(&self, id: &str) -> Option<&Menu> {
        self.menus.iter().find(|m| m.id == id)
    }

    pub fn get_menu_mut(&mut self, id: &str) -> Option<&mut Menu> {
        self.menus.iter_mut().find(|m| m.id == id)
    }

    pub fn open_menu(&mut self, index: usize) {
        if index < self.menus.len() {
            self.open_menu = Some(index);
        }
    }

    pub fn close_menu(&mut self) {
        self.open_menu = None;
    }

    pub fn is_menu_open(&self) -> bool {
        self.open_menu.is_some()
    }

    pub fn open_menu_id(&self) -> Option<&str> {
        self.open_menu.and_then(|i| self.menus.get(i).map(|m| m.id.as_str()))
    }

    pub fn select_next_menu(&mut self) {
        if let Some(current) = self.open_menu {
            self.open_menu = Some((current + 1) % self.menus.len());
        }
    }

    pub fn select_previous_menu(&mut self) {
        if let Some(current) = self.open_menu {
            self.open_menu = Some(if current == 0 {
                self.menus.len() - 1
            } else {
                current - 1
            });
        }
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        // Render menu bar
        self.render_menu_bar(frame, area);

        // Render open dropdown menu
        if let Some(menu_index) = self.open_menu {
            self.render_dropdown_menu(frame, area, menu_index);
        }
    }

    fn render_menu_bar(&self, frame: &mut Frame, area: Rect) {
        let menu_width = area.width / self.menus.len() as u16;
        let mut x = area.x;

        for menu in &self.menus {
            let label = format!(" {} ", menu.title);
            let menu_area = Rect {
                x,
                y: area.y,
                width: label.len() as u16,
                height: 1,
            };

            let is_selected = self.open_menu.and_then(|i| self.menus.get(i))
                .map(|m| m.id == menu.id)
                .unwrap_or(false);

            let style = if is_selected {
                self.theme.get_style(PaletteColor::Primary)
                    .add_modifier(Modifier::REVERSED)
            } else {
                self.theme.get_style(PaletteColor::Text)
            };

            let paragraph = Paragraph::new(label)
                .style(style)
                .alignment(Alignment::Left);
            paragraph.render(menu_area, frame.buffer_mut());

            x += menu_width;
        }
    }

    fn render_dropdown_menu(&self, frame: &mut Frame, bar_area: Rect, menu_index: usize) {
        if let Some(menu) = self.menus.get(menu_index) {
            let menu_x = bar_area.x + (menu_index as u16 * (bar_area.width / self.menus.len() as u16));

            let max_item_width = menu.items.iter()
                .map(|item| item.label.len() + item.shortcut.as_ref().map(|s| s.len() + 3).unwrap_or(0) + 4)
                .max()
                .unwrap_or(20) as u16;

            let menu_height = menu.items.iter()
                .filter(|i| !i.separator)
                .count() as u16 + menu.items.iter().filter(|i| i.separator).count() as u16;

            let dropdown_area = Rect {
                x: menu_x,
                y: bar_area.bottom(),
                width: max_item_width.min(bar_area.width - menu_x + 1),
                height: menu_height.min(bar_area.height - bar_area.bottom()),
            };

            // Clear area
            Clear.render(dropdown_area, frame.buffer_mut());

            // Render border
            Block::default()
                .borders(Borders::ALL)
                .border_style(self.theme.get_style(PaletteColor::Border))
                .render(dropdown_area, frame.buffer_mut());

            // Render menu items
            let mut y = dropdown_area.top() + 1;
            for item in &menu.items {
                if item.separator {
                    // Draw separator line
                    for x in dropdown_area.left() + 1..dropdown_area.right() {
                        frame.buffer_mut().get_mut(x, y)
                            .set_symbol("─")
                            .set_style(self.theme.get_style(PaletteColor::Border));
                    }
                    y += 1;
                } else {
                    let item_area = Rect {
                        x: dropdown_area.left() + 1,
                        y,
                        width: dropdown_area.width - 2,
                        height: 1,
                    };

                    let mut text = String::new();

                    // Checkmark for checked items
                    if item.checked {
                        text.push_str("[✓] ");
                    } else {
                        text.push_str("    ");
                    }

                    // Icon
                    if let Some(icon) = &item.icon {
                        text.push_str(icon);
                        text.push(' ');
                    }

                    // Label
                    text.push_str(&item.label);

                    // Shortcut
                    if let Some(shortcut) = &item.shortcut {
                        text.push_str(&format!("{:>width$}", format!("  {}", shortcut), width = max_item_width as usize - text.len()));
                    }

                    let style = if item.enabled {
                        self.theme.get_style(PaletteColor::Text)
                    } else {
                        self.theme.get_style(PaletteColor::Muted)
                    };

                    let paragraph = Paragraph::new(text)
                        .style(style)
                        .alignment(Alignment::Left);
                    paragraph.render(item_area, frame.buffer_mut());

                    y += 1;
                }
            }
        }
    }
}

impl Default for MenuBar {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}

// ============================================================================
// Context Menu
// ============================================================================

/// Right-click context menu
pub struct ContextMenu {
    /// Menu items
    items: Vec<MenuItem>,
    /// Position
    position: (u16, u16),
    /// Theme
    theme: Theme,
    /// Is visible
    visible: bool,
}

impl ContextMenu {
    pub fn new(theme: Theme) -> Self {
        Self {
            items: Vec::new(),
            position: (0, 0),
            theme,
            visible: false,
        }
    }

    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items = items;
        self
    }

    pub fn show_at(&mut self, x: u16, y: u16) {
        self.position = (x, y);
        self.visible = true;
    }

    pub fn hide(&mut self) {
        self.visible = false;
    }

    pub fn is_visible(&self) -> bool {
        self.visible
    }

    pub fn render(&self, frame: &mut Frame, parent_area: Rect) {
        if !self.visible {
            return;
        }

        let max_item_width = self.items.iter()
            .map(|item| item.label.len() + item.shortcut.as_ref().map(|s| s.len() + 3).unwrap_or(0) + 4)
            .max()
            .unwrap_or(20) as u16;

        let menu_height = self.items.iter()
            .filter(|i| !i.separator)
            .count() as u16 + self.items.iter().filter(|i| i.separator).count() as u16;

        // Calculate menu position (keep within bounds)
        let mut x = self.position.0;
        let mut y = self.position.1;

        if x + max_item_width > parent_area.right() {
            x = parent_area.right() - max_item_width - 1;
        }

        if y + menu_height > parent_area.bottom() {
            y = parent_area.bottom() - menu_height - 1;
        }

        let menu_area = Rect {
            x,
            y,
            width: max_item_width,
            height: menu_height,
        };

        // Clear area
        Clear.render(menu_area, frame.buffer_mut());

        // Render border
        Block::default()
            .borders(Borders::ALL)
            .border_style(self.theme.get_style(PaletteColor::Border))
            .render(menu_area, frame.buffer_mut());

        // Render items
        let mut item_y = menu_area.top() + 1;
        for item in &self.items {
            if item.separator {
                for sx in menu_area.left() + 1..menu_area.right() {
                    frame.buffer_mut().get_mut(sx, item_y)
                        .set_symbol("─")
                        .set_style(self.theme.get_style(PaletteColor::Border));
                }
                item_y += 1;
            } else {
                let item_area = Rect {
                    x: menu_area.left() + 1,
                    y: item_y,
                    width: menu_area.width - 2,
                    height: 1,
                };

                let mut text = String::new();

                if let Some(icon) = &item.icon {
                    text.push_str(icon);
                    text.push(' ');
                }

                text.push_str(&item.label);

                if let Some(shortcut) = &item.shortcut {
                    text.push_str(&format!("{:>width$}", format!("  {}", shortcut), width = max_item_width as usize - text.len()));
                }

                let style = if item.enabled {
                    self.theme.get_style(PaletteColor::Text)
                } else {
                    self.theme.get_style(PaletteColor::Muted)
                };

                let paragraph = Paragraph::new(text)
                    .style(style)
                    .alignment(Alignment::Left);
                paragraph.render(item_area, frame.buffer_mut());

                item_y += 1;
            }
        }
    }
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}

// ============================================================================
// Navigation Menu
// ============================================================================

/// Vertical navigation menu (sidebar)
pub struct NavigationMenu {
    /// Menu items
    items: Vec<MenuItem>,
    /// Selected item index
    selected: Option<usize>,
    /// Theme
    theme: Theme,
    /// Show icons
    show_icons: bool,
}

impl NavigationMenu {
    pub fn new(theme: Theme) -> Self {
        Self {
            items: Vec::new(),
            selected: None,
            theme,
            show_icons: true,
        }
    }

    pub fn with_items(mut self, items: Vec<MenuItem>) -> Self {
        self.selected = items.iter().position(|i| i.enabled && !i.separator);
        self.items = items;
        self
    }

    pub fn with_icons(mut self, show: bool) -> Self {
        self.show_icons = show;
        self
    }

    pub fn selected_item(&self) -> Option<&MenuItem> {
        self.selected.and_then(|i| self.items.get(i))
    }

    pub fn select_next(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let start = self.selected.unwrap_or(0);
        for offset in 1..=self.items.len() {
            let index = (start + offset) % self.items.len();
            if let Some(item) = self.items.get(index) {
                if item.enabled && !item.separator {
                    self.selected = Some(index);
                    return;
                }
            }
        }
    }

    pub fn select_previous(&mut self) {
        if self.items.is_empty() {
            return;
        }

        let start = self.selected.unwrap_or(0);
        for offset in 1..=self.items.len() {
            let index = if start >= offset {
                start - offset
            } else {
                self.items.len() - (offset - start)
            };
            if let Some(item) = self.items.get(index) {
                if item.enabled && !item.separator {
                    self.selected = Some(index);
                    return;
                }
            }
        }
    }

    pub fn set_selected(&mut self, id: &str) -> bool {
        if let Some(index) = self.items.iter().position(|i| i.id == id) {
            if let Some(item) = self.items.get(index) {
                if item.enabled && !item.separator {
                    self.selected = Some(index);
                    return true;
                }
            }
        }
        false
    }

    pub fn render(&self, frame: &mut Frame, area: Rect) {
        for (index, item) in self.items.iter().enumerate() {
            let item_area = Rect {
                x: area.x,
                y: area.y + index as u16,
                width: area.width,
                height: 1,
            };

            if item.separator {
                // Draw separator
                for x in area.left()..area.right() {
                    frame.buffer_mut().get_mut(x, item_area.y)
                        .set_symbol("─")
                        .set_style(self.theme.get_style(PaletteColor::Muted));
                }
            } else {
                let is_selected = self.selected == Some(index);
                let mut text = String::new();

                if self.show_icons {
                    if let Some(icon) = &item.icon {
                        text.push_str(icon);
                        text.push(' ');
                    } else {
                        text.push_str("  ");
                    }
                }

                text.push_str(&item.label);

                if self.show_shortcuts {
                    if let Some(shortcut) = &item.shortcut {
                        text.push_str(&format!("{:>width$}", format!("  {}", shortcut), width = area.width as usize - text.len()));
                    }
                }

                let style = if is_selected {
                    self.theme.get_style(PaletteColor::Primary)
                        .add_modifier(Modifier::REVERSED | Modifier::BOLD)
                } else if item.enabled {
                    self.theme.get_style(PaletteColor::Text)
                } else {
                    self.theme.get_style(PaletteColor::Muted)
                };

                let paragraph = Paragraph::new(text)
                    .style(style)
                    .alignment(Alignment::Left);
                paragraph.render(item_area, frame.buffer_mut());
            }
        }
    }
}

impl Default for NavigationMenu {
    fn default() -> Self {
        Self::new(Theme::default())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_menu_item() {
        let item = MenuItem::new("test".to_string(), "Test Item".to_string())
            .with_shortcut("Ctrl+T".to_string())
            .with_icon("📝".to_string())
            .with_checked(true);

        assert_eq!(item.id, "test");
        assert_eq!(item.label, "Test Item");
        assert!(item.checked);
        assert_eq!(item.shortcut, Some("Ctrl+T".to_string()));
    }

    #[test]
    fn test_menu_separator() {
        let separator = MenuItem::separator();
        assert!(separator.separator);
        assert!(!separator.enabled);
    }

    #[test]
    fn test_menu() {
        let mut menu = Menu::new("file".to_string(), "File".to_string());
        menu.add_item(MenuItem::new("new".to_string(), "New".to_string()));
        menu.add_separator();
        menu.add_item(MenuItem::new("open".to_string(), "Open".to_string()));

        assert_eq!(menu.items.len(), 3);
        assert!(menu.get_item("new").is_some());
        assert!(menu.get_item("invalid").is_none());
    }

    #[test]
    fn test_menu_bar() {
        let mut bar = MenuBar::new(Theme::default());
        bar.add_menu(Menu::new("file".to_string(), "File".to_string()));
        bar.add_menu(Menu::new("edit".to_string(), "Edit".to_string()));

        assert_eq!(bar.menus.len(), 2);
        assert!(!bar.is_menu_open());

        bar.open_menu(0);
        assert!(bar.is_menu_open());
        assert_eq!(bar.open_menu_id(), Some("file"));

        bar.select_next_menu();
        assert_eq!(bar.open_menu_id(), Some("edit"));

        bar.close_menu();
        assert!(!bar.is_menu_open());
    }

    #[test]
    fn test_context_menu() {
        let mut menu = ContextMenu::new(Theme::default())
            .with_items(vec![
                MenuItem::new("copy".to_string(), "Copy".to_string()),
                MenuItem::new("paste".to_string(), "Paste".to_string()),
            ]);

        assert!(!menu.is_visible());

        menu.show_at(10, 20);
        assert!(menu.is_visible());
        assert_eq!(menu.position, (10, 20));

        menu.hide();
        assert!(!menu.is_visible());
    }

    #[test]
    fn test_navigation_menu() {
        let mut menu = NavigationMenu::new(Theme::default())
            .with_items(vec![
                MenuItem::new("home".to_string(), "🏠 Home".to_string()),
                MenuItem::new("settings".to_string(), "⚙️ Settings".to_string()),
                MenuItem::new("about".to_string(), "ℹ️ About".to_string()),
            ]);

        assert_eq!(menu.selected, Some(0));

        menu.select_next();
        assert_eq!(menu.selected, Some(1));

        menu.select_previous();
        assert_eq!(menu.selected, Some(0));

        assert!(menu.set_selected("settings"));
        assert_eq!(menu.selected, Some(1));
    }
}
