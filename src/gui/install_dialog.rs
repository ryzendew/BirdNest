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
    InstallPackages,
    InstallationProgress(String),
    InstallationComplete,
    InstallationError(String),
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
pub struct InstallDialog {
    pub package_names: Vec<String>,
    pub package_info: Vec<PackageDetail>,
    pub is_loading: bool,
    pub is_installing: bool,
    pub is_complete: bool,
    pub installation_progress: String,
    pub theme: AppTheme,
    pub border_radius: f32,
    pub is_flatpak: bool,
}

impl InstallDialog {
    pub fn new(package_names: Vec<String>, is_flatpak: bool) -> Self {
        Self {
            package_names,
            package_info: Vec::new(),
            is_loading: true,
            is_installing: false,
            is_complete: false,
            installation_progress: String::new(),
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

        <InstallDialog as Application>::run(Settings {
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

impl Application for InstallDialog {
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
                let is_flatpak = self.is_flatpak;
                Command::perform(load_package_info(package_names, is_flatpak), |result| {
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
            Message::InstallPackages => {
                self.is_installing = true;
                self.installation_progress = "Preparing installation...".to_string();
                let package_names = self.package_names.clone();
                let is_flatpak = self.package_info.first().map(|p| p.is_flatpak).unwrap_or(false);
                Command::perform(install_packages(package_names, is_flatpak), |result| {
                    match result {
                        Ok(progress) => Message::InstallationProgress(progress),
                        Err(e) => Message::InstallationError(e.to_string()),
                    }
                })
            }
            Message::InstallationProgress(progress) => {
                let progress_clone = progress.clone();
                self.installation_progress = progress;
                if progress_clone.contains("Complete") ||
                   progress_clone.contains("Installed") ||
                   progress_clone.contains("complete") ||
                   progress_clone.to_lowercase().contains("success") {
                    Command::perform(async {}, |_| Message::InstallationComplete)
                } else {
                    Command::none()
                }
            }
            Message::InstallationComplete => {
                self.is_installing = false;
                self.is_complete = true;
                self.installation_progress = "Installation completed successfully!".to_string();
                Command::none()
            }
            Message::InstallationError(_msg) => {
                self.is_installing = false;
                Command::none()
            }
            Message::Cancel => {
                iced::window::close(iced::window::Id::MAIN)
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

impl InstallDialog {
    fn view_package_info(&self) -> Element<Message> {
        let theme = self.theme;
        let needs_sudo = !self.package_info.first().map(|p| p.is_flatpak).unwrap_or(false);
        
        let title_text = if self.package_info.len() == 1 {
            format!("Install {}", self.package_info[0].name)
        } else {
            format!("Install {} Packages", self.package_info.len())
        };

        let buttons = if self.is_complete {
            row![
                button("Exit")
                    .on_press(Message::Cancel)
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
                    if self.is_installing {
                        button("Installing...")
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                                radius: self.border_radius,
                                primary_color: theme.primary(),
                                text_color: Color::WHITE,
                                background_color: theme.background(),
                            })))
                            .padding(Padding::new(14.0))
                    } else {
                        button("Install")
                            .on_press(Message::InstallPackages)
                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                is_primary: true,
                                radius: self.border_radius,
                                primary_color: theme.primary(),
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
                    .style(iced::theme::Text::Color(theme.primary())),
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
                    .style(iced::theme::Text::Color(theme.primary())),
                Space::with_height(Length::Fixed(20.0)),
                text("Packages to install:")
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

        let progress_section = if !self.installation_progress.is_empty() {
            column![
                Space::with_height(Length::Fixed(20.0)),
                text(&self.installation_progress)
                    .size(14)
                    .style(iced::theme::Text::Color(if self.is_complete {
                        Color::from_rgb(0.0, 1.0, 0.0)
                    } else if self.installation_progress.contains("Error") || self.installation_progress.contains("error") {
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
                        if needs_sudo && !self.is_installing && !self.is_complete {
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
    
    #[cfg(debug_assertions)]
    eprintln!("[DEBUG] Loading package info for {} packages (flatpak: {})", package_names.len(), is_flatpak);
    
    let futures: Vec<_> = package_names.into_iter()
        .map(|pkg| {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Loading detail for package: {}", pkg);
            load_single_package_detail(pkg, is_flatpak)
        })
        .collect();
    
    let results: Vec<Result<PackageDetail, String>> = future::join_all(futures).await;
    
    let mut details = Vec::new();
    for result in results {
        match result {
            Ok(detail) => {
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG] Successfully loaded package detail: {}", detail.name);
                details.push(detail)
            },
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG] Warning: Failed to load package detail: {}", e);
                eprintln!("Warning: Failed to load package detail: {}", e);
            }
        }
    }
    
    if details.is_empty() {
        Err("Failed to load information for any packages".to_string())
    } else {
        #[cfg(debug_assertions)]
        eprintln!("[DEBUG] Loaded {} package details", details.len());
        Ok(details)
    }
}

async fn load_single_package_detail(package: String, is_flatpak: bool) -> Result<PackageDetail, String> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] load_single_package_detail() called for package: '{}', is_flatpak: {}", package, is_flatpak);
    
    // Validate Flatpak application ID format (must contain at least 2 periods)
    if is_flatpak && !package.contains('.') {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] load_single_package_detail: ERROR - Invalid Flatpak ID format: '{}' (must contain at least one period)", package);
        return Err(format!("Invalid Flatpak application ID: '{}'. Application IDs must be in the format 'org.example.App' (containing at least one period).", package));
    }
    
    // Count periods to ensure it's a valid Flatpak ID
    if is_flatpak {
        let period_count = package.matches('.').count();
        if period_count < 1 {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_single_package_detail: ERROR - Invalid Flatpak ID format: '{}' (must contain at least one period, found {})", package, period_count);
            return Err(format!("Invalid Flatpak application ID: '{}'. Application IDs must be in the format 'org.example.App' (containing at least one period).", package));
        }
    }
    
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command;
        
        if is_flatpak {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_single_package_detail: Loading flatpak package info...");
            // Try flatpak info first (for installed packages)
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_single_package_detail: Trying 'flatpak info {}'...", package);
            let info_output = match run_command("flatpak", &["info", &package], false) {
                Ok(output) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: 'flatpak info' succeeded, output length: {} bytes", output.len());
                    Ok(output)
                },
                Err(_e) => {
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: 'flatpak info' failed: {}, trying remote-info...", _e);
                    // Package not installed, try remote-info with common remotes
                    // Try flathub first (most common remote)
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Trying 'flatpak remote-info flathub {}'...", package);
                    run_command("flatpak", &["remote-info", "flathub", &package], false)
                        .or_else(|_e| {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] load_single_package_detail: flathub failed: {}, trying other remotes...", _e);
                            // Try other common remotes
                            for remote in &["fedora", "gnome-nightly", "kdeapps", "elementary"] {
                                #[cfg(debug_assertions)]
                                eprintln!("[FLATPAK DEBUG] load_single_package_detail: Trying remote: {}", remote);
                                if let Ok(output) = run_command("flatpak", &["remote-info", remote, &package], false) {
                                    #[cfg(debug_assertions)]
                                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Success with remote: {}", remote);
                                    return Ok(output);
                                }
                            }
                            // Last resort: try to find remote by listing all remotes and trying each
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] load_single_package_detail: All common remotes failed, listing all remotes...");
                            if let Ok(remotes_output) = run_command("flatpak", &["remotes", "--columns=name"], false) {
                                #[cfg(debug_assertions)]
                                eprintln!("[FLATPAK DEBUG] load_single_package_detail: Got remotes list, length: {} bytes", remotes_output.len());
                                for remote in remotes_output.lines() {
                                    let remote = remote.trim();
                                    if !remote.is_empty() {
                                        #[cfg(debug_assertions)]
                                        eprintln!("[FLATPAK DEBUG] load_single_package_detail: Trying remote: {}", remote);
                                        if let Ok(output) = run_command("flatpak", &["remote-info", remote, &package], false) {
                                            #[cfg(debug_assertions)]
                                            eprintln!("[FLATPAK DEBUG] load_single_package_detail: Success with remote: {}", remote);
                                            return Ok(output);
                                        }
                                    }
                                }
                            } else {
                                #[cfg(debug_assertions)]
                                eprintln!("[FLATPAK DEBUG] load_single_package_detail: Failed to list remotes");
                            }
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] load_single_package_detail: ERROR - All attempts failed");
                            Err(format!("Failed to get flatpak info for {} (package may not be available in any remote)", package))
                        })
                }
            }?;
            
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_single_package_detail: Got info output, parsing...");
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
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Found description (first line): {}", description);
                    is_first_line = false;
                    continue;
                }
                is_first_line = false;
                
                // Parse version
                if line.starts_with("Version:") {
                    version = line.replace("Version:", "").trim().to_string();
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Found version: {}", version);
                }
                // Parse description (if not found on first line)
                else if line.starts_with("Description:") {
                    description = line.replace("Description:", "").trim().to_string();
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Found description (line): {}", description);
                }
                // Parse size - flatpak remote-info uses "Download:" and "Installed:" format
                else if line.starts_with("Download:") {
                    let download_size = line.replace("Download:", "").trim().to_string();
                    if size.is_empty() {
                        size = format!("Download: {}", download_size);
                    } else {
                        size = format!("{} / Download: {}", size, download_size);
                    }
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Found download size: {}", download_size);
                }
                else if line.starts_with("Installed:") {
                    let installed_size = line.replace("Installed:", "").trim().to_string();
                    if size.is_empty() {
                        size = format!("Installed: {}", installed_size);
                    } else {
                        size = format!("{} / Installed: {}", size, installed_size);
                    }
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Found installed size: {}", installed_size);
                }
                // Also handle old format for flatpak info (installed packages)
                else if line.starts_with("Installed size:") || line.starts_with("Download size:") {
                    let size_str = line.replace("Installed size:", "")
                        .replace("Download size:", "")
                        .trim().to_string();
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_single_package_detail: Found size (old format): {}", size_str);
                    if size.is_empty() {
                        size = size_str;
                    } else {
                        size = format!("{} / {}", size, size_str);
                    }
                }
            }
            
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_single_package_detail: Creating PackageDetail - name: {}, version: {}, size: {}", package, version, size);
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

async fn install_packages(package_names: Vec<String>, is_flatpak: bool) -> Result<String, String> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] install_packages() called with {} packages (flatpak: {})", package_names.len(), is_flatpak);
    
    if is_flatpak {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] install_packages: Installing flatpak packages...");
        // Install flatpak packages
        for (idx, package) in package_names.iter().enumerate() {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] install_packages: Installing package {}/{}: {}", idx + 1, package_names.len(), package);
            
            let mut cmd = TokioCommand::new("flatpak");
            cmd.arg("install");
            cmd.arg("-y");
            cmd.arg(package);
            
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] install_packages: Executing command: flatpak install -y {}", package);
            
            let output = cmd
                .output()
                .await
                .map_err(|e| {
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] install_packages: Command execution failed: {}", e);
                    format!("Failed to execute installation: {}", e)
                })?;
            
            #[cfg(debug_assertions)]
            {
                eprintln!("[FLATPAK DEBUG] install_packages: Command exit status: {:?}", output.status);
                eprintln!("[FLATPAK DEBUG] install_packages: Command success: {}", output.status.success());
                if !output.stdout.is_empty() {
                    eprintln!("[FLATPAK DEBUG] install_packages: Command stdout (first 500 chars): {}", 
                        String::from_utf8_lossy(&output.stdout).chars().take(500).collect::<String>());
                }
                if !output.stderr.is_empty() {
                    eprintln!("[FLATPAK DEBUG] install_packages: Command stderr (first 500 chars): {}", 
                        String::from_utf8_lossy(&output.stderr).chars().take(500).collect::<String>());
                }
            }
            
            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] install_packages: ERROR - Installation failed for package: {}", package);
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] install_packages: Error details: {}", stderr);
                return Err(format!("Installation failed: {}", stderr));
            }
            
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] install_packages: Successfully installed flatpak package: {}", package);
        }
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] install_packages: All flatpak packages installed successfully");
        Ok("Installation Complete!".to_string())
    } else {
        // Install apt packages using pkexec
        #[cfg(debug_assertions)]
        eprintln!("[DEBUG] Installing apt packages: {:?}", package_names);
        
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("apt");
        cmd.arg("install");
        cmd.arg("-y");
        for name in &package_names {
            cmd.arg(name);
        }
        
        #[cfg(debug_assertions)]
        {
            let cmd_str = format!("pkexec apt install -y {}", package_names.join(" "));
            eprintln!("[DEBUG] Executing command: {}", cmd_str);
        }
        
        // Ensure DISPLAY is set for GUI password dialog
        if let Ok(display) = std::env::var("DISPLAY") {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Setting DISPLAY environment variable: {}", display);
            cmd.env("DISPLAY", display);
        }
        if let Ok(xauth) = std::env::var("XAUTHORITY") {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Setting XAUTHORITY environment variable");
            cmd.env("XAUTHORITY", xauth);
        }
        if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Setting WAYLAND_DISPLAY environment variable: {}", wayland);
            cmd.env("WAYLAND_DISPLAY", wayland);
        }
        
        let output = cmd
            .output()
            .await
            .map_err(|e| {
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG] Command execution failed: {}", e);
                format!("Failed to execute installation: {}. Make sure polkit is installed.", e)
            })?;
        
        #[cfg(debug_assertions)]
        {
            eprintln!("[DEBUG] Command exit status: {:?}", output.status);
            if !output.stdout.is_empty() {
                eprintln!("[DEBUG] Command stdout: {}", String::from_utf8_lossy(&output.stdout));
            }
            if !output.stderr.is_empty() {
                eprintln!("[DEBUG] Command stderr: {}", String::from_utf8_lossy(&output.stderr));
            }
        }
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() == Some(126) || output.status.code() == Some(127) {
                #[cfg(debug_assertions)]
                eprintln!("[DEBUG] Authentication cancelled or failed");
                return Err("Authentication cancelled or failed. Please try again.".to_string());
            }
            #[cfg(debug_assertions)]
            eprintln!("[DEBUG] Installation failed");
            return Err(format!("Installation failed: {}", stderr));
        }
        
        #[cfg(debug_assertions)]
        eprintln!("[DEBUG] All apt packages installed successfully");
        Ok("Installation Complete!".to_string())
    }
}

