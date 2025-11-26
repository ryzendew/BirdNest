use iced::{
    alignment, executor, Color,
    widget::{button, checkbox, column, container, row, scrollable, text, text_input, Space},
    Application, Command, Element, Length, Pixels, Settings, Theme as IcedTheme, Padding,
};
use std::sync::Arc;
use tokio::sync::mpsc;
use std::collections::HashSet;
use futures::future;

use crate::package_manager::PackageManager;
use crate::flatpak::FlatpakManager;

mod theme;
mod styles;
pub mod install_dialog;
pub mod remove_dialog;
pub mod conflict_dialog;
pub mod pikman_install_dialog;

use theme::Theme as AppTheme;
use styles::{RoundedButtonStyle, RoundedContainerStyle, CustomScrollableStyle, YellowTextInputStyle, YellowCheckboxStyle};

#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub size: String,
    pub source: PackageSource,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PackageSource {
    Default,
    Aur,
    Fedora,
    Alpine,
}

impl PackageSource {
    pub fn as_str(&self) -> &str {
        match self {
            PackageSource::Default => "System",
            PackageSource::Aur => "AUR",
            PackageSource::Fedora => "Fedora",
            PackageSource::Alpine => "Alpine",
        }
    }
    
    pub fn badge_color(&self) -> Color {
        match self {
            PackageSource::Default => Color::from_rgb(0.5, 0.5, 0.5),
            PackageSource::Aur => Color::from_rgb(0.8, 0.5, 0.5), // Calm red - lighter for better contrast
            PackageSource::Fedora => Color::from_rgb(0.5, 0.65, 0.9), // Calm blue - lighter for better contrast
            PackageSource::Alpine => Color::from_rgb(0.4, 0.65, 0.85), // Calm cyan-blue
        }
    }
    
    pub fn badge_text_color(&self, is_dark_theme: bool) -> Color {
        if is_dark_theme {
            Color::WHITE
        } else {
            Color::BLACK
        }
    }
}

#[derive(Debug, Clone)]
pub struct PackageDetail {
    pub name: String,
    #[allow(dead_code)]
    pub version: String,
    #[allow(dead_code)]
    pub description: String,
    #[allow(dead_code)]
    pub size: String,
    pub is_flatpak: bool,
}

#[derive(Debug, Clone)]
pub struct FlatpakInfo {
    pub name: String,
    pub description: String,
    pub version: String,
    pub application: String,
}

// UpdateInfo struct removed - system updates handled by separate app

/// Try to find the PikaOS icon path from common system locations
fn find_pika_icon_path() -> Option<String> {
    let icon_paths = [
        "/usr/share/pixmaps/pika-logo.png",
        "/usr/share/pixmaps/pika-logo.svg",
        "/usr/share/pixmaps/pika-logo-duotone.svg",
        "/usr/share/icons/desktop-base/scalable/emblems/emblem-pika.svg",
    ];
    
    for path in &icon_paths {
        if std::path::Path::new(path).exists() {
            return Some(path.to_string());
        }
    }
    
    None
}

