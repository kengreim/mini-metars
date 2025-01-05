#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use anyhow::anyhow;
use log::trace;
use regex::Regex;
use semver::Version;
use std::sync::LazyLock;
use tauri::{AppHandle, WebviewWindowBuilder};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons};

pub async fn check_for_updates(app: &AppHandle) -> Result<(), anyhow::Error> {
    static TAG_VERSION_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"release-v(.+)").unwrap());

    let octocrab = octocrab::instance();
    if let Ok(release) = octocrab
        .repos("kengreim", "mini-metars")
        .releases()
        .get_latest()
        .await
    {
        if let Some(Ok(latest_ver)) = TAG_VERSION_REGEX
            .captures(&release.tag_name)
            .map(|c| Version::parse(&c[1]))
        {
            trace!("Found latest version: {latest_ver}");
            if latest_ver > app.package_info().version {
                trace!("Latest version is newer than current version");
                let message = format!("A new version ({latest_ver}) was found. Do you want to open a window to download the installer?");
                let handle = app.clone();
                app.dialog()
                    .message(message)
                    .title("New version")
                    .buttons(MessageDialogButtons::YesNo)
                    .show(move |response| {
                        if response {
                            // Open new window
                            WebviewWindowBuilder::new(
                                &handle,
                                "update",
                                tauri::WebviewUrl::External(release.html_url),
                            )
                            .inner_size(1024.0, 768.0)
                            .build()
                            .unwrap();
                        }
                    });
            }
            Ok(())
        } else {
            Err(anyhow!(
                "Could not parse latest release version from {}",
                &release.tag_name
            ))
        }
    } else {
        Err(anyhow!("Could not fetch latest release from Github"))
    }
}
