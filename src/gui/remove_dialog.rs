use iced::{
    alignment, executor, Color,
    widget::{button, column, container, row, scrollable, text, Space},
    Application, Command, Element, Length, Pixels, Settings, Theme as IcedTheme, Padding,
    window,
};
use tokio::process::Command as TokioCommand;

use crate::gui::theme::Theme as AppTheme;
use crate::gui::styles::{RoundedButtonStyle, RoundedContainerStyle, CustomScrollableStyle};

#[derive(Debug, Clone)]
pub enum Message {
    LoadPackageInfo,
    PackageInfoLoaded(Vec<PackageDetail>),
    RemovePackages,
    ConfirmRemove,
    #[allow(dead_code)]
    RemovalProgress(String),
    TerminalOutput(String),
    RemovalComplete,
    RemovalError(String),
    ConflictDetected(String),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct PackageDetail {
    pub name: String,
    pub version: String,
    pub description: String,
    pub size: String,
    pub is_flatpak: bool,
}

#[derive(Debug)]
pub struct RemoveDialog {
    pub package_names: Vec<String>,
    pub package_info: Vec<PackageDetail>,
    pub is_loading: bool,
    pub is_removing: bool,
    pub is_complete: bool,
    pub show_confirmation: bool,
    pub removal_progress: String,
    pub terminal_output: String,
    pub conflict_message: Option<String>,
    pub theme: AppTheme,
    pub border_radius: f32,
    pub is_flatpak: bool,
}

impl RemoveDialog {
    pub fn new(package_names: Vec<String>, is_flatpak: bool) -> Self {
        Self {
            package_names,
            package_info: Vec::new(),
            is_loading: true,
            is_removing: false,
            is_complete: false,
            show_confirmation: false,
            removal_progress: String::new(),
            terminal_output: String::new(),
            conflict_message: None,
            theme: AppTheme::Dark,
            border_radius: 12.0,
            is_flatpak,
        }
    }

    #[allow(dead_code)]
    pub fn run_separate_window(package_names: Vec<String>) -> Result<(), iced::Error> {
        Self::run_separate_window_with_flatpak_flag(package_names, false)
    }