pub fn run() -> iced::Result {
    eprintln!("[DEBUG] gui::run() called - initializing GUI...");
    
    eprintln!("[DEBUG] Creating window settings...");
    
    let mut window_settings = iced::window::Settings {
        size: iced::Size::new(1200.0, 800.0),
        resizable: true,
        min_size: Some(iced::Size::new(800.0, 600.0)),
        ..Default::default()
    };
    
    // Set window icon if found
    // Note: Iced window icons typically need PNG format, not SVG
    // The desktop file will handle the SVG icon for the application launcher
    if let Some(icon_path) = find_pika_icon_path() {
        eprintln!("[DEBUG] Attempting to load PikaOS icon from: {}", icon_path);
        // Try to load as PNG first, then fall back to SVG if PNG loading fails
        if icon_path.ends_with(".png") {
            if let Ok(icon_image) = iced::window::icon::from_file(&icon_path) {
                window_settings.icon = Some(icon_image);
                eprintln!("[DEBUG] Window icon set successfully from PNG");
            }
        } else {
            // For SVG, we'll rely on the desktop file for the icon
            // Window icon might not support SVG directly
            eprintln!("[DEBUG] SVG icon found, will use desktop file for icon display");
        }
    } else {
        eprintln!("[DEBUG] PikaOS icon not found, using default");
    }
    
    eprintln!("[DEBUG] Creating application settings...");
    let settings = Settings {
        window: window_settings,
        default_text_size: Pixels(14.0),
        antialiasing: true,
        ..Default::default()
    };
    
    eprintln!("[DEBUG] Starting BirdNestGUI application...");
    match BirdNestGUI::run(settings) {
        Ok(_) => {
            eprintln!("[DEBUG] BirdNestGUI exited successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("[ERROR] BirdNestGUI failed: {:?}", e);
            Err(e)
        }
    }
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Message {
    SearchQueryChanged(String),
    Search,
    SearchResults(Vec<PackageInfo>),
    TogglePackage(String),
    InstallSelected,
    InstallPackage(String),
    RemovePackage(String),
    TabChanged(Tab),
    #[allow(dead_code)]
    OutputReceived(String),
    #[allow(dead_code)]
    ErrorReceived(String),
    #[allow(dead_code)]
    ClearOutput,
    LoadInstalledPackages,
    InstalledPackagesLoaded(Vec<PackageInfo>),
    ToggleInstalledPackage(String),
    RemoveSelectedPackages,
    LoadFlatpakApps,
    FlatpakAppsLoaded(Vec<FlatpakInfo>),
    RefreshLists,
    ThemeToggled,
    FlatpakSearchQueryChanged(String),
    FlatpakSearch,
    FlatpakSearchResults(Vec<FlatpakInfo>),
    FlatpakInstallPackage(String),
    FlatpakUpdateRepos,
    FlatpakUpgradeAll,
    FlatpakShowPackage(String),
    FlatpakClean,
    ShowInstallDialog(PackageDetail),
    HideInstallDialog,
    ConfirmInstall,
    PackageDetailLoaded(PackageDetail),
    ShowRemoveDialog(PackageDetail),
    HideRemoveDialog,
    ConfirmRemove,
    RemovePackageDetailLoaded(PackageDetail),
    RemovePackageDetailsLoaded(Vec<PackageDetail>),
    InstalledSearchQueryChanged(String),
    // Pikman messages
    PikmanSearchQueryChanged(String),
    PikmanSearch,
    PikmanSearchResults(Vec<PackageInfo>),
    PikmanFilterChanged(Option<String>),
    PikmanInstallSelected,
    PikmanInstallPackage(String),
    PikmanAutoremove,
    PikmanEnter(String),
    PikmanExport { package: String, name: Option<String> },
    PikmanInit { name: String, manager: Option<String> },
    PikmanLog,
    PikmanPurge(Vec<String>),
    PikmanRun { name: String, command: Vec<String> },
    PikmanUpgrades,
    PikmanUnexport { package: String, name: Option<String> },
    TogglePikmanPackage(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Search,
    Installed,
    Flatpak,
    Pikman,
}

#[derive(Debug)]
pub struct BirdNestGUI {
    current_tab: Tab,
    theme: AppTheme,
    search_query: String,
    search_results: Vec<PackageInfo>,
    selected_packages: HashSet<String>,
    installed_packages: Vec<PackageInfo>,
    installed_search_query: String,
    selected_installed: HashSet<String>,
    flatpak_apps: Vec<FlatpakInfo>,
    flatpak_search_query: String,
    flatpak_search_results: Vec<FlatpakInfo>,
    selected_flatpak: HashSet<String>,
    // Pikman state
    pikman_search_query: String,
    pikman_search_results: Vec<PackageInfo>,
    selected_pikman: HashSet<String>,
    pikman_filter: Option<String>, // "aur", "fedora", "alpine", None for default
    pikman_loading: bool,
    #[allow(dead_code)]
    install_dialog: Option<PackageDetail>,
    #[allow(dead_code)]
    remove_dialog: Option<PackageDetail>,
    #[allow(dead_code)]
    packages_to_remove: Vec<String>, // Store list of packages for batch removal
    output_log: Vec<String>,
    error_log: Vec<String>,
    #[allow(dead_code)]
    command_tx: Option<Arc<mpsc::UnboundedSender<GuiCommand>>>,
    border_radius: f32,
    // Loading state flags for lazy loading
    installed_loaded: bool,
    flatpak_loaded: bool,
    // Loading indicators
    installed_loading: bool,
    flatpak_loading: bool,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum GuiCommand {
    Search(String),
    Install(String),
    Remove(String),
    Update,
    Upgrade,
}

impl Application for BirdNestGUI {
    type Message = Message;
    type Theme = IcedTheme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(_flags: ()) -> (BirdNestGUI, Command<Message>) {
        eprintln!("[DEBUG] Application::new() called - initializing BirdNestGUI...");
        
        eprintln!("[DEBUG] Creating message channel...");
        let (tx, _rx) = mpsc::unbounded_channel();
        eprintln!("[DEBUG] Message channel created successfully");
        
        eprintln!("[DEBUG] Creating BirdNestGUI struct...");
        let gui = BirdNestGUI {
            current_tab: Tab::Search,
            theme: AppTheme::Dark,
            search_query: String::new(),
            search_results: Vec::new(),
            selected_packages: HashSet::new(),
            installed_packages: Vec::new(),
            installed_search_query: String::new(),
            selected_installed: HashSet::new(),
            flatpak_apps: Vec::new(),
            flatpak_search_query: String::new(),
            flatpak_search_results: Vec::new(),
            selected_flatpak: HashSet::new(),
            install_dialog: None,
            remove_dialog: None,
            packages_to_remove: Vec::new(),
            output_log: Vec::new(),
            error_log: Vec::new(),
            command_tx: Some(Arc::new(tx)),
            border_radius: 24.0, // EXTREME rounded for maximum bubble effect
            installed_loaded: false,
            flatpak_loaded: false,
            installed_loading: true, // Start loading immediately
            flatpak_loading: false,
            pikman_search_query: String::new(),
            pikman_search_results: Vec::new(),
            selected_pikman: HashSet::new(),
            pikman_filter: None,
            pikman_loading: false,
        };
        eprintln!("[DEBUG] BirdNestGUI struct created successfully");
        
        eprintln!("[DEBUG] Starting preload of installed packages...");
        let cmd = Command::perform(load_installed_packages(), Message::InstalledPackagesLoaded);
        eprintln!("[DEBUG] Preload command created, returning from Application::new()");
        
        (gui, cmd)
    }

    fn title(&self) -> String {
        String::from("BirdNest - Package Manager")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        eprintln!("[DEBUG] update() called with message: {:?}", std::mem::discriminant(&message));
        match message {
            Message::SearchQueryChanged(query) => {
                self.search_query = query;
                Command::none()
            }
            Message::Search => {
                let query = self.search_query.clone();
                if !query.is_empty() {
                    self.output_log.push(format!("Searching for: {}", query));
                    Command::perform(search_packages(query), Message::SearchResults)
                } else {
                    Command::none()
                }
            }
            Message::SearchResults(results) => {
                self.search_results = results;
                Command::none()
            }
            Message::InstallPackage(package) => {
                Command::perform(load_package_detail(package, false), |result| {
                    match result {
                        Ok(detail) => Message::PackageDetailLoaded(detail),
                        Err(e) => Message::ErrorReceived(format!("Failed to load package details: {}", e)),
                    }
                })
            }
            Message::PackageDetailLoaded(detail) => {
                // Launch separate install window as a separate process
                let package_names = vec![detail.name.clone()];
                let is_flatpak = detail.is_flatpak;
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("install-dialog");
                        for pkg in &package_names {
                            cmd.arg(pkg);
                        }
                        if is_flatpak {
                            cmd.arg("--flatpak");
                        }
                        let _ = cmd.spawn();
                    },
                    |_| Message::InstalledPackagesLoaded(Vec::new()),
                )
            }
            Message::ShowInstallDialog(detail) => {
                // Launch separate install window
                let package_names = if detail.name.contains("(and ") {
                    // Extract package names from selected_packages or selected_flatpak
                    if detail.is_flatpak {
                        self.selected_flatpak.iter().cloned().collect()
                    } else {
                        self.selected_packages.iter().cloned().collect()
                    }
                } else {
                    vec![detail.name.clone()]
                };
                
                if detail.is_flatpak {
                    self.selected_flatpak.clear();
                } else {
                    self.selected_packages.clear();
                }
                
                // Launch separate install window as a separate process
                let package_names_clone = package_names.clone();
                let is_flatpak = detail.is_flatpak;
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("install-dialog");
                        for pkg in &package_names_clone {
                            cmd.arg(pkg);
                        }
                        if is_flatpak {
                            cmd.arg("--flatpak");
                        }
                        let _ = cmd.spawn();
                    },
                    |_| Message::InstalledPackagesLoaded(Vec::new()),
                )
            }
            Message::HideInstallDialog => {
                // No longer needed with separate windows
                Command::none()
            }
            Message::ConfirmInstall => {
                // No longer needed - handled in separate window
                Command::none()
            }
            Message::RemovePackage(package) => {
                // Check if it's a Flatpak (contains a period, like org.example.App)
                let is_flatpak = package.contains('.');
                // Launch separate remove window as a separate process
                let packages = vec![package];
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("remove-dialog");
                        for pkg in &packages {
                            cmd.arg(pkg);
                        }
                        if is_flatpak {
                            cmd.arg("--flatpak");
                        }
                        let _ = cmd.spawn();
                    },
                    |_| Message::InstalledPackagesLoaded(Vec::new()),
                )
            }
            Message::RemovePackageDetailLoaded(_detail) => {
                // No longer needed - handled in separate window
                Command::none()
            }
            Message::ShowRemoveDialog(detail) => {
                // Launch separate remove window
                let package_names = if detail.name.contains("(and ") {
                    // Extract package names from selected_installed
                    self.selected_installed.iter().cloned().collect()
                } else {
                    vec![detail.name.clone()]
                };
                
                self.selected_installed.clear();
                
                // Launch separate remove window as a separate process
                let package_names_clone = package_names.clone();
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("remove-dialog");
                        for pkg in &package_names_clone {
                            cmd.arg(pkg);
                        }
                        let _ = cmd.spawn();
                    },
                    |_| Message::InstalledPackagesLoaded(Vec::new()),
                )
            }
            Message::HideRemoveDialog => {
                // No longer needed with separate windows
                Command::none()
            }
            Message::ConfirmRemove => {
                // No longer needed - handled in separate window
                Command::none()
            }
            Message::TabChanged(tab) => {
                eprintln!("[DEBUG] Tab changed to: {:?}", tab);
                self.current_tab = tab;
                match tab {
                    Tab::Flatpak => {
                        eprintln!("[DEBUG] Flatpak tab selected - loaded: {}, loading: {}", self.flatpak_loaded, self.flatpak_loading);
                        // Clear search results and query when switching to Flatpak tab
                        // This ensures we show installed apps by default after installation
                        self.flatpak_search_query.clear();
                        self.flatpak_search_results.clear();
                        self.selected_flatpak.clear();
                        // Always reload installed Flatpak apps to show newly installed packages
                        if !self.flatpak_loading {
                            eprintln!("[DEBUG] Starting to load Flatpak apps...");
                            self.flatpak_loading = true;
                            Command::perform(load_flatpak_apps(), |result| {
                                match result {
                                    Ok(apps) => {
                                        eprintln!("[DEBUG] Flatpak apps loaded successfully: {} apps", apps.len());
                                        Message::FlatpakAppsLoaded(apps)
                                    }
                                    Err(e) => {
                                        eprintln!("[ERROR] Failed to load Flatpak apps: {}", e);
                                        Message::ErrorReceived(e.to_string())
                                    }
                                }
                            })
                        } else {
                            eprintln!("[DEBUG] Flatpak apps already loading, skipping reload");
                            Command::none()
                        }
                    }
                    Tab::Installed => {
                        eprintln!("[DEBUG] Installed tab selected - loaded: {}, loading: {}", self.installed_loaded, self.installed_loading);
                        // If already loaded, show immediately. If loading, wait. Otherwise start loading.
                        if self.installed_loaded {
                            eprintln!("[DEBUG] Installed packages already loaded, showing immediately");
                            Command::none()
                        } else if self.installed_loading {
                            Command::none() // Already loading from startup
                        } else {
                            self.installed_loading = true;
                            Command::perform(load_installed_packages(), Message::InstalledPackagesLoaded)
                        }
                    }
                    _ => Command::none(),
                }
            }
            Message::InstalledPackagesLoaded(packages) => {
                eprintln!("[DEBUG] InstalledPackagesLoaded: {} packages loaded", packages.len());
                self.installed_packages = packages;
                self.installed_loaded = true;
                self.installed_loading = false;
                eprintln!("[DEBUG] Installed packages state updated - loaded: true, loading: false");
                Command::none()
            }
            Message::InstalledSearchQueryChanged(query) => {
                self.installed_search_query = query;
                Command::none()
            }
            Message::ToggleInstalledPackage(package) => {
                if self.selected_installed.contains(&package) {
                    self.selected_installed.remove(&package);
                } else {
                    self.selected_installed.insert(package);
                }
                Command::none()
            }
            Message::RemoveSelectedPackages => {
                let packages: Vec<String> = self.selected_installed.iter().cloned().collect();
                if packages.is_empty() {
                    return Command::none();
                }
                
                // Invalidate installed packages cache - will reload when user switches back to tab
                self.installed_loaded = false;
                invalidate_packages_cache();
                
                // Launch separate remove window as a separate process
                let packages_clone = packages.clone();
                self.selected_installed.clear();
                
                Command::perform(
                    async move {
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("remove-dialog");
                        for pkg in &packages_clone {
                            cmd.arg(pkg);
                        }
                        let _ = cmd.spawn();
                    },
                    |_| Message::InstalledPackagesLoaded(Vec::new()),
                )
            }
            Message::RemovePackageDetailsLoaded(_details) => {
                // No longer needed - handled in separate window
                Command::none()
            }
            Message::LoadFlatpakApps => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] Message::LoadFlatpakApps received");
                Command::perform(load_flatpak_apps(), |result| {
                    match result {
                        Ok(apps) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] LoadFlatpakApps: Success, loaded {} apps", apps.len());
                            Message::FlatpakAppsLoaded(apps)
                        },
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] LoadFlatpakApps: ERROR - {}", e);
                            Message::ErrorReceived(e.to_string())
                        },
                    }
                })
            }
            Message::FlatpakAppsLoaded(apps) => {
                eprintln!("[DEBUG] FlatpakAppsLoaded: {} apps loaded", apps.len());
                self.flatpak_apps = apps;
                self.flatpak_loaded = true;
                self.flatpak_loading = false;
                eprintln!("[DEBUG] Flatpak apps state updated - loaded: true, loading: false");
                Command::none()
            }
            Message::RefreshLists => {
                // Reset loaded flags to force reload
                self.installed_loaded = false;
                self.flatpak_loaded = false;
                // Invalidate cache to force fresh load
                invalidate_packages_cache();
                Command::perform(load_installed_packages(), Message::InstalledPackagesLoaded)
            }
            Message::LoadInstalledPackages => {
                if !self.installed_loading {
                    self.installed_loading = true;
                    self.installed_loaded = false;
                    Command::perform(load_installed_packages(), Message::InstalledPackagesLoaded)
                } else {
                    Command::none()
                }
            }
            Message::TogglePackage(package) => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] TogglePackage: Toggling package: '{}'", package);
                // Check if it's a flatpak or regular package
                if self.flatpak_search_results.iter().any(|f| f.application == package) {
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] TogglePackage: Package '{}' is a Flatpak (application ID)", package);
                    if self.selected_flatpak.contains(&package) {
                        self.selected_flatpak.remove(&package);
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] TogglePackage: Removed '{}' from selected_flatpak", package);
                    } else {
                        self.selected_flatpak.insert(package.clone());
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] TogglePackage: Added '{}' to selected_flatpak", package);
                    }
                } else {
                    if self.selected_packages.contains(&package) {
                        self.selected_packages.remove(&package);
                    } else {
                        self.selected_packages.insert(package);
                    }
                }
                Command::none()
            }
            Message::InstallSelected => {
                // Check if we're installing flatpaks or regular packages
                if !self.selected_flatpak.is_empty() {
                    let packages: Vec<String> = self.selected_flatpak.iter().cloned().collect();
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] InstallSelected: Installing {} Flatpak packages: {:?}", packages.len(), packages);
                    // Validate that all packages are valid Flatpak application IDs
                    for pkg in &packages {
                        if !pkg.contains('.') {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] InstallSelected: ERROR - Invalid Flatpak ID detected: '{}' (missing period)", pkg);
                            let error_msg = format!("Invalid Flatpak application ID: '{}'. Please select packages from the search results.", pkg);
                            return Command::perform(
                                async move { },
                                move |_| Message::ErrorReceived(error_msg),
                            );
                        }
                    }
                    self.selected_flatpak.clear();
                    // Launch install dialog for flatpaks
                    let packages_clone = packages.clone();
                    Command::perform(
                        async move {
                            use tokio::process::Command as TokioCommand;
                            let exe_path = std::env::current_exe()
                                .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                            let mut cmd = TokioCommand::new(&exe_path);
                            cmd.arg("install-dialog");
                            for pkg in &packages_clone {
                                #[cfg(debug_assertions)]
                                eprintln!("[FLATPAK DEBUG] InstallSelected: Adding package to install dialog: '{}'", pkg);
                                cmd.arg(pkg);
                            }
                            // Mark as flatpak by adding a special flag
                            cmd.arg("--flatpak");
                            let _ = cmd.spawn();
                        },
                        |_| Message::InstalledPackagesLoaded(Vec::new()),
                    )
                } else {
                    let packages: Vec<String> = self.selected_packages.iter().cloned().collect();
                    // Invalidate installed packages cache
                    self.installed_loaded = false;
                    invalidate_packages_cache();
                    if packages.is_empty() {
                        return Command::none();
                    }
                    self.selected_packages.clear();
                    // Launch install dialog for regular packages
                    let packages_clone = packages.clone();
                    Command::perform(
                        async move {
                            use tokio::process::Command as TokioCommand;
                            let exe_path = std::env::current_exe()
                                .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                            let mut cmd = TokioCommand::new(&exe_path);
                            cmd.arg("install-dialog");
                            for pkg in &packages_clone {
                                cmd.arg(pkg);
                            }
                            // Regular packages, not flatpak
                            let _ = cmd.spawn();
                        },
                        |_| Message::InstalledPackagesLoaded(Vec::new()),
                    )
                }
            }
            Message::ThemeToggled => {
                self.theme = match self.theme {
                    AppTheme::Light => AppTheme::Dark,
                    AppTheme::Dark => AppTheme::Light,
                };
                Command::none()
            }
            Message::FlatpakSearchQueryChanged(query) => {
                self.flatpak_search_query = query;
                Command::none()
            }
            Message::FlatpakSearch => {
                let query = self.flatpak_search_query.clone();
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] Message::FlatpakSearch received, query: '{}'", query);
                Command::perform(search_flatpak(query), |result| {
                    match result {
                        Ok(results) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] FlatpakSearch: Success, found {} results", results.len());
                            Message::FlatpakSearchResults(results)
                        },
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] FlatpakSearch: ERROR - {}", e);
                            Message::ErrorReceived(e.to_string())
                        },
                    }
                })
            }
            Message::FlatpakSearchResults(results) => {
                self.flatpak_search_results = results;
                Command::none()
            }
            Message::FlatpakInstallPackage(package) => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] Message::FlatpakInstallPackage received for package: '{}'", package);
                Command::perform(load_package_detail(package, true), |result| {
                    match result {
                        Ok(detail) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] FlatpakInstallPackage: Successfully loaded package detail: {}", detail.name);
                            Message::PackageDetailLoaded(detail)
                        },
                        Err(e) => {
                            #[cfg(debug_assertions)]
                            eprintln!("[FLATPAK DEBUG] FlatpakInstallPackage: ERROR - Failed to load package details: {}", e);
                            Message::ErrorReceived(format!("Failed to load package details: {}", e))
                        },
                    }
                })
            }
            Message::FlatpakUpdateRepos => {
                self.output_log.push("Updating Flatpak repositories...".to_string());
                Command::perform(update_flatpak_repos(), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::FlatpakUpgradeAll => {
                // Invalidate flatpak cache
                self.flatpak_loaded = false;
                self.output_log.push("Upgrading all Flatpaks...".to_string());
                Command::batch(vec![
                    Command::perform(upgrade_all_flatpaks(), |result| {
                        match result {
                            Ok(msg) => Message::OutputReceived(msg),
                            Err(e) => Message::ErrorReceived(e.to_string()),
                        }
                    }),
                    Command::perform(async move {
                        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
                        load_flatpak_apps().await
                    }, |result| {
                        match result {
                            Ok(apps) => Message::FlatpakAppsLoaded(apps),
                            Err(e) => Message::ErrorReceived(e.to_string()),
                        }
                    }),
                ])
            }
            Message::FlatpakShowPackage(package) => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] Message::FlatpakShowPackage received for package: '{}'", package);
                // Launch install dialog to show package info (it will show info even if not installing)
                let package_names = vec![package.clone()];
                Command::perform(
                    async move {
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] FlatpakShowPackage: Launching install dialog...");
                        use tokio::process::Command as TokioCommand;
                        let exe_path = std::env::current_exe()
                            .unwrap_or_else(|_| std::path::PathBuf::from("birdnest"));
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] FlatpakShowPackage: Executable path: {:?}", exe_path);
                        let mut cmd = TokioCommand::new(&exe_path);
                        cmd.arg("install-dialog");
                        for pkg in &package_names {
                            cmd.arg(pkg);
                        }
                        cmd.arg("--flatpak");
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] FlatpakShowPackage: Spawning process with args: install-dialog {} --flatpak", package_names.join(" "));
                        match cmd.spawn() {
                            Ok(_) => {
                                #[cfg(debug_assertions)]
                                eprintln!("[FLATPAK DEBUG] FlatpakShowPackage: Process spawned successfully");
                            },
                            Err(_e) => {
                                #[cfg(debug_assertions)]
                                eprintln!("[FLATPAK DEBUG] FlatpakShowPackage: ERROR - Failed to spawn process: {}", _e);
                            },
                        }
                    },
                    |_| Message::InstalledPackagesLoaded(Vec::new()),
                )
            }
            Message::FlatpakClean => {
                self.output_log.push("Cleaning Flatpak cache...".to_string());
                Command::perform(clean_flatpak(), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::OutputReceived(msg) => {
                self.output_log.push(msg);
                Command::none()
            }
            Message::ErrorReceived(msg) => {
                eprintln!("[ERROR] ErrorReceived: {}", msg);
                self.error_log.push(msg.clone());
                // Reset loading flags on error
                self.installed_loading = false;
                self.flatpak_loading = false;
                eprintln!("[DEBUG] Loading flags reset due to error");
                Command::none()
            }
            Message::ClearOutput => {
                self.output_log.clear();
                self.error_log.clear();
                Command::none()
            }
            // Pikman messages
            Message::PikmanSearchQueryChanged(query) => {
                self.pikman_search_query = query;
                Command::none()
            }
            Message::PikmanSearch => {
                if self.pikman_search_query.is_empty() {
                    Command::none()
                } else {
                    self.pikman_loading = true;
                    let query = self.pikman_search_query.clone();
                    let filter = self.pikman_filter.clone();
                    Command::perform(pikman_search(query, filter), |result| {
                        match result {
                            Ok(results) => Message::PikmanSearchResults(results),
                            Err(e) => Message::ErrorReceived(e.to_string()),
                        }
                    })
                }
            }
            Message::PikmanSearchResults(results) => {
                self.pikman_search_results = results;
                self.pikman_loading = false;
                Command::none()
            }
            Message::PikmanFilterChanged(filter) => {
                self.pikman_filter = filter;
                Command::none()
            }
            Message::TogglePikmanPackage(package) => {
                if self.selected_pikman.contains(&package) {
                    self.selected_pikman.remove(&package);
                } else {
                    self.selected_pikman.insert(package);
                }
                Command::none()
            }
            Message::PikmanInstallSelected => {
                let packages: Vec<String> = self.selected_pikman.iter().cloned().collect();
                if packages.is_empty() {
                    Command::none()
                } else {
                    use crate::gui::pikman_install_dialog::PikmanInstallDialog;
                    std::process::Command::new("birdnest")
                        .arg("install-dialog")
                        .args(&packages)
                        .spawn()
                        .ok();
                    // Also launch the pikman install dialog
                    let packages_clone = packages.clone();
                    std::thread::spawn(move || {
                        let _ = PikmanInstallDialog::run_separate_window(packages_clone);
                    });
                    self.selected_pikman.clear();
                    Command::none()
                }
            }
            Message::PikmanInstallPackage(package) => {
                use crate::gui::pikman_install_dialog::PikmanInstallDialog;
                let packages = vec![package];
                std::thread::spawn(move || {
                    let _ = PikmanInstallDialog::run_separate_window(packages);
                });
                Command::none()
            }
            Message::PikmanAutoremove => {
                self.output_log.push("Running pikman autoremove...".to_string());
                Command::perform(pikman_autoremove(), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanEnter(name) => {
                // This would need a dialog for container name input
                self.output_log.push(format!("Entering container: {}", name));
                Command::perform(pikman_enter(name), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanExport { package, name } => {
                self.output_log.push(format!("Exporting package: {}", package));
                Command::perform(pikman_export(package, name), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanInit { name, manager } => {
                self.output_log.push(format!("Initializing container: {}", name));
                Command::perform(pikman_init(name, manager), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanLog => {
                Command::perform(pikman_log(), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanPurge(packages) => {
                self.output_log.push(format!("Purging packages: {:?}", packages));
                Command::perform(pikman_purge(packages), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanRun { name, command } => {
                self.output_log.push(format!("Running command in container: {}", name));
                Command::perform(pikman_run(name, command), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanUpgrades => {
                Command::perform(pikman_upgrades(), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
            Message::PikmanUnexport { package, name } => {
                self.output_log.push(format!("Unexporting package: {}", package));
                Command::perform(pikman_unexport(package, name), |result| {
                    match result {
                        Ok(msg) => Message::OutputReceived(msg),
                        Err(e) => Message::ErrorReceived(e.to_string()),
                    }
                })
            }
        }
    }

    fn view(&self) -> Element<Message> {
        let theme = self.theme;
        let content = match self.current_tab {
            Tab::Search => self.view_search(),
            Tab::Installed => self.view_installed(),
            Tab::Flatpak => self.view_flatpak(),
            Tab::Pikman => self.view_pikman(),
        };

        let main_content = column![
            self.view_header(),
            self.view_tabs(),
            content,
        ]
        .spacing(15)
        .padding(Padding::new(24.0));
        
        // Main background - dark, no elevation
        container(main_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                radius: 0.0, // No radius for main background
                background: Some(theme.background()), // Use dark background, not card
                elevation: 0.0, // No elevation for main background
            })))
            .padding(Padding::new(16.0))
            .into()
    }

    fn theme(&self) -> IcedTheme {
        match self.theme {
            AppTheme::Light => IcedTheme::Light,
            AppTheme::Dark => IcedTheme::Dark,
        }
    }
}

impl BirdNestGUI {
    fn view_header(&self) -> Element<Message> {
        // Header removed - no longer needed
        Element::from(Space::with_height(Length::Fixed(0.0)))
    }

    fn view_tabs(&self) -> Element<Message> {
        let theme = self.theme;
        container(
            row![
                self.tab_button("Search", Tab::Search),
                self.tab_button("Installed", Tab::Installed),
                self.tab_button("Flatpak", Tab::Flatpak),
                self.tab_button("Pikman", Tab::Pikman),
                Space::with_width(Length::Fill),
                button(if theme == AppTheme::Dark { "Light" } else { "Dark" })
                    .on_press(Message::ThemeToggled)
                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                        is_primary: false,
                        radius: self.border_radius,
                        primary_color: theme.primary(),
                        text_color: Color::WHITE,
                        background_color: theme.background(),
                    })))
                    .padding(Padding::new(14.0)),
            ]
            .spacing(12)
            .align_items(alignment::Alignment::Center)
            .padding(Padding::new(16.0))
        )
        .width(Length::Fill)
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: self.border_radius,
            background: Some(theme.card_background()),
            elevation: 1.0, // Bubble effect for tab bar
        })))
        .into()
    }

    fn tab_button(&self, label: &str, tab: Tab) -> Element<Message> {
        let theme = self.theme;
        let is_active = self.current_tab == tab;
        button(text(label)
            .size(if is_active { 20.0 } else { 16.0 }))
            .on_press(Message::TabChanged(tab))
            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                is_primary: is_active,
                radius: self.border_radius,
                primary_color: theme.primary(),
                text_color: if is_active { Color::BLACK } else { Color::WHITE },
                background_color: theme.background(),
            })))
            .padding(Padding::new(14.0))
            .into()
    }

    fn view_search(&self) -> Element<Message> {
        let theme = self.theme;
        
        // Search section with rounded container
        let search_section = container(
            column![
                // Search bar with Search button
                row![
                    text_input("Search packages...", &self.search_query)
                        .on_input(Message::SearchQueryChanged)
                        .on_submit(Message::Search)
                        .padding(Padding::new(12.0))
                        .width(Length::Fill)
                        .style(iced::theme::TextInput::Custom(Box::new(YellowTextInputStyle {
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            background_color: theme.background(),
                            text_color: Color::BLACK,
                        }))),
                    button(text("Search")
                        .size(16.0))
                        .on_press(Message::Search)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: true,
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: Color::BLACK,
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(16.0)),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(10.0)),
                // Install button row
                row![
                    Space::with_width(Length::Fill),
                    {
                        if !self.search_results.is_empty() {
                            if self.selected_packages.is_empty() {
                                Element::from(button("Select packages to install")
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::WHITE,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            } else {
                                Element::from(button(text(format!("Install {} Selected", self.selected_packages.len()))
                                    .size(16.0))
                                    .on_press(Message::InstallSelected)
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: true,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::BLACK,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            }
                        } else {
                            Element::from(Space::with_width(Length::Fixed(0.0)))
                        }
                    },
                ]
                .spacing(8)
                .width(Length::Fill)
                .align_items(alignment::Alignment::Center),
            ]
            .spacing(16)
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0))
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: self.border_radius,
            background: Some(theme.card_background()),
            elevation: 1.5, // Elevated search section
        })));

        // Search results or empty state
        let content_section = if self.search_results.is_empty() {
            container(
                text(if self.search_query.is_empty() {
                    "Enter a search query to find packages"
                } else {
                    "No packages found"
                })
                .size(16)
                .style(iced::theme::Text::Color(theme.text()))
            )
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            // Show search results
            Element::from(
                container(
                    scrollable(
                        column(
                            self.search_results
                                .iter()
                                .map(|pkg| {
                                    let is_selected = self.selected_packages.contains(&pkg.name);
                                    button(
                                        container(
                                            row![
                                                checkbox("", is_selected)
                                                    .style(iced::theme::Checkbox::Custom(Box::new(YellowCheckboxStyle {
                                                        radius: 4.0,
                                                        primary_color: theme.primary(),
                                                    }))),
                                                column![
                                                    text(&pkg.name)
                                                        .size(if is_selected { 26.0 } else { 24.0 })
                                                        .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                        .width(Length::Fill),
                                                    {
                                                        if !pkg.description.is_empty() {
                                                            let display_text = if pkg.description.len() > 120 {
                                                                format!("{}...", &pkg.description[..120])
                                                            } else {
                                                                pkg.description.clone()
                                                            };
                                                            Element::from(text(&display_text)
                                                                .size(if is_selected { 14.0 } else { 12.0 })
                                                                .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                                .width(Length::Fill))
                                                        } else {
                                                            Element::from(Space::with_height(Length::Shrink))
                                                        }
                                                    },
                                                    {
                                                        let mut info_row = row![].spacing(12).width(Length::Fill);
                                                        if !pkg.version.is_empty() {
                                                            info_row = info_row.push(
                                                                Element::from(text(format!("Version: {}", pkg.version))
                                                                    .size(if is_selected { 13.0 } else { 11.0 })
                                                                    .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE })))
                                                            );
                                                        }
                                                        Element::from(info_row)
                                                    },
                                                ]
                                                .spacing(4)
                                                .width(Length::Fill),
                                            ]
                                            .spacing(12)
                                            .align_items(alignment::Alignment::Center)
                                            .padding(Padding::new(12.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                            radius: self.border_radius,
                                            background: if is_selected {
                                                Some(theme.primary().into())
                                            } else {
                                                Some(theme.card_background())
                                            },
                                            elevation: 1.0, // Subtle bubble effect for each package card
                                        })))
                                    )
                                    .on_press(Message::TogglePackage(pkg.name.clone()))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::WHITE,
                                        background_color: Color::TRANSPARENT,
                                    })))
                                    .into()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .spacing(6)
                        .padding(10)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                        background_color: theme.background(),
                        border_radius: self.border_radius,
                    })))
                )
                .width(Length::Fill)
                .height(Length::Fill)
            )
        };

        column![
            search_section,
            Space::with_height(Length::Fixed(16.0)),
            content_section,
        ]
        .spacing(20)
        .padding(Padding::new(24.0))
        .into()
    }

    fn view_installed(&self) -> Element<Message> {
        let theme = self.theme;
        
        // Search section with rounded container
        let search_section = container(
            column![
                // Search bar with Search button
                row![
                    text_input("Search installed packages...", &self.installed_search_query)
                        .on_input(Message::InstalledSearchQueryChanged)
                        .padding(Padding::new(12.0))
                        .width(Length::Fill)
                        .style(iced::theme::TextInput::Custom(Box::new(YellowTextInputStyle {
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            background_color: theme.background(),
                            text_color: Color::BLACK,
                        }))),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(10.0)),
                // Remove button row
                row![
                    Space::with_width(Length::Fill),
                    {
                        if !self.installed_packages.is_empty() {
                            if self.selected_installed.is_empty() {
                                Element::from(button("Select packages to remove")
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::WHITE,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            } else {
                                Element::from(button(text(format!("Remove {} Selected", self.selected_installed.len())))
                                    .on_press(Message::RemoveSelectedPackages)
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: true,
                                        radius: self.border_radius,
                                        primary_color: theme.danger(),
                                        text_color: Color::WHITE,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            }
                        } else {
                            Element::from(Space::with_width(Length::Fixed(0.0)))
                        }
                    },
                ]
                .spacing(8)
                .width(Length::Fill)
                .align_items(alignment::Alignment::Center),
            ]
            .spacing(16)
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0))
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: self.border_radius,
            background: Some(theme.card_background()),
            elevation: 1.5, // Elevated search section
        })));

        // Content section
        let content_section = if self.installed_loading {
            container(
                text("Loading installed packages...")
                    .size(16)
                    .style(iced::theme::Text::Color(theme.text()))
            )
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else if self.installed_packages.is_empty() {
            container(
                text("No packages installed")
                    .size(16)
                    .style(iced::theme::Text::Color(theme.text()))
            )
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            // Filter packages based on search query
            let filtered_packages: Vec<&PackageInfo> = if self.installed_search_query.is_empty() {
                self.installed_packages.iter().collect()
            } else {
                let query_lower = self.installed_search_query.to_lowercase();
                self.installed_packages
                    .iter()
                    .filter(|pkg| {
                        pkg.name.to_lowercase().contains(&query_lower) ||
                        (!pkg.description.is_empty() && pkg.description.to_lowercase().contains(&query_lower)) ||
                        pkg.version.to_lowercase().contains(&query_lower)
                    })
                    .collect()
            };

            // Show package list
            Element::from(
                container(
                    scrollable(
                        column(
                            filtered_packages
                                .iter()
                                .map(|pkg| {
                                    let is_selected = self.selected_installed.contains(&pkg.name);
                                    button(
                                        container(
                                            row![
                                                checkbox("", is_selected)
                                                    .style(iced::theme::Checkbox::Custom(Box::new(YellowCheckboxStyle {
                                                        radius: 4.0,
                                                        primary_color: theme.primary(),
                                                    }))),
                                                column![
                                                    text(&pkg.name)
                                                        .size(if is_selected { 26.0 } else { 24.0 })
                                                        .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                        .width(Length::Fill),
                                                    {
                                                        if !pkg.description.is_empty() {
                                                            let display_text = if pkg.description.len() > 120 {
                                                                format!("{}...", &pkg.description[..120])
                                                            } else {
                                                                pkg.description.clone()
                                                            };
                                                            Element::from(text(&display_text)
                                                                .size(if is_selected { 14.0 } else { 12.0 })
                                                                .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                                .width(Length::Fill))
                                                        } else {
                                                            Element::from(Space::with_height(Length::Shrink))
                                                        }
                                                    },
                                                    {
                                                        let mut info_row = row![].spacing(12).width(Length::Fill);
                                                        if !pkg.version.is_empty() {
                                                            info_row = info_row.push(
                                                                Element::from(text(format!("Version: {}", pkg.version))
                                                                    .size(if is_selected { 13.0 } else { 11.0 })
                                                                    .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE })))
                                                            );
                                                        }
                                                        Element::from(info_row)
                                                    },
                                                ]
                                                .spacing(4)
                                                .width(Length::Fill),
                                            ]
                                            .spacing(12)
                                            .align_items(alignment::Alignment::Center)
                                            .padding(Padding::new(12.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                            radius: self.border_radius,
                                            background: if is_selected {
                                                Some(theme.primary().into())
                                            } else {
                                                Some(theme.card_background())
                                            },
                                            elevation: 1.0, // Subtle bubble effect for each package card
                                        })))
                                    )
                                    .on_press(Message::ToggleInstalledPackage(pkg.name.clone()))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::WHITE,
                                        background_color: Color::TRANSPARENT,
                                    })))
                                    .into()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .spacing(6)
                        .padding(10)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                        background_color: theme.background(),
                        border_radius: self.border_radius,
                    })))
                )
                .width(Length::Fill)
                .height(Length::Fill)
            )
        };

        column![
            search_section,
            Space::with_height(Length::Fixed(16.0)),
            content_section,
        ]
        .spacing(20)
        .padding(Padding::new(24.0))
        .into()
    }


    fn view_flatpak(&self) -> Element<Message> {
        let theme = self.theme;
        
        // Search section with rounded container
        let search_section = container(
            column![
                // Search bar with Search button
                row![
                    text_input("Search Flatpak packages...", &self.flatpak_search_query)
                        .on_input(Message::FlatpakSearchQueryChanged)
                        .on_submit(Message::FlatpakSearch)
                        .padding(Padding::new(12.0))
                        .width(Length::Fill)
                        .style(iced::theme::TextInput::Custom(Box::new(YellowTextInputStyle {
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            background_color: theme.background(),
                            text_color: Color::BLACK,
                        }))),
                    button("Search")
                        .on_press(Message::FlatpakSearch)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: true,
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: Color::BLACK,
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(16.0)),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(10.0)),
                // Action buttons and Install button row
                row![
                    button("Update Repos")
                        .on_press(Message::FlatpakUpdateRepos)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: false,
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: Color::WHITE,
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                    button("Upgrade All")
                        .on_press(Message::FlatpakUpgradeAll)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: false,
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: Color::WHITE,
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                    button("Clean")
                        .on_press(Message::FlatpakClean)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: false,
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: Color::WHITE,
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                    Space::with_width(Length::Fill),
                    {
                        if !self.flatpak_search_results.is_empty() {
                            if self.selected_flatpak.is_empty() {
                                Element::from(button("Select packages to install")
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::WHITE,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            } else {
                                Element::from(button(text(format!("Install {} Selected", self.selected_flatpak.len()))
                                    .size(16.0))
                                    .on_press(Message::InstallSelected)
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: true,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::BLACK,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            }
                        } else {
                            Element::from(Space::with_width(Length::Fixed(0.0)))
                        }
                    },
                ]
                .spacing(8)
                .width(Length::Fill)
                .align_items(alignment::Alignment::Center),
            ]
            .spacing(16)
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0))
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: self.border_radius,
            background: Some(theme.card_background()),
            elevation: 1.5, // Elevated search section
        })));

        // Content section
        let content_section = if self.flatpak_loading {
            container(
                text("Loading Flatpak applications...")
                    .size(16)
                    .style(iced::theme::Text::Color(theme.text()))
            )
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else if !self.flatpak_search_results.is_empty() {
            // Show search results
            Element::from(
                container(
                    scrollable(
                        column(
                            self.flatpak_search_results
                                .iter()
                                .map(|fpkg| {
                                    let is_selected = self.selected_flatpak.contains(&fpkg.application);
                                    let pkg_name = fpkg.application.clone();
                                    button(
                                        container(
                                            row![
                                                checkbox("", is_selected)
                                                    .style(iced::theme::Checkbox::Custom(Box::new(YellowCheckboxStyle {
                                                        radius: 4.0,
                                                        primary_color: theme.primary(),
                                                    }))),
                                                column![
                                                    text(&fpkg.name)
                                                        .size(if is_selected { 26.0 } else { 24.0 })
                                                        .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                        .width(Length::Fill),
                                                    {
                                                        if !fpkg.description.is_empty() {
                                                            let display_text = if fpkg.description.len() > 120 {
                                                                format!("{}...", &fpkg.description[..120])
                                                            } else {
                                                                fpkg.description.clone()
                                                            };
                                                            Element::from(text(&display_text)
                                                                .size(if is_selected { 14.0 } else { 12.0 })
                                                                .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                                .width(Length::Fill))
                                                        } else {
                                                            Element::from(Space::with_height(Length::Shrink))
                                                        }
                                                    },
                                                    {
                                                        let mut info_row = row![].spacing(12).width(Length::Fill);
                                                        if !fpkg.version.is_empty() {
                                                            info_row = info_row.push(
                                                                Element::from(text(format!("Version: {}", fpkg.version))
                                                                    .size(11)
                                                                    .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE })))
                                                            );
                                                        }
                                                        if !fpkg.application.is_empty() {
                                                            info_row = info_row.push(
                                                                Element::from(text(format!("ID: {}", fpkg.application))
                                                                    .size(if is_selected { 13.0 } else { 11.0 })
                                                                    .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE })))
                                                            );
                                                        }
                                                        Element::from(info_row)
                                                    },
                                                ]
                                                .spacing(4)
                                                .width(Length::Fill),
                                            ]
                                            .spacing(12)
                                            .align_items(alignment::Alignment::Center)
                                            .padding(Padding::new(12.0))
                                        )
                                        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                            radius: self.border_radius,
                                            background: if is_selected {
                                                Some(theme.primary().into())
                                            } else {
                                                Some(theme.card_background())
                                            },
                                            elevation: 1.0, // Subtle bubble effect for each package card
                                        })))
                                    )
                                    .on_press(Message::TogglePackage(pkg_name.clone()))
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::WHITE,
                                        background_color: Color::TRANSPARENT,
                                    })))
                                    .into()
                                })
                                .collect::<Vec<_>>(),
                        )
                        .spacing(6)
                        .padding(10)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                        background_color: theme.background(),
                        border_radius: self.border_radius,
                    })))
                )
                .width(Length::Fill)
                .height(Length::Fill)
            )
        } else if !self.flatpak_apps.is_empty() {
                Element::from(
                    column![
                        container(
                            text(format!("{} Flatpak applications installed", self.flatpak_apps.len()))
                                .size(16)
                                .style(iced::theme::Text::Color(theme.text()))
                        )
                        .padding(Padding::new(16.0))
                        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                            radius: self.border_radius,
                            background: Some(theme.surface()),
                            elevation: 1.0,
                        })))
                        .width(Length::Fill),
                        scrollable(
                            column(
                                self.flatpak_apps
                                    .iter()
                                    .map(|app| {
                                        let app_id = app.application.clone();
                                        container(
                                            row![
                                                text(&app.name)
                                                    .size(16)
                                                    .style(iced::theme::Text::Color(theme.text())),
                                                Space::with_width(Length::Fill),
                                                button("Info")
                                                    .on_press(Message::FlatpakShowPackage(app_id.clone()))
                                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                        is_primary: false,
                                                        radius: self.border_radius,
                                                        primary_color: theme.primary(),
                                                        text_color: Color::WHITE,
                                                        background_color: theme.background(),
                                                    })))
                                                    .padding(Padding::new(14.0)),
                                                button("Remove")
                                                    .on_press(Message::RemovePackage(app_id.clone()))
                                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                        is_primary: false,
                                                        radius: self.border_radius,
                                                        primary_color: theme.danger(),
                                                        text_color: theme.danger(),
                                                        background_color: theme.background(),
                                                    })))
                                                    .padding(Padding::new(14.0)),
                                            ]
                                            .spacing(10)
                                            .align_items(alignment::Alignment::Center)
                                        )
                                        .padding(Padding::new(16.0))
                                        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                            radius: self.border_radius,
                                            background: Some(theme.surface()),
                                            elevation: 1.0, // Subtle bubble effect for package cards
                                        })))
                                        .width(Length::Fill)
                                        .into()
                                    })
                                    .collect::<Vec<Element<Message>>>(),
                            )
                            .spacing(10)
                            .padding(10)
                        )
                        .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                            background_color: theme.background(),
                            border_radius: self.border_radius,
                        })))
                        .height(Length::Fill),
                    ]
                    .spacing(15)
                )
        } else {
            container(
                text("No Flatpak applications installed. Use search to find and install packages.")
                    .size(16)
                    .style(iced::theme::Text::Color(theme.text()))
            )
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
        };

        column![
            search_section,
            Space::with_height(Length::Fixed(16.0)),
            content_section,
        ]
        .spacing(20)
        .padding(Padding::new(24.0))
        .into()
    }

    fn view_pikman(&self) -> Element<Message> {
        let theme = self.theme;
        
        // Responsive layout - stack vertically on small screens
        let _is_small_screen = false; // Could be made dynamic based on window size
        
        let search_section = container(
            column![
                // Search bar with Search button
                row![
                    text_input("Search packages...", &self.pikman_search_query)
                        .on_input(Message::PikmanSearchQueryChanged)
                        .on_submit(Message::PikmanSearch)
                        .padding(Padding::new(12.0))
                        .width(Length::Fill)
                        .style(iced::theme::TextInput::Custom(Box::new(YellowTextInputStyle {
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            background_color: theme.background(),
                            text_color: Color::BLACK,
                        }))),
                    button("Search")
                        .on_press(Message::PikmanSearch)
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: true,
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: Color::BLACK,
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(16.0)),
                ]
                .spacing(10)
                .width(Length::Fill),
                Space::with_height(Length::Fixed(10.0)),
                // Filter buttons and Install button row
                row![
                    text("Source:")
                        .size(14)
                        .style(iced::theme::Text::Color(theme.text())),
                    Space::with_width(Length::Fixed(10.0)),
                    button(if self.pikman_filter.is_none() { "✓ Default" } else { "Default" })
                        .on_press(Message::PikmanFilterChanged(None))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: self.pikman_filter.is_none(),
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: if self.pikman_filter.is_none() { Color::BLACK } else { Color::WHITE },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                    button(if self.pikman_filter.as_ref().map(|s| s == "aur").unwrap_or(false) { "✓ AUR" } else { "AUR" })
                        .on_press(Message::PikmanFilterChanged(Some("aur".to_string())))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: self.pikman_filter.as_ref().map(|s| s == "aur").unwrap_or(false),
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: if self.pikman_filter.as_ref().map(|s| s == "aur").unwrap_or(false) { Color::BLACK } else { Color::WHITE },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                    button(if self.pikman_filter.as_ref().map(|s| s == "fedora").unwrap_or(false) { "✓ Fedora" } else { "Fedora" })
                        .on_press(Message::PikmanFilterChanged(Some("fedora".to_string())))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: self.pikman_filter.as_ref().map(|s| s == "fedora").unwrap_or(false),
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: if self.pikman_filter.as_ref().map(|s| s == "fedora").unwrap_or(false) { Color::BLACK } else { Color::WHITE },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                    button(if self.pikman_filter.as_ref().map(|s| s == "alpine").unwrap_or(false) { "✓ Alpine" } else { "Alpine" })
                        .on_press(Message::PikmanFilterChanged(Some("alpine".to_string())))
                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                            is_primary: self.pikman_filter.as_ref().map(|s| s == "alpine").unwrap_or(false),
                            radius: self.border_radius,
                            primary_color: theme.primary(),
                            text_color: if self.pikman_filter.as_ref().map(|s| s == "alpine").unwrap_or(false) { Color::BLACK } else { Color::WHITE },
                            background_color: theme.background(),
                        })))
                        .padding(Padding::new(14.0)),
                    Space::with_width(Length::Fill),
                    {
                        if !self.pikman_search_results.is_empty() {
                            if self.selected_pikman.is_empty() {
                                Element::from(button("Select packages to install")
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: false,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::WHITE,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            } else {
                                Element::from(button(text(format!("Install {} Selected", self.selected_pikman.len()))
                                    .size(16.0))
                                    .on_press(Message::PikmanInstallSelected)
                                    .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                        is_primary: true,
                                        radius: self.border_radius,
                                        primary_color: theme.primary(),
                                        text_color: Color::BLACK,
                                        background_color: theme.background(),
                                    })))
                                    .padding(Padding::new(10.0)))
                            }
                        } else {
                            Element::from(Space::with_width(Length::Fixed(0.0)))
                        }
                    },
                ]
                .spacing(8)
                .width(Length::Fill)
                .align_items(alignment::Alignment::Center),
            ]
            .spacing(16)
        )
        .width(Length::Fill)
        .padding(Padding::new(20.0))
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: self.border_radius,
            background: Some(theme.card_background()),
            elevation: 1.5, // Elevated search section
        })));

        // Search results or commands section
        let content_section = if self.pikman_loading {
            container(
                text("Searching packages...")
                    .size(16)
                    .style(iced::theme::Text::Color(theme.text()))
            )
            .width(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else if !self.pikman_search_results.is_empty() {
            // Show search results
            Element::from(
                container(
                    scrollable(
                            column(
                                self.pikman_search_results
                                    .iter()
                                .map(|pkg| {
                                    let is_selected = self.selected_pikman.contains(&pkg.name);
                                    button(
                                            container(
                                                row![
                                                    checkbox("", is_selected)
                                                        .style(iced::theme::Checkbox::Custom(Box::new(YellowCheckboxStyle {
                                                            radius: 4.0,
                                                            primary_color: theme.primary(),
                                                        }))),
                                                    column![
                                                        row![
                                                            text(&pkg.name)
                                                                .size(if is_selected { 26.0 } else { 24.0 })
                                                                .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                                .width(Length::Fill),
                                                            // Source badge
                                                            container(
                                                                text(pkg.source.as_str())
                                                                    .size(10)
                                                                    .style(iced::theme::Text::Color(pkg.source.badge_text_color(matches!(self.theme, AppTheme::Dark))))
                                                            )
                                                            .padding(Padding::new(6.0))
                                        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                            radius: 4.0,
                                            background: Some(pkg.source.badge_color()),
                                            elevation: 0.5,
                                        }))),
                                                        ]
                                                        .spacing(8)
                                                        .width(Length::Fill)
                                                        .align_items(alignment::Alignment::Center),
                                                        text(&pkg.description)
                                                            .size(if is_selected { 14.0 } else { 12.0 })
                                                            .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                            .width(Length::Fill),
                                                        {
                                                            let mut info_row = row![].spacing(12).width(Length::Fill);
                                                            if !pkg.version.is_empty() {
                                                                info_row = info_row.push(
                                                                    text(format!("Version: {}", pkg.version))
                                                                        .size(11)
                                                                        .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                                );
                                                            }
                                                            if !pkg.size.is_empty() {
                                                                info_row = info_row.push(
                                                                    text(format!("Size: {}", pkg.size))
                                                                        .size(if is_selected { 13.0 } else { 11.0 })
                                                                        .style(iced::theme::Text::Color(if is_selected { Color::BLACK } else { Color::WHITE }))
                                                                );
                                                            }
                                                            info_row
                                                        },
                                                    ]
                                                    .spacing(4)
                                                    .width(Length::Fill),
                                                ]
                                                .spacing(16)
                                                .align_items(alignment::Alignment::Center)
                                                .padding(Padding::new(16.0))
                                            )
                                            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                                radius: self.border_radius,
                                                background: if is_selected {
                                                    Some(theme.primary().into())
                                                } else {
                                                    Some(theme.surface())
                                                },
                                                elevation: 1.0, // Subtle bubble effect for package cards
                                            })))
                                        )
                                        .on_press(Message::TogglePikmanPackage(pkg.name.clone()))
                                        .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                            is_primary: false,
                                            radius: self.border_radius,
                                            primary_color: theme.primary(),
                                            text_color: Color::WHITE,
                                            background_color: Color::TRANSPARENT,
                                        })))
                                        .into()
                                    })
                                    .collect::<Vec<_>>(),
                            )
                            .spacing(8)
                            .padding(16)
                        )
                        .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                            background_color: theme.background(),
                            border_radius: self.border_radius,
                        })))
                )
                .padding(Padding::new(16.0))
                    .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                        radius: self.border_radius,
                        background: Some(theme.surface()),
                        elevation: 1.0,
                    })))
                    .width(Length::Fill)
            )
        } else {
            // Show pikman commands
            Element::from(
                container(
                    scrollable(
                        column![
                            text("Pikman Commands")
                                .size(18)
                                .style(iced::theme::Text::Color(Color::WHITE)),
                            Space::with_height(Length::Fixed(15.0)),
                            // Container management
                            container(
                                column![
                                    text("Container Management")
                                        .size(16)
                                        .style(iced::theme::Text::Color(theme.text())),
                                    Space::with_height(Length::Fixed(10.0)),
                                    row![
                                        button("Init Container")
                                            .on_press(Message::PikmanInit { name: String::new(), manager: None })
                                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                is_primary: false,
                                                radius: self.border_radius,
                                                primary_color: theme.primary(),
                                                text_color: Color::WHITE,
                                                background_color: theme.background(),
                                            })))
                                            .padding(Padding::new(14.0)),
                                        button("Enter Container")
                                            .on_press(Message::PikmanEnter(String::new()))
                                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                is_primary: false,
                                                radius: self.border_radius,
                                                primary_color: theme.primary(),
                                                text_color: Color::WHITE,
                                                background_color: theme.background(),
                                            })))
                                            .padding(Padding::new(14.0)),
                                    ]
                                    .spacing(10),
                                ]
                                .spacing(10)
                            )
                            .padding(Padding::new(16.0))
                            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                radius: self.border_radius,
                                background: Some(theme.background()),
                                            elevation: 1.5, // Elevated section for package details
                            }))),
                            Space::with_height(Length::Fixed(15.0)),
                            // Package management
                            container(
                                column![
                                    text("Package Management")
                                        .size(16)
                                        .style(iced::theme::Text::Color(theme.text())),
                                    Space::with_height(Length::Fixed(10.0)),
                                    row![
                                        button("Autoremove")
                                            .on_press(Message::PikmanAutoremove)
                                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                is_primary: false,
                                                radius: self.border_radius,
                                                primary_color: theme.primary(),
                                                text_color: Color::WHITE,
                                                background_color: theme.background(),
                                            })))
                                            .padding(Padding::new(14.0)),
                                        button("Show Upgrades")
                                            .on_press(Message::PikmanUpgrades)
                                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                is_primary: false,
                                                radius: self.border_radius,
                                                primary_color: theme.primary(),
                                                text_color: Color::WHITE,
                                                background_color: theme.background(),
                                            })))
                                            .padding(Padding::new(14.0)),
                                        button("View Logs")
                                            .on_press(Message::PikmanLog)
                                            .style(iced::theme::Button::Custom(Box::new(RoundedButtonStyle {
                                                is_primary: false,
                                                radius: self.border_radius,
                                                primary_color: theme.primary(),
                                                text_color: Color::WHITE,
                                                background_color: theme.background(),
                                            })))
                                            .padding(Padding::new(14.0)),
                                    ]
                                    .spacing(10),
                                ]
                                .spacing(10)
                            )
                            .padding(Padding::new(16.0))
                            .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                                radius: self.border_radius,
                                background: Some(theme.background()),
                                            elevation: 1.5, // Elevated section for package details
                            }))),
                            Space::with_height(Length::Fixed(15.0)),
                            text("Note: Some commands require additional input. Use the CLI for full functionality.")
                                .size(12)
                                .style(iced::theme::Text::Color(theme.secondary_text())),
                        ]
                        .spacing(15)
                        .padding(10)
                    )
                    .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                        background_color: theme.background(),
                        border_radius: self.border_radius,
                    })))
                )
                .width(Length::Fill)
                .padding(Padding::new(16.0))
                .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
                    radius: self.border_radius,
                    background: Some(theme.surface()),
                    elevation: 1.0,
                })))
            )
        };

        column![
            search_section,
            Space::with_height(Length::Fixed(15.0)),
            content_section,
        ]
        .spacing(10)
        .width(Length::Fill)
        .into()
    }

    // System update functionality removed - handled by separate app
    // System update functionality removed - handled by separate app

    // Dialog view methods removed - dialogs are now separate windows

    #[allow(dead_code)]
    fn view_output(&self) -> Element<Message> {
        let theme = self.theme;
        let output_text = if !self.output_log.is_empty() {
            self.output_log.join("\n")
        } else if !self.error_log.is_empty() {
            format!("Errors:\n{}", self.error_log.join("\n"))
        } else {
            String::new()
        };

        container(
            scrollable(
                text(output_text)
                    .size(12)
                    .style(iced::theme::Text::Color(if !self.error_log.is_empty() {
                        theme.danger()
                    } else {
                        theme.secondary_text()
                    })),
            )
            .style(iced::theme::Scrollable::Custom(Box::new(CustomScrollableStyle {
                background_color: theme.background(),
                border_radius: self.border_radius,
            })))
            .height(Length::Fixed(150.0)),
        )
        .width(Length::Fill)
        .padding(Padding::new(12.0))
        .style(iced::theme::Container::Custom(Box::new(RoundedContainerStyle {
            radius: self.border_radius,
            background: Some(theme.surface()),
            elevation: 1.0,
        })))
        .into()
    }
}

