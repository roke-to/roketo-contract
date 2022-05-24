# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.0.3] - 2022-05-24
### Added
- locked streams may be updated while not started yet
### Changed
- `tokens_total_withdrawn` logic for cliffs
- `total_incoming` handling in `process_change_receiver`
- stream id generation uniqueness guarantee
- renamed stream methods

## [2.0.2] - 2022-05-24
### Changed
- InsufficientNearDeposit -> InsufficientDeposit

## [2.0.1] - 2022-05-20
### Added
- Transformation of wrapped NEAR into unwrapped ones at Finance contract
### Changed
- Gas needs for withdrawal were increased

## [2.0.0] - 2022-05-16
### Added
- Changelog
### Changed
- Versioning format
- Replaced VERSION.md to CHANGELOG.md in Web4
