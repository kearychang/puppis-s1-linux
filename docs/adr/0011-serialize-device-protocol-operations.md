# Serialize device protocol operations

The application uses one long-lived P1411 connection with exactly one in-flight command because responses carry a command name but no request identifier. Background reads yield to user operations, each settings transaction holds exclusive access from snapshot through verification and recovery, and disconnect discards stale queued reads rather than replaying them, trading throughput for unambiguous correlation and transaction isolation.