// Async functions for package operations
async fn search_packages(query: String) -> Vec<PackageInfo> {
    tokio::task::spawn_blocking(move || {
        match PackageManager::new() {
            Ok(_pm) => {
                use crate::utils::run_command;
                // Use apt-cache search which searches both names and descriptions
                // but returns results in a format we can parse
                let mut packages = match run_command("apt-cache", &["search", &query], false) {
                    Ok(output) => parse_apt_cache_search_output(&output),
                    Err(_) => vec![],
                };
                
                // Sort results to prioritize packages with query in name
                let query_lower = query.to_lowercase();
                packages.sort_by(|a, b| {
                    let a_name_lower = a.name.to_lowercase();
                    let b_name_lower = b.name.to_lowercase();
                    
                    // Check if query appears in name
                    let a_has_in_name = a_name_lower.contains(&query_lower);
                    let b_has_in_name = b_name_lower.contains(&query_lower);
                    
                    // Check if query is at the start of name
                    let a_starts_with = a_name_lower.starts_with(&query_lower);
                    let b_starts_with = b_name_lower.starts_with(&query_lower);
                    
                    match (a_starts_with, b_starts_with) {
                        (true, false) => std::cmp::Ordering::Less,
                        (false, true) => std::cmp::Ordering::Greater,
                        _ => {
                            match (a_has_in_name, b_has_in_name) {
                                (true, false) => std::cmp::Ordering::Less,
                                (false, true) => std::cmp::Ordering::Greater,
                                _ => a.name.cmp(&b.name), // Alphabetical if both match equally
                            }
                        }
                    }
                });
                
                packages
            }
            Err(_) => vec![],
        }
    })
    .await
    .unwrap_or_default()
}

