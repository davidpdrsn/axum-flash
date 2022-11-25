# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

# Unreleased

- Update to axum 0.6.0

# 0.6.0-rc.1 (03. September, 2022)

- Remove dependency on tower-cookies and use axum-extra 0.3.0-rc.1 instead ([#7])
- `axum-flash`'s MSRV is now 1.60.0 ([#2])

[#7]: https://github.com/davidpdrsn/axum-flash/pull/7

# 0.5.0 (23. August, 2022)

- Update to tower-cookies 0.7

# 0.4.0 (1. April, 2022)

- Update to axum-core 0.2 and tower-cookies 0.6 ([#3])

[#3]: https://github.com/davidpdrsn/axum-flash/pull/3

# 0.3.0 (10. February, 2022)

- Update `cookie` to 0.16 ([#2])
- Update `tower-cookies` to 0.5 ([#2])
- `axum-flash`'s MSRV is now 1.56.0 ([#2])

[#2]: https://github.com/davidpdrsn/axum-flash/pull/2

# 0.2.0 (03. December, 2021)

- Update to use axum-core 0.1 which requires users to depend on axum 0.4.

# 0.1.1 (21. November, 2021)

- Set cookie for all paths.

# 0.1.0 (07. November, 2021)

- Initial release.