    pub fn run_separate_window_with_flatpak_flag(package_names: Vec<String>, is_flatpak: bool) -> Result<(), iced::Error> {
        let dialog = Self::new(package_names, is_flatpak);

        let mut window_settings = window::Settings::default();
        window_settings.size = iced::Size::new(750.0, 800.0);
        window_settings.min_size = Some(iced::Size::new(600.0, 500.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        <RemoveDialog as Application>::run(Settings {
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

impl Application for RemoveDialog {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = executor::Default;
    type Flags = Self;

    fn new(flags: Self) -> (Self, Command<Message>) {
        let mut dialog = flags;
        let cmd = dialog.update(Message::LoadPackageInfo);
        (dialog, cmd)
    }

    fn title(&self) -> String {
        if !self.package_info.is_empty() {
            if self.package_info.len() == 1 {
                format!("Remove {} - BirdNest", self.package_info[0].name)
            } else {
                format!("Remove {} Packages - BirdNest", self.package_info.len())
            }
        } else {
            "Remove Package - BirdNest".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPackageInfo => {
                self.is_loading = true;
                let package_names = self.package_names.clone();
                let is_flatpak = self.is_flatpak;
                Command::perform(load_package_info(package_names, is_flatpak), |result| {
                    match result {
                        Ok(infos) => Message::PackageInfoLoaded(infos),
                        Err(e) => Message::RemovalError(e),
                    }
                })
            }
            Message::PackageInfoLoaded(infos) => {
                self.is_loading = false;
                self.package_info = infos;
                Command::none()
            }
            Message::RemovePackages => {
                // Show confirmation dialog first
                self.show_confirmation = true;
                Command::none()
            }
            Message::ConfirmRemove => {
                // User confirmed, proceed with removal
                eprintln!("[DEBUG] ConfirmRemove: User confirmed removal");
                self.show_confirmation = false;
                self.is_removing = true;
                self.removal_progress = "Preparing removal...".to_string();
                let package_names = self.package_names.clone();
                let is_flatpak = self.package_info.first().map(|p| p.is_flatpak).unwrap_or(false);
                // Store is_flatpak in self for use in the result handler
                self.is_flatpak = is_flatpak;
                
                eprintln!("[DEBUG] ConfirmRemove: Packages to remove: {:?}", package_names);
                eprintln!("[DEBUG] ConfirmRemove: Is flatpak: {}", is_flatpak);
                
                // Show the command that will be executed (without -y flag)
                let cmd_preview = if is_flatpak {
                    format!("flatpak uninstall {}\n", package_names.join(" "))
                } else {
                    format!("pkexec apt-get remove {}\n", package_names.join(" "))
                };
                self.terminal_output = format!("$ {}\n", cmd_preview.trim());
                eprintln!("[DEBUG] ConfirmRemove: Command preview: {}", cmd_preview.trim());
                
                Command::perform(remove_packages(package_names, is_flatpak), move |result| {
                    eprintln!("[DEBUG] ConfirmRemove: Removal command completed");
                    match result {
                        Ok((_progress, output)) => {
                            eprintln!("[DEBUG] ConfirmRemove: Removal succeeded, output length: {}", output.len());
                            // Always show terminal output first, completion detection will handle the rest
                            Message::TerminalOutput(output)
                        },
                        Err(e) => {
                            eprintln!("[DEBUG] ConfirmRemove: Removal failed with error: {}", e);
                            // Check if this is a conflict error
                            if e.starts_with("CONFLICT_DETECTED:") {
                                let conflict_msg = e.strip_prefix("CONFLICT_DETECTED:").unwrap_or(&e).to_string();
                                Message::ConflictDetected(conflict_msg)
                            } else {
                                Message::RemovalError(e.to_string())
                            }
                        },
                    }
                })
            }
            Message::TerminalOutput(output) => {
                eprintln!("[DEBUG] TerminalOutput: Received output, length: {}", output.len());
                eprintln!("[DEBUG] TerminalOutput: Output (first 500 chars): {}", &output.chars().take(500).collect::<String>());
                
                // Append terminal output first
                if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&output);
                
                // Check for conflicts in the output (after appending so we have full context)
                let full_output = self.terminal_output.clone();
                let conflict = detect_conflicts(&full_output);
                if let Some(conflict_msg) = conflict {
                    eprintln!("[DEBUG] TerminalOutput: Conflict detected: {}", conflict_msg);
                    return Command::perform(async {}, move |_| Message::ConflictDetected(conflict_msg));
                }
                
                // Update progress text based on output
                let output_lower = output.to_lowercase();
                if output_lower.contains("removing") || output_lower.contains("purging") {
                    self.removal_progress = "Removing packages...".to_string();
                    eprintln!("[DEBUG] TerminalOutput: Detected 'removing' or 'purging'");
                } else if output_lower.contains("reading") {
                    self.removal_progress = "Reading package lists...".to_string();
                    eprintln!("[DEBUG] TerminalOutput: Detected 'reading'");
                } else if output_lower.contains("building") {
                    self.removal_progress = "Building dependency tree...".to_string();
                    eprintln!("[DEBUG] TerminalOutput: Detected 'building'");
                } else if output_lower.contains("complete") || output_lower.contains("done") {
                    self.removal_progress = "Removal complete!".to_string();
                    eprintln!("[DEBUG] TerminalOutput: Detected 'complete' or 'done'");
                }
                
                // Check if removal is complete
                let is_complete = output_lower.contains("complete") ||
                   output_lower.contains("removed") ||
                   output_lower.contains("uninstalling") || // Flatpak uninstall output
                   output_lower.contains("done") ||
                   output_lower.contains("success") ||
                   output_lower.contains("finished") ||
                   output_lower.contains("0 upgraded, 0 newly installed");
                
                eprintln!("[DEBUG] TerminalOutput: Is complete check: {}", is_complete);
                
                if is_complete {
                    eprintln!("[DEBUG] TerminalOutput: Marking removal as complete");
                    Command::perform(async {}, |_| Message::RemovalComplete)
                } else {
                    Command::none()
                }
            }
            Message::ConflictDetected(conflict_msg) => {
                eprintln!("[DEBUG] ConflictDetected: {}", conflict_msg);
                self.is_removing = false;
                self.conflict_message = Some(conflict_msg.clone());
                
                // Launch conflict dialog as separate window
                let package_names = self.package_names.clone();
                let terminal_output = self.terminal_output.clone();
                let conflict_msg_clone = conflict_msg.clone();
                
                eprintln!("[DEBUG] ConflictDetected: Launching conflict dialog with {} packages", package_names.len());
                eprintln!("[DEBUG] ConflictDetected: Message length: {}, Output length: {}", conflict_msg_clone.len(), terminal_output.len());
                
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("conflict-dialog");
                        // Add packages as positional arguments
                        for pkg in &package_names {
                            cmd.arg(pkg);
                        }
                        // Add message and output as flags
                        // Escape the message and output properly for command line
                        cmd.arg("--message");
                        cmd.arg(&conflict_msg_clone);
                        cmd.arg("--output");
                        cmd.arg(&terminal_output);
                        
                        eprintln!("[DEBUG] ConflictDetected: Spawning conflict dialog process");
                        match cmd.spawn() {
                            Ok(_) => {
                                eprintln!("[DEBUG] ConflictDetected: Conflict dialog spawned successfully");
                            }
                            Err(e) => {
                                eprintln!("[DEBUG] ConflictDetected: Failed to spawn conflict dialog: {}", e);
                            }
                        }
                    },
                    |_| Message::Cancel, // Close the remove dialog after showing conflict
                )
            }
            Message::RemovalProgress(progress) => {
                self.removal_progress = progress;
                Command::none()
            }
            Message::RemovalComplete => {
                self.is_removing = false;
                self.is_complete = true;
                self.removal_progress = "Removal completed successfully!".to_string();
                if !self.terminal_output.contains("completed successfully") && !self.terminal_output.contains("Removal completed") {
                    if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                        self.terminal_output.push('\n');
                    }
                    self.terminal_output.push_str("✓ Removal completed successfully!");
                }
                Command::none()
            }
            Message::RemovalError(msg) => {
                eprintln!("[DEBUG] RemovalError: Error received: {}", msg);
                self.is_removing = false;
                if !self.terminal_output.is_empty() && !self.terminal_output.ends_with('\n') {
                    self.terminal_output.push('\n');
                }
                self.terminal_output.push_str(&format!("Error: {}\n", msg));
                Command::none()
            }
            Message::Cancel => {
                if self.show_confirmation {
                    // Just close the confirmation dialog, don't close the window
                    self.show_confirmation = false;
                    Command::none()
                } else {
                    iced::window::close(iced::window::Id::MAIN)
                }
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let theme = self.theme;
        
        if self.is_loading {
            container(
                column![
                    text("Loading package information...")
                        .size(18)
                        .style(iced::theme::Text::Color(theme.text())),
                    Space::with_height(Length::Fixed(20.0)),
                ]
                .spacing(15)
                .align_items(alignment::Alignment::Center)
                .padding(Padding::new(30.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                radius: self.border_radius,
                background: Some(theme.surface()),
                elevation: 1.5,
            })))
            .into()
        } else if !self.package_info.is_empty() {
            self.view_package_info()
        } else {
            container(
                text("Failed to load package information")
                    .size(18)
                    .style(iced::theme::Text::Color(theme.danger()))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .padding(Padding::new(30.0))
            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                radius: self.border_radius,
                background: Some(theme.surface()),
                elevation: 1.5,
            })))
            .into()
        }
    }

