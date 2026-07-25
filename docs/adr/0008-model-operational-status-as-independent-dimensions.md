# Model operational status as independent dimensions

Operational state is modeled independently across the physical USB link, host internet sharing, Puppis identity/protocol reachability, device configuration health, and client evidence. The dashboard may summarize these dimensions, but there is no single linear readiness state: each operation gates on the dimensions it actually requires so that, for example, working sharing is not hidden by unavailable device telemetry and a healthy USB link is not mistaken for working internet access.