#[allow(dead_code)]
fn parse_search_output(output: &str) -> Vec<PackageInfo> {
    let mut packages = Vec::new();
    let mut current_pkg: Option<PackageInfo> = None;
    
    for line in output.lines() {
        if line.trim().is_empty() {
            if let Some(pkg) = current_pkg.take() {
                packages.push(pkg);
            }
            continue;
        }
        
        if line.contains('/') && !line.starts_with(' ') {
            // New package line: "package/version description"
            if let Some(pkg) = current_pkg.take() {
                packages.push(pkg);
            }
            
            let parts: Vec<&str> = line.splitn(2, ' ').collect();
            if parts.len() >= 1 {
                let name_version = parts[0];
                let name_parts: Vec<&str> = name_version.split('/').collect();
                let name = name_parts[0].to_string();
                let version = name_parts.get(1).unwrap_or(&"").to_string();
                let description = parts.get(1).unwrap_or(&"").to_string();
                
                current_pkg = Some(PackageInfo {
                    name,
                    version,
                    description,
                    size: String::new(),
                    source: PackageSource::Default,
                });
            }
        } else if let Some(ref mut pkg) = current_pkg {
            // Continuation of description
            pkg.description.push_str(" ");
            pkg.description.push_str(line.trim());
        }
    }
    
    if let Some(pkg) = current_pkg {
        packages.push(pkg);
    }
    
    packages
}

