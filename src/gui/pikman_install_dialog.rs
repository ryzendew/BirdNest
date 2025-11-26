use iced::{
    alignment, executor, Color,
    widget::{button, column, container, row, scrollable, text, Space},
    Application, Command, Element, Length, Pixels, Settings, Theme as IcedTheme, Padding,
    window,
};
use tokio::process::Command as TokioCommand;
use std::fmt;

use crate::gui::theme::Theme as AppTheme;
use crate::gui::styles::{RoundedButtonStyle, RoundedContainerStyle, CustomScrollableStyle};

#[derive(Debug, Clone)]
pub enum Message {
    LoadPackageInfo,
    PackageInfoLoaded(Vec<PackageDetail>),
    InstallPackages,
    ConfirmInstall,
    #[allow(dead_code)]
    InstallationProgress(String),
    TerminalOutput(String),
    InstallationComplete,
    InstallationError(String),
    ConflictDetected(String),
    DistroChanged(Option<DistroType>),
    Cancel,
}

#[derive(Debug, Clone)]
pub struct PackageDetail {
    pub name: String,
    pub version: String,
    pub description: String,
    pub size: String,
    pub repository: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DistroType {
    Default,
    Aur,
    Fedora,
    Alpine,
}

impl fmt::Display for DistroType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl DistroType {
    #[allow(dead_code)]
    pub fn all() -> Vec<DistroType> {
        vec![
            DistroType::Default,
            DistroType::Aur,
            DistroType::Fedora,
            DistroType::Alpine,
        ]
    }

    pub fn as_str(&self) -> &str {
        match self {
            DistroType::Default => "Default (System)",
            DistroType::Aur => "AUR (Arch)",
            DistroType::Fedora => "Fedora",
            DistroType::Alpine => "Alpine",
        }
    }

    pub fn to_pikman_flag(&self) -> Option<&str> {
        match self {
            DistroType::Default => None,
            DistroType::Aur => Some("--aur"),
            DistroType::Fedora => Some("--fedora"),
            DistroType::Alpine => Some("--alpine"),
        }
    }
}

#[derive(Debug)]
pub struct PikmanInstallDialog {
    pub package_names: Vec<String>,
    pub package_info: Vec<PackageDetail>,
    pub is_loading: bool,
    pub is_installing: bool,
    pub is_complete: bool,
    pub show_confirmation: bool,
    pub installation_progress: String,
    pub terminal_output: String,
    pub conflict_message: Option<String>,
    pub selected_distro: Option<DistroType>,
    pub theme: AppTheme,
    pub border_radius: f32,
}

impl PikmanInstallDialog {
    pub fn new(package_names: Vec<String>) -> Self {
        Self {
            package_names,
            package_info: Vec::new(),
            is_loading: true,
            is_installing: false,
            is_complete: false,
            show_confirmation: false,
            installation_progress: String::new(),
            terminal_output: String::new(),
            conflict_message: None,
            selected_distro: Some(DistroType::Default),
            theme: AppTheme::Dark,
            border_radius: 12.0,
        }
    }

