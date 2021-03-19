# OIDCer

A Kratos UI for forwarding immediately to a single configured OIDC provider.

This project uses Rocket, so all Rocket configuration details apply: https://rocket.rs/v0.4/guide/configuration/.

In addition to the standard Rocket configuration, a "registration_endpoint" URL must be configured, which is the Kratos Get Registration Flow API, at `<kratos base url>/self-service/registration/flows`.

The root URL is where registration flows should be directed (and those flows should also be used for OIDC login, instead of the login flow which requires an existing user). Additionally, there is a `/healthz` endpoint for health (it doesn't have much to do, but it's there), and a `/metrics` endpoint for Prometheus-compatible metrics of request counts and latencies.