// Parse apt-cache search output format: "package - description"
fn parse_apt_cache_search_output(output: &str) -> Vec<PackageInfo> {
    let mut packages = Vec::new();
    let mut seen = std::collections::HashSet::new();
    
    for line in output.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        
        // Format: "package - description" or "package/version - description"
        if let Some(dash_pos) = line.find(" - ") {
            let name_part = line[..dash_pos].trim();
            let description = line[dash_pos + 3..].trim().to_string();
            
            // Handle package/version format
            let (name, version) = if name_part.contains('/') {
                let parts: Vec<&str> = name_part.splitn(2, '/').collect();
                (parts[0].to_string(), parts.get(1).unwrap_or(&"").to_string())
            } else {
                (name_part.to_string(), String::new())
            };
            
            // Skip duplicates
            if seen.contains(&name) {
                continue;
            }
            seen.insert(name.clone());
            
            packages.push(PackageInfo {
                name,
                version,
                description,
                size: String::new(),
                source: PackageSource::Default,
            });
        }
    }
    
    packages
}

async fn load_installed_packages() -> Vec<PackageInfo> {
    eprintln!("[DEBUG] load_installed_packages() async function called");
    tokio::task::spawn_blocking(|| {
        eprintln!("[DEBUG] load_installed_packages: Inside spawn_blocking");
        let start_time = std::time::Instant::now();
        
        // Try to load from cache first
        eprintln!("[DEBUG] load_installed_packages: Attempting to load from cache...");
        if let Some(cached) = load_packages_cache() {
            eprintln!("[DEBUG] load_installed_packages: Loaded {} packages from cache in {:?}", cached.len(), start_time.elapsed());
            return cached;
        }
        
        eprintln!("[DEBUG] load_installed_packages: Cache miss or invalid, loading from dpkg status file");
        
        // Read directly from dpkg status file - much faster than spawning dpkg-query
        use std::fs;
        let status_path = std::path::PathBuf::from("/var/lib/dpkg/status");
        let status_content = match fs::read_to_string(&status_path) {
            Ok(content) => content,
            Err(e) => {
                eprintln!("[DEBUG] load_installed_packages: Failed to read status file: {}, using fallback", e);
                return load_installed_packages_fallback();
            }
        };
        
        eprintln!("[DEBUG] load_installed_packages: Read status file in {:?}, length: {}", start_time.elapsed(), status_content.len());
        
        // Parse status file directly - much faster than spawning a process
        let parse_start = std::time::Instant::now();
        let mut packages = Vec::new();
        let mut current_name = String::new();
        let mut current_version = String::new();
        let mut is_installed = false;
        
        // Optimized parsing: single pass through the file
        for line in status_content.lines() {
            if line.starts_with("Package: ") {
                // Save previous package if it was installed
                if is_installed && !current_name.is_empty() {
                    packages.push(PackageInfo {
                        name: std::mem::take(&mut current_name),
                        version: std::mem::take(&mut current_version),
                        description: String::new(),
                        size: String::new(),
                        source: PackageSource::Default,
                    });
                }
                current_name = line[9..].trim().to_string();
                current_version.clear();
                is_installed = false;
            } else if line.starts_with("Version: ") {
                current_version = line[9..].trim().to_string();
            } else if line.starts_with("Status: ") {
                // Check if package is installed
                let status = &line[8..];
                is_installed = status.contains("install ok installed") || status.contains("install ok config-files");
            } else if line.is_empty() {
                // End of package entry - save if installed
                if is_installed && !current_name.is_empty() {
                    packages.push(PackageInfo {
                        name: std::mem::take(&mut current_name),
                        version: std::mem::take(&mut current_version),
                        description: String::new(),
                        size: String::new(),
                        source: PackageSource::Default,
                    });
                }
                is_installed = false;
            }
        }
        
        // Don't forget the last package if file doesn't end with newline
        if is_installed && !current_name.is_empty() {
            packages.push(PackageInfo {
                name: current_name,
                version: current_version,
                description: String::new(),
                size: String::new(),
                source: PackageSource::Default,
            });
        }
        
        eprintln!("[DEBUG] load_installed_packages: Parsed {} packages in {:?}, total: {:?}", 
                 packages.len(), parse_start.elapsed(), start_time.elapsed());
        
        // Save to cache for next time
        save_packages_cache(&packages);
        
        packages
    })
    .await
    .unwrap_or_default()
}