    pub fn run_separate_window(package_names: Vec<String>) -> Result<(), iced::Error> {
        let dialog = Self::new(package_names);

        let mut window_settings = window::Settings::default();
        window_settings.size = iced::Size::new(800.0, 900.0);
        window_settings.min_size = Some(iced::Size::new(600.0, 500.0));
        window_settings.resizable = true;
        window_settings.decorations = true;

        <PikmanInstallDialog as Application>::run(Settings {
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

impl Application for PikmanInstallDialog {
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
                format!("Install {} - BirdNest", self.package_info[0].name)
            } else {
                format!("Install {} Packages - BirdNest", self.package_info.len())
            }
        } else {
            "Install Package - BirdNest".to_string()
        }
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::LoadPackageInfo => {
                self.is_loading = true;
                let package_names = self.package_names.clone();
                Command::perform(load_package_info(package_names), |result| {
                    match result {
                        Ok(infos) => Message::PackageInfoLoaded(infos),
                        Err(e) => Message::InstallationError(e),
                    }
                })
            }
            Message::PackageInfoLoaded(infos) => {
                self.is_loading = false;
                self.package_info = infos;
                Command::none()
            }
            Message::DistroChanged(distro) => {
                self.selected_distro = distro.clone();
                Command::none()
            }
            Message::InstallPackages => {
                self.show_confirmation = true;
                Command::none()
            }
            Message::ConfirmInstall => {
                self.show_confirmation = false;
                self.is_installing = true;
                self.installation_progress = "Preparing installation...".to_string();
                self.terminal_output.clear();
                let package_names = self.package_names.clone();
                let distro = self.selected_distro.clone();
                Command::perform(install_packages(package_names, distro), |result| {
                    match result {
                        Ok((progress, output)) => {
                            if progress.contains("conflict") || progress.contains("error") || progress.contains("failed") {
                                Message::ConflictDetected(progress)
                            } else {
                                Message::TerminalOutput(output)
                            }
                        }
                        Err(e) => {
                            let err_msg = e.to_string();
                            if err_msg.contains("conflict") || err_msg.contains("depends") || err_msg.contains("broken") {
                                Message::ConflictDetected(err_msg)
                            } else {
                                Message::InstallationError(err_msg)
                            }
                        }
                    }
                })
            }
            Message::TerminalOutput(output) => {
                self.terminal_output = output.clone();
                if output.contains("Complete") || output.contains("Installed") || output.contains("complete") || output.to_lowercase().contains("success") {
                    Command::perform(async {}, |_| Message::InstallationComplete)
                } else {
                    Command::none()
                }
            }
            Message::InstallationProgress(progress) => {
                self.installation_progress = progress;
                Command::none()
            }
            Message::InstallationComplete => {
                self.is_installing = false;
                self.is_complete = true;
                self.installation_progress = "Installation completed successfully!".to_string();
                Command::none()
            }
            Message::InstallationError(msg) => {
                self.is_installing = false;
                self.installation_progress = format!("Error: {}", msg);
                Command::none()
            }
            Message::ConflictDetected(msg) => {
                self.is_installing = false;
                self.conflict_message = Some(msg);
                Command::none()
            }
            Message::Cancel => {
                window::close::<Message>(window::Id::MAIN)
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let theme = self.theme;
        
        if self.is_loading {
            return container(
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
            .into();
        }

        let content = if self.show_confirmation {
            view_confirmation(self, theme)
        } else if self.is_installing {
            view_installing(self, theme)
        } else if self.is_complete {
            view_complete(self, theme)
        } else {
            view_package_info(self, theme)
        };

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                radius: 0.0,
                background: Some(theme.background()),
                elevation: 0.0,
            })))
            .into()
    }

    fn theme(&self) -> IcedTheme {
        match self.theme {
            AppTheme::Dark => IcedTheme::Dark,
            AppTheme::Light => IcedTheme::Light,
        }
    }
}