    fn theme(&self) -> IcedTheme {
        match self.theme {
            AppTheme::Light => IcedTheme::Light,
            AppTheme::Dark => IcedTheme::Dark,
        }
    }
}

impl RemoveDialog {
    fn view_package_info(&self) -> Element<Message> {
        let theme = self.theme;
        let needs_sudo = !self.package_info.first().map(|p| p.is_flatpak).unwrap_or(false);
        
        let title_text = if self.package_info.len() == 1 {
            format!("Remove {}", self.package_info[0].name)
        } else {
            format!("Remove {} Packages", self.package_info.len())
        };

        // Show confirmation dialog if needed
        if self.show_confirmation {
            let confirmation_text = if self.package_info.len() == 1 {
                format!("Are you sure you want to remove {}?", self.package_info[0].name)
            } else {
                format!("Are you sure you want to remove these {} packages?", self.package_info.len())
            };
            
            return container(
                column![
                    text(&title_text)
                        .size(24)
                        .style(iced::theme::Text::Color(theme.danger())),
                    Space::with_height(Length::Fixed(30.0)),
                    text(&confirmation_text)
                        .size(18)
                        .style(iced::theme::Text::Color(theme.text())),
                    Space::with_height(Length::Fixed(40.0)),
                    row![
                        button("No")
                            .on_press(Message::Cancel)
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: false,
                                radius: self.border_radius,
                                primary_color: theme.primary(),
                                text_color: theme.text(),
                                background_color: theme.background(),
                            })))
                            .padding(Padding::new(14.0)),
                        Space::with_width(Length::Fixed(20.0)),
                        button("Yes")
                            .on_press(Message::ConfirmRemove)
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                                radius: self.border_radius,
                                primary_color: theme.danger(),
                                text_color: Color::WHITE,
                                background_color: theme.background(),
                            })))
                            .padding(Padding::new(14.0)),
                    ]
                    .align_items(alignment::Alignment::Center),
                ]
                .spacing(0)
                .align_items(alignment::Alignment::Center)
                .padding(Padding::new(40.0))
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                radius: self.border_radius,
                background: Some(theme.surface()),
                elevation: 1.5,
            })))
            .into();
        }
        
        let buttons = if self.is_complete {
            row![
                button("Exit")
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: self.border_radius,
                        primary_color: theme.danger(),
                        text_color: Color::WHITE,
                        background_color: theme.background(),
                    })))
                    .padding(Padding::new(14.0)),
            ]
            .spacing(10)
            .align_items(alignment::Alignment::Center)
        } else {
            row![
                button("Cancel")
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: false,
                        radius: self.border_radius,
                        primary_color: theme.primary(),
                        text_color: theme.text(),
                        background_color: theme.background(),
                    })))
                    .padding(Padding::new(14.0)),
                Space::with_width(Length::Fill),
                {
                    if self.is_removing {
                        button("Removing...")
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                                radius: self.border_radius,
                                primary_color: theme.danger(),
                                text_color: Color::WHITE,
                                background_color: theme.background(),
                            })))
                            .padding(Padding::new(14.0))
                    } else {
                        button("Remove")
                            .on_press(Message::RemovePackages)
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                                radius: self.border_radius,
                                primary_color: theme.danger(),
                                text_color: Color::WHITE,
                                background_color: theme.background(),
                            })))
                            .padding(Padding::new(14.0))
                    }
                },
            ]
            .spacing(10)
            .align_items(alignment::Alignment::Center)
        };

        let content = if self.package_info.len() == 1 {
            let detail = &self.package_info[0];
            column![
                text(&title_text)
                    .size(24)
                    .style(iced::theme::Text::Color(theme.danger())),
                Space::with_height(Length::Fixed(20.0)),
                row![
                    text("Version:")
                        .size(14)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&detail.version)
                        .size(14)
                        .style(iced::theme::Text::Color(theme.text())),
                ]
                .spacing(8),
                Space::with_height(Length::Fixed(4.0)),
                row![
                    text("Size:")
                        .size(14)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    text(&detail.size)
                        .size(14)
                        .style(iced::theme::Text::Color(theme.text())),
                ]
                .spacing(8),
                Space::with_height(Length::Fixed(12.0)),
                text("Description:")
                    .size(14)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(4.0)),
                scrollable(
                    text(&detail.description)
                        .size(14)
                        .style(iced::theme::Text::Color(theme.text()))
                )
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                    background_color: theme.surface(),
                    border_radius: self.border_radius,
                })))
                .height(Length::Fixed(200.0)),
            ]
            .spacing(0)
        } else {
            // Multiple packages
            let mut package_list = String::new();
            for (i, detail) in self.package_info.iter().enumerate() {
                let desc = if detail.description.len() > 80 {
                    format!("{}...", &detail.description[..80])
                } else {
                    detail.description.clone()
                };
                package_list.push_str(&format!("{}. {} (v{})\n   {}\n\n", i + 1, detail.name, detail.version, desc));
            }
            
            column![
                text(&title_text)
                    .size(24)
                    .style(iced::theme::Text::Color(theme.danger())),
                Space::with_height(Length::Fixed(20.0)),
                text("Packages to remove:")
                    .size(14)
                    .style(iced::theme::Text::Color(theme.secondary_text())),
                Space::with_height(Length::Fixed(4.0)),
                scrollable(
                    text(&package_list)
                        .size(14)
                        .style(iced::theme::Text::Color(theme.text()))
                )
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                    background_color: theme.surface(),
                    border_radius: self.border_radius,
                })))
                .height(Length::Fixed(300.0)),
            ]
            .spacing(0)
        };

        // Conflict message section (shown prominently if conflict detected)
        let conflict_section = if let Some(ref conflict_msg) = self.conflict_message {
            column![
                Space::with_height(Length::Fixed(20.0)),
                container(
                    column![
                        text("⚠️ Package Removal Conflict")
                            .size(18)
                            .style(iced::theme::Text::Color(theme.danger())),
                        Space::with_height(Length::Fixed(12.0)),
                        scrollable(
                            text(conflict_msg)
                                .size(13)
                                .style(iced::theme::Text::Color(theme.text()))
                        )
                        .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                            background_color: theme.surface(),
                            border_radius: self.border_radius,
                        })))
                        .height(Length::Fixed(200.0)),
                    ]
                    .spacing(10)
                    .padding(Padding::new(16.0))
                )
                .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                    radius: self.border_radius,
                    background: Some(Color::from_rgb(0.3, 0.1, 0.1)), // Reddish background for conflicts
                    elevation: 1.0,
                })))
                .width(Length::Fill),
            ]
            .spacing(0)
        } else {
            column![].spacing(0)
        };

        // Terminal output section (shown during removal or after completion)
        let terminal_section = if self.is_removing || (self.is_complete && !self.terminal_output.is_empty()) {
            column![
                Space::with_height(Length::Fixed(20.0)),
                text("Output:")
                    .size(14)
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(8.0)),
                scrollable(
                    text(&self.terminal_output)
                        .size(12)
                        .font(iced::Font::MONOSPACE)
                        .style(iced::theme::Text::Color(theme.text()))
                )
                .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                    background_color: theme.surface(),
                    border_radius: self.border_radius,
                })))
                .height(Length::Fixed(300.0)),
            ]
            .spacing(0)
        } else {
            column![].spacing(0)
        };

        let progress_section = if !self.removal_progress.is_empty() && !self.is_removing && !self.is_complete {
            column![
                Space::with_height(Length::Fixed(20.0)),
                text(&self.removal_progress)
                    .size(14)
                    .style(iced::theme::Text::Color(if self.is_complete {
                        Color::from_rgb(0.0, 1.0, 0.0)
                    } else if self.removal_progress.contains("Error") || self.removal_progress.contains("error") {
                        theme.danger()
                    } else {
                        theme.text()
                    })),
            ]
            .spacing(0)
        } else {
            column![].spacing(0)
        };

        container(
            column![
                scrollable(
                    column![
                        content,
                        if needs_sudo && !self.is_removing && !self.is_complete {
                            column![
                                Space::with_height(Length::Fixed(12.0)),
                                text("Administrator privileges will be requested")
                                    .size(12)
                                    .style(iced::theme::Text::Color(Color::from_rgb(1.0, 0.8, 0.0))),
                            ]
                            .spacing(0)
                        } else {
                            column![].spacing(0)
                        },
                        progress_section,
                        conflict_section,
                        terminal_section,
                    ]
                    .spacing(15)
                    .padding(Padding::new(20.0))
                )
                .height(Length::Fill),
                container(buttons)
                    .width(Length::Fill)
                    .padding(Padding::new(20.0))
                    .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                        radius: self.border_radius,
                        background: Some(theme.surface()),
                        elevation: 1.5,
                    }))),
            ]
            .spacing(0)
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
}

