// ------------------------------------------------------------------------------
// Filename: app_configs.rs
// Path: ./src/app_configs
// Original Author: Robert S.A. Nio
// Contributor:
// Date: 2024-07-28
// ------------------------------------------------------------------------------
// Description: This module defines compile-time constants for determining the
//              operating system and target family for the Ultra Fast File Search
//              Tool (UFFS). It provides flags for Unix, Windows, and macOS
//              environments to facilitate platform-specific configurations.
// ------------------------------------------------------------------------------
// Copyright © 2024 SKY, LLC. All Rights Reserved.
//
// This software is the confidential and proprietary information of SKY, LLC..
// ("Confidential Information"). You shall not disclose such Confidential
// Information and shall use it only in accordance with the terms of the license
// agreement you entered into with SKY, LLC..
// ------------------------------------------------------------------------------
// For more information on standards and best practices, refer to
// <CORPORATE DEVELOPMENT GUIDELINES LINK OR RESOURCE>
// ------------------------------------------------------------------------------

pub(crate) mod config {
    // Define a constant for Unix family
    #[cfg(target_family = "unix")]
    pub(crate) const IS_UNIX: bool = true;
    #[cfg(not(target_family = "unix"))]
    pub(crate) const IS_UNIX: bool = false;

    // Define a constant for Windows
    #[cfg(target_os = "windows")]
    pub(crate) const IS_WINDOWS: bool = true;
    #[cfg(not(target_os = "windows"))]
    pub(crate) const IS_WINDOWS: bool = false;

    // Define a constant for macOS
    #[cfg(target_os = "macos")]
    pub(crate) const IS_MACOS: bool = true;
    #[cfg(not(target_os = "macos"))]
    pub(crate) const IS_MACOS: bool = false;
}
