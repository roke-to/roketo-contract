# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
### Changed
- `account_unstake` signature fix

## [2.1.4] - 2022-06-01
### Added
- `available_to_withdraw_by_formula` in stream view

## [2.1.3] - 2022-05-31
### Changed
- updating commission logic for locked streams
- some readability improvements

## [2.1.2] - 2022-05-27
### Added
- view-method `get_streams`
- useful comments about inner details in the contract
- running storage deposit in NFT transfer
- web4 `get_streams`
### Changed
- logic of unlisted replaced with non-payment
- views simplification
- `get_account` view signature
- from and limit are optional for views
- stats calculation for non-payment tokens minor update
- taking `commission_on_transfer` in NFTs

## [2.1.1] - 2022-05-27
### Added
- `streaming_storage_needs_transfer`
### Changed
- `nft_change_receiver` is enabled
- sdk updated to 4.0.0-pre.9

## [2.1.0] - 2022-05-26
### Added
- `approved_nfts` list to Dao

## [2.0.6] - 2022-05-26
### Added
- web4 `get_account` and `get_stream` responses

## [2.0.5] - 2022-05-26
### Changed
- web4 `get_token` response updated

## [2.0.4] - 2022-05-26
### Changed
- UnreachableAccount and UnreachableStream errors replaced with AccountNotExist and StreamNotExists

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