async fn load_package_info(package_names: Vec<String>, is_flatpak: bool) -> Result<Vec<PackageDetail>, String> {
    use futures::future;
    
    let futures: Vec<_> = package_names.into_iter()
        .map(|pkg| load_single_package_detail(pkg, is_flatpak))
        .collect();
    
    let results: Vec<Result<PackageDetail, String>> = future::join_all(futures).await;
    
    let mut details = Vec::new();
    for result in results {
        match result {
            Ok(detail) => details.push(detail),
            Err(e) => {
                eprintln!("Warning: Failed to load package detail: {}", e);
            }
        }
    }
    
    if details.is_empty() {
        Err("Failed to load information for any packages".to_string())
    } else {
        Ok(details)
    }
}

async fn load_single_package_detail(package: String, is_flatpak: bool) -> Result<PackageDetail, String> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command;
        
        if is_flatpak {
            let info_output = run_command("flatpak", &["info", &package], false)
                .map_err(|e| format!("Failed to get flatpak info: {}", e))?;
            let mut version = String::new();
            let mut description = String::new();
            let mut size = String::new();
            
            let lines: Vec<&str> = info_output.lines().collect();
            let mut is_first_line = true;
            
            for line in &lines {
                let line = line.trim();
                if line.is_empty() {
                    continue;
                }
                
                // First non-empty line is usually the description (format: "Name - Description")
                if is_first_line && !line.contains(':') {
                    // Extract description from "Name - Description" format
                    if let Some(dash_pos) = line.find(" - ") {
                        description = line[dash_pos + 3..].trim().to_string();
                    } else {
                        description = line.to_string();
                    }
                    is_first_line = false;
                    continue;
                }
                is_first_line = false;
                
                // Parse version
                if line.starts_with("Version:") {
                    version = line.replace("Version:", "").trim().to_string();
                }
                // Parse description (if not found on first line)
                else if line.starts_with("Description:") {
                    description = line.replace("Description:", "").trim().to_string();
                }
                // Parse size
                else if line.starts_with("Installed size:") {
                    size = line.replace("Installed size:", "").trim().to_string();
                } else if line.starts_with("Installed:") {
                    size = line.replace("Installed:", "").trim().to_string();
                }
            }
            
            Ok(PackageDetail {
                name: package,
                version: if version.is_empty() { "Unknown".to_string() } else { version },
                description: if description.is_empty() { "No description available".to_string() } else { description },
                size: if size.is_empty() { "Unknown".to_string() } else { size },
                is_flatpak: true,
            })
        } else {
            let show_output = run_command("apt", &["show", &package], false)
                .map_err(|e| format!("Failed to get package info: {}", e))?;
            let mut version = String::new();
            let mut description = String::new();
            let mut size = String::new();
            
            for line in show_output.lines() {
                if line.starts_with("Version:") {
                    version = line.replace("Version:", "").trim().to_string();
                } else if line.starts_with("Description:") {
                    description = line.replace("Description:", "").trim().to_string();
                } else if line.starts_with("Installed-Size:") {
                    let size_kb = line.replace("Installed-Size:", "").trim().to_string();
                    if let Ok(kb) = size_kb.parse::<f64>() {
                        if kb >= 1024.0 {
                            size = format!("{:.2} MB", kb / 1024.0);
                        } else {
                            size = format!("{} KB", size_kb);
                        }
                    } else {
                        size = format!("{} KB", size_kb);
                    }
                }
            }
            
            if description.is_empty() {
                for line in show_output.lines() {
                    if line.starts_with("Description:") {
                        description = line.replace("Description:", "").trim().to_string();
                    } else if !description.is_empty() && line.starts_with(" ") {
                        description.push_str(" ");
                        description.push_str(line.trim());
                    }
                }
            }
            
            Ok(PackageDetail {
                name: package,
                version: if version.is_empty() { "Unknown".to_string() } else { version },
                description: if description.is_empty() { "No description available".to_string() } else { description },
                size: if size.is_empty() { "Unknown".to_string() } else { size },
                is_flatpak: false,
            })
        }
    })
    .await
    .map_err(|e| format!("Task error: {}", e))?
}

