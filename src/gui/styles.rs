use iced::{Border, Color};
use iced::widget::button::{Appearance as ButtonAppearance, StyleSheet as ButtonStyleSheet};
use iced::widget::container::{Appearance as ContainerAppearance, StyleSheet as ContainerStyleSheet};
use iced::widget::scrollable::{Appearance as ScrollableAppearance, StyleSheet as ScrollableStyleSheet};
use iced::widget::text_input::{Appearance as TextInputAppearance, StyleSheet as TextInputStyleSheet};
use iced::widget::checkbox::{Appearance as CheckboxAppearance, StyleSheet as CheckboxStyleSheet};

pub struct RoundedButtonStyle {
    pub is_primary: bool,
    pub radius: f32,
    pub primary_color: Color,
    pub text_color: Color,
    #[allow(dead_code)]
    pub background_color: Color,
}

impl ButtonStyleSheet for RoundedButtonStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ButtonAppearance {
        // Simulate STRONG gradient by using MUCH brighter top color
        let bg_color = if self.is_primary {
            // Create EXTREME gradient effect: MUCH brighter at top
            // Simulate strong 3D bubble with very light top
            Color {
                r: (self.primary_color.r * 1.25).min(1.0),
                g: (self.primary_color.g * 1.25).min(1.0),
                b: (self.primary_color.b * 1.25).min(1.0),
                a: self.primary_color.a,
            }
        } else {
            // Secondary buttons: MUCH lighter grey for strong 3D effect
            Color::from_rgba(0.55, 0.55, 0.57, 0.6)
        };
        
        ButtonAppearance {
            background: Some(iced::Background::Color(bg_color)),
            border: Border {
                radius: self.radius.into(),
                width: if self.is_primary { 0.0 } else { 3.0 },
                color: if self.is_primary {
                    Color::TRANSPARENT
                } else {
                    // Highlight border for 3D edge effect
                    Color::from_rgba(0.9, 0.85, 0.4, 0.5)
                },
            },
            text_color: self.text_color,
            shadow: iced::Shadow {
                // MAXIMUM shadows for EXTREME 3D bubble effect
                color: Color::from_rgba(0.0, 0.0, 0.0, 1.0), // Always maximum opacity
                offset: iced::Vector::new(0.0, if self.is_primary { 20.0 } else { 15.0 }),
                blur_radius: if self.is_primary { 50.0 } else { 35.0 },
            },
            shadow_offset: iced::Vector::default(),
        }
    }

    fn hovered(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        if self.is_primary {
            // Even brighter on hover - simulate light hitting the bubble
            let mut color = self.primary_color;
            color = Color {
                r: (color.r * 1.3).min(1.0),
                g: (color.g * 1.3).min(1.0),
                b: (color.b * 1.3).min(1.0),
                a: color.a,
            };
            appearance.background = Some(iced::Background::Color(color));
            // MAXIMUM shadow on hover - bubble EXPLODES out
            appearance.shadow = iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 1.0),
                offset: iced::Vector::new(0.0, 25.0),
                blur_radius: 60.0,
            };
        } else {
            // Secondary buttons: brighter on hover
            appearance.background = Some(iced::Background::Color(Color::from_rgba(0.55, 0.55, 0.57, 0.6)));
            appearance.border.color = Color::from_rgba(0.9, 0.85, 0.4, 0.7);
            appearance.border.width = 3.5;
            appearance.shadow = iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 1.0),
                offset: iced::Vector::new(0.0, 18.0),
                blur_radius: 40.0,
            };
        }
        appearance
    }

    fn pressed(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        if self.is_primary {
            let mut color = self.primary_color;
            color = Color {
                r: (color.r * 0.85).max(0.0),
                g: (color.g * 0.85).max(0.0),
                b: (color.b * 0.85).max(0.0),
                a: color.a,
            };
            appearance.background = Some(iced::Background::Color(color));
            // Pressed state - bubble pushed in
            appearance.shadow = iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.3),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            };
        } else {
            // Pressed state - bubble pushed in
            appearance.shadow = iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, 0.2),
                offset: iced::Vector::new(0.0, 2.0),
                blur_radius: 4.0,
            };
        }
        appearance
    }

    fn disabled(&self, style: &Self::Style) -> ButtonAppearance {
        let mut appearance = self.active(style);
        appearance.background = Some(iced::Background::Color(Color::from_rgba(0.5, 0.5, 0.5, 0.05)));
        appearance.text_color = Color::from_rgba(0.5, 0.5, 0.5, 0.5);
        appearance
    }
}

pub struct RoundedContainerStyle {
    pub radius: f32,
    pub background: Option<Color>,
    pub elevation: f32, // 0.0 = flat, higher = more elevated
}

