# Respect the host's existing upstream routing

Host internet sharing follows the host's effective routing and the application reports, but never selects, reprioritizes, disconnects, or rewrites, upstream connections. This includes VPN routing: v1 cannot promise that Puppis clients use or bypass a VPN, trading per-client route control for preserving host policy and avoiding disruption to unrelated connectivity.