async fn remove_packages(package_names: Vec<String>, is_flatpak: bool) -> Result<(String, String), String> {
    eprintln!("[DEBUG] remove_packages: Starting removal, packages: {:?}, is_flatpak: {}", package_names, is_flatpak);
    
    if is_flatpak {
        // Remove flatpak packages
        eprintln!("[DEBUG] remove_packages: Using flatpak uninstall");
        let mut all_output = String::new();
        for package in &package_names {
            eprintln!("[DEBUG] remove_packages: Removing flatpak package: {}", package);
            let mut cmd = TokioCommand::new("flatpak");
            cmd.arg("uninstall");
            // Add --noninteractive flag to skip confirmation (user already confirmed in GUI)
            cmd.arg("--noninteractive");
            cmd.arg("-y"); // Also add -y flag for yes to all prompts
            cmd.arg(package);
            
            eprintln!("[DEBUG] remove_packages: Executing: flatpak uninstall {}", package);
            let output = cmd
                .output()
                .await
                .map_err(|e| {
                    eprintln!("[DEBUG] remove_packages: Command execution error: {}", e);
                    format!("Failed to execute removal: {}", e)
                })?;
            
            eprintln!("[DEBUG] remove_packages: Command exit code: {:?}", output.status.code());
            eprintln!("[DEBUG] remove_packages: Command success: {}", output.status.success());
            
            // Capture stdout and stderr
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            
            eprintln!("[DEBUG] remove_packages: stdout length: {}, stderr length: {}", stdout.len(), stderr.len());
            if !stdout.is_empty() {
                eprintln!("[DEBUG] remove_packages: stdout: {}", stdout);
            }
            if !stderr.is_empty() {
                eprintln!("[DEBUG] remove_packages: stderr: {}", stderr);
            }
            
            if !all_output.is_empty() {
                all_output.push('\n');
            }
            if !stdout.is_empty() {
                all_output.push_str(&stdout);
            }
            if !stderr.is_empty() {
                if !all_output.is_empty() && !all_output.ends_with('\n') {
                    all_output.push('\n');
                }
                all_output.push_str(&stderr);
            }
            
            if !output.status.success() {
                eprintln!("[DEBUG] remove_packages: Removal failed for package: {}", package);
                return Err(format!("Removal failed: {}", stderr));
            }
            eprintln!("[DEBUG] remove_packages: Successfully removed package: {}", package);
        }
        eprintln!("[DEBUG] remove_packages: All flatpak packages removed successfully");
        Ok(("Removal Complete!".to_string(), all_output))
    } else {
        // Remove apt packages using pkexec
        eprintln!("[DEBUG] remove_packages: Using apt-get remove via pkexec");
        // Use apt-get instead of apt for more reliable output
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("apt-get");
        cmd.arg("remove");
        // Add -y flag since user already confirmed in GUI
        cmd.arg("-y");
        for name in &package_names {
            cmd.arg(name);
            eprintln!("[DEBUG] remove_packages: Adding package to remove: {}", name);
        }
        
        // Set DEBIAN_FRONTEND=noninteractive to avoid prompts (user already confirmed in GUI)
        cmd.env("DEBIAN_FRONTEND", "noninteractive");
        eprintln!("[DEBUG] remove_packages: Set DEBIAN_FRONTEND=noninteractive");
        
        // Ensure DISPLAY is set for GUI password dialog
        if let Ok(display) = std::env::var("DISPLAY") {
            let display_clone = display.clone();
            cmd.env("DISPLAY", display);
            eprintln!("[DEBUG] remove_packages: Set DISPLAY={}", display_clone);
        } else {
            eprintln!("[DEBUG] remove_packages: DISPLAY not set");
        }
        if let Ok(xauth) = std::env::var("XAUTHORITY") {
            cmd.env("XAUTHORITY", xauth);
            eprintln!("[DEBUG] remove_packages: Set XAUTHORITY");
        }
        if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
            let wayland_clone = wayland.clone();
            cmd.env("WAYLAND_DISPLAY", wayland);
            eprintln!("[DEBUG] remove_packages: Set WAYLAND_DISPLAY={}", wayland_clone);
        }
        
        // Also preserve PATH and other important env vars
        if let Ok(path) = std::env::var("PATH") {
            cmd.env("PATH", path);
        }
        
        let cmd_str = format!("pkexec apt-get remove -y {}", package_names.join(" "));
        eprintln!("[DEBUG] remove_packages: Executing command: {}", cmd_str);
        
        let output = cmd
            .output()
            .await
            .map_err(|e| {
                eprintln!("[DEBUG] remove_packages: Command execution error: {}", e);
                format!("Failed to execute removal: {}. Make sure polkit is installed.", e)
            })?;
        
        let exit_code = output.status.code();
        eprintln!("[DEBUG] remove_packages: Command exit code: {:?}", exit_code);
        eprintln!("[DEBUG] remove_packages: Command success: {}", output.status.success());
        
        // Capture stdout and stderr
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        
        eprintln!("[DEBUG] remove_packages: stdout length: {}, stderr length: {}", stdout.len(), stderr.len());
        if !stdout.is_empty() {
            eprintln!("[DEBUG] remove_packages: stdout (first 500 chars): {}", &stdout.chars().take(500).collect::<String>());
        }
        if !stderr.is_empty() {
            eprintln!("[DEBUG] remove_packages: stderr (first 500 chars): {}", &stderr.chars().take(500).collect::<String>());
        }
        
        let mut all_output = String::new();
        if !stdout.is_empty() {
            all_output.push_str(&stdout);
        }
        if !stderr.is_empty() {
            if !all_output.is_empty() && !all_output.ends_with('\n') {
                all_output.push('\n');
            }
            all_output.push_str(&stderr);
        }
        
        // If output is empty but command succeeded, apt might have run silently
        // This can happen when packages are already removed or don't exist
        if all_output.is_empty() && output.status.success() {
            eprintln!("[DEBUG] remove_packages: Command succeeded but output is empty");
            all_output = format!("Packages removed successfully.\nExit code: {:?}", exit_code);
        }
        
        if !output.status.success() {
            eprintln!("[DEBUG] remove_packages: Command failed");
            if exit_code == Some(126) || exit_code == Some(127) {
                eprintln!("[DEBUG] remove_packages: Authentication error (exit code {:?})", exit_code);
                return Err("Authentication cancelled or failed. Please try again.".to_string());
            }
            
            // Check for conflicts in the error output
            let combined_error = format!("{}\n{}", stdout, stderr);
            if let Some(conflict_msg) = detect_conflicts(&combined_error) {
                eprintln!("[DEBUG] remove_packages: Conflict detected in error output");
                // Return a special error that will trigger conflict dialog
                return Err(format!("CONFLICT_DETECTED:{}", conflict_msg));
            }
            
            // Include exit code in error for debugging
            let error_msg = if all_output.is_empty() {
                format!("Removal failed with exit code: {:?}", exit_code)
            } else {
                format!("Removal failed: {}\nExit code: {:?}", stderr, exit_code)
            };
            eprintln!("[DEBUG] remove_packages: Error message: {}", error_msg);
            return Err(error_msg);
        }
        
        eprintln!("[DEBUG] remove_packages: Removal completed successfully");
        Ok(("Removal Complete!".to_string(), all_output))
    }
}