// Cache functions
fn get_cache_path() -> Option<std::path::PathBuf> {
    let home = std::env::var("HOME").ok()?;
    Some(std::path::PathBuf::from(home).join(".config").join("birdnest").join("installed_packages.cache"))
}

fn get_dpkg_status_mtime() -> Option<std::time::SystemTime> {
    use std::fs;
    let status_path = std::path::PathBuf::from("/var/lib/dpkg/status");
    fs::metadata(&status_path).ok()?.modified().ok()
}

fn load_packages_cache() -> Option<Vec<PackageInfo>> {
    let cache_path = get_cache_path()?;
    let status_mtime = get_dpkg_status_mtime()?;
    
    use std::fs;
    let cache_metadata = fs::metadata(&cache_path).ok()?;
    let cache_mtime = cache_metadata.modified().ok()?;
    
    // Check if cache is newer than dpkg status file
    if cache_mtime < status_mtime {
        eprintln!("[DEBUG] load_packages_cache: Cache is older than dpkg status, invalidating");
        let _ = fs::remove_file(&cache_path);
        return None;
    }
    
    // Try to load cache
    let cache_data = fs::read(&cache_path).ok()?;
    
    // Use a simple binary format: first 8 bytes = count, then name\0version\0 pairs
    if cache_data.len() < 8 {
        return None;
    }
    
    let count = u64::from_le_bytes([
        cache_data[0], cache_data[1], cache_data[2], cache_data[3],
        cache_data[4], cache_data[5], cache_data[6], cache_data[7],
    ]) as usize;
    
    let mut packages = Vec::with_capacity(count);
    let mut pos = 8;
    
    for _ in 0..count {
        if pos >= cache_data.len() {
            return None; // Corrupted cache
        }
        
        // Read name (null-terminated)
        let name_start = pos;
        while pos < cache_data.len() && cache_data[pos] != 0 {
            pos += 1;
        }
        if pos >= cache_data.len() {
            return None;
        }
        let name = String::from_utf8_lossy(&cache_data[name_start..pos]).to_string();
        pos += 1; // Skip null terminator
        
        // Read version (null-terminated)
        if pos >= cache_data.len() {
            return None;
        }
        let version_start = pos;
        while pos < cache_data.len() && cache_data[pos] != 0 {
            pos += 1;
        }
        if pos >= cache_data.len() {
            return None;
        }
        let version = String::from_utf8_lossy(&cache_data[version_start..pos]).to_string();
        pos += 1; // Skip null terminator
        
        packages.push(PackageInfo {
            name,
            version,
            description: String::new(),
            size: String::new(),
            source: PackageSource::Default,
        });
    }
    
    eprintln!("[DEBUG] load_packages_cache: Loaded {} packages from cache", packages.len());
    Some(packages)
}

fn save_packages_cache(packages: &[PackageInfo]) {
    let cache_path = match get_cache_path() {
        Some(p) => p,
        None => {
            eprintln!("[DEBUG] save_packages_cache: Could not get cache path");
            return;
        }
    };
    
    use std::fs;
    // Create directory if it doesn't exist
    if let Some(parent) = cache_path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    
    // Use simple binary format for speed: count (8 bytes) + name\0version\0 pairs
    let mut cache_data = Vec::with_capacity(packages.len() * 50); // Estimate 50 bytes per package
    
    // Write count
    let count = packages.len() as u64;
    cache_data.extend_from_slice(&count.to_le_bytes());
    
    // Write packages
    for pkg in packages {
        cache_data.extend_from_slice(pkg.name.as_bytes());
        cache_data.push(0); // Null terminator
        cache_data.extend_from_slice(pkg.version.as_bytes());
        cache_data.push(0); // Null terminator
    }
    
    if let Err(e) = fs::write(&cache_path, cache_data) {
        eprintln!("[DEBUG] save_packages_cache: Failed to write cache: {}", e);
    } else {
        eprintln!("[DEBUG] save_packages_cache: Saved {} packages to cache", packages.len());
    }
}

// Function to invalidate cache (call after install/remove operations)
pub fn invalidate_packages_cache() {
    if let Some(cache_path) = get_cache_path() {
        let _ = std::fs::remove_file(&cache_path);
        eprintln!("[DEBUG] invalidate_packages_cache: Cache invalidated");
    }
}

// Fallback method using utils::run_command
fn load_installed_packages_fallback() -> Vec<PackageInfo> {
    use crate::utils::run_command;
    match run_command("dpkg-query", &["-W", "-f=${Package}\t${Version}\n"], false) {
        Ok(output) => {
            let mut packages = Vec::with_capacity(2000);
            for line in output.lines() {
                if line.trim().is_empty() {
                    continue;
                }
                
                if let Some((name, version)) = line.split_once('\t') {
                    let name = name.trim();
                    if !name.is_empty() {
                        packages.push(PackageInfo {
                            name: name.to_string(),
                            version: version.trim().to_string(),
                            description: String::new(),
                            size: String::new(),
                            source: PackageSource::Default,
                        });
                    }
                }
            }
            packages
        }
        Err(e) => {
            eprintln!("Error loading installed packages: {}", e);
            vec![]
        }
    }
}

// check_updates function removed - system updates handled by separate app
#[allow(dead_code)]
async fn _check_updates_removed() -> Vec<String> {
    tokio::task::spawn_blocking(|| {
        match PackageManager::new() {
            Ok(_pm) => {
                use crate::utils::run_command;
                match run_command("apt", &["list", "--upgradable"], false) {
                    Ok(output) => {
                        output.lines()
                            .skip(1) // Skip header
                            .filter_map(|line| {
                                if line.contains('/') {
                                    line.split('/').next().map(|s| s.to_string())
                                } else {
                                    None
                                }
                            })
                            .collect()
                    }
                    Err(_) => vec![],
                }
            }
            Err(_) => vec![],
        }
    })
    .await
    .unwrap_or_default()
}

async fn load_flatpak_apps() -> Result<Vec<FlatpakInfo>, anyhow::Error> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] load_flatpak_apps() async function called");
    tokio::task::spawn_blocking(|| {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: Inside spawn_blocking");
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: Creating FlatpakManager...");
        match FlatpakManager::new() {
            Ok(_fm) => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] load_flatpak_apps: FlatpakManager created, running 'flatpak list --columns=name,application'...");
                use crate::utils::run_command;
                // Use --columns=name,application to get both display name and application ID
                match run_command("flatpak", &["list", "--columns=name,application"], false) {
                    Ok(output) => {
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: 'flatpak list --columns=name,application' succeeded, parsing output...");
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: Output (first 200 chars): {}", output.chars().take(200).collect::<String>());
                        // Parse tab-separated output: name<TAB>application
                        let apps: Vec<FlatpakInfo> = output.lines()
                            .filter_map(|line| {
                                let trimmed = line.trim();
                                if !trimmed.is_empty() {
                                    let parts: Vec<&str> = trimmed.split('\t').collect();
                                    if parts.len() >= 2 {
                                        let name = parts[0].trim().to_string();
                                        let application = parts[1].trim().to_string();
                                        #[cfg(debug_assertions)]
                                        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: Found app - name: '{}', ID: '{}'", name, application);
                                        Some(FlatpakInfo {
                                            name,
                                            description: String::new(),
                                            version: String::new(),
                                            application,
                                        })
                                    } else {
                                        #[cfg(debug_assertions)]
                                        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: WARNING - Invalid line format: '{}'", trimmed);
                                        None
                                    }
                                } else {
                                    None
                                }
                            })
                            .collect();
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: Parsed {} Flatpak apps", apps.len());
                        Ok(apps)
                    }
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: ERROR - 'flatpak list' failed: {}", e);
                        Err(e)
                    }
                }
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] load_flatpak_apps: ERROR - Failed to create FlatpakManager: {}", e);
                Err(e)
            }
        }
    })
    .await
    .unwrap_or_else(|_| {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] load_flatpak_apps: ERROR - Task failed");
        Err(anyhow::anyhow!("Failed to load flatpak apps"))
    })
}