impl Default for RoundedContainerStyle {
    fn default() -> Self {
        Self {
            radius: 12.0,
            background: None,
            elevation: 0.0,
        }
    }
}

impl ContainerStyleSheet for RoundedContainerStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> ContainerAppearance {
        let palette = style.palette();
        
        // Simulate gradient: make cards slightly lighter at top for 3D bubble effect
        // But don't apply to main background (elevation 0.0)
        let bg_color = if self.elevation == 0.0 {
            // Main background - no gradient, use as-is
            self.background.unwrap_or(palette.background)
        } else if let Some(custom_bg) = self.background {
            // Cards - apply subtle gradient
            Color {
                r: (custom_bg.r * 1.08).min(1.0),
                g: (custom_bg.g * 1.08).min(1.0),
                b: (custom_bg.b * 1.08).min(1.0),
                a: custom_bg.a,
            }
        } else {
            // Default background for cards - subtle gradient
            Color {
                r: (palette.background.r * 1.05).min(1.0),
                g: (palette.background.g * 1.05).min(1.0),
                b: (palette.background.b * 1.05).min(1.0),
                a: palette.background.a,
            }
        };
        
        // Calculate shadow based on elevation - subtle for cards, stronger for elevated sections
        let shadow_opacity = if self.elevation == 0.0 {
            0.0 // No shadow for main background
        } else if self.elevation <= 1.0 {
            // Subtle shadow for individual cards
            0.3 + (self.elevation * 0.2)
        } else {
            // Stronger shadow for elevated sections
            (0.5 + (self.elevation - 1.0) * 0.3).min(1.0)
        };
        let shadow_offset = if self.elevation == 0.0 {
            0.0
        } else if self.elevation <= 1.0 {
            self.elevation * 4.0 // Subtle offset for cards
        } else {
            self.elevation * 8.0 // Stronger offset for sections
        };
        let shadow_blur = if self.elevation == 0.0 {
            0.0
        } else if self.elevation <= 1.0 {
            8.0 + self.elevation * 4.0 // Subtle blur for cards
        } else {
            15.0 + (self.elevation - 1.0) * 10.0 // Stronger blur for sections
        };
        
        // Visible border for 3D edge definition
        let border_opacity = if self.elevation == 0.0 {
            0.0 // No border for main background
        } else {
            0.2 + self.elevation * 0.15
        };
        
        ContainerAppearance {
            background: Some(iced::Background::Color(bg_color)),
            border: Border {
                radius: self.radius.into(),
                width: if self.elevation == 0.0 { 0.0 } else { 2.0 },
                color: Color::from_rgba(0.95, 0.9, 0.45, border_opacity.min(0.5)), // Visible yellow border
            },
            shadow: iced::Shadow {
                color: Color::from_rgba(0.0, 0.0, 0.0, shadow_opacity),
                offset: iced::Vector::new(0.0, shadow_offset),
                blur_radius: shadow_blur,
            },
            text_color: None,
        }
    }
}

pub struct RoundedMessageStyle {
    pub radius: f32,
}

impl ContainerStyleSheet for RoundedMessageStyle {
    type Style = iced::Theme;

    fn appearance(&self, style: &Self::Style) -> ContainerAppearance {
        let palette = style.palette();
        ContainerAppearance {
            background: Some(iced::Background::Color(palette.background)),
            border: Border {
                radius: self.radius.into(),
                width: 0.0,
                color: Color::TRANSPARENT,
            },
            ..Default::default()
        }
    }
}

pub struct CustomScrollableStyle {
    pub background_color: Color,
    pub border_radius: f32,
}

impl ScrollableStyleSheet for CustomScrollableStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> ScrollableAppearance {
        let is_dark = self.background_color.r < 0.5;
        let primary_color = if is_dark {
            Color::from_rgb(0.85, 0.75, 0.35) // Yellow for dark theme
        } else {
            Color::from_rgb(0.7, 0.6, 0.2) // Yellow for light theme
        };

        ScrollableAppearance {
            container: ContainerAppearance {
                background: None,
                border: Border::default(),
                ..Default::default()
            },
            scrollbar: iced::widget::scrollable::Scrollbar {
                background: Some(iced::Background::Color(Color::TRANSPARENT)),
                border: Border {
                    radius: 0.0.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
                scroller: iced::widget::scrollable::Scroller {
                    color: if is_dark {
                        Color::from_rgba(primary_color.r, primary_color.g, primary_color.b, 0.5)
                    } else {
                        Color::from_rgba(primary_color.r * 0.7, primary_color.g * 0.7, primary_color.b * 0.7, 0.5)
                    },
                    border: Border {
                        radius: (self.border_radius * 0.5).into(),
                        width: 0.0,
                        color: Color::TRANSPARENT,
                    },
                },
            },
            gap: None,
        }
    }