// Function to detect conflicts in apt-get output and extract user-friendly messages
fn detect_conflicts(output: &str) -> Option<String> {
    let output_lower = output.to_lowercase();
    
    // Check for specific conflict types and extract detailed information
    if output_lower.contains("unmet dependencies") || output_lower.contains("depends:") {
        return extract_dependency_conflict(output);
    }
    
    if output_lower.contains("conflicts with") {
        return extract_conflict_message(output, "Package conflicts detected");
    }
    
    if output_lower.contains("is held") || output_lower.contains("held") {
        return extract_conflict_message(output, "Package is held and cannot be removed");
    }
    
    if output_lower.contains("could not be removed") || output_lower.contains("cannot remove") {
        return extract_conflict_message(output, "Some packages could not be removed");
    }
    
    if output_lower.contains("broken packages") || output_lower.contains("dependency problems") {
        return extract_conflict_message(output, "Broken packages or dependency problems detected");
    }
    
    if output_lower.contains("you have held broken packages") {
        return extract_conflict_message(output, "Held broken packages prevent removal");
    }
    
    // Check for common error patterns that indicate conflicts
    if output_lower.contains("error") && (output_lower.contains("dependency") || output_lower.contains("conflict")) {
        return extract_conflict_message(output, "Dependency or conflict error detected");
    }
    
    None
}

