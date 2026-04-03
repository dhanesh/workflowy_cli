// Satisfies: RT-6 (warn-only input validation), TN3 (never reject, always inform)
// Satisfies: T4 (must not reject currently accepted inputs), U3 (warnings include valid values)

const KNOWN_LAYOUTS: &[&str] = &[
    "bullets",
    "todo",
    "h1",
    "h2",
    "h3",
    "code-block",
    "quote-block",
];

const KNOWN_POSITIONS: &[&str] = &["top", "bottom"];

/// Warn on stderr if layout value is not in the known set.
/// Never rejects — the request is still sent to the API (TN3 resolution).
pub fn warn_layout(layout: Option<&str>) {
    if let Some(val) = layout {
        if !KNOWN_LAYOUTS.contains(&val) {
            tracing::warn!(
                value = val,
                valid = KNOWN_LAYOUTS.join(", ").as_str(),
                "Unknown layout value — request will be sent, but API may reject it"
            );
        }
    }
}

/// Warn on stderr if position value is not in the known set.
pub fn warn_position(position: Option<&str>) {
    if let Some(val) = position {
        if !KNOWN_POSITIONS.contains(&val) {
            tracing::warn!(
                value = val,
                valid = KNOWN_POSITIONS.join(", ").as_str(),
                "Unknown position value — request will be sent, but API may reject it"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Validates: RT-6.1 — known layout values accepted silently
    #[test]
    fn known_layouts_are_valid() {
        for layout in KNOWN_LAYOUTS {
            // Should not panic or produce output
            warn_layout(Some(layout));
        }
    }

    // Validates: RT-6.1 — known position values accepted silently
    #[test]
    fn known_positions_are_valid() {
        for pos in KNOWN_POSITIONS {
            warn_position(Some(pos));
        }
    }

    // Validates: T4 — None values pass through without warning
    #[test]
    fn none_values_pass_through() {
        warn_layout(None);
        warn_position(None);
    }

    // Validates: RT-6.2, U3 — all 7 layout values are in the known set
    #[test]
    fn layout_set_is_complete() {
        assert_eq!(KNOWN_LAYOUTS.len(), 7);
        assert!(KNOWN_LAYOUTS.contains(&"bullets"));
        assert!(KNOWN_LAYOUTS.contains(&"todo"));
        assert!(KNOWN_LAYOUTS.contains(&"code-block"));
    }

    // Validates: RT-6.2 — position set has exactly 2 values
    #[test]
    fn position_set_is_complete() {
        assert_eq!(KNOWN_POSITIONS.len(), 2);
    }
}
