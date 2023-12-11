# Resolute (Mod Manager for Resonite)

This is a very early WIP project to create a friendly GUI for downloading, updating, and setup of Resonite mods.
The goal is to provide a cross-platform beginning-to-end setup experience for Resonite modding.

## Implemented features

- Manifest handling
  - Downloading
  - Caching
  - Parsing
- Rudimentary mod management
  - Listing
  - Installation
- Automatic app updates

## Planned features / Goals

- Proper mod management
  - Listing (installed and all available)
  - Installation
  - Tracking installed mods
  - Updating (automatic and manual checks)
  - Deletion
  - Detection of externally-installed mods
  - ResoniteModLoader/Harmony alerts (missing or outdated)
- Dependency resolution
  - Automatic installation
  - Conflict warnings
- CLI

## License

This project is licensed under the [GPLv3 license](https://www.gnu.org/licenses/gpl-3.0),
with the exception of the library code located in [crates/resolute](./crates/resolute),
which is licensed under the [LGPLv3 license](https://www.gnu.org/licenses/lgpl-3.0).
