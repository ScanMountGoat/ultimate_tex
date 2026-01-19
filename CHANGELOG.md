# ultimate_tex Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## 0.3.1 - 2026-01-19
### Fixed
* Fixed menu item text being cut off on some platforms.
* Fixed an issue where some BNTX files would rebuild incorrectly and cause crashes in game.
* Fixed a crash when opening a file dialog for the generated MacOS .app application bundle.
* Fixed a crash when attempting to open invalid image files.
* Fixed a crash when opening nutexb files with invalid mipmap counts.

## Removed
* Removed drag and drop support on Windows due to compatibility issues. This should hopefully be restored in a future release.

## 0.3.0 - 2025-01-24
### Changed
* Adjusted UI to show thumbnail previews for images. 
* Improvements for DDS decoding and encoding.

### Fixed
* Fixed an issue where some BNTX files would not load.

## 0.2.3 - 2023-12-21
### Fixed
* Fixed an issue where images with width and height not divisible by 4 failed to convert to compressed formats.

## 0.2.2 - 2023-09-11
### Fixed
* Fixed an issue where the chosen output format would not be used when exporting.

### Changed
* The footer now displays details of the error if a file fails to convert properly.

## 0.2.1 - 2023-08-25
### Fixed
* Fixed an issue that prevented exporting with mipmaps forced to "Disabled".

## 0.2.0 - 2023-08-25
### Added
* Added an application icon.

### Changed
* Reworked the user interface appearance and layout to for better readability, accessibility, and resizing behavior.
* Adjusted import and export operations to not freeze the UI and better utilize multiple threads.

## 0.1.3 - 2023-04-21
### Fixed
* Fixed an issue where generated BNTX files would not save properly and cause crashes when loaded in game.

## 0.1.2 - 2023-03-24
### Added
* Added an option to set the compression quality for all files.

### Fixed
* Fixed an issue where some pixels near the edges would be incorrectly set as transparent when decoding or changing formats.

## 0.1.1 - 2023-02-23
### Fixed
* Fixed corrupted mipmaps when generating bntx files from generated or existing mipmaps.
* Fixed some formats not working when encoding or decoding bntx files.

## 0.1.0 - 2023-02-04
First public release!