fn view_package_info(dialog: &PikmanInstallDialog, theme: AppTheme) -> Element<Message> {
    let mut content = column![
        text("Install Package")
            .size(24)
            .style(iced::theme::Text::Color(theme.primary())),
        Space::with_height(Length::Fixed(20.0)),
    ]
    .spacing(15);

    // Package info
    for pkg in &dialog.package_info {
        content = content.push(
            container(
                column![
                    text(&pkg.name)
                        .size(18)
                        .style(iced::theme::Text::Color(theme.text())),
                    text(&pkg.description)
                        .size(14)
                        .style(iced::theme::Text::Color(theme.secondary_text())),
                    row![
                        text(format!("Version: {}", pkg.version))
                            .size(12)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                        Space::with_width(Length::Fixed(20.0)),
                        text(format!("Repository: {}", pkg.repository))
                            .size(12)
                            .style(iced::theme::Text::Color(theme.secondary_text())),
                    ]
                    .spacing(10),
                ]
                .spacing(8)
            )
            .padding(Padding::new(16.0))
            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                radius: dialog.border_radius,
                background: Some(theme.background()),
                elevation: 0.0,
            })))
        );
    }

    // Distro selection - use buttons instead of pick_list to avoid lifetime issues
    content = content.push(
        container(
            column![
                text("Select Distribution Source:")
                    .size(14)
                    .style(iced::theme::Text::Color(theme.text())),
                row![
                    button(if dialog.selected_distro == Some(DistroType::Default) { "✓ Default" } else { "Default" })
                        .on_press(Message::DistroChanged(Some(DistroType::Default)))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: dialog.selected_distro == Some(DistroType::Default),
                            radius: dialog.border_radius,
                            primary_color: theme.primary(),
                            text_color: if dialog.selected_distro == Some(DistroType::Default) { Color::WHITE } else { theme.text() },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(10.0)),
                    button(if dialog.selected_distro == Some(DistroType::Aur) { "✓ AUR" } else { "AUR" })
                        .on_press(Message::DistroChanged(Some(DistroType::Aur)))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: dialog.selected_distro == Some(DistroType::Aur),
                            radius: dialog.border_radius,
                            primary_color: theme.primary(),
                            text_color: if dialog.selected_distro == Some(DistroType::Aur) { Color::WHITE } else { theme.text() },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(10.0)),
                    button(if dialog.selected_distro == Some(DistroType::Fedora) { "✓ Fedora" } else { "Fedora" })
                        .on_press(Message::DistroChanged(Some(DistroType::Fedora)))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: dialog.selected_distro == Some(DistroType::Fedora),
                            radius: dialog.border_radius,
                            primary_color: theme.primary(),
                            text_color: if dialog.selected_distro == Some(DistroType::Fedora) { Color::WHITE } else { theme.text() },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(10.0)),
                    button(if dialog.selected_distro == Some(DistroType::Alpine) { "✓ Alpine" } else { "Alpine" })
                        .on_press(Message::DistroChanged(Some(DistroType::Alpine)))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: dialog.selected_distro == Some(DistroType::Alpine),
                            radius: dialog.border_radius,
                            primary_color: theme.primary(),
                            text_color: if dialog.selected_distro == Some(DistroType::Alpine) { Color::WHITE } else { theme.text() },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(10.0)),
                ]
                .spacing(8)
                .width(Length::Fill),
            ]
            .spacing(10)
        )
        .padding(Padding::new(16.0))
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: dialog.border_radius,
            background: Some(theme.background()),
            elevation: 0.0,
        })))
    );

    // Conflict message if any
    if let Some(ref conflict) = dialog.conflict_message {
        content = content.push(
            container(
                column![
                    text("⚠️ Installation Conflict")
                        .size(16)
                        .style(iced::theme::Text::Color(Color::from_rgb(1.0, 0.6, 0.0))),
                    text(conflict)
                        .size(12)
                        .style(iced::theme::Text::Color(theme.danger())),
                ]
                .spacing(8)
            )
            .padding(Padding::new(16.0))
            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                radius: dialog.border_radius,
                background: Some(Color::from_rgba(1.0, 0.6, 0.0, 0.1)),
                elevation: 1.0,
            })))
        );
    }

    // Buttons
    content = content.push(
        row![
            button("Cancel")
                .on_press(Message::Cancel)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: false,
                    radius: dialog.border_radius,
                    primary_color: theme.primary(),
                    text_color: theme.text(),
                    background_color: theme.background(),
                })))
                .padding(Padding::new(14.0)),
            Space::with_width(Length::Fill),
            button("Install")
                .on_press(Message::InstallPackages)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: dialog.border_radius,
                    primary_color: theme.primary(),
                    text_color: Color::WHITE,
                    background_color: theme.background(),
                })))
                .padding(Padding::new(14.0)),
        ]
        .spacing(10)
    );

    container(
        scrollable(content)
            .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                background_color: theme.background(),
                border_radius: dialog.border_radius,
            })))
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .padding(Padding::new(20.0))
    .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
        radius: 0.0,
        background: Some(theme.background()),
        elevation: 0.0,
    })))
    .into()
}

