//! Build identity helpers.

pub const BASE_VERSION: &str = env!("CARGO_PKG_VERSION");
/// Fork identity shown in CLI and the in-app About overlay.
pub const FORK_OWNER: &str = "dihak";
pub const FORK_REPO: &str = "https://github.com/dihak/herdr";

pub fn channel() -> &'static str {
    non_empty(option_env!("HERDR_BUILD_CHANNEL")).unwrap_or("stable")
}

pub fn build_id() -> Option<&'static str> {
    non_empty(option_env!("HERDR_BUILD_ID"))
}

pub fn version() -> String {
    match channel() {
        "stable" => BASE_VERSION.to_string(),
        channel => match build_id() {
            Some(build_id) => format!("{BASE_VERSION}-{channel}.{build_id}"),
            None => format!("{BASE_VERSION}-{channel}"),
        },
    }
}

pub fn is_preview() -> bool {
    channel() == "preview"
}

fn non_empty(value: Option<&'static str>) -> Option<&'static str> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn stable_version_defaults_to_cargo_version() {
        assert!(!super::version().is_empty());
    }

    #[test]
    fn fork_identity_is_dihak() {
        assert_eq!(super::FORK_OWNER, "dihak");
        assert!(super::FORK_REPO.contains("dihak/herdr"));
    }
}