#[allow(dead_code)]
async fn install_package(package: String) -> Result<String, anyhow::Error> {
    use tokio::process::Command as TokioCommand;
    
    // Use pkexec for GUI privilege escalation (like Rustora)
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("apt");
    cmd.arg("install");
    cmd.arg("-y");
    cmd.arg(&package);
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(xauth) = std::env::var("XAUTHORITY") {
        cmd.env("XAUTHORITY", xauth);
    }
    if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
        cmd.env("WAYLAND_DISPLAY", wayland);
    }
    
    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute installation: {}. Make sure polkit is installed.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if user cancelled the password dialog (exit code 126 or 127)
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err(anyhow::anyhow!("Authentication cancelled or failed. Please try again."));
        }
        return Err(anyhow::anyhow!("Installation failed: {}", stderr));
    }
    
    Ok(format!("Successfully installed {}", package))
}

#[allow(dead_code)]
async fn remove_package(package: String) -> Result<String, anyhow::Error> {
    use tokio::process::Command as TokioCommand;
    
    // Use pkexec for GUI privilege escalation (like Rustora)
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("apt");
    cmd.arg("remove");
    cmd.arg("-y");
    cmd.arg(&package);
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(xauth) = std::env::var("XAUTHORITY") {
        cmd.env("XAUTHORITY", xauth);
    }
    if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
        cmd.env("WAYLAND_DISPLAY", wayland);
    }
    
    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute removal: {}. Make sure polkit is installed.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if user cancelled the password dialog (exit code 126 or 127)
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err(anyhow::anyhow!("Authentication cancelled or failed. Please try again."));
        }
        return Err(anyhow::anyhow!("Removal failed: {}", stderr));
    }
    
    Ok(format!("Successfully removed {}", package))
}

// update_lists function removed - system updates handled by separate app
#[allow(dead_code)]
async fn _update_lists_removed() -> Result<String, anyhow::Error> {
    use tokio::process::Command as TokioCommand;
    
    // Use pkexec for GUI privilege escalation (like Rustora)
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("apt");
    cmd.arg("update");
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(xauth) = std::env::var("XAUTHORITY") {
        cmd.env("XAUTHORITY", xauth);
    }
    if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
        cmd.env("WAYLAND_DISPLAY", wayland);
    }
    
    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute update: {}. Make sure polkit is installed.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if user cancelled the password dialog (exit code 126 or 127)
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err(anyhow::anyhow!("Authentication cancelled or failed. Please try again."));
        }
        return Err(anyhow::anyhow!("Update failed: {}", stderr));
    }
    
    Ok("Package lists updated".to_string())
}

// upgrade_all function removed - system updates handled by separate app
#[allow(dead_code)]
async fn _upgrade_all_removed() -> Result<String, anyhow::Error> {
    use tokio::process::Command as TokioCommand;
    
    // Use pkexec for GUI privilege escalation (like Rustora)
    let mut cmd = TokioCommand::new("pkexec");
    cmd.arg("apt");
    cmd.arg("upgrade");
    cmd.arg("-y");
    
    // Ensure DISPLAY is set for GUI password dialog
    if let Ok(display) = std::env::var("DISPLAY") {
        cmd.env("DISPLAY", display);
    }
    if let Ok(xauth) = std::env::var("XAUTHORITY") {
        cmd.env("XAUTHORITY", xauth);
    }
    if let Ok(wayland) = std::env::var("WAYLAND_DISPLAY") {
        cmd.env("WAYLAND_DISPLAY", wayland);
    }
    
    let output = cmd
        .output()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to execute upgrade: {}. Make sure polkit is installed.", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        // Check if user cancelled the password dialog (exit code 126 or 127)
        if output.status.code() == Some(126) || output.status.code() == Some(127) {
            return Err(anyhow::anyhow!("Authentication cancelled or failed. Please try again."));
        }
        return Err(anyhow::anyhow!("Upgrade failed: {}", stderr));
    }
    
    Ok("All packages upgraded".to_string())
}

async fn search_flatpak(query: String) -> Result<Vec<FlatpakInfo>, anyhow::Error> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] search_flatpak() called with query: '{}'", query);
    tokio::task::spawn_blocking(move || {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] search_flatpak: Creating FlatpakManager...");
        match FlatpakManager::new() {
            Ok(_fm) => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] search_flatpak: FlatpakManager created, executing search...");
                use crate::utils::run_command;
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] search_flatpak: Executing command: flatpak search {}", query);
                match run_command("flatpak", &["search", &query], false) {
                    Ok(output) => {
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] search_flatpak: Search completed, output length: {} bytes", output.len());
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] search_flatpak: Parsing search output...");
                        let results = parse_flatpak_search_output(&output);
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] search_flatpak: Parsed {} results", results.len());
                        Ok(results)
                    },
                    Err(e) => {
                        #[cfg(debug_assertions)]
                        eprintln!("[FLATPAK DEBUG] search_flatpak: ERROR - Command failed: {}", e);
                        Err(e)
                    },
                }
            }
            Err(e) => {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] search_flatpak: ERROR - FlatpakManager creation failed: {}", e);
                Err(e)
            },
        }
    })
    .await
    .unwrap_or_else(|_| {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] search_flatpak: ERROR - Task failed");
        Err(anyhow::anyhow!("Failed to search flatpak"))
    })
}

fn parse_flatpak_search_output(output: &str) -> Vec<FlatpakInfo> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] parse_flatpak_search_output() called, input length: {} bytes", output.len());
    let mut packages = Vec::new();
    let mut current_pkg: Option<FlatpakInfo> = None;
    
    #[cfg(debug_assertions)]
    let mut line_count = 0;
    for line in output.lines() {
        #[cfg(debug_assertions)]
        {
            line_count += 1;
            if line_count <= 5 {
                eprintln!("[FLATPAK DEBUG] parse_flatpak_search_output: Processing line {}: '{}'", line_count, line);
            }
        }
        if line.trim().is_empty() {
            if let Some(pkg) = current_pkg.take() {
                packages.push(pkg);
            }
            continue;
        }
        
        // Flatpak search output format (tab-separated): "Name	Description	Application ID	Version	Branch	Origin"
        // Example: "Extension Manager	Install GNOME Extensions	com.mattjakeman.ExtensionManager	0.6.5	stable	flathub"
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let name = parts[0].trim().to_string();
            let description = parts.get(1).map(|s| s.trim()).unwrap_or("").to_string();
            let application = parts[2].trim().to_string();
            let version = parts.get(3).map(|s| s.trim()).unwrap_or("").to_string();
            
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] parse_flatpak_search_output: Parsed - name: '{}', description: '{}', application: '{}', version: '{}'", name, description, application, version);
            
            if let Some(pkg) = current_pkg.take() {
                packages.push(pkg);
            }
            
            current_pkg = Some(FlatpakInfo {
                name,
                description: if description.is_empty() { "No description".to_string() } else { description },
                version,
                application,
            });
        } else if let Some(ref mut pkg) = current_pkg {
            // Continuation line for description
            pkg.description.push_str(" ");
            pkg.description.push_str(line.trim());
        }
    }
    
    if let Some(pkg) = current_pkg {
        packages.push(pkg);
    }
    
    packages
}

#[allow(dead_code)]
async fn install_flatpak(package: String) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        let fm = FlatpakManager::new()?;
        fm.install(&[package.clone()], true)?;
        Ok(format!("Successfully installed {}", package))
    })
    .await
    .unwrap()
}

async fn update_flatpak_repos() -> Result<String, anyhow::Error> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] update_flatpak_repos() called");
    tokio::task::spawn_blocking(|| {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] update_flatpak_repos: Creating FlatpakManager...");
        let fm = FlatpakManager::new()?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] update_flatpak_repos: Calling fm.update()...");
        fm.update()?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] update_flatpak_repos: Update completed successfully");
        Ok("Flatpak repositories updated".to_string())
    })
    .await
    .unwrap()
}

async fn upgrade_all_flatpaks() -> Result<String, anyhow::Error> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] upgrade_all_flatpaks() called");
    tokio::task::spawn_blocking(|| {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] upgrade_all_flatpaks: Creating FlatpakManager...");
        let fm = FlatpakManager::new()?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] upgrade_all_flatpaks: Calling fm.upgrade()...");
        fm.upgrade(&[], true)?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] upgrade_all_flatpaks: Upgrade completed successfully");
        Ok("All Flatpaks upgraded".to_string())
    })
    .await
    .unwrap()
}

#[allow(dead_code)]
async fn show_flatpak_info(package: String) -> Result<String, anyhow::Error> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] show_flatpak_info() called for package: '{}'", package);
    tokio::task::spawn_blocking(move || {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] show_flatpak_info: Creating FlatpakManager...");
        let _fm = FlatpakManager::new()?;
        use crate::utils::run_command;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] show_flatpak_info: Executing command: flatpak info {}", package);
        let output = run_command("flatpak", &["info", &package], false)?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] show_flatpak_info: Command completed, output length: {} bytes", output.len());
        Ok(output)
    })
    .await
    .unwrap()
}

async fn clean_flatpak() -> Result<String, anyhow::Error> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] clean_flatpak() called");
    tokio::task::spawn_blocking(|| {
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] clean_flatpak: Creating FlatpakManager...");
        let fm = FlatpakManager::new()?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] clean_flatpak: Calling fm.clean()...");
        fm.clean()?;
        #[cfg(debug_assertions)]
        eprintln!("[FLATPAK DEBUG] clean_flatpak: Clean completed successfully");
        Ok("Flatpak cache cleaned".to_string())
    })
    .await
    .unwrap()
}

async fn load_package_detail(package: String, is_flatpak: bool) -> Result<PackageDetail, anyhow::Error> {
    #[cfg(debug_assertions)]
    eprintln!("[FLATPAK DEBUG] load_package_detail() called for package: '{}', is_flatpak: {}", package, is_flatpak);
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command;
        
        if is_flatpak {
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_package_detail: Loading flatpak package info...");
            // Get flatpak info
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_package_detail: Executing command: flatpak info {}", package);
            let info_output = run_command("flatpak", &["info", &package], false)?;
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_package_detail: Command completed, output length: {} bytes", info_output.len());
            let mut version = String::new();
            let mut description = String::new();
            let mut size = String::new();
            
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_package_detail: Parsing info output...");
            for line in info_output.lines() {
                if line.starts_with("Version:") {
                    version = line.replace("Version:", "").trim().to_string();
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_package_detail: Found version: {}", version);
                } else if line.starts_with("Description:") {
                    description = line.replace("Description:", "").trim().to_string();
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_package_detail: Found description (length: {})", description.len());
                } else if line.starts_with("Installed size:") {
                    size = line.replace("Installed size:", "").trim().to_string();
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_package_detail: Found size: {}", size);
                }
            }
            
            // If size not found, try to get it from flatpak list
            if size.is_empty() {
                #[cfg(debug_assertions)]
                eprintln!("[FLATPAK DEBUG] load_package_detail: Size not found, trying flatpak list...");
                if let Ok(list_output) = run_command("flatpak", &["list", "--columns=name,size"], false) {
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_package_detail: Got list output, searching for package...");
                    for line in list_output.lines() {
                        if line.contains(&package) {
                            let parts: Vec<&str> = line.split('\t').collect();
                            if parts.len() >= 2 {
                                size = parts[1].trim().to_string();
                                #[cfg(debug_assertions)]
                                eprintln!("[FLATPAK DEBUG] load_package_detail: Found size from list: {}", size);
                            }
                        }
                    }
                } else {
                    #[cfg(debug_assertions)]
                    eprintln!("[FLATPAK DEBUG] load_package_detail: Failed to get list output");
                }
            }
            
            #[cfg(debug_assertions)]
            eprintln!("[FLATPAK DEBUG] load_package_detail: Creating PackageDetail - name: {}, version: {}, size: {}", package, version, size);
            Ok(PackageDetail {
                name: package,
                version: if version.is_empty() { "Unknown".to_string() } else { version },
                description: if description.is_empty() { "No description available".to_string() } else { description },
                size: if size.is_empty() { "Unknown".to_string() } else { size },
                is_flatpak: true,
            })
        } else {
            // Get apt package info
            let show_output = run_command("apt", &["show", &package], false)?;
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
            
            // If description is multi-line, get the full description
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
    .unwrap()
}

#[allow(dead_code)]
async fn load_multiple_package_details(packages: Vec<String>) -> Vec<PackageDetail> {
    // Load all package details in parallel
    let futures: Vec<_> = packages.into_iter()
        .map(|pkg| load_package_detail(pkg, false))
        .collect();
    
    let results: Vec<Result<PackageDetail, anyhow::Error>> = future::join_all(futures).await;
    
    // Collect successful results, skip failures
    let mut details = Vec::new();
    for result in results {
        match result {
            Ok(detail) => details.push(detail),
            Err(e) => {
                eprintln!("Warning: Failed to load package detail: {}", e);
                // Continue with other packages even if one fails
            }
        }
    }
    
    details
}

// System update functions removed - handled by separate app

// Pikman async functions
async fn pikman_search(query: String, filter: Option<String>) -> Result<Vec<PackageInfo>, anyhow::Error> {
    use tokio::process::Command as TokioCommand;
    
    // Try without sudo first
    // Note: --aur, --fedora, --alpine are GLOBAL options and must come BEFORE the command
    let mut cmd = TokioCommand::new("pikman");
    
    // Add global flags before the command
    if let Some(ref f) = filter {
        match f.as_str() {
            "aur" => {
                cmd.arg("--aur");
            }
            "fedora" => {
                cmd.arg("--fedora");
            }
            "alpine" => {
                cmd.arg("--alpine");
            }
            _ => {}
        }
    }
    
    cmd.arg("search");
    cmd.arg(&query);
    
    let output = cmd.output().await?;
    
    // If it succeeds without sudo, use that result
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Ok(parse_pikman_search_output(&stdout, filter.clone()));
    }
    
    // If it fails, check if it's a permission error
    let stderr = String::from_utf8_lossy(&output.stderr);
    let needs_sudo = stderr.contains("permission") || 
                     stderr.contains("Permission") ||
                     stderr.contains("denied") ||
                     output.status.code() == Some(1) && stderr.contains("sudo");
    
    // Only use pkexec if we actually need sudo
    if needs_sudo {
        let mut cmd = TokioCommand::new("pkexec");
        cmd.arg("pikman");
        cmd.arg("search");
        cmd.arg(&query);
        
        // Add global flags before the command
        if let Some(ref f) = filter {
            match f.as_str() {
                "aur" => {
                    cmd.arg("--aur");
                }
                "fedora" => {
                    cmd.arg("--fedora");
                }
                "alpine" => {
                    cmd.arg("--alpine");
                }
                _ => {}
            }
        }
        
        cmd.arg("search");
        cmd.arg(&query);
        
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
        
        let output = cmd.output().await?;
        
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if output.status.code() == Some(126) || output.status.code() == Some(127) {
                anyhow::bail!("Authentication failed or cancelled. Please try again.");
            }
            anyhow::bail!("Search failed: {}", stderr);
        }
        
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(parse_pikman_search_output(&stdout, filter))
    } else {
        // Not a permission error, return the original error
        anyhow::bail!("Search failed: {}", stderr);
    }
}