fn view_confirmation(dialog: &PikmanInstallDialog, theme: AppTheme) -> Element<Message> {
    let distro_name = dialog.selected_distro.as_ref()
        .map(|d| d.as_str())
        .unwrap_or("Default");
    
    container(
        column![
            text("Confirm Installation")
                .size(24)
                .style(iced::theme::Text::Color(theme.primary())),
            Space::with_height(Length::Fixed(20.0)),
            text(format!("Install {} package(s) from {}?", dialog.package_names.len(), distro_name))
                .size(16)
                .style(iced::theme::Text::Color(theme.text())),
            Space::with_height(Length::Fixed(20.0)),
            row![
                button("Cancel")
                    .on_press(Message::Cancel)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: false,
                        radius: dialog.border_radius,
                        primary_color: theme.primary(),
                        text_color: theme.text(),
                        background_color: theme.background(),
                    })))
                    .padding(Padding::new(14.0)),
                Space::with_width(Length::Fill),
                button("Confirm")
                    .on_press(Message::ConfirmInstall)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: true,
                        radius: dialog.border_radius,
                        primary_color: theme.primary(),
                        text_color: Color::WHITE,
                        background_color: theme.background(),
                    })))
                    .padding(Padding::new(14.0)),
            ]
            .spacing(10),
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
        radius: dialog.border_radius,
        background: Some(theme.surface()),
        elevation: 1.5,
    })))
    .into()
}

fn view_installing(dialog: &PikmanInstallDialog, theme: AppTheme) -> Element<Message> {
    container(
        column![
            text("Installing Package...")
                .size(24)
                .style(iced::theme::Text::Color(theme.primary())),
            Space::with_height(Length::Fixed(20.0)),
            if !dialog.terminal_output.is_empty() {
                Element::from(
                    container(
                        scrollable(
                            text(&dialog.terminal_output)
                                .size(12)
                                .style(iced::theme::Text::Color(theme.text()))
                        )
                        .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                            background_color: theme.surface(),
                            border_radius: dialog.border_radius,
                        })))
                        .height(Length::Fixed(400.0))
                    )
                    .padding(Padding::new(12.0))
                    .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                        radius: dialog.border_radius,
                        background: Some(theme.surface()),
                        elevation: 1.0,
                    })))
                )
            } else {
                Element::from(
                    container(
                        text(&dialog.installation_progress)
                            .size(14)
                            .style(iced::theme::Text::Color(theme.text()))
                    )
                    .width(Length::Fill)
                    .height(Length::Fixed(100.0))
                    .center_x()
                    .center_y()
                    .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                        radius: dialog.border_radius,
                        background: Some(theme.surface()),
                        elevation: 1.0,
                    })))
                )
            },
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
        radius: dialog.border_radius,
        background: Some(theme.surface()),
        elevation: 1.5,
    })))
    .into()
}

fn view_complete(dialog: &PikmanInstallDialog, theme: AppTheme) -> Element<Message> {
    container(
        column![
            text("Installation Complete")
                .size(24)
                .style(iced::theme::Text::Color(Color::from_rgb(0.4, 0.8, 0.4))),
            Space::with_height(Length::Fixed(20.0)),
            text(&dialog.installation_progress)
                .size(16)
                .style(iced::theme::Text::Color(theme.text())),
            Space::with_height(Length::Fixed(20.0)),
            button("Close")
                .on_press(Message::Cancel)
                .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                    is_primary: true,
                    radius: dialog.border_radius,
                    primary_color: theme.primary(),
                    text_color: Color::WHITE,
                    background_color: theme.background(),
                })))
                .padding(Padding::new(14.0)),
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
        radius: dialog.border_radius,
        background: Some(theme.surface()),
        elevation: 1.5,
    })))
    .into()
}