    fn hovered(&self, style: &Self::Style, _is_mouse_over_scrollbar: bool) -> ScrollableAppearance {
        let mut appearance = self.active(style);
        let is_dark = self.background_color.r < 0.5;
        let primary_color = if is_dark {
            Color::from_rgb(0.85, 0.75, 0.35) // Yellow for dark theme
        } else {
            Color::from_rgb(0.7, 0.6, 0.2) // Yellow for light theme
        };

        appearance.scrollbar.scroller.color = if is_dark {
            Color::from_rgba(primary_color.r, primary_color.g, primary_color.b, 0.7)
        } else {
            Color::from_rgba(primary_color.r * 0.7, primary_color.g * 0.7, primary_color.b * 0.7, 0.7)
        };
        appearance
    }

    fn dragging(&self, style: &Self::Style) -> ScrollableAppearance {
        let mut appearance = self.active(style);
        let is_dark = self.background_color.r < 0.5;
        let primary_color = if is_dark {
            Color::from_rgb(0.85, 0.75, 0.35) // Yellow for dark theme
        } else {
            Color::from_rgb(0.7, 0.6, 0.2) // Yellow for light theme
        };

        appearance.scrollbar.scroller.color = if is_dark {
            Color::from_rgba(primary_color.r, primary_color.g, primary_color.b, 0.9)
        } else {
            Color::from_rgba(primary_color.r * 0.7, primary_color.g * 0.7, primary_color.b * 0.7, 0.9)
        };
        appearance
    }
}

pub struct YellowTextInputStyle {
    pub radius: f32,
    pub primary_color: Color,
    pub background_color: Color,
    pub text_color: Color,
}

impl TextInputStyleSheet for YellowTextInputStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style) -> TextInputAppearance {
        TextInputAppearance {
            background: iced::Background::Color(self.background_color), // Normal background
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: self.primary_color, // Yellow border
            },
            icon_color: self.text_color,
        }
    }

    fn focused(&self, _style: &Self::Style) -> TextInputAppearance {
        TextInputAppearance {
            background: iced::Background::Color(self.background_color), // Normal background
            border: Border {
                radius: self.radius.into(),
                width: 2.0,
                color: self.primary_color, // Yellow border when focused
            },
            icon_color: self.text_color,
        }
    }

    fn placeholder_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(1.0, 1.0, 1.0, 0.5) // White with transparency for placeholder
    }

    fn value_color(&self, _style: &Self::Style) -> Color {
        Color::WHITE // White text on dark background
    }

    fn disabled_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.0, 0.0, 0.5)
    }

    fn selection_color(&self, _style: &Self::Style) -> Color {
        Color::from_rgba(0.0, 0.0, 0.0, 0.3) // Dark selection on yellow
    }

    fn disabled(&self, _style: &Self::Style) -> TextInputAppearance {
        TextInputAppearance {
            background: iced::Background::Color(Color::from_rgba(self.primary_color.r, self.primary_color.g, self.primary_color.b, 0.5)),
            border: Border {
                radius: self.radius.into(),
                width: 1.0,
                color: Color::from_rgba(self.primary_color.r, self.primary_color.g, self.primary_color.b, 0.5),
            },
            icon_color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
        }
    }
}

pub struct YellowCheckboxStyle {
    pub radius: f32,
    pub primary_color: Color,
}

impl CheckboxStyleSheet for YellowCheckboxStyle {
    type Style = iced::Theme;

    fn active(&self, _style: &Self::Style, is_checked: bool) -> CheckboxAppearance {
        CheckboxAppearance {
            background: iced::Background::Color(if is_checked {
                self.primary_color // Yellow when checked
            } else {
                Color::from_rgba(0.2, 0.2, 0.2, 1.0) // Dark background when unchecked
            }),
            icon_color: if is_checked {
                Color::BLACK // Black checkmark on yellow
            } else {
                Color::TRANSPARENT
            },
            border: Border {
                radius: self.radius.into(),
                width: 2.0,
                color: self.primary_color, // Yellow border always
            },
            text_color: Some(Color::WHITE),
        }
    }

    fn hovered(&self, style: &Self::Style, is_checked: bool) -> CheckboxAppearance {
        let mut appearance = self.active(style, is_checked);
        appearance.border.color = self.primary_color; // Yellow border on hover
        appearance
    }

    fn disabled(&self, style: &Self::Style, _is_checked: bool) -> CheckboxAppearance {
        let mut appearance = self.active(style, false);
        appearance.background = iced::Background::Color(Color::from_rgba(0.2, 0.2, 0.2, 0.5));
        appearance.border.color = Color::from_rgba(self.primary_color.r, self.primary_color.g, self.primary_color.b, 0.5);
        appearance
    }
}