fn extract_dependency_conflict(output: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    let mut conflict_lines = Vec::new();
    let mut in_dependency_section = false;
    
    for line in &lines {
        let line_lower = line.to_lowercase();
        if line_lower.contains("unmet dependencies") || 
           line_lower.contains("the following packages") ||
           line_lower.contains("depends:") ||
           line_lower.contains("predepends:") {
            in_dependency_section = true;
        }
        
        if in_dependency_section {
            conflict_lines.push(*line);
            // Stop when we see a solution or empty section
            if line_lower.contains("you can run") || 
               line_lower.contains("apt --fix-broken install") ||
               (conflict_lines.len() > 15 && line.trim().is_empty()) {
                break;
            }
        }
    }
    
    if !conflict_lines.is_empty() {
        let conflict_text = conflict_lines.join("\n");
        Some(format!("The following packages have unmet dependencies or dependency conflicts:\n\n{}\n\nThis usually means other packages depend on the package you're trying to remove, or removing it would break the system.", conflict_text))
    } else {
        Some("Dependency conflict detected. Other packages depend on the package(s) you're trying to remove.".to_string())
    }
}

fn extract_conflict_message(output: &str, title: &str) -> Option<String> {
    let lines: Vec<&str> = output.lines().collect();
    let mut relevant_lines = Vec::new();
    
    // Find the conflict-related section
    for (i, line) in lines.iter().enumerate() {
        let line_lower = line.to_lowercase();
        if line_lower.contains("conflict") || 
           line_lower.contains("held") ||
           line_lower.contains("cannot") ||
           line_lower.contains("error") {
            // Include context around the conflict
            let start = i.saturating_sub(1);
            let end = (i + 8).min(lines.len());
            for j in start..end {
                if !lines[j].trim().is_empty() {
                    relevant_lines.push(lines[j]);
                }
            }
            break;
        }
    }
    
    if !relevant_lines.is_empty() {
        let conflict_text = relevant_lines.join("\n");
        Some(format!("{}\n\nDetails:\n{}", title, conflict_text))
    } else {
        Some(title.to_string())
    }
}