async fn load_package_info(package_names: Vec<String>) -> Result<Vec<PackageDetail>, String> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command;
        let mut details = Vec::new();
        
        for name in package_names {
            // Try to get package info using pikman show
            if let Ok(output) = run_command("pikman", &["show", &name], false) {
                let mut detail = PackageDetail {
                    name: name.clone(),
                    version: "Unknown".to_string(),
                    description: String::new(),
                    size: "Unknown".to_string(),
                    repository: "Unknown".to_string(),
                };
                
                for line in output.lines() {
                    if line.starts_with("Version:") {
                        detail.version = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Description:") {
                        detail.description = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Size:") {
                        detail.size = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    } else if line.starts_with("Repository:") {
                        detail.repository = line.split(':').nth(1).unwrap_or("").trim().to_string();
                    }
                }
                
                details.push(detail);
            } else {
                // Fallback if pikman show fails
                details.push(PackageDetail {
                    name,
                    version: "Unknown".to_string(),
                    description: "Package information not available".to_string(),
                    size: "Unknown".to_string(),
                    repository: "Unknown".to_string(),
                });
            }
        }
        
        Ok(details)
    })
    .await
    .map_err(|e| format!("Failed to load package info: {}", e))?
}

async fn install_packages(
    package_names: Vec<String>,
    distro: Option<DistroType>,
) -> Result<(String, String), anyhow::Error> {
    use tokio::io::{AsyncBufReadExt, BufReader};
    
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("pikman");
    cmd.arg("install");
    cmd.arg("-y");
    
    if let Some(ref d) = distro {
        if let Some(flag) = d.to_pikman_flag() {
            cmd.arg(flag);
        }
    }
    
    for pkg in &package_names {
        cmd.arg(pkg);
    }
    
    // Preserve environment variables for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(xauth) = std::env::var("XAUTHORITY") {
        cmd.env("XAUTHORITY", xauth);
    }
    if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
        cmd.env("WAYLAND_DISPLAY", wayland);
    }
    if let Ok(path) = std::env::var("PATH") {
        cmd.env("PATH", path);
    }
    
    cmd.env("DEBIAN_FRONTEND", "noninteractive");
    
    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()?;
    let stdout = child.stdout.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stdout"))?;
    let stderr = child.stderr.take().ok_or_else(|| anyhow::anyhow!("Failed to capture stderr"))?;
    
    let mut stdout_reader = BufReader::new(stdout).lines();
    let mut stderr_reader = BufReader::new(stderr).lines();
    
    let mut combined_output = String::new();
    combined_output.push_str(&format!("Installing {} package(s)...\n", package_names.len()));
    combined_output.push_str(&format!("Packages: {}\n", package_names.join(", ")));
    combined_output.push_str("(You may be prompted for your password)\n");
    combined_output.push_str("--- Output ---\n");
    
    loop {
        tokio::select! {
            result = stdout_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        combined_output.push_str(&line);
                        combined_output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        combined_output.push_str(&format!("Error reading stdout: {}\n", e));
                        break;
                    }
                }
            }
            result = stderr_reader.next_line() => {
                match result {
                    Ok(Some(line)) => {
                        combined_output.push_str(&line);
                        combined_output.push('\n');
                    }
                    Ok(None) => break,
                    Err(e) => {
                        combined_output.push_str(&format!("Error reading stderr: {}\n", e));
                        break;
                    }
                }
            }
        }
    }
    
    let status = child.wait().await?;
    combined_output.push_str(&format!("\nExit code: {}\n", status.code().unwrap_or(-1)));
    
    if !status.success() {
        if status.code() == Some(126) || status.code() == Some(127) {
            combined_output.push_str("\n❌ Authentication failed or cancelled.\n");
            anyhow::bail!("Authentication failed or cancelled. Please try again.");
        }
        
        // Check for conflicts
        if combined_output.contains("conflict") || combined_output.contains("depends") || combined_output.contains("broken") {
            anyhow::bail!("Package conflict detected. See output for details.");
        }
        
        combined_output.push_str("\n❌ Installation failed.\n");
        anyhow::bail!("Installation failed. See output above for details.");
    }
    
    combined_output.push_str("\n✓ Packages installed successfully\n");
    
    Ok(("Installation complete".to_string(), combined_output))
}