fn parse_pikman_search_output(output: &str, filter: Option<String>) -> Vec<PackageInfo> {
    let mut packages = Vec::new();
    
    // Determine source and output format based on filter
    let source = match filter.as_deref() {
        Some("aur") => PackageSource::Aur,
        Some("fedora") => PackageSource::Fedora,
        Some("alpine") => PackageSource::Alpine,
        _ => PackageSource::Default,
    };
    
    // Determine output format based on filter
    match filter.as_deref() {
        Some("aur") => {
            // AUR format: "repository/package-name version (download_size installed_size) [status]"
            // Description on next line (indented with spaces)
            let mut current_pkg: Option<PackageInfo> = None;
            
            for line in output.lines() {
                let line = line.trim();
                if line.is_empty() {
                    if let Some(pkg) = current_pkg.take() {
                        packages.push(pkg);
                    }
                    continue;
                }
                
                // Skip lines that are not package entries
                if line.starts_with("Matched fields:") || line.starts_with("!!!") || line.starts_with("Warning:") {
                    continue;
                }
                
                // Check if this is a package line (contains '/' and starts with repository name)
                if line.contains('/') && !line.starts_with(' ') && !line.starts_with('\t') {
                    // Save previous package
                    if let Some(pkg) = current_pkg.take() {
                        packages.push(pkg);
                    }
                    
                    // Parse: "repository/package-name version (download_size installed_size) [status]"
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if !parts.is_empty() {
                        let name_version = parts[0];
                        let name_parts: Vec<&str> = name_version.split('/').collect();
                        if name_parts.len() >= 2 {
                            let name = name_parts[1].to_string(); // Get package name after repository/
                            let mut version = String::new();
                            let mut size = String::new();
                            
                            if parts.len() > 1 {
                                let rest = parts[1];
                                // Extract version (before parentheses)
                                if let Some(paren_pos) = rest.find('(') {
                                    version = rest[..paren_pos].trim().to_string();
                                    // Extract size from parentheses: "(download_size installed_size)"
                                    if let Some(close_pos) = rest[paren_pos..].find(')') {
                                        let size_content = &rest[paren_pos+1..paren_pos+close_pos];
                                        // Format: "download_size installed_size" or just "download_size"
                                        let size_parts: Vec<&str> = size_content.split_whitespace().collect();
                                        if size_parts.len() >= 2 {
                                            size = format!("{} / {}", size_parts[0], size_parts[1]);
                                        } else if size_parts.len() == 1 {
                                            size = size_parts[0].to_string();
                                        }
                                    }
                                } else {
                                    version = rest.trim().to_string();
                                }
                            }
                            
                            current_pkg = Some(PackageInfo {
                                name,
                                version,
                                description: String::new(),
                                size,
                                source: source.clone(),
                            });
                        }
                    }
                } else if line.starts_with("    ") || line.starts_with('\t') {
                    // Indented line is description continuation
                    if let Some(ref mut pkg) = current_pkg {
                        if !pkg.description.is_empty() {
                            pkg.description.push_str(" ");
                        }
                        pkg.description.push_str(line.trim());
                    }
                }
            }
            
            if let Some(pkg) = current_pkg {
                packages.push(pkg);
            }
        }
        Some("fedora") => {
            // Fedora format: "package-name.arch	description" (tab-separated)
            for line in output.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("Matched fields:") || line.starts_with("!!!") || line.starts_with("Warning:") || line.starts_with("Updating") {
                    continue;
                }
                
                // Split by tab or multiple spaces
                let parts: Vec<&str> = if line.contains('\t') {
                    line.split('\t').collect()
                } else {
                    // Fallback: split by 2+ spaces
                    line.split_whitespace().collect::<Vec<_>>()
                };
                
                if parts.len() >= 2 {
                    let name_arch = parts[0];
                    let description = parts[1..].join(" ");
                    
                    // Extract package name (remove .arch suffix)
                    let name = if let Some(dot_pos) = name_arch.rfind('.') {
                        name_arch[..dot_pos].to_string()
                    } else {
                        name_arch.to_string()
                    };
                    
                    packages.push(PackageInfo {
                        name,
                        version: String::new(), // Fedora output doesn't show version in search
                        description,
                        size: String::new(), // Fedora search doesn't show size
                        source: source.clone(),
                    });
                } else if parts.len() == 1 && !parts[0].is_empty() {
                    // Single field - might be package name only
                    let name_arch = parts[0];
                    let name = if let Some(dot_pos) = name_arch.rfind('.') {
                        name_arch[..dot_pos].to_string()
                    } else {
                        name_arch.to_string()
                    };
                    
                    packages.push(PackageInfo {
                        name,
                        version: String::new(),
                        description: String::new(),
                        size: String::new(),
                        source: source.clone(),
                    });
                }
            }
        }
        Some("alpine") => {
            // Alpine format: "package-name-version" (simple format, no description in basic search)
            for line in output.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with("!!!") || line.starts_with("Warning:") {
                    continue;
                }
                
                // Format is typically "package-name-version" or just "package-name"
                let name_version = line;
                
                // Try to extract version (last segment after last dash that looks like a version)
                let parts: Vec<&str> = name_version.split('-').collect();
                if parts.len() >= 2 {
                    // Last part might be version (contains numbers and dots)
                    let last_part = parts.last().unwrap_or(&"");
                    let is_version = last_part.chars().any(|c| c.is_ascii_digit());
                    
                    if is_version && parts.len() > 1 {
                        let name = parts[..parts.len()-1].join("-");
                        let version = last_part.to_string();
                        packages.push(PackageInfo {
                            name,
                            version,
                            description: String::new(),
                            size: String::new(), // Alpine search doesn't show size
                            source: source.clone(),
                        });
                    } else {
                        // No clear version, use whole string as name
                        packages.push(PackageInfo {
                            name: name_version.to_string(),
                            version: String::new(),
                            description: String::new(),
                            size: String::new(),
                            source: source.clone(),
                        });
                    }
                } else {
                    packages.push(PackageInfo {
                        name: name_version.to_string(),
                        version: String::new(),
                        description: String::new(),
                        size: String::new(),
                        source: source.clone(),
                    });
                }
            }
        }
        _ => {
            // Default format (system packages): "package/version description"
            let mut current_pkg: Option<PackageInfo> = None;
            
            for line in output.lines() {
                let line = line.trim();
                if line.is_empty() {
                    if let Some(pkg) = current_pkg.take() {
                        packages.push(pkg);
                    }
                    continue;
                }
                
                // Skip metadata lines
                if line.starts_with("Matched fields:") || line.starts_with("!!!") || line.starts_with("Warning:") {
                    continue;
                }
                
                if line.contains('/') && !line.starts_with(' ') && !line.starts_with('\t') {
                    // New package line: "package/version description"
                    if let Some(pkg) = current_pkg.take() {
                        packages.push(pkg);
                    }
                    
                    let parts: Vec<&str> = line.splitn(2, ' ').collect();
                    if !parts.is_empty() {
                        let name_version = parts[0];
                        let name_parts: Vec<&str> = name_version.split('/').collect();
                        let name = name_parts[0].to_string();
                        let version = name_parts.get(1).unwrap_or(&"").to_string();
                        let description = parts.get(1).unwrap_or(&"").to_string();
                        
                        current_pkg = Some(PackageInfo {
                            name,
                            version,
                            description,
                            size: String::new(), // Default search doesn't show size
                            source: source.clone(),
                        });
                    }
                } else if let Some(ref mut pkg) = current_pkg {
                    // Continuation of description
                    if !pkg.description.is_empty() {
                        pkg.description.push_str(" ");
                    }
                    pkg.description.push_str(line);
                }
            }
            
            if let Some(pkg) = current_pkg {
                packages.push(pkg);
            }
        }
    }
    
    packages
}

async fn pikman_autoremove() -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(|| {
        use crate::utils::run_command_interactive;
        run_command_interactive("pikman", &["autoremove", "-y"], false)?;
        Ok("Autoremove completed".to_string())
    })
    .await
    .unwrap()
}

async fn pikman_enter(name: String) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command_interactive;
        run_command_interactive("pikman", &["enter", &name], false)?;
        Ok(format!("Entered container: {}", name))
    })
    .await
    .unwrap()
}

async fn pikman_export(package: String, name: Option<String>) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command_interactive;
        let mut args = vec!["export", &package];
        if let Some(ref n) = name {
            args.push("-n");
            args.push(n);
        }
        run_command_interactive("pikman", &args, false)?;
        Ok(format!("Exported package: {}", package))
    })
    .await
    .unwrap()
}

async fn pikman_init(name: String, manager: Option<String>) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command_interactive;
        let mut args = vec!["init", &name];
        if let Some(ref m) = manager {
            args.push("-m");
            args.push(m);
        }
        run_command_interactive("pikman", &args, false)?;
        Ok(format!("Initialized container: {}", name))
    })
    .await
    .unwrap()
}

async fn pikman_log() -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(|| {
        use crate::utils::run_command;
        let output = run_command("pikman", &["log"], false)?;
        Ok(output)
    })
    .await
    .unwrap()
}

async fn pikman_purge(packages: Vec<String>) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command_interactive;
        let mut args = vec!["purge", "-y"];
        args.extend(packages.iter().map(|s| s.as_str()));
        run_command_interactive("pikman", &args, false)?;
        Ok("Purge completed".to_string())
    })
    .await
    .unwrap()
}

async fn pikman_run(name: String, command: Vec<String>) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command_interactive;
        let mut args = vec!["run", &name];
        args.extend(command.iter().map(|s| s.as_str()));
        run_command_interactive("pikman", &args, false)?;
        Ok(format!("Command executed in container: {}", name))
    })
    .await
    .unwrap()
}

async fn pikman_upgrades() -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(|| {
        use crate::utils::run_command;
        let output = run_command("pikman", &["upgrades"], false)?;
        Ok(output)
    })
    .await
    .unwrap()
}

async fn pikman_unexport(package: String, name: Option<String>) -> Result<String, anyhow::Error> {
    tokio::task::spawn_blocking(move || {
        use crate::utils::run_command_interactive;
        let mut args = vec!["unexport", &package];
        if let Some(ref n) = name {
            args.push("-n");
            args.push(n);
        }
        run_command_interactive("pikman", &args, false)?;
        Ok(format!("Unexported package: {}", package))
    })
    .await
    .unwrap()
}
