use iced::{
    alignment, executor, Color,
    widget::{button, column, container, row, scrollable, text, Space},
    Application, Command, Element, Length, Pixels, Settings, Theme as IcedTheme, Padding,
    window,
};

use crate::gui::theme::Theme as AppTheme;
use crate::gui::styles::{RoundedButtonStyle, RoundedContainerStyle, CustomScrollableStyle};

#[derive(Debug, Clone)]
pub enum Message {
    Close,
}

#[derive(Debug)]
pub struct ConflictDialog {
    package_names: Vec<String>,
    conflict_message: String,
    terminal_output: String,
    theme: AppTheme,
    border_radius: f32,
}

impl ConflictDialog {
    pub fn new(package_names: Vec<String>, conflict_message: String, terminal_output: String) -> Self {
        Self {
            package_names,
            conflict_message,
            terminal_output,
            theme: AppTheme::Dark,
            border_radius: 12.0,
        }
    }

    pub fn run_separate_window(package_names: Vec<String>, conflict_message: String, terminal_output: String) -> Result<(), iced::Error> {
        let dialog = Self::new(package_names, conflict_message, terminal_output);

        let mut window_settings = window::Settings::default();
        window_settings.size = iced::Size::new(800.0, 600.0);
        window_settings.min_size = Some(iced::Size::new(600.0, 400.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        <ConflictDialog as Application>::run(Settings {
            window: window_settings,
            flags: dialog,
            default_text_size: Pixels(14.0),
            antialiasing: true,
            id: None,
            fonts: Vec::new(),
            default_font: iced::Font::DEFAULT,
        })
    }
}

impl Application for ConflictDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        (flags, Command::none())
    }

    fn title(&self) -> String {
        if self.package_names.len() == 1 {
            format!("Cannot Remove {} - BirdNest", self.package_names[0])
        } else {
            format!("Cannot Remove {} Packages - BirdNest", self.package_names.len())
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Close => {
                iced::window::close(iced::window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let theme = self.theme;
        
        let title_text = if self.package_names.len() == 1 {
            format!("Cannot Remove {}", self.package_names[0])
        } else {
            format!("Cannot Remove {} Packages", self.package_names.len())
        };

        let package_list = if self.package_names.len() == 1 {
            format!("Package: {}", self.package_names[0])
        } else {
            format!("Packages:\n{}", self.package_names.iter().enumerate()
                .map(|(i, name)| format!("  {}. {}", i + 1, name))
                .collect::<Vec<_>>()
                .join("\n"))
        };
        
        // Parse the conflict message to extract the main reason
        let (main_reason, details) = parse_conflict_message(&self.conflict_message);

        container(
            column![
                text(&title_text)
                    .size(24)
                    .style(iced::theme::Text::Color(theme.danger())),
                Space::with_height(Length::Fixed(20.0)),
                text(&package_list)
                    .size(14)
                    .style(iced::theme::Text::Color(theme.text())),
                Space::with_height(Length::Fixed(20.0)),
                text("Why it can't be removed:")
                    .size(16)
                    .style(iced::theme::Text::Color(theme.danger())),
                Space::with_height(Length::Fixed(8.0)),
                container(
                    scrollable(
                        column![
                            text(&main_reason)
                                .size(14)
                                .style(iced::theme::Text::Color(theme.text())),
                            if !details.is_empty() {
                                column![
                                    Space::with_height(Length::Fixed(12.0)),
                                    text("Details:")
                                        .size(13)
                                        .style(iced::theme::Text::Color(theme.secondary_text())),
                                    Space::with_height(Length::Fixed(4.0)),
                                    text(&details)
                                        .size(12)
                                        .font(iced::Font::MONOSPACE)
                                        .style(iced::theme::Text::Color(theme.text())),
                                ]
                                .spacing(4)
                            } else {
                                column![].spacing(0)
                            },
                        ]
                        .spacing(8)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                        background_color: theme.surface(),
                        border_radius: self.border_radius,
                    })))
                    .height(Length::Fixed(200.0))
                )
                .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                    radius: self.border_radius,
                    background: Some(Color::from_rgb(0.3, 0.1, 0.1)), // Reddish background
                    elevation: 1.0,
                })))
                .width(Length::Fill)
                .padding(Padding::new(12.0)),
                if !self.terminal_output.is_empty() {
                    column![
                        Space::with_height(Length::Fixed(20.0)),
                        text("Terminal Output:")
                            .size(14)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_height(Length::Fixed(8.0)),
                        scrollable(
                            text(&self.terminal_output)
                                .size(11)
                                .font(iced::Font::MONOSPACE)
                                .style(iced::theme::Text::Color(theme.text()))
                        )
                        .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                            background_color: theme.surface(),
                            border_radius: self.border_radius,
                        })))
                        .height(Length::Fixed(150.0)),
                    ]
                    .spacing(0)
                } else {
                    column![].spacing(0)
                },
                Space::with_height(Length::Fill),
                row![
                    Space::with_width(Length::Fill),
                    button("Close")
                        .on_press(Message::Close)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: true,
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: Color::WHITE,
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                ]
                .spacing(10)
                .align_items(alignment::Alignment::Center),
            ]
            .spacing(15)
            .padding(Padding::new(30.0))
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: self.border_radius,
            background: Some(theme.background()),
            elevation: 0.0,
        })))
        .into()
    }

    fn theme(&self) -> IcedTheme {
        match self.theme {
            AppTheme::Light => IcedTheme::Light,
            AppTheme::Dark => IcedTheme::Dark,
        }
    }
}

// Parse conflict message to extract main reason and details
fn parse_conflict_message(conflict_msg: &str) -> (String, String) {
    // Try to extract the main reason (first line or before "Details:")
    let lines: Vec<&str> = conflict_msg.lines().collect();
    
    if lines.is_empty() {
        return ("Unknown conflict".to_string(), String::new());
    }
    
    // Look for "Details:" separator
    let mut main_reason_lines = Vec::new();
    let mut details_lines = Vec::new();
    let mut found_details = false;
    
    for line in &lines {
        if line.trim().to_lowercase().starts_with("details:") {
            found_details = true;
            continue;
        }
        
        if found_details {
            details_lines.push(*line);
        } else {
            main_reason_lines.push(*line);
        }
    }
    
    let main_reason = if main_reason_lines.is_empty() {
        // Fallback: use first few lines as main reason
        lines.iter().take(3).map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n")
    } else {
        main_reason_lines.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n")
    };
    
    let details = details_lines.iter().map(|s| s.trim()).filter(|s| !s.is_empty()).collect::<Vec<_>>().join("\n");
    
    // If main reason is too long, truncate and move rest to details
    if main_reason.len() > 200 {
        let truncated = main_reason.chars().take(200).collect::<String>();
        let rest = main_reason.chars().skip(200).collect::<String>();
        (truncated + "...", format!("{}\n{}", rest, details))
    } else {
        (main_reason, details)
    }
}

